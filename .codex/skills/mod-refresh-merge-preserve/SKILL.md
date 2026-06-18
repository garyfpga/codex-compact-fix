---
name: "mod-refresh-merge-preserve"
description: "Use only when invoked by mod-refresh-release or when explicitly asked to perform an upstream merge and compact-fix preservation."
---

# Mod Refresh Merge Preserve

## Purpose

Perform the real `upstream/main` merge for this compact-fix fork while preserving the behavior recorded in `docs/compact-fix/ChangeLog.md`.

This skill is not a general merge helper. Use it only as part of `$mod-refresh-release` or when the user explicitly asks to merge upstream and preserve the compact-fix contract.

## Hard Gates

Before mutating the repository:

1. Confirm the expected branch from the invoking plan or user request.
2. Confirm the worktree is clean enough for a merge with `git status --short`.
3. Confirm `upstream` is configured and fetch the target with `git fetch upstream main`.
4. Read `docs/compact-fix/ChangeLog.md` and treat its Preservation Checklist and Behavior Changes as the source of truth.
5. Confirm there is a preflight handoff from `$mod-refresh-preflight` or `$mod-refresh-release` that lists the expected upstream overlaps, likely conflicts, and preservation risks.

Stop before running `git merge` if the branch is unexpected, the worktree has unrelated changes, `upstream/main` cannot be fetched, the ChangeLog is missing, or the preflight handoff is unavailable.

## Subagent Policy

Use both subagents during this workflow:

- `merge-conflict-worker`: same model as main agent, `reasoning_effort = high`.
- `compact-preservation-reviewer`: same model as main agent, `reasoning_effort = high`.

Give each subagent bounded context: the preflight handoff, the current conflict or diff summary, and the relevant sections of `docs/compact-fix/ChangeLog.md`. The main agent remains responsible for applying edits, running verification, and deciding whether a missed-risk stop condition has been reached.

## Merge Workflow

1. Reconfirm the target:

   ```bash
   git branch --show-current
   git status --short
   git remote -v
   git fetch upstream main
   ```

2. Start the real upstream merge:

   ```bash
   git merge upstream/main
   ```

3. If conflicts appear, inventory them with:

   ```bash
   git status --short
   git diff --name-only --diff-filter=U
   ```

4. For each conflicted or semantically overlapping file, map it to the relevant ChangeLog behavior before editing. Use `merge-conflict-worker` for conflict analysis and proposed resolutions.

5. Resolve conflicts by preserving the compact-fix contract unless a newer, explicit user-approved plan says otherwise.

6. After conflicts are resolved, run only the non-test maintenance checks needed by touched files. Allowed examples include `just fmt`, schema generation, snapshot review or acceptance when generated UI/text artifacts intentionally changed, and dependency lock maintenance if dependencies changed. Do not run `just test`, `cargo test`, Bazel tests, full upstream suites, or focused upstream test commands unless the user, coordinator, or release plan explicitly requests tests. Build verification happens in `$mod-refresh-build`; do not run the release build from this skill by default.

7. Use `compact-preservation-reviewer` on the final diff before reporting completion.

Do not commit, tag, push, publish, or release from this skill unless the invoking release flow explicitly instructs that step.

## Preservation Checklist

Audit every merge resolution and final diff against these ChangeLog items:

- Preserve `remote_compact` config parsing, validation bounds, defaults, effective config resolution, tests, and `codex-rs/core/config.schema.json` generation.
- Preserve the shared remote-first fallback wrapper as the single policy owner for auto and manual compact routing.
- Preserve compact-only fast service tier behavior: use priority capacity for compact work when supported, omit the remote tier override for API-key auth, and do not affect ordinary sampling.
- Preserve auto and manual compact call-site routing through the shared wrapper, including V2 selection when its feature flag is enabled and local-only behavior when the provider lacks remote compaction.
- Preserve compact transport boundaries: V1 compact must use explicit retry settings, zero hidden transport retries, bounded visible attempts, configured timeout, TCP keepalive, and normal client headers/proxy/CA/cookie behavior.
- Preserve ordinary Responses retry behavior by keeping compact-specific retry policy out of generic non-compact endpoint defaults.
- Preserve V1 user-visible attempt counts, timeout wording, fallback warnings, failure categories, fallback warning counts, and clean-history restore behavior.
- Preserve V2 policy parity with the shared wrapper, including version-specific attempt budget, timeout semantics, warning labels, request-shape parity where intended, and no hidden stream retries that inflate visible attempts.
- Preserve compact integration tests, parity tests, config tests, and snapshots as source artifacts whenever request shape, warning text, fallback text, or config behavior changes; test commands are not run by default.
- Preserve the TUI display-only version label at `0.139.0+gary`; do not route display surfaces back to `CARGO_PKG_VERSION`.
- Preserve the Simple Power plan trail under `docs/simplepower/plans/` so future merge agents can read the rationale before changing code.
- Preserve `docs/compact-fix/ChangeLog.md` itself as the durable behavior map, updating it only when the merge intentionally changes the preserved behavior set.

## Missed-Risk Stop Condition

Stop and report a missed risk if preserving a mod requires a behavior choice that was not surfaced during preflight.

Examples include:

- Upstream rewrote a compact config, retry, fallback, call-site, or TUI version surface that preflight did not identify.
- A conflict has multiple plausible resolutions and the ChangeLog does not clearly choose one.
- Preserving the fork behavior would require changing a new upstream API or test contract that was not part of the preflight risk list.
- Verification reveals request-shape, warning-text, snapshot, schema, or version-label drift that was not anticipated by preflight.

When this happens, do not make a unilateral preservation choice. Leave the worktree in a diagnosable state, report `BLOCKED` or `NEEDS_CONTEXT`, list the file paths and behavior choices, explain why the issue was missed by preflight, and ask for coordinator direction.

## Final Report

Report:

- Current branch and upstream ref merged.
- Conflict files and how each was resolved.
- Preservation checklist result, including any items not touched.
- Verification commands and results.
- `skipped-test` status, including whether no test commands were requested or which explicit request caused tests to run.
- `skipped-Bazel` status, including whether no Bazel command was requested or which explicit request caused Bazel to run.
- Changed files.
- Whether the merge is ready for the next `$mod-refresh-release` step.
- Any residual risks, especially skipped tests or preservation assumptions.
