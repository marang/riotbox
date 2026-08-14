# RIOTBOX-1436 TR-909 Impact-Pocket Development Qualification

Date: 2026-08-14

Work class: `audible_golden_path_slice`

Result: `terminal_fail_closed`

## Research Transfer And Product Scope

RIOTBOX-1436 applies the preregistered RIOTBOX-1429 `H-LAYER-1` hypothesis to
the existing committed `tr909.set_slam` action. The new
`tr909_impact_pocket_v1` mechanism does not boost, replace, pitch, distort, or
otherwise alter the TR-909 hit. Instead, the competing non-TR-909 bed yields in
a short typed window around the selected kick or snare owner and returns to
unity immediately afterward. This is an `arrangement_impact` and contextual
`drum_punch` claim, not isolated `percussive_hard` evidence.

The action continues to use the established queue, commit, Session/replay,
observer, UI, render-projection, and exact RuntimeMix paths. Decision `RBX-287`
froze the activation policy, owner mapping, envelope topology, constants,
refusal behavior, and Development protocol before source access.

## Qualification Sessions And Access

Three bounded Development sessions were retained rather than concealing QA
harness failures:

1. The first session opened only the registered dense-break source and stopped
   fail-closed because the locality proof used artifact-relative rather than
   absolute transport phase. Its access record conservatively remained at
   `authorized_exact_path_open_started` for that file.
2. The second session verified the dense source and opened the registered tonal
   source. The impact proof passed, but an unrelated dense-only legacy gate
   rejected the tonal intentional-stay-out policy. Its explicit failure exit
   also exposed that the session-level status was not updated from `started`;
   the per-file record still correctly states
   `opened_then_render_failed_closed`. The sparse source was not opened.
3. The final session used the existing controlled-source path with its
   family-specific gates, while reusing the identical impact-pocket proof. It
   verified and rendered exactly the three preregistered Development files and
   completed successfully.

No session performed source-directory discovery or opened Holdout or commercial
reference audio. The DSP policy, profile mapping, constants, and thresholds did
not change after any source result.

## Technical Result

All three registered source families passed the real committed
`BreakReinforce` Slam path, exact callback-block RuntimeMix simulation, source
policy gates, impact locality, and limiter gates.

| Case | Source family | Profile / owner | Delta RMS | Changed frames inside owner window | Peak delta outside | Limited samples |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| `dense_beat03_130` | dense break | `steady_pulse` / kick downbeat | 0.014006 | 6,205 | 0.0 | 0 |
| `tonal_rusharp_120` | tonal riff | `steady_pulse` / kick downbeat | 0.014416 | 6,738 | 0.0 | 0 |
| `sparse_kicksnr_120` | sparse drums | `steady_pulse` / kick downbeat | 0.009931 | 6,722 | 0.0 | 0 |

The slammed TR-909 render is sample-identical between control and candidate.
For the dense review pair, the only changed region is the 129.8 ms owner
window: whole-bar waveform correlation is 0.989684, while correlation within
that window is 0.897008 and the local delta RMS is 0.052758. Relative to the
control, candidate energy in that window changes by -0.59 dB below 150 Hz,
-3.76 dB from 150 Hz to 1 kHz, and -1.90 dB from 1 to 8 kHz. This matches the
declared collision-space function: the competing bed yields most strongly in
the body/mid band while the unchanged kick owns the landing.

These measurements establish correct routing, bounded contrast, and absence of
leakage or limiter substitution. They do not establish musical quality or a
harder perceived landing.

## Human Review Result

The prepared full-mix A/B artifact is sample-exactly assigned as five repeats
of the slammed control followed by five repeats of the impact-pocket candidate.
Each half is 9.210417 seconds; the complete comparison is 18.420833 seconds,
48 kHz stereo PCM16, -18.4 LUFS integrated, with a -3.2 dBFS sample peak.

Audible contributors in both halves are the W-30 source-derived hook, unchanged
TR-909 support, and MC-202 instigator through the exact RuntimeMix and master
limiter path. The source-monitor lane is absent. B differs only by the
collision-local bed envelope.

The bounded full-mix comparison produced no perceptible contrast between A and
B. The technically verified 129.8 ms bed reduction therefore did not make the
Slam land harder in the product context. This is a musician-facing weak result,
not permission to increase the attenuation, window, or another frozen scalar
after hearing the output.

The mechanism closes fail-closed. Its implementation is removed from the
product branch before merge, while the preregistered contract and negative
evidence remain durable. No Holdout access or additional human playback is
authorized for `tr909_impact_pocket_v1`.

- `human_verdict: technically_ok_but_musically_weak`
- `quality_proof: false`
- `hardness_proof: false`
- Holdout access remains unauthorized.

## Evidence Identities

- Frozen Development contract: `171f21e2ddaafe11631f3022627a93bc6e1bdc328d9f85ce015d42f3a35f84ed`
- First failed-session access record: `0ec0f6754b3d0a616483e926fca45f26fda1d0b408c5a4a3df21741854510559`
- Second failed-session access record: `144095a7d048bd3fff65871ee379d24a4df2437db87358173580727d439750cb`
- Completed access record: `d8b2f8dcbda73e38c2a3e186272a9b29b77dcb83d83db1dbc23e0ba803982b51`
- Completed technical report: `1c3eaf01b43953f57e1500b8090042240aad015431da806f3fcb8b475df8a061`
- Looped A control: `6628e890bd9df74f0e84fd2cb228bce0c223a639075d138b451ffdb128c0f999`
- Looped B candidate: `ed0dac69bc818ef3b4842c49ad51b7ab23a4479047e904f9a472c6076ba32207`
- A-then-B comparison: `4df3f75f165dd4776e2adae098045978c6b1ad66ed187213e457f2c2b1049c04`
- Structured human verdict: `193938fe259f622789c6f37b620d7ae775bfb2283ae06592902248f3f0e7db56`
