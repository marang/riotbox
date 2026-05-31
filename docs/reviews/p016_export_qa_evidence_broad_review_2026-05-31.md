# P016 Export QA Evidence Broad Review - 2026-05-31

Scope:

- `crates/riotbox-core/src/session/export_types.rs`
- `crates/riotbox-core/src/export_qa.rs`
- `crates/riotbox-core/src/export_qa/stem_package_tests.rs`
- `crates/riotbox-app/src/jam_app/product_export.rs`
- related P016 export specs

Reason:

- Broad review cadence after RIOTBOX-1076, RIOTBOX-1075, RIOTBOX-1077,
  RIOTBOX-1078, and RIOTBOX-1079.
- Focused on the current typed export artifact evidence and stem QA gate
  contracts before any real stem writer lands.

## Findings

### 1. Opt-in stem evidence policies accept placeholder evidence

- Location: `crates/riotbox-core/src/export_qa.rs:171`
- Category: scope
- Severity: major
- Title: Lineage and fallback policies check presence, not evidence identity
- Description: `require_lineage_evidence` passes when any lineage field is
  present, and `require_fallback_comparison_evidence` passes when
  `fallback_comparison` is `Some`. The gate does not yet reject an empty source
  graph hash, blank capture ids, blank fallback reference identity, or a
  comparison payload with no metric fields. That is acceptable for the first
  structural slots, but too weak before a stem export scope depends on these
  policies as a QA gate.
- Suggestion: Add a narrow validation slice that rejects blank lineage identity
  and blank/metricless fallback comparison evidence when the corresponding
  policy is enabled. Keep threshold interpretation and real source-vs-fallback
  rendering out of that slice.

## No Findings

- Export receipt state remains the single persisted export truth; no second
  export persistence model was introduced.
- Recovery/hydration preflight now validates local artifact-set entries while
  keeping URI entries identity-only.
- Product-mix export remains the only implemented export command; stem package,
  DAW session, live recording, and host-audio capture remain unclaimed.
- `export_qa.rs` is back under the Rust review budget after the semantic test
  split.

## Follow-Up Tickets

- Create a P016 follow-up for stricter structural validation of lineage and
  fallback comparison evidence before enabling these policies for a real stem
  export scope.
