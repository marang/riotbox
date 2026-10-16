# `RIOTBOX-1569` — Bind product-mix export publication to the exact validated proof and copied bytes

- Ticket: `RIOTBOX-1569`
- Title: `Bind product-mix export publication to the exact validated proof and copied bytes`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1569/bind-product-mix-export-publication-to-the-exact-validated-proof-and`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `P016 maintenance protecting the accepted P023 trustworthy export handoff; no issue milestone returned`
- Status: `Done`
- Created: `2026-10-09`
- Started: `2026-10-09`
- Finished: `2026-10-09`
- Branch: `feature/riotbox-1569-product-mix-byte-identity`
- Linear branch: `feature/riotbox-1569-bind-product-mix-export-publication-to-the-exact-validated`
- Assignee: `Markus`
- Labels: `Bug`
- PR: [#1672](https://github.com/marang/riotbox/pull/1672)
- Merge commit: `c3f28432f0abe66e5d0b690eaf0a46e85b92588a`
- Deleted from Linear: `Pending archive merge and verified deletion`
- Verification: Historical deterministic mutation RED/GREEN; six public-method regressions + two copy-leaf failure tests + 23 prior export tests; full source-free local just ci; exact-head native CI 37969924335 passed
- Docs touched: `docs/research_decision_log.md`, `docs/reviews/riotbox_1569_product_mix_byte_identity_2026-10-09.md`, `docs/specs/action_lexicon_spec.md`
- Follow-ups: RIOTBOX-1570 separately owned a newly authorized one-shot silent-host phase; RIOTBOX-1576 is a related later architecture checkpoint

## Why This Ticket Existed

RIOTBOX-1568 statically verified a pre-existing ProductMix check/use identity gap. The musician-facing E path parsed proof bytes, hashed artifact/proof through separate reads and then copied reopened mutable paths. Replacing artifact A with B during that interval could publish B while the committed receipt claimed A. Changing proof bytes could similarly separate validation from publication, contrary to Action Lexicon section 6.8.

## What Shipped

- Parse, hash and publish one captured proof buffer, eliminating separate proof path reads as independent identity authorities.

- Stream and hash artifact bytes into a private temporary descriptor, require the expected validated digest, and retain the descriptor instead of reopening the mutable handoff path.

- Verify hashes of exclusive final-copy bytes before any successful receipt/Action/Session commit. Preserve create-new/no-overwrite semantics, complete identical-bundle idempotency, and fail-closed behavior for incomplete/different bundles.

- Failure cleanup removes only owned newly created outputs, not unrelated/pre-existing files or directories. Queue/observer failures remain explicit; copied artifact, proof, readiness and committed receipt identities agree on success.

- RBX-438 and Action Lexicon section 6.8 record the publication policy and resource/atomicity limits. Existing action, persistence, replay and realtime ownership remain unchanged; no new dependency, state, schema or hardlink requirement was introduced.

## Historical Verification And Outcomes

- The exact generated public-method regression reproduced successful receipt after swapping A for B on the old writer, then passed after repair. An initial short exact-filter command selected zero tests and was explicitly excluded from RED evidence; no timing sleeps or real source assets were needed.

- Six public snapshot/identity/destination/failure regressions, two copy-leaf failure regressions and all 23 prior product-export cases passed.

- Two independent Rust/contract reviews, final documentation review and coordinator self-review retained zero P0-P3 findings. Full source-free local just ci passed including strict all-target/all-feature Clippy; diff checks passed.

- The subsequent main synchronization added only the separately merged RIOTBOX-1568 checkpoint report; tested implementation remained unchanged.

- Native CI 37969924335 passed both existing jobs on exact reviewed head a75532044abe4ad0cb3bce1faaa662c12fa8f665 before merge. Historical closeout fast-forwarded clean main/origin/main and removed exact local/remote feature branches.

## Notes

- Private staging requires temporary disk space. Proof JSON remains uncapped. There is no crash-atomic two-file publication, power-loss, adversarial destination-mutation or aggregate resource guarantee.

- No real source/Holdout/commercial/capture audio, source-directory discovery, playback, host-audio metadata/services/devices, DAW, V1 retry, V2 enablement, TUI or Windows expansion occurred. Sound was unchanged and no new listening session or musical/hardness/host/DAW/release qualification was claimed.

- The issue was retained in Done after historical closeout. A separate future V2 host attempt was expressly authorized by the user after this repair, requiring its own prospective owner rather than changing or retrying prior phases.

- Detailed historical evidence: [implementation report](../../reviews/riotbox_1569_product_mix_byte_identity_2026-10-09.md).

- Archive verification on 2026-10-16 read the full current Linear issue, all returned comments (no further page), its linked implementation PR body/files and merge/check metadata. Linear was Done and the exact PR was MERGED with both existing native checks successful. The tests, source/audio exclusions and host observations above are historical evidence, not new executions; archival work did not run product/audio tests, query host audio or open source/ignored evidence artifacts.
