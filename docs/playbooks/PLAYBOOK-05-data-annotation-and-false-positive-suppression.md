# PLAYBOOK-05: Contrastive Data Annotation & False-Positive Suppression

**Owner:** Deputy Gemini & AI Safety / Dataset Curation Operators  
**License:** [MIT](../../LICENSE)  
**Mirrored Industry Standards:** Voxel51 FiftyOne (Hardness & Mistakenness), Cleanlab (Label Conflict Detection), Scale AI Nucleus (Autotag Hard-Negative Mining), Roboflow, CVAT, & Label Studio  

---

## 1. Objective: Eliminating Classifier False Positives at the Annotation Layer

Modern vision safety classifiers, VLM guardrails, and object detectors suffer from an epidemic of **false positives** (`15%–40%` false-positive rates on figurative art, classical sculpture, character design, warm skin-tone shading, and overlapping clothing folds). This happens because standard annotation pipelines only record **Positive Intervention Boxes (`class_id = 1`)**, leaving all unredacted high-frequency anatomy and linework implicit.

`heav3ns-censor v4.0` solves this via **Contrastive Dual-Polarity Grounding**:
1. **Positive Intervention Boxes (`class_id = 1`, `positive_intervention`):** Explicit anatomy requiring surgical SDF redaction (`execute`).
2. **Verified Benign Hard-Negative Boxes (`class_id = 0`, `verified_benign_hard_negative`):** Regions with high edge energy or warm skin-tone chrominance (`fp_hardness_score` $\in [5.0, 99.5]$) that have been verified safe and **must remain `100%` untouched (`0` pixel modifications)** (`annotate-negative` / `approve-original`).
3. **Automatic Polarity De-Confliction (`IoU < 0.20`):** Inspired by **Cleanlab** and **Voxel51 FiftyOne**, the engine automatically prevents or prunes any hard-negative box that overlaps an active positive redaction region (`IoU >= 0.20`), ensuring zero contradictory labels in exported training splits.

---

## 2. Step-by-Step Curation & Annotation Workflow

### Step 1: Ingest & Rank Confuser Hotspots (`batch-prepare`)
```bash
./target/release/heav3ns-censor batch-prepare
```
Each asset's `InspectPayload` returns up to 4 `candidate_hotspots` ranked by high-frequency gradient energy (`edge_energy`) and scored with:
- `fp_hardness_score` (`5.0` to `99.5`): Combines spatial edge density and warm skin-tone chrominance ratio to quantify how likely a naive classifier is to false-positive on the region.
- `confuser_category`: Automatic taxonomy classification:
  - `"composite_skin_and_crease_confuser"` (warm skin-tone gradient + sharp linework)
  - `"skin_tone_anatomical_silhouette"` (warm flesh-tone shading without explicit detail)
  - `"high_frequency_linework_or_seam"` (clothing seams, cross-hatching, specular highlights)
  - `"stylized_contour_confuser"` (general artistic contour)

### Step 2A: Annotate Explicit Assets with Positive + Hard-Negative Regions
When an image contains explicit detail in one localized region (`r1`) alongside benign artistic features elsewhere on the canvas:
```bash
# 1. Apply surgical positive redaction (automatically mines non-overlapping hotspots as hard negatives)
./target/release/heav3ns-censor execute \
  --asset-id heav3ns_01 \
  --region "r1=mosaic:440,500,120,100:pixels:rect:auto:0:explicit_anatomy"

# 2. (Optional) Explicitly add human/agent-verified benign confuser boxes (`class_id = 0`)
./target/release/heav3ns-censor annotate-negative \
  --asset-id heav3ns_01 \
  --append \
  --auto-mine-hotspots \
  --region "neg_crease=360,260,280,140:pixels:rect:clothing_fold_confuser:High-contrast garment seam verified non-explicit"
```
> **Automatic Polarity Guard:** If any `--region` passed to `annotate-negative` overlaps an active positive region (`IoU >= 0.20`), it is automatically skipped and logged in `audit_trail` (`skipped_polarity_conflicts`).

### Step 2B: Approve False-Positive Artistic Nudity Untouched (`approve-original`)
When an entire illustration is a classical silhouette or non-explicit figurative study:
```bash
./target/release/heav3ns-censor approve-original \
  --asset-id heav3ns_02 \
  --classification artistic_nudity_pass \
  --rationale "Classical dorsal silhouette with zero explicit anatomical detail."
```
- Copies the untouched original bit-identically (`SHA-256` match) to `assets/published/`.
- Automatically mines and scores the highest-energy contour/silhouette boxes as `verified_benign_hard_negative` (`class_id = 0`) with `fp_hardness_score`.

### Step 3: Interactive Visual Annotation in `ledger/dashboard.html`
1. Open `ledger/dashboard.html` in any browser.
2. Select **`+ Hard-Negative (Benign)`** in the Coordinate HUD.
3. Drag a green selection box over any false-positive confuser region on the canvas to generate a ready-to-run `annotate-negative` CLI command.
4. Review the **Hard-Negative Confuser Table** on each asset card to inspect `Confuser Taxonomy` and `Confuser Hardness (XX.X / 100)`.

### Step 4: Export Multi-Format Training & Active-Learning Manifests (`export-dataset`)
```bash
# Export verified safe training split (excludes Escalated assets)
./target/release/heav3ns-censor export-dataset --safe-only

# Or export full contrastive benchmark (includes Escalated assets)
./target/release/heav3ns-censor export-dataset
```
This deterministically writes four synchronized dataset artifacts in `ledger/`:
1. **`ledger/dataset_manifest.jsonl`**: HuggingFace `datasets` JSON Lines with `bbox_xywh_px` (COCO), `bbox_xyxy_norm1000` (VLM), `bbox_cxcywh_norm01` (YOLO), `confuser_category`, and `fp_hardness_score`.
2. **`ledger/annotations_coco.json`**: Standard MS-COCO Object Detection JSON (`category_id = 1` for `explicit_intervention_required`, `category_id = 2` for `verified_benign_hard_negative`) ready for **CVAT**, **Label Studio**, **Roboflow**, and **Voxel51 FiftyOne**.
3. **`ledger/vlm_safety_sft.jsonl`**: ShareGPT / VLM Grounded Safety Instruction-Tuning pairs teaching multimodal models both where to redact (`positive_interventions`) and which high-hardness confusers to leave untouched (`verified_benign_hard_negatives`).
4. **`ledger/DATASET_CARD.md`**: Auto-generated HuggingFace Dataset Card.

### Step 5: Audit Annotation Hygiene (`doctor --fix`)
```bash
./target/release/heav3ns-censor doctor --fix
```
Verifies `annotation_polarity_conflict:<asset_id>` (`passed == true`), confirming zero positive/hard-negative IoU collisions across the entire workspace.

