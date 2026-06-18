---
name: "mod-refresh-build"
description: "Build the Linux Codex CLI .mod release artifact for a mod release. Use when invoked by $mod-refresh-release or $mod-release-current, or when the user explicitly asks to build the Linux Codex CLI .mod binary."
---

# Mod Refresh Build

## Purpose

Build only the Linux Codex CLI binary for a mod release, verify the build-focused result, and place the release artifact in the repository root for the publish step. This skill may be invoked by `$mod-refresh-release` or `$mod-release-current`.

## Workflow

1. Confirm the repository is already in the release source state: post-merge for `$mod-refresh-release`, or the intended current `HEAD` for `$mod-release-current`. If invoked by `$mod-refresh-release`, read the release plan for the expected artifact name and record build results there when possible. If invoked by `$mod-release-current`, read the current-release handoff or run notes for the expected artifact name and record build results there when possible.
2. If code changed during the release run, run formatting before building:

   ```bash
   cd codex-rs
   just fmt
   ```

3. Build the Linux CLI binary from `codex-rs`:

   ```bash
   cd codex-rs
   cargo build -p codex-cli --release
   ```

   Use this Cargo command by default. Only use a more direct checked-in command if the repository has one at execution time and it clearly builds the same Linux Codex CLI binary without adding tests, Bazel, or unrelated targets.

4. Record `Tests: not run unless explicitly requested`. Do not run tests by default.
5. Record `Bazel: not used; using Cargo release build only`. Do not use Bazel by default.
6. If dependency lock maintenance is needed because dependencies changed, perform only that maintenance. It does not change the release build path and it does not authorize Bazel as a release build or test path.
7. Locate the built CLI binary. The expected default path is:

   ```text
   codex-rs/target/release/codex
   ```

   If that file is absent, inspect the release build output or Cargo metadata to locate the executable produced by the `codex-cli` package. Do not build additional packages to find it.

8. Copy the resulting CLI binary to the repository root. Use the artifact name from the release plan or coordinator request. If no artifact name is provided, ask for clarification before publishing; for build-only requests, copy to a clearly named repo-root `.mod` artifact and report the exact path.
9. Preserve executable permissions on the copied artifact. If needed, run `chmod +x <repo-root-artifact>`.

## Verification

After copying the artifact, use a `build-verifier` subagent with the same model as main agent and `reasoning_effort = high`. Ask it to verify, from the repository state and command output, that:

- `just fmt` ran from `codex-rs` after code changes when applicable.
- The build command targeted only `codex-cli` in release mode.
- `Tests: not run unless explicitly requested` is recorded and no tests ran unless explicitly requested.
- `Bazel: not used; using Cargo release build only` is recorded and no Bazel commands ran unless explicitly requested.
- The Linux CLI binary was located and copied to the repository root.
- The copied artifact path, size, executable bit, and source binary path are recorded.

Resolve any verifier findings before handing off to publish.

## Completion

Report the build command, formatting command if run, the recorded tests/Bazel decisions, skipped tests/Bazel status, source binary path, repo-root artifact path, verifier result, and any concerns. If invoked by `$mod-refresh-release` or `$mod-release-current`, update the release plan with the same details before returning control.
