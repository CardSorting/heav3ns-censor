# PLAYBOOK-01: Autonomous Batch Ingestion & VLM Triage

**Playbook ID:** `PB-01`  
**Owner:** Deputy Gemini (Autonomous Post-Production Department)  
**Trigger:** New artwork files placed in `assets/inbox/` or provided by file path.  

---

## 1. Objective

Ingest all incoming artwork into the cryptographic archive (`assets/originals/`), generate VLM coordinate inspection overlays (`assets/inspect/*_inspect_grid.png`) with automatic high-frequency hotspot hints, and classify each asset into one of the four commission categories.

---

## 2. Step-by-Step Procedure

### Step 1: Run `batch-prepare`
Execute the single-command batch ingestion and inspection packet generator:

```bash
./target/release/heav3ns-censor batch-prepare
```

**What this does automatically:**
1. Scans `assets/inbox/` for `.png`, `.jpg`, `.jpeg`, and `.webp` files.
2. Computes each file's `SHA-256` hash and writes a bit-identical master copy to `assets/originals/<ASSET_ID>_original.<EXT>`.
3. Renders `<ASSET_ID>_inspect_grid.png` with burned-in `Normalized1000` cyan `X100..X900` rulers, yellow `Y100..Y900` rulers, and pink interior `(300,300)` / `(500,500)` / `(700,700)` anchor badges.
4. Runs high-frequency interior gradient analysis to return the top `candidate_hotspots` and `recommended_mosaic_block_px` in JSON.

### Step 2: Multimodal Visual Inspection
For each asset in the `inspection_packets` JSON output:
1. Call `read_file` on `archive_original_path` and `inspect_grid_path`.
2. Evaluate the composition against the Commission Policy Taxonomy:
   - **No explicit genital anatomy or sexual acts?** -> Proceed to **Approve Untouched** (`approve-original`).
   - **Clear explicit anatomy or internal cleft lines?** -> Proceed to **[`PLAYBOOK-02`](PLAYBOOK-02-precision-censorship-and-incremental-region-editing.md)**.
   - **Genuine artistic/biomechanical ambiguity or severe focal-point conflict?** -> Proceed to **[`PLAYBOOK-03`](PLAYBOOK-03-ambiguity-escalation-revocation-and-artist-review.md)**.

### Step 3: Optional 3x Magnified Zoom-Crop (`--crop`)
If an anatomical feature is small (`< 100px`) or angled, refine your coordinates before executing censorship by generating a 3x nearest-neighbor magnified crop with burned-in global coordinates:

```bash
./target/release/heav3ns-censor inspect \
  --asset-id <ASSET_ID_OR_PREFIX> \
  --crop "400,280,200,120:pixels"
```

Read the resulting `zoom_crop_path` image to read pixel-accurate coordinates directly off the magnified ruler badges.

