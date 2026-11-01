# PLAYBOOK-02: Precision Censorship & Incremental Region Editing

**Playbook ID:** `PB-02`  
**Owner:** Deputy Gemini (Autonomous Post-Production Department)  
**Trigger:** Asset classified as `explicit_requires_intervention`.  

---

## 1. Selecting the Optimal Treatment & Shape Primitive

| Target Anatomical Feature | Recommended Method | Recommended Shape | Recommended Parameter |
| :--- | :--- | :--- | :--- |
| Broad explicit anatomical region | `mosaic` | `rect` or `ellipse` | `auto` (or integer $\ge \lceil \max(W,H)/100 \rceil + 2$) |
| Narrow linear cleft or crease | `solid_bar` | `pill` (round-capped capsule) | `auto@<angle_deg>` (or `#000000@<angle_deg>` / `#FFFFFF@<angle_deg>`) |
| Soft tonal transition / atmospheric study | `gaussian_blur` | `ellipse` | `12.0` to `18.0` with feather `4`–`8` |

---

## 2. Executing Initial Censorship

Run `execute` with one or more `--region` specifications (`--asset-id` accepts exact IDs, unique prefixes like `heav3ns_01`, or filenames):

```bash
./target/release/heav3ns-censor execute \
  --asset-id heav3ns_01 \
  --region "r1=mosaic:440,500,120,100:pixels:rect:14:0:explicit_lower_anatomy" \
  --region "r2=solid_bar:444,314,112,52:pixels:pill:auto@6.0:0:kuro_nuri_cleft_bar"
```

---

## 3. Incremental Region Refinement (`--append`, `--remove-region`, `--dry-run`)

When adjusting a single region on a multi-region asset, **do not re-specify unchanged regions**. Use incremental editing flags:

1. **Test a change non-destructively (`--dry-run`):**
   ```bash
   ./target/release/heav3ns-censor execute \
     --asset-id heav3ns_01 \
     --append \
     --dry-run \
     --region "r2=solid_bar:444,314,116,54:pixels:pill:#000000@7.5:0:kuro_nuri_cleft_bar"
   ```
2. **Upsert a specific region in-place (`--append` with `id=`):**
   ```bash
   ./target/release/heav3ns-censor execute \
     --asset-id heav3ns_01 \
     --append \
     --region "r2=solid_bar:444,314,112,52:pixels:pill:auto@6.0:0:kuro_nuri_cleft_bar"
   ```
3. **Remove an unneeded region (`--remove-region`):**
   ```bash
   ./target/release/heav3ns-censor execute \
     --asset-id heav3ns_01 \
     --remove-region r3
   ```

---

## 4. Visual QC & Multi-Scale Telemetry Check

1. Confirm `verification.overall_passed == true` and `verification.non_censored_pixel_diff_count == 0` in the JSON output.
2. Inspect `assets/previews/<ASSET_ID>_contact_sheet.png` using `read_file` to visually verify all 4 panels (Original + Contours, Published Edition, 540p Thumbnail Sim, and Modification Heatmap).
