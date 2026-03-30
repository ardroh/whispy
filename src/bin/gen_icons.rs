//! Dev tool: writes `assets/tray-*.png` and `assets/AppIcon.icns`.
//! Run from repo root: `cargo run --bin gen_icons`

use image::{Rgba, RgbaImage};
use std::path::Path;
use std::process::Command;

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let assets = root.join("assets");
    std::fs::create_dir_all(&assets).expect("create assets");

    let tray_idle = render_tray_idle(44);
    tray_idle
        .save(assets.join("tray-idle.png"))
        .expect("write tray-idle");

    let tray_rec = render_tray_recording(44);
    tray_rec
        .save(assets.join("tray-recording.png"))
        .expect("write tray-recording");

    let app = render_app_icon(1024);
    let src1024 = assets.join("app-icon-source-1024.png");
    app.save(&src1024).expect("write app 1024");

    if cfg!(target_os = "macos") {
        build_icns(&assets, &src1024);
    } else {
        eprintln!("Skipping .icns (macOS only). On a Mac, run this binary again to produce AppIcon.icns");
    }

    eprintln!("Wrote {}", assets.display());
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn mix(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Template-style tray glyph: bold "W" + EQ bars (black + alpha; macOS tints it).
fn render_tray_idle(size: u32) -> RgbaImage {
    let mut img = RgbaImage::from_pixel(size, size, Rgba([0, 0, 0, 0]));
    let s = size as f32;
    let cx = s * 0.5;
    let cy = s * 0.52;
    let scale = s / 44.0;
    const STROKE: f32 = 1.65;
    const BAR_R: f32 = 1.05;

    for y in 0..size {
        for x in 0..size {
            let px = (x as f32 + 0.5 - cx) / scale;
            let py = (y as f32 + 0.5 - cy) / scale;

            let l1 = thick_line(px, py, -15.0, -11.0, -8.0, 9.0, STROKE);
            let l2 = thick_line(px, py, -8.0, 9.0, 0.0, -3.0, STROKE);
            let l3 = thick_line(px, py, 0.0, -3.0, 8.0, 9.0, STROKE);
            let l4 = thick_line(px, py, 8.0, 9.0, 15.0, -11.0, STROKE);
            let w_stroke = smooth_complement(l1)
                .max(smooth_complement(l2))
                .max(smooth_complement(l3))
                .max(smooth_complement(l4));

            let b1 = sdf_capsule(px, py, -3.8, 6.2, -3.8, 9.3, BAR_R);
            let b2 = sdf_capsule(px, py, 0.0, 2.0, 0.0, 9.3, BAR_R);
            let b3 = sdf_capsule(px, py, 3.8, 6.2, 3.8, 9.3, BAR_R);
            let bars = smooth_complement(b1)
                .max(smooth_complement(b2))
                .max(smooth_complement(b3));

            let a = w_stroke.max(bars).clamp(0.0, 1.0);
            if a > 0.001 {
                let edge = smoothstep(0.0, 1.0, a);
                img.put_pixel(x, y, Rgba([0, 0, 0, (edge * 235.0) as u8]));
            }
        }
    }
    img
}

fn sdf_circle(x: f32, y: f32, cx: f32, cy: f32, r: f32) -> f32 {
    let dx = x - cx;
    let dy = y - cy;
    (dx * dx + dy * dy).sqrt() - r
}

fn sdf_capsule(x: f32, y: f32, ax: f32, ay: f32, bx: f32, by: f32, r: f32) -> f32 {
    let pa = (x - ax, y - ay);
    let ba = (bx - ax, by - ay);
    let len2 = ba.0 * ba.0 + ba.1 * ba.1;
    let h = ((pa.0 * ba.0 + pa.1 * ba.1) / len2).clamp(0.0, 1.0);
    let dx = pa.0 - ba.0 * h;
    let dy = pa.1 - ba.1 * h;
    (dx * dx + dy * dy).sqrt() - r
}

fn sdf_line(px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let vx = bx - ax;
    let vy = by - ay;
    let wx = px - ax;
    let wy = py - ay;
    let c2 = vx * vx + vy * vy;
    let t = if c2 < 1e-6 {
        0.0
    } else {
        (wx * vx + wy * vy) / c2
    };
    let t = t.clamp(0.0, 1.0);
    let dx = px - (ax + t * vx);
    let dy = py - (ay + t * vy);
    (dx * dx + dy * dy).sqrt()
}

fn thick_line(px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32, half: f32) -> f32 {
    sdf_line(px, py, ax, ay, bx, by) - half
}

fn smooth_complement(sdf: f32) -> f32 {
    1.0 - smoothstep(-1.1, 1.1, sdf)
}

fn render_tray_recording(size: u32) -> RgbaImage {
    let mut img = RgbaImage::from_pixel(size, size, Rgba([0, 0, 0, 0]));
    let s = size as f32;
    let cx = s * 0.5;
    let cy = s * 0.5;
    let r_outer = s * 0.38;
    let r_inner = s * 0.28;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            let d = (dx * dx + dy * dy).sqrt();

            let ring = (d - r_outer).abs() - 1.6;
            // Soft white ring
            if ring < 2.0 {
                let k = 1.0 - smoothstep(-1.0, 1.2, ring);
                let c = mix(0.94, 1.0, k);
                img.put_pixel(x, y, Rgba([(c * 255.0) as u8, (c * 255.0) as u8, (c * 255.0) as u8, (k * 200.0) as u8]));
                continue;
            }
            // Red core
            let a = 1.0 - smoothstep(r_inner - 0.8, r_inner + 1.0, d);
            if a > 0.0 {
                let mut r = 234u8;
                let mut g = 67u8;
                let mut b = 53u8;
                let gloss = (1.0 - d / r_inner).max(0.0);
                r = ((r as f32 + gloss * 40.0).min(255.0)) as u8;
                g = ((g as f32 + gloss * 30.0).min(255.0)) as u8;
                b = ((b as f32 + gloss * 20.0).min(255.0)) as u8;
                img.put_pixel(x, y, Rgba([r, g, b, (a * 255.0) as u8]));
            }
        }
    }
    img
}

fn render_app_icon(size: u32) -> RgbaImage {
    // Opaque black outside the rounded tile (avoids transparent “halo” in Finder / Spotlight).
    let mut img = RgbaImage::from_pixel(size, size, Rgba([0, 0, 0, 255]));
    let s = size as f32;
    let pad = s * 0.18;
    let r_corner = s * 0.22;

    for y in 0..size {
        for x in 0..size {
            let u = x as f32 / (s - 1.0);
            let v = y as f32 / (s - 1.0);
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            // Rounded-rect mask
            let ix = px - pad;
            let iy = py - pad;
            let w = s - 2.0 * pad;
            let h = s - 2.0 * pad;
            let qx = ix - w * 0.5;
            let qy = iy - h * 0.5;
            let bx = w * 0.5 - r_corner;
            let by = h * 0.5 - r_corner;
            let ax = qx.abs();
            let ay = qy.abs();
            let ox = ax - bx;
            let oy = ay - by;
            let d = if ox > 0.0 && oy > 0.0 {
                (ox * ox + oy * oy).sqrt()
            } else {
                ox.max(oy)
            } - r_corner;

            let mask = 1.0 - smoothstep(-1.5, 1.5, d);
            if mask < 0.01 {
                continue;
            }

            let r_top = mix(58.0, 22.0, u * 0.6 + v * 0.4);
            let g_top = mix(32.0, 120.0, u * 0.5 + v * 0.5);
            let b_top = mix(92.0, 130.0, (1.0 - u) * 0.4 + v * 0.6);
            let highlight =
                ((1.35 - ((px - s * 0.35).hypot(py - s * 0.32) / (s * 0.55))).max(0.0)).powf(1.8);
            let rr = ((r_top + highlight * 55.0).min(255.0)) as u8;
            let gg = ((g_top + highlight * 40.0).min(255.0)) as u8;
            let bb = ((b_top + highlight * 35.0).min(255.0)) as u8;

            let gx = (px - s * 0.5) / (s * 0.34);
            let gy = (py - s * 0.5) / (s * 0.34);
            let logo = app_logo_sdf(gx, gy);
            let logo_a = (1.0 - smoothstep(-0.85, 0.95, logo)).clamp(0.0, 1.0);

            let lr = 248u32;
            let lg = 250u32;
            let lb = 252u32;
            let ir = mix(rr as f32, lr as f32, logo_a);
            let ig = mix(gg as f32, lg as f32, logo_a);
            let ib = mix(bb as f32, lb as f32, logo_a);
            let fr = (ir * mask).clamp(0.0, 255.0) as u8;
            let fg = (ig * mask).clamp(0.0, 255.0) as u8;
            let fb = (ib * mask).clamp(0.0, 255.0) as u8;
            img.put_pixel(x, y, Rgba([fr, fg, fb, 255]));
        }
    }
    img
}

fn app_logo_sdf(x: f32, y: f32) -> f32 {
    let wave1 = (x * 6.5 + 1.2).sin() * 1.1 - y * 2.8 - 0.3;
    let wave2 = (x * 6.5 - 0.4).sin() * 1.1 - y * 2.8 + 1.1;
    let bands = thick_band(wave1, 0.22).min(thick_band(wave2, 0.22));

    let mic_head = sdf_circle(x, y, -0.08, -0.42, 0.48);
    let mic_body = sdf_capsule(x, y, -0.08, -0.12, -0.08, 0.92, 0.36);
    let mic = mic_head.min(mic_body);

    bands.min(mic)
}

fn thick_band(field: f32, half: f32) -> f32 {
    field.abs() - half
}

fn build_icns(assets: &Path, src1024: &Path) {
    let set_dir = assets.join("Whispy.iconset");
    let _ = std::fs::remove_dir_all(&set_dir);
    std::fs::create_dir_all(&set_dir).expect("iconset");

    let pairs: [(u32, &str); 10] = [
        (16, "icon_16x16.png"),
        (32, "icon_16x16@2x.png"),
        (32, "icon_32x32.png"),
        (64, "icon_32x32@2x.png"),
        (128, "icon_128x128.png"),
        (256, "icon_128x128@2x.png"),
        (256, "icon_256x256.png"),
        (512, "icon_256x256@2x.png"),
        (512, "icon_512x512.png"),
        (1024, "icon_512x512@2x.png"),
    ];

    for (dim, name) in pairs {
        let out = set_dir.join(name);
        let status = Command::new("sips")
            .args(["-z", &dim.to_string(), &dim.to_string()])
            .arg(src1024)
            .arg("--out")
            .arg(&out)
            .status();
        if status.is_err() || !status.unwrap().success() {
            eprintln!("sips failed for {} — is this macOS?", name);
            return;
        }
    }

    let icns_out = assets.join("AppIcon.icns");
    // iconutil requires the output path to end in `.icns` (not `.part`).
    let icns_tmp = std::env::temp_dir().join("whispy-gen-AppIcon.icns");
    let _ = std::fs::remove_file(&icns_tmp);
    let st = Command::new("iconutil")
        .args(["-c", "icns", "-o"])
        .arg(&icns_tmp)
        .arg(&set_dir)
        .status()
        .expect("iconutil");
    if !st.success() {
        eprintln!("iconutil failed (left existing AppIcon.icns untouched)");
        let _ = std::fs::remove_file(&icns_tmp);
        return;
    }
    if let Err(e) = std::fs::rename(&icns_tmp, &icns_out) {
        eprintln!("rename AppIcon.icns: {e}");
        let _ = std::fs::remove_file(&icns_tmp);
        return;
    }
    let _ = std::fs::remove_dir_all(&set_dir);
    eprintln!("Wrote {}", icns_out.display());
}
