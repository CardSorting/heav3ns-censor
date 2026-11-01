---
license: mit
task_categories:
  - object-detection
  - image-to-image
  - image-classification
tags:
  - ai-safety
  - false-positive-suppression
  - hard-negative-mining
  - dataset-cleaning
  - vlm-grounding
  - artistic-preservation
---

# HEAV3NS Contrastive Safety, Hard-Negative & Sanitized Vision Dataset Card

Generated deterministically by **`heav3ns-censor v4.0.0`**.

## Dataset Summary & False-Positive Suppression Provenance

- **Total Exported Images:** `3` (`safe_only = false`)
- **Safe Training Split (`is_safe_for_training_release == true`):** `2`
  - **Surgical Localized Redactions (`Verified`):** `1` (`0` exterior pixels altered; verified across Native, `1080p Q85`, and `540p Q75` JPEG)
  - **Preserved False-Positive Artistic Corpora (`ApprovedOriginal`):** `1` (`100%` bit-identical to original)
- **Quarantined Ambiguity Benchmark (`Escalated`):** `1` (isolated from `assets/published/`)
- **Region-Level Bounding Box Annotations (`COCO` + `Normalized1000` + `YOLO`):**
  - **Positive Intervention Boxes (`class_id = 1`, `positive_intervention`):** `2`
  - **Verified Benign Hard-Negative Boxes (`class_id = 0`, `verified_benign_hard_negative`):** `2` *(eliminates classifier false positives on classical silhouettes, skin-tone gradients, and artistic linework)*
- **Exported Manifests:**
  - `ledger/dataset_manifest.jsonl` (HuggingFace `datasets` / JSON Lines)
  - `ledger/annotations_coco.json` (MS-COCO Object Detection format for CVAT, Label Studio, Roboflow, & Voxel51 FiftyOne)

## Loading in Python (`datasets` / `json` / `pycocotools`)

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
