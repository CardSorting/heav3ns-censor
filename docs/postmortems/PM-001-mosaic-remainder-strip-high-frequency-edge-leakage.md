# PM-001: Mosaic Remainder-Strip High-Frequency Edge Leakage

- **Incident ID:** `PM-001`
- **Severity:** High (Compliance / Visual Obscuration Integrity)
- **Status:** Resolved & Verified by Unit Test (`test_mosaic_remainder_strip_pooling_and_peak_suppression`)
- **Affected Component:** [`src/engine.rs`](../../src/engine.rs) (`apply_censorship_regions`)

---

## 1. Summary

In early iterations of standard grid mosaic algorithms, when a censorship bounding box's width $W_r$ or height $H_r$ is not an exact integer multiple of the mosaic block size $b$, floor/step iteration leaves a narrow remainder strip ($r_w = W_r \bmod b$ or $r_h = H_r \bmod b$) along the right and bottom boundaries. When $r_w = 1\text{px}$ or $2\text{px}$, averaging over a $1\text{px}$-wide strip fails to suppress orthogonal high-frequency anatomical detail along the boundary, while arithmetic mean gradient suppression over the full $100\times100\text{px}$ box remains $> 96\%$, masking the defect.

---

## 2. Root-Cause Mathematical Analysis

Consider a mosaic region of dimensions $W_r = 103\text{px}, H_r = 97\text{px}$ with block size $b = 12\text{px}$:
- Along the Y-axis, $97 = 8 \times 12 + 1$.
- The 9th row of cells spans $y \in [y_0 + 96, y_0 + 97)$, which has height **$\Delta y = 1\text{px}$**.
- For a $12\times 1\text{px}$ horizontal strip, vertical gradient variance is nonexistent, and if both $r_w = 1$ and $r_h = 1$ occur at the bottom-right corner, the corner pixel is **not averaged with any neighbor at all** ($1\times 1$ block = raw original pixel!).

 Furthermore, because only $\sim 1\%$ of the pixels in the bounding box belong to the $1\text{px}$ remainder strip, a verifier that only checks mean gradient suppression ($\mu$) reports $\sim 98\%$ suppression and falsely passes the region.

---

## 3. Remediation (Root-Cause Fix, Not a Monkey-Patch)

1. **Remainder-Strip Edge Pooling ([`src/engine.rs`](../../src/engine.rs#L118-L162)):**  
   We introduced `half_block = (bs / 2).max(2)`. Before rendering a cell starting at `bx`:
   - If `x_end - bx < half_block`, the narrow strip was already absorbed into the preceding cell and is skipped.
   - If the *next* remainder `x_end - (bx + bs)` is positive and `< half_block`, the current cell's pooling and painting bound `cell_x1` is extended all the way to `x_end`.
   - Consequently, no mosaic cell can ever be narrower or shorter than $\ge 1.0b$ (or $\ge 0.5b$ if the entire region is tiny).
2. **95th-Percentile ($P_{95}$) Peak Suppression Metric ([`src/engine.rs`](../../src/engine.rs#L580-L634)):**  
   Added `peak_detail_suppression_ratio` ($P_{95}$) to `RegionVerificationMetric` and enforced `peak_suppression >= 0.80` alongside mean suppression so localized edge leaks can never hide behind region-wide averages.

