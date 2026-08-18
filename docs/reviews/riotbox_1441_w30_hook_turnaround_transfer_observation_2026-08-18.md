# RIOTBOX-1441 W-30 Hook-Turnaround Transfer Observation

Date: 2026-08-18  
Partition: Development only  
Result: `informative_human_transfer_evidence_formal_qualification_fail_closed`  
Mechanism changes: none  
Holdout access: none  
Commercial-reference access: none

## Question

Does the shipped, unchanged `w30_hook_turnaround_v1` remain musically useful
across five additional registered Development sources, and does it improve on
the underlying W-30 transformation often enough to justify continued product
use?

RIOTBOX-1441 did not authorize mechanism or threshold tuning. The source-bound
results below therefore describe where the existing gesture transfers and
where it does not.

## Technical Evidence

All five eventual listening sources produced exact RuntimeMix qualification
reports for the unchanged mechanism. Each report confirms:

- only `w30_preview` contributed audio
- the first relative beat and ordinary return were sample-exact
- both frozen articulation windows differed from control
- 128- and 257-frame callback partitions produced identical output
- capture lineage, grit, and Source Monitor state remained unchanged
- the missing-source control remained silent
- no clipping or limiter intervention occurred

The qualification helper initially compared boundaries by multiplying a
separately rounded one-beat length. That accumulated several frames of error at
non-integer product tempos. RIOTBOX-1441 now derives comparison offsets using
the same cumulative per-frame transport progression and boundary snap as the
runtime, with a focused regression at `172.26566` BPM. This changes no product
audio or turnaround constant.

## Human Listening Observations

The final judgments came from source-first playback followed by longer repeated
A and B presentations with a one-second separation. A was the ordinary W-30
source transformation; B applied `w30_hook_turnaround_v1` to that same
transformation.

| Case | Family | Final musician assessment | Long comparison SHA-256 | Duration |
| --- | --- | --- | --- | ---: |
| `freesound_alastair_pursloe_183441` | dense break | A and B are both usable. The first short comparison made B seem weaker, but source-first extended repetition showed that B is usable like the other positive examples; no clear preference remained. | `6fa77b2a28f58173c380295d2a8c6befefc61e16d0503d55f2fca0fca92845a9` | `16.987000` s |
| `freesound_dabromusic_266735` | dense break | A is good and B is better. The turnaround improves the transformed source. | `eb84da6a04ddeb3c95f24cc97f7e61891bc05238491a4e06c977e3b750e1e0a1` | `16.968333` s |
| `freesound_dr_skitz_353853` | sparse drums | Both transformations are very good, with the same favorable B-over-A assessment as the preceding example. | `e63046e60b3e6c017fdc4841a3987abc0e3a3de60ba9dde8cfe4c02798fa2bfb` | `18.298000` s |
| `freesound_jmarcosfer_591426` | sparse drums | Strong result matching the preceding two positive B-over-A examples. | `060fc410671a795770b2b09fb264ff1fa2bbde1268b66a6d4410964ac7da906e` | `16.312000` s |
| `freesound_aikighost_19059` | electronic drums | A is good. B remains clear, but reduces groove and musical utility and is therefore not preferred for this source. | `65a5b15f7426fab6ff4408b5e6bdbb42f09c376bba83f6a9b4d53f5bd9469442` | `19.320500` s |

The durable aggregate observation is:

- the underlying A transformation was useful on all five sources
- B was usable on four of five sources
- B clearly improved A on three sources
- one source supported B without a clear preference over A
- one source retained clarity but lost groove and musical usefulness under B

The initial judgment reversal on the first case is meaningful methodology
evidence: a short isolated A/B can underrepresent the musical role of a phrase
gesture. Source context and repeated phrase-scale playback made that case more
reliably assessable.

## Formal Qualification Boundary

The frozen v2 contract permits one paired review artifact per source with a
maximum duration of 10 seconds and requires one completed access log binding
the five exact final cases. The extended comparison artifacts above are
approximately 16--19 seconds, and the local access evidence does not contain
one completed session binding precisely those five final listening cases.

The listening observations are therefore useful product evidence but do not
satisfy the frozen formal transfer-qualification claim. RIOTBOX-1441 closes
fail-closed rather than retroactively relaxing its contract. Any future formal
rerun with source-first, repeated phrase-scale presentation requires a new
versioned contract and durable decision before source access; it must not tune
`w30_hook_turnaround_v1` from these results.

## Product Consequence

Keep Hook Turnaround as a performer-owned W-30 articulation. The observations
support continued use and show genuine transfer, but also show that it is not a
universal improvement. Future source intelligence may recommend or audition it
when the source supports the gesture; it should not apply it automatically or
hide the ordinary transformation.

