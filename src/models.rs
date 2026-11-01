use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetStatus {
    PendingInspection,
    Planned,
    Processed,
    Verified,
    ApprovedOriginal,
    Escalated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CensorshipClassification {
    ExplicitRequiresIntervention,
    ArtisticNudityPass,
    SuggestivePass,
    AmbiguousEscalate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum CensorshipMethod {
    /// Localized mosaic pixelation for explicit anatomical details.
    /// `block_size` is in pixels; if None or 0, dynamically calculated via Japanese Article 175 standard (max(dim)/100, min 4px).
    Mosaic {
        #[serde(default)]
        block_size: Option<u32>,
    },
    /// Soft obscuring treatment using halo-padded separable Gaussian blur.
    GaussianBlur {
        sigma: f32,
    },
    /// Deliberate graphic masking (kuro-nuri #000000, shiro-nuri #FFFFFF, or "auto" luminance-adaptive).
    SolidBar {
        /// Hex color like "#000000", "#FFFFFF", or "auto" (default "#000000")
        #[serde(default = "default_bar_color")]
        color: String,
        /// Optional rotation angle in degrees around region center (also supported on CensorshipRegion.angle_deg)
        #[serde(default)]
        angle_deg: f32,
    },
}

fn default_bar_color() -> String {
    "#000000".to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoordinateUnit {
    /// Absolute pixel coordinates (x, y, width, height)
    Pixels,
    /// Normalized 0..=1000 coordinates (standard VLM bounding box format)
    Normalized1000,
    /// Percentage 0.0..=100.0 coordinates
    Percent,
}

impl Default for CoordinateUnit {
    fn default() -> Self {
        Self::Pixels
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegionShape {
    /// Sharp-cornered rectangle (classic kuro-nuri bar or rectangular mosaic box)
    Rect,
    /// Smooth ellipse / oval mask
    Ellipse,
    /// Round-capped stadium / capsule shape (ideal for angled anatomical contours)
    Pill,
}

impl Default for RegionShape {
    fn default() -> Self {
        Self::Rect
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CensorshipRegion {
    pub id: String,
    pub label: String,
    pub rationale: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    #[serde(default)]
    pub unit: CoordinateUnit,
    #[serde(default)]
    pub shape: RegionShape,
    pub treatment: CensorshipMethod,
    /// Optional rotation angle in degrees around region center (applies to any shape/treatment)
    #[serde(default)]
    pub angle_deg: f32,
    /// Optional feathering radius (in pixels) at region boundary for natural integration
    #[serde(default)]
    pub feather_px: u32,
}

impl CensorshipRegion {
    /// Resolve nominal unrotated region coordinates into absolute pixel bounds `(x, y, width, height)` clamped to `(img_w, img_h)`.
    pub fn resolve_pixels(&self, img_w: u32, img_h: u32) -> (u32, u32, u32, u32) {
        if img_w == 0 || img_h == 0 {
            return (0, 0, 0, 0);
        }
        let x = if self.x.is_finite() { self.x } else { 0.0 };
        let y = if self.y.is_finite() { self.y } else { 0.0 };
        let width = if self.width.is_finite() { self.width } else { 1.0 };
        let height = if self.height.is_finite() { self.height } else { 1.0 };

        let (rx, ry, rw, rh) = match self.unit {
            CoordinateUnit::Pixels => (x, y, width, height),
            CoordinateUnit::Normalized1000 => (
                (x / 1000.0) * img_w as f32,
                (y / 1000.0) * img_h as f32,
                (width / 1000.0) * img_w as f32,
                (height / 1000.0) * img_h as f32,
            ),
            CoordinateUnit::Percent => (
                (x / 100.0) * img_w as f32,
                (y / 100.0) * img_h as f32,
                (width / 100.0) * img_w as f32,
                (height / 100.0) * img_h as f32,
            ),
        };

        let x0 = (rx.round().max(0.0) as u32).min(img_w.saturating_sub(1));
        let y0 = (ry.round().max(0.0) as u32).min(img_h.saturating_sub(1));
        let max_w = img_w.saturating_sub(x0);
        let max_h = img_h.saturating_sub(y0);
        let w = (rw.round().max(1.0) as u32).min(max_w);
        let h = (rh.round().max(1.0) as u32).min(max_h);
        (x0, y0, w, h)
    }

    /// Returns the effective rotation angle in degrees around the region center.
    /// Checks `self.angle_deg` first, then falls back to `SolidBar { angle_deg }` for backward compatibility.
    pub fn rotation_angle_deg(&self) -> f32 {
        if self.angle_deg.is_finite() && self.angle_deg.abs() > 0.01 {
            return self.angle_deg;
        }
        match &self.treatment {
            CensorshipMethod::SolidBar { angle_deg, .. } if angle_deg.is_finite() => *angle_deg,
            _ => 0.0,
        }
    }

    /// Computes the axis-aligned bounding box `(bx, by, bw, bh)` that fully encloses the region,
    /// including any rotation around the region center `(cx, cy)`.
    pub fn resolve_enclosing_aabb(&self, img_w: u32, img_h: u32) -> (u32, u32, u32, u32) {
        let (rx, ry, rw, rh) = self.resolve_pixels(img_w, img_h);
        let angle = self.rotation_angle_deg();
        if angle.abs() <= 0.01 || rw == 0 || rh == 0 {
            return (rx, ry, rw, rh);
        }

        let cx = rx as f32 + rw as f32 * 0.5;
        let cy = ry as f32 + rh as f32 * 0.5;
        let rad = angle.to_radians();
        let cos_a = rad.cos().abs();
        let sin_a = rad.sin().abs();

        let rot_w = (rw as f32 * cos_a + rh as f32 * sin_a).ceil();
        let rot_h = (rw as f32 * sin_a + rh as f32 * cos_a).ceil();

        let x0 = (cx - rot_w * 0.5).floor().max(0.0) as u32;
        let y0 = (cy - rot_h * 0.5).floor().max(0.0) as u32;
        let x1 = ((cx + rot_w * 0.5).ceil().max(1.0) as u32).min(img_w);
        let y1 = ((cy + rot_h * 0.5).ceil().max(1.0) as u32).min(img_h);

        let bx = x0.min(img_w.saturating_sub(1));
        let by = y0.min(img_h.saturating_sub(1));
        let bw = x1.saturating_sub(bx).max(1).min(img_w.saturating_sub(bx));
        let bh = y1.saturating_sub(by).max(1).min(img_h.saturating_sub(by));
        (bx, by, bw, bh)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateHotspot {
    pub rank: usize,
    pub pixel_bounds: (u32, u32, u32, u32),
    pub normalized1000_bounds: (u32, u32, u32, u32),
    pub detail_energy_score: f32,
    pub hint: String,
}

fn default_confuser_category() -> String {
    "benign_artistic_feature".to_string()
}

/// Verified Benign / Hard-Negative Region Annotation for False-Positive Suppression.
/// Marks a spatial region (e.g., classical silhouette, stylized muscle linework, clothing seam,
/// skin highlight) that resembles a trigger hotspot but is verified benign and MUST NOT be censored.
/// Never alters a single pixel of the artwork; exported to JSONL, COCO, and YOLO training splits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegativeAnnotationRegion {
    pub id: String,
    pub label: String,
    pub rationale: String,
    #[serde(default = "default_confuser_category")]
    pub confuser_category: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    #[serde(default)]
    pub unit: CoordinateUnit,
    #[serde(default)]
    pub shape: RegionShape,
}

impl NegativeAnnotationRegion {
    pub fn resolve_pixels(&self, img_w: u32, img_h: u32) -> (u32, u32, u32, u32) {
        if img_w == 0 || img_h == 0 {
            return (0, 0, 0, 0);
        }
        let x = if self.x.is_finite() { self.x } else { 0.0 };
        let y = if self.y.is_finite() { self.y } else { 0.0 };
        let width = if self.width.is_finite() { self.width } else { 1.0 };
        let height = if self.height.is_finite() { self.height } else { 1.0 };

        let (rx, ry, rw, rh) = match self.unit {
            CoordinateUnit::Pixels => (x, y, width, height),
            CoordinateUnit::Normalized1000 => (
                (x / 1000.0) * img_w as f32,
                (y / 1000.0) * img_h as f32,
                (width / 1000.0) * img_w as f32,
                (height / 1000.0) * img_h as f32,
            ),
            CoordinateUnit::Percent => (
                (x / 100.0) * img_w as f32,
                (y / 100.0) * img_h as f32,
                (width / 100.0) * img_w as f32,
                (height / 100.0) * img_h as f32,
            ),
        };

        let x0 = (rx.round().max(0.0) as u32).min(img_w.saturating_sub(1));
        let y0 = (ry.round().max(0.0) as u32).min(img_h.saturating_sub(1));
        let max_w = img_w.saturating_sub(x0);
        let max_h = img_h.saturating_sub(y0);
        let w = (rw.round().max(1.0) as u32).min(max_w);
        let h = (rh.round().max(1.0) as u32).min(max_h);
        (x0, y0, w, h)
    }
}

/// Computes Intersection-over-Union (IoU) between two pixel bounding boxes `(x, y, w, h)`.
pub fn compute_box_iou(a: (u32, u32, u32, u32), b: (u32, u32, u32, u32)) -> f32 {
    let ax1 = a.0 as f32;
    let ay1 = a.1 as f32;
    let ax2 = (a.0 + a.2) as f32;
    let ay2 = (a.1 + a.3) as f32;

    let bx1 = b.0 as f32;
    let by1 = b.1 as f32;
    let bx2 = (b.0 + b.2) as f32;
    let by2 = (b.1 + b.3) as f32;

    let ix1 = ax1.max(bx1);
    let iy1 = ay1.max(by1);
    let ix2 = ax2.min(bx2);
    let iy2 = ay2.min(by2);

    let iw = (ix2 - ix1).max(0.0);
    let ih = (iy2 - iy1).max(0.0);
    let inter = iw * ih;
    if inter <= 0.0 {
        return 0.0;
    }
    let area_a = (a.2 as f32) * (a.3 as f32);
    let area_b = (b.2 as f32) * (b.3 as f32);
    let union = (area_a + area_b - inter).max(1e-6);
    inter / union
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionVerificationMetric {
    pub region_id: String,
    pub pixel_bounds: (u32, u32, u32, u32),
    pub method_summary: String,
    /// Mean high-frequency edge energy reduction ratio (0.0 = no reduction, 1.0 = total elimination of fine detail)
    pub detail_suppression_ratio: f32,
    /// 95th-percentile peak high-contrast line suppression ratio (catches single 1px anatomical lines)
    #[serde(default)]
    pub peak_detail_suppression_ratio: f32,
    /// Suppression ratio after 1080p Lanczos3 downscale + JPEG Q85 recompression
    #[serde(default)]
    pub pub_1080p_suppression_ratio: f32,
    /// Suppression ratio after 540p thumbnail downscale + JPEG Q75 recompression
    #[serde(default)]
    pub pub_540p_suppression_ratio: f32,
    /// Survives downscaling & JPEG compression check at both 1080p and 540p resolutions
    pub survives_publication_resize: bool,
    /// Ratio of applied strength vs. Japanese Article 175 minimum requirement (>= 1.0)
    pub effective_strength_score: f32,
    pub passed: bool,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationReport {
    pub verified_at: DateTime<Utc>,
    /// True if all pixels outside censored regions are 100% bit-identical to the original artwork
    pub artistic_preservation_exact: bool,
    pub non_censored_pixel_diff_count: u64,
    pub resolution_preserved: bool,
    pub dimensions: (u32, u32),
    pub total_censored_area_percent: f32,
    pub localized_intervention_compliant: bool,
    pub region_metrics: Vec<RegionVerificationMetric>,
    pub publication_preview_paths: Vec<String>,
    pub overall_passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationInfo {
    pub escalated_at: DateTime<Utc>,
    pub reason: String,
    pub ambiguity_category: String,
    pub agent_notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetRecord {
    pub asset_id: String,
    pub original_filename: String,
    pub inbox_path: Option<String>,
    pub archive_original_path: String,
    pub sha256_original: String,
    pub width: u32,
    pub height: u32,
    pub status: AssetStatus,
    pub classification: Option<CensorshipClassification>,
    pub policy_profile: String,
    pub regions: Vec<CensorshipRegion>,
    /// Verified benign / hard-negative regions annotated to suppress false positives in downstream ML/VLM classifiers
    #[serde(default)]
    pub negative_regions: Vec<NegativeAnnotationRegion>,
    pub inspect_grid_path: Option<String>,
    pub published_edition_path: Option<String>,
    pub sha256_published: Option<String>,
    pub verification: Option<VerificationReport>,
    pub escalation: Option<EscalationInfo>,
    pub audit_trail: Vec<AuditEvent>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub actor: String,
    pub details: String,
}

/// Deterministic studio ledger using `BTreeMap` so `treatments_ledger.json` is 100% byte-reproducible.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ledger {
    pub commission_authority: String,
    pub commissioned_agent: String,
    pub principal_artist: String,
    pub version: String,
    pub updated_at: DateTime<Utc>,
    pub assets: BTreeMap<String, AssetRecord>,
}

impl Default for Ledger {
    fn default() -> Self {
        Self {
            commission_authority: "MAJO, Executive Chair".to_string(),
            commissioned_agent: "Deputy Gemini".to_string(),
            principal_artist: "EGG".to_string(),
            version: "4.0.0".to_string(),
            updated_at: Utc::now(),
            assets: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyGuide {
    pub id: String,
    pub name: String,
    pub legal_or_platform_basis: String,
    pub mosaic_rule: String,
    pub min_mosaic_block_divisor: u32,
    pub absolute_min_mosaic_px: u32,
    pub artistic_nudity_guidance: String,
    pub explicit_criteria: Vec<String>,
    pub escalation_triggers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorCheck {
    pub check_name: String,
    pub passed: bool,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorReport {
    pub checked_at: DateTime<Utc>,
    pub overall_healthy: bool,
    pub total_assets_audited: usize,
    pub orphaned_files_cleaned: usize,
    pub checks: Vec<DoctorCheck>,
    pub remediation_hints: Vec<String>,
}
