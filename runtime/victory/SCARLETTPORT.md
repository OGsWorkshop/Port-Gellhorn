# Provenance

This tree is the VIctory-based runtime used by ScarlettPort, vendored as plain source (no build output, no history). It is licensed GPL-3.0-or-later; `LICENSE` is unchanged.

## Lineage

1. **VIctory** by Marvisak and contributors (https://github.com/Marvisak/VIctory), GPL-3.0-or-later. The loader, the platform services, the Direct3D 12 back end and the shader recompiler start here.
2. **fxceless/victory-fork** (https://github.com/fxceless/victory-fork), GPL-3.0. Our base is its `main` at `4e17478`.
3. **VIcky** by Xeltax (https://github.com/Xeltax/VIcky), branch `fx-heapmap-4e`: the heap map for NVIDIA's view-heap cap and the per-program wave64 switch, plus commits by its contributors (including an integer-blend fix and a window-close fix). See `VICKY.md`.
4. **AnyXboxX** (LooperSalty, https://github.com/LooperSalty/victory): the stable stack and notes that our earlier base and this work drew on; its `lot5` branch was checked for overlap and licence (GPL-3.0).
5. **Community pull requests** picked on top, keeping their authors where the commit was applied as is. Numbers are those of the repository they came from (upstream VIctory for #1215; otherwise fxceless/victory-fork):
   - #1215 (upstream): direct writes into title memory, `VirtualQuery` only after a failure
   - #156: ray-tracing work (acceleration-structure reads by slot)
   - #173: ray-traversal fix (patch applied by hand; its author is Bennet0k)
   - #191, #141, #187: submission relay and in-flight waits (#187's relay left opt-in, `VICTORY_RELAY=1|2`)
   - #147: shader cache keyed by the translator's own source
   - #95, #92, #177: texture and memory-lock reworks, banded detiling
   - #47, #45, #42, #48: further fixes from the same group
   - Not taken: upstream #1529 and 6d021fe, pipeline pre-build commits c47b529 / 9c58a1c / 819f622, #175, #186 (conflicting designs).
6. **ScarlettPort changes** on top: Windows 10 Agility SDK opt-in, automatic input (`VICTORY_AUTO_INPUT`), a live press file with stick tokens, capture-every and frame dumps, fast boot (`VICTORY_FAST_BOOT`, testing only) and save slots (`VICTORY_SAVE_SLOT`), `SCARLETT_*` aliases for every `VICTORY_*` switch, a host descriptor-words table (descriptors read as numbers), exposure, ambient and shadow-mask stand-ins (off by default), `VICTORY_SHADER_OPT` and the relay left off by default, a `VICTORY_RAYTRACING=off` answer, and test switches (`VICTORY_TEST_NO_GPU`).

Authors whose commits are in the range `4e17478..fx-merge` (kept in the history of our working repository, not in this snapshot): Xeltax, fever308, iurysf, fxceless, UNYTIX, sedzz, ruvenesh, marquezini, gdawid94, Matheus, Looper_Salty, Last Thylacine, Christian Blomert, Bennet0k, Adrito, Ziggy.

Also credited: XCL by ivall and contributors (https://github.com/ivall/xcl, GPL-2.0-only), used as a reference for how a title behaves under another layer.

## This snapshot

- Source: branch `fx-merge` at `1c3c4ba` of our fork's working repository.
- `patches/victory-stack.patch` (repository root) is the diff of this tree against fxceless/victory-fork `main` at `4e17478` (`git diff 4e17478 fx-merge`). One line of `VICKY.md` (a former repository name) is edited in that copy.
- The base already took the title's name out of its tree; this snapshot keeps it that way.

Build and run: see `docs/BUILDING.md`, `scripts/setup-victory.ps1` and `scripts/play-victory.ps1`.
