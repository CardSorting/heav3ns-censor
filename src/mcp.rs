use crate::policy::get_policy_guides;
use crate::workspace::Workspace;
use anyhow::Result;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

pub fn run_mcp_stdio_server(workspace: &Workspace) -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let req: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                let err_resp = json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": { "code": -32700, "message": format!("Parse error: {}", e) }
                });
                writeln!(stdout, "{}", serde_json::to_string(&err_resp)?)?;
                stdout.flush()?;
                continue;
            }
        };

        let id = req.get("id").cloned();
        let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");

        if id.is_none() {
            continue;
        }
        let id_val = id.unwrap();

        let response = match method {
            "initialize" => json!({
                "jsonrpc": "2.0",
                "id": id_val,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "heav3ns-censor-mcp",
                        "version": "4.0.0"
                    }
                }
            }),
            "tools/list" => json!({
                "jsonrpc": "2.0",
                "id": id_val,
                "result": {
                    "tools": mcp_tool_definitions()
                }
            }),
            "tools/call" => {
                let params = req.get("params").cloned().unwrap_or(json!({}));
                let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let args = params.get("arguments").cloned().unwrap_or(json!({}));
                match dispatch_tool_call(workspace, tool_name, &args) {
                    Ok(output_val) => json!({
                        "jsonrpc": "2.0",
                        "id": id_val,
                        "result": {
                            "content": [{
                                "type": "text",
                                "text": serde_json::to_string_pretty(&output_val)?
                            }],
                            "isError": false
                        }
                    }),
                    Err(err) => json!({
                        "jsonrpc": "2.0",
                        "id": id_val,
                        "result": {
                            "content": [{
                                "type": "text",
                                "text": format!("ERROR: {:#}", err)
                            }],
                            "isError": true
                        }
                    }),
                }
            }
            _ => json!({
                "jsonrpc": "2.0",
                "id": id_val,
                "error": {
                    "code": -32601,
                    "message": format!("Method not found: {}", method)
                }
            }),
        };

        writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
        stdout.flush()?;
    }

    Ok(())
}

fn mcp_tool_definitions() -> Value {
    json!([
        {
            "name": "heav3ns_status",
            "description": "Get the current HEAV3NS censorship pipeline status, including pending assets, verified editions, escalated assets, unscanned inbox files, and next_recommended_action.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        },
        {
            "name": "heav3ns_onboard",
            "description": "Generate a context-aware, role-specific onboarding and handoff packet ('agent', 'developer', 'stakeholder', or 'all') with live workspace commands, invariants, and documentation map.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "role": {
                        "type": "string",
                        "enum": ["agent", "developer", "stakeholder", "all"],
                        "description": "Target persona (default: 'agent')"
                    }
                }
            }
        },
        {
            "name": "heav3ns_explain_asset",
            "description": "Generate a comprehensive diagnostic, cryptographic integrity, and stakeholder/agent handoff explanation card for a single asset.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "asset_id": {
                        "type": "string",
                        "description": "Asset ID, unique prefix, SHA-256 prefix, or filename"
                    }
                },
                "required": ["asset_id"]
            }
        },
        {
            "name": "heav3ns_doctor",
            "description": "Run a comprehensive cryptographic (SHA-256), bit-exact preservation, escalation-isolation, and orphan-hygiene health audit across the workspace.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "fix": {
                        "type": "boolean",
                        "description": "If true, automatically repairs missing directories, cleans orphaned files, and re-syncs reports."
                    }
                }
            }
        },
        {
            "name": "heav3ns_ingest",
            "description": "Scan assets/inbox (or ingest a specific file path), preserve untouched originals with SHA-256 verification, and register assets in the ledger.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Optional specific image path to ingest. If omitted, scans assets/inbox."
                    },
                    "policy": {
                        "type": "string",
                        "description": "Policy profile ID (default: jp_article_175_standard)"
                    }
                }
            }
        },
        {
            "name": "heav3ns_batch_prepare",
            "description": "Ingest all new inbox files and automatically generate coordinate inspection grids + candidate hotspot hints for every pending asset in one call.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "policy": {
                        "type": "string",
                        "description": "Policy profile ID (default: jp_article_175_standard)"
                    }
                }
            }
        },
        {
            "name": "heav3ns_inspect",
            "description": "Generate a coordinate inspection grid overlay (with burned-in Normalized1000 100..900 ruler badges & interior anchors), detect high-frequency candidate hotspots, and optionally generate a 3x magnified zoom-crop grid.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "asset_id": { "type": "string", "description": "Asset ID, unique prefix, or filename to inspect" },
                    "crop": { "type": "string", "description": "Optional zoom-crop spec 'x,y,w,h[:unit]' (e.g. '400,450,200,200:pixels' or '350,450,250,250:normalized1000')" }
                },
                "required": ["asset_id"]
            }
        },
        {
            "name": "heav3ns_execute_censorship",
            "description": "Apply localized Japanese-style censorship treatments (mosaic, gaussian_blur, or solid_bar with #000000, #FFFFFF, or 'auto' luminance-adaptive color) with rect, ellipse, or pill shapes and optional rotation angle_deg, verify bit-exact preservation + 1080p/540p JPEG resilience, and export the publication edition + 4-panel Broadcast QC Slate. Supports incremental region editing via 'append', 'remove_region_ids', and 'dry_run'.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "asset_id": { "type": "string", "description": "Asset ID, unique prefix, or filename" },
                    "append": { "type": "boolean", "description": "If true, merges/upserts regions by ID into existing regions rather than replacing all regions." },
                    "remove_region_ids": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Optional list of existing region IDs to remove before applying updates."
                    },
                    "dry_run": { "type": "boolean", "description": "If true, simulates censorship and runs verification without writing to published/ or mutating the ledger." },
                    "policy": { "type": "string", "description": "Optional policy profile override" },
                    "regions": {
                        "type": "array",
                        "description": "List of localized censorship regions",
                        "items": {
                            "type": "object",
                            "properties": {
                                "id": { "type": "string" },
                                "label": { "type": "string" },
                                "rationale": { "type": "string" },
                                "x": { "type": "number" },
                                "y": { "type": "number" },
                                "width": { "type": "number" },
                                "height": { "type": "number" },
                                "unit": { "type": "string", "enum": ["pixels", "normalized1000", "percent"] },
                                "shape": { "type": "string", "enum": ["rect", "ellipse", "pill"] },
                                "angle_deg": { "type": "number" },
                                "feather_px": { "type": "integer" },
                                "treatment": {
                                    "type": "object",
                                    "description": "Tagged union with 'method': 'mosaic' | 'gaussian_blur' | 'solid_bar'"
                                }
                            },
                            "required": ["id", "label", "rationale", "x", "y", "width", "height", "treatment"]
                        }
                    }
                },
                "required": ["asset_id"]
            }
        },
        {
            "name": "heav3ns_approve_original",
            "description": "Approve an asset that contains artistic nudity or suggestive imagery not requiring censorship, exporting the untouched original to published/ with full audit logging.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "asset_id": { "type": "string" },
                    "classification": { "type": "string", "enum": ["artistic_nudity_pass", "suggestive_pass"] },
                    "rationale": { "type": "string" }
                },
                "required": ["asset_id", "classification", "rationale"]
            }
        },
        {
            "name": "heav3ns_escalate",
            "description": "Escalate an asset for review when artistic preservation or compliance requirements are genuinely ambiguous. Automatically revokes any prior published file.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "asset_id": { "type": "string" },
                    "category": { "type": "string" },
                    "reason": { "type": "string" },
                    "agent_notes": { "type": "string" }
                },
                "required": ["asset_id", "category", "reason"]
            }
        },
        {
            "name": "heav3ns_reset_asset",
            "description": "Reset an asset back to PendingInspection, removing published/preview/escalation artifacts while preserving the untouched original.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "asset_id": { "type": "string" }
                },
                "required": ["asset_id"]
            }
        },
        {
            "name": "heav3ns_policy_guide",
            "description": "Query Japanese obscenity standards (Article 175) and platform censorship policy profiles.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }
    ])
}

fn dispatch_tool_call(workspace: &Workspace, tool_name: &str, args: &Value) -> Result<Value> {
    match tool_name {
        "heav3ns_status" => {
            let status = workspace.status_summary()?;
            Ok(serde_json::to_value(status)?)
        }
        "heav3ns_onboard" => {
            let role = args.get("role").and_then(|s| s.as_str()).unwrap_or("agent");
            let packet = workspace.onboard(role)?;
            Ok(serde_json::to_value(packet)?)
        }
        "heav3ns_explain_asset" => {
            let asset_id = args
                .get("asset_id")
                .and_then(|s| s.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing asset_id"))?;
            let packet = workspace.explain_asset(asset_id)?;
            Ok(serde_json::to_value(packet)?)
        }
        "heav3ns_doctor" => {
            let fix = args.get("fix").and_then(|b| b.as_bool()).unwrap_or(false);
            let report = workspace.run_doctor(fix)?;
            Ok(serde_json::to_value(report)?)
        }
        "heav3ns_ingest" => {
            let policy = args
                .get("policy")
                .and_then(|s| s.as_str())
                .unwrap_or("jp_article_175_standard");
            if let Some(fp) = args.get("file_path").and_then(|s| s.as_str()) {
                let rec = workspace.ingest_file(std::path::Path::new(fp), policy)?;
                Ok(serde_json::to_value(rec)?)
            } else {
                let recs = workspace.ingest_inbox(policy)?;
                Ok(serde_json::to_value(recs)?)
            }
        }
        "heav3ns_batch_prepare" => {
            let policy = args
                .get("policy")
                .and_then(|s| s.as_str())
                .unwrap_or("jp_article_175_standard");
            let payloads = workspace.batch_prepare(policy)?;
            Ok(serde_json::to_value(payloads)?)
        }
        "heav3ns_inspect" => {
            let asset_id = args
                .get("asset_id")
                .and_then(|s| s.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing asset_id"))?;
            let crop = args.get("crop").and_then(|s| s.as_str());
            let payload = workspace.inspect_asset_with_crop(asset_id, crop)?;
            Ok(serde_json::to_value(payload)?)
        }
        "heav3ns_execute_censorship" => {
            let asset_id = args
                .get("asset_id")
                .and_then(|s| s.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing asset_id"))?;
            let regions: Vec<crate::models::CensorshipRegion> =
                if let Some(regions_val) = args.get("regions").cloned() {
                    serde_json::from_value(regions_val)?
                } else {
                    Vec::new()
                };
            let append = args
                .get("append")
                .and_then(|b| b.as_bool())
                .unwrap_or(false);
            let remove_region_ids: Vec<String> = args
                .get("remove_region_ids")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let dry_run = args
                .get("dry_run")
                .and_then(|b| b.as_bool())
                .unwrap_or(false);
            let policy = args
                .get("policy")
                .and_then(|s| s.as_str())
                .map(|s| s.to_string());

            let (rec, ver) = workspace.execute_censorship_with_options(
                asset_id,
                crate::models::CensorshipClassification::ExplicitRequiresIntervention,
                regions,
                crate::workspace::ExecuteOptions {
                    append_regions: append,
                    remove_region_ids,
                    dry_run,
                    policy_override: policy,
                },
            )?;
            Ok(json!({
                "dry_run": dry_run,
                "asset": rec,
                "verification": ver
            }))
        }
        "heav3ns_approve_original" => {
            let asset_id = args
                .get("asset_id")
                .and_then(|s| s.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing asset_id"))?;
            let class_str = args
                .get("classification")
                .and_then(|s| s.as_str())
                .unwrap_or("artistic_nudity_pass");
            let classification = match class_str {
                "suggestive_pass" => crate::models::CensorshipClassification::SuggestivePass,
                _ => crate::models::CensorshipClassification::ArtisticNudityPass,
            };
            let rationale = args
                .get("rationale")
                .and_then(|s| s.as_str())
                .unwrap_or("Artistic nudity / non-explicit composition; approved without modification.");
            let rec = workspace.approve_untouched(asset_id, classification, rationale)?;
            Ok(serde_json::to_value(rec)?)
        }
        "heav3ns_escalate" => {
            let asset_id = args
                .get("asset_id")
                .and_then(|s| s.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing asset_id"))?;
            let category = args
                .get("category")
                .and_then(|s| s.as_str())
                .unwrap_or("artistic_or_compliance_ambiguity");
            let reason = args
                .get("reason")
                .and_then(|s| s.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing reason"))?;
            let notes = args
                .get("agent_notes")
                .and_then(|s| s.as_str())
                .unwrap_or("");
            let rec = workspace.escalate_asset(asset_id, category, reason, notes)?;
            Ok(serde_json::to_value(rec)?)
        }
        "heav3ns_reset_asset" => {
            let asset_id = args
                .get("asset_id")
                .and_then(|s| s.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing asset_id"))?;
            let rec = workspace.reset_asset(asset_id)?;
            Ok(serde_json::to_value(rec)?)
        }
        "heav3ns_policy_guide" => Ok(serde_json::to_value(get_policy_guides())?),
        _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
    }
}
