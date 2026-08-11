# RIOTBOX-1431 Agent Context Modularization

Date: 2026-08-11

Work class: `maintenance/regression`

## Outcome

Riotbox now loads a small always-on agent kernel and routes task-specific detail
through one-level modules. The change alters documentation ownership and lookup
only: no runtime, audio, product, schema, benchmark JSON, source partition,
algorithm, or threshold changed.

The mandatory bundle is `AGENTS.md` plus the `riotbox-development` and
`riotbox-rave-punk-production` entry skills. It fell by more than 70 percent in
both bytes and words, well beyond the required 35 percent.

## Reproducible Size Report

Measure the mandatory bundle with:

```bash
wc -w -c \
  AGENTS.md \
  .codex/skills/riotbox-development/SKILL.md \
  .codex/skills/riotbox-rave-punk-production/SKILL.md
```

| Measurement | Bytes | Words |
| --- | ---: | ---: |
| ticket-recorded baseline | 47,936 | 6,668 |
| actual branch baseline after RIOTBOX-1430 | 48,762 | 6,777 |
| RIOTBOX-1431 result | 14,098 | 1,810 |
| reduction from ticket baseline | 70.59% | 72.86% |
| reduction from actual branch baseline | 71.09% | 73.29% |
| acceptance ceiling | 31,158 | 4,334 |

The small baseline difference is intentional and disclosed: RIOTBOX-1430 added
the source-acquisition anti-overengineering guardrail before this branch began.

The split documents were also constrained against size inflation:

| Document family | Before | After | Change |
| --- | ---: | ---: | ---: |
| workflow, bytes | 47,972 | 47,037 | -1.95% |
| workflow, words | 7,433 | 7,106 | -4.40% |
| audio QA, bytes | 160,133 | 159,981 | -0.09% |
| audio QA, words | 20,486 | 20,216 | -1.32% |

The percussive-force entry document fell from 78,880 to 18,716 bytes and from
10,690 to 2,635 words. Its complete three-file family is 81,263 bytes because
the two exact-content modules add explicit authority and navigation headers.
That 3.02% storage increase buys a 76.27% reduction for normal typed-hardness
work without weakening or rewriting the frozen research material.

## Ownership And Routing Audit

| Task | Entry | Conditional owner |
| --- | --- | --- |
| architecture/product-spine change | `AGENTS.md` | relevant spec routed by `docs/README.md` |
| audible character, pattern, loop, or gesture | `riotbox-development` + `riotbox-rave-punk-production` | taste rubric plus audio-QA router |
| wrong, silent, identical, or weak sound | `riotbox-development` | `references/audio-output-qa.md` |
| source acquisition, holdout, fallback, or commercial-reference boundary | `riotbox-development` | `references/source-evidence-boundaries.md`, then the active audio-QA contract |
| human playback or verdict | `riotbox-listening-review` | `audio_qa/listening_review.md` |
| PR, review, or CI | workflow core | `workflow/github_pr_ci.md` |
| Linear issue/project lifecycle | workflow core | `workflow/linear_lifecycle.md` |
| archive, deletion, or branch cleanup | workflow core | `workflow/archive_cleanup.md` |
| long commands, Decision Log, or subagent context | workflow core | `workflow/context_hygiene.md` |
| typed percussive hardness | percussive-force core | research evidence or Stage-A design history only when needed |
| frozen Stage-A execution | exact Decision ID plus versioned benchmark JSON | never explanatory prose as a second execution authority |

All skill references are one level deep. References route back to canonical repo
contracts; they do not chain to more skill references.

## Exact Decision Lookup

The existing bounded helper now treats `RBX-NNN` and historical suffixed IDs
such as `RBX-015a` as exact IDs and returns only that delimited heading block:

```bash
just decision-search "RBX-252"
```

Free-text queries retain the previous bounded fixed-string/term search. No
semantic index, memory service, or second decision store was added. The fixture
recipe covers numeric, suffixed, delimited, final, missing, wildcard, and
shell-metacharacter cases.

## Preserved Boundaries

- Always-on safety pins remain in `AGENTS.md`: product spine, replay truth,
  five `ActionCommand` surfaces, realtime isolation, output proof, no product
  fallback, holdout/commercial-reference boundaries, and human-playback safety.
- The audio-QA core retains fail-closed evidence, exact output-path proof,
  source-to-decision lineage, development/holdout separation, realtime safety,
  automation-versus-human authority, and no-repeat playback rules.
- The two-generation human-review stop remains explicit in the listening module.
- Percussive-force algorithms, equations, thresholds, and historical evidence
  moved without retuning. Versioned benchmark JSON and exact Decision-Log
  decisions remain execution authority.
- Short-fork, bounded, disjoint subagent work is documented without creating a
  second orchestration system.

## Verification

Required closeout checks:

```bash
python /home/markus/.codex/skills/.system/skill-creator/scripts/quick_validate.py \
  .codex/skills/riotbox-development
python /home/markus/.codex/skills/.system/skill-creator/scripts/quick_validate.py \
  .codex/skills/riotbox-rave-punk-production
python /home/markus/.codex/skills/.system/skill-creator/scripts/quick_validate.py \
  .codex/skills/riotbox-listening-review
just decision-search "RBX-252"
just decision-search-fixtures
git diff --check
git diff --exit-code main -- docs/benchmarks
```

No human listening is required: this slice changes no sound recipe or audible
artifact, so `human_verdict` is not applicable.
