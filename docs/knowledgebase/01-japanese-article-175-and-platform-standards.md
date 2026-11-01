# KB-01: Japanese Article 175 & Cross-Platform Censorship Standards

**Document ID:** `KB-01`  
**Module Reference:** [`src/policy.rs`](../../src/policy.rs), [`src/engine.rs`](../../src/engine.rs)  

---

## 1. Japanese Penal Code Article 175 (*Keihō Dai-175-jō*)

In commercial Japanese illustration, doujinshi, manga, and digital art publishing (e.g., Pixiv, DLsite, Fanbox, Melonbooks), obscenity compliance is governed by industry standards derived from **Article 175 of the Penal Code of Japan**.

### 1.1 The $\max(W, H) / 100$ Mosaic Rule
Industry self-regulatory bodies (such as ethics review boards for digital comics and games) establish that when pixelation (*mozaiku*) is used to obscure explicit genitalia:
- The mosaic grid cell size $b$ must be at least **$1/100\text{th}$ of the longest dimension** of the image canvas, with an absolute floor of **$4\text{px}$** on low-resolution canvases:
  $$b_{\text{min}} = \max\left(4,\; \left\lceil \frac{\max(W, H)}{100} \right\rceil \right)$$
- `heav3ns-censor` enforces this lower bound automatically in [`recommended_mosaic_block_size`](../../src/policy.rs#L118-L130) and adds a safety margin (`+2px` on `jp_article_175_standard`, `+4px` on `social_media_safe`) so that Lanczos downscaling and JPEG $8\times8$ DCT quantization do not reconstruct recognizable contours at thumbnail scale.

### 1.2 Target Scope: What Must Be Obscured vs. Preserved
- **Requires Intervention (`explicit_requires_intervention`):**
  - Uncovered primary male or female genitalia.
  - Internal anatomical clefts, labia/urethral detail, or explicit penetration/contact points.
  - Cross-sectional anatomical depictions in adult illustration.
- **Does Not Require Intervention (`artistic_nudity_pass` / `suggestive_pass`):**
  - Classical full-body silhouettes, dorsal/back studies, and non-explicit figure contours.
  - Stylized mannequins or non-explicit anatomical volumes without internal genital linework.

---

## 2. Built-In Policy Profiles in `heav3ns-censor`

Query all active policy profiles at any time via `./target/release/heav3ns-censor policy-guide`:

| Policy Profile ID | Primary Use Case | Mosaic Formula | Preferred Idioms |
| :--- | :--- | :--- | :--- |
| `jp_article_175_standard` | Default commercial Japanese illustration & digital art release | $\max(6, \lceil \max(W,H)/100 \rceil + 2)$ | Remainder-pooled `mosaic` for area coverage; `solid_bar` (`pill`) for linear clefts |
| `kuro_nuri_graphic` | Editorial, monochrome, high-contrast, or cyber-gothic art | $\max(6, \lceil \max(W,H)/100 \rceil + 2)$ | High-contrast `#000000` (*kuro-nuri*), `#FFFFFF` (*shiro-nuri*), or `auto` rotated `pill`/`rect` bars |
| `social_media_safe` | Strict public feed / preview distribution (`X`, `Instagram`, `Bluesky`) | $\max(8, \lceil \max(W,H)/80 \rceil + 4)$ | Coarser `mosaic`, `solid_bar`, or heavy `gaussian_blur` ($\sigma \ge 12.0$) |

