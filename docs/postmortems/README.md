[🏠 Home (`README.md`)](../../README.md) &rsaquo; [📚 Docs Portal (`docs/`)](../README.md) &rsaquo; **🔬 Engineering Postmortems**

---

# Engineering Postmortems (`docs/postmortems/`)

Root-cause mathematical analyses and regression-tested remediations for subtle geometric, compositing, and multi-scale frequency-domain hazards uncovered during the development of `heav3ns-censor`.

| Incident ID | Title | Root Cause | Regression Unit Test |
| :--- | :--- | :--- | :--- |
| **[`PM-001`](PM-001-mosaic-remainder-strip-high-frequency-edge-leakage.md)** | **Mosaic Remainder-Strip High-Frequency Edge Leakage** | Non-multiple region widths ($W_r \bmod b \in \{1..3\}$) left un-pooled boundary strips | `test_mosaic_remainder_strip_pooling_and_peak_suppression` |
| **[`PM-002`](PM-002-rotated-aabb-clipping-and-background-gradient-dilution.md)** | **Rotated AABB Clipping & Background Gradient Dilution** | Unexpanded AABB clipped rotated bars; unmasked AABB diluted gradient verification | `test_rotated_pill_kuro_nuri_and_angled_mosaic` |
| **[`PM-003`](PM-003-downscaled-540p-fractional-mosaic-grid-misalignment.md)** | **Downscaled 540p Fractional Mosaic Grid Misalignment** | $0.6\times$ downscale mapped $14\text{px}$ blocks to fractional $8.4\text{px}$ steps, drifting from integer $8\text{px}$ | Verified in `verify_censored_edition` (`98.4%` at `540p`) |
| **[`PM-004`](PM-004-in-place-canvas-mutation-and-zero-sample-false-positives.md)** | **In-Place Canvas Mutation & Zero-Sample False Positives** | Spatial kernels read from mutable `canvas`; `0`-sample micro-regions vacuously passed | `test_degenerate_tiny_region_rejected_by_verifier`, `test_excessive_feather_clamping_preserves_opaque_core` |

---

**Related:** [📄 Technical Whitepaper (`TECHNICAL_WHITEPAPER.md`)](../whitepaper/TECHNICAL_WHITEPAPER.md) • [📐 Architecture Decision Records (`docs/adrs/`)](../adrs/README.md)
