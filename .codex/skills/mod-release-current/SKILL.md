---
name: "mod-release-current"
description: "Build and publish a mod release from the current HEAD without fetching upstream, running preflight, or merging. Use after feature work when the current commit is the intended release source."
---

# Mod Release Current

## Purpose

Use this skill as the explicit current-HEAD release entry point after feature work. It builds the checked-out commit and publishes that same commit without refreshing from upstream.

Do not run or invoke any upstream-refresh path:

- Do not run `git fetch upstream`.
- Do not run `$mod-refresh-preflight`.
- Do not run a merge simulation.
- Do not run `git merge upstream/main`.
- Do not invoke `$mod-refresh-release`.

## Required State Check

Before building, confirm and record:

- Current `HEAD` commit SHA.
- Current branch name.
- Worktree state from `git status --short`.
- Whether the user explicitly requested tests or Bazel commands.
- Release notes source and GitHub release destination, if already provided.

Stop before building if the current commit is not clearly the intended release source, if the branch is unclear, or if the worktree state could make the release source ambiguous.

## Test, Bazel, And Maintenance Policy

Record these exact decisions in the run notes and completion report unless the user explicitly requested otherwise:

- `Tests: not run unless explicitly requested`
- `Bazel: not used; using Cargo release build only`

Do not run tests or Bazel build/test commands unless explicitly requested by the user or coordinator. Use only non-test maintenance checks that are required to keep the current release artifact coherent, such as dependency lock maintenance required by already-present source changes. Record any such maintenance command and result.

## Execution Chain

Run the release chain in this exact order:

1. `$mod-refresh-build`
2. `$mod-refresh-publish`

Pass the confirmed current `HEAD`, branch, worktree state, test/Bazel decisions, release notes source, and GitHub release destination through the handoff context. The build step must use the current checkout as-is. The publish step must derive the version from the final `HEAD` and publish only after its packaging and safety gates pass.

## Subagent Policy

Before handing off to `$mod-refresh-publish`, use a `current-release-packaging-reviewer` subagent with the same model as the main agent and `reasoning_effort = high`.

Ask the reviewer to verify:

- The current `HEAD`, branch, and worktree state were captured before build.
- No upstream fetch, preflight, merge simulation, upstream merge, or `$mod-refresh-release` invocation occurred.
- `$mod-refresh-build` completed first and produced the repository-root artifact expected for publish.
- The exact test and Bazel decisions were recorded.
- Release notes source, GitHub release destination, artifact naming, and publish readiness are explicit.

Resolve reviewer findings before invoking `$mod-refresh-publish`. If no subagent facility is available, stop and report that the reviewer gate cannot be completed.

## Stop Conditions

Stop and ask for clarification before continuing when any of these are true:

- Current `HEAD` is not clearly the release source.
- The branch or worktree state is ambiguous.
- Uncommitted tracked changes could affect the final release commit.
- Release notes source or GitHub release destination is unclear.
- Build verification fails, is missing, or produces an unexpected artifact.
- Artifact naming does not match the version that `$mod-refresh-publish` will derive from `HEAD`.
- The packaging reviewer gate cannot run or reports unresolved findings.
- Any step would require upstream fetch, preflight, merge simulation, upstream merge, or `$mod-refresh-release`.
- Tests or Bazel commands seem necessary but were not explicitly requested.

## Completion

Report the current `HEAD`, branch, worktree state, build result, packaging reviewer result, publish result, release tag, uploaded artifact, GitHub release URL when available, and the exact recorded test/Bazel decisions.
