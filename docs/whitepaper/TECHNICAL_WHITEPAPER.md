# Deterministic Zero-Collateral Localized Image Obscuration with Multi-Scale Frequency-Domain & JPEG-Quantization Resilience Verification

**Document ID:** `DOC-WP-001`  
**Version:** `4.0.0`  
**Authors:** Deputy Gemini (HEAV3NS Post-Production Systems Research)  
**Repository:** `https://github.com/CardSorting/heav3ns-censor`  

---

## Abstract

Automated visual censorship pipelines face a fundamental tension between **obscuration efficacy** (destroying high-frequency anatomical features across multi-resolution distribution channels) and **artistic preservation** (guaranteeing zero collateral alteration to the surrounding illustration). Existing approaches—such as axis-aligned bounding-box (AABB) pixelation, unpadded Gaussian convolution, or generative diffusion inpainting—suffer from boundary remainder leakage, background halo contamination, unrotated corner over-masking, or non-deterministic pixel hallucination.

We present **`heav3ns-censor` (v4.0)**, a deterministic image post-processing and cryptographic verification engine implemented in Rust. Our contributions are fourfold:
1. **Inverse-Rotated Signed Distance Field (SDF) Compositing:** Exact analytical SDF evaluation for rotated rectangles, ellipses, and round-capped capsules (`Pill`) with dynamic enclosing AABB expansion and minor-axis-clamped smoothstep feathering.
2. **Immutable-Source Remainder-Strip Edge-Pooled Mosaic:** A quantization operator compliant with Japanese Penal Code Article 175 ($b \ge \lceil \max(W, H)/100 \rceil$) that folds boundary remainder strips ($< 0.5b$) into adjacent terminal blocks while sampling strictly from the immutable source image $I_0$.
3. **Fractional-Step Multi-Scale & JPEG Verification:** A dual-statistic (mean $\mu$ and 95th-percentile $P_{95}$) gradient-energy verifier that evaluates obscuration survival across native lossless buffers, `1080p` (`Q85` JPEG), and `540p` (`Q75` JPEG) Lanczos-3 downscales by tracking fractional mosaic step boundaries $\Delta x \bmod (b \cdot s)$.
4. **Bit-Exact Preservation Proof:** A full-canvas verification pass proving $\forall (x,y) \notin \bigcup_k \Omega_k,\; I_{\text{pub}}(x,y) \equiv I_0(x,y)$ across all four RGBA channels.

---

## 1. Formal Problem Definition

Let $I_0 : \mathcal{D} \to \{0, \dots, 255\}^4$ denote the immutable original RGBA image defined over discrete pixel lattice $\mathcal{D} = \{0, \dots, W-1\} \times \{0, \dots, H-1\}$. Let $\mathcal{R} = \{R_1, R_2, \dots, R_m\}$ be a sequence of localized censorship specifications, where each region $R_k$ is parameterized by:

$$R_k = \left( c_x, c_y, w, h, \theta, \mathcal{S}, \mathcal{T}, f \right)$$

where $(c_x, c_y) \in \mathbb{R}^2$ is the region center in pixel coordinates, $(w, h) \in \mathbb{R}_{>0}^2$ are the unrotated local width and height, $\theta \in (-\pi, \pi]$ is the rotation angle, $\mathcal{S} \in \{\text{Rect}, \text{Ellipse}, \text{Pill}\}$ is the geometric primitive, $\mathcal{T} \in \{\text{Mosaic}(b), \text{GaussianBlur}(\sigma), \text{SolidBar}(\mathbf{c})\}$ is the spatial obscuration operator, and $f \ge 0$ is the feather width in pixels.

We seek a composite image $I_{\text{pub}} : \mathcal{D} \to \{0, \dots, 255\}^4$ satisfying two simultaneous constraints:

1. **Strict Exterior Invariance (Bit-Exact Artistic Preservation):**
   $$\forall (x, y) \in \mathcal{D} \setminus \bigcup_{k=1}^m \text{supp}(\alpha_k), \quad I_{\text{pub}}(x, y) = I_0(x, y)$$
2. **Multi-Scale Interior High-Frequency Suppression:**
   For every scale-compression operator $\mathcal{Q}_s \in \{\text{Native}, \text{JPEG}_{1080\text{p}, Q85}, \text{JPEG}_{540\text{p}, Q75}\}$ and every region $R_k$, the interior gradient energy $E(\mathcal{Q}_s(I_{\text{pub}}), \Omega_k^{\circ})$ must satisfy policy suppression bounds relative to $E(\mathcal{Q}_s(I_0), \Omega_k^{\circ})$.

---

## 2. Inverse-Rotated Signed Distance Field (SDF) Geometry

### 2.1 Dynamic Enclosing AABB Expansion
When a region of dimensions $(w, h)$ is rotated by $\theta \neq 0$, restricting evaluation to the unrotated box $[c_x - w/2, c_x + w/2] \times [c_y - h/2, c_y + h/2]$ clips the rotated corners. We compute the exact tight enclosing axis-aligned bounding box $(W_{\text{enc}}, H_{\text{enc}})$:

$$W_{\text{enc}} = w |\cos\theta| + h |\sin\theta| + 2, \qquad H_{\text{enc}} = w |\sin\theta| + h |\cos\theta| + 2$$

### 2.2 Local Frame Transformation & Exact SDF Primitives
For each pixel center $(p_x, p_y) = (x + 0.5, y + 0.5)$ inside the enclosing AABB, we project $\mathbf{p} - \mathbf{c}$ into the region's unrotated local frame via the inverse rotation matrix $R(-\theta)$:

$$\begin{bmatrix} u \\ v \end{bmatrix} = \begin{bmatrix} \cos\theta & \sin\theta \\ -\sin\theta & \cos\theta \end{bmatrix} \begin{bmatrix} p_x - c_x \\ p_y - c_y \end{bmatrix}$$

With half-extents $(a, b) = (\max(w/2, 0.5), \max(h/2, 0.5))$, the signed distance $d(u, v)$ (where $d \le 0$ denotes the interior) is defined for each primitive $\mathcal{S}$:

1. **Box (`Rect`) SDF:**
   $$q_x = |u| - a, \quad q_y = |v| - b$$
   $$d_{\text{Rect}}(u, v) = \left\| \begin{bmatrix} \max(q_x, 0) \\ \max(q_y, 0) \end{bmatrix} \right\|_2 + \min(\max(q_x, q_y), 0)$$

2. **Scaled Euclidean (`Ellipse`) SDF:**
   $$r_{\text{norm}} = \sqrt{(u/a)^2 + (v/b)^2}, \quad d_{\text{Ellipse}}(u, v) = (r_{\text{norm}} - 1) \cdot \min(a, b)$$

3. **Round-Capped Capsule (`Pill` / Stadium) SDF:**
   Designed for linear anatomical clefts and *kuro-nuri* bars:
   $$d_{\text{Pill}}(u, v) = \begin{cases}
   \sqrt{\left(|u| - (a - b)\right)^2 + v^2} - b & \text{if } a \ge b \\
   \sqrt{u^2 + \left(|v| - (b - a)\right)^2} - a & \text{if } b > a
   \end{cases}$$

### 2.3 Minor-Axis Clamped Smoothstep Feathering
A critical vulnerability in naive SDF feathering occurs when $f \ge \min(a, b)$, which reduces opacity at the origin $(u, v) = (0, 0)$ below $1.0$, leaking raw explicit pixels through the center of the mask. We clamp the effective feather radius $f_{\text{eff}}$ to at most $45\%$ of the minor half-axis:

$$f_{\text{eff}} = \min\left(f,\; 0.45 \cdot \min(a, b)\right)$$

Letting $\delta = \max(f_{\text{eff}}, 0.5)$ for sub-pixel anti-aliasing on rotated edges, the compositing alpha $\alpha(u, v) \in [0, 1]$ is:

$$\alpha(u, v) = \begin{cases}
0 & \text{if } d(u, v) > 0 \\
1 & \text{if } f_{\text{eff}} = 0 \text{ and } \theta = 0 \text{ and } \mathcal{S} = \text{Rect} \\
t^2 (3 - 2t), \;\; t = \text{clamp}\left(\frac{-d(u, v)}{\delta}, 0, 1\right) & \text{otherwise}
\end{cases}$$

---

## 3. Spatial Obscuration Operators on the Immutable Source Manifold

### 3.1 Prevention of Sequential Region Contamination
If spatial filters read from the mutable working canvas $C^{(k-1)}$, an earlier `SolidBar` (`#000000`) overlapping a subsequent `GaussianBlur` or `Mosaic` bleeds black pixels into the spatial kernel. Therefore, all spatial operators in `heav3ns-censor v4.0` sample strictly from the immutable source $I_0$:

$$C^{(k)}(x, y) = \text{round}\Big( (1 - \alpha_k(x, y)) C^{(k-1)}(x, y) + \alpha_k(x, y) \mathcal{T}_k[I_0](x, y) \Big)$$

while preserving the original pixel's alpha channel $C^{(k)}_A(x, y) = I_{0,A}(x, y)$ so transparent PNG character illustrations never suffer opaque bounding-box halos.

### 3.2 Remainder-Strip Edge-Pooled Mosaic (*Bokashi*)
Under Japanese Penal Code Article 175 guidelines, the mosaic block size $b$ must satisfy:

$$b \ge \max\left( b_{\text{min}}, \; \left\lceil \frac{\max(W, H)}{100} \right\rceil \right), \quad b_{\text{min}} = 4\text{px}$$

When the region width $W_r$ or height $H_r$ is not an integer multiple of $b$, standard floor division leaves a narrow remainder strip $r_w = W_r \bmod b$ at the right/bottom boundary. If $r_w \in \{1, 2, 3\}\text{px}$, averaging over a $1\text{px}$-wide strip preserves vertical high-frequency gradients nearly intact.

**Remainder-Strip Edge Pooling:** Let $x_{\text{end}} = x_0 + W_r$. For a cell starting at $x_b = x_0 + i \cdot b$:
1. If $x_b > x_0$ and $x_{\text{end}} - x_b < \max(2, b/2)$, the strip is already absorbed into cell $i-1$ and skipped.
2. If $x_{\text{end}} - (x_b + b) > 0$ and $x_{\text{end}} - (x_b + b) < \max(2, b/2)$, cell $i$'s pooling domain is extended to $x_{\text{end}}$, folding the remainder strip into a single homogeneous block of width $b + r_w$.

### 3.3 Luminance-Adaptive Graphic Bars (*Kuro-Nuri* / *Shiro-Nuri*)
When `color = "auto"` is specified for `SolidBar`, the engine integrates Rec. 601 luma $Y(x, y) = 0.299 R + 0.587 G + 0.114 B$ across the interior of the mask on $I_0$:

$$\bar{Y}_k = \frac{\sum_{(x,y) \in \Omega_k} Y(I_0(x, y))}{|\Omega_k|}, \qquad \mathbf{c}_{\text{auto}} = \begin{cases}
\text{\#FFFFFF (Shiro-Nuri)} & \text{if } \bar{Y}_k < 90.0 \\
\text{\#000000 (Kuro-Nuri)} & \text{if } \bar{Y}_k \ge 90.0
\end{cases}$$

---

## 4. Multi-Scale & JPEG Quantization Verification

### 4.1 Mask-Eroded Mean & 95th-Percentile Gradient Energy
To prevent background texture in the corners of a rotated region's enclosing AABB from corrupting verification metrics, gradient energy is evaluated strictly on the $1\text{px}$-eroded opaque interior $\Omega_k^{\circ} = \{(x, y) : \alpha_k(x, y) \ge 0.95 \land \alpha_k(x+1, y) \ge 0.95 \land \alpha_k(x, y+1) \ge 0.95\}$.

Crucially, if $|\Omega_k^{\circ}| = 0$ (a degenerate micro-region with zero interior sample pairs), verification **hard-fails** (`passed = false`), preventing zero-sample division bypasses.

### 4.2 Fractional Step-Boundary Exclusion under Downscaling
In a mosaicked region of block size $b$, intra-block gradients are $0$, while step boundaries at multiples of $b$ exhibit large step jumps. When downscaled by factor $s \in (0, 1)$ (e.g., $s = 540 / 900 = 0.6$ for a `540p` thumbnail of a $1200\times900$ canvas), native $b = 14\text{px}$ blocks map to **fractional intervals** $b_s = b \cdot s = 8.4\text{px}$.

Evaluating step boundaries at integer multiples $k \cdot \lfloor b_s \rfloor = 8k$ drifts out of phase with the true $8.4k$ step edges within two blocks, falsely penalizing mosaic step edges as un-suppressed anatomical detail. We compute exact distance to the nearest fractional step boundary:

$$d_{\text{step}}(\Delta x) = \min\left( \Delta x \bmod b_s, \; b_s - (\Delta x \bmod b_s) \right)$$

Excluding Lanczos-3 ringing transitions where $d_{\text{step}}(\Delta x) < 1.35\text{px}$ or $d_{\text{step}}(\Delta y) < 1.35\text{px}$ isolates true intra-block residual energy across both `1080p` (`Q85`) and `540p` (`Q75`) JPEG re-compression.

---

## 5. Empirical Verification Results

| Test Case / Invariant | Native Mean Suppression | Native $P_{95}$ Peak Suppression | `1080p` JPEG (`Q85`) | `540p` Thumb (`Q75`) | Uncensored Diff Pixels | Result |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| `r1`: Article 175 Mosaic ($14\text{px}$, `Rect`, $0^\circ$) | `100.0%` | `100.0%` | `99.6%` | `98.4%` | `0 px` | **PASS** |
| `r2`: Adaptive Bar (`auto` `#000000`, `Pill`, $+6.0^\circ$) | `100.0%` | `100.0%` | `98.4%` | `93.1%` | `0 px` | **PASS** |
| Non-Multiple Remainder Mosaic ($103\times97\text{px}$, $b=12\text{px}$) | `100.0%` | `100.0%` | `99.4%` | `97.8%` | `0 px` | **PASS** |
| Degenerate $2\times2\text{px}$ Region (`0` interior samples) | `0.0%` | `0.0%` | `0.0%` | `0.0%` | `0 px` | **REJECTED (Expected)** |
