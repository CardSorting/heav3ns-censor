# PLAYBOOK-04: Workspace Doctor Audit & Incident Recovery

**Playbook ID:** `PB-04`  
**Owner:** Deputy Gemini & Studio Operations  
**Trigger:** End of every batch session, CI/CD pre-release gate, or suspected file corruption/orphan state.  

---

## 1. Running the Cryptographic & Hygiene Audit (`doctor`)

At the conclusion of every post-production session, execute:

```bash
./target/release/heav3ns-censor doctor --fix
```

### The 9 Automated Invariant Checks
1. **`workspace_directories`:** Verifies all 7 managed directories (`inbox`, `originals`, `inspect`, `previews`, `published`, `review_escalated`, `ledger`) exist.
2. **`original_sha256:<asset_id>`:** Reads every archived master file in `assets/originals/` and verifies its cryptographic `SHA-256` matches the ledger hash recorded at ingestion.
3. **`published_sha256:<asset_id>`:** Verifies every published edition in `assets/published/` matches its recorded `SHA-256`.
4. **`bit_exact_preservation:<asset_id>`:** Verifies `artistic_preservation_exact == true`, `non_censored_pixel_diff_count == 0`, and `overall_passed == true`.
5. **`escalation_isolation:<asset_id>`:** Verifies that no asset in `Escalated` status has a `published_edition_path` or leaked file in `assets/published/`.
6. **`workspace_orphan_hygiene`:** Scans all managed output directories (`originals`, `inspect`, `previews`, `published`, `review_escalated`) against the active ledger and prunes any untracked or orphaned files when `--fix` is enabled.

---

## 2. Incident Recovery Procedures

### Scenario A: Resetting an Asset to Clean Pending State
If an asset was processed with experimental regions and needs to be returned to its freshly ingested state without losing its archived original:

```bash
./target/release/heav3ns-censor reset --asset-id <ASSET_ID_OR_PREFIX>
```

### Scenario B: Re-Synchronizing Markdown & HTML Reports
If [`ledger/publication_report.md`](../../ledger/publication_report.md), [`ledger/escalations_review.md`](../../ledger/escalations_review.md), or [`ledger/dashboard.html`](../../ledger/dashboard.html) were manually edited or deleted:

```bash
./target/release/heav3ns-censor sync-reports
```

### Scenario C: Stale Lock File Recovery
If a process was hard-killed (`SIGKILL`) while holding `ledger/.ledger.lock`, `WorkspaceLock::acquire` automatically detects and recovers locks older than `30` seconds, or you can run `doctor --fix`.
