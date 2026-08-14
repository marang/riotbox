# Riotbox Review Artifacts

Review documents in this directory are point-in-time evidence. They preserve what
was true, risky, or recommended at the moment of a review. They are not the live
backlog and they should not be treated as proof that a finding is still open.

Before turning a review finding or recommended follow-up into a new ticket:

1. check the current implementation on `main`
2. search current Linear issues, including recently done or canceled issues
3. search `docs/archive/linear_issues/` only when ticket history is needed
4. decide whether the finding is still open, already shipped, duplicate,
   superseded, or intentionally deferred

If the finding is already closed, do not create a new implementation ticket.
Reference the shipped ticket or archive entry instead. If the finding is still
open, create the smallest bounded ticket that fits the current roadmap phase.

When a newer review refreshes an older one, link the newer document from the old
review when doing so is useful, but do not rewrite historical findings just to
make old reviews look current.

External reviews follow the same freshness rule. Treat them as useful
point-in-time evidence, not as automatically current backlog. If an external
review cites source-level risks that are partly stale, capture a refresh note
that separates the still-valid engineering risk from the already-shipped or
superseded wording before creating Linear tickets.

Current module-ownership refreshes:

- `external_review_refresh_2026-05-22.md`: external-review freshness check for
  `jam_app`, audio QA, and runtime ownership findings.
- `tui_include_shell_audit_2026-05-22.md`: TUI include-shell audit and
  leaf-first module-conversion recommendation.

Current P012 source-timing refreshes:

- `p012_real_source_timing_confidence_review_2026-05-22.md`: current local
  example confidence rows and the next downbeat-ambiguity surface slice.

Current P023 rejected-experiment closeouts:

- [riotbox_1422_h27_h30_rejected_experiment_closeout_2026-08-02.md](./riotbox_1422_h27_h30_rejected_experiment_closeout_2026-08-02.md):
  artifact-bound H27-H30 verdicts, retired V7-V10 mechanism family, and the
  narrow re-extraction boundary for stack-only work that never reached `main`.

Current P023 algorithm-value refresh:

- [riotbox_1433_audible_algorithm_value_audit_2026-08-12.md](./riotbox_1433_audible_algorithm_value_audit_2026-08-12.md):
  portfolio-wide retain/replace/retire evidence and the frozen bounded handoff
  to RIOTBOX-1432's product-owned W-30 hook selection.
- [riotbox_1432_w30_source_hook_selection_2026-08-14.md](./riotbox_1432_w30_source_hook_selection_2026-08-14.md):
  frozen three-source comparison, exact RuntimeMix identities, and fail-closed
  no-winner verdict with current product behavior retained.
