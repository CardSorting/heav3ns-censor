# HEAV3NS-CENSOR: Artistic Preservation & Post-Production Philosophy

**Document ID:** `DOC-PHIL-001`  
**Engine Version:** `heav3ns-censor v4.0.0`  
**Commissioning Authority:** MAJO, Executive Chair  
**Principal Artist:** EGG  
**Commissioned Agent:** Deputy Gemini  

---

## 1. First Principle: Artistic Preservation is Absolute

Modern automated moderation systems frequently treat visual artwork as disposable pixel buffers—cropping compositions, applying global blurs, or invoking generative diffusion inpainting to "rewrite" anatomy. Within the **HEAV3NS** studio, such approaches are strictly prohibited.

> **The Zero-Collateral Mandate:**  
> Censorship is a localized publication post-processing step, never a re-authoring of the artist's work. Every single pixel outside the minimum necessary censorship mask must remain **100% bit-identical** ($\text{RGBA}_{\text{published}}(x,y) \equiv \text{RGBA}_{\text{original}}(x,y)$) to the Principal Artist's original master file.

### Why Generative Inpainting is Forbidden
Generative inpainting (diffusion-based erasure or redraw) introduces three unacceptable failure modes in fine-art and illustration publishing:
1. **Hallucinated Brushwork & Palette Drift:** Diffusion decoders re-synthesize surrounding pixels in latent space, subtly corrupting line weight, halftone grain, and color grading across the bounding box.
2. **Erasure of Authorial Intent:** Re-drawing clothing, skin, or shadow where anatomy existed falsifies the artist's composition rather than honestly signaling a publication compliance mask.
3. **Non-Determinism:** Stochastic sampling prevents reproducible cryptographic auditing between master archives and published editions.

By contrast, **Japanese-style localized post-processing** (*Bokashi* mosaic and *Kuro-Nuri* / *Shiro-Nuri* graphic bars) is visually honest: the viewer immediately understands where the artist's original composition ends and where the publication compliance intervention begins.

---

## 2. Aesthetic Lineage: Japanese Publication Censorship (*Bokashi* & *Kuro-Nuri*)

Under Japanese Penal Code Article 175 (*Keihō Dai-175-jō*) and commercial manga/illustration publishing traditions, censorship is executed through three distinct visual idioms, each supported natively by `heav3ns-censor`:

| Idiom | Japanese Term | Aesthetic & Editorial Role | Mathematical Realization in `heav3ns-censor` |
| :--- | :--- | :--- | :--- |
| **Pixel Mosaic** | *Bokashi* / *Mozaiku* (モザイク) | Preserves local color harmony, lighting temperature, and silhouette continuity while destroying high-frequency anatomical detail. | Remainder-strip edge-pooled cell averaging ($\text{block\_size} \ge \max(W, H) / 100$, min $4\text{px}$) sampled strictly from the immutable original canvas. |
| **Graphic Solid Bar** | *Kuro-Nuri* (黒塗り) / *Shiro-Nuri* (白塗り) | Deliberate, high-contrast editorial typography/graphic design statement; ideal for narrow anatomical clefts or monochrome ink illustrations. | Inverse-rotated Signed Distance Field (`Rect`, `Ellipse`, or round-capped `Pill` capsule) with `#000000`, `#FFFFFF`, or luminance-adaptive `auto` contrast. |
| **Gaussian Soft Obscuration** | *Gausu Bokashi* (ガウスぼかし) | Smooth frequency attenuation for atmospheric or tonal transitions where hard block edges would clash with soft shading. | Separable 2D Gaussian convolution over a $3\sigma$-padded immutable source halo with clamped interior feathering ($\le 45\%$ of minor half-axis). |

---

## 3. Contextual Triage: Nudity is Not Automatically Obscenity

A core philosophical requirement of the [Executive Commission](../../commission.md) is **discernment**. Automated systems that blindly censor every skin-tone pixel destroy classical and contemporary figure art. `heav3ns-censor` codifies a four-tier taxonomy:

1. **`artistic_nudity_pass` (Approve Untouched):**  
   Classical silhouettes, figure studies, non-sexualized anatomical forms, and compositions devoid of explicit genital detail. These works are exported to `assets/published/` bit-identically (`SHA-256` match with `assets/originals/`).
2. **`suggestive_pass` (Approve Untouched):**  
   Provocative framing, partial coverage, or sensual atmosphere that remains compliant with publication standards without explicit anatomical exposure.
3. **`explicit_requires_intervention` (Localized Execution):**  
   Explicit primary genitalia, internal anatomical lines/clefts, or explicit sexual acts. Receives the **smallest effective localized intervention** that survives both native viewing and `1080p` / `540p` JPEG distribution downscaling.
4. **`ambiguous_escalate` (Human-in-the-Loop Escalation):**  
   Boundary cases—such as abstract biomechanical forms, heavy chiaroscuro shadow, or compositions where any censorship bar would sever the focal point. Rather than guessing, the autonomous agent **halts publication, revokes any prior published artifact, and escalates** to `ledger/escalations_review.md`.

---

## 4. The Studio Triad: Artist, Executive, and Autonomous Agent

The HEAV3NS studio workflow separates creative authorship, executive governance, and technical execution:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        EGG (Principal Artist)                           │
│    Focuses 100% on creative authorship, linework, color, & composition.  │
│   Zero burden of manual mosaic sizing, JPEG testing, or ledger keeping. │
└───────────────────────────────────┬─────────────────────────────────────┘
                                    │ Master Artwork (assets/inbox/)
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                   DEPUTY GEMINI (Autonomous Agent)                      │
│   Operates `heav3ns-censor v4.0` via CLI / MCP.                         │
│   1. Archives untouched original with SHA-256 verification.             │
│   2. Performs coarse-to-fine VLM inspection & 3x zoom-crop grounding.   │
│   3. Applies minimal localized censorship or approves untouched.        │
│   4. Verifies bit-exact preservation & 1080p/540p JPEG resilience.      │
└─────────────────┬───────────────────────────────────┬───────────────────┘
                  │ Verified Edition                  │ Genuine Ambiguity
                  ▼                                   ▼
┌───────────────────────────────────┐ ┌───────────────────────────────────┐
│   PUBLICATION DISTRIBUTION        │ │      MAJO (Executive Chair)       │
│   assets/published/ + QC Slate    │ │   Reviews escalated edge cases    │
│   ledger/dashboard.html           │ │   in ledger/escalations_review.md │
└───────────────────────────────────┘ └───────────────────────────────────┘
```

By automating the deterministic mathematics, multi-scale verification, and cryptographic bookkeeping, **Deputy Gemini** protects **EGG**'s creative velocity while giving **MAJO** complete auditability across every published asset.

