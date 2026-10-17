# Preset/Undo conflict barriers — RIOTBOX-1605

Date: 2026-10-17. Maintenance/regression for P023 recall reliability and the
single Core/Session replay truth. Follows the reproduced checkpoint1603 defect.

## Result

The existing Core command predicate now declares `PresetActivate` a writer of
SourceMonitorMode and Mc202Lane. Presets overwrite monitor mode and macro touch,
so an older typed snapshot may not overwrite those newer values. Presets remain
non-undoable; they do not write `last_fill_bar` and are not Fill-window barriers.
No preset value, DSP, source selector, recipe, public interface, schema, queue,
history model, persistent App state or new replay policy was introduced.

Musician action: activate a preset, then request Undo. The instrument preserves
the preset's overlapping state instead of restoring an older monitor/touch value
that would diverge from replay. A newer supported action after that preset can
still be undone. This is state/restore correctness, not a new musical pass.

## Compatibility contract

RBX-466 and the Replay/Session specs clarify an existing V2 requirement, not a
new chronology: newer non-undoable writers block overlapping typed domains.
Both live selection and Core V2 marker validation use the same predicate.

- Future live Undo honors the writer for all three policies.
- Already-invalid V2 cross-preset markers fail closed with Core
  `InvalidUndoTargetRelation`; App load retains its existing `InvalidSession`
  wrapper and explicit upgrade its existing `InvalidHistory` wrapper.
- No file/action/status rewrite, automatic policy upgrade, history adoption or
  inference from result prose occurs. Recovery/adoption needs a separate explicit
  decision rather than silent reinterpretation of divergent materialized state.
- Historical Legacy Core stack/serialization remains unchanged. Explicit
  Legacy→V2 upgrade refuses an invalid overlapping relation without mutation.
- A live Legacy candidate in an unrelated domain remains subject to its existing
  prospective global-stack admission; it does not gain V2 domain-stack semantics.

Blocked-Undo nonmutation evidence uses valid existing snapshot contracts. The
existing no-target normalization of invalid/missing snapshots remains permitted;
this repair does not promise that every `None` Undo is globally mutation-free.

## Test-first evidence and durable controls

The public App test
`jam_app::tests::preset_undo_conflicts::newer_preset_blocks_older_monitor_undo_before_session_or_id_mutation`
actually executed1 test and failed before production changes: monitor Blend →
V2preset → Undo returned a successful stale marker instead of None. After the
predicate correction the same1test passed. Logs:

- `/tmp/riotbox-1605-preset-monitor-red.log`, SHA-256
  `288dad4b13e12245a24804ec3190dc3a00671df541fe02a651a014159fd72745`.
- `/tmp/riotbox-1605-preset-monitor-green.log`, SHA-256
  `50b3c3afd605550b5591441ee4a8d3e0bfcd953a5a3f75af681d295a35006ecb`.

The7 executed App tests cover both preset versions and all live policies,
overlapping monitor/MC202-touch snapshots, exact Session/runtime/views/queue/ID
nonmutation for valid blocked histories, post-preset monitor/MC202 Undo and
live/replay parity, unrelated Fill/V2 and Legacy-admission controls, ordinary
save/load and payload-plus-suffix restore, invalid historical V2 load refusal,
unchanged input bytes and failed-target restore nonmutation. The historical bad
marker fixture explicitly models the old demonstrated relation; the corrected
live producer is not claimed to create it.

V1's actual monitor default is Blend, V2's Riotbox. Initial expanded tests wrongly
assumed both were Riotbox and attempted a no-op Blend control under V1. Those
fixture expectations were corrected to the canonical unchanged preset definitions
and a genuinely different Source control; no preset/product behavior was changed
to satisfy them. Only the subsequent passing run counts as coverage evidence.
Final App log `/tmp/riotbox-1605-app-conflicts-final.log`, SHA-256
`f777d42d92ef9f1fe9be7d9e68ba29599217e53533b5c1dbff0cbb4f85ef810c`.

The5 executed Core tests directly exercise planner and explicit upgrade:
both overlapping domains×both presets, unchanged same-input Legacy interpretation
and serialization, typed upgrade refusal/nonmutation, valid Fill crossing and
upgrade, and valid newer post-preset Undo that cannot reopen an older snapshot.
Log `/tmp/riotbox-1605-core-conflicts.log`, SHA-256
`5d269f8ea12c69640f2671969fe13b19eba2c600450aac022264a71f3d963462`.

Broader `cargo test -p riotbox-core -p riotbox-app --lib` passed995App tests and
570Core tests,0failed, with2existing manual release-mode benchmarks ignored.
Log `/tmp/riotbox-1605-core-app-lib.log`, SHA-256
`b7e5af55b952ea0b7ea66cfcfa45fe6d2677026d9618472f07fb3a6f5cd1924c`.
The new conflict tests use generated metadata or fresh owned temporary Session
files with no graph/source/capture hydration. The broader existing suite also
hydrates generated temporary WAV/Graph/capture fixtures, never actual user,
Holdout or commercial material. No host runtime or playback was used.

## Review and remaining gates

Independent Spec/Compatibility review retained0 findings and confirmed that no
new replay-policy version or user recovery choice is required for this bugfix.
Maintainer/Adversarial/Rust review, coordinator final self-review and full
exact-head local/native CI remain separately recorded in PR/Linear before merge.
The large existing command enum and planner test facade were reviewed by changed
responsibility; each new regression family has an ordinary semantic test owner,
not another policy system or mechanical file shard.

Final Spec/Evidence review found oneP3 wording ambiguity: an unqualified
no-hydration sentence could be read as covering the broad existing suite. It was
restricted to the new metadata-only conflict tests, with generated-fixture
hydration in the broad suite stated explicitly. No production change; the
follow-up self-review checks the corrected claim and original log evidence.

1604's independently gated RBX-465 must precede466 after Main integration. No
prior entry is renumbered or silently altered.1606drum phase and1607snapshot
provenance remain separate unfixed findings, not waived by these passing checks.

No actual user-file migration, source/capture/Holdout/commercial/ignored-material
access/discovery, host/device/service/TTY observation/runtime, playback, DAW or
Windows/TUI expansion. No human/musical/source-general/hardness/physical-latency/
endurance/release claim.1589 remains passed/consumed;1041/1036/originalALSA stay open.
