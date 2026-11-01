# KB-03: Dataset Cleaning, Multimodal AI Safety, & Non-Destructive Artistic Post-Production

**Document ID:** `KB-03`  
**Audience:** AI Safety & Alignment Researchers, Multimodal Dataset Curators, Computer Vision Engineers, and Studio Post-Production Leads (**MAJO** & **EGG**)  
**License:** [MIT](../../LICENSE)  

---

## 1. The Dual Mission: Dataset Sanitization & Studio Post-Production

Modern visual computing faces the exact same fundamental challenge across two seemingly distinct domains:

1. **AI Safety, Security & Multimodal Dataset Curation:** How do we sanitize large-scale vision-language datasets (e.g., removing explicit anatomical regions, sensitive visual tokens, or compliance violations) **without** discarding entire artistic corpora (*sample-dropping bias*) and **without** injecting synthetic diffusion artifacts (*generative inpainting distribution shift*)?
2. **Artistic Rendering & Publication Post-Production:** How do illustrators (**EGG**) and executive studio directors (**MAJO**) produce jurisdiction-compliant publication editions (Japanese Penal Code Article 175 *bokashi* mosaic, *kuro-nuri* / *shiro-nuri* solid graphic bars, feathered Gaussian blur) **without** altering a single pixel of brushwork, color grading, or alpha transparency outside the localized mask—while guaranteeing that the obscured region survives `1080p` social compression and `540p` mobile thumbnail downscaling?

**`heav3ns-censor`** unifies both domains under a single deterministic, cryptographically verified Rust engine.

---

## 2. Eliminating Sample-Dropping Bias & Inpainting Poisoning in ML Datasets

### 2.1 The Sample-Dropping Bias Problem
Conventional dataset cleaning pipelines run a coarse whole-image NSFW classifier (e.g., CLIP-based safety filters) and **delete the entire image** whenever the score exceeds a threshold $\tau$. This creates severe representation collapse:
- Classical figurative art, anatomical studies, life drawing, sculpture, and stylized character illustration are disproportionately purged as false positives.
- Even when an image contains only a tiny localized non-compliant region ($1\text{--}2\%$ of canvas area), discarding the entire sample destroys $98\text{--}99\%$ of valid surrounding visual context (lighting, pose, composition, typography).

### 2.2 The Generative Inpainting Poisoning Problem
Attempting to "scrub" sensitive regions using latent diffusion inpainting introduces three fatal flaws for both safety research and studio art:
1. **VAE Decode Bleed:** Encoding and decoding through a variational autoencoder alters every pixel across the entire image ($\text{RGBA}_{\text{out}}(x,y) \neq \text{RGBA}_{\text{orig}}(x,y)$).
2. **Hallucinated Anatomy & Distribution Shift:** Diffusion inpainters hallucinate synthetic textures and uncanny pseudo-anatomical artifacts that contaminate downstream vision encoders during pre-training or fine-tuning.
3. **Zero Auditability:** Stochastic sampling provides no deterministic mathematical guarantee that high-frequency edge features were actually suppressed across downscaled resolutions.

### 2.3 The `heav3ns-censor` Three-Way Triage Solution
Instead of binary whole-image deletion or stochastic inpainting, `heav3ns-censor` enforces a deterministic three-way triage that exports directly to [`ledger/dataset_manifest.jsonl`](../../ledger/dataset_manifest.jsonl):

| Triage State | Curation Action (`curation_action`) | `is_safe_for_training_release` | Impact on Dataset & Studio Corpus |
| :--- | :--- | :---: | :--- |
| **`Verified`** (`explicit_requires_intervention`) | `"surgical_region_redaction"` | **`true`** | Redacts only the exact analytical SDF mask (`rect`, `ellipse`, `pill` at `@angle_deg`). Preserves $\ge 98\%$ of the image **100% bit-identically** (`exterior_pixel_diff_count == 0`). |
| **`ApprovedOriginal`** (`artistic_nudity_pass` / `suggestive_pass`) | `"preserved_untouched_false_positive"` | **`true`** | Preserves classical silhouettes and non-explicit figurative art **100% untouched** (`SHA-256` identical to master), preventing classifier false-positive bias. |
| **`Escalated`** (`ambiguous_escalate`) | `"quarantined_for_human_review"` | **`false`** | Automatically revokes any `published/` file and isolates ambiguous edge cases into a dedicated red-teaming & human-in-the-loop review split. |

---

## 3. Exporting Structured Datasets (`dataset_manifest.jsonl` & `DATASET_CARD.md`)

Run `export-dataset` via CLI or the `heav3ns_export_dataset` MCP tool:

```bash
# Export the full contrastive safety dataset (Verified + ApprovedOriginal + Escalated)
./target/release/heav3ns-censor export-dataset

# Export strictly the compliance-verified training split (excludes Escalated / Pending)
./target/release/heav3ns-censor export-dataset --safe-only
```

### 3.1 Multi-Format Bounding Box & SDF Schema
Every record in `ledger/dataset_manifest.jsonl` includes pre-computed annotations across all three major computer vision & VLM standards:

- **`bbox_xywh_px`**: `[x, y, width, height]` in absolute image pixels (**COCO Object Detection standard**).
- **`bbox_xyxy_norm1000`**: `[x1, y1, x2, y2]` integer coordinates in `[0, 1000]` (**VLM Spatial Grounding token standard** used by Gemini, Qwen-VL, and LLaVA).
- **`bbox_cxcywh_norm01`**: `[center_x, center_y, width, height]` floating-point coordinates in `[0.0, 1.0]` (**YOLO / Ultralytics standard**).
- **`enclosing_aabb_px`**: `[ax, ay, aw, ah]` rotation-aware enclosing Axis-Aligned Bounding Box in pixels.
- **`shape` & `angle_deg`**: Exact analytical Signed Distance Field geometry (`rect`, `ellipse`, `pill`) and rotation angle in degrees.

### 3.2 Paired Counterfactual Safety & Machine Unlearning Benchmarks
Because `heav3ns-censor` preserves both `original_path` (`assets/originals/`, verified by `sha256_original`) and `sanitized_published_path` (`assets/published/`, verified by `sha256_published`) alongside exact pixel masks and multi-scale suppression telemetry, researchers can immediately construct:
- **Concept Erasure & Machine Unlearning Pairs:** Train or evaluate unlearning algorithms on `(original_path, sanitized_published_path, annotations)` where the exterior background is guaranteed to have `0` confounding pixel differences.
- **VLM Safety Grounding Benchmarks:** Evaluate whether multimodal models can accurately localize compliance-sensitive regions using `vlm_inspect_grid_path` and `bbox_xyxy_norm1000`.
- **Adversarial Downscale & Thumbnail Robustness Benchmarks:** Test whether safety redactions survive `1080p Q85` and `540p Q75` JPEG compression (`multiscale_jpeg_1080p_q85_passed` & `multiscale_jpeg_540p_q75_passed`).
