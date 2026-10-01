# RIOTBOX-1416: verified broad review, 2026-07-19

Classification: maintenance/regression documentation. Recorded: 2026-10-01.
Historical freshness-check baseline: `e3d70e1d451ee6cdf3ff46b9bc0f3ad7de6c0930`.
Current-main evidence baseline: `140da245baba7f32410665c5b976198391468ecd`.
Source: [Linear verification document](https://linear.app/riotbox/document/riotbox-broad-codebase-review-verification-2026-07-19-100ae8ef5374),
created 2026-07-19, last updated that day. This preserves its verified
classification, not its external raw review or a newly performed broad audit.

## Provenance and scope

The original external whole-repository review used Kimi 3 High and four
crate-focused agents. Its findings were subsequently freshness-checked against
the historical main baseline, canonical specs/decisions, existing reviews and
active/archived ticket history before follow-ups were classified. The then-active
RIOTBOX-1402 working tree was not modified by that verification.

The recorded assessment was that dependency direction, realtime separation,
typed errors, Core/Session ownership, replay and tests were sound foundations;
verified risks were concentrated in contract edges and known scaffolds overtaken
by later guardrails. This is a point-in-time assessment, not proof of absence of
other defects or a present musical/release verdict. Historical classifications
below remain separate from the subsequently completed work.

## Verified follow-up mapping

Priority and work class are the July verification's classification. The final
column reflects repository archives checked on 2026-10-01; it does not rewrite
the historical review or claim these implementation tests were rerun here.
Ticket links are retained even when the completed Linear issue has been deleted;
the corresponding local archive is the durable terminal-state evidence.

| July priority / class | Ticket | Verified July finding | Later disposition |
| --- | --- | --- | --- |
| High / maintenance/regression | [1407](https://linear.app/riotbox/issue/RIOTBOX-1407) | Live Space transport bypassed Action Lexicon commit/replay ownership. | [Done, #1430](../archive/linear_issues/RIOTBOX-1407.md): immediate existing play/pause actions own the toggle. |
| High / audible_vertical_slice | [1408](https://linear.app/riotbox/issue/RIOTBOX-1408) | Accepted synthetic W-30 resample scaffold conflicted with later product-output guardrails. | [Done, #1375](../archive/linear_issues/RIOTBOX-1408.md): capture-backed grain and fail-closed silence. Human verdict was technically acceptable but musically weak; no demo-ready promotion. |
| Medium / maintenance/regression | [1409](https://linear.app/riotbox/issue/RIOTBOX-1409) | Relationship endpoints, Scene mapping and decode profile retained stringly identity. | [Done, #1537](../archive/linear_issues/RIOTBOX-1409.md): typed identities and explicit V1-compatible migration. |
| Medium / maintenance/regression, benchmark | [1412](https://linear.app/riotbox/issue/RIOTBOX-1412) | Coherent snapshots were already solved; unchanged W-30 publications still paid repeated sample-read/copy cost. | [Done, #1527](../archive/linear_issues/RIOTBOX-1412.md): measured first, then callback-local unchanged-revision fast path. No device/xrun qualification. |
| Medium / contract_enabler | [1334](https://linear.app/riotbox/issue/RIOTBOX-1334) | Existing Sidecar ticket expanded for protocol/provenance/deadline/request-less-error/CWD trust boundaries. | [Done, #1432](../archive/linear_issues/RIOTBOX-1334.md); an existing ticket, not a duplicate opened by the review. |
| Medium / contract_enabler | [1410](https://linear.app/riotbox/issue/RIOTBOX-1410) | Source-monitor control updates could retain old PCM ownership after source replacement. | [Done, #1428](../archive/linear_issues/RIOTBOX-1410.md): explicit complete replacement, atomic prepared snapshot and control-side retirement. |
| Medium / maintenance/regression | [1411](https://linear.app/riotbox/issue/RIOTBOX-1411) | UI/Jam textual includes obscured semantic ownership; allowlist-only closeout was insufficient. | [Done, #1555](../archive/linear_issues/RIOTBOX-1411.md): semantic modules preserve existing behavior and test identities. |
| Low / maintenance/regression | [1414](https://linear.app/riotbox/issue/RIOTBOX-1414) | Broadcast effects, policy→view dependency and repeated boundary ordering weakened auditability. | [Done, #1535](../archive/linear_issues/RIOTBOX-1414.md): exhaustive dispatch, Session-owned timing and shared boundary rank. |
| Low / maintenance/regression | [1415](https://linear.app/riotbox/issue/RIOTBOX-1415) | Telemetry poison panics and repeated shared dependency ownership were hygiene debt. | [Done, #1541](../archive/linear_issues/RIOTBOX-1415.md): observable sticky recovery and workspace-owned unchanged dependencies. |

The July recommendation was 1407 then 1408, followed by roadmap-based
reassessment of 1409/1412/1334/1410, with 1411/1414/1415 in maintenance cadence.
This records that recommendation, not a new execution order after those closures.

## Existing, duplicate, deferred and branch-local classifications

- [RIOTBOX-1327](../archive/linear_issues/RIOTBOX-1327.md) had already delivered
  coherent revisioned snapshots. 1412 was specifically the separate cost
  measurement/unchanged-revision optimization, not a new coherence fix.
- 1334 already existed; the review expanded its scope instead of creating a
  second Sidecar issue. [RIOTBOX-1337](../archive/linear_issues/RIOTBOX-1337.md)
  separately owned the library CLI include shell, not the UI/Jam scope of 1411.
- [RIOTBOX-1413](../archive/linear_issues/RIOTBOX-1413.md) was created in triage,
  then canceled as intentionally deferred after [747](../archive/linear_issues/RIOTBOX-747.md)/
  [751](../archive/linear_issues/RIOTBOX-751.md) were found. Session v1 keeps
  stable MC-202 persisted labels behind typed behavior helpers; a wire migration
  belongs to Session vNext/new semantics, not an unsolicited standalone fix.
  See [the migration plan](mc202_typed_contract_migration_plan_2026-05-10.md),
  [Session contract](../specs/session_file_spec.md) and accepted RBX-028.
- The 73-versus-72 include-count failure was local to the then-uncommitted 1402
  tree, not verified main. It remained that branch's review obligation and did
  not receive a separate bug ticket. It is not claimed to exist on today's main.
- The W-30 tap was an intentional documented MVP scaffold, not an accidental
  hidden oscillator. Later no-placeholder/product-primitive guardrails made its
  retirement necessary. [The 1408 review](riotbox_1408_source_backed_w30_resample_review_2026-07-21.md)
  retains its technical proof and weak human verdict separately. Replacing the
  scaffold did not establish satisfactory musical output.

## Current-main anchors and canonical follow-through

These file:line references are pinned to the current-main baseline above, not
claimed as locations in the older July tree. They show where the bounded
follow-through lives; they do not constitute a new exhaustive implementation audit.

| Topic | Current-main file:line evidence | Owning evidence / contract |
| --- | --- | --- |
| Transport | `crates/riotbox-app/src/cli/event_loop.rs:93`; `jam_app/transport.rs:50` in the same crate | [Action Lexicon](../specs/action_lexicon_spec.md), [replay](../specs/replay_model_spec.md), 1407 archive. |
| W-30 resample | `crates/riotbox-app/src/jam_app/projection/w30_resample.rs:129` | [Audio Core](../specs/audio_core_spec.md), 1408 review: capture audio or explicit unavailable/silent routing. |
| Typed identity | `crates/riotbox-core/src/source_graph/relationship_identity.rs:16`; `session/source_decode_profile.rs:30`; `session/scene_source_binding.rs:12` in the same crate | [1409 review](riotbox_1409_typed_source_identity_2026-09-22.md), [Source Graph](../specs/source_graph_spec.md), Session contract. |
| Snapshot cache | `crates/riotbox-audio/src/runtime/w30_preview_snapshot.rs:169` | [1412 measurements and limits](riotbox_1412_w30_snapshot_cost_2026-09-22.md), Audio Core. |
| Source replacement | `crates/riotbox-audio/src/runtime/source_monitor.rs:146` | Explicit replacement and control-side retirement, Audio Core and 1410 archive. |
| Sidecar trust | `crates/riotbox-sidecar/src/client.rs:243`; `client.rs:333`; `path.rs:7` in the same crate | [Technology Stack](../specs/technology_stack_spec.md), Source Graph and 1334 archive. |
| Semantic UI/CLI | `crates/riotbox-app/src/ui.rs:3`; `cli.rs:4` in the same crate | [1411 review](riotbox_1411_semantic_ui_tests_2026-09-30.md), [1337 review](riotbox_1337_semantic_cli_2026-09-30.md), [module policy](../engineering/module_policy.md). |
| Side effects / layering | `crates/riotbox-app/src/jam_app/side_effects.rs:38` | [1414 review](riotbox_1414_commit_ownership_2026-09-22.md), Action Lexicon and Session contracts. |
| Telemetry / dependencies | `crates/riotbox-audio/src/runtime/telemetry.rs:79`; `Cargo.toml:15` | [1415 review](riotbox_1415_runtime_telemetry_2026-09-22.md), Audio Core and [Rust guidelines](../specs/rust_engineering_guidelines.md). |
| MC-202 deferral | `docs/specs/session_file_spec.md:392`; `docs/reviews/mc202_typed_contract_migration_plan_2026-05-10.md:133` | Session v1 label contract and RBX-028; canceled 1413 is not an open defect. |

## Documentation verification and limits

Verified the original Linear document and terminal archive mapping, existing
review/spec destinations and current-main source anchors. No implementation,
audio behavior, schema or product contract changes are included. No new technical
decision is accepted, so the Decision Log is unchanged. The 1402 work context
is not edited or checked out; this slice uses its own Linear-first docs branch.

All 30 distinct local link targets and 18 pinned source/document anchors check
successfully. Fresh full source-free just ci exits zero, without compiler/Clippy
warnings or errors; final formatting/diff and zero-include guard pass. Sequential
solo branch review and self-review find no remaining artifact defect. Exact-head
native CI, fresh reviews, merge/main synchronization and archive/cleanup remain
separate obligations; historical test/listening claims are not freshly earned.

No new source access, source-directory discovery, Development/Holdout/commercial
audio, DAW/device/playback or human listening. No subagents were used to record
this artifact; original historical delegation is provenance only. Past audio
proof/listening is referenced, not replayed or promoted into fresh evidence.
This documentation closes the missing durable review record, not a new broad
audit, source/musical/hardness/human/release pass or P023 completion.
