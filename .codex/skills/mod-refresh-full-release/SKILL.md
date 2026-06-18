---
name: "mod-refresh-full-release"
description: "Run a complete mod refresh release by preflighting upstream, merging, preserving local mods, building the Linux artifact, tagging, and publishing. Use when the user explicitly asks for the full upstream refresh release flow."
---

# Mod Refresh Full Release

## Purpose

Use this skill as the explicit full upstream refresh release entry point. It runs a fresh upstream preflight in the current session, then hands off to the mutating release orchestrator only when preflight is ready.

This skill covers the full flow:

1. `$mod-refresh-preflight`
2. `$mod-refresh-release`
3. `$mod-refresh-merge-preserve`
4. `$mod-refresh-build`
5. `$mod-refresh-publish`

## Required Preflight

Run `$mod-refresh-preflight` first in the current session. Treat preflight as fresh only when it was produced after the user's full-release request and reflects the current worktree, upstream target, release target, and intended artifact.

Do not mutate the worktree, merge, build, tag, publish, or invoke `$mod-refresh-release` if preflight is blocked or reports unresolved blockers. Report the blocker and ask for direction.

If preflight recommends ready to continue, invoke `$mod-refresh-release` with a clear current-session handoff.

## Handoff Requirements

Before invoking `$mod-refresh-release`, ensure the handoff or release plan records these exact decisions:

- `Tests: not run unless explicitly requested`
- `Bazel: not used; using Cargo release build only`

Include the preflight source, preflight recommendation, upstream ref, current branch, intended release target, expected artifact, and any compact-fix preservation risks that `$mod-refresh-release` must carry into its plan.

## Test, Bazel, And Maintenance Policy

Do not run `just test`, `cargo test`, Bazel tests, full upstream suites, focused upstream test commands, or Bazel build commands unless the user explicitly requests them for this release.

Use only non-test maintenance checks when needed: `just fmt`, schema generation, snapshot review or acceptance for intentionally changed generated UI/text artifacts, dependency lock maintenance when dependencies changed, and the Cargo release build performed by `$mod-refresh-build`.

If Rust dependencies changed, run the required dependency lock maintenance from the repository root and record the commands and results. This does not authorize Bazel as the release build or test path.

## Stop Conditions

Stop before mutation or publishing and ask for direction when any of these occur:

- Preflight is blocked or reports unresolved blockers.
- Merge preservation discovers an unsurfaced compact-fix behavior choice.
- Build fails or the expected artifact is missing.
- Artifact naming, release tag, release notes source, publish repository, publish destination, or existing release state is ambiguous.
- The current state diverges from preflight assumptions or the release handoff.
- `git tag` succeeds but `gh release create` fails; report the partial tag state and ask for explicit recovery direction.

## Subagent Policy

When using subagents for this skill, use the same model as the main agent and `reasoning_effort = high`.

- `full-release-preflight-reviewer`: Review the fresh preflight result for blockers, stale assumptions, and whether release continuation is justified.
- `full-release-chain-reviewer`: Review the full handoff and chain readiness for missing no-test/no-Bazel records, unclear artifact or publish details, and unsafe continuation.

## Completion

Finish with a concise release summary that includes the preflight result, `$mod-refresh-release` handoff, release plan path if created by the release orchestrator, merge result, build result, tag, published artifact name, and any follow-up risks.
