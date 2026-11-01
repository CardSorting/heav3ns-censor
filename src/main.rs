#![recursion_limit = "512"]
pub mod demo;
pub mod engine;
pub mod mcp;
pub mod models;
pub mod policy;
pub mod workspace;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use models::{
    CensorshipClassification, CensorshipMethod, CensorshipRegion, CoordinateUnit,
    NegativeAnnotationRegion, RegionShape,
};
use policy::get_policy_guides;
use serde_json::json;
use std::path::PathBuf;
use workspace::{ExecuteOptions, Workspace};

#[derive(Parser)]
#[command(
    name = "heav3ns-censor",
    version = "4.0.0",
    about = "Autonomous Japanese-Style Censorship & Publication Post-Processing Agentic Studio Engine for HEAV3NS",
    after_help = "ROLE-BASED QUICKSTART EXAMPLES:\n  # 1. Interactive Role Onboarding (agent | developer | stakeholder | all):\n  heav3ns-censor onboard --role agent\n\n  # 2. Batch-Ingest & Render VLM Coordinate Grids + Candidate Hotspots:\n  heav3ns-censor batch-prepare\n\n  # 3. 3x Magnified Zoom-Crop with Burned-In Global Pixel Coordinates:\n  heav3ns-censor inspect --asset-id heav3ns_01 --crop \"420,480,160,140:pixels\"\n\n  # 4. Execute Localized Censorship (Mosaic + Rotated Pill Bar with 'auto' contrast):\n  heav3ns-censor execute --asset-id heav3ns_01 \\\n    --region \"r1=mosaic:440,500,120,100:pixels:rect:14:0:explicit_lower_anatomy\" \\\n    --region \"r2=solid_bar:444,314,112,52:pixels:pill:auto@6.0:0:kuro_nuri_cleft_bar\"\n\n  # 5. Cryptographic SHA-256, Bit-Exact Preservation, & Hygiene Audit:\n  heav3ns-censor doctor --fix"
)]
struct Cli {
    /// Workspace root directory (defaults to current working directory)
    #[arg(long, global = true, env = "HEAV3NS_WORKSPACE")]
    workspace: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize the HEAV3NS post-production workspace directory structure and ledger
    Init,

    /// Output current workspace pipeline status, asset queues, next recommended action, and ledger paths as JSON
    Status,

    /// Generate a context-aware, role-specific onboarding & handoff guide (`agent`, `developer`, `stakeholder`, or `all`)
    Onboard {
        /// Target persona: `agent`, `developer`, `stakeholder`, or `all`
        #[arg(long, default_value = "all")]
        role: String,
    },

    /// Output a comprehensive diagnostic, cryptographic integrity, and handoff explanation for a single asset
    Explain {
        /// Asset ID, unique prefix, SHA-256 prefix, or filename in the ledger
        #[arg(long)]
        asset_id: String,
    },

    /// Run a comprehensive cryptographic (SHA-256), bit-exact preservation, escalation-isolation, and orphan-hygiene audit
    Doctor {
        /// Automatically repair missing directories, prune orphaned files, and re-sync reports
        #[arg(long)]
        fix: bool,
    },

    /// Output Japanese obscenity requirements (Article 175) & platform censorship policy profiles as JSON
    PolicyGuide,

    /// Ingest incoming artwork from assets/inbox/ (or a specific file), preserving untouched originals with SHA-256 verification
    Ingest {
        /// Optional specific image file path to ingest
        #[arg(long)]
        file: Option<PathBuf>,
        /// Default policy profile ID
        #[arg(long, default_value = "jp_article_175_standard")]
        policy: String,
    },

    /// Ingest all new inbox files and automatically generate coordinate inspection grids + hotspot hints for all pending assets
    BatchPrepare {
        /// Default policy profile ID
        #[arg(long, default_value = "jp_article_175_standard")]
        policy: String,
    },

    /// Generate a coordinate inspection overlay grid (with burned-in Normalized1000 100..900 badges & hotspot detection) for an asset
    Inspect {
        /// Asset ID, unique prefix, or filename in the ledger
        #[arg(long)]
        asset_id: String,

        /// Optional 3x magnified zoom-crop spec: `x,y,w,h[:unit]` (e.g. `420,480,160,140:pixels` or `350,450,250,250:normalized1000`)
        #[arg(long)]
        crop: Option<String>,
    },

    /// Execute localized censorship treatments on an asset, verify bit-exact preservation & 1080p/540p JPEG resilience, and export the publication edition + 4-panel QC Slate
    Execute {
        /// Asset ID, unique prefix, or filename in the ledger
        #[arg(long)]
        asset_id: String,

        /// Full JSON array of CensorshipRegion objects (alternative to CLI region flags)
        #[arg(long)]
        regions_json: Option<String>,

        /// Quick CLI shorthand for adding or updating one or more regions:
        /// Format: `[id=]method:x,y,w,h[:unit][:shape][:param[@angle_deg]][:feather][:label]`
        /// Shapes: `rect`, `ellipse`, `pill` (round-capped capsule)
        /// SolidBar color: `#000000` (kuro-nuri), `#FFFFFF` (shiro-nuri), or `auto` (luminance-adaptive)
        /// Examples:
        ///   `mosaic:440,500,120,100:pixels:rect:14:0:explicit_anatomy`
        ///   `r2=solid_bar:450,320,100,40:pixels:pill:auto@12:0:adaptive_bar`
        ///   `gaussian_blur:300,400,150,150:normalized1000:ellipse:12.0@-15:4:soft_obscure`
        #[arg(long = "region")]
        region_specs: Vec<String>,

        /// Incrementally merge/upsert `--region` specs by ID into the asset's existing regions rather than replacing all regions
        #[arg(long)]
        append: bool,

        /// Remove one or more existing region IDs (e.g. `--remove-region r2`) before applying updates
        #[arg(long = "remove-region")]
        remove_region_ids: Vec<String>,

        /// Simulate censorship and run full multi-scale verification without writing to `published/` or mutating the ledger
        #[arg(long)]
        dry_run: bool,

        /// Optional policy override
        #[arg(long)]
        policy: Option<String>,
    },

    /// Approve an asset as requiring no censorship (artistic nudity or suggestive imagery) and export untouched original to published/
    ApproveOriginal {
        /// Asset ID, unique prefix, or filename in the ledger
        #[arg(long)]
        asset_id: String,

        /// Classification: `artistic_nudity_pass` or `suggestive_pass`
        #[arg(long, default_value = "artistic_nudity_pass")]
        classification: String,

        /// Detailed artistic preservation & policy compliance rationale
        #[arg(long)]
        rationale: String,
    },

    /// Escalate an asset for human/executive review when artistic or compliance requirements are ambiguous
    Escalate {
        /// Asset ID, unique prefix, or filename in the ledger
        #[arg(long)]
        asset_id: String,

        /// Ambiguity category (e.g., `stylized_anatomy_ambiguity`, `composition_impact`, `platform_policy_conflict`)
        #[arg(long, default_value = "artistic_or_compliance_ambiguity")]
        category: String,

        /// Clear explanation of why the asset requires human determination
        #[arg(long)]
        reason: String,

        /// Additional technical or inspection notes from the agent
        #[arg(long, default_value = "")]
        notes: String,
    },

    /// Reset an asset back to PendingInspection, removing published/preview/escalation artifacts while preserving the untouched original
    Reset {
        /// Asset ID, unique prefix, or filename in the ledger
        #[arg(long)]
        asset_id: String,
    },

    /// Re-run verification and Markdown/HTML/JSONL report generation across the ledger
    SyncReports,

    /// Export a HuggingFace / JSONL dataset manifest (`ledger/dataset_manifest.jsonl`), MS-COCO JSON (`ledger/annotations_coco.json`), and Dataset Card (`ledger/DATASET_CARD.md`) with positive & hard-negative bounding boxes
    ExportDataset {
        /// Export only compliance-verified training records (`Verified` and `ApprovedOriginal`), excluding `Escalated` or pending items
        #[arg(long)]
        safe_only: bool,
    },

    /// Annotate verified benign / hard-negative regions on an asset (`annotate-negative`) without modifying any pixels, preventing false positives in downstream ML/VLM classifiers
    AnnotateNegative {
        /// Asset ID, unique prefix, or filename in the ledger
        #[arg(long)]
        asset_id: String,

        /// Shorthand hard-negative region spec: `[id=]x,y,w,h[:unit][:shape][:label][:rationale]`
        /// Example: `neg1=320,180,360,640:pixels:rect:classical_dorsal_silhouette:Verified benign artistic contour`
        #[arg(long = "region")]
        region_specs: Vec<String>,

        /// Incrementally merge/upsert `--region` specs into existing hard-negative annotations
        #[arg(long)]
        append: bool,

        /// Remove one or more existing hard-negative region IDs (e.g. `--remove-region neg1`)
        #[arg(long = "remove-region")]
        remove_region_ids: Vec<String>,

        /// Automatically mine non-overlapping high-frequency hotspots as verified benign hard-negative boxes
        #[arg(long)]
        auto_mine_hotspots: bool,
    },

    /// Generate synthetic HEAV3NS test assets in assets/inbox/ to demonstrate and test the autonomous pipeline
    DemoAssets,

    /// Start Model Context Protocol (MCP) stdio JSON-RPC server for autonomous agent tool integration
    Mcp,
}

pub fn parse_shorthand_region(spec: &str, idx: usize) -> Result<CensorshipRegion> {
    let parts: Vec<&str> = spec.split(':').collect();
    if parts.len() < 2 {
        anyhow::bail!(
            "Invalid --region spec '{}'. Expected format [id=]method:x,y,w,h[:unit][:shape][:param[@angle_deg]][:feather][:label]",
            spec
        );
    }
    let (explicit_id, method_name) = if let Some((id_part, m_part)) = parts[0].split_once('=') {
        let eid = id_part.trim();
        let m = m_part.trim().to_lowercase();
        (
            if eid.is_empty() {
                None
            } else {
                Some(eid.to_string())
            },
            m,
        )
    } else {
        (None, parts[0].trim().to_lowercase())
    };

    let coords: Vec<f32> = parts[1]
        .split(',')
        .map(|s| s.trim().parse::<f32>())
        .collect::<std::result::Result<Vec<f32>, _>>()
        .context("Failed to parse x,y,w,h coordinates in --region")?;
    if coords.len() != 4 || coords.iter().any(|c| !c.is_finite()) {
        anyhow::bail!(
            "Expected 4 finite comma-separated coordinates x,y,w,h in '{}'",
            parts[1]
        );
    }

    let unit = match parts.get(2).copied().unwrap_or("pixels") {
        "normalized1000" | "norm1000" | "norm" => CoordinateUnit::Normalized1000,
        "percent" | "pct" => CoordinateUnit::Percent,
        _ => CoordinateUnit::Pixels,
    };

    let shape = match parts.get(3).copied().unwrap_or("rect") {
        "ellipse" | "oval" | "circle" => RegionShape::Ellipse,
        "pill" | "capsule" | "stadium" => RegionShape::Pill,
        _ => RegionShape::Rect,
    };

    let raw_param = parts.get(4).copied().unwrap_or("");
    let (param_str, angle_deg) = if let Some((p, a)) = raw_param.split_once('@') {
        let ang = a.parse::<f32>().unwrap_or(0.0);
        (p, if ang.is_finite() { ang } else { 0.0 })
    } else {
        (raw_param, 0.0)
    };

    let treatment = match method_name.as_str() {
        "mosaic" => {
            let bs = if param_str.is_empty() || param_str == "auto" {
                None
            } else {
                Some(param_str.parse::<u32>().unwrap_or(0))
            };
            CensorshipMethod::Mosaic { block_size: bs }
        }
        "gaussian_blur" | "blur" => {
            let sigma = if param_str.is_empty() {
                10.0
            } else {
                param_str.parse::<f32>().unwrap_or(10.0)
            };
            CensorshipMethod::GaussianBlur { sigma }
        }
        "solid_bar" | "bar" => {
            let color = if param_str.is_empty() {
                "#000000".to_string()
            } else {
                param_str.to_string()
            };
            CensorshipMethod::SolidBar { color, angle_deg }
        }
        other => anyhow::bail!(
            "Unknown censorship method '{}'. Use mosaic, gaussian_blur, or solid_bar.",
            other
        ),
    };

    let feather_px = parts
        .get(5)
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    let label = parts
        .get(6)
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("region_{}", idx + 1));

    Ok(CensorshipRegion {
        id: explicit_id.unwrap_or_else(|| format!("r{}", idx + 1)),
        label: label.clone(),
        rationale: format!("Localized {} intervention on {}", method_name, label),
        x: coords[0],
        y: coords[1],
        width: coords[2],
        height: coords[3],
        unit,
        shape,
        treatment,
        angle_deg,
        feather_px,
    })
}

/// Parses a verified benign hard-negative region shorthand:
/// `[id=]x,y,w,h[:unit][:shape][:label][:rationale]`
pub fn parse_negative_shorthand(spec: &str, idx: usize) -> Result<NegativeAnnotationRegion> {
    let parts: Vec<&str> = spec.split(':').collect();
    if parts.is_empty() {
        anyhow::bail!(
            "Invalid negative --region spec '{}'. Expected format [id=]x,y,w,h[:unit][:shape][:label][:rationale]",
            spec
        );
    }
    let (explicit_id, coord_part) = if let Some((id_part, c_part)) = parts[0].split_once('=') {
        let eid = id_part.trim();
        (
            if eid.is_empty() {
                None
            } else {
                Some(eid.to_string())
            },
            c_part.trim(),
        )
    } else {
        (None, parts[0].trim())
    };

    let coords: Vec<f32> = coord_part
        .split(',')
        .map(|s| s.trim().parse::<f32>())
        .collect::<std::result::Result<Vec<f32>, _>>()
        .context("Failed to parse x,y,w,h coordinates in negative --region")?;
    if coords.len() != 4 || coords.iter().any(|c| !c.is_finite()) {
        anyhow::bail!(
            "Expected 4 finite comma-separated coordinates x,y,w,h in '{}'",
            coord_part
        );
    }

    let unit = match parts.get(1).copied().unwrap_or("pixels") {
        "normalized1000" | "norm1000" | "norm" => CoordinateUnit::Normalized1000,
        "percent" | "pct" => CoordinateUnit::Percent,
        _ => CoordinateUnit::Pixels,
    };

    let shape = match parts.get(2).copied().unwrap_or("rect") {
        "ellipse" | "oval" | "circle" => RegionShape::Ellipse,
        "pill" | "capsule" | "stadium" => RegionShape::Pill,
        _ => RegionShape::Rect,
    };

    let label = parts
        .get(3)
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("benign_confuser_{}", idx + 1));

    let rationale = parts
        .get(4)
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            format!(
                "Verified benign artistic feature ({}) — hard-negative annotation to prevent classifier false positives",
                label
            )
        });

    Ok(NegativeAnnotationRegion {
        id: explicit_id.unwrap_or_else(|| format!("neg{}", idx + 1)),
        label: label.clone(),
        rationale,
        confuser_category: label,
        fp_hardness_score: 0.0,
        x: coords[0],
        y: coords[1],
        width: coords[2],
        height: coords[3],
        unit,
        shape,
    })
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = cli
        .workspace
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let workspace = Workspace::new(root);

    match cli.command {
        Commands::Init => {
            workspace.init()?;
            let status = workspace.status_summary()?;
            println!("{}", serde_json::to_string_pretty(&status)?);
        }
        Commands::Status => {
            let status = workspace.status_summary()?;
            println!("{}", serde_json::to_string_pretty(&status)?);
        }
        Commands::Onboard { role } => {
            let packet = workspace.onboard(&role)?;
            println!("{}", serde_json::to_string_pretty(&packet)?);
        }
        Commands::Explain { asset_id } => {
            let packet = workspace.explain_asset(&asset_id)?;
            println!("{}", serde_json::to_string_pretty(&packet)?);
        }
        Commands::Doctor { fix } => {
            let report = workspace.run_doctor(fix)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if !report.overall_healthy {
                std::process::exit(1);
            }
        }
        Commands::PolicyGuide => {
            let guides = get_policy_guides();
            println!("{}", serde_json::to_string_pretty(&guides)?);
        }
        Commands::Ingest { file, policy } => {
            if let Some(fp) = file {
                let rec = workspace.ingest_file(&fp, &policy)?;
                println!("{}", serde_json::to_string_pretty(&rec)?);
            } else {
                let recs = workspace.ingest_inbox(&policy)?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "ingested_count": recs.len(),
                        "assets": recs
                    }))?
                );
            }
        }
        Commands::BatchPrepare { policy } => {
            let payloads = workspace.batch_prepare(&policy)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "prepared_count": payloads.len(),
                    "inspection_packets": payloads
                }))?
            );
        }
        Commands::Inspect { asset_id, crop } => {
            let payload = workspace.inspect_asset_with_crop(&asset_id, crop.as_deref())?;
            println!("{}", serde_json::to_string_pretty(&payload)?);
        }
        Commands::Execute {
            asset_id,
            regions_json,
            region_specs,
            append,
            remove_region_ids,
            dry_run,
            policy,
        } => {
            let mut regions: Vec<CensorshipRegion> = Vec::new();
            if let Some(raw_json) = regions_json {
                let mut parsed: Vec<CensorshipRegion> = serde_json::from_str(&raw_json)
                    .context("Failed to parse --regions-json as Vec<CensorshipRegion>")?;
                regions.append(&mut parsed);
            }
            for (idx, spec) in region_specs.iter().enumerate() {
                let mut reg = parse_shorthand_region(spec, idx)?;
                let has_explicit_id = spec.split(':').next().unwrap_or("").contains('=');
                if append && !has_explicit_id {
                    reg.id = format!("auto_{}", idx + 1);
                }
                regions.push(reg);
            }
            if regions.is_empty() && remove_region_ids.is_empty() {
                anyhow::bail!(
                    "No censorship regions provided. Supply --regions-json, --region, or --remove-region."
                );
            }

            let (record, report) = workspace.execute_censorship_with_options(
                &asset_id,
                CensorshipClassification::ExplicitRequiresIntervention,
                regions,
                ExecuteOptions {
                    append_regions: append,
                    remove_region_ids,
                    dry_run,
                    policy_override: policy,
                },
            )?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": if dry_run { "dry_run_completed" } else { "completed" },
                    "dry_run": dry_run,
                    "asset": record,
                    "verification": report
                }))?
            );
        }
        Commands::ApproveOriginal {
            asset_id,
            classification,
            rationale,
        } => {
            let class_enum = match classification.as_str() {
                "suggestive_pass" => CensorshipClassification::SuggestivePass,
                _ => CensorshipClassification::ArtisticNudityPass,
            };
            let record = workspace.approve_untouched(&asset_id, class_enum, &rationale)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": "approved_original",
                    "asset": record
                }))?
            );
        }
        Commands::Escalate {
            asset_id,
            category,
            reason,
            notes,
        } => {
            let record = workspace.escalate_asset(&asset_id, &category, &reason, &notes)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": "escalated",
                    "asset": record
                }))?
            );
        }
        Commands::Reset { asset_id } => {
            let record = workspace.reset_asset(&asset_id)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": "reset_to_pending",
                    "asset": record
                }))?
            );
        }
        Commands::SyncReports => {
            workspace.sync_markdown_reports()?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": "reports_synced",
                    "publication_report": workspace.publication_manifest_md_file(),
                    "escalation_report": workspace.escalation_md_file(),
                    "dashboard_html": workspace.dashboard_html_file(),
                    "dataset_manifest_jsonl": workspace.dataset_manifest_jsonl_file(),
                    "coco_annotations_json": workspace.coco_annotations_json_file(),
                    "vlm_safety_sft_jsonl": workspace.vlm_sft_jsonl_file(),
                    "dataset_card_md": workspace.dataset_card_md_file()
                }))?
            );
        }
        Commands::ExportDataset { safe_only } => {
            let report = workspace.export_dataset(safe_only)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Commands::AnnotateNegative {
            asset_id,
            region_specs,
            append,
            remove_region_ids,
            auto_mine_hotspots,
        } => {
            let mut neg_regions = Vec::new();
            for (idx, spec) in region_specs.iter().enumerate() {
                let mut nr = parse_negative_shorthand(spec, idx)?;
                let has_explicit_id = spec.split(':').next().unwrap_or("").contains('=');
                if append && !has_explicit_id {
                    nr.id = format!("auto_{}", idx + 1);
                }
                neg_regions.push(nr);
            }
            let record = workspace.annotate_negative(
                &asset_id,
                neg_regions,
                append,
                remove_region_ids,
                auto_mine_hotspots,
            )?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": "hard_negative_annotations_updated",
                    "hard_negative_count": record.negative_regions.len(),
                    "asset": record
                }))?
            );
        }
        Commands::DemoAssets => {
            workspace.init()?;
            let created = demo::generate_demo_inbox_assets(&workspace.inbox_dir())?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": "demo_assets_created",
                    "files": created
                }))?
            );
        }
        Commands::Mcp => {
            workspace.init()?;
            mcp::run_mcp_stdio_server(&workspace)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};
    use std::env;

    #[test]
    fn test_mosaic_remainder_strip_pooling_and_peak_suppression() -> Result<()> {
        let mut orig = RgbaImage::new(400, 400);
        for y in 0..400 {
            for x in 0..400 {
                // High frequency detail inside non-multiple bounds (150..253, 150..247) -> 103x97
                // With block_size=12, 103 % 12 = 7 and 97 % 12 = 1 (1px remainder strip!)
                if x >= 150 && x < 253 && y >= 150 && y < 247 {
                    let v = if (x + y) % 2 == 0 { 245 } else { 15 };
                    orig.put_pixel(x, y, Rgba([v, 40, 80, 255]));
                } else {
                    orig.put_pixel(x, y, Rgba([(x % 255) as u8, (y % 255) as u8, 128, 255]));
                }
            }
        }

        let regions = vec![CensorshipRegion {
            id: "r1".to_string(),
            label: "explicit_anatomy".to_string(),
            rationale: "Test remainder-strip edge-pooled mosaic".to_string(),
            x: 150.0,
            y: 150.0,
            width: 103.0,
            height: 97.0,
            unit: CoordinateUnit::Pixels,
            shape: RegionShape::Rect,
            treatment: CensorshipMethod::Mosaic {
                block_size: Some(12),
            },
            angle_deg: 0.0,
            feather_px: 0,
        }];

        let censored =
            engine::apply_censorship_regions(&orig, &regions, "jp_article_175_standard");
        let tmp_dir = env::temp_dir().join("heav3ns_test_v3_mosaic");
        let report = engine::verify_censored_edition(
            &orig,
            &censored,
            &regions,
            "jp_article_175_standard",
            &tmp_dir,
            "test_asset",
        )?;

        assert!(report.artistic_preservation_exact);
        assert_eq!(report.non_censored_pixel_diff_count, 0);
        assert!(report.overall_passed);
        assert!(report.region_metrics[0].detail_suppression_ratio > 0.95);
        assert!(report.region_metrics[0].peak_detail_suppression_ratio > 0.95);
        Ok(())
    }

    #[test]
    fn test_rotated_pill_kuro_nuri_and_angled_mosaic() -> Result<()> {
        let mut orig = RgbaImage::new(500, 500);
        for y in 0..500 {
            for x in 0..500 {
                let v = if (x * 3 + y * 5) % 2 == 0 { 220 } else { 30 };
                orig.put_pixel(x, y, Rgba([v, 100, 150, 255]));
            }
        }

        // Parse a 35-degree rotated Pill kuro-nuri bar AND a -20-degree rotated Ellipse mosaic
        let r1 = parse_shorthand_region(
            "solid_bar:180,140,120,30:pixels:pill:#000000@35.0:0:angled_pill_bar",
            0,
        )?;
        let r2 = parse_shorthand_region(
            "mosaic:180,300,110,70:pixels:ellipse:12@-20.0:0:angled_ellipse_mosaic",
            1,
        )?;
        let regions = vec![r1, r2];

        let censored = engine::apply_censorship_regions(&orig, &regions, "kuro_nuri_graphic");
        let tmp_dir = env::temp_dir().join("heav3ns_test_v3_rot");
        let report = engine::verify_censored_edition(
            &orig,
            &censored,
            &regions,
            "kuro_nuri_graphic",
            &tmp_dir,
            "test_rotated_shapes",
        )?;

        assert!(report.artistic_preservation_exact);
        assert_eq!(report.non_censored_pixel_diff_count, 0);
        assert!(report.overall_passed);
        Ok(())
    }

    #[test]
    fn test_transparent_png_alpha_preservation() -> Result<()> {
        // Semi-transparent character artwork (alpha = 128)
        let mut orig = RgbaImage::from_pixel(200, 200, Rgba([200, 100, 150, 128]));
        for y in 60..140 {
            for x in 60..140 {
                let v = if (x + y) % 2 == 0 { 240 } else { 40 };
                orig.put_pixel(x, y, Rgba([v, 80, 120, 128]));
            }
        }
        let r = parse_shorthand_region("mosaic:60,60,80,80:pixels:rect:10:0:trans_mosaic", 0)?;
        let censored = engine::apply_censorship_regions(&orig, &[r], "jp_article_175_standard");

        // Alpha inside the mosaicked region must remain 128 (not forced to 255!)
        assert_eq!(censored.get_pixel(100, 100)[3], 128);
        // Pixels outside remain untouched
        assert_eq!(censored.get_pixel(10, 10), orig.get_pixel(10, 10));
        Ok(())
    }

    #[test]
    fn test_escalation_isolation_revokes_published_file() -> Result<()> {
        let tmp_ws = env::temp_dir().join(format!("heav3ns_ws_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp_ws);
        let ws = Workspace::new(&tmp_ws);
        ws.init()?;
        demo::generate_demo_inbox_assets(&ws.inbox_dir())?;
        let ingested = ws.ingest_inbox("jp_article_175_standard")?;
        let target_id = &ingested[0].asset_id;

        // First approve original -> creates published file
        let approved = ws.approve_untouched(
            target_id,
            CensorshipClassification::ArtisticNudityPass,
            "Initial pass",
        )?;
        let pub_file = PathBuf::from(approved.published_edition_path.unwrap());
        assert!(pub_file.exists());

        // Now escalate the same asset -> must revoke and remove the published file!
        let escalated = ws.escalate_asset(
            target_id,
            "compliance_review",
            "Re-classified as ambiguous",
            "Testing revocation",
        )?;
        assert!(escalated.published_edition_path.is_none());
        assert!(!pub_file.exists(), "Published file must be deleted when escalated");

        let doc = ws.run_doctor(false)?;
        assert!(doc.overall_healthy);

        let _ = std::fs::remove_dir_all(&tmp_ws);
        Ok(())
    }

    #[test]
    fn test_auto_contrast_solid_bar_luminance_adaptation() -> Result<()> {
        let mut orig = RgbaImage::new(300, 300);
        for y in 0..300 {
            for x in 0..300 {
                if y < 150 {
                    // Dark region (mean lum < 90) -> auto bar must choose shiro-nuri (#FFFFFF)
                    let v = if (x + y) % 2 == 0 { 50 } else { 10 };
                    orig.put_pixel(x, y, Rgba([v, v, v, 255]));
                } else {
                    // Bright region (mean lum >= 90) -> auto bar must choose kuro-nuri (#000000)
                    let v = if (x + y) % 2 == 0 { 240 } else { 180 };
                    orig.put_pixel(x, y, Rgba([v, v, v, 255]));
                }
            }
        }

        let r_dark = parse_shorthand_region(
            "dark_bar=solid_bar:60,40,120,40:pixels:pill:auto:0:dark_zone",
            0,
        )?;
        let r_light = parse_shorthand_region(
            "light_bar=solid_bar:60,200,120,40:pixels:pill:auto:0:light_zone",
            1,
        )?;
        let censored =
            engine::apply_censorship_regions(&orig, &[r_dark, r_light], "kuro_nuri_graphic");

        // Center of dark_bar (120, 60) must be pure white (#FFFFFF)
        assert_eq!(censored.get_pixel(120, 60), &Rgba([255, 255, 255, 255]));
        // Center of light_bar (120, 220) must be pure black (#000000)
        assert_eq!(censored.get_pixel(120, 220), &Rgba([0, 0, 0, 255]));
        Ok(())
    }

    #[test]
    fn test_excessive_feather_clamping_preserves_opaque_core() -> Result<()> {
        let mut orig = RgbaImage::new(300, 300);
        for y in 0..300 {
            for x in 0..300 {
                let v = if (x + y) % 2 == 0 { 250 } else { 10 };
                orig.put_pixel(x, y, Rgba([v, 100, 120, 255]));
            }
        }

        // Request an absurd 200px feather on a 60x60 region; engine clamps feather to <= 45% of half-axis
        let r = parse_shorthand_region("solid_bar:100,100,60,60:pixels:rect:#000000:200:clamped_feather", 0)?;
        let censored = engine::apply_censorship_regions(&orig, &[r.clone()], "kuro_nuri_graphic");
        let tmp_dir = env::temp_dir().join("heav3ns_test_v4_feather");
        let report = engine::verify_censored_edition(
            &orig,
            &censored,
            &[r],
            "kuro_nuri_graphic",
            &tmp_dir,
            "test_feather_clamp",
        )?;

        assert!(report.overall_passed);
        assert_eq!(censored.get_pixel(130, 130), &Rgba([0, 0, 0, 255]));
        let _ = std::fs::remove_dir_all(&tmp_dir);
        Ok(())
    }

    #[test]
    fn test_degenerate_tiny_region_rejected_by_verifier() -> Result<()> {
        let mut orig = RgbaImage::new(200, 200);
        for y in 0..200 {
            for x in 0..200 {
                let v = if (x + y) % 2 == 0 { 240 } else { 20 };
                orig.put_pixel(x, y, Rgba([v, v, v, 255]));
            }
        }

        // A degenerate 2x2 pixel region has 0 interior samples after 1px erosion and must NOT pass verification
        let r = parse_shorthand_region("solid_bar:50,50,2,2:pixels:rect:#000000:0:tiny_degenerate", 0)?;
        let censored = engine::apply_censorship_regions(&orig, &[r.clone()], "kuro_nuri_graphic");
        let tmp_dir = env::temp_dir().join("heav3ns_test_v4_tiny");
        let report = engine::verify_censored_edition(
            &orig,
            &censored,
            &[r],
            "kuro_nuri_graphic",
            &tmp_dir,
            "test_tiny",
        )?;

        assert!(
            !report.overall_passed,
            "Degenerate 2x2 region with 0 interior samples must fail verification"
        );
        let _ = std::fs::remove_dir_all(&tmp_dir);
        Ok(())
    }

    #[test]
    fn test_incremental_region_append_upsert_remove_and_dry_run() -> Result<()> {
        let tmp_ws = env::temp_dir().join(format!("heav3ns_ws_v4_inc_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp_ws);
        let ws = Workspace::new(&tmp_ws);
        ws.init()?;
        demo::generate_demo_inbox_assets(&ws.inbox_dir())?;
        let ingested = ws.ingest_inbox("jp_article_175_standard")?;
        let target_id = &ingested[0].asset_id;

        // 1. Initial execute with r1
        let r1 = parse_shorthand_region("mosaic:440,500,120,100:pixels:rect:14:0:explicit_anatomy", 0)?;
        let (rec1, _) = ws.execute_censorship(
            target_id,
            CensorshipClassification::ExplicitRequiresIntervention,
            vec![r1],
            None,
        )?;
        assert_eq!(rec1.regions.len(), 1);
        assert_eq!(rec1.regions[0].id, "r1");

        // 2. Append a second region (auto-assigned r2)
        let mut r_add = parse_shorthand_region("solid_bar:444,314,112,52:pixels:pill:auto@6.0:0:cleft_bar", 0)?;
        r_add.id = "auto_1".to_string();
        let (rec2, _) = ws.execute_censorship_with_options(
            target_id,
            CensorshipClassification::ExplicitRequiresIntervention,
            vec![r_add],
            ExecuteOptions {
                append_regions: true,
                ..Default::default()
            },
        )?;
        assert_eq!(rec2.regions.len(), 2);
        assert_eq!(rec2.regions[0].id, "r1");
        assert_eq!(rec2.regions[1].id, "r2");

        // 3. Dry-run removing r1 and adding r3 -> must NOT mutate saved ledger!
        let r_dry = parse_shorthand_region("r3=mosaic:300,300,80,80:pixels:rect:14:0:dry_test", 0)?;
        let (sim_rec, sim_ver) = ws.execute_censorship_with_options(
            target_id,
            CensorshipClassification::ExplicitRequiresIntervention,
            vec![r_dry],
            ExecuteOptions {
                append_regions: true,
                remove_region_ids: vec!["r1".to_string()],
                dry_run: true,
                policy_override: None,
            },
        )?;
        assert!(sim_ver.overall_passed);
        assert_eq!(sim_rec.regions.len(), 2); // r2 and r3 in dry-run simulation
        // Verify ledger on disk still has r1 and r2 untouched
        let reloaded = ws.load_ledger()?;
        assert_eq!(reloaded.assets[target_id].regions.len(), 2);
        assert_eq!(reloaded.assets[target_id].regions[0].id, "r1");

        // 4. Actually remove r2 via remove_region_ids
        let (rec3, _) = ws.execute_censorship_with_options(
            target_id,
            CensorshipClassification::ExplicitRequiresIntervention,
            Vec::new(),
            ExecuteOptions {
                append_regions: true,
                remove_region_ids: vec!["r2".to_string()],
                dry_run: false,
                policy_override: None,
            },
        )?;
        assert_eq!(rec3.regions.len(), 1);
        assert_eq!(rec3.regions[0].id, "r1");

        // 5. Verify onboard and explain handoff packets
        let onboard_pkt = ws.onboard("agent")?;
        assert_eq!(onboard_pkt.selected_role, "agent");
        assert_eq!(onboard_pkt.engine_version, "4.0.0");

        let explain_pkt = ws.explain_asset(target_id)?;
        assert!(explain_pkt.cryptographic_integrity_verified);
        assert!(explain_pkt.bit_exact_preservation_verified);
        assert_eq!(explain_pkt.active_regions_count, 1);

        let _ = std::fs::remove_dir_all(&tmp_ws);
        Ok(())
    }

    #[test]
    fn test_dataset_cleaning_jsonl_export_and_safety_filtering() -> Result<()> {
        let tmp_ws = env::temp_dir().join(format!("heav3ns_ws_v4_ds_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp_ws);
        let ws = Workspace::new(&tmp_ws);
        ws.init()?;
        demo::generate_demo_inbox_assets(&ws.inbox_dir())?;
        let ingested = ws.ingest_inbox("jp_article_175_standard")?;
        assert_eq!(ingested.len(), 3);

        // 1. Surgical redaction on asset 0 (Verified)
        let r1 = parse_shorthand_region("mosaic:440,500,120,100:pixels:rect:14:0:explicit_anatomy", 0)?;
        ws.execute_censorship(
            &ingested[0].asset_id,
            CensorshipClassification::ExplicitRequiresIntervention,
            vec![r1],
            None,
        )?;

        // 2. Preserve false-positive artistic nudity on asset 1 (ApprovedOriginal)
        ws.approve_untouched(
            &ingested[1].asset_id,
            CensorshipClassification::ArtisticNudityPass,
            "Classical silhouette",
        )?;

        // 3. Quarantine ambiguous asset 2 (Escalated)
        ws.escalate_asset(
            &ingested[2].asset_id,
            "stylized_anatomy_ambiguity",
            "Requires review",
            "Quarantined",
        )?;

        // Export full contrastive benchmark (safe_only = false) -> 3 records
        let full_ds = ws.export_dataset(false)?;
        assert_eq!(full_ds.total_records_exported, 3);
        assert_eq!(full_ds.safe_training_records_count, 2);
        assert_eq!(full_ds.surgical_redaction_count, 1);
        assert_eq!(full_ds.preserved_untouched_count, 1);
        assert_eq!(full_ds.quarantined_escalated_count, 1);
        assert_eq!(full_ds.records[0].annotations[0].bbox_xywh_px, [440, 500, 120, 100]);
        assert_eq!(full_ds.records[0].exterior_pixel_diff_count, 0);

        // Export safe-only training split (safe_only = true) -> 2 records (excludes Escalated)
        let safe_ds = ws.export_dataset(true)?;
        assert_eq!(safe_ds.total_records_exported, 2);
        assert!(safe_ds.records.iter().all(|r| r.is_safe_for_training_release));

        // Verify researcher onboarding track
        let res_onboard = ws.onboard("researcher")?;
        assert_eq!(res_onboard.selected_role, "researcher");

        let _ = std::fs::remove_dir_all(&tmp_ws);
        Ok(())
    }

    #[test]
    fn test_hard_negative_annotation_and_coco_export_prevents_false_positives() -> Result<()> {
        let tmp_ws = env::temp_dir().join(format!("heav3ns_ws_v4_neg_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp_ws);
        let ws = Workspace::new(&tmp_ws);
        ws.init()?;
        demo::generate_demo_inbox_assets(&ws.inbox_dir())?;
        let ingested = ws.ingest_inbox("jp_article_175_standard")?;

        // 1. Approve artistic silhouette (asset 1) -> auto-mines hard-negative contour hotspots
        let art_rec = ws.approve_untouched(
            &ingested[1].asset_id,
            CensorshipClassification::ArtisticNudityPass,
            "Classical dorsal silhouette",
        )?;
        assert!(
            !art_rec.negative_regions.is_empty(),
            "ApprovedOriginal asset must auto-mine benign hard-negative hotspots"
        );

        // 2. Explicitly annotate a verified benign confuser region via annotate_negative
        let custom_neg = parse_negative_shorthand(
            "neg_custom=280,140,440,720:pixels:rect:classical_dorsal_silhouette:Stylized dorsal anatomy with zero explicit detail",
            0,
        )?;
        let updated_art = ws.annotate_negative(
            &ingested[1].asset_id,
            vec![custom_neg],
            true,
            Vec::new(),
            false,
        )?;
        assert!(updated_art
            .negative_regions
            .iter()
            .any(|n| n.id == "neg_custom"));

        // 3. Export dataset and verify MS-COCO JSON + hard-negative polarity (class_id == 0)
        let ds = ws.export_dataset(true)?;
        assert!(ds.total_hard_negative_boxes >= 2);
        assert!(std::path::Path::new(&ds.coco_annotations_path).exists());
        let coco_raw = std::fs::read_to_string(&ds.coco_annotations_path)?;
        let coco_val: serde_json::Value = serde_json::from_str(&coco_raw)?;
        assert_eq!(coco_val["categories"].as_array().unwrap().len(), 2);
        assert!(
            coco_val["annotations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|a| a["polarity"] == "verified_benign_hard_negative")
        );

        let _ = std::fs::remove_dir_all(&tmp_ws);
        Ok(())
    }

    #[test]
    fn test_polarity_conflict_prevention_and_confuser_hardness_scoring() -> Result<()> {
        let tmp_ws = env::temp_dir().join(format!("heav3ns_ws_v4_pol_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp_ws);
        let ws = Workspace::new(&tmp_ws);
        ws.init()?;
        demo::generate_demo_inbox_assets(&ws.inbox_dir())?;
        let ingested = ws.ingest_inbox("jp_article_175_standard")?;
        let target_id = &ingested[0].asset_id;

        // 1. Apply positive intervention region at (440, 500, 120, 100)
        let r1 = parse_shorthand_region("mosaic:440,500,120,100:pixels:rect:14:0:explicit_anatomy", 0)?;
        let (rec1, _) = ws.execute_censorship(
            target_id,
            CensorshipClassification::ExplicitRequiresIntervention,
            vec![r1],
            None,
        )?;
        assert_eq!(rec1.regions.len(), 1);
        for nr in &rec1.negative_regions {
            assert!(
                nr.fp_hardness_score > 0.0,
                "Auto-mined hard-negative must have a positive confuser hardness score"
            );
        }

        // 2. Attempt to add a conflicting negative box right on top of the positive box (440, 500, 120, 100)
        // AND a valid non-overlapping benign hard-negative box at (100, 100, 160, 160)
        let neg_conflict = parse_negative_shorthand(
            "neg_bad=445,505,110,90:pixels:rect:conflicting_box:Should be auto-rejected due to IoU >= 0.20 with r1",
            0,
        )?;
        let neg_valid = parse_negative_shorthand(
            "neg_good=100,100,160,160:pixels:rect:benign_artistic_feature:Valid non-overlapping hard negative",
            1,
        )?;
        let rec2 = ws.annotate_negative(
            target_id,
            vec![neg_conflict, neg_valid],
            true,
            Vec::new(),
            false,
        )?;

        assert!(
            !rec2.negative_regions.iter().any(|n| n.id == "neg_bad"),
            "Polarity collision prevention must reject hard-negative box overlapping positive region"
        );
        let good_neg = rec2
            .negative_regions
            .iter()
            .find(|n| n.id == "neg_good")
            .expect("Non-overlapping hard-negative box must be retained");
        assert!(good_neg.fp_hardness_score >= 5.0);
        assert_ne!(good_neg.confuser_category, "benign_artistic_feature");

        // 3. Verify Doctor check passes with zero polarity collisions
        let doc = ws.run_doctor(false)?;
        assert!(doc.overall_healthy);
        assert!(doc
            .checks
            .iter()
            .any(|c| c.check_name.starts_with("annotation_polarity_conflict:") && c.passed));

        // 4. Verify VLM Safety SFT JSONL export
        let ds = ws.export_dataset(true)?;
        assert!(std::path::Path::new(&ds.vlm_sft_jsonl_path).exists());
        let sft_raw = std::fs::read_to_string(&ds.vlm_sft_jsonl_path)?;
        assert!(sft_raw.contains("verified_benign_hard_negatives"));
        assert!(sft_raw.contains("fp_hardness_score"));

        let _ = std::fs::remove_dir_all(&tmp_ws);
        Ok(())
    }
}
