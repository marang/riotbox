# RIOTBOX-1595 — explicit conservative replay upgrade copy

Date: 2026-10-16. Base: RIOTBOX-1594 merge
`5333df221d6ea16697abeff294bc8e306e29d7ec`. P017 maintenance removing the
P023 legacy-navigation operator blockage under RIOTBOX-1041. RBX-455 and
Session §9 / replay §7.2 were frozen before production implementation.

## Operator gap and scope

RBX-454 safely refuses backward legacy navigation and already provides a
conservative Core upgrader, but the delivered CLI had no caller. The new
exclusive `--replay-chronology-upgrade-copy` mode requires explicit input and
new sibling destination. It dispatches before state loading, observer, terminal
or audio runtime. It never changes ordinary load/save or repairs ambiguous
history. Existing Core chronology admission remains unchanged.

Core persistence reads one ordinary non-symlink input once. Typed Session
decoding and conservative history checks use that buffer. Borrowed raw JSON
locates only the admitted policy token; every other byte stays identical,
including unknown extensions, duplicate opaque keys, extreme-number spelling,
escapes and absent defaulted fields. Already-V2 input is validated and copied
byte-for-byte with a distinct disposition. This is not full Session qualification.

Canonical sibling parents preserve relative reference bases without opening
referenced files. Reject occupied targets (including dangling symlinks), special
or symlink input, nonordinary leaf names, missing/different parents and lexical
or resolved reserved generation storage. Share the existing Graph publication
mechanism in a private Core owner: exclusive temporary creation, complete write,
atomic no-replace hardlink and best-effort owned cleanup. No overwrite fallback,
directory creation for this command, power-loss guarantee or hostile concurrent
directory-writer guarantee. The input is never written by this operation.

## Evidence

- Genuine pre-implementation CLI reachability RED: existing parser rejected
  the new mode after successful compilation; exit101,
  `/tmp/riotbox-1595-cli-red.log`, SHA-256
  `a021a8d7eb244c9bcbc753e0561f07bd4c19089d47381970e98066b41a63a8f1`.
- Eight new CLI tests passed in
  `/tmp/riotbox-1595-cli-green-fixed.log`; eight existing launch-argument tests
  passed in `/tmp/riotbox-1595-existing-launch-args.log`. No weakened wildcard
  matches; the new launch variant is explicit on all exhaustive surfaces.
- Core persistence selection:29 passed, no ignored/failing tests,
  `/tmp/riotbox-1595-core-copy-reviewed-final.log`. Covers complete legacy and
  already-V2 copies, same replay/new-file reload, conservative refusals, malformed/
  missing/duplicate escaped typed fields (two valid duplicate root logs), opaque
  byte fidelity, absent defaults and relative source/Graph/capture references.
- Publication controls cover occupied/raced destinations, symlink/hardlink
  aliases, reserved storage, temporary collision ownership, partial temporary
  writes, unsupported write/link distinction and cleanup failure after success.
  Existing immutable-Graph publication regressions pass unchanged.
- Actual headless binary integration passed:
  `/tmp/riotbox-1595-binary-smoke.log`. Missing generated source remains unopened;
  a second invocation refuses the occupied destination without changing either
  file. Thin writer tests cover write and flush failure after publication and
  explicitly acknowledge the published destination instead of claiming no write.

An early test fixture tried to bind a newly generated Unix socket; the sandbox
rejected fixture construction, not production admission. Removed that unnecessary
socket fixture; directory/symlink nonregular-input controls remain. No host or
device was inspected and no permission bypass or test skip was introduced.
The first reviewed test rerun also caught an incorrect test-only enum import;
it was corrected before the final29-test pass.

## Review and disposition

Independent Rust review retained one P2: the initial shared publisher erased
preparation-versus-link error provenance and could mislabel an unsupported write
as unavailable hardlinks. Fixed with a private typed stage distinction; both
callers retain ordinary preparation I/O errors and classify only link-stage
Unsupported specially. Injected stage controls and existing Graph tests pass.
The reviewer verified the correction; no other production findings remain.

Independent Spec Guardian/evidence/CLI review found no production defects.
Its requested relative-reference/absent-default/valid-duplicate-root and actual
binary controls are now present and executed. Coordinator self-review covers
correctness, one Core authority, semantic module ownership, CLI isolation,
preservation/error claims and tests; no open P0–P3 findings. Complete local/native
CI remain mandatory PR gates; final exact-head status belongs to PR/Linear
closeout, not an assumed focused pass.

## Boundaries

No new ActionCommand (queue/commit/replay surfaces intentionally unchanged),
action record, receipt, persistent App state, second Session/replay model,
source/DSP recipe or threshold, recording schema or new dependency. Enables
the existing serde_json raw_value feature. Observer has only the inert exhaustive
mode summary arm; this command never constructs it. No actual user Session
migration, source/capture/Holdout/commercial/ignored-attempt access or discovery,
host/device/service observation/runtime, playback, DAW or Windows/TUI expansion.
Generated metadata proof is not source, human, musical, hardness, latency,
endurance or release qualification. RIOTBOX-1041/1036 and original ALSA cause
remain open; RIOTBOX-1589's successful consumed host observation is unchanged.
