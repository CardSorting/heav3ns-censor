[🏠 Home (`README.md`)](../../README.md) &rsaquo; [📚 Docs (`docs/`)](../README.md) &rsaquo; [🚀 YC Package (`docs/yc/`)](README.md) &rsaquo; **03: `ycombinator.com/apply` Application Blueprint**

---

# Official Y Combinator Application Blueprint (`https://www.ycombinator.com/apply`)

> **How to Use This Blueprint:** Every answer below is written in the **exact concise, plain-spoken, metric-dense style YC Group Partners prefer**—free of marketing fluff, anchored in the upstream **Preservation-Aware Dataset Curation & Contrastive Safety Alignment** thesis from [`yc.md`](../../yc.md), and backed directly by working code in this repository.

---

## Section 1: Company Overview

### 1. Describe what your company does in 50 characters or less.
*(Strict limit: 50 characters including spaces. Choose one of the three verified options below:)*

- **Option A (Recommended — 49 characters):**
  ```text
  Preservation-aware visual dataset curation for AI
  ```
- **Option B (Problem-First — 50 characters):**
  ```text
  Fix false positives in AI vision training datasets
  ```
- **Option C (Infrastructure-First — 48 characters):**
  ```text
  Surgical dataset curation & safety engine for AI
  ```

---

### 2. What is your company going to make? Please describe your product and what it does or will do.

We build **HEAV3NS (`heav3ns-censor`)**, an open-core Rust engine and agentic MCP platform for **preservation-aware visual dataset curation and false-positive suppression** in generative AI pipelines.

Today, multimodal AI teams cleaning visual training datasets face two broken choices when a safety classifier triggers on an image:
1. **Sample-Dropping Bias:** Discard the entire image. Published research (`arXiv:2409.17156`) shows standard NSFW classifiers (`NudeNet`, `OpenNSFW`) have a **15%–40% false-positive rate** on classical art, sculpture, anatomical studies, and stylized illustration—erasing millions of high-value training samples.
2. **Reconstruction Poisoning:** Use diffusion inpainting to modify the flagged region, which alters surrounding pixels through VAE re-encoding and injects synthetic artifacts into training corpora.

Our Rust engine (`heav3ns-censor v4.0`, MIT License) solves both problems deterministically:
- **Zero-Collateral Surgical Redaction:** When an image contains a localized sensitive region (e.g., 1.5% of the canvas), our Signed Distance Field (SDF) pixel engine masks only that exact rotated contour (`mosaic`, `solid_bar`, or `gaussian_blur`) while guaranteeing **100% bit-identical preservation (`0` pixel diffs, SHA-256 verified)** across the remaining 98.5% of the artwork, verified against `1080p` and `540p` JPEG recompression.
- **Contrastive Hard-Negative Mining (`fp_hardness_score`):** When an image or region is a benign confuser (warm skin-tone shading, garment creases, classical silhouettes), our engine scores its false-positive hardness (`0–100`), deduplicates boxes via Hardness-Weighted NMS, and exports synchronized **MS-COCO (`annotations_coco.json`)**, **HuggingFace (`dataset_manifest.jsonl`)**, **VLM SFT (`vlm_safety_sft.jsonl`)**, and **Direct Preference Optimization (`vlm_safety_dpo.jsonl`)** datasets so labs can train safety classifiers and VLMs that stop hallucinating false positives.

---

### 3. Where do you live now, and where would the company be based after YC?
San Francisco Bay Area, CA / Silicon Valley (or your current location -> San Francisco Bay Area after YC).

---

## Section 2: Progress & Traction

### 4. How far along are you?
We have built and shipped **`heav3ns-censor v4.0.0`** (MIT-licensed open-source Rust 2021 codebase, `3,600+` lines of zero-unsafe Rust, `12/12` automated mathematical invariant tests passing).

What is live and working today in our repository (`https://github.com/CardSorting/heav3ns-censor`):
1. **Deterministic Rust Pixel & SDF Geometry Engine (`src/engine.rs`):** Supports axis-aligned and arbitrary-angle (`@<angle_deg>`) `Rect`, `Ellipse`, and round-capped `Pill` treatments with immutable-source pooling and automatic luminance contrast adaptation.
2. **Multi-Scale Frequency & JPEG-Quantization Verifier:** Automatically verifies that redactions survive `1080p` (`Q85`) and `540p` (`Q80`) `4:2:0` chroma-subsampled JPEG compression (checking both mean and 95th-percentile peak gradient suppression) while verifying `non_censored_pixel_diff_count == 0` outside the mask.
3. **Contrastive Data Curation & False-Positive Suite (`src/workspace.rs`):** Automated confuser hardness scoring (`fp_hardness_score`), Polarity De-Confliction (`IoU < 0.20`), Hardness-Weighted NMS (`IoU < 0.45`), `dataset-audit` CLI health checker, and 5-format dataset exporter (`COCO`, `JSONL`, `ShareGPT SFT`, `Contrastive DPO`, and `HuggingFace Dataset Card`).
4. **Dual Interface (`13 CLI Commands` + `15 JSON-RPC 2.0 MCP Tools` + Interactive `dashboard.html`):** Works headlessly with autonomous VLM agents (`Claude`, `Gemini`, `GPT-4o`) and provides a zero-dependency browser QC studio with live vector grounding boxes (`[b]`) and A/B inspection (`[o]`).
5. **Production Dogfooding:** Currently operating in production for **HEAV3NS Studio** (`MAJO` & `EGG`) for automated artwork compliance and dataset curation.

---

### 5. How long have each of you been working on this? How much of that has been full-time?
*(Fill in exact months/weeks for EGG & MAJO — emphasize the high-velocity engineering sprint that produced v1.0 through v4.0 of the Rust engine, MCP server, and contrastive dataset pipeline).*

---

### 6. What tech stack are you using, or planning to use, to build this product?
- **Core Engine:** Pure **Rust (`2021 Edition`, `#![forbid(unsafe_code)]`)** for deterministic, memory-safe, sub-second image processing (`image`, `serde`, `sha2`, `clap`).
- **Agent Protocol:** Native **Model Context Protocol (`MCP` JSON-RPC 2.0 over `stdio`)** exposing 15 structured tools to multimodal VLMs.
- **Interactive QC Studio:** Zero-dependency Vanilla HTML5/CSS3/JS (`ledger/dashboard.html`) with SVG/DOM vector bounding-box overlays and interactive coordinate HUD.
- **Dataset & ML Interop:** Native exports for **HuggingFace `datasets` (`JSONL`)**, **MS-COCO (`JSON`)**, **Ultralytics YOLO**, **ShareGPT VLM Supervised Fine-Tuning (`SFT`)**, and **TRL / Alignment Handbook Direct Preference Optimization (`DPO`)**.

---

## Section 3: Founder-Market Fit & Origin Story

### 7. Why did you pick this idea to work on? Do you have domain expertise in this area? How do you know people need what you're making?

We started from a problem we experienced firsthand as artists and builders at **HEAV3NS**.

Initially, we needed an autonomous post-production system to prepare our own studio artwork (`EGG`, Principal Artist; `MAJO`, Executive Chair) for cross-platform publication (such as Japanese Article 175 mosaic/bar standards) without ruining the underlying art or wasting hours on manual Photoshop masking.

When we tested existing AI tools, we discovered something shocking:
1. Off-the-shelf NSFW classifiers falsely flagged classical silhouettes, anatomical linework, and warm skin-tone shading **up to 40% of the time**.
2. Generative inpainting tools mutated every pixel in the canvas through VAE re-encoding, destroying brushwork and alpha channels.

When we built a deterministic Rust pixel engine (`heav3ns-censor`) that pairs VLM spatial grounding with **zero-collateral localized redaction (`0` exterior pixel diffs)** and **explicit hard-negative confuser annotations**, we realized we hadn't just built a studio post-production tool—**we had solved a massive upstream bottleneck in how generative AI training datasets are cleaned.**

Because we are both artists and systems engineers, we saw what pure ML teams missed: you shouldn't throw away a masterpiece or smear synthetic diffusion noise over it just because 1.5% of the canvas needs localized redaction—and you must record verified benign hard-negatives so safety models stop censoring human art.

---

## Section 4: Market, Differentiation, & Monetization

### 8. Who are your competitors? What do you understand about your business that they don't?

Our competitors fall into three buckets:
1. **Coarse Legacy NSFW Classifiers (`NudeNet`, `OpenNSFW`, `LAION-Safety`, `Hive`):** They treat curation as a binary image-level or coarse-box rejection problem, causing massive sample-dropping bias (`15%–40%` false positives on figurative art) and offering zero dataset recovery or contrastive `DPO` export.
2. **Generative Inpainting Pipelines (`Stable Diffusion Inpainting`, commercial cleanup APIs):** They hallucinate synthetic pixels and corrupt non-target pixels via lossy VAE encoding, making their outputs toxic for foundation model pre-training.
3. **General-Purpose Labeling Platforms (`Scale AI`, `Roboflow`, `Voxel51 FiftyOne`, `Cleanlab`, `Label Studio`):** They provide great general dataset management, but lack a **deterministic, cryptographically verified pixel redaction engine**, **multi-scale JPEG DCT survival verification (`1080p`/`540p`)**, and **automated artistic/anatomical confuser hardness scoring (`fp_hardness_score`)**.

**What we understand that they don't:**
Data curation for visual safety is not a binary deletion problem—it is a **signal preservation and contrastive grounding problem**. By preserving 98.5% of an image bit-identically (`non_censored_pixel_diff_count == 0`) while exporting paired positive (`class_id = 1`) and verified benign hard-negative (`class_id = 0`) boxes in `COCO`, `SFT`, and `DPO` formats, we turn discarded "toxic" datasets into high-value alignment goldmines.

---

### 9. How do or will you make money? How much could you make?

We follow the proven **Open-Core + Data Foundry** business model (**Roboflow YC S20**, **Supabase YC S20**, **Scale AI YC S16**):

1. **Open-Source Core (`MIT License` — Free):** Local Rust CLI & MCP server drives bottom-up adoption among AI researchers, fine-tuners, and digital studios.
2. **HEAV3NS Cloud Curation API & Lakehouse (`$499 – $4,999/month` SaaS + per-10k image volume pricing):** Hosted high-throughput curation pipeline with S3/HuggingFace connectors, automated VLM triage fleets, multi-seat escalation review, and cryptographic EU AI Act / Article 175 audit certificates.
3. **Enterprise Contrastive Alignment Datasets (`$100k – $1M+/year` Annual Licensing):** Licensing massive, artist-and-agent-verified **Contrastive False-Positive Suppression Datasets (`SFT` + `DPO`)** to foundation model labs and generative image/video platforms.

**Market Scale:**
- **Bottom-up:** `500` generative AI startups, model fine-tuning studios, and digital media platforms at `$50k` ACV = `$25M` ARR; `50` frontier/enterprise AI labs & dataset providers at `$500k` ACV = `$25M` ARR (`$50M` near-term ARR beachhead).
- **Top-down:** Expanding our deterministic zero-collateral redaction engine into **medical imaging PHI de-identification (HIPAA)**, **autonomous vehicle privacy redaction**, and **enterprise video compliance** addresses the broader **`$14.8B` AI Data Curation & Visual Compliance market**.

---

## Section 5: The 10-Minute YC Partner Interview Gauntlet (Q&A Prep)

YC interviews last **10 minutes** and are rapid-fire. Memorize these crisp, 20-second responses:

### Q1: *"So what do you actually do?"*
> **"We build preservation-aware visual dataset curation for AI. Today, when AI labs clean image datasets, coarse safety filters falsely delete up to 40% of figurative art and anatomy, or corrupt images with diffusion inpainting. Our open-source Rust engine surgically masks only the exact sensitive pixels with zero exterior pixel drift, while auto-mining benign hard-negative confusers into ready-to-train COCO and DPO datasets."**

### Q2: *"Why not just drop the flagged images? Doesn't the internet have billions of images?"*
> **"Not of high-aesthetic human art, anatomy, and complex lighting. Frontier image and video models are hitting a quality wall specifically because bulk safety filters systematically strip out classical art, figurative studies, and dynamic human poses (`arXiv:2409.17156`). Dropping 35% of your highest-aesthetic art corpus biases the model's weights and degrades anatomical generation."**

### Q3: *"Why can't an AI lab just write a Python script with OpenCV `cv2.GaussianBlur`?"*
> **"Three reasons we proved in our engineering postmortems: First, naive bounding-box blurs leak high-frequency edges at the boundary and fail after `540p` `4:2:0` JPEG thumbnail compression—our engine simulates DCT quantization and verifies 95th-percentile frequency suppression. Second, axis-aligned boxes destroy up to 60% more artwork than necessary on angled anatomy, whereas we use analytical inverse-rotated Signed Distance Fields (`Pill`, `Ellipse`). Third, OpenCV doesn't mine contrastive hard-negative confusers or export VLM `DPO` preference pairs."**

### Q4: *"How do you go from an open-source Rust repo to $100M ARR?"*
> **"The exact same way Roboflow and Scale AI did. Single-node open-source CLI wins developers and researchers bottom-up. When an enterprise AI team needs to run autonomous VLM triage and human escalation across 50 million images in S3 with compliance audit logs—or wants to license pre-curated contrastive DPO datasets to fix false positives in their safety guardrail—they pay us `$50k` to `$500k` a year."**
