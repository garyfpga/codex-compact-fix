---
name: mod-refresh-release
description: "Orchestrate a mutating mod refresh release. Use this skill when asked to actually merge upstream, preserve local mods, build the Linux CLI binary, tag, and publish a mod release; it requires a fresh current-session mod-refresh-preflight report, records a release plan under docs/mod-refresh/plans/, and chains merge preservation, build, and publish gates."
---

# Mod Refresh Release

## Purpose

Use this skill as the mutation entry point for a full mod refresh release. Require a fresh preflight report, write a release-run plan, then chain the merge, build, and publish skills in order.

When the user wants one request to cover preflight through publish, prefer `$mod-refresh-full-release` as the full upstream refresh entry point. Use `$mod-refresh-release` when a fresh current-session preflight handoff already exists or the user explicitly asks to run the release orchestrator from that state.

## Required Preflight

Before planning or mutating, require a fresh `mod-refresh-preflight` report from the current session. Treat a preflight as fresh only when it was produced after the current release request and reflects the current worktree, upstream target, release target, and intended artifact.

If no fresh current-session report is available, stop and either run `$mod-refresh-preflight` or ask the user/coordinator to provide one. Do not reuse an old report from disk or a previous session without revalidating it in the current session.

## Release Plan

Before running merge work, save the release-run plan and notes under:

```text
docs/mod-refresh/plans/YYYY-MM-DD-<short-topic>.md
```

Use the current local date for `YYYY-MM-DD`. Use a short lowercase hyphenated topic that identifies the upstream or release target.

Include:

- Fresh preflight report source and summary.
- Release objective, upstream ref, current branch, expected release tag, and expected artifact name.
- Ordered execution checklist for merge preservation, build verification, and publish.
- Local mod preservation risks and files/areas that need special attention.
- Build command, expected Linux CLI binary path, and verification command/output to capture.
- `Tests: not run unless explicitly requested`.
- `Bazel: not used; using Cargo release build only`.
- Publish command or process, tag name, release notes source, and artifact upload target.
- Approval and stop-condition notes.

After drafting the plan, use a `release-plan-reviewer` subagent with the same model as main agent and `reasoning_effort = high` to review the plan for missing gates, stale preflight assumptions, unclear artifact naming, and unsafe publish steps. Resolve reviewer findings in the plan before mutating.

## Execution Chain

Run the release chain in this exact order:

1. `$mod-refresh-merge-preserve`
2. `$mod-refresh-build`
3. `$mod-refresh-publish`

Keep the release plan updated as each stage completes. Record relevant commands, results, changed refs, built artifact paths, and decisions in the plan file.

## Test, Bazel, And Maintenance Policy

Do not run tests or Bazel build/test commands unless the user explicitly requests them. The release flow uses the Cargo release build path only.

Non-test maintenance checks are allowed when they are needed to keep the release artifacts coherent. If dependencies changed, run the required dependency lock maintenance and record the command and result in the release plan.

## Stop Conditions

Stop before publishing and ask for approval or clarification when any of these are unclear:

- Merge preservation is incomplete, conflicted, unreviewed, or does not clearly preserve the local mods identified by preflight.
- Build verification did not run, failed, produced unexpected output, or does not clearly validate the Linux CLI binary.
- Artifact naming, release tag, release notes source, or publish destination is ambiguous.
- The current state diverges from the preflight assumptions or release plan.

Do not tag or publish until merge preservation, build verification, artifact naming, and publish destination are explicit in the plan. If the user explicitly approves a clarified publish path, record that approval in the plan before continuing to `$mod-refresh-publish`.

## Completion

Finish with a concise release summary that includes the plan path, preflight source, merge result, build verification result, tag, published artifact name, and any follow-up risks.
