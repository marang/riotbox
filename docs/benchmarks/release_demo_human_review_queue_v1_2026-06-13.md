# Release-Demo Human Review Queue v1

P023 adds a CI-safe review queue that turns release-demo candidates into an
explicit human listening handoff.

The queue reads the release-grade demo bank and source-family coverage report,
then emits JSON and Markdown artifacts with every unverified candidate, its
artifact refs, its source-family gap, and the next review action. It does not
listen, does not score musical taste, and does not grant demo-ready status.

Run it with:

```bash
just release-demo-human-review-queue-fixtures
```

Generated local artifacts:

```text
artifacts/audio_qa/local-release-demo-human-review-queue/release-demo-human-review-queue.json
artifacts/audio_qa/local-release-demo-human-review-queue/release-demo-human-review-queue.md
```

The gate currently expects high-priority review work for `bad_timing`,
`pad_noise`, and `weak_source`, medium-priority review work for `sparse_drums`,
and all queued candidates to remain `human_verdict: unverified`,
`demo_readiness: unverified`, and `quality_claim: false`.

This is useful to the software because the next human-review work becomes a
stable contract instead of scattered artifact paths. It is useful to the
musician because every candidate points to the exact WAV, metrics, and prompt
that need listening before anyone can claim the output is demo-ready.
