//! Original generated artwork for the header card: a dusk gradient, a low sun and a perspective grid.
use eframe::egui::{Color32, ColorImage};

fn lerp(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0., 1.);
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

pub fn header() -> ColorImage {
    let (w, h) = (960usize, 540usize);
    let horizon = h as f32 * 0.62;
    let top = [18., 10., 40.];
    let mid = [96., 24., 90.];
    let low = [255., 96., 84.];
    let ground = [14., 8., 30.];
    let (cx, cy, r) = (w as f32 * 0.62, horizon - 34., 118.);
    let mut pixels = Vec::with_capacity(w * h);
    for y in 0..h {
        for x in 0..w {
            let (fx, fy) = (x as f32, y as f32);
            let mut c = if fy < horizon {
                let t = fy / horizon;
                if t < 0.6 { lerp(top, mid, t / 0.6) } else { lerp(mid, low, (t - 0.6) / 0.4) }
            } else {
                ground
            };
            // Sun disc with horizontal cut lines.
            let d = ((fx - cx).powi(2) + (fy - cy).powi(2)).sqrt();
            if d < r && fy < horizon {
                let t = (fy - (cy - r)) / (2. * r);
                let band = ((fy - cy) / 9.).sin();
                if !(t > 0.55 && band > 0.35) {
                    c = lerp([255., 214., 120.], [255., 70., 110.], t);
                }
            } else if d < r + 26. && fy < horizon {
                c = lerp(c, [255., 120., 100.], 0.18 * (1. - (d - r) / 26.));
            }
            // Perspective grid on the ground.
            if fy >= horizon {
                let depth = (fy - horizon) / (h as f32 - horizon) + 0.001;
                let z = 1. / depth;
                let line_z = ((z * 2.2).fract() < 0.06 / depth.max(0.15)) as u8 as f32;
                let lx = (fx - w as f32 / 2.) / (w as f32 / 2.) * z * 0.6;
                let line_x = ((lx * 6.).fract().abs() < 0.04 * z.min(8.)) as u8 as f32;
                let g = ((line_z.max(line_x)) * (0.55 * depth.sqrt() + 0.1)).clamp(0., 1.);
                c = lerp(c, [90., 200., 255.], g * 0.5);
            }
            pixels.push(Color32::from_rgb(c[0] as u8, c[1] as u8, c[2] as u8));
        }
    }
    ColorImage { size: [w, h], pixels }
}
