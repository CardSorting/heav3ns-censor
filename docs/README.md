[🏠 Home (`README.md`)](../README.md) &rsaquo; **📚 Documentation Portal (`docs/`)**

---

# HEAV3NS-CENSOR Documentation & Research Portal

Welcome to the complete engineering, operational, and academic documentation suite for **`heav3ns-censor` (`v4.0.0`)**, organized using the **[Diátaxis Framework](https://diataxis.fr/)** (*Onboarding & Tutorials*, *How-To Playbooks*, *Domain Reference*, and *Architecture & Explanation*).

---

## 1. Role-Based Onboarding & Handoff (`docs/onboarding/` & `docs/stakeholder-handoff/`)

| Persona | Guide | Primary Focus | Live CLI Command |
| :--- | :--- | :--- | :--- |
| **🔬 AI Safety & Dataset Researchers** | **[`03-dataset-cleaning-and-ai-safety-research.md`](knowledgebase/03-dataset-cleaning-and-ai-safety-research.md)** | Eliminating sample-dropping bias & inpainting poisoning, paired counterfactual safety benchmarks, exporting `dataset_manifest.jsonl` (`COCO` + `Normalized1000` + `YOLO`) | `./target/release/heav3ns-censor onboard --role researcher` |
| **🎨 Studio Stakeholders** (**MAJO** & **EGG**) | **[`STAKEHOLDER_HANDOFF.md`](stakeholder-handoff/STAKEHOLDER_HANDOFF.md)** | Executive charter verification, Interactive Studio QC Dashboard (`dashboard.html` with `[o]` A/B toggle), escalation review | `./target/release/heav3ns-censor onboard --role stakeholder` |
| **🤖 Autonomous AI Agents** (`Deputy Gemini` / MCP) | **[`AGENT_ONBOARDING.md`](onboarding/AGENT_ONBOARDING.md)** | Decision flowchart, VLM coordinate grid grounding, $3\times$ zoom-crop, CLI/MCP tool mapping | `./target/release/heav3ns-censor onboard --role agent` |
| **🛠️ Rust Engineers & Contributors** | **[`DEVELOPER_ONBOARDING.md`](onboarding/DEVELOPER_ONBOARDING.md)** | 6-module Rust architecture map (`src/`), 15-minute extension recipes, 9-test verification suite | `./target/release/heav3ns-censor onboard --role developer` |

---

## 2. Foundational Tri-Pillar Documents

- **[Artistic Preservation & Post-Production Philosophy (`PHILOSOPHY.md`)](philosophy/PHILOSOPHY.md)**  
  Why generative inpainting is forbidden, the zero-collateral pixel mandate (`non_censored_pixel_diff_count == 0`), Japanese *Bokashi* & *Kuro-Nuri* lineage, and the Artist–Executive–Agent triad.
- **[Executive & Studio Stakeholder Handoff (`STAKEHOLDER_HANDOFF.md`)](stakeholder-handoff/STAKEHOLDER_HANDOFF.md)**  
  Complete operational handoff for **MAJO** (Executive Chair) and **EGG** (Principal Artist).
- **[Academic & Systems Technical Whitepaper (`TECHNICAL_WHITEPAPER.md`)](whitepaper/TECHNICAL_WHITEPAPER.md)**  
  *Deterministic Zero-Collateral Localized Image Obscuration with Multi-Scale Frequency-Domain & JPEG-Quantization Resilience Verification*.

---

## 3. Domain Reference Knowledgebase ([`docs/knowledgebase/`](knowledgebase/INDEX.md))

- **[`INDEX.md`: Knowledgebase Master Index & Domain Glossary](knowledgebase/INDEX.md)**
- **[`KB-01`: Japanese Penal Code Article 175 & Cross-Platform Censorship Standards](knowledgebase/01-japanese-article-175-and-platform-standards.md)**
- **[`KB-02`: Coordinate Systems & Multimodal VLM Grounding](knowledgebase/02-coordinate-systems-and-vlm-grounding.md)**
- **[`KB-03`: Dataset Cleaning, Multimodal AI Safety, & Non-Destructive Artistic Post-Production](knowledgebase/03-dataset-cleaning-and-ai-safety-research.md)**

---

## 4. Operational Playbooks ([`docs/playbooks/`](playbooks/README.md))

- **[`PLAYBOOK-01`: Autonomous Batch Ingestion & VLM Triage](playbooks/PLAYBOOK-01-autonomous-batch-ingestion-and-vlm-triage.md)**
- **[`PLAYBOOK-02`: Precision Censorship & Incremental Region Editing](playbooks/PLAYBOOK-02-precision-censorship-and-incremental-region-editing.md)**
- **[`PLAYBOOK-03`: Ambiguity Escalation, Revocation, & Artist Review](playbooks/PLAYBOOK-03-ambiguity-escalation-revocation-and-artist-review.md)**
- **[`PLAYBOOK-04`: Workspace Doctor Audit & Incident Recovery](playbooks/PLAYBOOK-04-workspace-doctor-audit-and-incident-recovery.md)**
- **[`PLAYBOOK-05`: Contrastive Data Annotation & False-Positive Suppression](playbooks/PLAYBOOK-05-data-annotation-and-false-positive-suppression.md)**

---

## 5. Architecture Decision Records ([`docs/adrs/`](adrs/README.md))

- **[`ADR-0001`: Deterministic Rust Pixel Engine Over Generative Inpainting](adrs/ADR-0001-deterministic-pixel-engine-over-generative-inpainting.md)**
- **[`ADR-0002`: Immutable-Source Sampling and Inverse-Rotated SDF Masks](adrs/ADR-0002-immutable-source-sampling-and-inverse-rotated-sdf-masks.md)**
- **[`ADR-0003`: Multi-Scale JPEG Resilience and P95 Peak Verification](adrs/ADR-0003-multiscale-jpeg-resilience-and-p95-peak-verification.md)**
- **[`ADR-0004`: Atomic BTreeMap Ledger and Dual CLI/MCP Interface](adrs/ADR-0004-atomic-btree-ledger-and-dual-cli-mcp-interface.md)**

---

## 6. Engineering Postmortems ([`docs/postmortems/`](postmortems/README.md))

- **[`PM-001`: Mosaic Remainder-Strip High-Frequency Edge Leakage](postmortems/PM-001-mosaic-remainder-strip-high-frequency-edge-leakage.md)**
- **[`PM-002`: Rotated AABB Clipping & Background Gradient Dilution](postmortems/PM-002-rotated-aabb-clipping-and-background-gradient-dilution.md)**
- **[`PM-003`: Downscaled 540p Fractional Mosaic Grid Misalignment](postmortems/PM-003-downscaled-540p-fractional-mosaic-grid-misalignment.md)**
- **[`PM-004`: In-Place Canvas Mutation & Zero-Sample Verification False Positives](postmortems/PM-004-in-place-canvas-mutation-and-zero-sample-false-positives.md)**

