use anyhow::Result;
use image::{Rgba, RgbaImage};
use std::path::{Path, PathBuf};

/// Generates 3 synthetic HEAV3NS test artworks in `inbox_dir` so the autonomous workflow
/// can be verified end-to-end immediately:
/// 1. `heav3ns_01_explicit_study.png` (1200x900) — Contains high-frequency anatomical detail lines in a localized focal zone (420..580, 480..620) requiring Japanese mosaic censorship.
/// 2. `heav3ns_02_artistic_silhouette.png` (1000x1000) — Classical artistic nudity silhouette with atmospheric gradients, requiring NO censorship (`artistic_nudity_pass`).
/// 3. `heav3ns_03_biomech_ambiguity.png` (1000x800) — Surreal biomechanical abstract geometry intertwined with central organic motifs, designed to test escalation (`AmbiguousEscalate`).
pub fn generate_demo_inbox_assets(inbox_dir: &Path) -> Result<Vec<PathBuf>> {
    std::fs::create_dir_all(inbox_dir)?;
    let mut created = Vec::new();

    // 1. Explicit anatomical detail test artwork
    let path1 = inbox_dir.join("heav3ns_01_explicit_study.png");
    let mut img1 = RgbaImage::new(1200, 900);
    for y in 0..900 {
        for x in 0..1200 {
            let r = ((x as f32 / 1200.0) * 60.0 + 25.0) as u8;
            let g = ((y as f32 / 900.0) * 45.0 + 20.0) as u8;
            let b = 65u8;
            img1.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
    // Draw stylized figure torso & limbs
    for y in 200..780 {
        for x in 350..650 {
            let dx = (x as i32 - 500).abs();
            if dx < 130 {
                img1.put_pixel(x, y, Rgba([232, 190, 172, 255]));
            }
        }
    }
    // Draw high-frequency explicit anatomical line-work inside localized zone (440..560, 500..600)
    // and secondary detail zone (460..540, 320..370)
    for y in 500..600 {
        for x in 440..560 {
            let fine_pattern = ((x * 7 + y * 13) % 2 == 0) || ((x as i32 - 500).abs() < 3);
            if fine_pattern {
                img1.put_pixel(x, y, Rgba([145, 45, 65, 255]));
            } else {
                img1.put_pixel(x, y, Rgba([245, 185, 180, 255]));
            }
        }
    }
    for y in 320..360 {
        for x in 450..550 {
            if (y - 340i32 as u32) < 4 || (x + y) % 3 == 0 {
                img1.put_pixel(x, y, Rgba([160, 60, 75, 255]));
            }
        }
    }
    img1.save(&path1)?;
    created.push(path1);

    // 2. Artistic nudity classical silhouette (no explicit genitalia)
    let path2 = inbox_dir.join("heav3ns_02_artistic_silhouette.png");
    let mut img2 = RgbaImage::new(1000, 1000);
    for y in 0..1000 {
        for x in 0..1000 {
            let cx = x as f32 - 500.0;
            let cy = y as f32 - 500.0;
            let dist = (cx * cx + cy * cy).sqrt();
            if dist < 260.0 {
                let shade = (200.0 - (dist / 260.0) * 90.0) as u8;
                img2.put_pixel(x, y, Rgba([shade, shade.saturating_sub(20), shade, 255]));
            } else {
                img2.put_pixel(x, y, Rgba([18, 22, 34, 255]));
            }
        }
    }
    img2.save(&path2)?;
    created.push(path2);

    // 3. Ambiguous biomechanical composition requiring escalation
    let path3 = inbox_dir.join("heav3ns_03_biomech_ambiguity.png");
    let mut img3 = RgbaImage::new(1000, 800);
    for y in 0..800 {
        for x in 0..1000 {
            let v = (((x ^ y) & 0xFF) as u8) / 3 + 30;
            img3.put_pixel(x, y, Rgba([v, v / 2, v.saturating_add(40), 255]));
        }
    }
    img3.save(&path3)?;
    created.push(path3);

    Ok(created)
}

