# RIOTBOX-1399 Scoped PR Validation

- Date: 2026-08-26
- Work class: workflow and source-boundary maintenance
- Result: source-free normal PR gate and guarded broad phase/release gate

## Outcome

The normal `just ci` path is now source-free. It covers Rust formatting,
tests, linting, sidecar contracts, synthetic audio/observer seams, generated
listening manifests, Decision Log search fixtures, tracked JSON, and diff
integrity. It cannot transitively reach the broad audio-QA recipes or their
registered Development-source generators; mutation fixtures reject either
wiring regression.

The previous broad baseline remains available as
`RIOTBOX_BROAD_AUDIO_QA_ACCESS=registered-development-only just ci-broad`.
Both that entry point and `just audio-qa-ci` fail closed before broad work when
the exact acknowledgement is absent or incorrect. The acknowledgement is an
accidental-access guard, not authorization: Holdout audio, commercial
references, and source-directory discovery remain forbidden.

## Measurements

The legacy `just ci` was stopped fail-closed after 616 seconds. It had already
opened several registered Development sources and had not completed; its top
level contained 24 direct `just` calls and the broad audio body contained 76.

The replacement source-free `just ci` passed in 158 seconds. That is at least
74% less wall time than the incomplete legacy observation. Its top level now
contains six direct `just` calls and its synthetic audio body contains seven.
Recipe count is implementation evidence, not a quality score.

The final guarded `just ci-broad` passed in 715 seconds. Its broad-extra body
contains 23 direct `just` calls and the locked audio body contains 64. The
compact log has SHA-256
`e3795a67089303c958716da87a914cd174f74f5e0ebc6cff336075b65300c1e5`.
The log records one `professional-output-suite-smoke` invocation and one suite
generator invocation, with no formerly independent covered source-pack recipe.

An earlier 610-second integration run reached the same generated suite and
MC-202 pack but stopped because the label-corpus fixture still assumed the old
standalone listening-pack path. The fixture now derives its review path from
the supplied pack manifest and accepts the reused suite and real-source pack
identities. A focused listening-pack reuse check then passed in less than one
second while reusing both the professional WAV pack and Dense performance pack.

## Retained Coverage And Boundary

The source-free gate validates its synthetic exact mix while holding the same
lock used to generate that mutable artifact. The broad gate does not reread
that shared path later. It still runs the retained phase/release baseline,
including the professional suite, its child validators and mutation fixtures,
one separate MC-202 real-source pack, closeout generation, release-demo checks,
source-timing probes, recovery probes, and stage-style stability. The
professional suite's already generated Dense and professional WAV children are
passed into the listening pack instead of rerendered.

The successful broad run used only its explicitly acknowledged registered
Development inputs. It did not authorize or access Holdout audio, commercial
references, or source directories. No cleanup of the reusable `target/` or
`artifacts/` trees was performed.

This change makes PR feedback smaller and source-safe. It adds no audio
mechanism, tuning, musical verdict, source-general evidence, Holdout evidence,
release-quality proof, or Riotbox 1.0 completion claim. Audible slices must
still run every exact product, source, and listening gate required by their
frozen claim.
