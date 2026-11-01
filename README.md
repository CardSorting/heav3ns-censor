# HEAV3NS-CENSOR (v4.0)
**Autonomous Japanese-Style Censorship & Publication Post-Processing Studio Engine**

- **Commissioning Authority:** MAJO, Executive Chair (`commission.md`)
- **Commissioned Agent:** Deputy Gemini (`AGENTS.md`)
- **Principal Artist:** EGG

---

## Architecture & Core Guarantees

`heav3ns-censor` is a deterministic, zero-collateral-damage post-production engine built in Rust for autonomous operation via CLI or Model Context Protocol (`MCP` stdio server).

1. **Bit-Exact Artistic Preservation (`non_censored_pixel_diff_count == 0`):**
   - Originals are archived untouched in `assets/originals/` with cryptographic `SHA-256` verification.
   - All spatial censorship filters (`Mosaic`, `GaussianBlur`, `SolidBar`) sample strictly from the immutable original canvas and composite through an exact inverse-rotated Signed Distance Field (`SDF`) alpha mask (`Rect`, `Ellipse`, or round-capped `Pill` capsule).
   - Every pixel outside the declared censorship regions is verified to remain **100% bit-identical** (RGBA) to the original artwork, including full alpha-channel transparency preservation.
2. **Japanese Penal Code Article 175 & Platform Compliance:**
   - **Remainder-Strip Edge-Pooled Mosaic (`mosaic`):** Enforces Article 175 block sizing ($\ge \max(W, H) / 100$, minimum `4px`) and folds boundary remainder strips (`< 50%` of block size) into adjacent full cells to eliminate 1–3px high-frequency edge leakage.
   - **Graphic Bar Masking (`solid_bar`):** Supports *kuro-nuri* (`#000000`), *shiro-nuri* (`#FFFFFF`), and luminance-adaptive `auto` bar color selection, with `rect`, `ellipse`, or round-capped `pill` capsule geometry and arbitrary `@<angle_deg>` rotation.
   - **Halo-Padded Separable Gaussian Blur (`gaussian_blur`):** Samples a $3\sigma$ padded neighborhood from the immutable original image to eliminate dark border bleed, with feather clamping (`<= 45%` of the minor half-axis) so the central core never becomes semi-transparent.
3. **Multi-Scale & JPEG Compression Resilience Verification:**
   - Every published edition is automatically verified across **Native Resolution** (both Mean and 95th-Percentile Peak gradient suppression), **1080p Social JPEG (`Q85`)**, and **540p Thumbnail JPEG (`Q75`)**, rejecting degenerate zero-sample regions.
   - Generates a **4-Panel Broadcast QC Slate** (`assets/previews/<ASSET_ID>_contact_sheet.png`) and an interactive **Studio QC Application** (`ledger/dashboard.html`) with live mouse-drag coordinate/bounding-box measurement and one-click CLI command generation.

---

## Quick Start & Autonomous CLI Commands

```bash
# 1. Batch-ingest assets/inbox/ and generate VLM coordinate grids + hotspot hints
./target/release/heav3ns-censor batch-prepare

# 2. Generate a 3x magnified zoom-crop grid with burned-in global coordinates
./target/release/heav3ns-censor inspect --asset-id heav3ns_01 --crop "420,480,160,140:pixels"

# 3. Execute localized censorship (supports [id=]method:..., @angle_deg, auto bar color, --append, --remove-region, --dry-run)
./target/release/heav3ns-censor execute \
  --asset-id heav3ns_01 \
  --region "r1=mosaic:440,500,120,100:pixels:rect:14:0:explicit_anatomy" \
  --region "r2=solid_bar:444,314,112,52:pixels:pill:auto@6.0:0:kuro_nuri_cleft"

# 4. Incrementally append or update a single region without re-specifying existing regions
./target/release/heav3ns-censor execute \
  --asset-id heav3ns_01 \
  --append \
  --region "r2=solid_bar:444,314,112,52:pixels:pill:#000000@6.0:0:kuro_nuri_cleft"

# 5. Approve artistic nudity / suggestive imagery untouched
./target/release/heav3ns-censor approve-original \
  --asset-id heav3ns_02 \
  --classification artistic_nudity_pass \
  --rationale "Classical artistic silhouette with no explicit genital detail."

# 6. Escalate genuine compliance or artistic ambiguity (automatically revokes any prior published file)
./target/release/heav3ns-censor escalate \
  --asset-id heav3ns_03 \
  --category "stylized_anatomy_ambiguity" \
  --reason "Abstract biomechanical groin core requires artist/executive determination."

# 7. Run cryptographic SHA-256, bit-exact preservation, and orphan-hygiene audit
./target/release/heav3ns-censor doctor --fix
```

