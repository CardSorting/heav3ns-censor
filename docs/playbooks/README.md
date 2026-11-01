[🏠 Home (`README.md`)](../../README.md) &rsaquo; [📚 Docs Portal (`docs/`)](../README.md) &rsaquo; **📋 Operational Playbooks (`SOPs`)**

---

# Operational Playbooks (`docs/playbooks/`)

Standard Operating Procedures (`SOPs`) for autonomous agents (`Deputy Gemini`) and studio operators running the HEAV3NS post-production pipeline.

| Playbook ID | Title | Trigger Condition | Key CLI Commands |
| :--- | :--- | :--- | :--- |
| **[`PLAYBOOK-01`](PLAYBOOK-01-autonomous-batch-ingestion-and-vlm-triage.md)** | **Autonomous Batch Ingestion & VLM Triage** | New artwork placed in `assets/inbox/` | `batch-prepare`, `inspect --crop` |
| **[`PLAYBOOK-02`](PLAYBOOK-02-precision-censorship-and-incremental-region-editing.md)** | **Precision Censorship & Incremental Region Editing** | Asset classified as `explicit_requires_intervention` | `execute --region`, `--append`, `--remove-region`, `--dry-run` |
| **[`PLAYBOOK-03`](PLAYBOOK-03-ambiguity-escalation-revocation-and-artist-review.md)** | **Ambiguity Escalation, Revocation, & Artist Review** | Genuine artistic or compliance ambiguity (`ambiguous_escalate`) | `escalate`, `approve-original`, `explain` |
| **[`PLAYBOOK-04`](PLAYBOOK-04-workspace-doctor-audit-and-incident-recovery.md)** | **Workspace Doctor Audit & Incident Recovery** | End-of-session verification, CI gate, or state reset | `doctor --fix`, `reset`, `sync-reports` |
| **[`PLAYBOOK-05`](PLAYBOOK-05-data-annotation-and-false-positive-suppression.md)** | **Contrastive Data Annotation & False-Positive Suppression** | Hard-negative confuser curation, active learning hardness scoring, & dataset export | `annotate-negative`, `export-dataset`, `doctor --fix` |

---

**Related:** [🤖 Agent Onboarding (`AGENT_ONBOARDING.md`)](../onboarding/AGENT_ONBOARDING.md) • [🏛️ Stakeholder Handoff (`STAKEHOLDER_HANDOFF.md`)](../stakeholder-handoff/STAKEHOLDER_HANDOFF.md)

