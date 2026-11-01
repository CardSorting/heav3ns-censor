# PM-002: Rotated AABB Clipping & Background Gradient Dilution

- **Incident ID:** `PM-002`
- **Severity:** High (Geometry & Verification Accuracy)
- **Status:** Resolved & Verified by Unit Test (`test_rotated_pill_kuro_nuri_and_angled_mosaic`)
- **Affected Components:** [`src/models.rs`](../../src/models.rs), [`src/engine.rs`](../../src/engine.rs)

---

## 1. Summary

When applying an angled *kuro-nuri* bar (`SolidBar` with `angle_deg = 35.0`) over a diagonal anatomical line, two coupled bugs occurred in early prototypes:
1. **Corner Clipping:** Iterating pixel coordinates only over the unrotated bounding box $[x, x+w] \times [y, y+h]$ clipped the ends of the rotated bar wherever the rotated geometry extended above $y$ or below $y+h$.
2. **Background Gradient Dilution in Verification:** Computing gradient suppression across the entire enclosing rectangular bounding box included the untouched high-frequency background pixels in the four triangular corners outside the rotated bar, causing a `100%` opaque black bar to report only `44%` gradient suppression and fail verification.

---

## 2. Root-Cause Analysis

- A rectangle or capsule of dimensions $(w, h)$ rotated by $\theta$ around its center $(c_x, c_y)$ has bounding extents:
  $$W_{\text{enc}} = w|\cos\theta| + h|\sin\theta|, \qquad H_{\text{enc}} = w|\sin\theta| + h|\cos\theta|$$
  For $w = 120, h = 30, \theta = 35^\circ$, $H_{\text{enc}} = 120(0.5736) + 30(0.8192) \approx 93.4\text{px} \gg 30\text{px}$. Clipping to $h = 30\text{px}$ truncated $68\%$ of the rotated bar's vertical span.
- Conversely, once expanded to $W_{\text{enc}} \times H_{\text{enc}}$, the rotated bar occupies only $\frac{w \cdot h}{W_{\text{enc}} \cdot H_{\text{enc}}} \approx 34\%$ of the enclosing rectangle's area; the remaining $66\%$ is untouched background illustration.

---

## 3. Remediation

1. **Dynamic Enclosing AABB ([`CensorshipRegion::resolve_enclosing_aabb`](../../src/models.rs#L112-L133)):**  
   Automatically computes $(W_{\text{enc}}, H_{\text{enc}})$ from $(w, h, \theta)$ clamped to canvas bounds.
2. **Mask-Eroded Interior Verification ([`compute_masked_detail_metrics`](../../src/engine.rs#L580-L634)):**  
   Verification samples gradient pairs $(x, y), (x+1, y), (x, y+1)$ **strictly where `region_mask_alpha >= 0.95` for all three points** ($1\text{px}$ interior erosion). Untouched background corners around rotated `Pill`, `Ellipse`, or `Rect` regions are 100% excluded from obscuration telemetry.

