# ADR-0001: Deterministic Rust Pixel Engine Over Generative Inpainting

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** MAJO (Executive Chair), EGG (Principal Artist), Deputy Gemini
- **Technical Scope:** [`src/engine.rs`](../../src/engine.rs)

---

## Context and Problem Statement

HEAV3NS requires an autonomous post-production pipeline to transform master illustrations into publication-compliant editions under Japanese Penal Code Article 175 and global platform guidelines. Two architectural paradigms exist for automated visual censorship:
1. **Generative Diffusion / Inpainting:** Passing the artwork through a Stable Diffusion / Flux / VLM image-editing model to redraw or erase explicit anatomy.
2. **Deterministic Localized Signal Processing:** Using a multimodal VLM strictly as a *spatial sensor and policy classifier* while delegating all pixel manipulation to a deterministic, cryptographically verifiable Rust image-processing engine.

## Decision Drivers

- **Non-Negotiable Commission Mandate:** Section 2.1 of [`commission.md`](../../commission.md) mandates that *"Censorship must never come at the expense of the artwork's soul, composition, lighting, color grading, linework, or surrounding visual fidelity."*
- **Bit-Exact Exterior Invariance:** Uncensored pixels must remain 100% bit-identical (`non_censored_pixel_diff_count == 0`).
- **Auditability & Reproducibility:** Studio executives must be able to re-run identical region specifications and obtain identical `SHA-256` output hashes.

## Considered Options

1. **Generative VLM / Diffusion Inpainting**
2. **External Shell Scripting around ImageMagick / FFmpeg**
3. **Custom Deterministic Rust Studio Engine (`heav3ns-censor`)**

## Decision Outcome

Chosen option: **Option 3 — Custom Deterministic Rust Studio Engine (`heav3ns-censor`)**, paired with VLM multimodal coordinate grounding.

### Consequences

- **Positive:**
  - Guarantees `0` pixel modifications outside the declared Signed Distance Field (`SDF`) masks.
  - Preserves alpha transparency channels bit-for-bit on layered character illustrations.
  - Zero GPU/VRAM requirement; executes full censorship + multi-scale `1080p`/`540p` JPEG verification + 4-panel QC slate generation in `< 200ms` on CPU.
- **Negative / Trade-offs:**
  - Requires explicit geometric primitive modeling (`Rect`, `Ellipse`, `Pill` + rotation `@angle_deg`) in Rust rather than freehand brush masks. Mitigated by implementing exact analytical SDFs for rotated capsules and ellipses.

