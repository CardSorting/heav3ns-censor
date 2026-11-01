---
license: mit
task_categories:
  - image-to-image
  - object-detection
  - image-classification
tags:
  - ai-safety
  - dataset-cleaning
  - localized-redaction
  - vlm-grounding
  - artistic-preservation
---

# HEAV3NS Sanitized Vision & Safety Benchmark Dataset Card

Generated deterministically by **`heav3ns-censor v4.0.0`**.

## Dataset Summary & Safety Provenance

- **Total Exported Records:** `3` (`safe_only = false`)
- **Safe Training Split (`is_safe_for_training_release == true`):** `2`
  - **Surgical Localized Redactions (`Verified`):** `1` (`0` exterior pixels altered; verified across Native, `1080p Q85`, and `540p Q75` JPEG)
  - **Preserved False-Positive Artistic Corpora (`ApprovedOriginal`):** `1` (`100%` bit-identical to original)
- **Quarantined Ambiguity Benchmark (`Escalated`):** `1` (isolated from `assets/published/`)
- **Manifest File:** `ledger/dataset_manifest.jsonl`

## Loading in Python (`datasets` / `json`)

```python
import json
from pathlib import Path

records = [
    json.loads(line)
    for line in Path("ledger/dataset_manifest.jsonl").read_text().splitlines()
    if line.strip()
]
safe_split = [r for r in records if r["is_safe_for_training_release"]]
print(f"Loaded {len(records)} total records ({len(safe_split)} compliance-verified safe records).")
```
