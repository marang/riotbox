# Source-ingest byte identity — RIOTBOX-1559

Date: 2026-10-03. Base: `942a8a4412028ce626faae6d8d93c7831af2e7df`.
Classification: maintenance/regression of the P023 source-ingest boundary.
The concrete risk is binding source metadata, musical features and timing to
different WAV bytes, invalidating subsequent source-backed decisions.

## Change and contract

The Python provider now hashes and decodes one captured byte buffer. Rust
enrichment reads through the existing bounded regular-file reader, matches the
encoded-WAV SHA-256 to the returned Source Graph, then decodes that same buffer.
Encoded bytes are released before timing analysis. A mismatch returns an
explicit ingest error before graph mutation, manual-grid installation, hook
analysis or Session/Graph publication. Existing restore checks still apply.

RBX-429 and Source Graph §6 own this boundary. Source path/symlink policy,
RBX-425 resource limits, analysis algorithms, schemas and public ingest APIs
remain unchanged. There is no new Action, queue path, app-local state,
persistence model, runtime/DSP mechanism or musical claim. The existing error
surface reports failure; the musician's previous saved state remains available.

## Reproduction and verification

- Python public `analyze_source_file` request: after a complete read, atomically
  replace a generated loud mono WAV with a distinct silent stereo WAV. Before
  the fix, the original hash accompanied the replacement's sample rate
  (`16000 != 8000`). After the fix, hash, provenance, source ID, metadata,
  Source Map energy and phrase features all describe the original buffer.
- Actual App ingest with the real Python provider/protocol: replace the generated
  source after provider analysis and before its reply. The original code failed
  the expected rejection assertion. The fixed code rejects even a one-bit PCM
  change with identical format/length. External and embedded saved Sessions
  remain byte-identical; the external alias remains byte-identical. Restoring
  the original WAV reloads the same Session, graph and PCM cache. New external
  and embedded destinations create neither output directory nor saved files.
- Focused App `ingest` suite: 25 passed. `live_source_timing`: 8 passed, including
  unchanged-source timing, explicit confirmation and manual-grid persistence.
  These filters overlap; their counts are not additive.
- Python contract suite: 3 passed. Formatting and diff whitespace checks passed.
- Full source-free `just ci`: passed, including Rust and Python suites,
  synthetic audio/observer/manifest contracts and strict all-feature Clippy.

Transient evidence: `/tmp/riotbox-1559-python-{red,green,suite}.log`,
`/tmp/riotbox-1559-ingest-{red,green,tests}.log`,
`/tmp/riotbox-1559-timing-tests.log`, `/tmp/riotbox-1559-ci.log`.

## Branch review

Two independent reviewers retained no P0–P3 findings: Spec and Evidence Auditor,
and Adversarial Implementation Reviewer with Rust/domain and compatibility
focus. They traced complete read/enrichment/publication paths, checked the
regressions and RED/GREEN logs, and independently reran the Python tests and
diff checks. Rust/full CI execution belongs to the coordinator. The Python
implementer is not credited with independent review of their own changes.

Coordinator integration and short self-review also found no remaining findings.
The new regression module has one semantic owner, imports are explicit, the
small production edit adds no facade state/dependency, and the existing source
reader owns admission. No module extraction is needed for this bounded repair.

## Limits

Matching captured-buffer hashes are not an atomic filesystem snapshot and
cannot guarantee later file immutability. Subsequent hydration can still reject
changed source audio. Existing graphs are not silently migrated or retrospectively
qualified. Python allocations, aggregate memory and I/O deadlines remain outside
this repair. No new dependency, source library access, Development/Holdout or
commercial read, host device, DAW, playback or human/musical qualification was
required. The five-branch current-state audit was completed in RIOTBOX-1558;
this is the first substantive slice after that checkpoint.
