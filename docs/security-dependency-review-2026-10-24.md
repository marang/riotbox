# Dependency security review — 2026-10-24

Maintenance tracked in LAB-389. This branch adds weekly Cargo/GitHub Actions
Dependabot updates, a RustSec CI gate, immutable Action pins and a compatible
Cargo.lock refresh. It does not establish release or audio-runtime readiness.

## Remaining advisory findings

An OSV query of all 214 registry packages in the updated lockfile returned:

| Package | Finding | Upgrade boundary |
| --- | --- | --- |
| lru 0.12.5 | RUSTSEC-2026-0002 and RUSTSEC-2026-0253; GHSA-rhfx-m35p-ff5j is an overlapping alias | Ratatui 0.29 requires lru 0.12. Ratatui 0.30.2 / ratatui-core 0.1.2 uses lru 0.18; this is a candidate migration, not verified app compatibility. |
| zip 8.6.0 | RUSTSEC-2026-0367, RUSTSEC-2026-0368, RUSTSEC-2026-0369 | Patched zip starts at 9.0.0. Pinned dawproject 0.11.0 requires zip ^8.6.0. Adding zip 9 directly would leave the vulnerable copy. An upstream update or a separately reviewed dependency patch is required. |
| paste 1.0.15 | RUSTSEC-2024-0436 | Unmaintained-package warning through Ratatui, not a demonstrated exploit. |

These are dependency-version findings. Exploitability of each application path
has not been established. Do not suppress them or treat the compatible update
as complete security remediation. The new audit job is expected to remain red
until the applicable findings are resolved.

Primary records:

- https://rustsec.org/advisories/RUSTSEC-2026-0002.html
- https://rustsec.org/advisories/RUSTSEC-2026-0253.html
- https://rustsec.org/advisories/RUSTSEC-2026-0367.html
- https://rustsec.org/advisories/RUSTSEC-2026-0368.html
- https://rustsec.org/advisories/RUSTSEC-2026-0369.html
- https://rustsec.org/advisories/RUSTSEC-2024-0436.html

## Validation and limitations

`cargo test --locked -p riotbox-core -p riotbox-sidecar` passed (614 tests,
one existing ignored test). Full workspace tests were attempted but require
missing ALSA development libraries/pkg-config. No audio playback or source
material was accessed. The cargo-audit executable could not start because the
local environment lacks /proc/self/exe; the OSV/RustSec database query was used
instead. Workflow syntax was checked with actionlint; GitHub CI has not run.

Publication proceeds through a dedicated pull request after GitHub App write
permissions were granted. Complete GitHub CI, dependency migration and review
are still required before merge; no release readiness is established here.
