<div align="center">

# `heav3ns-censor` (v4.0)

**Deterministic Japanese-Style Localized Image Obscuration & Multi-Scale Verification Studio Engine**

[![Rust 2021](https://img.shields.io/badge/rust-2021_edition-orange.svg)](Cargo.toml)
[![Version](https://img.shields.io/badge/version-4.0.0-00ffd5.svg)](CHANGELOG.md)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT_OR_Apache--2.0-blue.svg)](LICENSE)
[![MCP Protocol](https://img.shields.io/badge/MCP-2024--11--05_stdio-ff328c.svg)](src/mcp.rs)
[![Preservation Invariant](https://img.shields.io/badge/exterior_pixel_diff-0_px_(bit_exact)-3fb950.svg)](docs/whitepaper/TECHNICAL_WHITEPAPER.md)

**[Philosophy](docs/philosophy/PHILOSOPHY.md)** • **[Stakeholder Handoff](docs/stakeholder-handoff/STAKEHOLDER_HANDOFF.md)** • **[Technical Whitepaper](docs/whitepaper/TECHNICAL_WHITEPAPER.md)** • **[Knowledgebase](docs/knowledgebase/INDEX.md)** • **[ADRs](docs/adrs/)** • **[Playbooks](docs/playbooks/)** • **[Postmortems](docs/postmortems/)** • **[Contributing](CONTRIBUTING.md)**

</div>

---

## Overview & Abstract

**`heav3ns-censor`** is an autonomous, cryptographically auditable post-production censorship engine written in Rust for **HEAV3NS** (**MAJO**, Executive Chair; **EGG**, Principal Artist; **Deputy Gemini**, Commissioned Autonomous Agent).

Unlike generative diffusion/inpainting tools that hallucinate brushwork or alter surrounding composition, `heav3ns-censor` pairs **multimodal Vision-Language Model (VLM) spatial grounding** with a **deterministic Signed Distance Field (SDF) pixel engine**. It guarantees that every pixel outside the declared localized censorship masks remains **100% bit-identical** ($\text{RGBA}_{\text{pub}}(x,y) \equiv \text{RGBA}_{\text{orig}}(x,y)$) to the artist's untouched master file, while mathematically verifying that obscured regions survive `1080p` (`Q85` JPEG) and `540p` (`Q75` Thumbnail JPEG) distribution downscaling.

---

## System Architecture

```
                         ┌─────────────────────────────────────────┐
                         │       Incoming Master Illustrations     │
                         │              assets/inbox/              │
                         └────────────────────┬────────────────────┘
                                              │
                                              ▼
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                         1. CRYPTOGRAPHIC ARCHIVE & VLM GROUNDING                        │
│  • Preserves untouched master in `assets/originals/` with SHA-256 verification.         │
│  • Renders `*_inspect_grid.png` with burned-in Normalized1000 cyan/yellow ruler badges, │
│    pink interior anchors (300/500/700), & automatic high-frequency hotspot detection.   │
│  • Supports 3x nearest-neighbor zoom-crop grids (`inspect --crop`) with global coords.  │
└─────────────────────────────────────────────┬───────────────────────────────────────────┘
                                              │
                 ┌────────────────────────────┼────────────────────────────┐
                 ▼                            ▼                            ▼
┌────────────────────────────────┐ ┌──────────────────────┐ ┌─────────────────────────────┐
│  2A. LOCALIZED INTERVENTION    │ │ 2B. ARTISTIC PASS    │ │ 2C. AMBIGUITY ESCALATION    │
│  `execute` (`mosaic`,          │ │ `approve-original`   │ │ `escalate`                  │
│  `solid_bar`, `gaussian_blur`) │ │ Bit-identical export │ │ Revokes any published file; │
│  Inverse-rotated SDF (`rect`,  │ │ for classical/non-   │ │ isolates asset & logs to    │
│  `ellipse`, `pill` + `@angle`) │ │ explicit nudity.     │ │ `escalations_review.md`.    │
└────────────────┬───────────────┘ └──────────┬───────────┘ └──────────────┬──────────────┘
                 │                            │                            │
                 └────────────────────────────┼────────────────────────────┘
                                              ▼
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                   3. MULTI-SCALE VERIFICATION & BROADCAST QC TELEMETRY                  │
│  • Verifies `non_censored_pixel_diff_count == 0` (100% bit-exact exterior preservation) │
│  • Verifies Mean + P95 Peak gradient suppression across Native, 1080p Q85, & 540p Q75   │
│  • Generates 4-Panel Broadcast QC Slate (`*_contact_sheet.png`) & `dashboard.html`      │
│  • Persists deterministic BTreeMap ledger via RAII lock + atomic temp-file rename       │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## Core Mathematical & Engineering Guarantees

| Capability | Mathematical Formulation | Implementation |
| :--- | :--- | :--- |
| **Bit-Exact Exterior Preservation** | $\forall (x,y) \notin \bigcup_k \text{supp}(\alpha_k),\; I_{\text{pub}}(x,y) \equiv I_0(x,y)$ (including alpha transparency) | [`src/engine.rs`](src/engine.rs) |
| **Inverse-Rotated Analytical SDFs** | Exact signed distance for rotated `Rect`, `Ellipse`, and round-capped `Pill` capsules with dynamic enclosing AABB $(W_{\text{enc}}, H_{\text{enc}})$ | [`src/models.rs`](src/models.rs) |
| **Immutable-Source Remainder Mosaic** | Samples strictly from $I_0$; enforces Article 175 block size $b \ge \lceil \max(W,H)/100 \rceil$ and folds remainder strips ($< 0.5b$) into terminal cells | [`src/engine.rs`](src/engine.rs) |
| **Luminance-Adaptive Graphic Bars** | Supports `#000000` (*kuro-nuri*), `#FFFFFF` (*shiro-nuri*), or `auto` ($\bar{Y} < 90 \implies \text{\#FFFFFF}$, else $\text{\#000000}$) | [`src/engine.rs`](src/engine.rs) |
| **Multi-Scale Fractional Verification** | Evaluates $1\text{px}$-eroded interior Mean & $P_{95}$ Peak suppression across Native, `1080p` (`Q85`), and `540p` (`Q75`) at fractional steps $\Delta x \bmod (b \cdot s)$ | [`src/engine.rs`](src/engine.rs) |
| **Transactional Workspace Hygiene** | `WorkspaceLock` (`ledger/.ledger.lock`) + atomic temp-rename + `BTreeMap` ledger + 9-check `doctor --fix` audit | [`src/workspace.rs`](src/workspace.rs) |

---

## Documentation & Research Architecture

This repository includes a complete engineering documentation suite designed for studio handoff, academic reproducibility, and autonomous agent operation:

### 1. Foundational Tri-Pillar Documents
- **[Artistic Preservation & Post-Production Philosophy](docs/philosophy/PHILOSOPHY.md):** Zero-collateral ethics, why generative inpainting is prohibited, Japanese *Bokashi* / *Kuro-Nuri* lineage, and the Artist–Executive–Agent triad.
- **[Executive & Studio Stakeholder Handoff Guide](docs/stakeholder-handoff/STAKEHOLDER_HANDOFF.md):** Operational handoff for **MAJO** (Executive Chair), **EGG** (Principal Artist), and studio operators, including the Interactive Studio QC Dashboard guide.
- **[Technical Whitepaper](docs/whitepaper/TECHNICAL_WHITEPAPER.md):** *Deterministic Zero-Collateral Localized Image Obscuration with Multi-Scale Frequency-Domain & JPEG-Quantization Resilience Verification*.

### 2. Domain Knowledgebase (`docs/knowledgebase/`)
- **[Knowledgebase Master Index & Glossary](docs/knowledgebase/INDEX.md)**
- **[`KB-01`: Japanese Penal Code Article 175 & Cross-Platform Censorship Standards](docs/knowledgebase/01-japanese-article-175-and-platform-standards.md)**
- **[`KB-02`: Coordinate Systems & Multimodal VLM Grounding](docs/knowledgebase/02-coordinate-systems-and-vlm-grounding.md)**

### 3. Architecture Decision Records (`docs/adrs/`)
- **[`ADR-0001`: Deterministic Rust Pixel Engine Over Generative Inpainting](docs/adrs/ADR-0001-deterministic-pixel-engine-over-generative-inpainting.md)**
- **[`ADR-0002`: Immutable-Source Sampling and Inverse-Rotated SDF Masks](docs/adrs/ADR-0002-immutable-source-sampling-and-inverse-rotated-sdf-masks.md)**
- **[`ADR-0003`: Multi-Scale JPEG Resilience and P95 Peak Verification](docs/adrs/ADR-0003-multiscale-jpeg-resilience-and-p95-peak-verification.md)**
- **[`ADR-0004`: Atomic BTreeMap Ledger and Dual CLI/MCP Interface](docs/adrs/ADR-0004-atomic-btree-ledger-and-dual-cli-mcp-interface.md)**

### 4. Operational Playbooks (`docs/playbooks/`)
- **[`PLAYBOOK-01`: Autonomous Batch Ingestion & VLM Triage](docs/playbooks/PLAYBOOK-01-autonomous-batch-ingestion-and-vlm-triage.md)**
- **[`PLAYBOOK-02`: Precision Censorship & Incremental Region Editing](docs/playbooks/PLAYBOOK-02-precision-censorship-and-incremental-region-editing.md)**
- **[`PLAYBOOK-03`: Ambiguity Escalation, Revocation, & Artist Review](docs/playbooks/PLAYBOOK-03-ambiguity-escalation-revocation-and-artist-review.md)**
- **[`PLAYBOOK-04`: Workspace Doctor Audit & Incident Recovery](docs/playbooks/PLAYBOOK-04-workspace-doctor-audit-and-incident-recovery.md)**

### 5. Engineering Postmortems (`docs/postmortems/`)
- **[`PM-001`: Mosaic Remainder-Strip High-Frequency Edge Leakage](docs/postmortems/PM-001-mosaic-remainder-strip-high-frequency-edge-leakage.md)**
- **[`PM-002`: Rotated AABB Clipping & Background Gradient Dilution](docs/postmortems/PM-002-rotated-aabb-clipping-and-background-gradient-dilution.md)**
- **[`PM-003`: Downscaled 540p Fractional Mosaic Grid Misalignment](docs/postmortems/PM-003-downscaled-540p-fractional-mosaic-grid-misalignment.md)**
- **[`PM-004`: In-Place Canvas Mutation & Zero-Sample Verification False Positives](docs/postmortems/PM-004-in-place-canvas-mutation-and-zero-sample-false-positives.md)**

---

## Installation & Quick Start

### Build from Source
Requires Rust `1.75+` (`cargo`):

```bash
git clone https://github.com/CardSorting/heav3ns-censor.git
cd heav3ns-censor
cargo test --release
cargo build --release
```

### End-to-End Autonomous CLI Workflow

```bash
# 1. Generate synthetic benchmark assets in assets/inbox/ (optional demo)
./target/release/heav3ns-censor demo-assets

# 2. Batch-ingest inbox files, archive SHA-256 originals, & render VLM coordinate grids + hotspots
./target/release/heav3ns-censor batch-prepare

# 3. Generate a 3x magnified zoom-crop grid with burned-in global pixel coordinates
./target/release/heav3ns-censor inspect \
  --asset-id heav3ns_01 \
  --crop "420,480,160,140:pixels"

# 4. Execute localized censorship (Mosaic + Rotated Pill SolidBar with luminance-adaptive 'auto' color)
./target/release/heav3ns-censor execute \
  --asset-id heav3ns_01 \
  --region "r1=mosaic:440,500,120,100:pixels:rect:14:0:explicit_lower_anatomy" \
  --region "r2=solid_bar:444,314,112,52:pixels:pill:auto@6.0:0:kuro_nuri_cleft_bar"

# 5. Incrementally upsert or remove a single region (--append, --remove-region, --dry-run)
./target/release/heav3ns-censor execute \
  --asset-id heav3ns_01 \
  --append \
  --region "r2=solid_bar:444,314,112,52:pixels:pill:#000000@6.0:0:kuro_nuri_cleft_bar"

# 6. Approve artistic nudity / suggestive imagery untouched (100% bit-identical to original)
./target/release/heav3ns-censor approve-original \
  --asset-id heav3ns_02 \
  --classification artistic_nudity_pass \
  --rationale "Classical dorsal silhouette with zero explicit genital depiction."

# 7. Escalate genuine compliance or artistic ambiguity (automatically revokes any prior published file)
./target/release/heav3ns-censor escalate \
  --asset-id heav3ns_03 \
  --category "stylized_anatomy_ambiguity" \
  --reason "Abstract biomechanical groin core requires artist/executive determination."

# 8. Run cryptographic SHA-256, bit-exact preservation, & orphan-hygiene audit
./target/release/heav3ns-censor doctor --fix
```

---

## Region Shorthand Syntax Reference

The `--region` flag accepts a compact, expressive specification:

```text
[id=]method:x,y,w,h[:unit][:shape][:param[@angle_deg]][:feather][:label]
```

| Field | Values | Description |
| :--- | :--- | :--- |
| `id=` | Optional string (`r1=`, `cleft=`) | Explicit region ID for incremental `--append` upsert or `--remove-region`. |
| `method` | `mosaic`, `solid_bar`, `gaussian_blur` | Spatial obscuration algorithm. |
| `x,y,w,h` | `f32,f32,f32,f32` | Unrotated region bounding box coordinates. |
| `unit` | `pixels`, `normalized1000`, `percent` | Coordinate system (`normalized1000` matches burned-in `100..900` grid badges). |
| `shape` | `rect`, `ellipse`, `pill` | Analytical SDF shape (`pill` is a round-capped stadium capsule). |
| `param[@angle]` | e.g., `14`, `auto@6.0`, `#000000@-15`, `12.0` | Block size (`mosaic`), hex/`auto` color (`solid_bar`), or $\sigma$ (`gaussian_blur`), plus optional rotation angle in degrees. |
| `feather` | Integer pixels (`0` default) | Smoothstep edge feathering (clamped to $\le 45\%$ of minor half-axis). |
| `label` | String identifier | Human-readable audit label recorded in `treatments_ledger.json`. |

---

## Model Context Protocol (MCP) Server Integration

`heav3ns-censor` includes a native JSON-RPC 2.0 `stdio` MCP server (`./target/release/heav3ns-censor mcp`), pre-configured in [`.agents/mcp_config.json`](.agents/mcp_config.json):

| MCP Tool Name | Description |
| :--- | :--- |
| `heav3ns_status` | Returns pipeline queues, ledger paths, and `next_recommended_action`. |
| `heav3ns_doctor` | Runs the 9-point cryptographic SHA-256, preservation, and orphan-hygiene audit (`fix: bool`). |
| `heav3ns_batch_prepare` | Ingests all new `inbox/` files and renders VLM inspection grids + hotspot hints in one call. |
| `heav3ns_ingest` | Ingests a specific file or scans `assets/inbox/`. |
| `heav3ns_inspect` | Generates coordinate inspection grid and optional 3x magnified zoom-crop (`crop`). |
| `heav3ns_execute_censorship` | Executes localized censorship with `append`, `remove_region_ids`, and `dry_run` support. |
| `heav3ns_approve_original` | Approves artistic/suggestive nudity untouched with SHA-256 verification. |
| `heav3ns_escalate` | Escalates ambiguous assets to `ledger/escalations_review.md` and revokes any published file. |
| `heav3ns_reset_asset` | Resets an asset to `PendingInspection` while preserving its untouched original. |
| `heav3ns_policy_guide` | Outputs Japanese Article 175 and platform policy profiles as structured JSON. |

---

## Empirical Verification & Benchmark Suite

Run `cargo test --release` to execute all 8 automated mathematical and state-hygiene unit tests:

| Test Suite | Invariant Verified | Status |
| :--- | :--- | :---: |
| `test_mosaic_remainder_strip_pooling_and_peak_suppression` | Non-multiple mosaic ($103\times97\text{px}, b=12$) folds $1\text{px}$ remainder strip; Mean & $P_{95} > 95\%$ | **PASS** |
| `test_rotated_pill_kuro_nuri_and_angled_mosaic` | $+35^\circ$ rotated `Pill` bar & $-20^\circ$ `Ellipse` mosaic without AABB clipping; `0` exterior diff px | **PASS** |
| `test_transparent_png_alpha_preservation` | Semi-transparent character layer ($\alpha = 128$) preserves exact alpha channel inside mosaic | **PASS** |
| `test_escalation_isolation_revokes_published_file` | Transitioning `ApprovedOriginal -> Escalated` revokes and deletes `published/` artifact | **PASS** |
| `test_auto_contrast_solid_bar_luminance_adaptation` | `color="auto"` selects `#FFFFFF` (*shiro-nuri*) on dark luma and `#000000` (*kuro-nuri*) on bright luma | **PASS** |
| `test_excessive_feather_clamping_preserves_opaque_core` | Requesting `feather=200px` on a $60\times60\text{px}$ box clamps feather to $\le 45\%$ of half-axis | **PASS** |
| `test_degenerate_tiny_region_rejected_by_verifier` | Degenerate $2\times2\text{px}$ region (`0` interior eroded samples) hard-fails verification (`passed == false`) | **PASS** |
| `test_incremental_region_append_upsert_remove_and_dry_run` | Verifies `--append`, `id=` in-place upsert, `--remove-region`, and non-mutating `--dry-run` | **PASS** |

---

## Academic Citation (`BibTeX` & `CITATION.cff`)

This repository includes a machine-readable [`CITATION.cff`](CITATION.cff) file for GitHub, Zenodo, and Zotero integration. If you reference the mathematical SDF compositing, remainder-strip mosaic pooling, or fractional-step multi-scale verification architecture in academic or technical work, please cite:

```bibtex
@software{heav3ns_censor_2026,
  author       = {Deputy Gemini and EGG and MAJO},
  title        = {{heav3ns-censor: Deterministic Zero-Collateral Localized Image Obscuration with Multi-Scale Frequency-Domain and JPEG-Quantization Resilience Verification}},
  year         = {2026},
  version      = {4.0.0},
  publisher    = {HEAV3NS Post-Production Systems Research},
  url          = {https://github.com/CardSorting/heav3ns-censor}
}
```

---

## Open-Source Governance, Security, & Licensing

- **License:** Dual-licensed under **[MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE)** at your option (see [`LICENSE`](LICENSE) for full details and studio artwork copyright notice).
- **Contributing:** See [`CONTRIBUTING.md`](CONTRIBUTING.md) for architectural invariants, test requirements, and ADR guidelines.
- **Governance:** See [`GOVERNANCE.md`](GOVERNANCE.md) for the Triad Studio & Open-Source Governance model (**MAJO**, **EGG**, **Deputy Gemini**, and community contributors).
- **Security Policy:** See [`SECURITY.md`](SECURITY.md) for cryptographic `SHA-256` master verification, escalation isolation guarantees, and vulnerability reporting.
- **Code of Conduct:** Governed by the Contributor Covenant v2.1 in [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md).
- **Release History:** Tracked in [`CHANGELOG.md`](CHANGELOG.md).

