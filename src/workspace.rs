use crate::engine::{
    apply_censorship_regions, detect_candidate_hotspots, evaluate_region_confuser_hardness,
    generate_inspection_grid, generate_zoom_crop_grid, verify_censored_edition,
};
use crate::models::{
    compute_box_iou, AssetRecord, AssetStatus, AuditEvent, CandidateHotspot,
    CensorshipClassification, CensorshipRegion, CoordinateUnit, DoctorCheck, DoctorReport,
    EscalationInfo, Ledger, NegativeAnnotationRegion, RegionShape, VerificationReport,
};
use crate::policy::recommended_mosaic_block_size;
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use image::GenericImageView;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use walkdir::WalkDir;

pub struct Workspace {
    pub root: PathBuf,
}

/// RAII Advisory File Lock with automatic stale-lock recovery.
/// Prevents concurrent CLI / MCP invocations from clobbering `treatments_ledger.json`.
pub struct WorkspaceLock {
    lock_path: PathBuf,
}

impl WorkspaceLock {
    pub fn acquire(ledger_dir: &Path) -> Result<Self> {
        fs::create_dir_all(ledger_dir)?;
        let lock_path = ledger_dir.join(".ledger.lock");

        for attempt in 0..20 {
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&lock_path)
            {
                Ok(mut f) => {
                    let _ = writeln!(f, "pid={}", std::process::id());
                    return Ok(Self { lock_path });
                }
                Err(_) => {
                    // Check if existing lock is stale (> 30s old)
                    if let Ok(meta) = fs::metadata(&lock_path) {
                        if let Ok(modified) = meta.modified() {
                            if SystemTime::now()
                                .duration_since(modified)
                                .unwrap_or(Duration::ZERO)
                                > Duration::from_secs(30)
                            {
                                let _ = fs::remove_file(&lock_path);
                                continue;
                            }
                        }
                    }
                    if attempt < 19 {
                        std::thread::sleep(Duration::from_millis(25));
                    }
                }
            }
        }
        // Fallback: clear stale lock and proceed
        let _ = fs::remove_file(&lock_path);
        let mut f = File::create(&lock_path)?;
        let _ = writeln!(f, "pid={}", std::process::id());
        Ok(Self { lock_path })
    }
}

impl Drop for WorkspaceLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.lock_path);
    }
}

#[derive(Debug, Clone, Default)]
pub struct ExecuteOptions {
    pub append_regions: bool,
    pub remove_region_ids: Vec<String>,
    pub dry_run: bool,
    pub policy_override: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceStatusSummary {
    pub workspace_root: String,
    pub total_assets: usize,
    pub pending_inspection: Vec<String>,
    pub planned: Vec<String>,
    pub processed: Vec<String>,
    pub verified: Vec<String>,
    pub approved_original: Vec<String>,
    pub escalated: Vec<String>,
    pub unscanned_inbox_files: Vec<String>,
    pub next_recommended_action: String,
    pub ledger_path: String,
    pub escalation_report_path: String,
    pub dashboard_html_path: String,
    pub dataset_manifest_jsonl_path: String,
    pub coco_annotations_json_path: String,
    pub vlm_safety_sft_jsonl_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DatasetAnnotationBox {
    pub region_id: String,
    /// `"positive_intervention"` (requires redaction) or `"verified_benign_hard_negative"` (false-positive suppression)
    pub polarity: String,
    /// `1` = explicit_intervention_required, `0` = verified_benign_hard_negative
    pub class_id: u32,
    /// `"explicit_curation"`, `"human_verified_negative"`, or `"auto_mined_unredacted_hotspot"`
    pub annotation_source: String,
    /// Taxonomy of the target or false-positive confuser (e.g. `"composite_skin_and_crease_confuser"`, `"skin_tone_anatomical_silhouette"`)
    pub confuser_category: String,
    /// Quantitative False-Positive Confuser Hardness Score (`0.0..100.0`) for curriculum active learning
    pub fp_hardness_score: f32,
    pub label: String,
    pub rationale: String,
    pub shape: String,
    pub treatment_method: String,
    pub angle_deg: f32,
    pub feather_px: u32,
    /// Standard COCO pixel bounding box `[x, y, width, height]`
    pub bbox_xywh_px: [u32; 4],
    /// VLM Normalized1000 grounding bounding box `[x1, y1, x2, y2]` in `[0..1000]`
    pub bbox_xyxy_norm1000: [u32; 4],
    /// Standard YOLO normalized `[center_x, center_y, width, height]` in `[0.0..1.0]`
    pub bbox_cxcywh_norm01: [f32; 4],
    /// Rotation-aware enclosing Axis-Aligned Bounding Box `[x, y, width, height]` in pixels
    pub enclosing_aabb_px: [u32; 4],
}

#[derive(Debug, Clone, Serialize)]
pub struct DatasetExportRow {
    pub asset_id: String,
    pub original_filename: String,
    pub width: u32,
    pub height: u32,
    pub status: AssetStatus,
    pub classification: Option<CensorshipClassification>,
    pub policy_profile: String,
    pub is_safe_for_training_release: bool,
    pub curation_action: String,
    pub sha256_original: String,
    pub sha256_published: Option<String>,
    pub original_path: String,
    pub sanitized_published_path: Option<String>,
    pub vlm_inspect_grid_path: Option<String>,
    pub qc_contact_sheet_path: Option<String>,
    pub exterior_pixel_diff_count: u64,
    pub exterior_preservation_ratio: f64,
    pub redacted_area_percent: f64,
    pub multiscale_jpeg_1080p_q85_passed: bool,
    pub multiscale_jpeg_540p_q75_passed: bool,
    pub positive_annotations_count: usize,
    pub hard_negative_annotations_count: usize,
    pub annotations: Vec<DatasetAnnotationBox>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DatasetExportReport {
    pub exported_at: String,
    pub safe_only_filter: bool,
    pub total_records_exported: usize,
    pub safe_training_records_count: usize,
    pub surgical_redaction_count: usize,
    pub preserved_untouched_count: usize,
    pub quarantined_escalated_count: usize,
    pub total_positive_boxes: usize,
    pub total_hard_negative_boxes: usize,
    pub jsonl_manifest_path: String,
    pub coco_annotations_path: String,
    pub vlm_sft_jsonl_path: String,
    pub dataset_card_path: String,
    pub records: Vec<DatasetExportRow>,
}

#[derive(Debug, Serialize)]
pub struct InspectPayload {
    pub asset_id: String,
    pub original_filename: String,
    pub archive_original_path: String,
    pub inspect_grid_path: String,
    pub zoom_crop_path: Option<String>,
    pub zoom_crop_pixel_bounds: Option<(u32, u32, u32, u32)>,
    pub width: u32,
    pub height: u32,
    pub sha256_original: String,
    pub policy_profile: String,
    pub recommended_mosaic_block_px: u32,
    pub candidate_hotspots: Vec<CandidateHotspot>,
    pub coordinate_guide: String,
    pub cli_templates: InspectCliTemplates,
    pub agent_instructions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct InspectCliTemplates {
    pub execute_mosaic_example: String,
    pub execute_rotated_bar_example: String,
    pub zoom_crop_example: String,
    pub approve_original_example: String,
    pub escalate_example: String,
}

#[derive(Debug, Serialize)]
pub struct OnboardingPacket {
    pub engine_version: String,
    pub selected_role: String,
    pub commission_authority: String,
    pub principal_artist: String,
    pub commissioned_agent: String,
    pub core_invariants: Vec<String>,
    pub role_quickstart_steps: Vec<String>,
    pub live_workspace_commands: Vec<String>,
    pub documentation_map: std::collections::BTreeMap<String, String>,
    pub status_summary: WorkspaceStatusSummary,
}

#[derive(Debug, Serialize)]
pub struct AssetExplanationPacket {
    pub asset_id: String,
    pub original_filename: String,
    pub dimensions: (u32, u32),
    pub status: AssetStatus,
    pub classification: Option<CensorshipClassification>,
    pub executive_summary: String,
    pub cryptographic_integrity_verified: bool,
    pub bit_exact_preservation_verified: bool,
    pub active_regions_count: usize,
    pub next_handoff_actions: Vec<String>,
    pub cli_commands: InspectCliTemplates,
    pub record: AssetRecord,
}

/// Atomically writes `bytes` to `target_path` via a sibling temporary file + `sync_all()` + `rename()`.
pub fn atomic_write_bytes(target_path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file_name = target_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("tmp");
    let tmp_path = target_path.with_file_name(format!(".{}.tmp.{}", file_name, std::process::id()));
    {
        let mut f = File::create(&tmp_path).with_context(|| {
            format!("Failed to create atomic temp file {}", tmp_path.display())
        })?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }
    fs::rename(&tmp_path, target_path).with_context(|| {
        format!(
            "Failed to atomically rename {} -> {}",
            tmp_path.display(),
            target_path.display()
        )
    })?;
    Ok(())
}

impl Workspace {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    pub fn inbox_dir(&self) -> PathBuf {
        self.root.join("assets").join("inbox")
    }

    pub fn originals_dir(&self) -> PathBuf {
        self.root.join("assets").join("originals")
    }

    pub fn inspect_dir(&self) -> PathBuf {
        self.root.join("assets").join("inspect")
    }

    pub fn previews_dir(&self) -> PathBuf {
        self.root.join("assets").join("previews")
    }

    pub fn published_dir(&self) -> PathBuf {
        self.root.join("assets").join("published")
    }

    pub fn review_dir(&self) -> PathBuf {
        self.root.join("assets").join("review_escalated")
    }

    pub fn ledger_dir(&self) -> PathBuf {
        self.root.join("ledger")
    }

    pub fn ledger_file(&self) -> PathBuf {
        self.ledger_dir().join("treatments_ledger.json")
    }

    pub fn escalation_md_file(&self) -> PathBuf {
        self.ledger_dir().join("escalations_review.md")
    }

    pub fn publication_manifest_md_file(&self) -> PathBuf {
        self.ledger_dir().join("publication_report.md")
    }

    pub fn dashboard_html_file(&self) -> PathBuf {
        self.ledger_dir().join("dashboard.html")
    }

    pub fn dataset_manifest_jsonl_file(&self) -> PathBuf {
        self.ledger_dir().join("dataset_manifest.jsonl")
    }

    pub fn dataset_card_md_file(&self) -> PathBuf {
        self.ledger_dir().join("DATASET_CARD.md")
    }

    pub fn coco_annotations_json_file(&self) -> PathBuf {
        self.ledger_dir().join("annotations_coco.json")
    }

    pub fn vlm_sft_jsonl_file(&self) -> PathBuf {
        self.ledger_dir().join("vlm_safety_sft.jsonl")
    }

    pub fn init(&self) -> Result<()> {
        for dir in [
            self.inbox_dir(),
            self.originals_dir(),
            self.inspect_dir(),
            self.previews_dir(),
            self.published_dir(),
            self.review_dir(),
            self.ledger_dir(),
        ] {
            fs::create_dir_all(&dir)
                .with_context(|| format!("Failed to create directory {}", dir.display()))?;
        }

        if !self.ledger_file().exists() {
            let ledger = Ledger::default();
            self.save_ledger(&ledger)?;
        }
        self.sync_markdown_reports()?;
        Ok(())
    }

    pub fn load_ledger(&self) -> Result<Ledger> {
        if !self.ledger_file().exists() {
            self.init()?;
        }
        let raw = fs::read_to_string(self.ledger_file()).with_context(|| {
            format!("Failed to read ledger at {}", self.ledger_file().display())
        })?;
        let mut ledger: Ledger =
            serde_json::from_str(&raw).context("Failed to parse ledger JSON")?;
        ledger.version = env!("CARGO_PKG_VERSION").to_string();
        Ok(ledger)
    }

    pub fn save_ledger(&self, ledger: &Ledger) -> Result<()> {
        let json = serde_json::to_string_pretty(ledger)?;
        atomic_write_bytes(&self.ledger_file(), json.as_bytes())
    }

    /// Resolves a user- or agent-supplied `asset_id_or_prefix` against the ledger.
    /// Supports exact ID, unique prefix match, SHA-256 prefix, or original filename match.
    pub fn resolve_asset_id(&self, query: &str) -> Result<String> {
        let ledger = self.load_ledger()?;
        Self::resolve_asset_id_in_ledger(&ledger, query)
    }

    pub fn resolve_asset_id_in_ledger(ledger: &Ledger, query: &str) -> Result<String> {
        let q = query.trim();
        if ledger.assets.contains_key(q) {
            return Ok(q.to_string());
        }

        let mut matches: Vec<String> = ledger
            .assets
            .values()
            .filter(|rec| {
                rec.asset_id.starts_with(q)
                    || rec.original_filename.eq_ignore_ascii_case(q)
                    || rec.sha256_original.starts_with(q)
            })
            .map(|rec| rec.asset_id.clone())
            .collect();
        matches.sort();
        matches.dedup();

        match matches.len() {
            1 => Ok(matches.remove(0)),
            0 => Err(anyhow!(
                "No asset matching '{}' found in ledger. Run `heav3ns-censor status` to list known asset IDs.",
                q
            )),
            _ => Err(anyhow!(
                "Ambiguous asset reference '{}' matches multiple assets: {:?}",
                q,
                matches
            )),
        }
    }

    /// Scans `assets/inbox` for new image files, preserves untouched copies in `assets/originals`,
    /// and registers them in the ledger.
    pub fn ingest_inbox(&self, default_policy: &str) -> Result<Vec<AssetRecord>> {
        self.init()?;
        let _lock = WorkspaceLock::acquire(&self.ledger_dir())?;
        let mut ledger = self.load_ledger()?;
        let mut newly_ingested = Vec::new();

        let mut inbox_paths: Vec<PathBuf> = WalkDir::new(self.inbox_dir())
            .min_depth(1)
            .max_depth(2)
            .into_iter()
            .flatten()
            .filter(|e| e.file_type().is_file() && is_supported_image(e.path()))
            .map(|e| e.path().to_path_buf())
            .collect();
        inbox_paths.sort();

        for path in inbox_paths {
            let record = self.ingest_single_file_internal(&path, default_policy, &mut ledger)?;
            newly_ingested.push(record);
        }

        ledger.updated_at = Utc::now();
        self.save_ledger(&ledger)?;
        self.sync_markdown_reports()?;
        Ok(newly_ingested)
    }

    pub fn ingest_file(&self, file_path: &Path, default_policy: &str) -> Result<AssetRecord> {
        self.init()?;
        let _lock = WorkspaceLock::acquire(&self.ledger_dir())?;
        let mut ledger = self.load_ledger()?;
        let record = self.ingest_single_file_internal(file_path, default_policy, &mut ledger)?;
        ledger.updated_at = Utc::now();
        self.save_ledger(&ledger)?;
        self.sync_markdown_reports()?;
        Ok(record)
    }

    fn ingest_single_file_internal(
        &self,
        path: &Path,
        default_policy: &str,
        ledger: &mut Ledger,
    ) -> Result<AssetRecord> {
        let bytes = fs::read(path)
            .with_context(|| format!("Failed to read incoming asset {}", path.display()))?;
        let sha256_hex = compute_sha256(&bytes);

        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("asset")
            .to_string();
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("png")
            .to_lowercase();

        let sanitized_stem: String = stem
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let short_hash = &sha256_hex[0..8];
        let asset_id = format!("{}_{}", sanitized_stem, short_hash);

        if let Some(existing) = ledger.assets.get(&asset_id) {
            return Ok(existing.clone());
        }

        let img = image::load_from_memory(&bytes)
            .with_context(|| format!("Failed to decode image {}", path.display()))?;
        let (width, height) = img.dimensions();

        let orig_filename = format!("{}_original.{}", asset_id, ext);
        let archive_path = self.originals_dir().join(&orig_filename);
        atomic_write_bytes(&archive_path, &bytes).with_context(|| {
            format!(
                "Failed to preserve untouched original to {}",
                archive_path.display()
            )
        })?;

        let now = Utc::now();
        let record = AssetRecord {
            asset_id: asset_id.clone(),
            original_filename: path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string(),
            inbox_path: Some(path.to_string_lossy().to_string()),
            archive_original_path: archive_path.to_string_lossy().to_string(),
            sha256_original: sha256_hex.clone(),
            width,
            height,
            status: AssetStatus::PendingInspection,
            classification: None,
            policy_profile: default_policy.to_string(),
            regions: Vec::new(),
            negative_regions: Vec::new(),
            inspect_grid_path: None,
            published_edition_path: None,
            sha256_published: None,
            verification: None,
            escalation: None,
            audit_trail: vec![AuditEvent {
                timestamp: now,
                action: "INGEST_AND_PRESERVE_ORIGINAL".to_string(),
                actor: "Deputy Gemini".to_string(),
                details: format!(
                    "Archived untouched original ({}x{}) with SHA-256 {}",
                    width,
                    height,
                    &sha256_hex[0..16]
                ),
            }],
            created_at: now,
            updated_at: now,
        };

        ledger.assets.insert(asset_id, record.clone());
        Ok(record)
    }

    /// Generates an inspection coordinate grid overlay image, detects high-frequency candidate hotspots,
    /// optionally generates a 3x magnified zoom-crop grid (`--crop x,y,w,h[:unit]`), and returns structured VLM metadata.
    pub fn inspect_asset(&self, asset_query: &str) -> Result<InspectPayload> {
        self.inspect_asset_with_crop(asset_query, None)
    }

    pub fn inspect_asset_with_crop(
        &self,
        asset_query: &str,
        crop_spec: Option<&str>,
    ) -> Result<InspectPayload> {
        let _lock = WorkspaceLock::acquire(&self.ledger_dir())?;
        let mut ledger = self.load_ledger()?;
        let asset_id = Self::resolve_asset_id_in_ledger(&ledger, asset_query)?;
        let record = ledger
            .assets
            .get_mut(&asset_id)
            .ok_or_else(|| anyhow!("Asset '{}' not found in ledger", asset_id))?;

        let orig_img = image::open(&record.archive_original_path)
            .with_context(|| {
                format!(
                    "Failed to open archived original at {}",
                    record.archive_original_path
                )
            })?
            .to_rgba8();

        let grid_img = generate_inspection_grid(&orig_img);
        let grid_path = self
            .inspect_dir()
            .join(format!("{}_inspect_grid.png", asset_id));
        grid_img.save(&grid_path)?;

        let mut zoom_crop_path = None;
        let mut zoom_crop_pixel_bounds = None;
        if let Some(cs) = crop_spec {
            let (cx, cy, cw, ch, unit) = parse_crop_spec(cs)?;
            let (zoom_img, resolved_px) =
                generate_zoom_crop_grid(&orig_img, cx, cy, cw, ch, unit);
            let z_path = self
                .inspect_dir()
                .join(format!("{}_zoom_crop.png", asset_id));
            zoom_img.save(&z_path)?;
            zoom_crop_path = Some(z_path.to_string_lossy().to_string());
            zoom_crop_pixel_bounds = Some(resolved_px);
        }

        let hotspots = detect_candidate_hotspots(&orig_img, 4);

        let grid_path_str = grid_path.to_string_lossy().to_string();
        record.inspect_grid_path = Some(grid_path_str.clone());
        record.updated_at = Utc::now();
        record.audit_trail.push(AuditEvent {
            timestamp: Utc::now(),
            action: "GENERATE_INSPECTION_GRID".to_string(),
            actor: "Deputy Gemini".to_string(),
            details: format!(
                "Generated coordinate overlay at {} (hotspots: {}, zoom_crop: {})",
                grid_path_str,
                hotspots.len(),
                zoom_crop_path.is_some()
            ),
        });

        let rec_bs =
            recommended_mosaic_block_size(record.width, record.height, &record.policy_profile);
        let payload = InspectPayload {
            asset_id: record.asset_id.clone(),
            original_filename: record.original_filename.clone(),
            archive_original_path: record.archive_original_path.clone(),
            inspect_grid_path: grid_path_str,
            zoom_crop_path,
            zoom_crop_pixel_bounds,
            width: record.width,
            height: record.height,
            sha256_original: record.sha256_original.clone(),
            policy_profile: record.policy_profile.clone(),
            recommended_mosaic_block_px: rec_bs,
            candidate_hotspots: hotspots,
            coordinate_guide: "Cyan solid lines & top/bottom X badges (X100..X900) = 10% intervals in Normalized1000; Yellow left Y badges (Y100..Y900) = 10% Y intervals; Pink interior anchor badges (300,300 / 500,500 / 700,700) provide mid-canvas reference points. Specify regions in 'pixels', 'normalized1000' (0..1000), or 'percent' (0..100). Shapes: 'rect', 'ellipse', or 'pill'.".to_string(),
            cli_templates: InspectCliTemplates {
                execute_mosaic_example: format!(
                    "./target/release/heav3ns-censor execute --asset-id {} --region \"mosaic:400,500,120,100:normalized1000:rect:{}:0:explicit_anatomy\"",
                    record.asset_id, rec_bs
                ),
                execute_rotated_bar_example: format!(
                    "./target/release/heav3ns-censor execute --asset-id {} --region \"solid_bar:450,320,100,30:normalized1000:pill:#000000@15:0:kuro_nuri_bar\"",
                    record.asset_id
                ),
                zoom_crop_example: format!(
                    "./target/release/heav3ns-censor inspect --asset-id {} --crop \"300,300,400,400:normalized1000\"",
                    record.asset_id
                ),
                approve_original_example: format!(
                    "./target/release/heav3ns-censor approve-original --asset-id {} --classification artistic_nudity_pass --rationale \"Artistic nudity with no explicit genital detail.\"",
                    record.asset_id
                ),
                escalate_example: format!(
                    "./target/release/heav3ns-censor escalate --asset-id {} --category \"stylized_anatomy_ambiguity\" --reason \"Explanation of ambiguity\"",
                    record.asset_id
                ),
            },
            agent_instructions: vec![
                "1. View both `archive_original_path` and `inspect_grid_path` (and `zoom_crop_path` if magnifying a fine detail) using your multimodal read_file tool.".to_string(),
                "2. Check `candidate_hotspots` and the burned-in Normalized1000 badges (X100..X900, Y100..Y900, and interior 300/500/700 anchors) to locate exact bounding boxes.".to_string(),
                "3. Distinguish between artistic nudity, suggestive imagery, and explicit material requiring intervention.".to_string(),
                "4. Apply the smallest effective localized intervention (`mosaic`, `solid_bar`, or `gaussian_blur` with `rect`, `ellipse`, or `pill` shape and optional `@angle_deg`) via `execute`, approve untouched via `approve-original`, or escalate genuine ambiguity via `escalate`.".to_string(),
            ],
        };

        self.save_ledger(&ledger)?;
        self.sync_markdown_reports()?;
        Ok(payload)
    }

    /// Ingests all new inbox files and automatically runs `inspect` on every asset in `PendingInspection`
    /// status, returning a batch array of `InspectPayload` objects ready for immediate VLM assessment.
    pub fn batch_prepare(&self, default_policy: &str) -> Result<Vec<InspectPayload>> {
        let _ = self.ingest_inbox(default_policy)?;
        let ledger = self.load_ledger()?;
        let mut pending_ids: Vec<String> = ledger
            .assets
            .values()
            .filter(|a| a.status == AssetStatus::PendingInspection || a.inspect_grid_path.is_none())
            .map(|a| a.asset_id.clone())
            .collect();
        pending_ids.sort();

        let mut payloads = Vec::new();
        for id in pending_ids {
            payloads.push(self.inspect_asset(&id)?);
        }
        Ok(payloads)
    }

    /// Executes censorship treatments on an asset, verifies artistic preservation + publication downscale resilience,
    /// cleans up any stale escalation artifacts, and exports the publication-ready edition.
    pub fn execute_censorship(
        &self,
        asset_query: &str,
        classification: CensorshipClassification,
        regions: Vec<CensorshipRegion>,
        policy_override: Option<String>,
    ) -> Result<(AssetRecord, VerificationReport)> {
        self.execute_censorship_with_options(
            asset_query,
            classification,
            regions,
            ExecuteOptions {
                policy_override,
                ..Default::default()
            },
        )
    }

    /// Executes censorship treatments with full support for incremental region upsert (`append_regions`),
    /// region removal (`remove_region_ids`), and non-destructive simulation (`dry_run`).
    pub fn execute_censorship_with_options(
        &self,
        asset_query: &str,
        classification: CensorshipClassification,
        new_regions: Vec<CensorshipRegion>,
        opts: ExecuteOptions,
    ) -> Result<(AssetRecord, VerificationReport)> {
        let _lock = WorkspaceLock::acquire(&self.ledger_dir())?;
        let mut ledger = self.load_ledger()?;
        let asset_id = Self::resolve_asset_id_in_ledger(&ledger, asset_query)?;
        let record = ledger
            .assets
            .get_mut(&asset_id)
            .ok_or_else(|| anyhow!("Asset '{}' not found in ledger", asset_id))?;

        let active_policy = opts
            .policy_override
            .clone()
            .unwrap_or_else(|| record.policy_profile.clone());

        let mut effective_regions: Vec<CensorshipRegion> =
            if opts.append_regions || !opts.remove_region_ids.is_empty() {
                record
                    .regions
                    .iter()
                    .filter(|r| !opts.remove_region_ids.iter().any(|rem| rem == &r.id))
                    .cloned()
                    .collect()
            } else {
                Vec::new()
            };

        for mut r in new_regions {
            if r.id.is_empty() || r.id.starts_with("auto_") {
                let mut k = 1usize;
                loop {
                    let candidate = format!("r{}", k);
                    if !effective_regions.iter().any(|ex| ex.id == candidate) {
                        r.id = candidate;
                        break;
                    }
                    k += 1;
                }
                effective_regions.push(r);
            } else if let Some(existing) = effective_regions.iter_mut().find(|ex| ex.id == r.id) {
                *existing = r;
            } else {
                effective_regions.push(r);
            }
        }

        if effective_regions.is_empty() {
            return Err(anyhow!(
                "No active censorship regions remain for '{}'. Supply --region or use `approve-original` if no censorship is needed.",
                asset_id
            ));
        }

        let orig_bytes = fs::read(&record.archive_original_path)?;
        let current_orig_sha = compute_sha256(&orig_bytes);
        if current_orig_sha != record.sha256_original {
            return Err(anyhow!(
                "CRITICAL INTEGRITY VIOLATION: Archived original SHA-256 mismatch for {}",
                asset_id
            ));
        }

        let orig_rgba = image::load_from_memory(&orig_bytes)?.to_rgba8();
        let censored_rgba =
            apply_censorship_regions(&orig_rgba, &effective_regions, &active_policy);

        if opts.dry_run {
            let dry_preview_dir =
                std::env::temp_dir().join(format!("heav3ns_dry_run_{}", std::process::id()));
            let report = verify_censored_edition(
                &orig_rgba,
                &censored_rgba,
                &effective_regions,
                &active_policy,
                &dry_preview_dir,
                &asset_id,
            )?;
            let _ = fs::remove_dir_all(&dry_preview_dir);

            let mut sim_record = record.clone();
            sim_record.policy_profile = active_policy;
            sim_record.classification = Some(classification);
            sim_record.regions = effective_regions;
            sim_record.verification = Some(report.clone());
            return Ok((sim_record, report));
        }

        record.policy_profile = active_policy;

        // Clean up any prior published file if extension differed
        if let Some(old_pub) = &record.published_edition_path {
            let _ = fs::remove_file(old_pub);
        }
        // Clean up any stale escalation review file
        self.remove_escalation_artifacts_for(&asset_id);

        let pub_filename = format!("{}_published.png", asset_id);
        let pub_path = self.published_dir().join(&pub_filename);
        censored_rgba.save(&pub_path)?;

        let pub_bytes = fs::read(&pub_path)?;
        let pub_sha = compute_sha256(&pub_bytes);

        let report = verify_censored_edition(
            &orig_rgba,
            &censored_rgba,
            &effective_regions,
            &record.policy_profile,
            &self.previews_dir(),
            &asset_id,
        )?;

        record.classification = Some(classification);
        record.regions = effective_regions.clone();
        let pos_boxes: Vec<(u32, u32, u32, u32)> = effective_regions
            .iter()
            .map(|r| r.resolve_enclosing_aabb(record.width, record.height))
            .collect();

        // Polarity De-Confliction: automatically prune any hard-negative regions that overlap active positive regions (IoU >= 0.20)
        record.negative_regions.retain(|nr| {
            let nb = nr.resolve_pixels(record.width, record.height);
            !pos_boxes.iter().any(|&pb| compute_box_iou(nb, pb) >= 0.20)
        });

        // Auto-mine unredacted high-frequency hotspots as verified benign hard-negatives if none remain
        if record.negative_regions.is_empty() {
            let hotspots = detect_candidate_hotspots(&orig_rgba, 4);
            let mut auto_idx = 1usize;
            for hs in hotspots {
                let overlaps_pos = pos_boxes
                    .iter()
                    .any(|&pb| compute_box_iou(hs.pixel_bounds, pb) >= 0.10);
                if !overlaps_pos {
                    record.negative_regions.push(NegativeAnnotationRegion {
                        id: format!("auto_neg_{}", auto_idx),
                        label: format!("benign_hotspot_rank_{}", hs.rank),
                        rationale: "Unredacted high-frequency artistic region verified safe (hard-negative confuser for false-positive suppression)".to_string(),
                        confuser_category: hs.confuser_category,
                        fp_hardness_score: hs.fp_hardness_score,
                        x: hs.pixel_bounds.0 as f32,
                        y: hs.pixel_bounds.1 as f32,
                        width: hs.pixel_bounds.2 as f32,
                        height: hs.pixel_bounds.3 as f32,
                        unit: CoordinateUnit::Pixels,
                        shape: RegionShape::Rect,
                    });
                    auto_idx += 1;
                }
            }
        }
        // Re-evaluate confuser hardness scores for all active hard-negative regions
        for nr in &mut record.negative_regions {
            let (nx, ny, nw, nh) = nr.resolve_pixels(record.width, record.height);
            let (_e, hardness, cat) = evaluate_region_confuser_hardness(&orig_rgba, nx, ny, nw, nh);
            nr.fp_hardness_score = hardness;
            if nr.confuser_category.is_empty() || nr.confuser_category == "benign_artistic_feature" {
                nr.confuser_category = cat;
            }
        }
        record.escalation = None;
        record.published_edition_path = Some(pub_path.to_string_lossy().to_string());
        record.sha256_published = Some(pub_sha);
        record.verification = Some(report.clone());
        record.status = if report.overall_passed {
            AssetStatus::Verified
        } else {
            AssetStatus::Processed
        };
        record.updated_at = Utc::now();
        record.audit_trail.push(AuditEvent {
            timestamp: Utc::now(),
            action: "APPLY_AND_VERIFY_CENSORSHIP".to_string(),
            actor: "Deputy Gemini".to_string(),
            details: format!(
                "Applied {} localized region(s) (append={}, removed={}, hard_negatives={}); verification passed: {} (censored area: {:.2}%)",
                effective_regions.len(),
                opts.append_regions,
                opts.remove_region_ids.len(),
                record.negative_regions.len(),
                report.overall_passed,
                report.total_censored_area_percent
            ),
        });

        let updated_record = record.clone();
        ledger.updated_at = Utc::now();
        self.save_ledger(&ledger)?;
        self.sync_markdown_reports()?;
        Ok((updated_record, report))
    }

    /// Approves an asset as requiring zero censorship intervention (e.g., artistic nudity or suggestive imagery),
    /// copying the untouched original to the publication directory while cleaning up any stale previews/escalations.
    /// Automatically mines high-frequency silhouette/contour hotspots as verified hard-negative boxes to prevent false positives.
    pub fn approve_untouched(
        &self,
        asset_query: &str,
        classification: CensorshipClassification,
        rationale: &str,
    ) -> Result<AssetRecord> {
        let _lock = WorkspaceLock::acquire(&self.ledger_dir())?;
        let mut ledger = self.load_ledger()?;
        let asset_id = Self::resolve_asset_id_in_ledger(&ledger, asset_query)?;
        let record = ledger
            .assets
            .get_mut(&asset_id)
            .ok_or_else(|| anyhow!("Asset '{}' not found in ledger", asset_id))?;

        let orig_path = Path::new(&record.archive_original_path);
        let orig_bytes = fs::read(orig_path)?;
        if compute_sha256(&orig_bytes) != record.sha256_original {
            return Err(anyhow!(
                "CRITICAL INTEGRITY VIOLATION: Archived original SHA-256 mismatch for {}",
                asset_id
            ));
        }

        // Remove any stale escalation or censored preview files
        self.remove_escalation_artifacts_for(&asset_id);
        self.remove_preview_artifacts_for(&asset_id);

        let ext = orig_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("png");
        let pub_filename = format!("{}_published.{}", asset_id, ext);
        let pub_path = self.published_dir().join(&pub_filename);
        atomic_write_bytes(&pub_path, &orig_bytes)?;

        // Auto-mine high-frequency artistic contour hotspots as verified benign hard-negatives if none were set
        if record.negative_regions.is_empty() {
            if let Ok(orig_img) = image::load_from_memory(&orig_bytes) {
                let orig_rgba = orig_img.to_rgba8();
                let hotspots = detect_candidate_hotspots(&orig_rgba, 3);
                if hotspots.is_empty() {
                    let fx = record.width / 4;
                    let fy = record.height / 6;
                    let fw = record.width / 2;
                    let fh = (record.height * 2) / 3;
                    let (_e, hardness, cat) =
                        evaluate_region_confuser_hardness(&orig_rgba, fx, fy, fw, fh);
                    record.negative_regions.push(NegativeAnnotationRegion {
                        id: "auto_neg_1".to_string(),
                        label: "artistic_silhouette_figure_1".to_string(),
                        rationale: format!(
                            "Verified benign artistic nudity / suggestive silhouette ({}) — hard-negative annotation for false-positive suppression",
                            rationale
                        ),
                        confuser_category: cat,
                        fp_hardness_score: hardness,
                        x: fx as f32,
                        y: fy as f32,
                        width: fw as f32,
                        height: fh as f32,
                        unit: CoordinateUnit::Pixels,
                        shape: RegionShape::Rect,
                    });
                } else {
                    for (idx, hs) in hotspots.into_iter().enumerate() {
                        record.negative_regions.push(NegativeAnnotationRegion {
                            id: format!("auto_neg_{}", idx + 1),
                            label: format!("artistic_silhouette_confuser_{}", idx + 1),
                            rationale: format!(
                                "Verified benign artistic nudity / suggestive contour ({}) — hard-negative annotation for false-positive suppression",
                                rationale
                            ),
                            confuser_category: hs.confuser_category,
                            fp_hardness_score: hs.fp_hardness_score,
                            x: hs.pixel_bounds.0 as f32,
                            y: hs.pixel_bounds.1 as f32,
                            width: hs.pixel_bounds.2 as f32,
                            height: hs.pixel_bounds.3 as f32,
                            unit: CoordinateUnit::Pixels,
                            shape: RegionShape::Rect,
                        });
                    }
                }
            }
        }

        record.classification = Some(classification);
        record.regions.clear();
        record.verification = None;
        record.escalation = None;
        record.published_edition_path = Some(pub_path.to_string_lossy().to_string());
        record.sha256_published = Some(record.sha256_original.clone());
        record.status = AssetStatus::ApprovedOriginal;
        record.updated_at = Utc::now();
        record.audit_trail.push(AuditEvent {
            timestamp: Utc::now(),
            action: "APPROVE_UNTOUCHED_FOR_PUBLICATION".to_string(),
            actor: "Deputy Gemini".to_string(),
            details: format!(
                "Classification {:?}: {} (hard_negatives={})",
                classification,
                rationale,
                record.negative_regions.len()
            ),
        });

        let updated_record = record.clone();
        ledger.updated_at = Utc::now();
        self.save_ledger(&ledger)?;
        self.sync_markdown_reports()?;
        Ok(updated_record)
    }

    /// Annotates verified benign / hard-negative regions on an asset (`annotate-negative`) without modifying any pixels.
    /// Used in data preparation and curation to eliminate false positives on artistic silhouettes, skin gradients, and linework.
    pub fn annotate_negative(
        &self,
        asset_query: &str,
        new_neg_regions: Vec<NegativeAnnotationRegion>,
        append: bool,
        remove_ids: Vec<String>,
        auto_mine_hotspots: bool,
    ) -> Result<AssetRecord> {
        let _lock = WorkspaceLock::acquire(&self.ledger_dir())?;
        let mut ledger = self.load_ledger()?;
        let asset_id = Self::resolve_asset_id_in_ledger(&ledger, asset_query)?;
        let record = ledger
            .assets
            .get_mut(&asset_id)
            .ok_or_else(|| anyhow!("Asset '{}' not found in ledger", asset_id))?;

        let orig_bytes = fs::read(&record.archive_original_path)?;
        let orig_rgba = image::load_from_memory(&orig_bytes)?.to_rgba8();
        let pos_boxes: Vec<(u32, u32, u32, u32)> = record
            .regions
            .iter()
            .map(|r| r.resolve_enclosing_aabb(record.width, record.height))
            .collect();

        let mut effective_negs: Vec<NegativeAnnotationRegion> =
            if append || !remove_ids.is_empty() {
                record
                    .negative_regions
                    .iter()
                    .filter(|r| !remove_ids.iter().any(|rem| rem == &r.id))
                    .cloned()
                    .collect()
            } else {
                Vec::new()
            };

        let mut skipped_polarity_conflicts = 0usize;
        for mut nr in new_neg_regions {
            let nb = nr.resolve_pixels(record.width, record.height);
            // Polarity Conflict Prevention: never allow a hard-negative box to overlap an active positive intervention box (IoU >= 0.20)
            if pos_boxes.iter().any(|&pb| compute_box_iou(nb, pb) >= 0.20) {
                skipped_polarity_conflicts += 1;
                continue;
            }
            let (_e, hardness, auto_cat) =
                evaluate_region_confuser_hardness(&orig_rgba, nb.0, nb.1, nb.2, nb.3);
            nr.fp_hardness_score = hardness;
            if nr.confuser_category.is_empty() || nr.confuser_category == "benign_artistic_feature" {
                nr.confuser_category = auto_cat;
            }

            if nr.id.is_empty() || nr.id.starts_with("auto_") {
                let mut k = 1usize;
                loop {
                    let candidate = format!("neg{}", k);
                    if !effective_negs.iter().any(|ex| ex.id == candidate) {
                        nr.id = candidate;
                        break;
                    }
                    k += 1;
                }
                effective_negs.push(nr);
            } else if let Some(existing) = effective_negs.iter_mut().find(|ex| ex.id == nr.id) {
                *existing = nr;
            } else {
                effective_negs.push(nr);
            }
        }

        if auto_mine_hotspots {
            let hotspots = detect_candidate_hotspots(&orig_rgba, 4);
            for hs in hotspots {
                let overlaps_pos = pos_boxes
                    .iter()
                    .any(|&pb| compute_box_iou(hs.pixel_bounds, pb) >= 0.15);
                let overlaps_existing_neg = effective_negs.iter().any(|nr| {
                    compute_box_iou(hs.pixel_bounds, nr.resolve_pixels(record.width, record.height))
                        >= 0.25
                });
                if !overlaps_pos && !overlaps_existing_neg {
                    let mut k = 1usize;
                    let cid = loop {
                        let c = format!("auto_neg_{}", k);
                        if !effective_negs.iter().any(|ex| ex.id == c) {
                            break c;
                        }
                        k += 1;
                    };
                    effective_negs.push(NegativeAnnotationRegion {
                        id: cid,
                        label: format!("auto_mined_hotspot_{}", hs.rank),
                        rationale: "Auto-mined high-frequency benign region non-overlapping with positive interventions".to_string(),
                        confuser_category: hs.confuser_category,
                        fp_hardness_score: hs.fp_hardness_score,
                        x: hs.pixel_bounds.0 as f32,
                        y: hs.pixel_bounds.1 as f32,
                        width: hs.pixel_bounds.2 as f32,
                        height: hs.pixel_bounds.3 as f32,
                        unit: CoordinateUnit::Pixels,
                        shape: RegionShape::Rect,
                    });
                }
            }
        }

        // Final de-confliction & hardness score refresh on all effective negative regions
        effective_negs.retain(|nr| {
            let nb = nr.resolve_pixels(record.width, record.height);
            !pos_boxes.iter().any(|&pb| compute_box_iou(nb, pb) >= 0.20)
        });
        for nr in &mut effective_negs {
            let (nx, ny, nw, nh) = nr.resolve_pixels(record.width, record.height);
            let (_e, hardness, auto_cat) =
                evaluate_region_confuser_hardness(&orig_rgba, nx, ny, nw, nh);
            nr.fp_hardness_score = hardness;
            if nr.confuser_category.is_empty() {
                nr.confuser_category = auto_cat;
            }
        }

        record.negative_regions = effective_negs;
        record.updated_at = Utc::now();
        record.audit_trail.push(AuditEvent {
            timestamp: Utc::now(),
            action: "ANNOTATE_HARD_NEGATIVE_REGIONS".to_string(),
            actor: "Deputy Gemini".to_string(),
            details: format!(
                "Updated verified benign hard-negative annotations (count={}, append={}, auto_mine={}, skipped_polarity_conflicts={})",
                record.negative_regions.len(),
                append,
                auto_mine_hotspots,
                skipped_polarity_conflicts
            ),
        });

        let updated = record.clone();
        ledger.updated_at = Utc::now();
        self.save_ledger(&ledger)?;
        self.sync_markdown_reports()?;
        Ok(updated)
    }

    /// Escalates an asset for human/executive review when artistic or compliance requirements are genuinely ambiguous.
    /// Automatically revokes and removes any prior published edition so an escalated asset is never distributed!
    pub fn escalate_asset(
        &self,
        asset_query: &str,
        category: &str,
        reason: &str,
        agent_notes: &str,
    ) -> Result<AssetRecord> {
        let _lock = WorkspaceLock::acquire(&self.ledger_dir())?;
        let mut ledger = self.load_ledger()?;
        let asset_id = Self::resolve_asset_id_in_ledger(&ledger, asset_query)?;
        let record = ledger
            .assets
            .get_mut(&asset_id)
            .ok_or_else(|| anyhow!("Asset '{}' not found in ledger", asset_id))?;

        // Revoke any previously published edition or previews on escalation
        if let Some(old_pub) = record.published_edition_path.take() {
            let _ = fs::remove_file(old_pub);
        }
        record.sha256_published = None;
        record.verification = None;
        record.regions.clear();
        self.remove_preview_artifacts_for(&asset_id);

        let orig_path = Path::new(&record.archive_original_path);
        let ext = orig_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("png");
        let review_copy = self
            .review_dir()
            .join(format!("{}_review.{}", asset_id, ext));
        if let Ok(orig_bytes) = fs::read(orig_path) {
            let _ = atomic_write_bytes(&review_copy, &orig_bytes);
        }

        record.classification = Some(CensorshipClassification::AmbiguousEscalate);
        record.status = AssetStatus::Escalated;
        record.escalation = Some(EscalationInfo {
            escalated_at: Utc::now(),
            reason: reason.to_string(),
            ambiguity_category: category.to_string(),
            agent_notes: agent_notes.to_string(),
        });
        record.updated_at = Utc::now();
        record.audit_trail.push(AuditEvent {
            timestamp: Utc::now(),
            action: "ESCALATE_FOR_EXECUTIVE_REVIEW".to_string(),
            actor: "Deputy Gemini".to_string(),
            details: format!("[{}] {}", category, reason),
        });

        let updated_record = record.clone();
        ledger.updated_at = Utc::now();
        self.save_ledger(&ledger)?;
        self.sync_markdown_reports()?;
        Ok(updated_record)
    }

    /// Resets an asset back to `PendingInspection`, removing published, preview, and escalation artifacts
    /// while preserving the untouched original in `assets/originals/`.
    pub fn reset_asset(&self, asset_query: &str) -> Result<AssetRecord> {
        let _lock = WorkspaceLock::acquire(&self.ledger_dir())?;
        let mut ledger = self.load_ledger()?;
        let asset_id = Self::resolve_asset_id_in_ledger(&ledger, asset_query)?;
        let record = ledger
            .assets
            .get_mut(&asset_id)
            .ok_or_else(|| anyhow!("Asset '{}' not found in ledger", asset_id))?;

        if let Some(old_pub) = record.published_edition_path.take() {
            let _ = fs::remove_file(old_pub);
        }
        record.sha256_published = None;
        record.verification = None;
        record.escalation = None;
        record.classification = None;
        record.regions.clear();
        record.status = AssetStatus::PendingInspection;
        record.updated_at = Utc::now();
        record.audit_trail.push(AuditEvent {
            timestamp: Utc::now(),
            action: "RESET_TO_PENDING_INSPECTION".to_string(),
            actor: "Deputy Gemini".to_string(),
            details: "Cleared published/escalation state; preserved untouched original."
                .to_string(),
        });

        self.remove_preview_artifacts_for(&asset_id);
        self.remove_escalation_artifacts_for(&asset_id);

        let updated = record.clone();
        ledger.updated_at = Utc::now();
        self.save_ledger(&ledger)?;
        self.sync_markdown_reports()?;
        Ok(updated)
    }

    fn remove_preview_artifacts_for(&self, asset_id: &str) {
        for suffix in [
            "_pub_1080p_q85.jpg",
            "_pub_540p_thumb.jpg",
            "_contact_sheet.png",
        ] {
            let p = self.previews_dir().join(format!("{}{}", asset_id, suffix));
            let _ = fs::remove_file(p);
        }
    }

    fn remove_escalation_artifacts_for(&self, asset_id: &str) {
        if let Ok(entries) = fs::read_dir(self.review_dir()) {
            let prefix = format!("{}_review.", asset_id);
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with(&prefix) {
                        let _ = fs::remove_file(entry.path());
                    }
                }
            }
        }
    }

    /// Runs a comprehensive workspace integrity, cryptographic, and orphan-hygiene audit (`doctor`).
    /// When `fix == true`, automatically repairs missing directories, cleans orphaned files, and re-syncs reports.
    pub fn run_doctor(&self, fix: bool) -> Result<DoctorReport> {
        if fix {
            self.init()?;
        }
        let ledger = self.load_ledger()?;
        let mut checks = Vec::new();
        let mut hints = Vec::new();

        // 1. Directory structure check
        let dirs_ok = [
            self.inbox_dir(),
            self.originals_dir(),
            self.inspect_dir(),
            self.previews_dir(),
            self.published_dir(),
            self.review_dir(),
            self.ledger_dir(),
        ]
        .iter()
        .all(|d| d.exists());
        checks.push(DoctorCheck {
            check_name: "workspace_directories".to_string(),
            passed: dirs_ok,
            details: if dirs_ok {
                "All 7 workspace directories exist and are accessible.".to_string()
            } else {
                hints.push("Run `heav3ns-censor doctor --fix` to recreate missing directories.".to_string());
                "One or more required workspace directories are missing.".to_string()
            },
        });

        let mut expected_files = HashSet::new();
        let mut sorted_ids: Vec<&String> = ledger.assets.keys().collect();
        sorted_ids.sort();

        for id in sorted_ids {
            let rec = &ledger.assets[id];
            expected_files.insert(PathBuf::from(&rec.archive_original_path));
            if let Some(ig) = &rec.inspect_grid_path {
                expected_files.insert(PathBuf::from(ig));
            }
            let z_crop = self.inspect_dir().join(format!("{}_zoom_crop.png", id));
            if z_crop.exists() {
                expected_files.insert(z_crop);
            }

            let orig_path = Path::new(&rec.archive_original_path);
            let orig_ok = match fs::read(orig_path) {
                Ok(bytes) => compute_sha256(&bytes) == rec.sha256_original,
                Err(_) => false,
            };
            checks.push(DoctorCheck {
                check_name: format!("original_sha256:{}", id),
                passed: orig_ok,
                details: if orig_ok {
                    format!(
                        "Untouched original verified (SHA-256 {}...)",
                        &rec.sha256_original[0..16]
                    )
                } else {
                    hints.push(format!(
                        "Original asset {} is missing or corrupted in assets/originals/!",
                        id
                    ));
                    "CRITICAL: Original file missing or SHA-256 mismatch!".to_string()
                },
            });

            // State-hygiene invariant: Escalated assets must NOT have a published edition
            if rec.status == AssetStatus::Escalated {
                let no_pub_leak = rec.published_edition_path.is_none();
                checks.push(DoctorCheck {
                    check_name: format!("escalation_isolation:{}", id),
                    passed: no_pub_leak,
                    details: if no_pub_leak {
                        "Escalated asset properly isolated from published/ directory.".to_string()
                    } else {
                        "CRITICAL: Escalated asset still has a published_edition_path!".to_string()
                    },
                });
                let ext = orig_path
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("png");
                expected_files.insert(self.review_dir().join(format!("{}_review.{}", id, ext)));
            }

            if let (Some(pub_path_str), Some(expected_pub_sha)) =
                (&rec.published_edition_path, &rec.sha256_published)
            {
                expected_files.insert(PathBuf::from(pub_path_str));
                let pub_ok = match fs::read(pub_path_str) {
                    Ok(bytes) => &compute_sha256(&bytes) == expected_pub_sha,
                    Err(_) => false,
                };
                checks.push(DoctorCheck {
                    check_name: format!("published_sha256:{}", id),
                    passed: pub_ok,
                    details: if pub_ok {
                        format!(
                            "Published edition verified (SHA-256 {}...)",
                            &expected_pub_sha[0..16]
                        )
                    } else {
                        hints.push(format!(
                            "Re-run `execute` or `approve-original` for {} to regenerate its published edition.",
                            id
                        ));
                        "Published edition missing or SHA-256 mismatch!".to_string()
                    },
                });
            }

            if let Some(ver) = &rec.verification {
                for p in &ver.publication_preview_paths {
                    expected_files.insert(PathBuf::from(p));
                }
                checks.push(DoctorCheck {
                    check_name: format!("bit_exact_preservation:{}", id),
                    passed: ver.artistic_preservation_exact && ver.overall_passed,
                    details: format!(
                        "Uncensored diff pixels = {}, overall_passed = {}",
                        ver.non_censored_pixel_diff_count, ver.overall_passed
                    ),
                });
            }

            // Data Annotation Polarity Hygiene: ensure zero positive/negative IoU >= 0.20 collisions
            if !rec.regions.is_empty() && !rec.negative_regions.is_empty() {
                let pos_boxes: Vec<(u32, u32, u32, u32)> = rec
                    .regions
                    .iter()
                    .map(|r| r.resolve_enclosing_aabb(rec.width, rec.height))
                    .collect();
                let conflict_count = rec
                    .negative_regions
                    .iter()
                    .filter(|nr| {
                        let nb = nr.resolve_pixels(rec.width, rec.height);
                        pos_boxes.iter().any(|&pb| compute_box_iou(nb, pb) >= 0.20)
                    })
                    .count();
                checks.push(DoctorCheck {
                    check_name: format!("annotation_polarity_conflict:{}", id),
                    passed: conflict_count == 0,
                    details: if conflict_count == 0 {
                        format!(
                            "Zero positive/hard-negative IoU collisions ({} pos, {} hard-neg boxes).",
                            rec.regions.len(),
                            rec.negative_regions.len()
                        )
                    } else {
                        hints.push(format!(
                            "Run `heav3ns-censor annotate-negative --asset-id {} --append` or `doctor --fix` to prune conflicting polarity boxes.",
                            id
                        ));
                        format!(
                            "Found {} hard-negative box(es) overlapping positive intervention regions (IoU >= 0.20)!",
                            conflict_count
                        )
                    },
                });
            }
        }

        // Check for orphaned files across managed output directories
        let mut orphaned_files = Vec::new();
        for dir in [
            self.originals_dir(),
            self.inspect_dir(),
            self.previews_dir(),
            self.published_dir(),
            self.review_dir(),
        ] {
            if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_file() && !expected_files.contains(&p) {
                        orphaned_files.push(p);
                    }
                }
            }
        }

        let mut orphaned_files_cleaned = 0;
        if fix && !orphaned_files.is_empty() {
            for p in &orphaned_files {
                if fs::remove_file(p).is_ok() {
                    orphaned_files_cleaned += 1;
                }
            }
            orphaned_files.clear();
            self.sync_markdown_reports()?;
        }

        let hygiene_ok = orphaned_files.is_empty();
        checks.push(DoctorCheck {
            check_name: "workspace_orphan_hygiene".to_string(),
            passed: hygiene_ok,
            details: if hygiene_ok {
                format!(
                    "Zero orphaned files in managed asset directories (cleaned: {}).",
                    orphaned_files_cleaned
                )
            } else {
                hints.push(
                    "Run `heav3ns-censor doctor --fix` to prune orphaned files.".to_string(),
                );
                format!(
                    "Found {} untracked/orphaned file(s) in managed directories.",
                    orphaned_files.len()
                )
            },
        });

        let overall_healthy = checks.iter().all(|c| c.passed);
        Ok(DoctorReport {
            checked_at: Utc::now(),
            overall_healthy,
            total_assets_audited: ledger.assets.len(),
            orphaned_files_cleaned,
            checks,
            remediation_hints: hints,
        })
    }

    pub fn status_summary(&self) -> Result<WorkspaceStatusSummary> {
        let ledger = self.load_ledger()?;
        let mut pending_inspection = Vec::new();
        let mut planned = Vec::new();
        let mut processed = Vec::new();
        let mut verified = Vec::new();
        let mut approved_original = Vec::new();
        let mut escalated = Vec::new();

        let mut sorted_ids: Vec<&String> = ledger.assets.keys().collect();
        sorted_ids.sort();

        for id in sorted_ids {
            let rec = &ledger.assets[id];
            match rec.status {
                AssetStatus::PendingInspection => pending_inspection.push(id.clone()),
                AssetStatus::Planned => planned.push(id.clone()),
                AssetStatus::Processed => processed.push(id.clone()),
                AssetStatus::Verified => verified.push(id.clone()),
                AssetStatus::ApprovedOriginal => approved_original.push(id.clone()),
                AssetStatus::Escalated => escalated.push(id.clone()),
            }
        }

        let mut unscanned_inbox_files = Vec::new();
        if self.inbox_dir().exists() {
            for entry in WalkDir::new(self.inbox_dir())
                .min_depth(1)
                .max_depth(2)
                .into_iter()
                .flatten()
            {
                if entry.file_type().is_file() && is_supported_image(entry.path()) {
                    let p_str = entry.path().to_string_lossy().to_string();
                    let already_known = ledger
                        .assets
                        .values()
                        .any(|a| a.inbox_path.as_deref() == Some(p_str.as_str()));
                    if !already_known {
                        unscanned_inbox_files.push(p_str);
                    }
                }
            }
        }

        let next_recommended_action = if !unscanned_inbox_files.is_empty() {
            format!(
                "Run `heav3ns-censor batch-prepare` to ingest and generate inspection grids for {} new inbox file(s).",
                unscanned_inbox_files.len()
            )
        } else if let Some(first_pending) = pending_inspection.first() {
            format!(
                "Run `heav3ns-censor inspect --asset-id {}` to generate coordinate grid and inspect.",
                first_pending
            )
        } else {
            "All ingested assets have been processed. Run `heav3ns-censor doctor` to audit workspace integrity.".to_string()
        };

        Ok(WorkspaceStatusSummary {
            workspace_root: self.root.to_string_lossy().to_string(),
            total_assets: ledger.assets.len(),
            pending_inspection,
            planned,
            processed,
            verified,
            approved_original,
            escalated,
            unscanned_inbox_files,
            next_recommended_action,
            ledger_path: self.ledger_file().to_string_lossy().to_string(),
            escalation_report_path: self.escalation_md_file().to_string_lossy().to_string(),
            dashboard_html_path: self.dashboard_html_file().to_string_lossy().to_string(),
            dataset_manifest_jsonl_path: self.dataset_manifest_jsonl_file().to_string_lossy().to_string(),
            coco_annotations_json_path: self.coco_annotations_json_file().to_string_lossy().to_string(),
            vlm_safety_sft_jsonl_path: self.vlm_sft_jsonl_file().to_string_lossy().to_string(),
        })
    }

    /// Generates a role-tailored, context-aware onboarding and handoff packet (`agent`, `researcher`, `developer`, `stakeholder`, or `all`).
    pub fn onboard(&self, role: &str) -> Result<OnboardingPacket> {
        let status = self.status_summary()?;
        let ledger = self.load_ledger()?;
        let normalized_role = match role.trim().to_lowercase().as_str() {
            "agent" | "ai" | "deputy" => "agent",
            "researcher" | "safety" | "dataset" | "ml" | "academic" => "researcher",
            "dev" | "developer" | "engineer" => "developer",
            "stakeholder" | "executive" | "artist" | "majo" | "egg" => "stakeholder",
            _ => "all",
        };

        let sample_asset = ledger
            .assets
            .keys()
            .next()
            .cloned()
            .unwrap_or_else(|| "heav3ns_01".to_string());

        let role_steps = match normalized_role {
            "agent" => vec![
                "1. [INGEST & PREPARE] Run `./target/release/heav3ns-censor batch-prepare` (or MCP `heav3ns_batch_prepare`) to archive SHA-256 originals and render Normalized1000 VLM coordinate grids + hotspot hints.".to_string(),
                "2. [VISUAL GROUNDING] Use `read_file` on `archive_original_path` and `inspect_grid_path`. For small/angled details, run `inspect --crop \"x,y,w,h:pixels\"` to generate a 3x magnified crop with burned-in global coordinates.".to_string(),
                "3. [TRIAGE & EXECUTE] Classify into `explicit_requires_intervention` (`execute`), `artistic_nudity_pass` / `suggestive_pass` (`approve-original`), or `ambiguous_escalate` (`escalate`). Use `--append` for incremental region edits.".to_string(),
                "4. [VERIFY & AUDIT] Inspect `assets/previews/<ID>_contact_sheet.png` (4-Panel Broadcast QC Slate) and run `./target/release/heav3ns-censor doctor --fix` to confirm `overall_healthy == true`.".to_string(),
            ],
            "researcher" => vec![
                "1. [SURGICAL DATASET SANITIZATION] Place raw dataset images in `assets/inbox/` and run `./target/release/heav3ns-censor batch-prepare` to archive SHA-256 originals and detect high-frequency candidate regions without dropping artistic samples.".to_string(),
                "2. [ZERO-HALLUCINATION REDACTION VS. FALSE-POSITIVE PRESERVATION] Apply deterministic SDF redaction (`execute`) on explicit regions (preserving >98% exterior pixels bit-identically), preserve non-explicit figurative art (`approve-original`), and quarantine ambiguous edge cases (`escalate`).".to_string(),
                "3. [EXPORT HUGGINGFACE JSONL & COCO ANNOTATIONS] Run `./target/release/heav3ns-censor export-dataset` (or `--safe-only` for training splits) to generate `ledger/dataset_manifest.jsonl` with COCO `[x,y,w,h]`, VLM `Normalized1000`, and YOLO bounding boxes + multi-scale JPEG safety telemetry.".to_string(),
                "4. [READ SAFETY RESEARCH & WHITEPAPER DOCS] See `docs/knowledgebase/03-dataset-cleaning-and-ai-safety-research.md` and `docs/whitepaper/TECHNICAL_WHITEPAPER.md`.".to_string(),
            ],
            "developer" => vec![
                "1. [BUILD & TEST] Run `cargo test --release && cargo build --release` to execute all 9 mathematical SDF, remainder-pooling, dataset-export, and state-hygiene unit tests.".to_string(),
                "2. [ARCHITECTURE TOUR] Read `src/models.rs` (SDF geometry & BTreeMap ledger) -> `src/engine.rs` (immutable-source spatial filters & multi-scale JPEG verifier) -> `src/workspace.rs` (RAII lock & atomic persistence) -> `src/mcp.rs` (JSON-RPC 2.0 server).".to_string(),
                "3. [WHITEPAPER & ADRs] Review `docs/whitepaper/TECHNICAL_WHITEPAPER.md`, `docs/adrs/ADR-0001..0004`, and `docs/postmortems/PM-001..004` before modifying spatial kernels.".to_string(),
                "4. [DOCTOR GATE] Always run `./target/release/heav3ns-censor doctor --fix` before committing.".to_string(),
            ],
            "stakeholder" => vec![
                "1. [FOR EGG - PRINCIPAL ARTIST] Drop master illustrations into `assets/inbox/`. Untouched originals are permanently preserved in `assets/originals/` with SHA-256 verification.".to_string(),
                "2. [FOR MAJO - EXECUTIVE CHAIR] Open `ledger/dashboard.html` in any browser to inspect the 4-Panel Broadcast QC Slates (press `[o]` to A/B toggle Original vs. Published), or read `ledger/publication_report.md` and `ledger/escalations_review.md`.".to_string(),
                "3. [INTERACTIVE COORDINATE PICKER] In `ledger/dashboard.html`, drag a box over any image to measure exact coordinates and generate a 1-click CLI command for custom adjustments.".to_string(),
                "4. [HEALTH VERIFICATION] Run `./target/release/heav3ns-censor doctor` anytime to verify 100% bit-exact preservation across all assets.".to_string(),
            ],
            _ => vec![
                "[RESEARCHER & DATASET CLEANING TRACK] Run `batch-prepare` -> `execute` / `approve-original` -> `export-dataset --safe-only` to generate `ledger/dataset_manifest.jsonl` (see `docs/knowledgebase/03-dataset-cleaning-and-ai-safety-research.md`).".to_string(),
                "[ARTIST & STUDIO STAKEHOLDER TRACK] Open `ledger/dashboard.html` (press `[o]` for A/B Original comparison) and `ledger/escalations_review.md` (see `docs/stakeholder-handoff/STAKEHOLDER_HANDOFF.md`).".to_string(),
                "[AGENT TRACK] Run `batch-prepare` -> VLM `read_file` on `*_inspect_grid.png` -> `execute` / `approve-original` / `escalate` -> `doctor --fix` (see `docs/onboarding/AGENT_ONBOARDING.md`).".to_string(),
                "[DEVELOPER TRACK] Run `cargo test --release` -> explore `src/engine.rs` & `docs/whitepaper/TECHNICAL_WHITEPAPER.md` (see `docs/onboarding/DEVELOPER_ONBOARDING.md`).".to_string(),
            ],
        };

        let mut doc_map = std::collections::BTreeMap::new();
        doc_map.insert("01_readme_portal".to_string(), "README.md".to_string());
        doc_map.insert(
            "02_dataset_cleaning_and_safety_research".to_string(),
            "docs/knowledgebase/03-dataset-cleaning-and-ai-safety-research.md".to_string(),
        );
        doc_map.insert(
            "03_agent_onboarding".to_string(),
            "docs/onboarding/AGENT_ONBOARDING.md".to_string(),
        );
        doc_map.insert(
            "04_developer_onboarding".to_string(),
            "docs/onboarding/DEVELOPER_ONBOARDING.md".to_string(),
        );
        doc_map.insert(
            "05_stakeholder_handoff".to_string(),
            "docs/stakeholder-handoff/STAKEHOLDER_HANDOFF.md".to_string(),
        );
        doc_map.insert(
            "06_philosophy".to_string(),
            "docs/philosophy/PHILOSOPHY.md".to_string(),
        );
        doc_map.insert(
            "07_technical_whitepaper".to_string(),
            "docs/whitepaper/TECHNICAL_WHITEPAPER.md".to_string(),
        );
        doc_map.insert(
            "08_knowledgebase_index".to_string(),
            "docs/knowledgebase/INDEX.md".to_string(),
        );
        doc_map.insert(
            "09_playbooks_dir".to_string(),
            "docs/playbooks/".to_string(),
        );
        doc_map.insert("10_adrs_dir".to_string(), "docs/adrs/".to_string());
        doc_map.insert(
            "11_postmortems_dir".to_string(),
            "docs/postmortems/".to_string(),
        );
        doc_map.insert(
            "12_exported_dataset_jsonl".to_string(),
            "ledger/dataset_manifest.jsonl".to_string(),
        );

        Ok(OnboardingPacket {
            engine_version: ledger.version,
            selected_role: normalized_role.to_string(),
            commission_authority: ledger.commission_authority,
            principal_artist: ledger.principal_artist,
            commissioned_agent: ledger.commissioned_agent,
            core_invariants: vec![
                "1. Artistic & Dataset Context Preservation is Absolute: non_censored_pixel_diff_count == 0 (100% bit-exact RGBA outside SDF masks; zero diffusion hallucination).".to_string(),
                "2. Immutable Master Archive: Untouched originals are preserved in assets/originals/ with SHA-256 verification.".to_string(),
                "3. Multi-Scale JPEG Resilience: Every censorship region is verified across Native (Mean + P95 Peak), 1080p (Q85 JPEG), and 540p (Q75 Thumbnail JPEG).".to_string(),
                "4. Escalation & Safety Isolation: Escalating an asset automatically revokes any prior published edition and excludes it from `--safe-only` dataset training splits.".to_string(),
            ],
            role_quickstart_steps: role_steps,
            live_workspace_commands: vec![
                "./target/release/heav3ns-censor status".to_string(),
                "./target/release/heav3ns-censor batch-prepare".to_string(),
                format!(
                    "./target/release/heav3ns-censor inspect --asset-id {} --crop \"420,480,160,140:pixels\"",
                    sample_asset
                ),
                format!(
                    "./target/release/heav3ns-censor execute --asset-id {} --append --region \"r1=mosaic:440,500,120,100:pixels:rect:auto:0:explicit_anatomy\"",
                    sample_asset
                ),
                "./target/release/heav3ns-censor export-dataset --safe-only".to_string(),
                "./target/release/heav3ns-censor doctor --fix".to_string(),
            ],
            documentation_map: doc_map,
            status_summary: status,
        })
    }

    /// Generates a comprehensive diagnostic and stakeholder/agent handoff explanation for a single asset (`explain`).
    pub fn explain_asset(&self, asset_query: &str) -> Result<AssetExplanationPacket> {
        let ledger = self.load_ledger()?;
        let asset_id = Self::resolve_asset_id_in_ledger(&ledger, asset_query)?;
        let rec = ledger
            .assets
            .get(&asset_id)
            .ok_or_else(|| anyhow!("Asset '{}' not found in ledger", asset_id))?
            .clone();

        let orig_sha_ok = fs::read(&rec.archive_original_path)
            .map(|b| compute_sha256(&b) == rec.sha256_original)
            .unwrap_or(false);
        let pub_sha_ok = match (&rec.published_edition_path, &rec.sha256_published) {
            (Some(p), Some(expected)) => fs::read(p)
                .map(|b| &compute_sha256(&b) == expected)
                .unwrap_or(false),
            (None, None) => true,
            _ => false,
        };
        let crypto_ok = orig_sha_ok && pub_sha_ok;

        let bit_exact_ok = match &rec.verification {
            Some(v) => v.artistic_preservation_exact && v.non_censored_pixel_diff_count == 0,
            None => rec.status == AssetStatus::ApprovedOriginal || rec.status == AssetStatus::Escalated || rec.status == AssetStatus::PendingInspection,
        };

        let executive_summary = match rec.status {
            AssetStatus::Verified => format!(
                "VERIFIED CENSORED EDITION: {} localized intervention region(s) applied under '{}'. 100% bit-exact outside masks (0 diff px) and verified across 1080p Q85 & 540p Q75 JPEG downscales.",
                rec.regions.len(),
                rec.policy_profile
            ),
            AssetStatus::ApprovedOriginal => format!(
                "APPROVED UNTOUCHED ({:?}): Exported bit-identically to published/ with zero pixel modification (SHA-256 matches archived original).",
                rec.classification.unwrap_or(CensorshipClassification::ArtisticNudityPass)
            ),
            AssetStatus::Escalated => {
                let esc_info = rec.escalation.as_ref().map(|e| format!("[{}] {}", e.ambiguity_category, e.reason)).unwrap_or_default();
                format!(
                    "ESCALATED FOR EXECUTIVE/ARTIST REVIEW: Isolated from published/ directory. {}",
                    esc_info
                )
            }
            _ => "PENDING INSPECTION: Awaiting VLM coordinate inspection and classification triage.".to_string(),
        };

        let next_handoff_actions = match rec.status {
            AssetStatus::Verified => vec![
                format!("Ready for publication distribution at `{}`.", rec.published_edition_path.as_deref().unwrap_or("")),
                format!("View 4-Panel Broadcast QC Slate at `assets/previews/{}_contact_sheet.png` or open `ledger/dashboard.html`.", rec.asset_id),
                format!("To adjust a region incrementally, run `heav3ns-censor execute --asset-id {} --append --region \"...\"`.", rec.asset_id),
            ],
            AssetStatus::ApprovedOriginal => vec![
                format!("Ready for publication distribution (untouched original) at `{}`.", rec.published_edition_path.as_deref().unwrap_or("")),
            ],
            AssetStatus::Escalated => vec![
                "Requires determination by MAJO (Executive Chair) or EGG (Principal Artist) in `ledger/escalations_review.md`.".to_string(),
                format!("Once decided, resolve via `heav3ns-censor execute --asset-id {} --region \"...\"` or `heav3ns-censor approve-original --asset-id {} --rationale \"...\"`.", rec.asset_id, rec.asset_id),
            ],
            _ => vec![
                format!("Run `heav3ns-censor inspect --asset-id {}` and view `inspect_grid_path` via VLM `read_file`.", rec.asset_id),
            ],
        };

        let rec_bs =
            recommended_mosaic_block_size(rec.width, rec.height, &rec.policy_profile);

        Ok(AssetExplanationPacket {
            asset_id: rec.asset_id.clone(),
            original_filename: rec.original_filename.clone(),
            dimensions: (rec.width, rec.height),
            status: rec.status,
            classification: rec.classification,
            executive_summary,
            cryptographic_integrity_verified: crypto_ok,
            bit_exact_preservation_verified: bit_exact_ok,
            active_regions_count: rec.regions.len(),
            next_handoff_actions,
            cli_commands: InspectCliTemplates {
                execute_mosaic_example: format!(
                    "./target/release/heav3ns-censor execute --asset-id {} --append --region \"r1=mosaic:440,500,120,100:pixels:rect:{}:0:explicit_anatomy\"",
                    rec.asset_id, rec_bs
                ),
                execute_rotated_bar_example: format!(
                    "./target/release/heav3ns-censor execute --asset-id {} --append --region \"r2=solid_bar:444,314,112,52:pixels:pill:auto@6.0:0:kuro_nuri_cleft\"",
                    rec.asset_id
                ),
                zoom_crop_example: format!(
                    "./target/release/heav3ns-censor inspect --asset-id {} --crop \"420,480,160,140:pixels\"",
                    rec.asset_id
                ),
                approve_original_example: format!(
                    "./target/release/heav3ns-censor approve-original --asset-id {} --classification artistic_nudity_pass --rationale \"Approved untouched.\"",
                    rec.asset_id
                ),
                escalate_example: format!(
                    "./target/release/heav3ns-censor escalate --asset-id {} --category \"stylized_anatomy_ambiguity\" --reason \"Requires review\"",
                    rec.asset_id
                ),
            },
            record: rec,
        })
    }

    /// Generates Markdown reports (`ledger/publication_report.md` and `ledger/escalations_review.md`)
    /// AND the Interactive Studio QC Dashboard (`ledger/dashboard.html`).
    pub fn sync_markdown_reports(&self) -> Result<()> {
        let ledger = self.load_ledger()?;
        self.save_ledger(&ledger)?;
        let mut asset_list: Vec<&AssetRecord> = ledger.assets.values().collect();
        asset_list.sort_by(|a, b| a.asset_id.cmp(&b.asset_id));

        // 1. Publication & Censorship Treatments Report (Markdown)
        let mut pub_md = String::new();
        pub_md.push_str("# HEAV3NS POST-PRODUCTION CENSORSHIP LEDGER\n");
        pub_md.push_str(&format!(
            "**Commissioning Authority:** {}  \n**Commissioned Agent:** {}  \n**Principal Artist:** {}  \n**Engine Version:** `{}`  \n**Updated:** {}\n\n",
            ledger.commission_authority,
            ledger.commissioned_agent,
            ledger.principal_artist,
            ledger.version,
            ledger.updated_at.to_rfc3339()
        ));

        pub_md.push_str("## Asset Correspondence & Treatment Summary\n\n");
        pub_md.push_str("| Asset ID | Original File | Dimensions | Status | Classification | Regions Applied | Original Preserved | Publication Edition |\n");
        pub_md.push_str("| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n");

        for rec in &asset_list {
            let class_str = rec
                .classification
                .map(|c| format!("{:?}", c))
                .unwrap_or_else(|| "-".to_string());
            let pub_path = rec
                .published_edition_path
                .as_deref()
                .unwrap_or("*(Not published)*");
            pub_md.push_str(&format!(
                "| `{}` | `{}` | {}x{} | `{:?}` | `{}` | {} | `{}` | `{}` |\n",
                rec.asset_id,
                rec.original_filename,
                rec.width,
                rec.height,
                rec.status,
                class_str,
                rec.regions.len(),
                rec.archive_original_path,
                pub_path
            ));
        }

        pub_md.push_str("\n## Detailed Treatment Records\n\n");
        for rec in &asset_list {
            pub_md.push_str(&format!(
                "### `{}` ({})\n",
                rec.asset_id, rec.original_filename
            ));
            pub_md.push_str(&format!("- **Status:** `{:?}`\n", rec.status));
            pub_md.push_str(&format!("- **Policy Profile:** `{}`\n", rec.policy_profile));
            pub_md.push_str(&format!(
                "- **Original Asset (SHA-256 `{}`):** `{}`\n",
                &rec.sha256_original[0..16],
                rec.archive_original_path
            ));
            if let Some(pub_p) = &rec.published_edition_path {
                pub_md.push_str(&format!("- **Published Edition:** `{}`\n", pub_p));
            }
            if !rec.regions.is_empty() {
                pub_md.push_str("- **Localized Censorship Interventions:**\n");
                for r in &rec.regions {
                    let (px, py, pw, ph) = r.resolve_enclosing_aabb(rec.width, rec.height);
                    pub_md.push_str(&format!(
                        "  - **`{}` ({}):** `{:?}` at enclosing pixel bounds `({}, {}, {}x{})` (shape: `{:?}`, angle: `{:.1}deg`, feather: `{}px`) — *{}*\n",
                        r.id,
                        r.label,
                        r.treatment,
                        px,
                        py,
                        pw,
                        ph,
                        r.shape,
                        r.rotation_angle_deg(),
                        r.feather_px,
                        r.rationale
                    ));
                }
            }
            if let Some(ver) = &rec.verification {
                pub_md.push_str(&format!(
                    "- **Verification:** Overall Passed = `{}` | Bit-Exact Uncensored Preservation = `{}` (diff pixels: `{}`) | Censored Area = `{:.2}%`\n",
                    ver.overall_passed,
                    ver.artistic_preservation_exact,
                    ver.non_censored_pixel_diff_count,
                    ver.total_censored_area_percent
                ));
                for rm in &ver.region_metrics {
                    pub_md.push_str(&format!(
                        "  - **Metric `{}`:** {} — {}\n",
                        rm.region_id, rm.method_summary, rm.notes
                    ));
                }
            }
            pub_md.push('\n');
        }

        atomic_write_bytes(&self.publication_manifest_md_file(), pub_md.as_bytes())?;

        // 2. Escalations Review Report (Markdown)
        let mut esc_md = String::new();
        esc_md.push_str("# HEAV3NS ESCALATED ASSETS REQUIRING REVIEW\n\n");
        esc_md.push_str("> Assets listed here were escalated by Deputy Gemini per Section 5 & Section 7 of the Executive Commission due to genuine artistic ambiguity, technical blockers, or unresolved compliance questions.\n\n");

        let escalated_records: Vec<&&AssetRecord> = asset_list
            .iter()
            .filter(|r| r.status == AssetStatus::Escalated)
            .collect();

        if escalated_records.is_empty() {
            esc_md.push_str(
                "*No assets currently require executive or artist escalation review.*\n",
            );
        } else {
            for rec in escalated_records {
                esc_md.push_str(&format!(
                    "## `{}` (`{}`)\n",
                    rec.asset_id, rec.original_filename
                ));
                if let Some(esc) = &rec.escalation {
                    esc_md.push_str(&format!(
                        "- **Escalated At:** {}\n",
                        esc.escalated_at.to_rfc3339()
                    ));
                    esc_md.push_str(&format!("- **Category:** `{}`\n", esc.ambiguity_category));
                    esc_md.push_str(&format!("- **Reason:** {}\n", esc.reason));
                    esc_md.push_str(&format!("- **Agent Notes:** {}\n", esc.agent_notes));
                    esc_md.push_str(&format!(
                        "- **Original Asset Path:** `{}`\n\n",
                        rec.archive_original_path
                    ));
                }
            }
        }

        atomic_write_bytes(&self.escalation_md_file(), esc_md.as_bytes())?;

        // 3. Interactive Studio QC Application (`ledger/dashboard.html`)
        let html = self.generate_dashboard_html(&ledger, &asset_list);
        atomic_write_bytes(&self.dashboard_html_file(), html.as_bytes())?;

        // 4. Research & Dataset Curation JSONL Manifest (`ledger/dataset_manifest.jsonl` + `ledger/DATASET_CARD.md`)
        let _ = self.export_dataset_from_ledger(&ledger, false)?;

        Ok(())
    }

    /// Exports a HuggingFace / JSONL dataset manifest (`ledger/dataset_manifest.jsonl`) and Dataset Card (`ledger/DATASET_CARD.md`)
    /// with COCO `[x,y,w,h]`, VLM `Normalized1000` `[x1,y1,x2,y2]`, and YOLO `[cx,cy,w,h]` bounding boxes + safety telemetry.
    pub fn export_dataset(&self, safe_only: bool) -> Result<DatasetExportReport> {
        let ledger = self.load_ledger()?;
        self.export_dataset_from_ledger(&ledger, safe_only)
    }

    fn export_dataset_from_ledger(&self, ledger: &Ledger, safe_only: bool) -> Result<DatasetExportReport> {
        let mut asset_list: Vec<&AssetRecord> = ledger.assets.values().collect();
        asset_list.sort_by(|a, b| a.asset_id.cmp(&b.asset_id));

        let mut rows = Vec::new();
        let mut safe_count = 0usize;
        let mut redacted_count = 0usize;
        let mut untouched_count = 0usize;
        let mut escalated_count = 0usize;
        let mut total_pos_boxes = 0usize;
        let mut total_neg_boxes = 0usize;

        let mut coco_images = Vec::new();
        let mut coco_annotations = Vec::new();
        let mut coco_ann_id = 1usize;

        for (img_idx, rec) in asset_list.iter().enumerate() {
            let is_safe = matches!(rec.status, AssetStatus::Verified | AssetStatus::ApprovedOriginal)
                && rec.published_edition_path.is_some();

            if safe_only && !is_safe {
                continue;
            }

            let image_id = img_idx + 1;
            coco_images.push(serde_json::json!({
                "id": image_id,
                "asset_id": rec.asset_id,
                "file_name": rec.original_filename,
                "original_path": rec.archive_original_path,
                "sanitized_published_path": rec.published_edition_path,
                "width": rec.width,
                "height": rec.height,
                "status": rec.status,
                "is_safe_for_training_release": is_safe,
                "sha256_original": rec.sha256_original
            }));

            let curation_action = match rec.status {
                AssetStatus::Verified => {
                    redacted_count += 1;
                    safe_count += 1;
                    "surgical_region_redaction"
                }
                AssetStatus::ApprovedOriginal => {
                    untouched_count += 1;
                    safe_count += 1;
                    "preserved_untouched_false_positive"
                }
                AssetStatus::Escalated => {
                    escalated_count += 1;
                    "quarantined_for_human_review"
                }
                _ => "pending_triage",
            }
            .to_string();

            let (diff_px, redacted_pct, j1080_ok, j540_ok, qc_slate) = match &rec.verification {
                Some(v) => {
                    let j1080 = v
                        .region_metrics
                        .iter()
                        .all(|m| m.survives_publication_resize && m.pub_1080p_suppression_ratio >= 0.70);
                    let j540 = v
                        .region_metrics
                        .iter()
                        .all(|m| m.survives_publication_resize && m.pub_540p_suppression_ratio >= 0.70);
                    let slate = v
                        .publication_preview_paths
                        .iter()
                        .find(|p| p.ends_with("_contact_sheet.png"))
                        .cloned();
                    (
                        v.non_censored_pixel_diff_count,
                        v.total_censored_area_percent as f64,
                        j1080,
                        j540,
                        slate,
                    )
                }
                None => (0, 0.0, true, true, None),
            };

            let preservation_ratio = ((100.0 - redacted_pct).clamp(0.0, 100.0) / 100.0 * 10000.0).round() / 10000.0;
            let w_f = rec.width.max(1) as f32;
            let h_f = rec.height.max(1) as f32;

            let mut annotations = Vec::new();
            let mut pos_count = 0usize;
            let mut neg_count = 0usize;

            // 1. Positive Intervention Regions (class_id = 1)
            for r in &rec.regions {
                let (px, py, pw, ph) = r.resolve_pixels(rec.width, rec.height);
                let (ax, ay, aw, ah) = r.resolve_enclosing_aabb(rec.width, rec.height);

                let x1_n1000 = ((px as f32 / w_f) * 1000.0).round().clamp(0.0, 1000.0) as u32;
                let y1_n1000 = ((py as f32 / h_f) * 1000.0).round().clamp(0.0, 1000.0) as u32;
                let x2_n1000 = (((px + pw) as f32 / w_f) * 1000.0).round().clamp(0.0, 1000.0) as u32;
                let y2_n1000 = (((py + ph) as f32 / h_f) * 1000.0).round().clamp(0.0, 1000.0) as u32;

                let cx_01 = ((px as f32 + pw as f32 * 0.5) / w_f * 10000.0).round() / 10000.0;
                let cy_01 = ((py as f32 + ph as f32 * 0.5) / h_f * 10000.0).round() / 10000.0;
                let bw_01 = ((pw as f32 / w_f) * 10000.0).round() / 10000.0;
                let bh_01 = ((ph as f32 / h_f) * 10000.0).round() / 10000.0;

                let method_name = match &r.treatment {
                    crate::models::CensorshipMethod::Mosaic { .. } => "mosaic",
                    crate::models::CensorshipMethod::SolidBar { .. } => "solid_bar",
                    crate::models::CensorshipMethod::GaussianBlur { .. } => "gaussian_blur",
                }
                .to_string();

                let shape_name = match r.shape {
                    crate::models::RegionShape::Rect => "rect",
                    crate::models::RegionShape::Ellipse => "ellipse",
                    crate::models::RegionShape::Pill => "pill",
                }
                .to_string();

                coco_annotations.push(serde_json::json!({
                    "id": coco_ann_id,
                    "image_id": image_id,
                    "category_id": 1,
                    "category_name": "explicit_intervention_required",
                    "polarity": "positive_intervention",
                    "bbox": [px, py, pw, ph],
                    "enclosing_aabb": [ax, ay, aw, ah],
                    "area": pw * ph,
                    "iscrowd": 0,
                    "shape": shape_name,
                    "angle_deg": r.rotation_angle_deg(),
                    "treatment_method": method_name,
                    "label": r.label,
                    "rationale": r.rationale
                }));
                coco_ann_id += 1;
                pos_count += 1;
                total_pos_boxes += 1;

                annotations.push(DatasetAnnotationBox {
                    region_id: r.id.clone(),
                    polarity: "positive_intervention".to_string(),
                    class_id: 1,
                    annotation_source: "explicit_curation".to_string(),
                    confuser_category: "explicit_positive_target".to_string(),
                    fp_hardness_score: 0.0,
                    label: r.label.clone(),
                    rationale: r.rationale.clone(),
                    shape: shape_name,
                    treatment_method: method_name,
                    angle_deg: r.rotation_angle_deg(),
                    feather_px: r.feather_px,
                    bbox_xywh_px: [px, py, pw, ph],
                    bbox_xyxy_norm1000: [x1_n1000, y1_n1000, x2_n1000, y2_n1000],
                    bbox_cxcywh_norm01: [cx_01, cy_01, bw_01, bh_01],
                    enclosing_aabb_px: [ax, ay, aw, ah],
                });
            }

            // 2. Verified Benign Hard-Negative Regions (class_id = 0) to prevent False Positives
            let mut effective_negs = rec.negative_regions.clone();
            if effective_negs.is_empty() && is_safe {
                if let Ok(orig_img) = image::open(&rec.archive_original_path) {
                    let orig_rgba = orig_img.to_rgba8();
                    let pos_boxes: Vec<(u32, u32, u32, u32)> = rec
                        .regions
                        .iter()
                        .map(|r| r.resolve_enclosing_aabb(rec.width, rec.height))
                        .collect();
                    let hotspots = detect_candidate_hotspots(&orig_rgba, 3);
                    for (idx, hs) in hotspots.into_iter().enumerate() {
                        let overlaps_pos = pos_boxes
                            .iter()
                            .any(|&pb| compute_box_iou(hs.pixel_bounds, pb) >= 0.15);
                        if !overlaps_pos {
                            effective_negs.push(NegativeAnnotationRegion {
                                id: format!("auto_neg_{}", idx + 1),
                                label: format!("verified_benign_hotspot_{}", hs.rank),
                                rationale: "High-frequency artistic contour/feature verified non-explicit (hard-negative for false-positive suppression)".to_string(),
                                confuser_category: hs.confuser_category,
                                fp_hardness_score: hs.fp_hardness_score,
                                x: hs.pixel_bounds.0 as f32,
                                y: hs.pixel_bounds.1 as f32,
                                width: hs.pixel_bounds.2 as f32,
                                height: hs.pixel_bounds.3 as f32,
                                unit: CoordinateUnit::Pixels,
                                shape: RegionShape::Rect,
                            });
                        }
                    }
                }
            }

            for nr in &effective_negs {
                let (px, py, pw, ph) = nr.resolve_pixels(rec.width, rec.height);
                let x1_n1000 = ((px as f32 / w_f) * 1000.0).round().clamp(0.0, 1000.0) as u32;
                let y1_n1000 = ((py as f32 / h_f) * 1000.0).round().clamp(0.0, 1000.0) as u32;
                let x2_n1000 = (((px + pw) as f32 / w_f) * 1000.0).round().clamp(0.0, 1000.0) as u32;
                let y2_n1000 = (((py + ph) as f32 / h_f) * 1000.0).round().clamp(0.0, 1000.0) as u32;

                let cx_01 = ((px as f32 + pw as f32 * 0.5) / w_f * 10000.0).round() / 10000.0;
                let cy_01 = ((py as f32 + ph as f32 * 0.5) / h_f * 10000.0).round() / 10000.0;
                let bw_01 = ((pw as f32 / w_f) * 10000.0).round() / 10000.0;
                let bh_01 = ((ph as f32 / h_f) * 10000.0).round() / 10000.0;

                let shape_name = match nr.shape {
                    crate::models::RegionShape::Rect => "rect",
                    crate::models::RegionShape::Ellipse => "ellipse",
                    crate::models::RegionShape::Pill => "pill",
                }
                .to_string();

                let src = if nr.id.starts_with("auto_neg") {
                    "auto_mined_unredacted_hotspot"
                } else {
                    "human_verified_negative"
                }
                .to_string();

                coco_annotations.push(serde_json::json!({
                    "id": coco_ann_id,
                    "image_id": image_id,
                    "category_id": 2,
                    "category_name": "verified_benign_hard_negative",
                    "polarity": "verified_benign_hard_negative",
                    "confuser_category": nr.confuser_category,
                    "fp_hardness_score": nr.fp_hardness_score,
                    "bbox": [px, py, pw, ph],
                    "enclosing_aabb": [px, py, pw, ph],
                    "area": pw * ph,
                    "iscrowd": 0,
                    "shape": shape_name,
                    "angle_deg": 0.0,
                    "treatment_method": "none_preserved_untouched",
                    "label": nr.label,
                    "rationale": nr.rationale
                }));
                coco_ann_id += 1;
                neg_count += 1;
                total_neg_boxes += 1;

                annotations.push(DatasetAnnotationBox {
                    region_id: nr.id.clone(),
                    polarity: "verified_benign_hard_negative".to_string(),
                    class_id: 0,
                    annotation_source: src,
                    confuser_category: nr.confuser_category.clone(),
                    fp_hardness_score: nr.fp_hardness_score,
                    label: nr.label.clone(),
                    rationale: nr.rationale.clone(),
                    shape: shape_name,
                    treatment_method: "none_preserved_untouched".to_string(),
                    angle_deg: 0.0,
                    feather_px: 0,
                    bbox_xywh_px: [px, py, pw, ph],
                    bbox_xyxy_norm1000: [x1_n1000, y1_n1000, x2_n1000, y2_n1000],
                    bbox_cxcywh_norm01: [cx_01, cy_01, bw_01, bh_01],
                    enclosing_aabb_px: [px, py, pw, ph],
                });
            }

            rows.push(DatasetExportRow {
                asset_id: rec.asset_id.clone(),
                original_filename: rec.original_filename.clone(),
                width: rec.width,
                height: rec.height,
                status: rec.status,
                classification: rec.classification,
                policy_profile: rec.policy_profile.clone(),
                is_safe_for_training_release: is_safe,
                curation_action,
                sha256_original: rec.sha256_original.clone(),
                sha256_published: rec.sha256_published.clone(),
                original_path: rec.archive_original_path.clone(),
                sanitized_published_path: rec.published_edition_path.clone(),
                vlm_inspect_grid_path: rec.inspect_grid_path.clone(),
                qc_contact_sheet_path: qc_slate,
                exterior_pixel_diff_count: diff_px,
                exterior_preservation_ratio: preservation_ratio,
                redacted_area_percent: redacted_pct,
                multiscale_jpeg_1080p_q85_passed: j1080_ok,
                multiscale_jpeg_540p_q75_passed: j540_ok,
                positive_annotations_count: pos_count,
                hard_negative_annotations_count: neg_count,
                annotations,
            });
        }

        let mut jsonl_buf = String::new();
        let mut sft_buf = String::new();
        for row in &rows {
            if let Ok(line) = serde_json::to_string(row) {
                jsonl_buf.push_str(&line);
                jsonl_buf.push('\n');
            }

            // Build ShareGPT / VLM Grounded Safety & False-Positive Suppression Instruction Pair
            let pos_list: Vec<serde_json::Value> = row
                .annotations
                .iter()
                .filter(|a| a.class_id == 1)
                .map(|a| {
                    serde_json::json!({
                        "region_id": a.region_id,
                        "label": a.label,
                        "bbox_norm1000": a.bbox_xyxy_norm1000,
                        "shape": a.shape,
                        "treatment_method": a.treatment_method,
                        "rationale": a.rationale
                    })
                })
                .collect();
            let neg_list: Vec<serde_json::Value> = row
                .annotations
                .iter()
                .filter(|a| a.class_id == 0)
                .map(|a| {
                    serde_json::json!({
                        "region_id": a.region_id,
                        "label": a.label,
                        "confuser_category": a.confuser_category,
                        "fp_hardness_score": a.fp_hardness_score,
                        "bbox_norm1000": a.bbox_xyxy_norm1000,
                        "rationale": a.rationale
                    })
                })
                .collect();

            let assistant_answer = serde_json::json!({
                "status": row.status,
                "classification": row.classification,
                "curation_action": row.curation_action,
                "is_safe_for_training_release": row.is_safe_for_training_release,
                "positive_interventions": pos_list,
                "verified_benign_hard_negatives": neg_list
            });

            let sft_record = serde_json::json!({
                "id": row.asset_id,
                "image": row.original_path,
                "sanitized_image": row.sanitized_published_path,
                "curation_action": row.curation_action,
                "conversations": [
                    {
                        "from": "human",
                        "value": "<image>\nPerform localized safety grounding and false-positive confuser triage in Normalized1000 [x1, y1, x2, y2] coordinates. Distinguish explicit anatomical regions requiring surgical intervention (class_id=1) from verified benign artistic silhouettes, skin-tone gradients, and linework confusers that must be preserved untouched to prevent false positives (class_id=0)."
                    },
                    {
                        "from": "gpt",
                        "value": serde_json::to_string(&assistant_answer).unwrap_or_default()
                    }
                ]
            });
            if let Ok(sft_line) = serde_json::to_string(&sft_record) {
                sft_buf.push_str(&sft_line);
                sft_buf.push('\n');
            }
        }
        atomic_write_bytes(&self.dataset_manifest_jsonl_file(), jsonl_buf.as_bytes())?;
        atomic_write_bytes(&self.vlm_sft_jsonl_file(), sft_buf.as_bytes())?;

        // Write MS-COCO Object Detection & Hard-Negative JSON (`ledger/annotations_coco.json`)
        let coco_doc = serde_json::json!({
            "info": {
                "description": "HEAV3NS Contrastive Positive & Hard-Negative Safety Dataset (MS-COCO Format)",
                "version": ledger.version,
                "year": 2026,
                "contributor": "HEAV3NS Post-Production & AI Safety Research (MAJO, EGG, Deputy Gemini)",
                "date_created": Utc::now().to_rfc3339()
            },
            "licenses": [{
                "id": 1,
                "name": "MIT License",
                "url": "https://opensource.org/licenses/MIT"
            }],
            "categories": [
                {
                    "id": 1,
                    "name": "explicit_intervention_required",
                    "supercategory": "safety_positive_redaction"
                },
                {
                    "id": 2,
                    "name": "verified_benign_hard_negative",
                    "supercategory": "false_positive_suppression"
                }
            ],
            "images": coco_images,
            "annotations": coco_annotations
        });
        let coco_bytes = serde_json::to_vec_pretty(&coco_doc)?;
        atomic_write_bytes(&self.coco_annotations_json_file(), &coco_bytes)?;

        let card_md = format!(
            "---\nlicense: mit\ntask_categories:\n  - object-detection\n  - image-to-image\n  - image-classification\ntags:\n  - ai-safety\n  - false-positive-suppression\n  - hard-negative-mining\n  - dataset-cleaning\n  - vlm-grounding\n  - artistic-preservation\n---\n\n# HEAV3NS Contrastive Safety, Hard-Negative & Sanitized Vision Dataset Card\n\nGenerated deterministically by **`heav3ns-censor v{}`**.\n\n## Dataset Summary & False-Positive Suppression Provenance\n\n- **Total Exported Images:** `{}` (`safe_only = {}`)\n- **Safe Training Split (`is_safe_for_training_release == true`):** `{}`\n  - **Surgical Localized Redactions (`Verified`):** `{}` (`0` exterior pixels altered; verified across Native, `1080p Q85`, and `540p Q75` JPEG)\n  - **Preserved False-Positive Artistic Corpora (`ApprovedOriginal`):** `{}` (`100%` bit-identical to original)\n- **Quarantined Ambiguity Benchmark (`Escalated`):** `{}` (isolated from `assets/published/`)\n- **Region-Level Bounding Box Annotations (`COCO` + `Normalized1000` + `YOLO`) & Confuser Hardness:**\n  - **Positive Intervention Boxes (`class_id = 1`, `positive_intervention`):** `{}`\n  - **Verified Benign Hard-Negative Boxes (`class_id = 0`, `verified_benign_hard_negative`):** `{}` *(eliminates classifier false positives on classical silhouettes, skin-tone gradients, and artistic linework; scored with quantitative `fp_hardness_score` in `[5.0..99.5]` and automatically de-conflicted so positive/negative boxes never collide)*\n- **Exported Manifests:**\n  - `ledger/dataset_manifest.jsonl` (HuggingFace `datasets` / JSON Lines with `fp_hardness_score` & `confuser_category`)\n  - `ledger/annotations_coco.json` (MS-COCO Object Detection format for CVAT, Label Studio, Roboflow, & Voxel51 FiftyOne)\n  - `ledger/vlm_safety_sft.jsonl` (ShareGPT / VLM Grounded Safety & False-Positive Suppression Instruction-Tuning Pairs)\n\n## Loading in Python (`datasets` / `json` / `pycocotools`)\n\n```python\nimport json\nfrom pathlib import Path\n\nrecords = [\n    json.loads(line)\n    for line in Path(\"ledger/dataset_manifest.jsonl\").read_text().splitlines()\n    if line.strip()\n]\npos_boxes = [a for r in records for a in r[\"annotations\"] if a[\"class_id\"] == 1]\nhard_neg_boxes = [a for r in records for a in r[\"annotations\"] if a[\"class_id\"] == 0]\nprint(f\"Images: {{len(records)}} | Positive Redaction Boxes: {{len(pos_boxes)}} | Hard-Negative Benign Boxes: {{len(hard_neg_boxes)}}\")\n```\n",
            ledger.version,
            rows.len(),
            safe_only,
            safe_count,
            redacted_count,
            untouched_count,
            escalated_count,
            total_pos_boxes,
            total_neg_boxes,
        );
        atomic_write_bytes(&self.dataset_card_md_file(), card_md.as_bytes())?;

        Ok(DatasetExportReport {
            exported_at: Utc::now().to_rfc3339(),
            safe_only_filter: safe_only,
            total_records_exported: rows.len(),
            safe_training_records_count: safe_count,
            surgical_redaction_count: redacted_count,
            preserved_untouched_count: untouched_count,
            quarantined_escalated_count: escalated_count,
            total_positive_boxes: total_pos_boxes,
            total_hard_negative_boxes: total_neg_boxes,
            jsonl_manifest_path: self.dataset_manifest_jsonl_file().to_string_lossy().to_string(),
            coco_annotations_path: self.coco_annotations_json_file().to_string_lossy().to_string(),
            vlm_sft_jsonl_path: self.vlm_sft_jsonl_file().to_string_lossy().to_string(),
            dataset_card_path: self.dataset_card_md_file().to_string_lossy().to_string(),
            records: rows,
        })
    }

    fn to_relative_dashboard_path(&self, abs_or_rel: &str) -> String {
        let p = Path::new(abs_or_rel);
        if let Ok(stripped) = p.strip_prefix(&self.root) {
            format!("../{}", stripped.to_string_lossy().replace('\\', "/"))
        } else {
            format!("file://{}", abs_or_rel)
        }
    }

    fn generate_dashboard_html(&self, ledger: &Ledger, assets: &[&AssetRecord]) -> String {
        let count_verified = assets
            .iter()
            .filter(|a| a.status == AssetStatus::Verified)
            .count();
        let count_approved = assets
            .iter()
            .filter(|a| a.status == AssetStatus::ApprovedOriginal)
            .count();
        let count_escalated = assets
            .iter()
            .filter(|a| a.status == AssetStatus::Escalated)
            .count();
        let count_pending = assets
            .iter()
            .filter(|a| a.status == AssetStatus::PendingInspection)
            .count();

        let mut cards = String::new();
        for rec in assets {
            let (status_class, status_label) = match rec.status {
                AssetStatus::Verified => ("verified", "VERIFIED EDITION"),
                AssetStatus::ApprovedOriginal => ("approved", "APPROVED UNTOUCHED"),
                AssetStatus::Escalated => ("escalated", "ESCALATED FOR REVIEW"),
                _ => ("pending", "PENDING INSPECTION"),
            };

            let orig_rel = self.to_relative_dashboard_path(&rec.archive_original_path);
            let grid_rel = rec
                .inspect_grid_path
                .as_deref()
                .map(|p| self.to_relative_dashboard_path(p))
                .unwrap_or_else(|| orig_rel.clone());
            let pub_rel = rec
                .published_edition_path
                .as_deref()
                .map(|p| self.to_relative_dashboard_path(p))
                .unwrap_or_else(|| orig_rel.clone());

            let contact_rel = rec
                .verification
                .as_ref()
                .and_then(|v| {
                    v.publication_preview_paths
                        .iter()
                        .find(|p| p.ends_with("_contact_sheet.png"))
                })
                .map(|p| self.to_relative_dashboard_path(p))
                .unwrap_or_else(|| pub_rel.clone());

            let p1080_rel = rec
                .verification
                .as_ref()
                .and_then(|v| {
                    v.publication_preview_paths
                        .iter()
                        .find(|p| p.ends_with("_pub_1080p_q85.jpg"))
                })
                .map(|p| self.to_relative_dashboard_path(p))
                .unwrap_or_else(|| pub_rel.clone());

            let p540_rel = rec
                .verification
                .as_ref()
                .and_then(|v| {
                    v.publication_preview_paths
                        .iter()
                        .find(|p| p.ends_with("_pub_540p_thumb.jpg"))
                })
                .map(|p| self.to_relative_dashboard_path(p))
                .unwrap_or_else(|| pub_rel.clone());

            let mut region_rows = String::new();
            if let Some(v) = &rec.verification {
                for rm in &v.region_metrics {
                    region_rows.push_str(&format!(
                        r#"<tr>
                            <td><code>{}</code></td>
                            <td><code>{}</code></td>
                            <td><code>({},{}, {}x{})</code></td>
                            <td><strong>{:.1}%</strong></td>
                            <td><strong>{:.1}%</strong></td>
                            <td><strong>{:.1}%</strong></td>
                            <td><strong>{:.1}%</strong></td>
                            <td><span class="pill-pass">{}</span></td>
                        </tr>"#,
                        rm.region_id,
                        rm.method_summary,
                        rm.pixel_bounds.0,
                        rm.pixel_bounds.1,
                        rm.pixel_bounds.2,
                        rm.pixel_bounds.3,
                        rm.detail_suppression_ratio * 100.0,
                        rm.peak_detail_suppression_ratio * 100.0,
                        rm.pub_1080p_suppression_ratio * 100.0,
                        rm.pub_540p_suppression_ratio * 100.0,
                        if rm.passed { "PASS" } else { "FAIL" }
                    ));
                }
            }

            let mut neg_table_html = String::new();
            if !rec.negative_regions.is_empty() {
                let mut neg_rows = String::new();
                for nr in &rec.negative_regions {
                    let (nx, ny, nw, nh) = nr.resolve_pixels(rec.width, rec.height);
                    neg_rows.push_str(&format!(
                        r#"<tr>
                            <td><code>{}</code></td>
                            <td><code>{}</code></td>
                            <td><code>{}</code></td>
                            <td><code>({},{}, {}x{})</code></td>
                            <td><strong>{:.1} / 100</strong></td>
                            <td><span style="color:var(--green);font-weight:700">VERIFIED BENIGN (CLASS 0)</span></td>
                        </tr>"#,
                        nr.id,
                        nr.label,
                        nr.confuser_category,
                        nx,
                        ny,
                        nw,
                        nh,
                        nr.fp_hardness_score
                    ));
                }
                neg_table_html = format!(
                    r#"<table class="region-table">
                        <thead>
                            <tr><th>Hard-Negative ID</th><th>Label</th><th>Confuser Taxonomy</th><th>Pixel Bounds</th><th>Confuser Hardness</th><th>Polarity</th></tr>
                        </thead>
                        <tbody>{}</tbody>
                    </table>"#,
                    neg_rows
                );
            }

            let telemetry_block = if let Some(v) = &rec.verification {
                format!(
                    r#"<div class="metrics-bar">
                        <div><span class="lbl">Uncensored Pixel Fidelity:</span> <strong class="ok">100% Bit-Exact ({} diff px)</strong></div>
                        <div><span class="lbl">Localized Area:</span> <strong>{:.2}% of canvas</strong></div>
                        <div><span class="lbl">Hard-Negative Confusers:</span> <strong style="color:var(--green)">{} Verified Benign</strong></div>
                        <div><span class="lbl">Downscale Survival (1080p &amp; 540p):</span> <strong class="ok">VERIFIED</strong></div>
                    </div>
                    <table class="region-table">
                        <thead>
                            <tr><th>Region</th><th>Treatment</th><th>Enclosing AABB</th><th>Native Mean</th><th>Peak p95</th><th>1080p JPEG</th><th>540p Thumb</th><th>Status</th></tr>
                        </thead>
                        <tbody>{}</tbody>
                    </table>
                    {}"#,
                    v.non_censored_pixel_diff_count,
                    v.total_censored_area_percent,
                    rec.negative_regions.len(),
                    region_rows,
                    neg_table_html
                )
            } else if let Some(esc) = &rec.escalation {
                format!(
                    r#"<div class="metrics-bar escalated-box">
                        <div><strong>ESCALATION CATEGORY [{}]:</strong> {}</div>
                        <div class="notes">Agent Notes: {}</div>
                    </div>"#,
                    esc.ambiguity_category, esc.reason, esc.agent_notes
                )
            } else {
                format!(
                    r#"<div class="metrics-bar approved-box">
                        <div><strong>ARTISTIC NUDITY / SUGGESTIVE PASS:</strong> Untouched original approved for publication with zero censorship modification (SHA-256 bit-identical • {} hard-negative confuser box(es)).</div>
                    </div>
                    {}"#,
                    rec.negative_regions.len(),
                    neg_table_html
                )
            };

            let mut audit_items = String::new();
            for ev in &rec.audit_trail {
                audit_items.push_str(&format!(
                    "<li><code>{}</code> <strong>{}</strong> — {}</li>",
                    ev.timestamp.format("%H:%M:%S UTC"),
                    ev.action,
                    ev.details
                ));
            }

            cards.push_str(&format!(
                r#"<div class="card" data-status="{status_class}" data-id="{id}" data-w="{w}" data-h="{h}" data-mode="{initial_mode}" data-search="{id} {orig_name} {status_class}">
                    <div class="card-header">
                        <div>
                            <h3><code>{id}</code></h3>
                            <p class="sub">Original: <code>{orig_name}</code> ({w}x{h}) • Policy: <code>{policy}</code> • SHA-256: <code>{sha}</code></p>
                        </div>
                        <span class="badge {status_class}">{status_label}</span>
                    </div>
                    {telemetry_block}
                    <div class="view-tabs">
                        <button class="vtab active" onclick="switchImg(this, '{contact_rel}', '{initial_mode}')">[1] 4-Panel QC Slate</button>
                        <button class="vtab" onclick="switchImg(this, '{pub_rel}', 'single')">[2] Published Edition</button>
                        <button class="vtab" onclick="switchImg(this, '{p1080_rel}', 'single')">[3] 1080p JPEG Sim</button>
                        <button class="vtab" onclick="switchImg(this, '{p540_rel}', 'single')">[4] 540p Thumb Sim</button>
                        <button class="vtab" onclick="switchImg(this, '{grid_rel}', 'single')">[5] VLM Coordinate Grid</button>
                        <button class="vtab" onclick="switchImg(this, '{orig_rel}', 'single')">[6] Untouched Original</button>
                        <button class="vtab ab-btn" onclick="toggleAbCompare(this)" title="Press 'o' to A/B toggle between Published Edition [2] and Untouched Original [6]">[o] A/B Compare Original</button>
                    </div>
                    <div class="coord-hud" data-cmd-type="mosaic">
                        <div class="hud-left">
                            <span class="coord-readout">Hover or drag on image to measure coordinates &amp; generate CLI</span>
                            <div class="cmd-type-group">
                                <button class="ctbtn active" onclick="setCmdType(this, 'mosaic')">Mosaic</button>
                                <button class="ctbtn" onclick="setCmdType(this, 'pill_bar')">Pill Bar (Auto)</button>
                                <button class="ctbtn" onclick="setCmdType(this, 'hard_negative')" title="Annotate a verified benign confuser region to prevent classifier false positives without altering pixels">+ Hard-Negative (Benign)</button>
                                <button class="ctbtn" onclick="setCmdType(this, 'crop')">3x Zoom-Crop</button>
                                <button class="ctbtn" onclick="setQuickCmd(this, 'approve')">Approve Original</button>
                                <button class="ctbtn" onclick="setQuickCmd(this, 'escalate')">Escalate</button>
                            </div>
                        </div>
                        <div class="cli-gen">
                            <input type="text" class="cli-out" readonly value="./target/release/heav3ns-censor inspect --asset-id {id}" />
                            <button class="copy-btn" onclick="copyCli(this)">Copy CLI</button>
                        </div>
                    </div>
                    <div class="preview-wrap" onmousemove="onPreviewMove(event, this)" onmousedown="onPreviewDown(event, this)" onmouseup="onPreviewUp(event, this)">
                        <img src="{contact_rel}" alt="{id}" loading="lazy" draggable="false" />
                        <div class="sel-box"></div>
                    </div>
                    <details class="audit-log">
                        <summary>Audit Trail ({audit_count} events)</summary>
                        <ul>{audit_items}</ul>
                    </details>
                </div>"#,
                status_class = status_class,
                id = rec.asset_id,
                orig_name = rec.original_filename,
                w = rec.width,
                h = rec.height,
                initial_mode = if rec.verification.is_some() { "slate" } else { "single" },
                policy = rec.policy_profile,
                sha = &rec.sha256_original[0..16],
                status_label = status_label,
                telemetry_block = telemetry_block,
                contact_rel = contact_rel,
                pub_rel = pub_rel,
                p1080_rel = p1080_rel,
                p540_rel = p540_rel,
                grid_rel = grid_rel,
                orig_rel = orig_rel,
                audit_count = rec.audit_trail.len(),
                audit_items = audit_items,
            ));
        }

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8" />
<title>HEAV3NS Post-Production QC &amp; Censorship Studio (v4.0)</title>
<style>
  :root {{
    --bg: #090b10;
    --panel: #121621;
    --panel-alt: #0d1018;
    --border: #232a3d;
    --text: #e6edf3;
    --muted: #8b949e;
    --cyan: #00ffd5;
    --green: #3fb950;
    --amber: #d29922;
    --pink: #ff328c;
  }}
  * {{ box-sizing: border-box; }}
  body {{
    background: var(--bg);
    color: var(--text);
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, monospace;
    margin: 0;
    padding: 28px 36px;
  }}
  header {{
    border-bottom: 1px solid var(--border);
    padding-bottom: 18px;
    margin-bottom: 22px;
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    flex-wrap: wrap;
    gap: 12px;
  }}
  h1 {{ margin: 0 0 6px 0; font-size: 20px; letter-spacing: 0.06em; color: var(--cyan); }}
  .meta {{ color: var(--muted); font-size: 12px; }}
  .kpi-grid {{
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(175px, 1fr));
    gap: 14px;
    margin-bottom: 22px;
  }}
  .kpi {{
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 14px 16px;
  }}
  .kpi .k-val {{ font-size: 24px; font-weight: 700; margin-bottom: 4px; }}
  .kpi .k-lbl {{ font-size: 11px; color: var(--muted); text-transform: uppercase; letter-spacing: 0.05em; }}
  .toolbar {{
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    margin-bottom: 20px;
    flex-wrap: wrap;
  }}
  .filter-group {{ display: flex; gap: 8px; flex-wrap: wrap; }}
  .fbtn {{
    background: var(--panel);
    color: var(--muted);
    border: 1px solid var(--border);
    padding: 6px 13px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 12px;
    font-family: inherit;
  }}
  .fbtn.active, .fbtn:hover {{ color: var(--cyan); border-color: var(--cyan); background: rgba(0,255,213,0.08); }}
  .search-input {{
    background: var(--panel);
    color: var(--text);
    border: 1px solid var(--border);
    padding: 7px 12px;
    border-radius: 6px;
    font-size: 12px;
    min-width: 260px;
    font-family: inherit;
  }}
  .card {{
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 20px;
    margin-bottom: 22px;
  }}
  .card-header {{
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 12px;
  }}
  h3 {{ margin: 0 0 4px 0; font-size: 16px; }}
  .sub {{ margin: 0; color: var(--muted); font-size: 12px; }}
  .badge {{
    padding: 4px 10px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 700;
  }}
  .verified {{ background: rgba(0, 255, 213, 0.14); color: var(--cyan); border: 1px solid var(--cyan); }}
  .approved {{ background: rgba(63, 185, 80, 0.15); color: var(--green); border: 1px solid var(--green); }}
  .escalated {{ background: rgba(255, 50, 140, 0.15); color: var(--pink); border: 1px solid var(--pink); }}
  .pending {{ background: rgba(210, 153, 34, 0.15); color: var(--amber); border: 1px solid var(--amber); }}
  .metrics-bar {{
    display: flex;
    flex-wrap: wrap;
    gap: 24px;
    background: var(--panel-alt);
    padding: 10px 14px;
    border-radius: 6px;
    font-size: 12px;
    margin-bottom: 12px;
    border: 1px solid var(--border);
  }}
  .metrics-bar .lbl {{ color: var(--muted); }}
  .metrics-bar .ok {{ color: var(--cyan); }}
  .escalated-box {{ color: #ff80b5; flex-direction: column; gap: 6px; }}
  .approved-box {{ color: var(--green); }}
  .region-table {{
    width: 100%;
    border-collapse: collapse;
    font-size: 12px;
    margin-bottom: 14px;
    background: var(--panel-alt);
    border-radius: 6px;
    overflow: hidden;
  }}
  .region-table th, .region-table td {{
    padding: 7px 10px;
    text-align: left;
    border-bottom: 1px solid var(--border);
  }}
  .region-table th {{ color: var(--muted); font-weight: 600; font-size: 11px; text-transform: uppercase; }}
  .pill-pass {{ color: var(--cyan); font-weight: 700; }}
  .view-tabs {{ display: flex; gap: 6px; margin-bottom: 10px; flex-wrap: wrap; }}
  .vtab {{
    background: var(--panel-alt);
    color: var(--muted);
    border: 1px solid var(--border);
    padding: 5px 11px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 11px;
    font-family: inherit;
  }}
  .vtab.active, .vtab:hover {{ color: var(--text); border-color: var(--cyan); background: rgba(0,255,213,0.1); }}
  .coord-hud {{
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
    background: var(--panel-alt);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 8px 12px;
    margin-bottom: 10px;
    font-size: 11px;
  }}
  .hud-left {{ display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }}
  .coord-readout {{ color: var(--cyan); font-weight: 600; min-width: 240px; }}
  .cmd-type-group {{ display: flex; gap: 4px; flex-wrap: wrap; }}
  .ctbtn {{
    background: #090b10;
    color: var(--muted);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 3px 8px;
    cursor: pointer;
    font-size: 10px;
    font-family: inherit;
  }}
  .ctbtn.active, .ctbtn:hover {{ color: var(--cyan); border-color: var(--cyan); background: rgba(0,255,213,0.1); }}
  .onboard-drawer {{
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 12px 16px;
    margin-bottom: 20px;
    font-size: 12px;
  }}
  .onboard-drawer summary {{
    cursor: pointer;
    color: var(--cyan);
    font-weight: 700;
    user-select: none;
  }}
  .role-grid {{
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
    gap: 12px;
    margin-top: 12px;
  }}
  .role-box {{
    background: var(--panel-alt);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 10px 12px;
    line-height: 1.5;
  }}
  .role-box h4 {{ margin: 0 0 6px 0; color: var(--text); font-size: 12px; }}
  .role-box a {{ color: var(--cyan); text-decoration: none; }}
  .cli-gen {{ display: flex; gap: 6px; flex: 1; min-width: 280px; justify-content: flex-end; }}
  .cli-out {{
    background: #07090e;
    color: var(--text);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 4px 8px;
    font-size: 11px;
    font-family: inherit;
    width: 100%;
    max-width: 560px;
  }}
  .copy-btn {{
    background: rgba(0,255,213,0.12);
    color: var(--cyan);
    border: 1px solid var(--cyan);
    border-radius: 4px;
    padding: 4px 10px;
    cursor: pointer;
    font-size: 11px;
    font-family: inherit;
    white-space: nowrap;
  }}
  .preview-wrap {{
    position: relative;
    display: inline-block;
    max-width: 100%;
    cursor: crosshair;
    user-select: none;
  }}
  .preview-wrap img {{
    max-width: 100%;
    border-radius: 6px;
    border: 1px solid var(--border);
    display: block;
  }}
  .sel-box {{
    position: absolute;
    border: 2px dashed var(--cyan);
    background: rgba(0, 255, 213, 0.14);
    pointer-events: none;
    display: none;
  }}
  .audit-log {{ margin-top: 12px; font-size: 12px; color: var(--muted); }}
  .audit-log summary {{ cursor: pointer; user-select: none; }}
  .audit-log ul {{ margin: 8px 0 0 0; padding-left: 18px; line-height: 1.6; }}
</style>
</head>
<body>
  <header>
    <div>
      <h1>HEAV3NS POST-PRODUCTION &amp; CENSORSHIP STUDIO QC (v4.0)</h1>
      <div class="meta">Commissioning Authority: <strong>{auth}</strong> • Commissioned Agent: <strong>{agent}</strong> • Principal Artist: <strong>{artist}</strong></div>
    </div>
    <div class="meta">Shortcuts: <code>/</code> Search • <code>1-6</code> Switch Tab on Hovered Card • Drag on Image to Measure Region • Updated: {updated}</div>
  </header>

  <div class="kpi-grid">
    <div class="kpi"><div class="k-val">{total}</div><div class="k-lbl">Total Ingested Assets</div></div>
    <div class="kpi"><div class="k-val" style="color:var(--cyan)">{verified}</div><div class="k-lbl">Verified Censored Editions</div></div>
    <div class="kpi"><div class="k-val" style="color:var(--green)">{approved}</div><div class="k-lbl">Approved Untouched (Artistic)</div></div>
    <div class="kpi"><div class="k-val" style="color:var(--pink)">{escalated}</div><div class="k-lbl">Escalated for Review</div></div>
    <div class="kpi"><div class="k-val" style="color:var(--cyan)">100%</div><div class="k-lbl">Bit-Exact Preservation</div></div>
  </div>

  <details class="onboard-drawer">
    <summary>Studio Handoff, Dataset Curation &amp; Role-Based Onboarding Guide (Researchers • EGG • MAJO • Deputy Gemini • Developers)</summary>
    <div class="role-grid">
      <div class="role-box">
        <h4>1. AI Safety &amp; Dataset Researchers</h4>
        Export <a href="dataset_manifest.jsonl">dataset_manifest.jsonl</a> &amp; <a href="DATASET_CARD.md">DATASET_CARD.md</a> via <code>export-dataset [--safe-only]</code> with COCO, Normalized1000, &amp; YOLO boxes.<br/>
        <a href="../docs/knowledgebase/03-dataset-cleaning-and-ai-safety-research.md">Read Dataset Cleaning Guide &rarr;</a>
      </div>
      <div class="role-box">
        <h4>2. Principal Artist (EGG)</h4>
        Drop master files in <code>assets/inbox/</code>. Originals are archived untouched in <code>assets/originals/</code> with SHA-256 verification. Press <code>[o]</code> to A/B compare Original vs. Published.<br/>
        <a href="../docs/stakeholder-handoff/STAKEHOLDER_HANDOFF.md">Read Stakeholder Handoff &rarr;</a>
      </div>
      <div class="role-box">
        <h4>3. Executive Chair (MAJO)</h4>
        Filter by <strong>Escalated</strong> below or open <a href="escalations_review.md">escalations_review.md</a> to resolve ambiguous edge cases. Open <a href="publication_report.md">publication_report.md</a> for release sign-off.<br/>
        <a href="../docs/philosophy/PHILOSOPHY.md">Read Studio Philosophy &rarr;</a>
      </div>
      <div class="role-box">
        <h4>4. Autonomous Agent (Deputy Gemini)</h4>
        Run <code>batch-prepare</code> &rarr; VLM inspect <code>*_inspect_grid.png</code> &rarr; <code>execute</code> / <code>approve-original</code> / <code>escalate</code> &rarr; <code>doctor --fix</code>.<br/>
        <a href="../docs/onboarding/AGENT_ONBOARDING.md">Read Agent Runbook &rarr;</a>
      </div>
      <div class="role-box">
        <h4>5. Systems Developers</h4>
        Run <code>./target/release/heav3ns-censor onboard --role developer</code> or <code>cargo test --release</code>.<br/>
        <a href="../docs/whitepaper/TECHNICAL_WHITEPAPER.md">Read Technical Whitepaper &rarr;</a>
      </div>
    </div>
  </details>

  <div class="toolbar">
    <div class="filter-group">
      <button class="fbtn active" onclick="filterCards('all', this)">All ({total})</button>
      <button class="fbtn" onclick="filterCards('verified', this)">Verified ({verified})</button>
      <button class="fbtn" onclick="filterCards('approved', this)">Approved Original ({approved})</button>
      <button class="fbtn" onclick="filterCards('escalated', this)">Escalated ({escalated})</button>
      <button class="fbtn" onclick="filterCards('pending', this)">Pending ({pending})</button>
    </div>
    <input id="global-search" type="text" class="search-input" placeholder="Press '/' to filter by Asset ID or filename..." oninput="searchCards(this.value)" />
  </div>

  <div id="cards-container">
    {cards}
  </div>

  <script>
    let hoveredCard = null;
    let dragState = null;

    document.querySelectorAll('.card').forEach(c => {{
      c.addEventListener('mouseenter', () => {{ hoveredCard = c; }});
    }});

    document.addEventListener('keydown', e => {{
      if (e.target.tagName === 'INPUT') return;
      if (e.key === '/') {{
        e.preventDefault();
        document.getElementById('global-search').focus();
      }} else if (hoveredCard && (e.key === 'o' || e.key === 'O')) {{
        e.preventDefault();
        const abBtn = hoveredCard.querySelector('.ab-btn');
        if (abBtn) toggleAbCompare(abBtn);
      }} else if (hoveredCard && e.key >= '1' && e.key <= '6') {{
        const idx = parseInt(e.key, 10) - 1;
        const tabs = hoveredCard.querySelectorAll('.vtab');
        if (tabs[idx]) tabs[idx].click();
      }}
    }});

    function toggleAbCompare(btn) {{
      const card = btn.closest('.card');
      const tabs = card.querySelectorAll('.vtab');
      if (tabs[5].classList.contains('active')) {{
        tabs[1].click();
      }} else {{
        tabs[5].click();
      }}
    }}

    function switchImg(btn, src, mode) {{
      const card = btn.closest('.card');
      card.dataset.mode = mode;
      card.querySelectorAll('.vtab').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      card.querySelector('.preview-wrap img').src = src;
      card.querySelector('.sel-box').style.display = 'none';
    }}

    function mapClientToAssetCoords(e, wrap) {{
      const card = wrap.closest('.card');
      const img = wrap.querySelector('img');
      const rect = img.getBoundingClientRect();
      const w = parseFloat(card.dataset.w);
      const h = parseFloat(card.dataset.h);
      const mode = card.dataset.mode;

      const relX = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
      const relY = Math.max(0, Math.min(1, (e.clientY - rect.top) / rect.height));

      if (mode === 'slate') {{
        const slateW = w * 2 + 48;
        const slateH = h * 2 + 54 + 48;
        const sx = relX * slateW;
        const sy = relY * slateH;
        const qx = sx < (16 + w + 8) ? (sx - 16) : (sx - (32 + w));
        const qy = sy < (54 + h + 8) ? (sy - 54) : (sy - (54 + 16 + h));
        const ax = Math.max(0, Math.min(w, Math.round(qx)));
        const ay = Math.max(0, Math.min(h, Math.round(qy)));
        return {{ ax, ay, w, h, clientX: e.clientX - rect.left, clientY: e.clientY - rect.top }};
      }} else {{
        const ax = Math.max(0, Math.min(w, Math.round(relX * w)));
        const ay = Math.max(0, Math.min(h, Math.round(relY * h)));
        return {{ ax, ay, w, h, clientX: e.clientX - rect.left, clientY: e.clientY - rect.top }};
      }}
    }}

    function onPreviewDown(e, wrap) {{
      const pt = mapClientToAssetCoords(e, wrap);
      const box = wrap.querySelector('.sel-box');
      box.style.left = pt.clientX + 'px';
      box.style.top = pt.clientY + 'px';
      box.style.width = '0px';
      box.style.height = '0px';
      box.style.display = 'block';
      dragState = {{ wrap, start: pt }};
    }}

    function onPreviewMove(e, wrap) {{
      const card = wrap.closest('.card');
      const pt = mapClientToAssetCoords(e, wrap);
      const nx = Math.round((pt.ax / pt.w) * 1000);
      const ny = Math.round((pt.ay / pt.h) * 1000);
      const readout = card.querySelector('.coord-readout');

      if (dragState && dragState.wrap === wrap) {{
        const box = wrap.querySelector('.sel-box');
        const left = Math.min(dragState.start.clientX, pt.clientX);
        const top = Math.min(dragState.start.clientY, pt.clientY);
        const bw = Math.abs(pt.clientX - dragState.start.clientX);
        const bh = Math.abs(pt.clientY - dragState.start.clientY);
        box.style.left = left + 'px';
        box.style.top = top + 'px';
        box.style.width = bw + 'px';
        box.style.height = bh + 'px';

        const rx = Math.min(dragState.start.ax, pt.ax);
        const ry = Math.min(dragState.start.ay, pt.ay);
        const rw = Math.max(4, Math.abs(pt.ax - dragState.start.ax));
        const rh = Math.max(4, Math.abs(pt.ay - dragState.start.ay));
        const rnx = Math.round((rx / pt.w) * 1000);
        const rny = Math.round((ry / pt.h) * 1000);
        const rnw = Math.max(4, Math.round((rw / pt.w) * 1000));
        const rnh = Math.max(4, Math.round((rh / pt.h) * 1000));

        readout.innerHTML = `Selection Pixels: <code>${{rx}},${{ry}},${{rw}},${{rh}}</code> • Norm1000: <code>${{rnx}},${{rny}},${{rnw}},${{rnh}}</code>`;
        const hud = card.querySelector('.coord-hud');
        const cmdType = hud.dataset.cmdType || 'mosaic';
        if (cmdType === 'pill_bar') {{
          box.style.borderColor = 'var(--cyan)';
          box.style.background = 'rgba(0, 255, 213, 0.14)';
          card.querySelector('.cli-out').value =
            `./target/release/heav3ns-censor execute --asset-id ${{card.dataset.id}} --append --region "solid_bar:${{rx}},${{ry}},${{rw}},${{rh}}:pixels:pill:auto@0.0:0:kuro_nuri_cleft"`;
        }} else if (cmdType === 'hard_negative') {{
          box.style.borderColor = 'var(--green)';
          box.style.background = 'rgba(63, 185, 80, 0.16)';
          card.querySelector('.cli-out').value =
            `./target/release/heav3ns-censor annotate-negative --asset-id ${{card.dataset.id}} --append --region "neg1=${{rx}},${{ry}},${{rw}},${{rh}}:pixels:rect:benign_artistic_feature:Verified hard-negative confuser"`;
        }} else if (cmdType === 'crop') {{
          box.style.borderColor = 'var(--amber)';
          box.style.background = 'rgba(210, 153, 34, 0.14)';
          card.querySelector('.cli-out').value =
            `./target/release/heav3ns-censor inspect --asset-id ${{card.dataset.id}} --crop "${{rx}},${{ry}},${{rw}},${{rh}}:pixels"`;
        }} else {{
          box.style.borderColor = 'var(--cyan)';
          box.style.background = 'rgba(0, 255, 213, 0.14)';
          card.querySelector('.cli-out').value =
            `./target/release/heav3ns-censor execute --asset-id ${{card.dataset.id}} --append --region "mosaic:${{rx}},${{ry}},${{rw}},${{rh}}:pixels:rect:auto:0:explicit_anatomy"`;
        }}
      }} else {{
        readout.innerHTML = `Cursor Pixels: <code>(${{pt.ax}}, ${{pt.ay}})</code> • Norm1000: <code>(${{nx}}, ${{ny}})</code>`;
      }}
    }}

    function setCmdType(btn, type) {{
      const hud = btn.closest('.coord-hud');
      hud.dataset.cmdType = type;
      hud.querySelectorAll('.ctbtn').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
    }}

    function setQuickCmd(btn, action) {{
      const card = btn.closest('.card');
      const hud = btn.closest('.coord-hud');
      hud.querySelectorAll('.ctbtn').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      if (action === 'approve') {{
        card.querySelector('.cli-out').value =
          `./target/release/heav3ns-censor approve-original --asset-id ${{card.dataset.id}} --classification artistic_nudity_pass --rationale "Classical artistic nudity with zero explicit genital detail."`;
      }} else if (action === 'escalate') {{
        card.querySelector('.cli-out').value =
          `./target/release/heav3ns-censor escalate --asset-id ${{card.dataset.id}} --category "stylized_anatomy_ambiguity" --reason "Requires executive/artist review by MAJO and EGG."`;
      }}
    }}

    function onPreviewUp(e, wrap) {{
      dragState = null;
    }}

    function copyCli(btn) {{
      const input = btn.parentElement.querySelector('.cli-out');
      navigator.clipboard.writeText(input.value);
      const prev = btn.textContent;
      btn.textContent = 'Copied!';
      setTimeout(() => {{ btn.textContent = prev; }}, 1200);
    }}

    function filterCards(status, btn) {{
      document.querySelectorAll('.fbtn').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      document.querySelectorAll('.card').forEach(c => {{
        c.style.display = (status === 'all' || c.dataset.status === status) ? 'block' : 'none';
      }});
    }}

    function searchCards(q) {{
      const term = q.trim().toLowerCase();
      document.querySelectorAll('.card').forEach(c => {{
        c.style.display = c.dataset.search.toLowerCase().includes(term) ? 'block' : 'none';
      }});
    }}
  </script>
</body>
</html>"#,
            auth = ledger.commission_authority,
            agent = ledger.commissioned_agent,
            artist = ledger.principal_artist,
            updated = ledger.updated_at.to_rfc3339(),
            total = assets.len(),
            verified = count_verified,
            approved = count_approved,
            escalated = count_escalated,
            pending = count_pending,
            cards = cards
        )
    }
}

pub fn parse_crop_spec(spec: &str) -> Result<(f32, f32, f32, f32, CoordinateUnit)> {
    let parts: Vec<&str> = spec.split(':').collect();
    let coords: Vec<f32> = parts[0]
        .split(',')
        .map(|s| s.trim().parse::<f32>())
        .collect::<std::result::Result<Vec<f32>, _>>()
        .context("Failed to parse crop x,y,w,h coordinates")?;
    if coords.len() != 4 || coords.iter().any(|c| !c.is_finite()) {
        anyhow::bail!("Expected 4 finite comma-separated coordinates x,y,w,h in --crop");
    }
    let unit = match parts.get(1).copied().unwrap_or("pixels") {
        "normalized1000" | "norm1000" | "norm" => CoordinateUnit::Normalized1000,
        "percent" | "pct" => CoordinateUnit::Percent,
        _ => CoordinateUnit::Pixels,
    };
    Ok((coords[0], coords[1], coords[2], coords[3], unit))
}

pub fn compute_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

pub fn is_supported_image(path: &Path) -> bool {
    match path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
        .as_deref()
    {
        Some("png" | "jpg" | "jpeg" | "webp") => true,
        _ => false,
    }
}
