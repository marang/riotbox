# RIOTBOX-1443 W-30 Pitch-Dive Four-Source Transfer Observation v3

Status: completed with four positive transfer observations
Partition: Development only
Mechanism changes: none
Holdout access: prohibited
Commercial-reference access: prohibited

## Version Reason

The v2 replacement case `freesound_jmarcosfer_591426` also stopped before
candidate rendering or playback: its registered 94 BPM disagreed with the
unchanged current Rust timing candidate of 141.07 BPM. v3 replaces only that
case with the already registered, previously qualified tonal Development case
`tonal_rusharp_120`. No further source-pool search is authorized.

## Exact Source Boundary

Create a fresh v3 access log, then open only:

| Case | Family | Path | SHA-256 | Declared BPM |
| --- | --- | --- | --- | ---: |
| `freesound_alastair_pursloe_183441` | dense break | `data/test_audio/external/RIOTBOX-1430/freesound-v3-pool/01_dense_183441.wav` | `b342ee4a9412de14f460c2c295634c53801f2549c71bfc486644a1b02030abc9` | `135.0` |
| `freesound_dabromusic_266735` | dense break | `data/test_audio/external/RIOTBOX-1430/freesound-v3-pool/05_dense_266735.wav` | `b3ee8908b0433e9d286f6174369cfebe78ee928656e52935d1992fdb2dba7c73` | `172.0` |
| `freesound_dr_skitz_353853` | sparse drums | `data/test_audio/external/RIOTBOX-1430/freesound-v3-pool/09_sparse_353853.wav` | `e75e1e6248d07b63ad58b8ee74a35c8cac066db808ef3e5daf256f20a5ba858d` | `120.0` |
| `tonal_rusharp_120` | tonal riff | `data/test_audio/examples/DH_RushArp_120_A.wav` | `ec2a0c930eb338bf81cd5cb4b5fef487e07c140ad40181e1d92b2a0990334e0e` | `120.0` |

Do not discover any source directory or substitute another case. The tonal case
uses `downbeat_seconds = 0.0`. The first three v1 control renders may be reused
only after their exact source hashes are verified again in the v3 session.

## Unchanged Gates

All RBX-299 render, technical, presentation, listening, and stopping rules from
the v1 transfer brief remain unchanged. Each composite is source context, one
second silence, A repeated twice, one second silence, then B repeated twice.
Transfer evidence may reject but may not tune `w30_pitch_dive_v1`.
