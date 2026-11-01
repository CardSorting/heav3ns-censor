# KB-02: Coordinate Systems & Multimodal VLM Grounding

**Document ID:** `KB-02`  
**Module Reference:** [`src/engine.rs`](../../src/engine.rs), [`src/models.rs`](../../src/models.rs), [`src/workspace.rs`](../../src/workspace.rs)  

---

## 1. The VLM Spatial Grounding Problem

Vision-Language Models (VLMs) excel at semantic classification ("this image contains explicit lower-body anatomy") but frequently exhibit $20\text{px}$–$80\text{px}$ coordinate drift when asked to estimate raw bounding boxes on an unmarked high-resolution canvas. In localized censorship, a $40\text{px}$ drift either leaves explicit anatomy uncovered or unnecessarily masks adjacent artistic linework.

`heav3ns-censor` solves VLM spatial grounding through a **three-stage coarse-to-fine visual scaffolding architecture**.

---

## 2. Three Supported Coordinate Units

Every region (`--region`) and zoom-crop (`--crop`) accepts three interchangeable coordinate systems (`x, y, width, height`):

1. **`pixels` (`CoordinateUnit::Pixels`):**  
   Exact integer/floating-point pixel coordinates in $[0, W] \times [0, H]$.
2. **`normalized1000` (`CoordinateUnit::Normalized1000`):**  
   Resolution-independent coordinates in $[0, 1000] \times [0, 1000]$, matching standard VLM bounding-box tokenization and matching the burned-in `100..900` grid labels on `*_inspect_grid.png`.
3. **`percent` (`CoordinateUnit::Percent`):**  
   Percentage coordinates in $[0.0, 100.0] \times [0.0, 100.0]$.

---

## 3. Burned-In Bitmap Inspection Grids (`*_inspect_grid.png`)

When `heav3ns-censor inspect` or `batch-prepare` runs on an asset, [`generate_inspection_grid`](../../src/engine.rs#L649-L735) renders a non-destructive copy in `assets/inspect/<ASSET_ID>_inspect_grid.png` containing:
- **10% Major Grid Lines (`100, 200, ..., 900` in `Normalized1000`):** High-contrast cyan lines with a dark halo for visibility over both white and black backgrounds.
- **5% Minor Dashed Sub-Grid Lines (`50, 150, ..., 950`):** Subtle yellow dashed lines for $2.5\%$–$5\%$ interpolation.
- **Edge & Interior Bitmap Ruler Badges:**
  - Top & Bottom edges: `X100`, `X200`, ..., `X900` cyan badges.
  - Left edge: `Y100`, `Y200`, ..., `Y900` yellow badges.
  - Interior canvas intersections: `(300,300)`, `(500,500)`, `(700,700)` pink anchor badges so the VLM never has to trace across a wide canvas to read an axis label.

---

## 4. Coarse-to-Fine 3x Magnified Zoom-Crop (`inspect --crop`)

For small or angled anatomical details (such as a narrow cleft requiring a rotated `pill` bar), the agent executes:

```bash
./target/release/heav3ns-censor inspect \
  --asset-id heav3ns_01 \
  --crop "400,280,200,120:pixels"
```

[`generate_zoom_crop_grid`](../../src/engine.rs#L740-L815) extracts the sub-region, magnifies it **$3\times$ via nearest-neighbor scaling** (preserving exact pixel boundaries), overlays a fine $50\text{px}$ sub-grid, and burns the **global original-image pixel coordinates** (`X400`, `Y300`, etc.) directly onto the magnified crop. The agent can then read exact pixel coordinates from the magnified image without mental coordinate math.
