# PM-004: In-Place Canvas Mutation & Zero-Sample Verification False Positives

- **Incident ID:** `PM-004`
- **Severity:** High (Correctness & Verification Hardening)
- **Status:** Resolved & Verified by Unit Tests (`test_degenerate_tiny_region_rejected_by_verifier`, `test_excessive_feather_clamping_preserves_opaque_core`)
- **Affected Component:** [`src/engine.rs`](../../src/engine.rs)

---

## 1. Summary

During the `v4.0` deep security and mathematical audit of `src/engine.rs`, two subtle edge-case vulnerabilities were discovered:
1. **In-Place Canvas Mutation Contamination:** In `apply_censorship_regions`, `Mosaic` cell pooling and `GaussianBlur` $3\sigma$ halo extraction read pixel values from the mutable working `canvas` instead of the immutable `original` buffer.
2. **Zero-Sample Verification Bypass:** In `verify_censored_edition`, if a region had `0` valid interior sample pairs after $1\text{px}$ mask erosion (e.g., a degenerate $2\times2\text{px}$ region or a region whose `feather_px` exceeded its minor half-axis), `compute_masked_detail_metrics` returned `orig_mean = 0.0`, which the verifier treated as a flat color region (`suppression = 1.0 -> PASS`).

---

## 2. Root-Cause Analysis

- **In-Place Read After Write:**  
  When remainder-strip pooling extended a mosaic cell's sampling window, or when a `GaussianBlur` region overlapped an earlier `#000000` `SolidBar` region, reading from `canvas` meant the spatial kernel ingested black bar pixels or already-quantized mosaic pixels rather than the true underlying artwork.
- **Vacuous Truth on Empty Sample Set:**  
  When `sample_count == 0`, `orig_mean` defaulted to `0.0`. The branch `if orig_mean <= 0.01 { 1.0 }` intended for genuinely flat-colored artwork regions inadvertently triggered on empty sample sets.

---

## 3. Remediation

1. **Immutable Source Sampling ([`src/engine.rs`](../../src/engine.rs#L65-L214)):**  
   All spatial kernels (`Mosaic`, `GaussianBlur`, and `resolve_bar_color`) now read strictly from `original: &RgbaImage` while writing to `canvas: RgbaImage`.
2. **Feather Radius Clamping ([`src/engine.rs`](../../src/engine.rs#L52-L55)):**  
   Clamped `feather_px` to `(half_w.min(half_h) * 0.45).max(0.0)` so excessive feather parameters can never make the center of a censorship region semi-transparent.
3. **Explicit `sample_count > 0` Gate ([`src/engine.rs`](../../src/engine.rs#L406-L468)):**  
   `compute_masked_detail_metrics` now returns `(mean, p95, sample_count)` and `verify_censored_edition` enforces `native_samples > 0` for `passed = true`.

