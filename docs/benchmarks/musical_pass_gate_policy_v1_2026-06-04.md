# Musical Pass Gate Policy v1

`riotbox.musical_pass_gate_policy.v1` defines which verdict language Riotbox may
use for technical validity, automated musical evidence, human listening, and
future calibrated agent judgment.

Only two states may claim musical pass:

- `human_musical_pass`: a structured `riotbox.listening_review.v1` human
  verdict approved the audible result for the stated use.
- `calibrated_agent_musical_pass`: a future offline audio judge has enough
  labeled Riotbox evidence for a bounded source family and still records
  `human_verdict: unverified`.

States that must not claim musical pass:

- `technical_fail`
- `technical_pass`
- `agent_fail`
- `agent_weak`
- `agent_promising`
- `human_musical_fail`

The policy exists because `agent_promising` and passing deterministic metrics are
useful merge evidence, but they are not taste approval. For software, this keeps
PRs, CI, and generated reports from overclaiming. For the musician, it means
Riotbox does not call a loop "good" until the hook, bass pressure, destructive
contrast, source character, and replay value survive the right review layer.

The committed fixture requires any future `calibrated_agent_musical_pass` to
stay offline-QA-only, use matched pass/weak/fail labels, and keep human review
separate.

Run:

```bash
just musical-pass-gate-policy-fixtures
```
