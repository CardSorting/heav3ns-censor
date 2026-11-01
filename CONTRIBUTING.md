# Contributing to `heav3ns-censor`

Thank you for your interest in contributing to **`heav3ns-censor`**, the open-source deterministic Japanese-style localized image obscuration and multi-scale verification engine.

Whether you are a computer-vision researcher, a Rust systems engineer, a digital preservation specialist, or an autonomous AI agent, please review the architectural invariants below before opening an issue or pull request.

---

## 1. Non-Negotiable Architectural Invariants

Every pull request that touches [`src/engine.rs`](src/engine.rs), [`src/models.rs`](src/models.rs), or [`src/workspace.rs`](src/workspace.rs) **must preserve** the following invariants:

1. **Bit-Exact Exterior Preservation (`non_censored_pixel_diff_count == 0`):**  
   Pixels outside the declared localized Signed Distance Field (`SDF`) masks (`region_mask_alpha(x, y) == 0.0`) must never be modified—not even by $\pm 1$ LSB in RGB or Alpha.
2. **Immutable-Source Spatial Sampling:**  
   All spatial filters (`Mosaic`, `GaussianBlur`, and `SolidBar` auto-luminance measurement) must read strictly from the immutable `original: &RgbaImage` buffer while writing to `canvas: RgbaImage`, preventing sequential region contamination.
3. **Zero Generative Hallucination:**  
   Do not introduce stochastic/diffusion inpainting or global canvas resizing/re-coloring as a substitute for localized deterministic censorship.
4. **Escalation Safety Isolation:**  
   Transitioning an asset to `AssetStatus::Escalated` must immediately revoke and delete any previously published edition in `assets/published/`.
5. **Deterministic Serialization & Atomic Persistence:**  
   All ledger structures must use `BTreeMap` (not `HashMap`) for deterministic ordering, and all disk writes must use `atomic_write_bytes` under `WorkspaceLock`.

---

## 2. Development & Verification Workflow

### Prerequisites
- Rust `1.75+` (`rustup` stable toolchain)

### Running the Test & Audit Suite
Before submitting a pull request, run the full release test suite and workspace doctor check:

```bash
# 1. Check formatting and linter warnings
cargo fmt --check
cargo clippy -- -D warnings

# 2. Run all unit & mathematical invariant tests
cargo test --release

# 3. Build release binary and run workspace cryptographic & hygiene audit
cargo build --release
./target/release/heav3ns-censor doctor --fix
```

---

## 3. Architecture Decision Records (ADRs) & Postmortems

- **Significant Architectural Changes:** Propose a new ADR in [`docs/adrs/`](docs/adrs/) following the template in [`ADR-0001`](docs/adrs/ADR-0001-deterministic-pixel-engine-over-generative-inpainting.md).
- **Bug Fixes & Mathematical Edge Cases:** Any root-cause fix for a geometric, compositing, or verification bug must include:
  1. A deterministic regression test in [`src/main.rs`](src/main.rs).
  2. If the bug represents a class of mathematical or systems failure, a brief postmortem in [`docs/postmortems/`](docs/postmortems/).

---

## 4. License of Contributions

By submitting a contribution to this repository, you agree that your contribution is licensed under the **MIT License** as described in [`LICENSE`](LICENSE).

