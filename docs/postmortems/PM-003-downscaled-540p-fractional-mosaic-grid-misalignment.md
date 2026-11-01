# PM-003: Downscaled 540p Fractional Mosaic Grid Misalignment

- **Incident ID:** `PM-003`
- **Severity:** Medium-High (False-Negative Multi-Scale Verification)
- **Status:** Resolved & Verified in `v4.0`
- **Affected Component:** [`src/engine.rs`](../../src/engine.rs) (`verify_censored_edition`, `compute_masked_detail_metrics`)

---

## 1. Summary

When verifying `heav3ns_01_explicit_study` ($1200\times900$) with a $14\text{px}$ Article 175 mosaic (`r1`), native resolution suppression scored `100.0%` and `1080p` scored `99.6%`, but initial `540p` thumbnail verification (`720x540`, scale factor $s = 0.6$) reported only `13.0%` suppression despite visual inspection showing complete obscuration.

---

## 2. Root-Cause Mathematical Analysis

1. **Integer Step Drift:**  
   At native resolution ($s = 1.0$), $14\text{px}$ mosaic blocks have step edges at $\Delta x \in \{14, 28, 42, 56, 70, 84, \dots\}$.
   When downscaled by $s = 540 / 900 = 0.6$, the true step boundaries in the `540p` image lie at **fractional pixel coordinates**:
   $$k \cdot (14 \times 0.6) = k \cdot 8.4\text{px} \implies \{8.4,\; 16.8,\; 25.2,\; 33.6,\; 42.0,\; 50.4,\dots\}$$
   The initial verifier rounded the scaled block size to an integer `scaled_bs = (14 * 0.6).round() = 8`, skipping $\Delta x \bmod 8 == 0$ ($\{8, 16, 24, 32, 40\}$). By the 3rd block ($25.2$ vs $24$), the integer filter missed every single mosaic block boundary, measuring the high-contrast mosaic block steps as "unsuppressed anatomical detail."
2. **Lanczos-3 Reference Shrinkage:**  
   Downscaling the original high-frequency checkerboard anatomy by $0.6\times$ via Lanczos-3 low-pass filtering attenuated the *original* reference gradient `o_mean` at `540p`, artificially inflating the ratio `c_mean / o_mean`.

---

## 3. Remediation

1. **Floating-Point Fractional Step Tracking ([`src/engine.rs`](../../src/engine.rs#L604-L622)):**  
   Replaced integer modulo `(dx % scaled_bs)` with continuous floating-point step distance:
   ```rust
   let bs_f = (nb as f32) * scale_factor;
   let rem_x = dx % bs_f;
   let dist_x = rem_x.min(bs_f - rem_x);
   ```
   Excluding a $1.35\text{px}$ Lanczos transition margin around fractional multiples of $b \cdot s$ restored true intra-block `540p` suppression measurement (`98.4%`).
2. **Baseline Reference Floor:**  
   Anchored downscaled suppression denominator against `o_mean.max(orig_native_mean * 0.5)` so low-pass filtering of the reference thumbnail never penalizes an opaque or flat-pooled censorship region.
