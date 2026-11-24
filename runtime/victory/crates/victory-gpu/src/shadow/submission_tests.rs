use super::*;

#[test]
fn repeated_gpu_writes_keep_their_first_copy_back_order() {
    let Some(host) = host() else { return };
    let mut title = [0u8; 3];
    let start = title.as_mut_ptr() as usize;
    let make_shadow = || {
        let len = title.len() as u64;
        let (buffer, data) = host.shared_buffer(len).unwrap();
        // These lists have no GPU commands. The deferred work below writes the mapped bytes.
        Arc::new(Shadow::new(buffer, data, 0, start as u64, len))
    };
    let a = make_shadow();
    let b = make_shadow();
    let first = host.new_list().unwrap();
    first.defer_refresh(&a, start, 3);
    first.defer_refresh(&b, start, 3);
    first.defer_gpu_write(&a, start, 2);
    first.defer_gpu_write(&a, start, 2);
    first.defer_gpu_write(&a, start, 1);
    first.close();

    let second = host.new_list().unwrap();
    second.defer_gpu_write(&b, start, 2);
    second.defer(move || {
        // SAFETY: the title array lives through both synchronous submissions.
        unsafe { (start as *mut u8).add(2).write(33) };
    });
    second.defer_refresh(&a, start + 2, 1);
    second.defer_gpu_write(&a, start, 2);
    second.defer_host(move || {
        assert_eq!(
            *a.pending.lock().unwrap(),
            [(start, start + 2), (start, start + 1)]
        );
        assert_eq!(*b.pending.lock().unwrap(), [(start, start + 2)]);
        // SAFETY: both shadows own three mapped bytes, used by this submitting thread.
        unsafe {
            assert_eq!(
                a.data.add(2).read(),
                33,
                "a duplicate must still flush refreshes"
            );
            a.data.write(a.data.read() + 11);
            b.data.write(b.data.read() + 22);
        }
    });
    second.close();

    for expected in [22, 44] {
        host.execute(&[&first, &second]);
        assert_eq!(
            title,
            [expected, 0, 33],
            "the second shadow writes back last"
        );
    }
}

/// A range brought up to date is not asked again in the same submission
/// until a step that may write title memory has run: lists of a submission
/// read the same big heap through many ranges.
#[test]
fn a_range_is_refreshed_once_until_the_title_may_have_written_it() {
    let Some(host) = host() else { return };
    let mut title = [1u8, 2, 3, 4];
    let start = title.as_mut_ptr() as usize;
    let (buffer, data) = host.shared_buffer(4).unwrap();
    let shadow = Arc::new(Shadow::new(buffer, data, 0, start as u64, 4));
    let (other_buffer, other_data) = host.shared_buffer(4).unwrap();
    let other = Arc::new(Shadow::new(other_buffer, other_data, 0, start as u64, 4));
    let list = host.new_list().unwrap();
    // Refreshed, and flushed by the next shader-write declaration.
    list.defer_refresh(&shadow, start, 4);
    list.defer_gpu_write(&other, start, 1);
    // The title changes without the batch knowing (a host step): the
    // second refresh of the same range is skipped.
    list.defer_host(move || unsafe { (start as *mut u8).add(1).write(20) });
    list.defer_refresh(&shadow, start, 3);
    list.defer_gpu_write(&other, start + 1, 1);
    let seen = shadow.clone();
    list.defer_host(move || {
        // SAFETY: four mapped bytes, used by this thread.
        assert_eq!(unsafe { seen.data.add(1).read() }, 2, "skipped as fresh");
    });
    // A step writing title memory makes it due again: the copy follows.
    list.defer(move || unsafe { (start as *mut u8).add(2).write(30) });
    list.defer_refresh(&shadow, start, 4);
    list.defer_gpu_write(&other, start + 2, 1);
    let seen = shadow.clone();
    list.defer_host(move || {
        // SAFETY: as above.
        unsafe {
            assert_eq!(seen.data.add(2).read(), 30, "refreshed after a title write");
            // The earlier host write is picked up with it (whole range).
            assert_eq!(seen.data.add(1).read(), 20);
        }
    });
    list.close();
    host.execute(&[&list]);
}
