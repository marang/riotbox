# RIOTBOX-1500 — Capture artifact isolation

Classification: maintenance/regression. Base `4db71218`. This corrects the P1
ordinary-workflow data replacement reproduced by RIOTBOX-1499. It does not
change capture selection, PCM encoding, rendering, performer actions or taste.

## Failure and correction

Both Sessions started with the valid local ID `cap-01`; the old writer mapped
that ID to the shared `captures/cap-01.wav` and truncated it. The public
capture/save/restore regression failed before the fix with
`other Session overwrote capture`. This happened even before the second
Session's JSON save, so changing Session-save ordering alone cannot repair it.

The existing capture I/O module now exclusively allocates a fresh opaque WAV
filename, writes the existing encoded buffer, then retains that same file.
Only after success does App install its real locator and encoded-byte identity
in Core `CaptureRef` and decode the same buffer into the runtime cache. Both
source-window and internal bus-print writers use this mechanism. There is no
replacement, content deduplication, new dependency, hardlink requirement or
directory discovery. Session-local IDs and lineage remain unchanged.

RBX-376 and Session spec section 11 freeze the contract. The existing
`tempfile` dependency supplies create-new allocation and ordinary-error cleanup;
the allocated name is retained directly, not renamed/copied into a potentially
occupied destination. An interrupted write or failed later Session save may
leave an unreferenced file. No scanner promotes those files into Session truth,
and no automatic garbage collection is introduced. Power-loss durability,
atomic directory visibility and concurrent writers of one Session JSON are
not promised. Existing relative/absolute legacy locators remain readable.

## Verification

New public App regressions exercise:

- different source captures in two Sessions sharing one directory, preserving
  the first WAV byte for byte and restoring both with `Loaded` identities;
- two real queued/committed internal bus prints with the same local capture ID,
  preserving both saved outputs and lineage;
- a second Session save failing on an invalid external Graph destination after
  capture materialization, leaving the first WAV and previous Session JSON
  unchanged and loadable;
- a saved legacy `captures/cap-01.wav` surviving a new Session's capture;
- artifact write failure leaving explicit pending diagnostics, no trusted
  identity/cache entry and no replacement of existing content.

Writer-level tests inject failure after two bytes have reached the newly
allocated file, then require cleanup of only that file. They also cover invalid
encoding before allocation and repeated identical content receiving distinct
files without touching the existing legacy destination. A partial-write hook
is private to the publication implementation, not a new product configuration.

The original red regression passes after correction. The expanded capture
selection run passes 107 tests. Existing raw/printed capture and replay tests
now resolve the actual Session locator instead of inventing a filename from
the local capture ID; their audio, identity, lineage and replay assertions are
retained. All audio here is newly generated synthetic fixture material; no real
source, Holdout, commercial reference, device or human playback was accessed.

Full normal-parallel local `just ci` passes, including Rust/Python tests,
synthetic audio/observer gates, format checks, contract/JSON validation and
strict all-target/all-feature Clippy. The local log is
`/tmp/riotbox-1500-ci.log`; no test-thread serialization was needed.

## Branch review

Solo `code-review` / Rust lenses cover failure ordering, file ownership, legacy
compatibility, both writer consumers, replay, callback isolation, tests and
contract changes. The durable state remains Core's existing locator/identity;
App stores no shadow identity or namespace. File I/O remains outside realtime.
There is no new `ActionCommand`, so no new queue/commit/replay surface is needed.
Existing action diagnostics and hydration status remain the observer surface.

No additional correctness finding was retained. The pre-existing fixed-name
test assumptions were updated to consume persisted locators, with their
substantive assertions preserved. Follow-up self-review found no further issue.
The shared writer remains in its existing semantic module (159 lines including
tests); no mechanical include shard or speculative storage framework was added.
This is storage-integrity evidence, not musical or release qualification.
