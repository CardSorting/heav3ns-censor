# ADR-0004: Atomic BTreeMap Ledger and Dual CLI/MCP Interface

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Deputy Gemini
- **Technical Scope:** [`src/workspace.rs`](../../src/workspace.rs), [`src/models.rs`](../../src/models.rs), [`src/mcp.rs`](../../src/mcp.rs), [`src/main.rs`](../../src/main.rs)

---

## Context and Problem Statement

Autonomous agents may invoke `heav3ns-censor` either via shell commands (`CLI`) or via structured Model Context Protocol (`MCP`) tool calls, potentially in rapid succession or parallel subagent workflows. Previously:
- Standard `HashMap<String, AssetRecord>` caused non-deterministic key ordering in `ledger/treatments_ledger.json`, creating noisy Git diffs on every save.
- Direct `fs::write` risked corrupting `treatments_ledger.json` if a process was interrupted mid-write.
- Modifying one region on a multi-region asset required re-specifying all existing regions from scratch.

## Decision Outcome

1. **Deterministic `BTreeMap` Ledger:**  
   [`Ledger.assets`](../../src/models.rs#L172-L180) uses `std::collections::BTreeMap<String, AssetRecord>`, guaranteeing lexicographically sorted, byte-deterministic JSON serialization.
2. **RAII Advisory Lock (`WorkspaceLock`) + Atomic Temp-and-Rename (`atomic_write_bytes`):**  
   All mutating workspace operations acquire `ledger/.ledger.lock` (with automatic 30-second stale-lock recovery) and write via a PID-scoped sibling temp file + `sync_all()` + POSIX `fs::rename()`.
3. **Incremental Region Editing (`--append`, `--remove-region`, `--dry-run`):**  
   Both CLI (`execute`) and MCP (`heav3ns_execute_censorship`) support upserting individual regions by ID (`r2=solid_bar:...`), removing specific region IDs, and simulating verification in a temporary directory (`--dry-run`) without mutating `published/` or `treatments_ledger.json`.
