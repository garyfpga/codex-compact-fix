---
name: mod-refresh-release
description: "Orchestrate a mutating mod refresh release. Use this skill when asked to actually merge upstream, preserve local mods, build the Linux CLI binary, tag, and publish a mod release; it requires a fresh current-session mod-refresh-preflight report, records a release plan under docs/mod-refresh/plans/, and chains merge preservation, build, and publish gates."
---

# Mod Refresh Release

## Purpose

Use this skill as the mutation entry point for a full mod refresh release. Require a fresh preflight report, write a release-run plan, then chain the merge, build, and publish skills in order.

When the user wants one request to cover preflight through publish, prefer `$mod-refresh-full-release` as the full upstream refresh entry point. Use `$mod-refresh-release` when a fresh current-session preflight handoff already exists or the user explicitly asks to run the release orchestrator from that state.

## Required Preflight

Before planning or mutating, require a fresh `mod-refresh-preflight` report from the current session. Treat a preflight as fresh only when it was produced after the current release request and reflects the current worktree, upstream target, upstream target SHA, release target, and intended artifact.

If no fresh current-session report is available, stop and either run `$mod-refresh-preflight` or ask the user/coordinator to provide one. Do not reuse an old report from disk or a previous session without revalidating it in the current session. Stop if the preflight does not explicitly report the `upstream target SHA` from `git rev-parse upstream/main`.

## Release Plan

Before running merge work, save the release-run plan and notes under:

```text
docs/mod-refresh/plans/YYYY-MM-DD-<short-topic>.md
```

Use the current local date for `YYYY-MM-DD`. Use a short lowercase hyphenated topic that identifies the upstream or release target.

Include:

- Fresh preflight report source and summary.
- Release objective, upstream ref, upstream target SHA, current branch, latest stable upstream release source for major/minor version components, and expected artifact path `codex`.
- Expected `upstreamhash.txt` value: the preflight upstream target SHA, as one full 40-character lowercase hex SHA line.
- Expected `modversion.txt` value: `1` for a new upstream refresh, unless the handoff records a different explicit approved positive decimal integer.
- Expected release version contract: `<latest-upstream-major>.<latest-upstream-minor>.<first5-upstreamhash>.<modversion>.mod`; do not derive the suffix from final `HEAD`, `git rev-parse --short`, or the upstream SemVer patch component.
- Computed release version handoff: `version="${base_series}.${upstream_short}.${mod_version}.mod"` must be passed into the build as `CODEX_CLI_RELEASE_VERSION="${version}"`.
- Ordered execution checklist for merge preservation, build verification, and publish.
- Local mod preservation risks and files/areas that need special attention.
- Build command `CODEX_CLI_RELEASE_VERSION="${version}" cargo build -p codex-cli --release`, expected Linux CLI binary path, and `./codex --version` output to capture.
- `Tests: not run unless explicitly requested`.
- `Bazel: not used; using Cargo release build only`.
- Publish command or process, tag name, latest-stable upstream release source, release notes source, and artifact upload target.
- Approval and stop-condition notes.

After drafting the plan, use a `release-plan-reviewer` subagent with the same model as main agent and `reasoning_effort = high` to review the plan for missing gates, stale preflight assumptions, unclear artifact path handling, and unsafe publish steps. Resolve reviewer findings in the plan before mutating.

## Execution Chain

Run the release chain in this exact order:

1. `$mod-refresh-merge-preserve`
2. `$mod-refresh-build`
3. `$mod-refresh-publish`

Keep the release plan updated as each stage completes. Record relevant commands, results, changed refs, built artifact paths, metadata file values, and decisions in the plan file.

After `$mod-refresh-merge-preserve` completes and before continuing to publish, verify the actual metadata files still match the plan:

- `upstreamhash.txt` exists and contains exactly the expected full 40-character lowercase hex upstream SHA line.
- `modversion.txt` exists and contains exactly the expected positive decimal integer line.

Stop if either file is missing, malformed, or divergent from the release plan.

Before invoking `$mod-refresh-build`, compute and record the metadata release version from the latest stable upstream base series plus the checked-in metadata files. Pass that exact value through the build handoff so the build command uses `CODEX_CLI_RELEASE_VERSION="${version}" cargo build -p codex-cli --release`. After build, record the copied artifact's `./codex --version` output and stop before publish unless it is exactly `codex-cli ${version}`.

## Test, Bazel, And Maintenance Policy

Do not run tests or Bazel build/test commands unless the user explicitly requests them. The release flow uses the Cargo release build path only.

Non-test maintenance checks are allowed when they are needed to keep the release artifacts coherent. If dependencies changed, run the required dependency lock maintenance and record the command and result in the release plan.

## Stop Conditions

Stop before publishing and ask for approval or clarification when any of these are unclear:

- Merge preservation is incomplete, conflicted, unreviewed, or does not clearly preserve the local mods identified by preflight.
- Build verification did not run, failed, produced unexpected output, or does not clearly validate the Linux CLI binary.
- The build handoff does not pass the computed `.mod` version through `CODEX_CLI_RELEASE_VERSION`.
- Artifact path, release tag base, release notes source, or publish destination is ambiguous.
- The current state diverges from the preflight assumptions or release plan.
- The release plan or handoff is missing expected `upstreamhash.txt` or `modversion.txt` values.
- `upstreamhash.txt` or `modversion.txt` is missing, malformed, or divergent from the expected release plan values before publish.
- Any publish handoff or release version note still derives the suffix from final `HEAD`, `git rev-parse --short`, or the upstream SemVer patch component.

Do not tag or publish until merge preservation, build verification, artifact path, and publish destination are explicit in the plan. If the user explicitly approves a clarified publish path, record that approval in the plan before continuing to `$mod-refresh-publish`.

## Completion

Finish with a concise release summary that includes the plan path, preflight source, expected and actual metadata values, merge result, build verification result, tag, published artifact name, and any follow-up risks.
