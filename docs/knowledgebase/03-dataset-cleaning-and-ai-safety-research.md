# KB-03: Data Annotation, False-Positive Suppression, Dataset Cleaning & Artistic Post-Production

**Document ID:** `KB-03`  
**Audience:** AI Safety & Alignment Researchers, Data Annotation & Curation Engineers, Computer Vision Researchers, and Studio Post-Production Leads (**MAJO** & **EGG**)  
**License:** [MIT](../../LICENSE)  

---

## 1. Why the Industry is Plagued with False Positives (And How We Fix It)

Modern vision-language models (VLMs), safety classifiers, and content moderation detectors suffer from an endemic **False-Positive Crisis**:
- **Root Cause #1 — Whole-Image Binary Labels Without Hard-Negative Boxes:** Traditional safety datasets only label whole images (`"unsafe"` vs. `"safe"`) or only draw bounding boxes around *positive* explicit violations. They never annotate **Verified Benign Hard-Negative Regions** (e.g., classical dorsal silhouettes, skin-tone lighting gradients, collarbone/shoulder contours, anime/manga muscle linework, or clothing seams). Consequently, detectors overfit to superficial skin-tone or high-contrast contour cues and flag benign artistic work as false positives.
- **Root Cause #2 — Sample-Dropping Bias:** When a classifier flags an image, conventional pipelines delete the entire sample, erasing figurative art and anatomical corpora from pre-training splits.
- **Root Cause #3 — Generative Inpainting Poisoning:** Scrubbing regions with diffusion inpainting alters every pixel via VAE decoding and injects hallucinated latent artifacts.

### Mirroring Industry-Standard Data Curation Platforms (`CVAT`, `Label Studio`, `Roboflow`, `Voxel51 FiftyOne`, `Scale Nucleus`)
`heav3ns-censor` implements the proven **Contrastive Positive + Hard-Negative Region Annotation** architecture used by world-class computer vision curation platforms:

| Annotation Polarity | Class ID | CLI / MCP Command | Pixel Modification | Purpose in Model Training & Safety Evaluation |
| :--- | :---: | :--- | :---: | :--- |
| **`positive_intervention`** (`explicit_intervention_required`) | `1` | `execute` / `heav3ns_execute_censorship` | Applies localized SDF redaction (`0` exterior diff px) | Teaches detectors & VLMs the exact spatial boundaries of explicit anatomy requiring intervention. |
| **`verified_benign_hard_negative`** (`false_positive_suppression`) | `0` | `annotate-negative` / `heav3ns_annotate_negative` *(+ auto-mined hotspots)* | **`0` pixels altered** (100% untouched) | Explicitly teaches detectors & safety filters what **not** to flag—eliminating false positives on classical silhouettes, skin highlights, and stylized linework. |

---

## 2. Hard-Negative Annotation & Automatic Confuser Mining (`annotate-negative`)

### 2.1 Automatic Hard-Negative Hotspot Mining
Whenever `heav3ns-censor` processes an asset via `execute` or `approve-original`:
1. The engine computes the top high-frequency gradient/contrast regions (`detect_candidate_hotspots`).
2. Every hotspot whose Intersection-over-Union ($\text{IoU}$) with all positive intervention boxes is $< 0.10$ is a region that looked like a high-contrast candidate hotspot to a spatial detector, but was verified by the VLM/artist to be **benign artistic content**.
3. These non-overlapping hotspots are automatically recorded as **`verified_benign_hard_negative` (`class_id = 0`)** bounding boxes!

### 2.2 Explicit Hard-Negative Curation via CLI, MCP, or Interactive `dashboard.html`
Researchers and artists can also explicitly annotate verified benign confuser regions without altering a single pixel of the artwork:

```bash
# Explicitly tag a benign confuser region + auto-mine non-overlapping hotspots
./target/release/heav3ns-censor annotate-negative \
  --asset-id heav3ns_02 \
  --append \
  --auto-mine-hotspots \
  --region "neg_silhouette=280,140,440,720:pixels:rect:classical_dorsal_silhouette:Stylized dorsal anatomy with zero explicit detail"
```

In **[`ledger/dashboard.html`](../../ledger/dashboard.html)**, click **`+ Hard-Negative (Benign)`** in the coordinate toolbar and drag an **Emerald Green** bounding box over any false-positive confuser region to generate the exact CLI command in 1 click.

---

## 3. Dual Industry-Standard Exports (`dataset_manifest.jsonl` & `annotations_coco.json`)

Running `./target/release/heav3ns-censor export-dataset [--safe-only]` (or `sync-reports`) generates three synchronized artifacts in `ledger/`:

1. **[`ledger/dataset_manifest.jsonl`](../../ledger/dataset_manifest.jsonl) (HuggingFace `datasets` / VLM Grounding JSONL):**
   - Each line includes `positive_annotations_count`, `hard_negative_annotations_count`, `exterior_pixel_diff_count` (`0`), and an `annotations` array where every box carries:
     - `polarity` (`"positive_intervention"` vs. `"verified_benign_hard_negative"`) & `class_id` (`1` vs. `0`)
     - `confuser_category` (`"composite_skin_and_crease_confuser"`, `"skin_tone_anatomical_silhouette"`, `"high_frequency_linework_or_seam"`, `"stylized_contour_confuser"`)
     - `fp_hardness_score` (`5.0` to `99.5` quantitative false-positive confuser hardness score for active learning, inspired by **Voxel51 FiftyOne**)
     - `bbox_xywh_px` (**COCO** `[x, y, w, h]`)
     - `bbox_xyxy_norm1000` (**VLM Grounding Tokens** `[x1, y1, x2, y2]` in `[0..1000]`)
     - `bbox_cxcywh_norm01` (**Ultralytics YOLO** `[cx, cy, w, h]` in `[0.0..1.0]`)
2. **[`ledger/annotations_coco.json`](../../ledger/annotations_coco.json) (MS-COCO Object Detection & Hard-Negative Standard):**
   - Ready for direct import into **CVAT**, **Label Studio**, **Roboflow**, **Voxel51 FiftyOne**, **MMDetection**, and **Detectron2** with categories `1: explicit_intervention_required` and `2: verified_benign_hard_negative` (automatically de-conflicted with **Cleanlab-style** `IoU < 0.20` polarity collision prevention).
3. **[`ledger/vlm_safety_sft.jsonl`](../../ledger/vlm_safety_sft.jsonl) (ShareGPT / VLM Grounded Safety Instruction-Tuning Pairs):**
   - Multi-turn `<image>` grounded conversations teaching VLMs both positive intervention boxes (`class_id = 1`) and why verified benign hard-negative boxes (`class_id = 0`) must **not** be flagged as false positives.
4. **[`ledger/DATASET_CARD.md`](../../ledger/DATASET_CARD.md) (HuggingFace Hub Dataset Card):**
   - Summarizes split counts, positive vs. hard-negative box counts, and cryptographic provenance.
   - See **[`PLAYBOOK-05`](../playbooks/PLAYBOOK-05-data-annotation-and-false-positive-suppression.md)** for the complete operator runbook.
