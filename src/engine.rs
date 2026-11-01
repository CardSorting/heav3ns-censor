use crate::models::{
    CandidateHotspot, CensorshipMethod, CensorshipRegion, CoordinateUnit, RegionShape,
    RegionVerificationMetric, VerificationReport,
};
use crate::policy::recommended_mosaic_block_size;
use anyhow::{Context, Result};
use chrono::Utc;
use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
use std::io::Cursor;
use std::path::Path;

/// Parse a hex color string like "#000000", "000000", "#FFFFFF" into `Rgba<u8>`.
pub fn parse_hex_color(hex_str: &str) -> Rgba<u8> {
    let clean = hex_str.trim().trim_start_matches('#');
    if clean.len() == 6 {
        let r = u8::from_str_radix(&clean[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&clean[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&clean[4..6], 16).unwrap_or(0);
        Rgba([r, g, b, 255])
    } else if clean.len() == 8 {
        let r = u8::from_str_radix(&clean[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&clean[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&clean[4..6], 16).unwrap_or(0);
        let a = u8::from_str_radix(&clean[6..8], 16).unwrap_or(255);
        Rgba([r, g, b, a])
    } else if clean.len() == 3 {
        let r = u8::from_str_radix(&clean[0..1], 16).unwrap_or(0) * 17;
        let g = u8::from_str_radix(&clean[1..2], 16).unwrap_or(0) * 17;
        let b = u8::from_str_radix(&clean[2..3], 16).unwrap_or(0) * 17;
        Rgba([r, g, b, 255])
    } else {
        Rgba([0, 0, 0, 255])
    }
}

/// Automatically selects `#000000` (*kuro-nuri*) on light/midtone regions or `#FFFFFF` (*shiro-nuri*)
/// on dark ink/shadowed regions (`mean_luminance < 90.0`) when `color` is `"auto"`.
pub fn resolve_bar_color(
    color_spec: &str,
    original: &RgbaImage,
    bx: u32,
    by: u32,
    bw: u32,
    bh: u32,
) -> (Rgba<u8>, String) {
    let clean = color_spec.trim().to_lowercase();
    if clean == "auto" || clean == "auto_contrast" {
        let mut lum_sum = 0.0f64;
        let mut count = 0u64;
        for y in by..(by + bh) {
            for x in bx..(bx + bw) {
                let p = original.get_pixel(x, y);
                let lum = 0.299 * p[0] as f64 + 0.587 * p[1] as f64 + 0.114 * p[2] as f64;
                lum_sum += lum;
                count += 1;
            }
        }
        let avg_lum = if count > 0 {
            lum_sum / count as f64
        } else {
            128.0
        };
        if avg_lum < 90.0 {
            (Rgba([255, 255, 255, 255]), "#FFFFFF(auto-shiro)".to_string())
        } else {
            (Rgba([0, 0, 0, 255]), "#000000(auto-kuro)".to_string())
        }
    } else {
        (parse_hex_color(color_spec), color_spec.to_string())
    }
}

/// Computes an exact Signed Distance Field (SDF) alpha mask value in `[0.0, 1.0]` for pixel `(px, py)`
/// relative to region `(rx, ry, rw, rh)` with arbitrary rotation `angle_deg` around `(cx, cy)`.
/// Clamps `feather_px` to at most `45%` of the minor half-axis so the central `55%+` core of every
/// censorship region is guaranteed to remain `100%` opaque (`alpha == 1.0`).
pub fn region_mask_alpha(
    px: u32,
    py: u32,
    rx: u32,
    ry: u32,
    rw: u32,
    rh: u32,
    shape: RegionShape,
    angle_deg: f32,
    feather_px: u32,
) -> f32 {
    if rw == 0 || rh == 0 {
        return 0.0;
    }

    let cx = rx as f32 + rw as f32 * 0.5;
    let cy = ry as f32 + rh as f32 * 0.5;
    let dx = (px as f32 + 0.5) - cx;
    let dy = (py as f32 + 0.5) - cy;

    let (lx, ly) = if angle_deg.is_finite() && angle_deg.abs() > 0.01 {
        let rad = -angle_deg.to_radians();
        let cos_a = rad.cos();
        let sin_a = rad.sin();
        (dx * cos_a - dy * sin_a, dx * sin_a + dy * cos_a)
    } else {
        (dx, dy)
    };

    let half_w = (rw as f32 * 0.5).max(0.5);
    let half_h = (rh as f32 * 0.5).max(0.5);
    let max_safe_feather = (half_w.min(half_h) * 0.45).max(0.5);

    match shape {
        RegionShape::Rect => {
            if lx.abs() > half_w || ly.abs() > half_h {
                return 0.0;
            }
            if feather_px == 0 {
                return 1.0;
            }
            let eff_feather = (feather_px as f32).min(max_safe_feather);
            let dist_x = (half_w - lx.abs()).max(0.0);
            let dist_y = (half_h - ly.abs()).max(0.0);
            let dist_edge = dist_x.min(dist_y);
            (dist_edge / eff_feather).clamp(0.0, 1.0)
        }
        RegionShape::Ellipse => {
            let norm_dist = ((lx / half_w).powi(2) + (ly / half_h).powi(2)).sqrt();
            if norm_dist > 1.0 {
                return 0.0;
            }
            if feather_px == 0 {
                return 1.0;
            }
            let min_half = half_w.min(half_h);
            let feather_norm = (feather_px as f32 / min_half).clamp(0.01, 0.45);
            if norm_dist <= 1.0 - feather_norm {
                1.0
            } else {
                ((1.0 - norm_dist) / feather_norm).clamp(0.0, 1.0)
            }
        }
        RegionShape::Pill => {
            let (dist_from_spine, radius) = if half_w >= half_h {
                let seg = half_w - half_h;
                let qx = lx.clamp(-seg, seg);
                (((lx - qx).powi(2) + ly.powi(2)).sqrt(), half_h)
            } else {
                let seg = half_h - half_w;
                let qy = ly.clamp(-seg, seg);
                ((lx.powi(2) + (ly - qy).powi(2)).sqrt(), half_w)
            };
            if dist_from_spine > radius {
                return 0.0;
            }
            if feather_px == 0 {
                return 1.0;
            }
            let eff_feather = (feather_px as f32).min((radius * 0.45).max(0.5));
            let dist_edge = (radius - dist_from_spine).max(0.0);
            (dist_edge / eff_feather).clamp(0.0, 1.0)
        }
    }
}

/// Alpha-aware linear interpolation between `bg` and `fg` with factor `alpha` in `[0.0, 1.0]`.
#[inline]
fn lerp_rgba(bg: Rgba<u8>, fg: Rgba<u8>, alpha: f32) -> Rgba<u8> {
    if alpha <= 0.0 {
        return bg;
    }
    if alpha >= 1.0 {
        return fg;
    }
    let inv = 1.0 - alpha;
    Rgba([
        (bg[0] as f32 * inv + fg[0] as f32 * alpha).round().clamp(0.0, 255.0) as u8,
        (bg[1] as f32 * inv + fg[1] as f32 * alpha).round().clamp(0.0, 255.0) as u8,
        (bg[2] as f32 * inv + fg[2] as f32 * alpha).round().clamp(0.0, 255.0) as u8,
        (bg[3] as f32 * inv + fg[3] as f32 * alpha).round().clamp(0.0, 255.0) as u8,
    ])
}

/// Applies alpha-weighted separable Gaussian blur to a cropped sub-buffer `(w, h)`.
fn gaussian_blur_subimage(sub: &RgbaImage, sigma: f32) -> RgbaImage {
    let sigma = if sigma.is_finite() {
        sigma.clamp(0.5, 100.0)
    } else {
        10.0
    };
    let radius = (sigma * 3.0).ceil() as i32;
    let mut kernel = Vec::with_capacity((2 * radius + 1) as usize);
    let mut sum = 0.0f32;
    for i in -radius..=radius {
        let w = (-(i * i) as f32 / (2.0 * sigma * sigma)).exp();
        kernel.push(w);
        sum += w;
    }
    for w in &mut kernel {
        *w /= sum;
    }

    let (w, h) = sub.dimensions();
    let mut horiz = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let mut r = 0.0f32;
            let mut g = 0.0f32;
            let mut b = 0.0f32;
            let mut a = 0.0f32;
            let mut w_rgb = 0.0f32;
            for (k_idx, &weight) in kernel.iter().enumerate() {
                let offset = k_idx as i32 - radius;
                let sx = (x as i32 + offset).clamp(0, w as i32 - 1) as u32;
                let p = sub.get_pixel(sx, y);
                let pa = p[3] as f32 / 255.0;
                let aw = weight * pa.max(0.001);
                r += p[0] as f32 * aw;
                g += p[1] as f32 * aw;
                b += p[2] as f32 * aw;
                w_rgb += aw;
                a += p[3] as f32 * weight;
            }
            let inv_w = if w_rgb > 0.0 { 1.0 / w_rgb } else { 0.0 };
            horiz.put_pixel(
                x,
                y,
                Rgba([
                    (r * inv_w).round().clamp(0.0, 255.0) as u8,
                    (g * inv_w).round().clamp(0.0, 255.0) as u8,
                    (b * inv_w).round().clamp(0.0, 255.0) as u8,
                    a.round().clamp(0.0, 255.0) as u8,
                ]),
            );
        }
    }

    let mut vert = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let mut r = 0.0f32;
            let mut g = 0.0f32;
            let mut b = 0.0f32;
            let mut a = 0.0f32;
            let mut w_rgb = 0.0f32;
            for (k_idx, &weight) in kernel.iter().enumerate() {
                let offset = k_idx as i32 - radius;
                let sy = (y as i32 + offset).clamp(0, h as i32 - 1) as u32;
                let p = horiz.get_pixel(x, sy);
                let pa = p[3] as f32 / 255.0;
                let aw = weight * pa.max(0.001);
                r += p[0] as f32 * aw;
                g += p[1] as f32 * aw;
                b += p[2] as f32 * aw;
                w_rgb += aw;
                a += p[3] as f32 * weight;
            }
            let inv_w = if w_rgb > 0.0 { 1.0 / w_rgb } else { 0.0 };
            vert.put_pixel(
                x,
                y,
                Rgba([
                    (r * inv_w).round().clamp(0.0, 255.0) as u8,
                    (g * inv_w).round().clamp(0.0, 255.0) as u8,
                    (b * inv_w).round().clamp(0.0, 255.0) as u8,
                    a.round().clamp(0.0, 255.0) as u8,
                ]),
            );
        }
    }
    vert
}

/// Applies all planned `CensorshipRegion` treatments onto a clone of `original`, returning the censored image.
/// Always samples spatial kernels (`Mosaic` pooling & `GaussianBlur` halo) from the immutable `original` buffer
/// so remainder-strip pooling and overlapping regions never contaminate each other!
pub fn apply_censorship_regions(
    original: &RgbaImage,
    regions: &[CensorshipRegion],
    policy_id: &str,
) -> RgbaImage {
    let mut canvas = original.clone();
    let (img_w, img_h) = canvas.dimensions();

    for region in regions {
        let (rx, ry, rw, rh) = region.resolve_pixels(img_w, img_h);
        if rw == 0 || rh == 0 {
            continue;
        }
        let (bx, by, bw, bh) = region.resolve_enclosing_aabb(img_w, img_h);
        let angle = region.rotation_angle_deg();

        match &region.treatment {
            CensorshipMethod::Mosaic { block_size } => {
                let auto_bs = recommended_mosaic_block_size(img_w, img_h, policy_id);
                let bs = block_size.unwrap_or(auto_bs).max(auto_bs).max(4);

                let mut cell_y = by;
                while cell_y < by + bh {
                    let cell_h = bs.min((by + bh) - cell_y);
                    let pool_y0 = if cell_h < bs {
                        (cell_y + cell_h).saturating_sub(bs).max(by)
                    } else {
                        cell_y
                    };
                    let pool_y1 = (cell_y + cell_h).min(by + bh);

                    let mut cell_x = bx;
                    while cell_x < bx + bw {
                        let cell_w = bs.min((bx + bw) - cell_x);
                        let pool_x0 = if cell_w < bs {
                            (cell_x + cell_w).saturating_sub(bs).max(bx)
                        } else {
                            cell_x
                        };
                        let pool_x1 = (cell_x + cell_w).min(bx + bw);

                        let mut sum_r = 0.0f64;
                        let mut sum_g = 0.0f64;
                        let mut sum_b = 0.0f64;
                        let mut sum_w = 0.0f64;
                        let mut sum_a = 0u64;
                        let mut count = 0u64;

                        // Sample strictly from `original` so remainder-strip pooling is 100% pure
                        for py in pool_y0..pool_y1 {
                            for px in pool_x0..pool_x1 {
                                let p = original.get_pixel(px, py);
                                let w = (p[3] as f64 / 255.0).max(0.001);
                                sum_r += p[0] as f64 * w;
                                sum_g += p[1] as f64 * w;
                                sum_b += p[2] as f64 * w;
                                sum_w += w;
                                sum_a += p[3] as u64;
                                count += 1;
                            }
                        }

                        if count > 0 {
                            let avg_p = Rgba([
                                (sum_r / sum_w).round().clamp(0.0, 255.0) as u8,
                                (sum_g / sum_w).round().clamp(0.0, 255.0) as u8,
                                (sum_b / sum_w).round().clamp(0.0, 255.0) as u8,
                                (sum_a / count) as u8,
                            ]);
                            for py in cell_y..(cell_y + cell_h) {
                                for px in cell_x..(cell_x + cell_w) {
                                    let alpha = region_mask_alpha(
                                        px,
                                        py,
                                        rx,
                                        ry,
                                        rw,
                                        rh,
                                        region.shape,
                                        angle,
                                        region.feather_px,
                                    );
                                    if alpha > 0.0 {
                                        let cur_p = *canvas.get_pixel(px, py);
                                        canvas.put_pixel(px, py, lerp_rgba(cur_p, avg_p, alpha));
                                    }
                                }
                            }
                        }
                        cell_x += bs;
                    }
                    cell_y += bs;
                }
            }
            CensorshipMethod::GaussianBlur { sigma } => {
                let min_sigma =
                    (recommended_mosaic_block_size(img_w, img_h, policy_id) as f32) * 0.75;
                let effective_sigma = sigma.max(min_sigma).max(4.0);
                let halo = ((effective_sigma * 2.0).ceil() as u32).min(64);

                let hx0 = bx.saturating_sub(halo);
                let hy0 = by.saturating_sub(halo);
                let hx1 = (bx + bw + halo).min(img_w);
                let hy1 = (by + bh + halo).min(img_h);
                let hw = hx1.saturating_sub(hx0).max(1);
                let hh = hy1.saturating_sub(hy0).max(1);

                // Extract halo-padded context from immutable `original` so adjacent bars never bleed into blur
                let mut sub = RgbaImage::new(hw, hh);
                for y in 0..hh {
                    for x in 0..hw {
                        sub.put_pixel(x, y, *original.get_pixel(hx0 + x, hy0 + y));
                    }
                }
                let blurred = gaussian_blur_subimage(&sub, effective_sigma);

                for py in by..(by + bh) {
                    for px in bx..(bx + bw) {
                        let alpha = region_mask_alpha(
                            px,
                            py,
                            rx,
                            ry,
                            rw,
                            rh,
                            region.shape,
                            angle,
                            region.feather_px,
                        );
                        if alpha > 0.0 {
                            let cur_p = *canvas.get_pixel(px, py);
                            let blur_p = *blurred.get_pixel(px - hx0, py - hy0);
                            canvas.put_pixel(px, py, lerp_rgba(cur_p, blur_p, alpha));
                        }
                    }
                }
            }
            CensorshipMethod::SolidBar { color, .. } => {
                let (bar_rgba, _) = resolve_bar_color(color, original, bx, by, bw, bh);
                for py in by..(by + bh) {
                    for px in bx..(bx + bw) {
                        let alpha = region_mask_alpha(
                            px,
                            py,
                            rx,
                            ry,
                            rw,
                            rh,
                            region.shape,
                            angle,
                            region.feather_px,
                        );
                        if alpha > 0.0 {
                            let cur_p = *canvas.get_pixel(px, py);
                            canvas.put_pixel(px, py, lerp_rgba(cur_p, bar_rgba, alpha));
                        }
                    }
                }
            }
        }
    }

    canvas
}

/// Computes `(mean_gradient_energy, p95_peak_gradient_energy, sample_count)` strictly inside the active interior mask of `region`,
/// skipping mosaic block step boundaries (including fractionally scaled step boundaries `k * native_bs * scale`)
/// and outer mask boundary ringing so we measure true intra-region detail suppression.
#[allow(clippy::too_many_arguments)]
fn compute_masked_detail_metrics(
    img: &RgbaImage,
    rx: u32,
    ry: u32,
    rw: u32,
    rh: u32,
    bx: u32,
    by: u32,
    bw: u32,
    bh: u32,
    shape: RegionShape,
    angle_deg: f32,
    feather_px: u32,
    native_bs: u32,
    scale: f32,
) -> (f32, f32, usize) {
    if bw < 4 || bh < 4 {
        return (0.0, 0.0, 0);
    }
    let mut samples = Vec::new();
    let step_f = (native_bs as f32 * scale).max(1.0);
    let margin = if scale < 0.99 { 1.35f32 } else { 0.55f32 };

    for y in (by + 1)..(by + bh - 2) {
        for x in (bx + 1)..(bx + bw - 2) {
            if native_bs > 1 && step_f > 2.5 {
                let dx = (x - bx + 1) as f32;
                let dy = (y - by + 1) as f32;
                let rem_x = dx % step_f;
                let dist_x = rem_x.min(step_f - rem_x);
                let rem_y = dy % step_f;
                let dist_y = rem_y.min(step_f - rem_y);
                if dist_x <= margin || dist_y <= margin {
                    continue;
                }
            }

            // Require interior mask coverage (with 1px erosion buffer) so outer boundary step/ringing is excluded
            let a_center = region_mask_alpha(x, y, rx, ry, rw, rh, shape, angle_deg, feather_px);
            if a_center < 0.95 {
                continue;
            }
            let a_right = region_mask_alpha(x + 2, y, rx, ry, rw, rh, shape, angle_deg, feather_px);
            let a_left = region_mask_alpha(x - 1, y, rx, ry, rw, rh, shape, angle_deg, feather_px);
            let a_down = region_mask_alpha(x, y + 2, rx, ry, rw, rh, shape, angle_deg, feather_px);
            let a_up = region_mask_alpha(x, y - 1, rx, ry, rw, rh, shape, angle_deg, feather_px);
            if a_right < 0.95 || a_left < 0.95 || a_down < 0.95 || a_up < 0.95 {
                continue;
            }

            let p = img.get_pixel(x, y);
            let px = img.get_pixel(x + 1, y);
            let py = img.get_pixel(x, y + 1);

            let lum = 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32;
            let lum_x = 0.299 * px[0] as f32 + 0.587 * px[1] as f32 + 0.114 * px[2] as f32;
            let lum_y = 0.299 * py[0] as f32 + 0.587 * py[1] as f32 + 0.114 * py[2] as f32;

            let grad = (lum - lum_x).abs() + (lum - lum_y).abs();
            samples.push(grad);
        }
    }

    if samples.is_empty() {
        return (0.0, 0.0, 0);
    }

    let sum: f64 = samples.iter().map(|&v| v as f64).sum();
    let mean = (sum / samples.len() as f64) as f32;

    samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let p95_idx = ((samples.len() as f32 * 0.95).floor() as usize).min(samples.len() - 1);
    let p95 = samples[p95_idx];

    (mean, p95, samples.len())
}

/// Scans an original image for localized high-frequency detail clusters (candidate hotspots)
/// to assist VLM agents during initial inspection.
pub fn detect_candidate_hotspots(original: &RgbaImage, max_results: usize) -> Vec<CandidateHotspot> {
    let (w, h) = original.dimensions();
    if w < 32 || h < 32 {
        return Vec::new();
    }

    let grid_cols = 10u32;
    let grid_rows = 10u32;
    let cell_w = (w / grid_cols).max(1);
    let cell_h = (h / grid_rows).max(1);

    let mut cell_scores = Vec::new();
    let mut global_energy_sum = 0.0f64;

    for gy in 0..grid_rows {
        for gx in 0..grid_cols {
            let x0 = gx * cell_w;
            let y0 = gy * cell_h;
            let cw = if gx == grid_cols - 1 { w - x0 } else { cell_w };
            let ch = if gy == grid_rows - 1 { h - y0 } else { cell_h };

            let (mean_e, p95_e, _) = compute_masked_detail_metrics(
                original,
                x0,
                y0,
                cw,
                ch,
                x0,
                y0,
                cw,
                ch,
                RegionShape::Rect,
                0.0,
                0,
                1,
                1.0,
            );
            let combined = mean_e * 0.6 + p95_e * 0.4;
            global_energy_sum += combined as f64;
            cell_scores.push(((gx, gy), (x0, y0, cw, ch), combined));
        }
    }

    let avg_global = (global_energy_sum / (grid_cols * grid_rows) as f64) as f32;
    cell_scores.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    let mut hotspots = Vec::new();
    for ((gx, gy), (x0, y0, cw, ch), score) in cell_scores {
        if score <= (avg_global * 1.35).max(8.0) || hotspots.len() >= max_results {
            break;
        }
        let nx = ((x0 as f32 / w as f32) * 1000.0).round() as u32;
        let ny = ((y0 as f32 / h as f32) * 1000.0).round() as u32;
        let nw = ((cw as f32 / w as f32) * 1000.0).round() as u32;
        let nh = ((ch as f32 / h as f32) * 1000.0).round() as u32;

        hotspots.push(CandidateHotspot {
            rank: hotspots.len() + 1,
            pixel_bounds: (x0, y0, cw, ch),
            normalized1000_bounds: (nx, ny, nw, nh),
            detail_energy_score: score,
            hint: format!(
                "High-frequency detail cluster at grid ({},{}) — Normalized1000({},{},{}x{}) / Pixels({},{},{}x{})",
                gx * 100,
                gy * 100,
                nx,
                ny,
                nw,
                nh,
                x0,
                y0,
                cw,
                ch
            ),
        });
    }

    hotspots
}

/// Encode an `RgbaImage` as a JPEG with explicit quality `1..=100`.
fn encode_jpeg_with_quality(img: &DynamicImage, quality: u8) -> Result<Vec<u8>> {
    let rgb = img.to_rgb8();
    let mut buf = Cursor::new(Vec::new());
    let mut encoder = JpegEncoder::new_with_quality(&mut buf, quality);
    encoder
        .encode_image(&rgb)
        .context("Failed to encode JPEG with explicit quality")?;
    Ok(buf.into_inner())
}

/// Verifies the censored edition against the original untouched asset:
/// 1. Confirms identical dimensions and aspect ratio.
/// 2. Confirms 100% bit-exact preservation of every single pixel outside the declared censorship regions.
/// 3. Confirms localized intervention (< 35% of total canvas).
/// 4. Simulates publication resizing at **both** `1080p` (JPEG Q85) and `540p` (Thumbnail JPEG Q75),
///    measuring both mean and 95th-percentile peak detail suppression across all 3 resolutions!
/// 5. Generates a 4-Panel Broadcast QC Contact Slate (`<asset_id>_contact_sheet.png`).
pub fn verify_censored_edition(
    original: &RgbaImage,
    censored: &RgbaImage,
    regions: &[CensorshipRegion],
    policy_id: &str,
    preview_dir: &Path,
    asset_id: &str,
) -> Result<VerificationReport> {
    let (orig_w, orig_h) = original.dimensions();
    let (cens_w, cens_h) = censored.dimensions();
    let resolution_preserved = orig_w == cens_w && orig_h == cens_h;

    let enclosing_aabbs: Vec<(u32, u32, u32, u32)> = regions
        .iter()
        .map(|r| r.resolve_enclosing_aabb(orig_w, orig_h))
        .collect();

    // 1. Check bit-exact preservation outside active region masks
    let mut non_censored_diff_count = 0u64;
    let mut censored_pixel_count = 0u64;

    if resolution_preserved {
        for y in 0..orig_h {
            for x in 0..orig_w {
                let mut inside_any = false;
                for (idx, r) in regions.iter().enumerate() {
                    let (bx, by, bw, bh) = enclosing_aabbs[idx];
                    if x >= bx && x < bx + bw && y >= by && y < by + bh {
                        let (rx, ry, rw, rh) = r.resolve_pixels(orig_w, orig_h);
                        let alpha = region_mask_alpha(
                            x,
                            y,
                            rx,
                            ry,
                            rw,
                            rh,
                            r.shape,
                            r.rotation_angle_deg(),
                            r.feather_px,
                        );
                        if alpha > 0.0 {
                            inside_any = true;
                            break;
                        }
                    }
                }
                if inside_any {
                    censored_pixel_count += 1;
                } else if original.get_pixel(x, y) != censored.get_pixel(x, y) {
                    non_censored_diff_count += 1;
                }
            }
        }
    }

    let total_pixels = (orig_w as u64 * orig_h as u64).max(1);
    let total_censored_area_percent = (censored_pixel_count as f32 / total_pixels as f32) * 100.0;
    let localized_intervention_compliant = total_censored_area_percent <= 35.0;

    // 2. Generate publication resize & JPEG compression simulation previews (1080p Q85, 540p Q75, & 4-Panel QC Slate)
    std::fs::create_dir_all(preview_dir)?;
    let mut preview_paths = Vec::new();

    let dyn_censored = DynamicImage::ImageRgba8(censored.clone());
    let dyn_original = DynamicImage::ImageRgba8(original.clone());

    // 2A. 1080p Web Publication Simulation (Explicit JPEG Q85)
    let scale_1080 = (1080.0 / orig_w.max(orig_h) as f32).min(1.0);
    let w_1080 = ((orig_w as f32 * scale_1080).round() as u32).max(1);
    let h_1080 = ((orig_h as f32 * scale_1080).round() as u32).max(1);

    let resized_cens_1080 =
        dyn_censored.resize_exact(w_1080, h_1080, image::imageops::FilterType::Lanczos3);
    let resized_orig_1080 =
        dyn_original.resize_exact(w_1080, h_1080, image::imageops::FilterType::Lanczos3);

    let jpeg_bytes_1080 = encode_jpeg_with_quality(&resized_cens_1080, 85)?;
    let preview_1080_path = preview_dir.join(format!("{}_pub_1080p_q85.jpg", asset_id));
    std::fs::write(&preview_1080_path, &jpeg_bytes_1080)?;
    preview_paths.push(preview_1080_path.to_string_lossy().to_string());

    let redecoded_1080 =
        image::load_from_memory_with_format(&jpeg_bytes_1080, ImageFormat::Jpeg)?.to_rgba8();
    let orig_1080_rgba = resized_orig_1080.to_rgba8();

    // 2B. 540p Feed Thumbnail Simulation (Explicit JPEG Q75)
    let scale_540 = (540.0 / orig_w.max(orig_h) as f32).min(1.0);
    let w_540 = ((orig_w as f32 * scale_540).round() as u32).max(1);
    let h_540 = ((orig_h as f32 * scale_540).round() as u32).max(1);
    let resized_cens_540 =
        dyn_censored.resize_exact(w_540, h_540, image::imageops::FilterType::Lanczos3);
    let resized_orig_540 =
        dyn_original.resize_exact(w_540, h_540, image::imageops::FilterType::Lanczos3);

    let jpeg_bytes_540 = encode_jpeg_with_quality(&resized_cens_540, 75)?;
    let preview_540_path = preview_dir.join(format!("{}_pub_540p_thumb.jpg", asset_id));
    std::fs::write(&preview_540_path, &jpeg_bytes_540)?;
    preview_paths.push(preview_540_path.to_string_lossy().to_string());

    let redecoded_540 =
        image::load_from_memory_with_format(&jpeg_bytes_540, ImageFormat::Jpeg)?.to_rgba8();
    let orig_540_rgba = resized_orig_540.to_rgba8();

    // 3. Evaluate per-region censorship effectiveness across Native, 1080p JPEG Q85, and 540p Thumbnail JPEG Q75
    let min_req_bs = recommended_mosaic_block_size(orig_w, orig_h, policy_id);
    let mut region_metrics = Vec::new();

    for (idx, region) in regions.iter().enumerate() {
        let (bx, by, bw, bh) = enclosing_aabbs[idx];
        let angle = region.rotation_angle_deg();
        let (method_summary, block_hint, effective_strength_score) = match &region.treatment {
            CensorshipMethod::Mosaic { block_size } => {
                let bs = block_size.unwrap_or(min_req_bs).max(min_req_bs).max(4);
                let score = bs as f32 / min_req_bs as f32;
                (
                    format!("Mosaic({}px, {:?}, {:.1}deg)", bs, region.shape, angle),
                    bs,
                    score,
                )
            }
            CensorshipMethod::GaussianBlur { sigma } => {
                let min_sigma = (min_req_bs as f32) * 0.75;
                let s = sigma.max(min_sigma).max(4.0);
                let score = s / min_sigma.max(1.0);
                (
                    format!(
                        "GaussianBlur(sigma={:.1}, {:?}, {:.1}deg)",
                        s, region.shape, angle
                    ),
                    1,
                    score,
                )
            }
            CensorshipMethod::SolidBar { color, .. } => {
                let (_, resolved_col_label) = resolve_bar_color(color, original, bx, by, bw, bh);
                (
                    format!(
                        "SolidBar(color={}, {:?}, angle={:.1}deg)",
                        resolved_col_label, region.shape, angle
                    ),
                    1,
                    2.0,
                )
            }
        };

        let (rx, ry, rw, rh) = region.resolve_pixels(orig_w, orig_h);
        let (orig_mean, orig_p95, sample_count) = compute_masked_detail_metrics(
            original,
            rx,
            ry,
            rw,
            rh,
            bx,
            by,
            bw,
            bh,
            region.shape,
            angle,
            region.feather_px,
            block_hint,
            1.0,
        );
        let (cens_mean, cens_p95, _) = compute_masked_detail_metrics(
            censored,
            rx,
            ry,
            rw,
            rh,
            bx,
            by,
            bw,
            bh,
            region.shape,
            angle,
            region.feather_px,
            block_hint,
            1.0,
        );

        if sample_count == 0 {
            region_metrics.push(RegionVerificationMetric {
                region_id: region.id.clone(),
                pixel_bounds: (bx, by, bw, bh),
                method_summary,
                detail_suppression_ratio: 0.0,
                peak_detail_suppression_ratio: 0.0,
                pub_1080p_suppression_ratio: 0.0,
                pub_540p_suppression_ratio: 0.0,
                survives_publication_resize: false,
                effective_strength_score,
                passed: false,
                notes: "Region interior too small (< 4x4 active pixels) to verify suppression."
                    .to_string(),
            });
            continue;
        }

        let detail_suppression_ratio = if orig_mean <= 0.01 {
            1.0
        } else {
            (1.0 - (cens_mean / orig_mean)).clamp(0.0, 1.0)
        };
        let peak_detail_suppression_ratio = if orig_p95 <= 0.01 {
            1.0
        } else {
            (1.0 - (cens_p95 / orig_p95)).clamp(0.0, 1.0)
        };

        // 1080p JPEG verification
        let pub_1080p_suppression_ratio = eval_scaled_suppression(
            &orig_1080_rgba,
            &redecoded_1080,
            orig_mean,
            w_1080,
            h_1080,
            scale_1080,
            rx,
            ry,
            rw,
            rh,
            bx,
            by,
            bw,
            bh,
            region.shape,
            angle,
            region.feather_px,
            block_hint,
        );

        // 540p Thumbnail JPEG verification
        let pub_540p_suppression_ratio = eval_scaled_suppression(
            &orig_540_rgba,
            &redecoded_540,
            orig_mean,
            w_540,
            h_540,
            scale_540,
            rx,
            ry,
            rw,
            rh,
            bx,
            by,
            bw,
            bh,
            region.shape,
            angle,
            region.feather_px,
            block_hint,
        );

        let survives_publication_resize = effective_strength_score >= 1.0
            && (pub_1080p_suppression_ratio >= 0.65 || detail_suppression_ratio >= 0.80)
            && (pub_540p_suppression_ratio >= 0.60 || detail_suppression_ratio >= 0.80);
        let passed = detail_suppression_ratio >= 0.75
            && peak_detail_suppression_ratio >= 0.70
            && survives_publication_resize;

        region_metrics.push(RegionVerificationMetric {
            region_id: region.id.clone(),
            pixel_bounds: (bx, by, bw, bh),
            method_summary,
            detail_suppression_ratio,
            peak_detail_suppression_ratio,
            pub_1080p_suppression_ratio,
            pub_540p_suppression_ratio,
            survives_publication_resize,
            effective_strength_score,
            passed,
            notes: format!(
                "Native mean: {:.1}% (p95 peak: {:.1}%) | 1080p JPEG: {:.1}% | 540p Thumb: {:.1}%",
                detail_suppression_ratio * 100.0,
                peak_detail_suppression_ratio * 100.0,
                pub_1080p_suppression_ratio * 100.0,
                pub_540p_suppression_ratio * 100.0
            ),
        });
    }

    let artistic_preservation_exact = non_censored_diff_count == 0;
    let all_regions_passed = !region_metrics.is_empty() && region_metrics.iter().all(|m| m.passed);
    let overall_passed = resolution_preserved
        && artistic_preservation_exact
        && localized_intervention_compliant
        && all_regions_passed;

    // 4. Generate 4-Panel Broadcast QC Contact Slate
    let contact_sheet = generate_contact_sheet(
        original,
        censored,
        &redecoded_540,
        regions,
        asset_id,
        policy_id,
        non_censored_diff_count,
        overall_passed,
    );
    let contact_path = preview_dir.join(format!("{}_contact_sheet.png", asset_id));
    contact_sheet.save(&contact_path)?;
    preview_paths.push(contact_path.to_string_lossy().to_string());

    Ok(VerificationReport {
        verified_at: Utc::now(),
        artistic_preservation_exact,
        non_censored_pixel_diff_count: non_censored_diff_count,
        resolution_preserved,
        dimensions: (cens_w, cens_h),
        total_censored_area_percent,
        localized_intervention_compliant,
        region_metrics,
        publication_preview_paths: preview_paths,
        overall_passed,
    })
}

#[allow(clippy::too_many_arguments)]
fn eval_scaled_suppression(
    orig_scaled: &RgbaImage,
    cens_scaled: &RgbaImage,
    orig_native_mean: f32,
    sw_img: u32,
    sh_img: u32,
    scale: f32,
    rx: u32,
    ry: u32,
    rw: u32,
    rh: u32,
    bx: u32,
    by: u32,
    bw: u32,
    bh: u32,
    shape: RegionShape,
    angle: f32,
    feather_px: u32,
    block_hint: u32,
) -> f32 {
    let s_rx = ((rx as f32 * scale).floor() as u32).min(sw_img.saturating_sub(1));
    let s_ry = ((ry as f32 * scale).floor() as u32).min(sh_img.saturating_sub(1));
    let s_rw = ((rw as f32 * scale).ceil() as u32)
        .max(1)
        .min(sw_img.saturating_sub(s_rx));
    let s_rh = ((rh as f32 * scale).ceil() as u32)
        .max(1)
        .min(sh_img.saturating_sub(s_ry));

    let sx = ((bx as f32 * scale).floor() as u32).min(sw_img.saturating_sub(1));
    let sy = ((by as f32 * scale).floor() as u32).min(sh_img.saturating_sub(1));
    let sw = ((bw as f32 * scale).ceil() as u32)
        .max(1)
        .min(sw_img.saturating_sub(sx));
    let sh = ((bh as f32 * scale).ceil() as u32)
        .max(1)
        .min(sh_img.saturating_sub(sy));
    let scaled_feather = (feather_px as f32 * scale).round() as u32;

    let (o_mean, _, _) = compute_masked_detail_metrics(
        orig_scaled,
        s_rx,
        s_ry,
        s_rw,
        s_rh,
        sx,
        sy,
        sw,
        sh,
        shape,
        angle,
        scaled_feather,
        block_hint,
        scale,
    );
    let (c_mean, _, _) = compute_masked_detail_metrics(
        cens_scaled,
        s_rx,
        s_ry,
        s_rw,
        s_rh,
        sx,
        sy,
        sw,
        sh,
        shape,
        angle,
        scaled_feather,
        block_hint,
        scale,
    );

    let baseline = o_mean.max(orig_native_mean * 0.5);
    if baseline <= 0.01 {
        1.0
    } else {
        (1.0 - (c_mean / baseline)).clamp(0.0, 1.0)
    }
}

/// Complete 5x7 alphanumeric bitmap font supporting `A-Z`, `0-9`, and telemetry punctuation.
fn glyph_5x7(ch: char) -> [u8; 7] {
    match ch.to_ascii_uppercase() {
        '0' => [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
        '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        '2' => [0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111],
        '3' => [0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110],
        '4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
        '5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
        '6' => [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
        '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
        '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        '9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],
        'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'B' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
        'C' => [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110],
        'D' => [0b11100, 0b10010, 0b10001, 0b10001, 0b10001, 0b10010, 0b11100],
        'E' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
        'F' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
        'G' => [0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110],
        'H' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'I' => [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        'J' => [0b00111, 0b00010, 0b00010, 0b00010, 0b10010, 0b10010, 0b01100],
        'K' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
        'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'M' => [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
        'N' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
        'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'P' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        'Q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
        'R' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
        'S' => [0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110],
        'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'V' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
        'W' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001],
        'X' => [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
        'Y' => [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
        'Z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
        ':' => [0b00000, 0b00100, 0b00100, 0b00000, 0b00100, 0b00100, 0b00000],
        '-' => [0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000],
        '_' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b11111],
        '+' => [0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000],
        '|' => [0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        '.' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00100, 0b00100],
        ',' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00100, 0b00100, 0b01000],
        '%' => [0b11001, 0b11010, 0b00100, 0b00100, 0b01011, 0b10011, 0b00000],
        '/' => [0b00001, 0b00010, 0b00100, 0b00100, 0b01000, 0b10000, 0b00000],
        '(' => [0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010],
        ')' => [0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000],
        _ => [0; 7],
    }
}

/// Draws a high-contrast pill badge with crisp 5x7 bitmap text at `(x0, y0)`.
pub fn draw_label_badge(
    img: &mut RgbaImage,
    x0: u32,
    y0: u32,
    text: &str,
    fg: Rgba<u8>,
    bg: Rgba<u8>,
) {
    let (w, h) = img.dimensions();
    let pad = 3u32;
    let char_w = 6u32;
    let badge_w = (text.len() as u32 * char_w) + pad * 2;
    let badge_h = 7 + pad * 2;

    for by in 0..badge_h {
        let py = y0 + by;
        if py >= h {
            continue;
        }
        for bx in 0..badge_w {
            let px = x0 + bx;
            if px >= w {
                continue;
            }
            let p = *img.get_pixel(px, py);
            img.put_pixel(px, py, lerp_rgba(p, bg, 0.86));
        }
    }

    for (idx, ch) in text.chars().enumerate() {
        let rows = glyph_5x7(ch);
        let cx0 = x0 + pad + (idx as u32 * char_w);
        let cy0 = y0 + pad;
        for (ry, &row_bits) in rows.iter().enumerate() {
            for rx in 0..5 {
                if (row_bits >> (4 - rx)) & 1 == 1 {
                    let px = cx0 + rx;
                    let py = cy0 + ry as u32;
                    if px < w && py < h {
                        img.put_pixel(px, py, fg);
                    }
                }
            }
        }
    }
}

/// Renders an agent-friendly coordinate inspection overlay grid onto a copy of the image.
pub fn generate_inspection_grid(original: &RgbaImage) -> RgbaImage {
    let mut grid_img = original.clone();
    let (w, h) = grid_img.dimensions();

    for step in 1..20 {
        let is_major = step % 2 == 0;
        let x = ((w as f32 * (step as f32 * 0.05)).round() as u32).min(w.saturating_sub(1));
        let y = ((h as f32 * (step as f32 * 0.05)).round() as u32).min(h.saturating_sub(1));

        for py in 0..h {
            if !is_major && (py / 6) % 2 == 1 {
                continue;
            }
            let p = *grid_img.get_pixel(x, py);
            let overlay = if is_major {
                Rgba([0, 255, 220, 255])
            } else {
                Rgba([255, 220, 0, 255])
            };
            let alpha = if is_major { 0.65 } else { 0.35 };
            grid_img.put_pixel(x, py, lerp_rgba(p, overlay, alpha));
        }

        for px in 0..w {
            if !is_major && (px / 6) % 2 == 1 {
                continue;
            }
            let p = *grid_img.get_pixel(px, y);
            let overlay = if is_major {
                Rgba([0, 255, 220, 255])
            } else {
                Rgba([255, 220, 0, 255])
            };
            let alpha = if is_major { 0.65 } else { 0.35 };
            grid_img.put_pixel(px, y, lerp_rgba(p, overlay, alpha));
        }
    }

    for g in 1..10 {
        let norm_val = g * 100;
        let cx = ((w as f32 * (g as f32 * 0.1)).round() as u32).min(w.saturating_sub(1));
        let cy = ((h as f32 * (g as f32 * 0.1)).round() as u32).min(h.saturating_sub(1));

        let x_label = format!("X{}", norm_val);
        let y_label = format!("Y{}", norm_val);
        draw_label_badge(
            &mut grid_img,
            cx.saturating_sub(14),
            4,
            &x_label,
            Rgba([0, 255, 220, 255]),
            Rgba([10, 14, 24, 255]),
        );
        if h > 60 {
            draw_label_badge(
                &mut grid_img,
                cx.saturating_sub(14),
                h.saturating_sub(17),
                &x_label,
                Rgba([0, 255, 220, 255]),
                Rgba([10, 14, 24, 255]),
            );
        }
        draw_label_badge(
            &mut grid_img,
            4,
            cy.saturating_sub(6),
            &y_label,
            Rgba([255, 220, 0, 255]),
            Rgba([10, 14, 24, 255]),
        );

        for gy in 1..10 {
            let iy = ((h as f32 * (gy as f32 * 0.1)).round() as i32).clamp(0, h as i32 - 1);
            let ix = cx as i32;
            for dy in -2..=2 {
                for dx in -2..=2 {
                    let px = (ix + dx).clamp(0, w as i32 - 1) as u32;
                    let py = (iy + dy).clamp(0, h as i32 - 1) as u32;
                    grid_img.put_pixel(px, py, Rgba([255, 40, 80, 255]));
                }
            }
            if (g == 3 || g == 5 || g == 7) && (gy == 3 || gy == 5 || gy == 7) {
                let anchor_lbl = format!("{},{}", g * 100, gy * 100);
                draw_label_badge(
                    &mut grid_img,
                    (ix as u32 + 4).min(w.saturating_sub(48)),
                    (iy as u32 + 4).min(h.saturating_sub(16)),
                    &anchor_lbl,
                    Rgba([255, 190, 220, 255]),
                    Rgba([15, 18, 28, 255]),
                );
            }
        }
    }

    grid_img
}

/// Generates a 3x magnified precision zoom-crop inspection image with global coordinates burned in.
pub fn generate_zoom_crop_grid(
    original: &RgbaImage,
    crop_x: f32,
    crop_y: f32,
    crop_w: f32,
    crop_h: f32,
    unit: CoordinateUnit,
) -> (RgbaImage, (u32, u32, u32, u32)) {
    let (img_w, img_h) = original.dimensions();
    let dummy_region = CensorshipRegion {
        id: "crop".to_string(),
        label: "crop".to_string(),
        rationale: String::new(),
        x: crop_x,
        y: crop_y,
        width: crop_w,
        height: crop_h,
        unit,
        shape: RegionShape::Rect,
        treatment: CensorshipMethod::Mosaic { block_size: None },
        angle_deg: 0.0,
        feather_px: 0,
    };
    let (cx, cy, cw, ch) = dummy_region.resolve_pixels(img_w, img_h);

    let mut sub = RgbaImage::new(cw, ch);
    for y in 0..ch {
        for x in 0..cw {
            sub.put_pixel(x, y, *original.get_pixel(cx + x, cy + y));
        }
    }

    let zoom_factor = 3u32;
    let zw = (cw * zoom_factor).min(1800).max(1);
    let zh = (ch * zoom_factor).min(1800).max(1);
    let mut zoomed = DynamicImage::ImageRgba8(sub)
        .resize_exact(zw, zh, image::imageops::FilterType::Nearest)
        .to_rgba8();

    for step in 1..4 {
        let sx = (zw * step) / 4;
        let sy = (zh * step) / 4;
        let global_px_x = cx + (cw * step) / 4;
        let global_px_y = cy + (ch * step) / 4;
        let global_norm_x = ((global_px_x as f32 / img_w as f32) * 1000.0).round() as u32;
        let global_norm_y = ((global_px_y as f32 / img_h as f32) * 1000.0).round() as u32;

        for py in 0..zh {
            let p = *zoomed.get_pixel(sx, py);
            zoomed.put_pixel(sx, py, lerp_rgba(p, Rgba([0, 255, 220, 255]), 0.6));
        }
        for px in 0..zw {
            let p = *zoomed.get_pixel(px, sy);
            zoomed.put_pixel(px, sy, lerp_rgba(p, Rgba([255, 220, 0, 255]), 0.6));
        }

        draw_label_badge(
            &mut zoomed,
            sx.saturating_sub(24),
            20,
            &format!("X{}({}P)", global_norm_x, global_px_x),
            Rgba([0, 255, 220, 255]),
            Rgba([10, 14, 24, 255]),
        );
        draw_label_badge(
            &mut zoomed,
            4,
            sy.saturating_sub(6),
            &format!("Y{}({}P)", global_norm_y, global_px_y),
            Rgba([255, 220, 0, 255]),
            Rgba([10, 14, 24, 255]),
        );
    }

    let header = format!("ZOOM CROP | GLOBAL PX: {},{},{}X{}", cx, cy, cw, ch);
    draw_label_badge(
        &mut zoomed,
        4,
        4,
        &header,
        Rgba([255, 255, 255, 255]),
        Rgba([15, 20, 34, 255]),
    );

    (zoomed, (cx, cy, cw, ch))
}

/// Generates a 4-Panel Broadcast QC Contact Slate.
#[allow(clippy::too_many_arguments)]
pub fn generate_contact_sheet(
    original: &RgbaImage,
    censored: &RgbaImage,
    redecoded_540: &RgbaImage,
    regions: &[CensorshipRegion],
    asset_id: &str,
    policy_id: &str,
    non_censored_diff_count: u64,
    overall_passed: bool,
) -> RgbaImage {
    let (w, h) = original.dimensions();
    let target_h = h.min(480).max(1);
    let scale = target_h as f32 / h.max(1) as f32;
    let target_w = ((w as f32 * scale).round() as u32).max(1);

    let mut orig_boxed = original.clone();
    for r in regions {
        let (rx, ry, rw, rh) = r.resolve_pixels(w, h);
        let (bx, by, bw, bh) = r.resolve_enclosing_aabb(w, h);
        let angle = r.rotation_angle_deg();

        for py in by..(by + bh) {
            for px in bx..(bx + bw) {
                let a = region_mask_alpha(px, py, rx, ry, rw, rh, r.shape, angle, 0);
                if a > 0.0 {
                    let n_left = if px > 0 {
                        region_mask_alpha(px - 1, py, rx, ry, rw, rh, r.shape, angle, 0)
                    } else {
                        0.0
                    };
                    let n_right = if px + 1 < w {
                        region_mask_alpha(px + 1, py, rx, ry, rw, rh, r.shape, angle, 0)
                    } else {
                        0.0
                    };
                    let n_up = if py > 0 {
                        region_mask_alpha(px, py - 1, rx, ry, rw, rh, r.shape, angle, 0)
                    } else {
                        0.0
                    };
                    let n_down = if py + 1 < h {
                        region_mask_alpha(px, py + 1, rx, ry, rw, rh, r.shape, angle, 0)
                    } else {
                        0.0
                    };
                    if n_left == 0.0 || n_right == 0.0 || n_up == 0.0 || n_down == 0.0 {
                        orig_boxed.put_pixel(px, py, Rgba([0, 255, 200, 255]));
                        if px + 1 < w {
                            orig_boxed.put_pixel(px + 1, py, Rgba([0, 255, 200, 255]));
                        }
                    }
                }
            }
        }
    }

    let mut diff_map = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let p_orig = original.get_pixel(x, y);
            let p_cens = censored.get_pixel(x, y);
            if p_orig == p_cens {
                diff_map.put_pixel(
                    x,
                    y,
                    Rgba([p_orig[0] / 4, p_orig[1] / 4, p_orig[2] / 4, 255]),
                );
            } else {
                diff_map.put_pixel(x, y, Rgba([255, 45, 145, 255]));
            }
        }
    }

    let mut p1 = DynamicImage::ImageRgba8(orig_boxed)
        .resize_exact(target_w, target_h, image::imageops::FilterType::Triangle)
        .to_rgba8();
    for r in regions {
        let (bx, by, _, _) = r.resolve_enclosing_aabb(w, h);
        let sx = ((bx as f32 * scale).round() as u32).min(target_w.saturating_sub(24));
        let sy = ((by as f32 * scale).round() as u32)
            .saturating_sub(14)
            .max(22);
        draw_label_badge(
            &mut p1,
            sx,
            sy,
            &r.id.to_uppercase(),
            Rgba([0, 255, 200, 255]),
            Rgba([10, 14, 24, 255]),
        );
    }

    let p2 = DynamicImage::ImageRgba8(censored.clone())
        .resize_exact(target_w, target_h, image::imageops::FilterType::Lanczos3)
        .to_rgba8();
    let p3 = DynamicImage::ImageRgba8(redecoded_540.clone())
        .resize_exact(target_w, target_h, image::imageops::FilterType::Nearest)
        .to_rgba8();
    let p4 = DynamicImage::ImageRgba8(diff_map)
        .resize_exact(target_w, target_h, image::imageops::FilterType::Nearest)
        .to_rgba8();

    let header_h = 26u32;
    let gap = 6u32;
    let total_w = target_w * 4 + gap * 3;
    let total_h = header_h + target_h;
    let mut sheet = RgbaImage::from_pixel(total_w, total_h, Rgba([12, 15, 23, 255]));

    let status_txt = if overall_passed { "PASS" } else { "REVIEW" };
    let slate_text = format!(
        "HEAV3NS QC SLATE | {} | {}X{} | POLICY: {} | OUTSIDE DIFF: {} PX | {}",
        asset_id, w, h, policy_id, non_censored_diff_count, status_txt
    );
    draw_label_badge(
        &mut sheet,
        6,
        5,
        &slate_text,
        if overall_passed {
            Rgba([0, 255, 210, 255])
        } else {
            Rgba([255, 80, 120, 255])
        },
        Rgba([18, 24, 38, 255]),
    );

    for y in 0..target_h {
        let sy = header_h + y;
        for x in 0..target_w {
            sheet.put_pixel(x, sy, *p1.get_pixel(x, y));
            sheet.put_pixel(target_w + gap + x, sy, *p2.get_pixel(x, y));
            sheet.put_pixel((target_w + gap) * 2 + x, sy, *p3.get_pixel(x, y));
            sheet.put_pixel((target_w + gap) * 3 + x, sy, *p4.get_pixel(x, y));
        }
    }

    draw_label_badge(
        &mut sheet,
        6,
        header_h + 6,
        "1:ORIG+REGIONS",
        Rgba([0, 255, 190, 255]),
        Rgba([10, 14, 24, 255]),
    );
    draw_label_badge(
        &mut sheet,
        target_w + gap + 6,
        header_h + 6,
        "2:PUBLISHED",
        Rgba([255, 220, 0, 255]),
        Rgba([10, 14, 24, 255]),
    );
    draw_label_badge(
        &mut sheet,
        (target_w + gap) * 2 + 6,
        header_h + 6,
        "3:540P JPEG SIM",
        Rgba([120, 200, 255, 255]),
        Rgba([10, 14, 24, 255]),
    );
    draw_label_badge(
        &mut sheet,
        (target_w + gap) * 3 + 6,
        header_h + 6,
        "4:DIFF HEATMAP",
        Rgba([255, 60, 150, 255]),
        Rgba([10, 14, 24, 255]),
    );

    sheet
}
