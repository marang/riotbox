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
