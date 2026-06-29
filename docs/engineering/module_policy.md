# Riotbox Module Policy

Version: 0.1
Status: Draft
Audience: Rust contributors, reviewers, coding agents

---

## Purpose

This policy keeps Riotbox reviewable without turning file-size limits into
mechanical source sharding.

Small files are useful, but semantic Rust modules are more important than
line-count compliance.

## Core Rule

Prefer real Rust modules over textual `include!` splits.

`include!` is allowed only for:

- generated code with a documented generator
- deliberately embedded static artifacts
- narrow macro or compatibility cases that cannot be represented cleanly as a
  normal module

Do not use `include!` as a normal way to make a large Rust file appear smaller.

## File Size Guidance

- Aim for files that are easy to review, typically under roughly 400-700 lines.
- Treat line count as a warning signal, not as the design goal.
- A file may exceed that range when further splitting would make ownership less
  clear; record the reason in a short comment, review note, or decision log.
- Do not split large type or enum files by line number. Split by domain concept,
  visibility boundary, test ownership, or stable API boundary.

## Migration Rule

When replacing textual includes, the first slice must be mechanical:

- no behavior change
- no feature work in the same PR
- preserve public API compatibility with `pub use` where appropriate
- make imports explicit enough that child module dependencies are reviewable
- keep tests green before and after the move

After migration, follow-up slices may improve naming, visibility, and ownership.

## Target Shape

Prefer this:

```rust
pub mod defaults;
pub mod export;
pub mod mc202;
pub mod model;
pub mod validation;

pub use defaults::*;
pub use export::*;
pub use mc202::*;
pub use model::*;
pub use validation::*;
```

Over this:

```rust
include!("session/version_types.rs");
include!("session/mc202_types.rs");
include!("session/defaults.rs");
```

The `pub use` layer can preserve compatibility, but the true ownership tree
should be visible as modules.

## Include Inventory And Guardrail

The initial RIOTBOX-1321 inventory lives in
`docs/engineering/textual_include_inventory_2026-06-29.md`.

To refresh the raw scan:

```bash
rg 'include!' crates --glob '*.rs'
```

For each include site record:

- owning file
- included files
- purpose
- generated/static/compatibility vs mechanical split
- migration risk

A manual guardrail rejects unexpected new textual include owners or changed
include counts while the allowlist is still being reduced:

```bash
scripts/check_no_textual_includes.sh
```

The current allowlist is
`docs/engineering/textual_include_allowlist.txt`. Update it only in PRs that
intentionally remove or convert include sites, or after a reviewed exception.
The guardrail should remain a developer check until the migration is far enough
to make it a hard CI gate.

## Review Checklist

Before merging a module-policy slice, answer:

- Did this remove or avoid a textual include used only for line-count pressure?
- Is the new boundary semantic rather than numbered or arbitrary?
- Are public exports preserved or intentionally changed?
- Are imports explicit enough for reviewers to see dependencies?
- Did the PR avoid unrelated feature or behavior changes?
- Are tests and formatting green?
