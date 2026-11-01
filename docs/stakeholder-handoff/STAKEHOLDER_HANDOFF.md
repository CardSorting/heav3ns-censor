# HEAV3NS-CENSOR: Executive & Studio Stakeholder Handoff Guide

**Document ID:** `DOC-HANDOFF-001`  
**Engine Version:** `heav3ns-censor v4.0.0`  
**Prepared For:** MAJO (Executive Chair), EGG (Principal Artist), & Studio Operations  
**Prepared By:** Deputy Gemini (Autonomous Post-Production Department)  

---

## 1. Executive Summary

The `heav3ns-censor` (`v4.0.0`) Rust studio engine and autonomous workflow infrastructure are complete, verified, and deployed in this repository. The system fulfills every mandate of the [HEAV3NS Executive Commission](../../commission.md):

- **100% Original Master Preservation:** Every incoming asset in `assets/inbox/` is immediately archived untouched in `assets/originals/` and verified by `SHA-256` checksum on every subsequent operation.
- **Bit-Exact Non-Censored Fidelity:** Across all censored editions, `non_censored_pixel_diff_count == 0` is mathematically enforced and verified pixel-by-pixel.
- **Multi-Scale & Compression Resilience:** Every censorship region is verified not only at native resolution (mean + 95th-percentile peak gradient suppression), but also after `1080p` (`Q85` JPEG) and `540p` (`Q75` Thumbnail JPEG) downscaling.
- **Escalation Safety Isolation:** Escalating an asset to `ledger/escalations_review.md` automatically revokes and deletes any previously published file in `assets/published/`, preventing accidental distribution of unreviewed work.

---

## 2. Stakeholder Roles & Handoff Workflows

### For EGG (Principal Artist)
1. **Drop Artwork:** Place finished master illustrations (`.png`, `.jpg`, `.jpeg`, `.webp`) into `assets/inbox/`.
2. **Zero Technical Overhead:** You do not need to create censorship layers, calculate Article 175 mosaic block sizes, or export social media thumbnails.
3. **Optional Visual Review:** Open `ledger/dashboard.html` in any browser to inspect the **4-Panel Broadcast QC Slate**, toggle between your untouched original and the published edition, or drag a selection box on any image to suggest exact coordinates if you want a custom adjustment.

### For MAJO (Executive Chair)
1. **Review Publication Manifest:** Open [`ledger/publication_report.md`](../../ledger/publication_report.md) or [`ledger/dashboard.html`](../../ledger/dashboard.html) to review all `Verified` and `ApprovedOriginal` assets ready for release.
2. **Review Escalations Queue:** Check [`ledger/escalations_review.md`](../../ledger/escalations_review.md) (or filter by **Escalated** in `ledger/dashboard.html`). Only genuinely ambiguous edge cases appear here, complete with VLM coordinate grids and agent notes.
3. **Run Health Audit Anytime:** Execute `./target/release/heav3ns-censor doctor` to run a full cryptographic (`SHA-256`), bit-exact preservation, and workspace hygiene check in under `50ms`.

### For Autonomous Agents (Deputy Gemini / Peer Agents)
- **Rules & Directives:** Loaded automatically from [`AGENTS.md`](../../AGENTS.md).
- **Operational Runbook Skill:** Located at [`.agents/skills/heav3ns-censor-operator/SKILL.md`](../../.agents/skills/heav3ns-censor-operator/SKILL.md).
- **MCP Server Integration:** Configured in [`.agents/mcp_config.json`](../../.agents/mcp_config.json) (`./target/release/heav3ns-censor mcp`), exposing 10 structured tools (`heav3ns_status`, `heav3ns_doctor`, `heav3ns_ingest`, `heav3ns_batch_prepare`, `heav3ns_inspect`, `heav3ns_execute_censorship`, `heav3ns_approve_original`, `heav3ns_escalate`, `heav3ns_reset_asset`, `heav3ns_policy_guide`).

---

## 3. Workspace Directory Map & Artifact Contract

```
HEAV3NS-CENSOR/
├── commission.md                  # Executive Commission Charter (MAJO)
├── AGENTS.md                      # Always-on Autonomous Directives (Deputy Gemini)
├── Cargo.toml                     # Rust v4.0.0 Studio Engine Manifest
├── src/                           # Rust Source Code (engine, workspace, models, policy, mcp, demo)
├── assets/
│   ├── inbox/                     # [INPUT] Drop incoming artwork here
│   ├── originals/                 # [IMMUTABLE ARCHIVE] SHA-256 verified untouched masters
│   ├── inspect/                   # [VLM GROUNDING] Coordinate grids & 3x magnified zoom-crops
│   ├── previews/                  # [QC TELEMETRY] 4-Panel QC Slates + 1080p/540p JPEG simulations
│   ├── published/                 # [RELEASE OUTPUT] Verified censored & approved untouched editions
│   └── review_escalated/          # [ISOLATED QUEUE] Escalated assets awaiting human decision
└── ledger/
    ├── treatments_ledger.json     # Deterministic BTreeMap JSON source-of-truth ledger
    ├── publication_report.md      # Human-readable Markdown publication manifest
    ├── escalations_review.md      # Executive & Artist escalation review queue
    └── dashboard.html             # Interactive Studio QC Web Application (v4.0)
```

---

## 4. Using the Interactive Studio QC Dashboard (`ledger/dashboard.html`)

Open [`ledger/dashboard.html`](../../ledger/dashboard.html) in any modern browser (no local server required; uses relative paths):

1. **6-Tab Multi-Resolution Switcher (Keys `1`–`6`):**
   - `[1] 4-Panel QC Slate`: Side-by-side view of Original + Target Contours, Published Edition, 540p Thumbnail Sim, and Bit-Exact Modification Heatmap.
   - `[2] Published Edition`: Full-resolution PNG output.
   - `[3] 1080p JPEG Sim`: Simulated social media distribution (`Q85` JPEG).
   - `[4] 540p Thumb Sim`: Simulated mobile/feed thumbnail (`Q75` JPEG).
   - `[5] VLM Coordinate Grid`: Burned-in `Normalized1000` cyan/yellow rulers and pink interior anchors.
   - `[6] Untouched Original`: Archived master file from `assets/originals/`.
2. **Interactive Coordinate & Bounding-Box Inspector:**
   - **Hover** over any image to read live `Cursor Pixels: (X, Y)` and `Norm1000: (X, Y)`.
   - **Click and drag** across any region (even directly on a quadrant of the 4-Panel QC Slate) to draw a cyan rubber-band box and automatically generate a ready-to-run `--region` CLI command with one-click **Copy CLI**.

---

## 5. Verification & Acceptance Checklist

| Commission Requirement | Status | Verification Mechanism |
| :--- | :--- | :--- |
| Preserve untouched originals independently | **VERIFIED** | `assets/originals/` + `original_sha256` check in `doctor` |
| Zero modification outside localized censorship regions | **VERIFIED** | `non_censored_pixel_diff_count == 0` in `VerificationReport` |
| Support Japanese Article 175 mosaic, solid bars, & Gaussian blur | **VERIFIED** | `src/engine.rs` (`Mosaic`, `SolidBar`, `GaussianBlur` + `Rect`/`Ellipse`/`Pill` + `@angle_deg`) |
| Distinguish artistic/suggestive nudity from explicit anatomy | **VERIFIED** | `approve-original` vs `execute` vs `escalate` state machine |
| Survive publication downscaling (`1080p` & `540p` JPEG) | **VERIFIED** | `pub_1080p_suppression_ratio` & `pub_540p_suppression_ratio` |
| Maintain auditable correspondence ledger & escalation queue | **VERIFIED** | `ledger/treatments_ledger.json`, `publication_report.md`, `escalations_review.md` |
