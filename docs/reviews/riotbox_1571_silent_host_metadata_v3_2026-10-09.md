# RIOTBOX-1571 — source-free V3 module metadata repair

Date: 2026-10-09. P017 operational maintenance removing the concrete
RIOTBOX-1041 tooling blocker established by the consumed RIOTBOX-1570 attempt.
This is not an audible slice, a successful host attempt or musical progress.

## Frozen contract and unchanged history

[V3](../benchmarks/silent_host_observation_v3.md) / RBX-440 was committed before
implementation at `0d046d7c`. Protocol SHA-256:
`a0934097be7b2f160ff698b3d69c1dd7180e86defcdc0974c5e23606297ef85b`.
Every predecessor numeric/profile/access/containment field is unchanged.

V1 SHA-256 remains
`ba26bc50db85a6b745738917100689a7ca5ac3c2aa95cc7c699db2ef404358d3`;
V2 remains
`9b8517f13285e36a6082fb57131602949c89758199d57ccef42a26f4a542bcea`.
Their contracts, the exact consumed 1570 command and its six generated tests,
the driver and all Rust/runtime/DSP code are unchanged. No ignored attempt
evidence was opened or modified during this repair.

## Practical change

The existing module seam pairs exact JSON argument payloads with actual
canonical short-text indices. It consumes full payload frames instead of
splitting raw arguments into lines, keeps original argument values and models
text-mode newline/final-record normalization explicitly. JSON order or an
invented JSON index never establishes module ownership. Ambiguous candidates,
projections, stale pairs, duplicate IDs and incomplete multisets fail closed.

Native `libpipewire-module-*` bodies are explicitly opaque; apostrophes,
comments and header-looking contents cannot manufacture Pulse ownership.
Their real indices still participate in bound-ID replacement checks. All
other module candidates retain strict exact Pulse type, top-level owner and
stereo-token checks. Before load, validated exact-name Node absence prevents
an existing native-created sink collision; no collision is mutated or unloaded.

The existing absolute deadlines cover both metadata queries and cooperative
framing/ownership work. A small in-memory argument reader also checks during
one long Pulse token, rather than only between yielded tokens. Deadline errors
retain their attribution instead of being wrapped as token-syntax errors.
There is no new launcher, dependency, product state or parallel runtime.

Current orchestration loads V3 through a named pinned loader. The historical
V2 loader remains solely for compatibility with the frozen generated command
tests; neither loader grants execution authority. The ordinary CLI remains
blocked before any host query, file creation or subprocess.

## Regression evidence

Independent generated public-seam RED controls on the original parser proved:

- Native multiline/tab arguments fail as malformed short-module columns.
- An apostrophe in an unrelated native comment fails as malformed Pulse tokens.

Durable RED log: `/tmp/riotbox-1571-independent-adversarial-red.log`.
The same independent test module subsequently passes 19 tests:
`/tmp/riotbox-1571-independent-adversarial-green.log`.

A coordinator performance/operations check retained one implementation finding
before PR: token-level checks alone do not cooperate with the caller's deadline
inside a single large quoted token. A deterministic synthetic-clock public-seam
test fails before the reader correction and passes afterward:
`/tmp/riotbox-1571-token-deadline-red.log` and
`/tmp/riotbox-1571-token-deadline-green.log`. These are artificial clocks/data,
not new production time thresholds or real-host observations.

All 134 source-free silent-host tests pass with ResourceWarnings fatal
(`/tmp/riotbox-1571-fixtures-final.log`). Coverage includes actual text IDs,
multiline/tab/CRLF and apostrophe native bodies, forward terminal normalization,
argument-space preservation, fake headers, opaque owner-like content, JSON
reordering/extra-index non-authority, multiplicity, canonical/duplicate IDs,
wrong/conflicting owners, prefix/projection ambiguity, malformed/stale/partial
tables, native replacement before/after unload, missing acknowledgement,
shared deadlines between queries/during framing/within a token, pre-load Node
collision, historical V2 command behavior, disabled CLI and existing route/
watchdog/signal/publication/containment cases.

Two independent cross-integration reviews and coordinator self-review retain
zero P0–P3 findings after the resolved intra-token deadline finding. Reviewers
exclude their own implementations/test files from independent attribution;
their independent focused checks passed (59 cross-integration tests, 17 framing
tests and 18 legacy module tests). Full source-free local `just ci` passed
(`/tmp/riotbox-1571-ci.log`), including all 134 generated host tests.
Native CI on the final PR head gates merge. Exact CI/PR/review closeout is
recorded in Linear and the project journal. No generated result is promoted
into real-host or human evidence.

## Residual limits and authorization

Paired metadata reads and subsequent unload are sampled, not atomic. Matching
snapshots cannot prevent malicious inter-command replacement. Ambiguous prefix
or newline/terminal projection matches and churn intentionally reject even
some benign tables, without retry or backtracking. Native bodies are not fully
semantically validated. Existing per-command response limits are not an
aggregate RSS cap; cooperative deadlines do not protect against kernel stalls
or every non-interruptible library operation. Existing process-escape,
SIGKILL/power-loss and adversarial-server exclusions remain.

No actual host/service metadata, CPAL, source/Holdout/commercial/capture audio,
source-directory discovery, physical/default output, DAW, listening, TUI or
Windows expansion occurred. No ActionCommand, Session/replay or Rust change.
The musician gets a repaired observation tool, not a new sound or performed
feature. RIOTBOX-1041 remains open, and V1/V2 remain consumed negative evidence.
Any future real attempt still needs fresh explicit user authorization and a
separately bound prospective owner/reviewed revision/binaries. V3 is source-free
only and grants no callback, device, endurance, musical, hardness or release pass.
