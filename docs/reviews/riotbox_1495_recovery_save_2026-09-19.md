# RIOTBOX-1495 — recovery/save and external Graph storage

Status: merged in PR #1521 at `b0d8571afeb1070e5604fb1d71623ef276eaa438`.
Local source-free CI and remote Rust CI `35462169360` passed; independent review
and self-review have no remaining blocking findings.

## Scope

Maintenance of the existing Session-last/immutable-generation transaction, not
a new persistence model. A recovered external Graph Session must support edit,
save and reload even when its mutable alias contains invalid JSON or UTF-8.
RBX-374 defines the bounded recovery authority and hardlink-required storage
contract. Capture content identity is separately scoped in RIOTBOX-1497.

## Behavior and evidence

- Public `JamAppState::from_json_files` -> edit -> `save` -> reload reproduced
  the original JSON error before the fix (`/tmp/riotbox-1495-red.log`).
- The persisted Session's path/hash reference and exact generation authorize
  replacement; an edited graph or unrelated valid generation cannot substitute.
- Missing/malformed/hash-invalid generation failures preserve Session and alias.
- Injected interruption at all existing transaction checkpoints preserves the
  previous recoverable Session/graph pair.
- A missing output not named by the persisted Session remains fresh output:
  embedded-to-external conversion and relocation retain Session-last ordering.
- Invalid UTF-8 was separately reproduced as `Io(InvalidData)` before correction
  (`/tmp/riotbox-1495-utf8-red.log`). Other alias I/O failures remain errors.
- Core injects unsupported hard-link calls for existing and missing destinations:
  no replacement/publication, temporary-file cleanup, preserved error source.
  Other link errors remain ordinary I/O errors. App checkpoint injection proves
  propagation and unchanged mutable state, not an actual filesystem integration.
- Focused transaction suite: 16 passed (`/tmp/riotbox-1495-recovery-final.log`).
  Core immutable-publication suite: four passed. Full source-free `just ci`
  passed (`/tmp/riotbox-1495-ci.log`), including the final App suite, synthetic
  QA, formatting, contracts and strict all-target/all-feature Clippy.

## Review findings and disposition

1. **P2 — fresh Graph destinations incorrectly treated as recovery.** Independent
   App review confirmed the embedded/external relocation regression. Fixed by
   distinguishing a persisted reference to the missing destination from fresh
   output. Regression reproduced red, then passed green.
2. **P2 — invalid UTF-8 alias remained unsaveable after recovery.** Independent
   App review found the JSON reader's separate InvalidData path. Fixed with the
   same persisted-generation authority and a public roundtrip regression.
3. **P2 — unsupported write errors mislabeled as hard-link failures.** Parent
   self-review and independent Core review found combined write/link error
   classification. Fixed by handling write failures before calling/classifying
   the hard-link operation. PermissionDenied and unsupported-link tests pass.

Generation publication and its filesystem tests now have a semantic Core module
owner, `persistence/graph_generation.rs`; the public API and error owner stay in
`persistence.rs`. No new dependency, runtime recovery flag, ActionCommand,
Session schema or replay truth is introduced. The cohesive transaction test file
is slightly over the soft 500-line signal; this alone does not justify shards.

Independent final App/Core/docs re-review and parent self-review found no
remaining blocking findings. The readable valid-JSON alias path retains its
existing legacy-preservation behavior; the changed recovery authority applies
to missing aliases and malformed JSON/UTF-8, not a new general recovery policy.

## Limits

No source, Holdout, commercial audio or human playback was accessed. No sound
change or musical verdict is claimed. Real exFAT/FAT32/OS-matrix execution,
power-loss durability and concurrent writers are not qualified. Unsupported
external storage is rejected; no copying fallback is introduced. Capture WAV
identity remains open in RIOTBOX-1497.
