# Security & Integrity Policy

**Project:** `heav3ns-censor`  
**Supported Version:** `4.0.x` (Current Release)  

---

## 1. Supported Versions

| Version | Supported | Status |
| :--- | :---: | :--- |
| `4.0.x` | **Yes** | Active production & research release |
| `3.0.x` | No | Superseded by `4.0.0` (see [`PM-004`](docs/postmortems/PM-004-in-place-canvas-mutation-and-zero-sample-false-positives.md)) |
| `< 3.0` | No | Early internal prototypes |

---

## 2. Security & Cryptographic Threat Model

Because `heav3ns-censor` acts as a compliance gatekeeper between unedited studio master illustrations (`assets/originals/`) and public release distributions (`assets/published/`), we treat **verification bypasses** and **state-isolation leaks** as security-critical vulnerabilities:

1. **Cryptographic Master Integrity (`SHA-256`):**  
   Every archived master in `assets/originals/` has its `SHA-256` recorded at ingestion. Every subsequent `execute`, `approve-original`, and `doctor` invocation re-verifies the file's `SHA-256` before reading pixel data, aborting immediately on any mismatch.
2. **Escalation Isolation Guarantee:**  
   When an asset is escalated via `escalate`, any existing file in `assets/published/` is immediately unlinked, and `doctor` audits the `escalation_isolation` invariant to ensure no escalated asset can leak into a published directory.
3. **Zero-Sample Verification Hardening:**  
   As documented in [`PM-004`](docs/postmortems/PM-004-in-place-canvas-mutation-and-zero-sample-false-positives.md), degenerate micro-regions (`0` interior eroded samples) hard-fail verification (`passed = false`) and feather radii are clamped to $\le 45\%$ of the minor half-axis so excessive feathering can never create a transparent center.
4. **Concurrency & Crash Safety:**  
   All ledger and artifact writes acquire an advisory lock (`ledger/.ledger.lock`) and use atomic sibling temp-file creation + `fsync` (`sync_all()`) + `rename()`.

---

## 3. Reporting a Vulnerability

If you discover a security vulnerability, a verification bypass (e.g., a censorship configuration that passes `verify_censored_edition` while leaking unobscured explicit detail), or a file-system traversal issue in the CLI/MCP server:

1. **Do NOT open a public issue** if the vulnerability could cause accidental distribution of uncensored assets in production deployments.
2. Open a private security advisory via [GitHub Security Advisories](https://github.com/CardSorting/heav3ns-censor/security/advisories/new) or contact the **HEAV3NS** maintainers directly.
3. Include a minimal reproducible `--region` command or unit test demonstrating the issue. We aim to triage and patch verified mathematical or security issues within 48 hours alongside a documented engineering postmortem in [`docs/postmortems/`](docs/postmortems/).
