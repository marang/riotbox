# Temporary dawproject dependency patch

Upstream: https://github.com/bkbkb-net/dawproject, release 0.11.0.
The crates.io archive SHA-256 is `dfc710e130791a1525972bc45184f4eccf5c23d647c87c75db03948f9b996dcf`.

The dependency requirement changes: zip 8.6.0 -> 9.0.0 with the same
features. The reader additionally propagates ZIP 9 filename-decoding errors rather than
assuming names are infallible. All other library source, README and dual-license
texts are copied unchanged from that published archive. upstream-source-sha256.json records the
original source hashes. Upstream tests/assets/tools are not vendored; Riotbox's
existing synthetic archive, XML schema, receipt and replay tests validate its
actual integration. No private audio or generated binary is included.

Upstream 0.11.0 and current main still require zip ^8.6.0, which cannot resolve
to the patched 9.x release. The root Cargo patch prevents a second vulnerable
zip copy; a direct zip dependency would not. Remove this patch when an upstream
release supports patched zip and passes the same integration gates.

Tracks LAB-389 and RUSTSEC-2026-0367, RUSTSEC-2026-0368, RUSTSEC-2026-0369.
