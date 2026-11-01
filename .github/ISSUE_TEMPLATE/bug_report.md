---
name: Bug Report / Mathematical Edge Case
about: Report a geometric SDF, spatial kernel, or multi-scale verification bug
title: "[BUG] "
labels: ["bug"]
assignees: []
---

## Summary of Issue
Describe the unexpected behavior or mathematical edge case.

## Reproduction Steps / CLI Command
Provide the exact `heav3ns-censor` CLI command or `--region` specification:
```bash
./target/release/heav3ns-censor execute --asset-id ... --region "..."
```

## Expected vs. Actual Verification Telemetry
- **Expected:** `overall_passed: true`, `non_censored_pixel_diff_count: 0`
- **Actual:** (Paste JSON output from `execute` or `doctor`)

## Environment
- Engine Version (`./target/release/heav3ns-censor --version`): `4.0.0`
- OS / Architecture:

