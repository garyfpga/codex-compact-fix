# Mod Refresh Plan: upstream 283bc

## Objective
Refresh the compact-fix fork from `upstream/main`, preserve the fork-local compact and model-picker behavior, build the Linux CLI artifact, and publish the `.mod` release from a publish branch.

## Fresh preflight
- Source: current-session `$mod-refresh-full-release` preflight on 2026-06-24.
- Current branch: `pub/mod-refresh-2026-06-24`.
- Release source HEAD before merge: `de5b6c832a46313815ca303d1e7264975ef6dfe6`.
- Upstream ref: `upstream/main`.
- Upstream target SHA: `283bc4cf011047314b4804c0f1ccd06e4f6a95c5`.
- Merge base: `b21f0e7a98c05a570cc227e2ec62a2e29c3a7225`.
- Worktree before preflight simulation: clean.
- Temporary merge simulation: removed after use; main worktree remains clean.
- Recommendation: ready to continue because the user explicitly requested the full merge release, with high-risk preservation notes carried forward.
- Preflight reviewer: continuation approved; conflicts are surfaced and concentrated in compact/auth paths expected for merge preservation.
- Release-plan reviewer: no blocking findings after committing this plan and notes; upstream target, pre-merge HEAD, latest stable upstream release, target tag, and GitHub release absence were checked.
- Full-release-chain reviewer: stop before merge while plan and notes are untracked; record reviewer gates and commit the plan artifacts before real merge. Current metadata and repository-root `codex` are expected to be stale before merge/build and must be updated before build/publish.

## Preflight conflict inventory
The temporary merge simulation returned conflicts in:

- `codex-rs/core/src/client.rs`
- `codex-rs/core/src/compact.rs`
- `codex-rs/core/src/compact_remote.rs`
- `codex-rs/core/src/compact_remote_v2.rs`
- `codex-rs/core/src/lib.rs`
- `codex-rs/core/src/session/turn.rs`
- `codex-rs/login/src/auth/default_client.rs`

## Upstream overlap summary
Compact-relevant upstream touches include:

- `codex-rs/codex-api/src/lib.rs`
- `codex-rs/core/config.schema.json`
- `codex-rs/core/src/client.rs`
- `codex-rs/core/src/compact.rs`
- `codex-rs/core/src/compact_remote.rs`
- `codex-rs/core/src/compact_remote_v2.rs`
- `codex-rs/core/src/config/config_tests.rs`
- `codex-rs/core/src/config/mod.rs`
- `codex-rs/core/src/lib.rs`
- `codex-rs/core/src/session/turn.rs`
- `codex-rs/core/src/tasks/compact.rs`
- `codex-rs/core/tests/suite/compact_remote.rs`
- `codex-rs/core/tests/suite/compact_remote_parity.rs`
- `codex-rs/login/src/auth/default_client.rs`

Model-picker, slash-command, and TUI display upstream touches include:

- `codex-rs/tui/src/app/event_dispatch.rs`
- `codex-rs/tui/src/app_event.rs`
- `codex-rs/tui/src/bottom_pane/status_surface_preview.rs`
- `codex-rs/tui/src/chatwidget/status_surfaces.rs`
- `codex-rs/tui/src/chatwidget/tests/popups_and_settings.rs`
- `codex-rs/tui/src/chatwidget/tests/status_surface_previews.rs`
- `codex-rs/tui/src/slash_command.rs`

Broad unrelated upstream changes also touched app-server, exec-server, code-mode, plugin, SDK, CI, protocol, sandboxing, thread-store, and dependency files.

## Preservation risks
Carry these `docs/compact-fix/ChangeLog.md` behavior groups into merge resolution:

- Preserve `remote_compact` config parsing, validation bounds, defaults, effective config resolution, config tests, and `codex-rs/core/config.schema.json` generation.
- Preserve the shared remote-first fallback wrapper as the single policy owner for auto and manual compact routing.
- Preserve compact-only fast service tier behavior, including API-key auth request-shape behavior.
- Preserve auto and manual compact routing through the shared wrapper, including V2 selection when the feature flag is enabled and local-only behavior when the provider lacks remote compaction.
- Preserve compact transport boundaries: explicit V1 retry settings, zero hidden transport retries, bounded visible attempts, configured timeout, TCP keepalive, normal client headers, proxy, CA, and cookie behavior.
- Preserve ordinary Responses retry behavior by keeping compact-specific retry policy out of generic endpoint defaults.
- Preserve V1 visible attempt counts, timeout wording, fallback warnings, failure categories, fallback warning counts, and clean-history restore behavior.
- Preserve V2 policy parity through the shared wrapper without hidden stream retries inflating visible attempts.
- Preserve compact integration tests, parity tests, config tests, and snapshots as source artifacts when request shape, warning text, fallback text, or config behavior changes.
- Preserve metadata-based `.mod` release version behavior through `CODEX_CLI_RELEASE_VERSION`.
- Preserve `/model` as session-only and `/modelp` as persistent, including nested model, reasoning, and Plan-mode picker paths.
- Preserve the Simple Power plan trail and the compact-fix changelog.

## Release metadata contract
- Expected `upstreamhash.txt`: `283bc4cf011047314b4804c0f1ccd06e4f6a95c5`
- Expected `modversion.txt`: `1`
- Latest stable upstream release source checked during planning: `0.142.0` (`rust-v0.142.0`).
- Base series: `0.142`.
- Upstream short: `283bc`.
- Expected metadata suffix: `283bc.1.mod`.
- Expected release version: `0.142.283bc.1.mod`.
- Version contract: `<latest-upstream-major>.<latest-upstream-minor>.<first5-upstreamhash>.<modversion>.mod`.
- Build handoff: `version="${base_series}.${upstream_short}.${mod_version}.mod"` and `CODEX_CLI_RELEASE_VERSION="${version}" cargo build -p codex-cli --release`.
- Do not derive the suffix from final `HEAD`, `git rev-parse --short`, or the upstream SemVer patch component.

## Execution checklist
0. Plan artifact gate:
   - Resolve release-plan reviewer and full-release-chain reviewer findings by recording the reviewer results in this plan.
   - Commit this plan and its release notes before the real merge so `git status --short` is clean for merge preservation.
1. Merge preservation:
   - Reconfirm branch, clean worktree, upstream remote, upstream target SHA, and metadata expectations.
   - Run `git fetch upstream main` and confirm `git rev-parse upstream/main` is still `283bc4cf011047314b4804c0f1ccd06e4f6a95c5`.
   - Read `docs/compact-fix/ChangeLog.md` again and treat it as the source of truth for every conflict resolution.
   - Validate expected metadata shapes before merge: `upstreamhash.txt` value is one full 40-character lowercase hex SHA line and `modversion.txt` value is one positive decimal integer line.
   - Run the real `git merge upstream/main`.
   - Use `merge-conflict-worker` after the real conflict inventory and before applying conflict resolutions.
   - Resolve the seven known conflicts by preserving the compact-fix behavior groups above.
   - Update `upstreamhash.txt` and `modversion.txt` to the expected values.
   - Run required non-test maintenance only.
   - If Rust dependency changes survive the merge, run and record `just bazel-lock-update` and `just bazel-lock-check` from the repository root. This is dependency lock maintenance only and does not authorize Bazel build or test.
   - Run `cd codex-rs && just fmt` after code changes.
   - Use `compact-preservation-reviewer` on the final merge diff before reporting merge completion.
   - Do not run tests unless explicitly requested.
   - Do not run Bazel build or Bazel test unless explicitly requested.
2. Build verification:
   - Recompute latest stable upstream release and metadata version.
   - Stop if the recomputed latest stable upstream release differs from `0.142.0`, unless the coordinator approves the resulting new version.
   - Build with `CODEX_CLI_RELEASE_VERSION="${version}" cargo build -p codex-cli --release` from `codex-rs`.
   - Copy `codex-rs/target/release/codex` to repository-root `codex`.
   - Verify `./codex --version` is exactly `codex-cli 0.142.283bc.1.mod`.
   - Use `build-verifier` after the artifact is copied and before publish handoff.
3. Publish:
   - Recompute and revalidate version from checked-in metadata.
   - Stop if the recomputed latest stable upstream release differs from `0.142.0`, unless the coordinator approves the resulting new version.
   - Confirm no local tag, remote tag, or GitHub release exists for `0.142.283bc.1.mod`.
   - Confirm repository-root `codex` exists, is executable, and reports the expected version.
   - Use `release-packaging-reviewer` before creating any tag or GitHub release.
   - Tag final HEAD with annotated tag `0.142.283bc.1.mod`.
   - Push `refs/tags/0.142.283bc.1.mod` to `origin`.
   - Create the GitHub release in `garyfpga/codex-compact-fix` with title `0.142.283bc.1.mod`.
   - Upload repository-root artifact `codex`.
   - If `git tag`, `git push`, or `gh release create` partially succeeds and a later step fails, inspect and report local tag, remote tag, and GitHub release state before any retry or recovery.

## Test and Bazel decisions
- Tests: not run unless explicitly requested.
- Bazel: not used; using Cargo release build only.

## Build details
- Build command: `CODEX_CLI_RELEASE_VERSION="0.142.283bc.1.mod" cargo build -p codex-cli --release` from `codex-rs`.
- Expected Linux CLI binary path: `codex-rs/target/release/codex`.
- Expected copied artifact path: `codex`.
- Expected version output: `codex-cli 0.142.283bc.1.mod`.

## Publish details
- Publish repository: `garyfpga/codex-compact-fix`.
- `gh` repository selection must be explicit: use `--repo garyfpga/codex-compact-fix` for release commands because implicit `gh` repo detection in this checkout may resolve to `openai/codex`.
- Artifact path and uploaded asset name: `codex`.
- Tag and title: `0.142.283bc.1.mod`.
- Release notes source: `docs/mod-refresh/plans/2026-06-24-upstream-283bc-release-notes.md`.

## Stop conditions
Stop and ask for direction if:

- Upstream target SHA differs from `283bc4cf011047314b4804c0f1ccd06e4f6a95c5`.
- A conflict exposes a compact-fix behavior choice not surfaced in this plan.
- `upstreamhash.txt` or `modversion.txt` is missing, malformed, dirty after staging expectations, or divergent from this plan.
- The recomputed latest stable upstream release differs from `0.142.0` without explicit coordinator approval for the resulting new version.
- Build output is missing or `./codex --version` is not exactly `codex-cli 0.142.283bc.1.mod`.
- Release notes, publish repository, tag, artifact path, or existing release state becomes ambiguous.
- `git tag` or `git push` succeeds but a later publish step fails; report local tag, remote tag, and GitHub release state before any retry or recovery.
