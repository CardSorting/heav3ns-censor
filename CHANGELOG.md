# Changelog

All notable changes to **`heav3ns-censor`** are documented in this file.  
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [4.0.0] - 2026-11-01

### Added
- **Luminance-Adaptive Graphic Bars (`auto`):** `SolidBar` now supports `color = "auto"` (or `"auto_contrast"`), measuring mean Rec. 601 luminance inside the target mask on the immutable original image and automatically selecting `#FFFFFF` (*shiro-nuri*) when $\bar{Y} < 90.0$ or `#000000` (*kuro-nuri*) when $\bar{Y} \ge 90.0$.
- **Incremental Region Editing (`--append`, `--remove-region`, `--dry-run`):** Both CLI (`execute`) and MCP (`heav3ns_execute_censorship`) now support `[id=]method:...` shorthand, in-place region upsert (`--append`), selective region removal (`--remove-region <ID>`), and non-mutating simulation (`--dry-run`).
- **Interactive Coordinate & Bounding-Box Inspector in `ledger/dashboard.html`:** Hover or drag a selection box on any asset tab (including 2×2 quadrant mapping on the **4-Panel Broadcast QC Slate**) to view live `Pixels (X, Y, W, H)` and `Normalized1000 (X, Y, W, H)` coordinates and generate a one-click copyable `--region` CLI command. Added keyboard shortcuts (`/` to search, `1`–`6` to switch view tabs).
- **Complete Research & Open-Source Documentation Suite:** Added [`docs/philosophy/PHILOSOPHY.md`](docs/philosophy/PHILOSOPHY.md), [`docs/stakeholder-handoff/STAKEHOLDER_HANDOFF.md`](docs/stakeholder-handoff/STAKEHOLDER_HANDOFF.md), [`docs/whitepaper/TECHNICAL_WHITEPAPER.md`](docs/whitepaper/TECHNICAL_WHITEPAPER.md), [`docs/knowledgebase/`](docs/knowledgebase/INDEX.md), [`docs/adrs/`](docs/adrs/), [`docs/playbooks/`](docs/playbooks/), [`docs/postmortems/`](docs/postmortems/), `LICENSE`, `CITATION.cff`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, `GOVERNANCE.md`, and GitHub Actions CI.

### Fixed
- **Immutable-Source Spatial Sampling ([`PM-004`](docs/postmortems/PM-004-in-place-canvas-mutation-and-zero-sample-false-positives.md)):** Fixed in-place canvas read hazard where `Mosaic` remainder-strip pooling or `GaussianBlur` halo extraction read from the mutable `canvas` instead of the immutable `original` buffer.
- **Minor-Axis Feather Clamping & Zero-Sample Guard ([`PM-004`](docs/postmortems/PM-004-in-place-canvas-mutation-and-zero-sample-false-positives.md)):** Clamped `feather_px` to $\le 45\%$ of the minor half-axis (`half_w.min(half_h) * 0.45`) and enforced `sample_count > 0` in `verify_censored_edition` so degenerate tiny regions (`2x2` px) cannot falsely pass verification.
- **Deterministic `BTreeMap` Ledger ([`ADR-0004`](docs/adrs/ADR-0004-atomic-btree-ledger-and-dual-cli-mcp-interface.md)):** Switched `Ledger.assets` from `HashMap` to `BTreeMap<String, AssetRecord>` for deterministic lexicographical key ordering.
- **Explicit JPEG Quality Encoding:** Upgraded downscale simulators (`1080p` and `540p`) to use `JpegEncoder::new_with_quality` (`Q85` and `Q75`).

---

## [3.0.0] - 2026-11-01

### Added
- **Round-Capped Capsule (`RegionShape::Pill`) & Universal `@<angle_deg>` Rotation:** Analytical capsule SDF and universal rotation across `Mosaic`, `GaussianBlur`, and `SolidBar`.
- **Coarse-to-Fine 3x Magnified Zoom-Crop (`inspect --crop`):** Nearest-neighbor 3x magnification with burned-in global pixel coordinate badges and automatic high-frequency hotspot detection (`detect_candidate_hotspots`).
- **Workspace Lock & Atomic Persistence:** Added `WorkspaceLock` (`ledger/.ledger.lock`) and `atomic_write_bytes`.

### Fixed
- **Remainder-Strip Mosaic Edge Pooling ([`PM-001`](docs/postmortems/PM-001-mosaic-remainder-strip-high-frequency-edge-leakage.md)):** Folded narrow right/bottom remainder strips ($< 50\%$ of block size) into adjacent full mosaic cells.
- **Fractional Mosaic Step Tracking at `540p` ([`PM-003`](docs/postmortems/PM-003-downscaled-540p-fractional-mosaic-grid-misalignment.md)):** Fixed false-negative `540p` suppression scores by tracking floating-point step intervals $b \cdot s$.

---

## [2.0.0] - 2026-11-01

### Added
- **Dynamic Enclosing AABB & Mask-Eroded Verification ([`PM-002`](docs/postmortems/PM-002-rotated-aabb-clipping-and-background-gradient-dilution.md)):** Prevented rotated corner clipping and excluded untouched background corners from gradient verification.
- **4-Panel Broadcast QC Slate & `doctor --fix` Command:** Automated visual contact sheets and workspace hygiene auditing.

---

## [1.0.0] - 2026-11-01

### Added
- Initial Rust studio engine (`heav3ns-censor`) with `ingest`, `inspect`, `execute`, `approve-original`, `escalate`, and JSON-RPC 2.0 `mcp` stdio server fulfilling [`commission.md`](commission.md).

