# RIOTBOX-1422 H27-H30 Rejected Experiment Closeout

- Date: `2026-08-02`
- Ticket: `RIOTBOX-1422`
- Work class: `maintenance/regression` closeout of a canceled audible experiment
- Disposition: preserve evidence; ship no recipe or Rust from the abandoned stack

## Conclusion

H27's pushed branch passed Rust CI. H28-H30 remained dirty local generations,
not GitHub-CI-tested revisions. Their manifests preserve exact-path technical
passes; H30 additionally records local `just ci`, audio-QA, and branch-review
passes. None earned a musical pass. The checks proved timing, ownership, body
retention, clipping safety, event duration, and event-local spectral change.
They did not prove that Hard sounded harder, made a stronger hook, or became
worth triggering or looping.

PR [#1377](https://github.com/marang/riotbox/pull/1377) was not merged. PRs
[#1378](https://github.com/marang/riotbox/pull/1378),
[#1379](https://github.com/marang/riotbox/pull/1379),
[#1380](https://github.com/marang/riotbox/pull/1380), and
[#1381](https://github.com/marang/riotbox/pull/1381) merged only into that
abandoned feature stack, not into `main`. Therefore no RIOTBOX-1422 Hard
recipe, supporting Rust implementation, or child-ticket enabler shipped to
`main`.

## Artifact-Bound Verdicts

| Hypothesis | Recipe | Manifest SHA-256 | Exact Base-to-Hard A/B SHA-256 | Human result |
| --- | --- | --- | --- | --- |
| H27 | V7 | `22900ec4d89d285b639ec95cee7ffef35284b6bbe845ab8905975c3cf843268e` | `0de0f6cddfc2331ab37b058e0ae5e50567f4620b537d54f99c7cdaa4007a2bce` | Technically okay but musically weak, recessed, and not accepted as loopable. |
| H28 | V8 | `e7d5165814f7e38d7571b085ce1d3e7ccdb27628a26b38aec2c6612a71b082b9` | `19ec4df6389934f23940e0e0e61e7b85b7b9b5214b8e73b0c91c04876efdd713` | The product-timing-corrected comparison sounded duller, not harder. |
| H29 | V9 | `436c870c51f3e256d310896531e78879d8dd3ca28e1e9ebbcdc6e2c56952f185` | `f62043eb63ae4fabaf3c31080406af8898e0c722c9319b51a4f46d177e04cacd` | Base and Hard sounded the same; no useful hardness appeared. |
| H30 | V10 | `53a9d7ce1b158b04930632003de8501cafd8222246e5116eafde86aeac6f9d16` | `bb683cddf9f07cf2b16db9616e0cdb3efa4e881fe963afdd6e5218e43631b7ad` | No useful distinction from the preceding attempts. |

The H28 verdict is bound to the corrected A/B above, not the earlier render
that mixed the approximate confirmation BPM with the stored Source Graph
product timing. H28 and H29 preserve artifact-bound human observations inside
their historical manifests, but no standard listening-review pack exists for
either generation. The H30 WAV and structured review pack remain local-only
and are not committed in this closeout. The local `review.json` has SHA-256
`97b4e2331938bc259e069ae4968da9b05ad8ea10b5fdc706fcbfef5319d5364c`.
H30 is therefore hash-bound historical evidence, not an artifact-complete
review pack.

## Retired Mechanism Family

V7-V10 and the continuous-Base plus sparse parallel-overlay mechanism family
are retired. Later versions changed increasingly specific measurements while
the continuous Base remained the dominant musical object. Another gain, hold,
EQ, filter, saturation, trigger-mask, or threshold adjustment inside that
family is not a new hypothesis and must not be promoted for playback.

The accepted Base was not cleanly separable from the abandoned branch and its
rejected Hard work. It did not ship to `main`. The Base hashes in the manifests
are evidence identities only; they do not establish a shipped recipe or
product implementation.

## Narrow Re-Extraction Boundary

The following commits contain potentially reusable stack-only work. None is
an ancestor of `main`, and none should be merged or cherry-picked wholesale:

- `c67e75a06911cb4c06b42054a5215aa0495dc211`: exact Hard-recipe reachability preflight
- `58233c6dfb8a57ed976d10adbbf0fee15cf52d6a`: source-adaptive hit-shaper reachability
- `47b170db54a331ea798bec069237086249528521`: source-backed external-tempo phase
- `783776247f2a5dd0a17b6452744c317509746915`: performer-owned Hard intent through the product spine
- `9edb8f1f5033c20c0a2f805be498f1cd392d9fb9`: source-coherent V7 implementation mixed with rejected experiment work

The H28 QA correction that uses the stored Source Graph product BPM instead
of the approximate confirmation BPM was never isolated in a clean commit. The
source worktree's tracked `git diff HEAD` bytes, which include that correction
and later rejected experimental edits, have SHA-256
`64801bde7294422e8533eb7e36872a3ad8be535e40763cd44db3ce3bb92318f1`.
That hash excludes untracked files. A future issue must re-extract the timing
correction narrowly with focused tests; the dirty patch is provenance, not an
application unit.

## Follow-Up Order

1. `RIOTBOX-1429` defines the percussive-force and beat-impact construct and
   preregisters falsifiable evidence.
2. `RIOTBOX-1428` may then reconstruct source impact events and gate a local
   Hard takeover through an isolated-event human pass before product
   integration.

The successor path must preserve the existing Source Graph, Session, Action
Lexicon, queue/commit, capture-lineage, replay, fail-closed, and realtime
boundaries. It must not treat these archived technical passes as musical
acceptance.
