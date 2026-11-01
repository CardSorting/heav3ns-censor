# HEAV3NS-CENSOR Knowledgebase

Welcome to the central engineering and domain knowledgebase for **`heav3ns-censor` (`v4.0.0`)**. This documentation suite is structured for studio operators, autonomous AI agents, computer vision researchers, and open-source contributors.

---

## 1. Foundational Tri-Pillar Documents

| Pillar | Document | Audience | Summary |
| :--- | :--- | :--- | :--- |
| **Philosophy** | [`docs/philosophy/PHILOSOPHY.md`](../philosophy/PHILOSOPHY.md) | Artists, Curators, Ethics & Policy Researchers | Zero-collateral artistic preservation, rejection of generative inpainting, Japanese *Bokashi* & *Kuro-Nuri* aesthetic lineage, and the Artist–Executive–Agent triad. |
| **Stakeholder Handoff** | [`docs/stakeholder-handoff/STAKEHOLDER_HANDOFF.md`](../stakeholder-handoff/STAKEHOLDER_HANDOFF.md) | MAJO (Executive Chair), EGG (Principal Artist), Studio Ops | End-to-end operational handoff, directory contract, interactive Studio QC Dashboard guide, and acceptance verification matrix. |
| **Technical Whitepaper** | [`docs/whitepaper/TECHNICAL_WHITEPAPER.md`](../whitepaper/TECHNICAL_WHITEPAPER.md) | CV/Graphics Researchers, Systems Engineers | Formal mathematical derivation of inverse-rotated SDF primitives, remainder-strip edge pooling, and fractional-step multi-scale JPEG verification. |

---

## 2. Domain Knowledgebase Articles

- **[`KB-01`: Japanese Article 175 & Cross-Platform Censorship Standards](01-japanese-article-175-and-platform-standards.md)**  
   Detailed breakdown of Japanese Penal Code Article 175 mosaic block sizing ($\max(W, H)/100$), *kuro-nuri* / *shiro-nuri* editorial conventions, and policy profiles (`jp_article_175_standard`, `kuro_nuri_graphic`, `social_media_safe`).
- **[`KB-02`: Coordinate Systems & Multimodal VLM Grounding](02-coordinate-systems-and-vlm-grounding.md)**  
  How `heav3ns-censor` bridges the spatial grounding gap in Vision-Language Models using burned-in `Normalized1000` bitmap badges, automatic high-frequency hotspot detection, and 3x magnified zoom-crop grids.

---

## 3. Architecture Decision Records (`docs/adrs/`)

- **[`ADR-0001`: Deterministic Pixel Engine Over Generative Inpainting](../adrs/ADR-0001-deterministic-pixel-engine-over-generative-inpainting.md)**
- **[`ADR-0002`: Immutable-Source Sampling & Inverse-Rotated SDF Masks](../adrs/ADR-0002-immutable-source-sampling-and-inverse-rotated-sdf-masks.md)**
- **[`ADR-0003`: Multi-Scale JPEG Resilience & P95 Peak Verification](../adrs/ADR-0003-multiscale-jpeg-resilience-and-p95-peak-verification.md)**
- **[`ADR-0004`: Atomic BTreeMap Ledger & Dual CLI/MCP Interface](../adrs/ADR-0004-atomic-btree-ledger-and-dual-cli-mcp-interface.md)**

---

## 4. Operational Playbooks (`docs/playbooks/`)

- **[`PLAYBOOK-01`: Autonomous Batch Ingestion & VLM Triage](../playbooks/PLAYBOOK-01-autonomous-batch-ingestion-and-vlm-triage.md)**
- **[`PLAYBOOK-02`: Precision Censorship & Incremental Region Editing](../playbooks/PLAYBOOK-02-precision-censorship-and-incremental-region-editing.md)**
- **[`PLAYBOOK-03`: Ambiguity Escalation, Revocation, & Artist Review](../playbooks/PLAYBOOK-03-ambiguity-escalation-revocation-and-artist-review.md)**
- **[`PLAYBOOK-04`: Workspace Doctor Audit & Incident Recovery](../playbooks/PLAYBOOK-04-workspace-doctor-audit-and-incident-recovery.md)**

---

## 5. Engineering Postmortems (`docs/postmortems/`)

- **[`PM-001`: Mosaic Remainder-Strip High-Frequency Edge Leakage](../postmortems/PM-001-mosaic-remainder-strip-high-frequency-edge-leakage.md)**
- **[`PM-002`: Rotated AABB Clipping & Background Gradient Dilution](../postmortems/PM-002-rotated-aabb-clipping-and-background-gradient-dilution.md)**
- **[`PM-003`: Downscaled 540p Fractional Mosaic Grid Misalignment](../postmortems/PM-003-downscaled-540p-fractional-mosaic-grid-misalignment.md)**
- **[`PM-004`: In-Place Canvas Mutation & Zero-Sample Verification False Positives](../postmortems/PM-004-in-place-canvas-mutation-and-zero-sample-false-positives.md)**

---

## 6. Domain Glossary

| Term | Definition |
| :--- | :--- |
| **Bokashi (ぼかし) / Mosaic** | Pixelation censorship treatment that replaces square blocks of pixels with their arithmetic mean color. |
| **Kuro-Nuri (黒塗り)** | Solid black (`#000000`) graphic bar or capsule used in Japanese print/digital media to mask explicit anatomical lines/clefts. |
| **Shiro-Nuri (白塗り)** | Solid white (`#FFFFFF`) graphic bar used on dark-background illustrations where black bars would lack visual contrast. |
| **Enclosing AABB** | The dynamically expanded axis-aligned bounding box $(W_{\text{enc}}, H_{\text{enc}})$ guaranteed to contain a shape rotated by $\theta$. |
| **Normalized1000** | Resolution-independent coordinate space $[0, 1000] \times [0, 1000]$ used by multimodal VLMs and burned into inspection grids. |
| **Remainder-Strip Pooling** | Folding a boundary strip narrower than $0.5 \times \text{block\_size}$ into the preceding full mosaic cell so no narrow edge strip leaks detail. |
