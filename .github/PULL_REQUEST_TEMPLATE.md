## Summary
Describe the changes introduced by this Pull Request and link any relevant ADR (`docs/adrs/`) or Postmortem (`docs/postmortems/`).

## Architectural Invariant Checklist
- [ ] **Bit-Exact Preservation:** `non_censored_pixel_diff_count == 0` outside declared SDF masks (`alpha == 0.0`).
- [ ] **Immutable Source Sampling:** Spatial filters sample strictly from `original: &RgbaImage`, never from mutable `canvas`.
- [ ] **Multi-Scale Resilience:** `1080p` (`Q85` JPEG) and `540p` (`Q75` Thumbnail JPEG) verification passes.
- [ ] **Unit Tests:** `cargo test --release` passes with a regression test covering the new behavior.
- [ ] **Workspace Doctor:** `./target/release/heav3ns-censor doctor` reports `overall_healthy: true`.
