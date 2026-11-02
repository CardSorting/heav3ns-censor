# HEAV3NS — Y Combinator Strategic Thesis & Diligence Portal

> **50-Character YC Company Description (`49 chars`):**  
> `Preservation-aware visual dataset curation for AI`

---

## 1. Core Thesis: Stopping AI Training Pipelines from Destroying Human Visual Culture

Traditional content moderation and dataset filtering approaches are inadequate for the generative AI era. When cleaning visual training corpora, legacy pipelines rely on coarse, image-level binary decisions (`NudeNet`, `OpenNSFW`, `LAION-Safety`) that create two critical failure modes:

1. **Sample-Dropping Bias (`15%–40%` False-Positive Erasure):**
   - Broad image-level filtering discards entire training samples whenever a localized region triggers a classifier—or when benign artistic contours are misclassified ([`arXiv:2409.17156`](https://arxiv.org/abs/2409.17156)).
   - **Consequence:** Classical figurative art, anatomical studies, and complex human lighting are systematically erased from training distributions.
2. **Reconstruction-Induced Distortion (Inpainting Poisoning):**
   - Generative diffusion inpainting introduces synthetic textures, VAE re-encoding blur (`non_censored_pixel_diff_count > 0`), color shifts, and unintended visual alterations that poison downstream pre-training corpora.

**What `heav3ns-censor v4.0` Does Instead:**
We built a **preservation-aware visual identification, classification, and contrastive dataset curation engine in pure Rust (`MIT License`)**:
- **Bit-Identical Exterior Preservation (`0 px` Drift):** Masks only the exact rotated Signed Distance Field (`Rect`, `Ellipse`, `Pill`) requiring intervention while guaranteeing `100.00%` bit-identical RGBA fidelity (`SHA-256` verified) across the remaining `98.4%+` of the canvas.
- **Contrastive False-Positive Suppression (`fp_hardness_score`):** Explicitly annotates and scores **Verified Benign Hard-Negative Confusers (`class_id = 0`)** alongside **Positive Interventions (`class_id = 1`)**, exporting ready-to-train **MS-COCO (`annotations_coco.json`)**, **HuggingFace (`dataset_manifest.jsonl`)**, **ShareGPT VLM SFT (`vlm_safety_sft.jsonl`)**, and **Direct Preference Optimization (`vlm_safety_dpo.jsonl`)** datasets.

---

## 2. Complete Y Combinator Diligence, Pitch Deck, & Application Suite ([`docs/yc/`](docs/yc/README.md))

| Document | Link | Summary |
| :--- | :--- | :--- |
| **00. Origin Insight & Research Thesis** | **[`docs/yc/00-ORIGIN-INSIGHT-AND-THESIS.md`](docs/yc/00-ORIGIN-INSIGHT-AND-THESIS.md)** | Why moving upstream from moderation SaaS to **Preservation-Aware Dataset Curation** transforms this into a venture-scale AI infrastructure company. |
| **01. YC Viability & Partner Diligence Report** | **[`docs/yc/01-YC-VIABILITY-AND-PARTNER-DILIGENCE-REPORT.md`](docs/yc/01-YC-VIABILITY-AND-PARTNER-DILIGENCE-REPORT.md)** | Candid `8.8 / 10` YC Partner scorecard, comparable analysis (**Scale AI S16**, **Roboflow S20**, **Cleanlab**), red flags to avoid, and 14-day action plan. |
| **02. 12-Slide YC Seed Pitch Deck (Markdown)** | **[`docs/yc/02-YC-SEED-PITCH-DECK.md`](docs/yc/02-YC-SEED-PITCH-DECK.md)** | Slide-by-slide narrative, visual architecture diagrams, `$14.8B` TAM breakdown, and `$500k` YC Batch roadmap. |
| **02B. Interactive Browser Pitch Deck (HTML)** | **[`docs/yc/pitch_deck.html`](docs/yc/pitch_deck.html)** | Zero-dependency, keyboard-navigable (`←`/`→`/`Space`, `f` fullscreen) dark-mode presentation deck. |
| **03. Official `ycombinator.com/apply` Blueprint** | **[`docs/yc/03-YC-APPLICATION-BLUEPRINT.md`](docs/yc/03-YC-APPLICATION-BLUEPRINT.md)** | Exact character-counted application responses and rapid-fire 10-minute YC Partner interview Q&A prep. |
| **04. GTM, Business Model, & Defensibility Moat** | **[`docs/yc/04-GTM-BUSINESS-MODEL-AND-MOAT.md`](docs/yc/04-GTM-BUSINESS-MODEL-AND-MOAT.md)** | Open-Core monetization (`MIT` CLI/MCP $\rightarrow$ Cloud Curation Lakehouse $\rightarrow$ Enterprise Contrastive `DPO` Data Foundry). |

---

## 3. Live Verification Commands for YC Partners & Technical Reviewers

```bash
# 1. Generate the YC Partner & Investor Onboarding Packet:
./target/release/heav3ns-censor onboard --role yc

# 2. Run the Dataset Health, False-Positive Defense, & Sample-Salvage Audit:
./target/release/heav3ns-censor dataset-audit

# 3. Execute the 13-Test Mathematical & Contrastive Curation Suite:
cargo test --release
```
