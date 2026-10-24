# Dependency security review — 2026-10-24

Maintenance tracked in LAB-389 / PR #1711. Weekly Cargo/Actions Dependabot,
immutable Action pins and scheduled/on-PR cargo-audit are included.

## Remediation

- Ratatui 0.30.2 resolves lru 0.18.5 and removes paste. The old lru 0.12.5
  advisories (RUSTSEC-2026-0002/0253) and paste maintenance warning no longer
  occur in the module graph. Crossterm remains 0.29.
- Upstream dawproject 0.11.0 and current main still require zip ^8.6.0.
  A narrowly vendored Cargo patch preserves the exact published library except
  for propagating ZIP 9's fallible filename decoding. It requires zip >=9.0,
  resolving only zip 9.1.0, addressing RUSTSEC-2026-0367/0368/0369.
  `vendor/dawproject/PATCH.md` records provenance, archive hash, scope and removal
  condition. A CI source-hash check rejects any additional library delta.
- Source-free watchdog regressions now accept both direct SIGKILL (-9) and
  shell-encoded SIGKILL (137). They still assert the generated child is no
  longer running and cleanup is verified. Missing-command status remains 127;
  preserved nonempty stderr is required without assuming GNU-specific wording.
  Production process-group ownership and cleanup code are unchanged.

These are dependency findings, not proof the application exercised the affected
ZIP extraction operations. No advisory is suppressed.

## Evidence and boundaries

`cargo check --locked -p riotbox-app` passes with the actual installed ALSA
shared library and workspace-local pkgconf. Cargo tree confirms a single zip
9.1.0 and lru 0.18.5, with no paste. An OSV batch query of all 278 registry
packages in the candidate lockfile returns no findings. Three synthetic ZIP 9 integration tests pass (valid names and bytes, legacy
CP437 names, invalid container errors). The existing DAWproject suite passes
33 tests locally; four additional export tests reach the absent xmllint schema
tool. Full cargo-audit and Rust/Python CI remain required on this candidate
before merge.

The full local process suite was attempted but cannot access the sandbox's
missing /proc filesystem; this does not establish a process regression or a
passing cleanup result. Existing production source, audio-source acquisition,
audio playback, real device behavior, human listening and release qualification
are outside this maintenance evidence. No release readiness is claimed.
