[🏠 Home (`README.md`)](../../README.md) &rsaquo; [📚 Docs (`docs/`)](../README.md) &rsaquo; [🚀 YC Package (`docs/yc/`)](README.md) &rsaquo; **03: `ycombinator.com/apply` Application Blueprint**

---

# Official Y Combinator Application Blueprint (`https://www.ycombinator.com/apply`)

> **Founder Profile Integrated:** Incorporates the verified operator and technical credentials of **William Andrew Cruz** (**HEAV3NS**, Founding Independent Industry Operator; past completed technical roles at **OpenAI** as AI Safety Researcher Intern `2023–2024`, **Meta** as AI Intern `2022–2023`, and **Nous Research** `May–Aug 2026`; directing `230+` open-source repositories; leading Silicon Slopes Founder Mentor) alongside Executive Chair **MAJO**.

---

## Section 1: Company Overview

### 1. Describe what your company does in 50 characters or less.
*(Strict limit: 50 characters including spaces. Verified 49-character answer:)*

```text
Preservation-aware visual dataset curation for AI
```
*(49 characters)*

---

### 2. What is your company going to make? Please describe your product and what it does or will do.

We build **HEAV3NS (`heav3ns-censor`)**, an open-core Rust engine and agentic MCP platform for **preservation-aware visual dataset curation and contrastive false-positive suppression** in generative AI pipelines.

When multimodal AI teams clean visual training datasets today, they face two destructive failure modes:
1. **Sample-Dropping Bias (`15%–40%` False-Positive Loss):** Legacy NSFW classifiers (`NudeNet`, `OpenNSFW`, `LAION-Safety`) make coarse whole-image binary rejections. Published research (`arXiv:2409.17156`) demonstrates these classifiers have a **15%–40% false-positive rate** on classical painting, sculpture, anatomical studies, and garment folds—erasing millions of high-aesthetic human artworks from training corpora.
2. **Inpainting Poisoning (VAE Reconstruction Drift):** Pipelines that attempt to modify flagged regions with diffusion inpainting pass the image through a lossy VAE, mutating 100% of exterior pixels and injecting synthetic artifacts that degrade downstream model training.

Our pure Rust engine (`heav3ns-censor v4.0`, MIT License) solves both problems deterministically:
- **Zero-Collateral Surgical Redaction (`Class 1`):** When an image contains a localized sensitive region (e.g., `1.6%` of the canvas), our inverse-rotated Signed Distance Field (SDF) engine masks only that exact contour (`mosaic`, `solid_bar`, or `gaussian_blur`) while guaranteeing **100% bit-identical preservation (`0 px` exterior diff, SHA-256 verified)** across the remaining `98.4%` of the canvas, verified against `1080p` and `540p` `4:2:0` JPEG recompression.
- **Contrastive Hard-Negative Mining (`Class 0` + `fp_hardness_score`):** When a region is a benign confuser (classical silhouettes, warm skin-tone shading, garment seams), our engine scores its false-positive hardness (`5.0–99.5`), deduplicates boxes via Hardness-Weighted NMS (`IoU < 0.45`), enforces Polarity De-Confliction (`IoU < 0.20`), and exports synchronized **MS-COCO (`annotations_coco.json`)**, **HuggingFace (`dataset_manifest.jsonl`)**, **ShareGPT VLM SFT (`vlm_safety_sft.jsonl`)**, and non-degenerate **Direct Preference Optimization (`vlm_safety_dpo.jsonl`)** datasets.

---

### 3. Where do you live now, and where would the company be based after YC?
**Current:** Salt Lake City, UT (`95 N. 300 W.`)  
**After YC:** San Francisco Bay Area, CA / Salt Lake City, UT.

---

## Section 2: Founder-Market Fit & Credentials

### 4. Tell us about the founder(s) and why you are uniquely qualified to build this.

**William Andrew Cruz (`EGG` — Founder, Principal Artist & Systems Architect, HEAV3NS):**
- **Active Role:** Founding Independent Industry Operator at **HEAV3NS** (Oct 2026–Present, Salt Lake City, UT), directing agentic systems, Rust software architecture, and developer tooling across **230+ open-source repositories** (`github.com/CardSorting`).
- **Past Frontier AI Lab Foundation:**
  - **OpenAI** — *AI Safety Researcher Intern (2023–2024, Completed Past Technical Internship)*
  - **Meta** — *AI Intern (2022–2023, Completed Past Technical Internship)*
  - **Nous Research** — *DevRel / Community Lead SLC (May–Aug 2026, Completed Past Role)*
- **Ecosystem & Technical Credentials:** Leading Silicon Slopes Founder Mentor (guided hundreds of startup founders); certified `PSM I`, `PSPO I`, `CompTIA Security+ ce`, `CompTIA A+ ce`, and `Six Sigma Black Belt`.
- **Commissioning & Operations Leadership:** Partnered with **MAJO** (Executive Chair, HEAV3NS) on studio compliance governance and escalation standards.

**Why This Founder Duality Is Our Unfair Advantage:**
Most ML safety engineers treat images as disposable tensors—which is why legacy pipelines blindly delete Bernini sculptures or smear synthetic diffusion noise over human brushwork. Because I have worked inside **OpenAI Safety, Meta AI, and Nous Research** AND operate as a working visual artist (`EGG`), I experienced both sides of this failure mode firsthand and built the mathematical Rust engine to bridge **artistic preservation** and **contrastive AI safety alignment**.

---

## Section 3: Progress, Velocity, & Technical Proof

### 5. How far along are you?
We have built, verified, and open-sourced **`heav3ns-censor v4.0.0`** (`3,700+` lines of zero-unsafe Rust 2021, `MIT License`, `13/13` automated mathematical & curation invariant tests passing) at `https://github.com/CardSorting/heav3ns-censor`:
1. **Analytical Rotated SDF Pixel Engine (`src/engine.rs`):** Supports `Rect`, `Ellipse`, and round-capped `Pill` primitives at arbitrary `@<angle_deg>` with immutable-source remainder pooling and luminance-adaptive contrast (`auto`).
2. **Multi-Scale Frequency & JPEG Verifier:** Simulates `1080p (Q85)` and `540p (Q80)` `4:2:0` chroma-subsampled JPEG compression, verifying Mean $\ge 70\%$ and $P_{95}$ Peak $\ge 60\%$ suppression while guaranteeing `non_censored_pixel_diff_count == 0` outside masks.
3. **Contrastive False-Positive Suite (`src/workspace.rs`):** Automated confuser hardness scoring (`fp_hardness_score`), Polarity De-Confliction (`IoU < 0.20`), Hardness-Weighted NMS (`IoU < 0.45`), `dataset-audit` CLI health checker (`100.0/100.0` health score, `99.20%` ground-truth pixel preservation), and 5-format exporter (`COCO`, `JSONL`, `VLM SFT`, `Contrastive DPO`, `Dataset Card`).
4. **15-Tool Agentic MCP Server & Interactive QC Studio (`src/mcp.rs` & `ledger/dashboard.html`):** Works headlessly with Claude/Gemini/GPT-4o over JSON-RPC 2.0 `stdio` and provides a zero-dependency browser QC studio with `[b]` live Cyan/Green Dual-Polarity Grounding Boxes and `[o]` A/B comparison.

---

## Section 4: Business Model, Market Size, & 90-Day YC Plan

### 6. How do or will you make money? How much could you make?
We combine the **Roboflow (YC S20) / Astral** bottom-up open-source wedge with the **Scale AI (YC S16)** contrastive data foundry model across a **`$14.8B` TAM**:

1. **Tier 1 — Open-Source Community Core (`MIT License` — Free Forever):** Local Rust CLI, 15-tool MCP server, and local HTML QC studio. Drives viral bottom-up adoption among AI researchers, fine-tuners, and digital studios.
2. **Tier 2 — HEAV3NS Cloud Curation Lakehouse (`$499–$4,999/mo` Usage SaaS):** Hosted S3/GCS/HuggingFace Hub batch pipelines, automated VLM triage fleets, multi-seat escalation queues, and cryptographic EU AI Act / Article 175 audit certificates.
3. **Tier 3 — Enterprise Contrastive DPO Foundry (`$100k–$1M+/yr` Licensing):** Licensing curated, artist-and-agent-verified **Contrastive Hard-Negative Alignment Datasets (`SFT` + `DPO`)** to frontier VLM labs, plus expanding our zero-collateral pixel redaction engine into medical PHI de-identification (HIPAA) and autonomous vehicle video privacy.

### 7. What is your 90-Day Plan during the YC Batch?
- **Step 1 (Weeks 1–3):** Release **`HEAV3NS-ArtSafety-100K-DPO`** on Hugging Face (100,000 classical & figurative artworks falsely flagged by `NudeNet`/`OpenNSFW`, restored with verified hard-negative `COCO`, `SFT`, & `DPO` annotations).
- **Step 2 (Weeks 4–7):** Launch the self-serve **HEAV3NS Cloud Curation API** (`POST /v1/curate-batch`) with native S3 and HuggingFace Hub bucket sync (`10` paying teams / `$15k` MRR target).
- **Step 3 (Weeks 8–12 — Demo Day):** Close `5` enterprise foundation model & generative studio contracts (`$40k+` MRR / **`$500k` ARR run-rate by Demo Day**).
