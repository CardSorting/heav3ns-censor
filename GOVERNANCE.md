# Project Governance

**Project:** `heav3ns-censor`  
**Charter:** [`commission.md`](commission.md)  

---

## 1. Governance Structure

`heav3ns-censor` operates under a **Triad Studio & Open-Source Governance Model** established by the HEAV3NS Executive Commission:

| Role | Title / Identifier | Responsibilities & Authority |
| :--- | :--- | :--- |
| **Commissioning Authority** | **MAJO** (Executive Chair) | Final authority on publication compliance standards, platform policy profiles, and escalated asset sign-off (`ledger/escalations_review.md`). |
| **Principal Artist** | **EGG** (Principal Artist) | Final authority on artistic preservation, composition integrity, and visual aesthetics of censorship idioms (*Bokashi* vs. *Kuro-Nuri* / *Shiro-Nuri*). |
| **Autonomous Maintainer** | **Deputy Gemini** | Day-to-day autonomous operation of the post-production pipeline, Rust engine maintenance, mathematical verification hardening, and ADR/Postmortem authorship. |
| **Open-Source Contributors** | Community & Researchers | Submitting bug reports, mathematical optimizations, new platform policy profiles, and documentation enhancements under [`CONTRIBUTING.md`](CONTRIBUTING.md). |

---

## 2. Decision-Making Process

1. **Core Preservation Invariants (`src/engine.rs`):**  
   Any change affecting pixel preservation (`non_censored_pixel_diff_count == 0`), alpha channel fidelity, or master `SHA-256` verification requires an Architecture Decision Record in [`docs/adrs/`](docs/adrs/) and 100% passing unit tests + `doctor --fix` verification.
2. **Policy Profiles (`src/policy.rs`):**  
   Additions or adjustments to obscenity/platform policy profiles (`jp_article_175_standard`, `kuro_nuri_graphic`, `social_media_safe`) are governed by **MAJO** (Executive Chair) and documented in [`KB-01`](docs/knowledgebase/01-japanese-article-175-and-platform-standards.md).
3. **Artistic Ambiguity Escalations:**  
   In production studio workflows, any asset flagged as `AmbiguousEscalate` is isolated from `assets/published/` until reviewed by **MAJO** and/or **EGG**.
