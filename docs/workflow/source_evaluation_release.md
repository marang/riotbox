# Source evaluation alpha release v1

Owner: RIOTBOX-1609. Decision: RBX-468. First version: `0.1.0-alpha.1`.
This supplements, and does not waive, the normal PR and audio-QA gates.

## Scope and authority

The first published Riotbox distribution is a **source-only evaluation alpha**
under the existing source-available LICENSE, for local private non-commercial
evaluation. There is no prior release to maintain. It exposes the existing
prototype, not a new mechanism, stable Session-format guarantee, universal source
quality, typed hardness, or stage-ready instrument.

Roadmap §13 and DoD §17 explicitly describe 1.0/stage readiness. RIOTBOX-1039
remains that release-cut anchor; 1038's full beta matrix, 1041's real-device
soak/latency/endurance, and 1036's complete DAW export are not automatically
prerequisites for this distribution. PRD §6.2 excludes full DAW export and
polished cross-platform packaging from MVP. No platform binaries, audio examples,
or installer are promised. Linux is the locally verified build environment;
Windows Sidecar CI is not Windows instrument qualification.

The [normal release regression gate](github_pr_ci.md#pr-and-broad-audio-qa-gates)
still applies: source-free `just ci`, **authorized `just ci-broad`**, applicable
output-path regressions, reviewed exact artifacts and native CI. Calling this
an alpha does not excuse failed gates. New musical claims still require their
own source and listening qualification.

## Evidence and limitations

- RIOTBOX-1475 reconciled all six families, including Dense, for its historical
  bounded P023 demo scope. Its dated review is not fresh current-build listening,
  a 1.0 pass, or permission to reopen its audio.
- Source-free tests and Development regression establish technical compatibility,
  not human taste, Holdout generalization, physical host safety or endurance.
  No new listening is solicited here.
- RIOTBOX-1606 preserves recipes and fixes scheduling; the existing listening
  technical-correction rule applies, not automatic verdict transfer.
- RIOTBOX-1589's generated virtual-output evidence stays historical and consumed.
  No fresh host, loudspeaker, DAW, device or service access is authorized here.
- Known RIOTBOX-1607: malformed/edited snapshot payloads may carry provenance
  after their restore cursor. Ordinary Save was not shown to produce this.
  Do not trust edited/third-party snapshots; complete recovery-input integrity
  is not claimed. Full export/arrangement and physical live qualification remain
  bounded by their existing contracts and open issues.

## Frozen Development access

The user authorized one `just ci-broad` invocation on 2026-10-24 after reviewed
frozen preflight, with exactly the seven entries in
`docs/benchmarks/source_evaluation_alpha_development_access_v1.json`.
No source-directory discovery, substitution, Holdout/commercial access, download,
playback or output-device initialization is permitted.

This is legacy broad-regression registration, not RIOTBOX-1428 Stage A. Existing
Stage-A protocols, registry versions, thresholds and algorithms remain immutable.
Before original-file access, reject protected case/path identities, symlinks,
nonregular files, hardlinks and all available expected-hash collisions using
tracked protected metadata (the union of registries v1, v2 and v3), never
protected audio. Five historical full-source
hashes must match. Two explicitly named legacy entries have no historical full
hash in inspected records: their single exact-file hash enrollment is separately
authorized, not claimed as a pre-existing hash pass. Immediately reject a
protected hash collision before decoding/rendering.

Read each original at most once, verify bounded size and stable file identity,
and stage accepted bytes only into a fresh owned validation worktree. Freeze all
seven resulting identities before the sole broad invocation. Keep a bounded
metadata access receipt and complete command log. On any boundary/identity
failure stop closed: no substitution, retry, tuning or silent policy revision.
Do not delete existing QA outputs. Broad recipes may repeatedly read the seven
staged copies and their own generated fixtures; one invocation is not one
internal read. Clear external source/handoff environment overrides first.

## Candidate, package, and publication gates

1. Linear In Progress before the release branch. Consistent workspace and four
   local lockfile versions; no unrelated dependency updates.
2. Review this contract, access policy and helpers before Development access.
   Synthetic helper tests never open real sources or initialize audio.
3. Clean committed candidate: full source-free `just ci`, the sole approved
   guarded `just ci-broad`, vendor source/license verifier. Preserve exact
   candidate commit/tree, commands/results and log hashes.
4. Deterministic source tar.gz from the **Git commit**, not the filesystem:
   Cargo.lock, root LICENSE, vendor correction and both upstream licenses.
   No ignored sources, sessions, credentials, generated review audio, binaries
   or build output. The 18 tiny committed WAV identity-test placeholders are
   not recordings or musical demos and remain part of the test suite.
5. Extract into a fresh owned directory; verify structure, required files,
   workspace/lock versions and vendor correction. Run
   `cargo build --locked --release -p riotbox-app --bin riotbox-app` and only
   `--help` (currently a nonzero usage exit before runtime initialization).
   This proves packaging/parser behavior, not host audio.
6. Independent branch review, self-review, PR, green native Rust/security checks,
   no unresolved relevant findings, then merge. Observe existing Windows CI
   without expanding support claims.
7. Merged commit must have the exact reviewed candidate tree to reuse candidate
   tests. Any code/recipe/policy change invalidates reuse. Consumed one-shot
   authorization never permits retry. Verify merged-main workflows/version,
   synchronize refs, and rebuild/verify the final merged-commit source package.
8. Only then tag exact commit as `v0.1.0-alpha.1` and publish a GitHub prerelease
   with reviewed notes. Attach source tar.gz, a JSON evidence manifest binding
   tag/commit/tree, gate commands/results and workflow URLs, and SHA256SUMS for
   both payload assets. No Development audio or derived review packs.
9. Download published assets freshly; verify checksums, source contents, tag,
   prerelease status and workflow results. Never silently replace a published
   asset or move a tag: corrections require a new reviewed release/version.
   Finish Linear, sync main and clean only completed owned branches/worktrees.
   Keep RIOTBOX-1608 Done and retained.

Evidence belongs in release assets and PR/Linear updates: a source commit cannot
contain its own final hash. Missing gates remain blockers, never forecast passes.
