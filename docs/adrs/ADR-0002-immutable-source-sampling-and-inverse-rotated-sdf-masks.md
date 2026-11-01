# ADR-0002: Immutable-Source Sampling and Inverse-Rotated SDF Masks

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Deputy Gemini
- **Technical Scope:** [`src/engine.rs`](../../src/engine.rs), [`src/models.rs`](../../src/models.rs)

---

## Context and Problem Statement

In Japanese illustration post-production, explicit anatomical features are rarely axis-aligned rectangles. Furthermore, an artwork may require multiple adjacent or overlapping censorship regions (for example, an area `Mosaic` over lower anatomy adjacent to an angled *kuro-nuri* `SolidBar` over an anatomical cleft).

Early prototypes (`v1.0`–`v3.0`) revealed three subtle geometric and compositing hazards:
1. **Rotated Corner Clipping:** Rotating a bar inside its unrotated $(x, y, w, h)$ bounding box clipped the rotated corners.
2. **In-Place Canvas Bleed:** Sampling `Mosaic` remainder strips or `GaussianBlur` $3\sigma$ halos from the mutable working `canvas` allowed an earlier `SolidBar` (`#000000`) to bleed into an adjacent blur or mosaic block.
3. **Excessive Feather Core Leakage:** Specifying `feather_px` larger than `min(w, h) / 2` reduced the SDF opacity at the center of the region below `1.0`, leaking explicit pixels.

## Decision Outcome

1. **Dynamic Enclosing AABB Expansion (`resolve_enclosing_aabb`):**  
   Every region with $\theta \neq 0$ automatically expands its evaluation bounding box to $W_{\text{enc}} = w|\cos\theta| + h|\sin\theta| + 2$ and $H_{\text{enc}} = w|\sin\theta| + h|\cos\theta| + 2$.
2. **Universal Inverse-Rotated Analytical SDFs (`region_mask_alpha`):**  
   All three shapes (`Rect`, `Ellipse`, and round-capped `Pill` capsule) project pixel coordinates $(x+0.5, y+0.5)$ into the unrotated local frame via $R(-\theta)$ and evaluate exact analytical signed distance functions.
3. **Immutable-Source Sampling (`original` vs. `canvas`):**  
   In [`apply_censorship_regions`](../../src/engine.rs#L65-L214), all spatial neighborhoods (`Mosaic` block pooling, `GaussianBlur` $3\sigma$ halo extraction, and `SolidBar` `auto` luminance calculation) read strictly from the immutable `original` image while compositing onto `canvas`.
4. **Minor-Axis Feather Clamping:**  
   `feather_px` is clamped to $\le 0.45 \cdot \min(w/2, h/2)$ so the central core of every censorship region remains 100% opaque.

