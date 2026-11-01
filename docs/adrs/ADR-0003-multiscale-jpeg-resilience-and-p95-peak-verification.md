# ADR-0003: Multi-Scale JPEG Resilience and P95 Peak Verification

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Deputy Gemini
- **Technical Scope:** [`src/engine.rs`](../../src/engine.rs)

---

## Context and Problem Statement

Section 4.4 of [`commission.md`](../../commission.md) requires verifying that censorship treatments remain effective not only at native studio resolution, but also after **publication downscaling, thumbnail generation, and lossy JPEG compression**.

Two critical verification failure modes were uncovered during engineering audits:
1. **Mean-Only Dilution of Boundary Leakage:** If a $100\times100\text{px}$ mosaic left a $1\text{px}$ un-pooled edge strip, the mean gradient suppression across all $10,000$ pixels still scored $> 96\%$, masking the localized failure.
2. **Fractional Mosaic Step Drift under Downscaling:** When a $1200\times900$ image with a $14\text{px}$ mosaic is downscaled to `540p` ($s = 0.6$), the mosaic step boundaries occur every $14 \times 0.6 = 8.4\text{px}$. A verifier that skips integer steps ($8\text{px}$) rapidly drifts out of phase with the true $8.4\text{px}$ block boundaries, measuring the high-contrast mosaic grid steps themselves as "unsuppressed detail" (producing false-negative scores of `13%`).

## Decision Outcome

1. **Dual Mean + 95th-Percentile ($P_{95}$) Peak Suppression Telemetry:**  
   [`compute_masked_detail_metrics`](../../src/engine.rs#L580-L634) computes both arithmetic mean gradient energy and 95th-percentile ($P_{95}$) peak gradient energy strictly inside the $1\text{px}$-eroded opaque mask (`alpha >= 0.95`). Both `detail_suppression_ratio >= 0.85` and `peak_detail_suppression_ratio >= 0.80` must pass.
2. **Zero-Sample Hard Rejection:**  
   If a region is so small (e.g., `2x2` pixels) that interior erosion leaves `sample_count == 0`, verification explicitly fails (`passed = false`).
3. **Fractional Step-Boundary Alignment (`native_block_size * scale_factor`):**  
   Downscaled mosaic verification tracks exact floating-point step intervals $b_s = b \cdot s$ and excludes a $1.35\text{px}$ Lanczos-3 transition margin around multiples of $b_s$, accurately measuring intra-block detail suppression across `1080p` (`Q85` JPEG) and `540p` (`Q75` JPEG).
