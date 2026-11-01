[🏠 Home (`README.md`)](../../README.md) &rsaquo; [📚 Docs Portal (`docs/`)](../README.md) &rsaquo; **📐 Architecture Decision Records (`ADRs`)**

---

# Architecture Decision Records (`docs/adrs/`)

This directory records the core architectural and mathematical design decisions governing `heav3ns-censor`.

| ADR ID | Title | Status | Primary Modules |
| :--- | :--- | :---: | :--- |
| **[`ADR-0001`](ADR-0001-deterministic-pixel-engine-over-generative-inpainting.md)** | **Deterministic Rust Pixel Engine Over Generative Inpainting** | Accepted | [`src/engine.rs`](../../src/engine.rs) |
| **[`ADR-0002`](ADR-0002-immutable-source-sampling-and-inverse-rotated-sdf-masks.md)** | **Immutable-Source Sampling and Inverse-Rotated SDF Masks** | Accepted | [`src/engine.rs`](../../src/engine.rs), [`src/models.rs`](../../src/models.rs) |
| **[`ADR-0003`](ADR-0003-multiscale-jpeg-resilience-and-p95-peak-verification.md)** | **Multi-Scale JPEG Resilience and P95 Peak Verification** | Accepted | [`src/engine.rs`](../../src/engine.rs) |
| **[`ADR-0004`](ADR-0004-atomic-btree-ledger-and-dual-cli-mcp-interface.md)** | **Atomic BTreeMap Ledger and Dual CLI/MCP Interface** | Accepted | [`src/workspace.rs`](../../src/workspace.rs), [`src/mcp.rs`](../../src/mcp.rs) |

---

**Related:** [📄 Technical Whitepaper (`TECHNICAL_WHITEPAPER.md`)](../whitepaper/TECHNICAL_WHITEPAPER.md) • [🔬 Engineering Postmortems (`docs/postmortems/`)](../postmortems/README.md)

