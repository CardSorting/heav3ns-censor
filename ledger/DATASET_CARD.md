---
license: mit
task_categories:
  - object-detection
  - image-to-image
  - image-classification
tags:
  - ai-safety
  - dataset-curation
  - false-positive-suppression
  - hard-negative-mining
  - dpo
  - vlm-grounding
  - artistic-preservation
---

# HEAV3NS Preservation-Aware Contrastive Safety, Hard-Negative & Sanitized Vision Dataset Card

Generated deterministically by **`heav3ns-censor v4.0.0`**.

## 1. Preservation-Aware Curation & False-Positive Suppression Provenance

Eliminates the two primary failure modes of conventional generative AI dataset cleaning (**Sample-Dropping Bias** per `arXiv:2409.17156` and **Generative Inpainting Distortion**):

- **Total Exported Images:** `3` (`safe_only = false`)
- **Safe Training Split (`is_safe_for_training_release == true`):** `2`
  - **Surgical Localized Redactions (`Verified`):** `1` (`0` exterior pixels altered; verified across Native, `1080p Q85`, and `540p Q75` JPEG)
  - **Preserved False-Positive Artistic Corpora (`ApprovedOriginal`):** `1` (`100%` bit-identical to original)
- **Quarantined Ambiguity Benchmark (`Escalated`):** `1` (isolated from `assets/published/`)
- **Contrastive Dual-Polarity Grounding (`COCO` + `Normalized1000` + `YOLO`) & Confuser Hardness:**
  - **Positive Intervention Boxes (`class_id = 1`, `positive_intervention`):** `2`
  - **Verified Benign Hard-Negative Boxes (`class_id = 0`, `verified_benign_hard_negative`):** `3` *(eliminates classifier false positives on classical silhouettes, skin-tone gradients, and artistic linework; scored with quantitative `fp_hardness_score` in `[5.0..99.5]`, deduplicated via Hardness-Weighted NMS `IoU < 0.45`, and automatically de-conflicted via `IoU < 0.20`)*

## 2. Synchronized Multi-Format Exports (`ledger/`)

1. **`ledger/dataset_manifest.jsonl`**: HuggingFace `datasets` / JSON Lines with `bbox_xywh_px` (COCO), `bbox_xyxy_norm1000` (VLM), `bbox_cxcywh_norm01` (YOLO), `fp_hardness_score`, and `confuser_category`.
2. **`ledger/annotations_coco.json`**: Standard MS-COCO Object Detection format for **CVAT**, **Label Studio**, **Roboflow**, and **Voxel51 FiftyOne**.
3. **`ledger/vlm_safety_sft.jsonl`**: ShareGPT / VLM Grounded Safety & False-Positive Suppression Instruction-Tuning Pairs.
4. **`ledger/vlm_safety_dpo.jsonl`**: Direct Preference Optimization (`DPO`) Contrastive `(prompt, chosen, rejected)` Preference Triples penalizing false-positive over-censorship and unsafe premature approvals.

## 3. Loading in Python (`datasets` / `json` / `pycocotools`)

```python
import json
from pathlib import Path

records = [
    json.loads(line)
    for line in Path("ledger/dataset_manifest.jsonl").read_text().splitlines()
    if line.strip()
]
pos_boxes = [a for r in records for a in r["annotations"] if a["class_id"] == 1]
hard_neg_boxes = [a for r in records for a in r["annotations"] if a["class_id"] == 0]
print(f"Images: {len(records)} | Positive Redaction Boxes: {len(pos_boxes)} | Hard-Negative Benign Boxes: {len(hard_neg_boxes)}")
```
