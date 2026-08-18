# RIOTBOX-1443 W-30 Pitch-Dive Four-Source Transfer Observation v2

Status: terminal fail-closed pre-candidate
Partition: Development only
Mechanism changes: none
Holdout access: prohibited
Commercial-reference access: prohibited

## Version Reason

The v1 session stopped before candidate rendering or playback because registered
case `freesound_aikighost_19059` declared 120 BPM while the unchanged Rust
timing analysis found 117.90 BPM, outside the existing 1.00 BPM admission
tolerance. v2 replaces only that case. It does not change the pitch-dive
mechanism, thresholds, render order, or listening presentation.

## Exact Source Boundary

Create a new access log before any v2 source read. Open only:

| Case | Family | Path | SHA-256 | Declared BPM |
| --- | --- | --- | --- | ---: |
| `freesound_alastair_pursloe_183441` | dense break | `data/test_audio/external/RIOTBOX-1430/freesound-v3-pool/01_dense_183441.wav` | `b342ee4a9412de14f460c2c295634c53801f2549c71bfc486644a1b02030abc9` | `135.0` |
| `freesound_dabromusic_266735` | dense break | `data/test_audio/external/RIOTBOX-1430/freesound-v3-pool/05_dense_266735.wav` | `b3ee8908b0433e9d286f6174369cfebe78ee928656e52935d1992fdb2dba7c73` | `172.0` |
| `freesound_dr_skitz_353853` | sparse drums | `data/test_audio/external/RIOTBOX-1430/freesound-v3-pool/09_sparse_353853.wav` | `e75e1e6248d07b63ad58b8ee74a35c8cac066db808ef3e5daf256f20a5ba858d` | `120.0` |
| `freesound_jmarcosfer_591426` | sparse drums | `data/test_audio/external/RIOTBOX-1430/freesound-v3-pool/10_sparse_591426.wav` | `a9be617421c8a3b53c6a4f1d400ba80ca5b83fa330482c4da056701927fbccd8` | `94.0` |

Do not discover the containing directory or substitute another source. Reuse of
v1 product-control bytes for the first three cases is allowed only after their
exact source hashes are verified again in the v2 session.

## Unchanged Gates

All RBX-299 render, technical, presentation, listening, and stopping rules from
`docs/plans/riotbox_1443_w30_pitch_dive_four_source_transfer.md` remain
unchanged. In particular, each composite is source context, one second silence,
A repeated twice, one second silence, then B repeated twice. Transfer evidence
may reject but may not tune `w30_pitch_dive_v1`.
