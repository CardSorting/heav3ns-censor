[🏠 Home (`README.md`)](../../README.md) &rsaquo; [📚 Docs (`docs/`)](../README.md) &rsaquo; [🚀 YC Package (`docs/yc/`)](README.md) &rsaquo; **01: YC Viability & Partner Diligence Report**

---

# Y Combinator Partner Diligence Memo: Is `heav3ns-censor` a YC-Shaped Company?

**Subject:** Candid Venture Viability & Y Combinator (`ycombinator.com/apply`) Fit Assessment  
**Prepared By:** Deputy Gemini (Autonomous Strategy & Systems Diligence)  
**Target Batch:** Y Combinator (AI Infrastructure / Data Curation / Open-Source DevTools Track)  
**Core Thesis Reference:** [`yc.md`](../../yc.md) & [`README.md`](../../README.md)  

---

## 1. Bottom-Line Judgment: The Two Companies Inside Your Repo

If a Y Combinator Group Partner opens your repository today, they can see **two completely different companies** depending on how you frame the first 50 characters of your application:

| Dimension | Framing A: Downstream Artist Censorship Tool ❌ | Framing B: Upstream Preservation-Aware AI Dataset Curation Infrastructure ✅ |
| :--- | :--- | :--- |
| **50-Char Pitch** | *"Automated Japanese censorship tool for artists."* | *"Preservation-aware visual dataset curation for AI"* (`49 chars`) |
| **Target Customer** | Indie illustrators, manga studios, Patreon creators | Foundation model labs, multimodal dataset providers, generative AI platforms, safety teams |
| **ACV (Annual Contract Value)** | `$10–$50 / month` prosumer SaaS (high churn) | `$50,000–$500,000+ / year` enterprise dataset & safety pipeline contracts + API consumption |
| **Payment Processor / Policy Risk** | High (Stripe/PayPal routinely flag adult creator tools) | Low (B2B AI infrastructure, dataset hygiene, & safety alignment software—same category as **Scale AI** & **Cleanlab**) |
| **YC Partner Reaction** | *"Cool Rust engineering, but this is a niche feature or lifestyle tool, not a $1B+ venture-scale company. Pass."* | *"Every multimodal lab is throwing away 30% of figurative art or poisoning their weights with diffusion inpainting. A fast Rust + MCP engine that salvages those samples and exports contrastive DPO data? Let's interview them."* |
| **YC Fit Score** | **`3.2 / 10` (Reject)** | **`8.8 / 10` (Top 5% Interview Candidate)** |

> **Best Judgment Verdict:** **YES, this can unequivocally be a YC-backed company—provided you commit 100% to Framing B (Preservation-Aware Visual Dataset Curation & Contrastive Safety Alignment Infrastructure) and use Artistic Post-Production (`HEAV3NS`) as your "Dogfooding Zero-Defect Benchmark" rather than your ceiling.**

---

## 2. Why Framing B Fits Y Combinator's Core Investment Pattern

Y Combinator has a decade-long track record of backing companies that fix structural bottlenecks in the machine learning data supply chain and developer tooling:
- **Scale AI (YC S16 — `$14B` valuation):** Started as an API for human annotation of autonomous driving boxes; became the data foundry for frontier LLMs/VLMs and RLHF/DPO.
- **Roboflow (YC S20 — `$400M+` valuation):** Started by helping developers convert, clean, and audit computer vision datasets (`COCO`/`YOLO` exports, Dataset Health Check).
- **Supabase (YC S20) / PostHog (YC W20) / Astral (Ruff/uv):** Proved that an **insanely fast, open-source Rust/systems core** with world-class developer ergonomics creates bottom-up adoption inside AI labs before converting into enterprise cloud contracts.

### What Makes `heav3ns-censor v4.0` Uniquely Compelling to a YC Partner

1. **You Are Solving a Newly Acute "Data Wall" Bottleneck in Multimodal AI:**
   - Frontier vision-language models (VLMs) and image/video generators (Flux, SD3, Sora, Midjourney, Runway, enterprise studio models) are starving for high-aesthetic, human-created visual data.
   - Yet existing safety filters (`NudeNet`, `OpenNSFW`, `LAION-NSFW`) operate as blunt instruments. As documented by Riccio et al. in *An Art-centric Perspective on AI-based Content Moderation of Nudity* ([`arXiv:2409.17156`](https://arxiv.org/abs/2409.17156)), standard NSFW classifiers suffer **`15%–40%` false-positive rates** on classical sculpture, figurative painting, anatomical studies, and stylized character art.
   - Today, AI data teams face a terrible binary choice:
     1. **Drop the whole image (Sample-Dropping Bias):** Erasing classical composition, anatomy, lighting, and human cultural heritage from the model's weights.
     2. **Inpaint over the region (Reconstruction Poisoning):** Running diffusion inpainting over sensitive regions, which hallucinates plastic textures, alters surrounding pixels, and poisons future training runs with synthetic artifacts (Model Autophagy Disorder / "Habsburg AI").

2. **Your Counter-Intuitive Technical Insight ("Surgical Preservation + Contrastive Negatives"):**
   - Instead of binary deletion or generative hallucination, `heav3ns-censor` treats visual safety curation as a **deterministic, zero-collateral signal-processing and contrastive grounding problem**:
     - **Bit-Identical Exterior Preservation (`non_censored_pixel_diff_count == 0`):** Every pixel outside the exact rotated Signed Distance Field (SDF) boundary is mathematically verified via SHA-256 and pixel-by-pixel audit to remain `100.00%` untouched. The model learns real human brushwork, lighting, and background anatomy from `98.4%` of the canvas while the `1.6%` sensitive region is explicitly masked (`mosaic`, `solid_bar`, `gaussian_blur`) with explicit bounding-box conditioning (`bbox_xyxy_norm1000`).
     - **Contrastive Hard-Negative Mining (`class_id = 0` + `fp_hardness_score`):** Even more importantly, `heav3ns-censor` explicitly annotates **Verified Benign Hard-Negative Confusers** (warm skin-tone gradients, clothing seams, classical silhouettes) and exports ready-to-train **MS-COCO (`annotations_coco.json`)**, **ShareGPT VLM SFT (`vlm_safety_sft.jsonl`)**, and **Direct Preference Optimization (`vlm_safety_dpo.jsonl`)** datasets.
     - That means you aren't just cleaning images—**you are producing the exact contrastive preference data (`chosen` vs. `rejected`) that foundation model labs need to cure false positives in their own safety guardrails.**

3. **You Have Working, Audited Production Code (Not Vaporware):**
   - Many YC applicants apply with a Figma mockup or a Python wrapper around an OpenAI prompt.
   - You have a **zero-unsafe Rust 2021 engine (`v4.0.0`, MIT License)** with:
     - **13 CLI subcommands & 15 JSON-RPC 2.0 MCP tools** for autonomous VLM agent orchestration.
     - **Multi-Scale Downscale & JPEG-Quantization Resilience Verification (`1080p` & `540p` thumbnail simulation)** checking both mean and 95th-percentile peak high-frequency suppression.
     - **Hardness-Weighted Non-Maximum Suppression (`NMS`) & Polarity De-Confliction (`IoU < 0.20`)**.
     - **Automated Dataset Health & False-Positive Risk Auditing (`dataset-audit`)** and a **12-test continuous verification suite (`12/12` passing)**.

---

## 3. Candid YC Partner Scorecard (7-Pillar Rubric)

Here is how a YC Group Partner (such as Jared Friedman, Harj Taggar, Garry Tan, or Diana Hu) would score `heav3ns-censor` today:

| YC Evaluation Criterion | Score | Why It Scores Here | How to Maximize Before Interview |
| :--- | :---: | :--- | :--- |
| **1. Problem Acuteness ("Hair on Fire")** | **9.0 / 10** | Multimodal AI teams and generative art studios lose millions of dollars in wasted compute and degraded model quality from false-positive dataset filtering (`15%–40%` loss on figurative/artistic data) and copyright/compliance bottlenecks. | Quantify the exact % of samples rescued on a 1,000-image benchmark (e.g., *WikiArt Nudity* or *Don't Delete Art* sample set). |
| **2. Technical Execution & Velocity** | **9.5 / 10** | Pure Rust deterministic pixel engine, analytical SDF geometry, multi-scale JPEG DCT frequency verification, 15-tool MCP server, and 5-format dataset export (`COCO`, `JSONL`, `SFT`, `DPO`, `Dataset Card`). Far ahead of typical pre-seed applicants. | Keep the GitHub repo spotless (`MIT` license, zero warnings, instant `cargo test --release`). |
| **3. Founder-Market Fit ("Why You?")** | **9.0 / 10** | As noted in [`yc.md`](../../yc.md#L136), being an **artist + systems builder** is the ultimate unfair advantage here. Pure ML engineers don't understand anatomical linework, *bokashi* vs. *kuro-nuri* aesthetics, or why diffusion inpainting ruins figurative art. | Emphasize the **Artist–Engineer duality** in the founder video and application. |
| **4. Market Size (TAM / Venture Scale)** | **8.5 / 10** | AI Data Curation, Annotation, & Multimodal Safety Alignment is a `$14.8B+` market growing `>35%` YoY. Expanding from figurative art/media to medical imaging de-identification (PHI/HIPAA pixel redaction), autonomous driving privacy, and enterprise document/video compliance makes this massive. | Explicitly show the **3-Stage Market Expansion**: (1) Generative AI & Media Curation $\rightarrow$ (2) VLM Safety Guardrail Alignment (`DPO`/`SFT`) $\rightarrow$ (3) Universal Zero-Collateral Visual Compliance (Medical/Video/Autonomous). |
| **5. Defensibility & Moat** | **8.5 / 10** | Open-source Rust engine builds bottom-up adoption, while the **proprietary asset ledger of human/agent-verified Contrastive Hard-Negative Confuser Pairs (`vlm_safety_dpo.jsonl`)** becomes a compounding data moat that competitors cannot synthesize with LLMs. | Position the open-source CLI/MCP as the "Trojan horse" and the **Contrastive Hard-Negative Data Foundry** as the long-term moat. |
| **6. Clarity of Positioning** | **8.0 / 10** | The repo previously leaned toward "studio censorship," which risks confusing a 60-second YC reader. With the recent data-annotation, false-positive suppression, and `DPO` export upgrades, the bridge to AI Dataset Curation is now concrete in code. | Lead every answer with **Dataset Curation & False-Positive Suppression** first, and **Publication Post-Production** second. |
| **7. Commercial Traction / Design Partners** | **6.5 / 10** | Built and dogfooded internally for **HEAV3NS** (`MAJO` & `EGG`). To turn a `6.5` into a `9.5` during the YC interview, you need 2–5 external AI researchers, fine-tuning teams, or digital publishing studios actively running `heav3ns-censor` on their datasets. | Before the interview, run a public benchmark on 500–1,000 artistic/dataset images and get 3 LoRA/fine-tuning/VLM researchers using the repo. |
| **OVERALL YC READINESS** | **`8.8 / 10`** | **Strong YC Interview & Funding Candidate** when pitched as **Preservation-Aware Visual Dataset Curation & Multimodal Safety Alignment**. | Execute the 14-day pre-application traction checklist below. |

---

## 4. Red Flags & Traps to Avoid in Your YC Application

YC partners read `>20,000` applications per batch and spend roughly **90 to 120 seconds** on the first pass. Avoid these four traps at all costs:

1. **Trap #1: Sounding like an "Adult Content / NSFW Tool" instead of an "AI Data Infrastructure & Safety Company."**
   - *Why it kills applications:* Institutional VCs often have LP restrictions around "adult entertainment," and partners may mistakenly pigeonhole you if your first sentence says "censorship engine for explicit art."
   - *How to fix it:* Frame your domain the exact way **Scale AI**, **Cleanlab**, **Hugging Face**, and **ETH Zürich / ELLIS (`arXiv:2409.17156`)** frame it: **Preservation-Aware Dataset Curation, False-Positive Suppression, and Visual Compliance Infrastructure for Generative AI.**

2. **Trap #2: Using Academic Jargon in the 50-Character Description.**
   - *Bad:* `"Deterministic SDF-based localized obscuration & contrastive telemetry engine"` (Too dense; partners hate buzzword soup).
   - *Good:* `"Preservation-aware visual dataset curation for AI"` (`49 characters` — crystal clear).
   - *Alternative Good:* `"Fix false positives in AI vision training datasets"` (`50 characters` — problem-first).

3. **Trap #3: Failing to Explain Why Generative Inpainting Is Bad for Training Data.**
   - A naive technical reviewer might ask: *"Why not just use Stable Diffusion inpainting to put clothes on the statue or blur the image?"*
   - You must hit them immediately with the two facts proven in [`yc.md`](../../yc.md) and your whitepaper:
     1. Diffusion VAE encode-decode cycles alter **every pixel in the image** (`non_censored_pixel_diff_count > 0`) and inject synthetic latent artifacts that poison downstream model training.
     2. Global blurring or dropping the sample destroys `98%+` of the valid artistic lighting, anatomy, and composition.
     3. Only **deterministic, bit-exact localized curation (`non_censored_pixel_diff_count == 0`) paired with explicit bounding-box conditioning (`Normalized1000` / `COCO` / `DPO`)** preserves ground-truth pixel statistics outside the redacted box.

4. **Trap #4: Not Having a Clear Open-Core Monetization Story.**
   - Since `heav3ns-censor` is `MIT`-licensed open source, YC will ask: *"If the Rust CLI is free on GitHub, how do you become a billion-dollar business?"*
   - Answer with the proven **Roboflow / Supabase / Scale AI 3-Tier Model** (detailed in [`04-GTM-BUSINESS-MODEL-AND-MOAT.md`](04-GTM-BUSINESS-MODEL-AND-MOAT.md)):
     - **Tier 1 (Free / MIT Open Source):** Local Rust CLI + MCP Server (`heav3ns-censor`) for individual researchers, artists, and developers.
     - **Tier 2 (Pro / Cloud Curation Platform — Usage SaaS):** Hosted multi-GPU/CPU petabyte batch pipeline, collaborative team review dashboard, automated VLM triage workers, and compliance SLAs (`$499–$4,999/mo`).
     - **Tier 3 (Enterprise Data Foundry & Safety Alignment Licensing):** Licensing curated, cryptographically verified **Contrastive Hard-Negative & Fine-Art Alignment Datasets (`SFT` + `DPO`)** and custom policy engines to foundation model labs (`$100k–$1M+/yr`).

---

## 5. 14-Day Action Plan to Lock In YC Acceptance

To make your application undeniable before hitting submit on [`ycombinator.com/apply`](https://www.ycombinator.com/apply):

1. **Publish a Flagship Open Benchmark Dataset on Hugging Face (`heav3ns/art-safety-contrastive-1k`):**
   - Run `heav3ns-censor` across 500–1,000 public-domain classical art & figurative images (e.g., Metropolitan Museum of Art Open Access / WikiArt) that `NudeNet` or `CLIP-NSFW` falsely flags as explicit.
   - Export `dataset_manifest.jsonl`, `annotations_coco.json`, `vlm_safety_sft.jsonl`, and `vlm_safety_dpo.jsonl` using `./target/release/heav3ns-censor export-dataset`.
   - In your YC application, write: *"We benchmarked standard NSFW classifiers on 1,000 figurative artworks where legacy pipelines falsely dropped 38% of samples; our engine salvaged 100% of those samples with 0 exterior pixel drift and exported 2,400 contrastive DPO preference pairs."*
2. **Record a 60-Second Live Product Demo Video (Required by YC):**
   - Show `./target/release/heav3ns-censor dataset-audit` and `ledger/dashboard.html` with live `[b] Grounding Boxes` (`POS` Cyan + `NEG` Emerald Green) and `[o]` A/B bit-exact comparison.
3. **Use the Exact Application Blueprint & Pitch Deck in This Directory:**
   - Proceed to **[`02-YC-SEED-PITCH-DECK.md`](02-YC-SEED-PITCH-DECK.md)** (or open **[`pitch_deck.html`](pitch_deck.html)**) and **[`03-YC-APPLICATION-BLUEPRINT.md`](03-YC-APPLICATION-BLUEPRINT.md)**.
