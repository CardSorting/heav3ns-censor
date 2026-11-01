# PLAYBOOK-03: Ambiguity Escalation, Revocation, & Artist Review

**Playbook ID:** `PB-03`  
**Owner:** Deputy Gemini, MAJO (Executive Chair), EGG (Principal Artist)  
**Trigger:** Genuine compliance or artistic ambiguity encountered during inspection or post-processing.  

---

## 1. When to Escalate (Commission Section 5 & Section 7)

Deputy Gemini is commissioned to make routine technical determinations autonomously, **but must never guess** when:
1. **`stylized_anatomy_ambiguity`:** Biomechanical, cybernetic, or abstract forms sit on the boundary between non-anatomical sculpture and explicit depiction.
2. **`composition_impact`:** Any compliant censorship region would sever the primary focal point or destroy the narrative meaning of EGG's illustration.
3. **`platform_policy_conflict`:** Target distribution platforms have contradictory requirements that require an executive edition split.

---

## 2. Executing an Escalation (with Automatic Publication Revocation)

Run the `escalate` command with a clear `--category`, `--reason`, and `--notes`:

```bash
./target/release/heav3ns-censor escalate \
  --asset-id heav3ns_03 \
  --category "stylized_anatomy_ambiguity" \
  --reason "Abstract biomechanical groin core has high-contrast vertical cleft lines; unclear whether stylized armor or explicit anatomy under Article 175." \
  --notes "Candidate coordinates: (340, 290, 120x130) in pixels. Awaiting MAJO/EGG determination."
```

### Automatic Safety Guarantees on Escalation
- If the asset previously had a file in `assets/published/` or previews in `assets/previews/`, `escalate` **immediately revokes and deletes** those published/preview files so an unreviewed asset can never leak into a release bundle.
- Copies the untouched asset to `assets/review_escalated/<ASSET_ID>_review.<EXT>`.
- Appends a structured entry to [`ledger/escalations_review.md`](../../ledger/escalations_review.md) and flags the card in [`ledger/dashboard.html`](../../ledger/dashboard.html).

---

## 3. Resolving an Escalated Asset After Executive / Artist Decision

Once MAJO or EGG provides a determination:
- **If approved untouched:** Run `./target/release/heav3ns-censor approve-original --asset-id <ID> --classification artistic_nudity_pass --rationale "Executive sign-off by MAJO"`.
- **If localized censorship is directed:** Run `./target/release/heav3ns-censor execute --asset-id <ID> --region "..."`.
- Both commands automatically clear the escalation state, remove the file from `assets/review_escalated/`, and update `ledger/escalations_review.md`.

