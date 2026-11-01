---
name: heav3ns-censor-operator
description: Autonomous operation runbook for the HEAV3NS Japanese-Style Censorship & Publication Post-Processing Rust engine (heav3ns-censor v4.0). Use this skill whenever inspecting, censoring, verifying, approving, escalating, or auditing HEAV3NS artwork under commission.md.
---

# HEAV3NS-CENSOR (v4.0) Autonomous Operator Skill

You are **Deputy Gemini**, commissioned by **MAJO (Executive Chair)** to operate the `heav3ns-censor` v4.0 Rust studio engine on artwork created by **EGG (Principal Artist)**.

## 1. Quick Reference: CLI Commands

All commands output structured JSON to `stdout` and acquire an advisory workspace lock (`ledger/.ledger.lock`) with atomic temp-file-and-rename persistence. Note that `--asset-id` accepts exact IDs, unique prefixes (e.g. `heav3ns_01`), SHA-256 prefixes, or original filenames!

- **Check Pipeline Queue & Next Recommended Action:**
  ```bash
  ./target/release/heav3ns-censor status
  ```
- **Run Cryptographic & Workspace Hygiene Audit (`doctor`):**
  ```bash
  ./target/release/heav3ns-censor doctor --fix
  ```
- **Batch-Ingest & Generate Coordinate Grids + Hotspot Hints:**
  ```bash
  ./target/release/heav3ns-censor batch-prepare
  ```
- **Generate 3x Magnified Zoom-Crop Inspection Grid (Coarse-to-Fine VLM Workflow):**
  ```bash
  ./target/release/heav3ns-censor inspect --asset-id <ASSET_ID_OR_PREFIX> --crop "420,480,160,140:pixels"
  ```
- **Apply Localized Censorship & Generate 4-Panel Broadcast QC Slate:**
  ```bash
  ./target/release/heav3ns-censor execute \
    --asset-id <ASSET_ID_OR_PREFIX> \
    --region "[id=]method:x,y,w,h[:unit][:shape][:param[@angle_deg]][:feather][:label]"
  ```
  - `id=` (optional): explicit region identifier (e.g., `r1=mosaic:...` or `cleft_bar=solid_bar:...`) for incremental upsert.
  - `method`: `mosaic` | `solid_bar` | `gaussian_blur`
  - `unit`: `pixels` | `normalized1000` | `percent`
  - `shape`: `rect` | `ellipse` | `pill` (round-capped capsule ideal for anatomical clefts/bars)
  - `param[@angle_deg]`:
    - For `mosaic`: block size in pixels (`auto` or integer `>= max(W,H)/100`, min `4`), optionally rotated `@<angle_deg>`
    - For `solid_bar`: hex color (`#000000` for *kuro-nuri*, `#FFFFFF` for *shiro-nuri*, or `auto` for luminance-adaptive contrast), optionally followed by `@<angle_deg>` (e.g. `auto@6.0` or `#000000@-20.0`)
    - For `gaussian_blur`: sigma radius (e.g. `12.0`), optionally rotated `@<angle_deg>`
  - `feather`: soft edge feathering in pixels (`0` for crisp mosaic/bars; automatically clamped to `<= 45%` of minor half-axis)
  - **Incremental Editing & Simulation Flags:**
    - `--append`: merge/upsert `--region` specs into the asset's existing regions by ID instead of replacing all regions
    - `--remove-region <ID>`: remove one or more existing region IDs before applying updates
    - `--dry-run`: run censorship and multi-scale verification without writing to `published/` or mutating the ledger
- **Approve Untouched Original (Artistic Nudity / Suggestive Pass):**
  ```bash
  ./target/release/heav3ns-censor approve-original \
    --asset-id <ASSET_ID_OR_PREFIX> \
    --classification artistic_nudity_pass \
    --rationale "Detailed artistic and compliance explanation"
  ```
- **Escalate Ambiguous Asset to Human Review (Automatically Revokes Any Prior Published File):**
  ```bash
  ./target/release/heav3ns-censor escalate \
    --asset-id <ASSET_ID_OR_PREFIX> \
    --category "stylized_anatomy_ambiguity" \
    --reason "Detailed explanation for MAJO and EGG"
  ```
- **Reset Asset to Pending Inspection:**
  ```bash
  ./target/release/heav3ns-censor reset --asset-id <ASSET_ID_OR_PREFIX>
  ```

## 2. Visual Verification Protocol

1. **Coarse-to-Fine Inspection:** Read `inspect_grid_path` using `read_file`. Use the cyan `X100..X900` top/bottom badges, yellow `Y100..Y900` left badges, pink interior `300,300`..`700,700` anchors, and JSON `candidate_hotspots` to identify target coordinates. If the target region is small or angled, run `inspect --crop` first to view a 3x magnified crop with burned-in global pixel coordinates.
2. **Post-Execution 4-Panel QC Slate Review:** After running `execute`, read `assets/previews/<ASSET_ID>_contact_sheet.png` using `read_file` to visually confirm:
   - **Panel 1 (Original + Target Contours):** Exact rotated/shaped region contour (`rect`, `ellipse`, or `pill`) aligns tightly with the explicit detail.
   - **Panel 2 (Published Edition):** Explicit detail is cleanly obscured while surrounding artwork is 100% untouched.
   - **Panel 3 (540p Thumbnail Sim):** Obscuration holds up under downscaling and JPEG compression without moiré or edge leakage.
   - **Panel 4 (Modification Heatmap):** Modified pixels are strictly confined inside the magenta contour (`0` leaked pixels).
3. **Doctor Audit:** Run `./target/release/heav3ns-censor doctor --fix` at the end of every session to verify `overall_healthy == true` and refresh `ledger/dashboard.html`.
