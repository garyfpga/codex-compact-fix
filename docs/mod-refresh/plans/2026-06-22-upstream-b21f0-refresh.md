# Mod Refresh Plan: upstream b21f0

## Objective
Refresh the compact-fix fork from `upstream/main`, preserve the fork-local compact and model-picker behavior, build the Linux CLI artifact, and publish the `.mod` release.

## Fresh preflight
- Source: current-session `$mod-refresh-full-release` preflight on 2026-06-22.
- Current branch: `feat/mod-refresh-b21f0e`.
- Release source HEAD before merge: `0cab38db73f65c36f75acd19e0246e31b48460a7`.
- Upstream ref: `upstream/main`.
- Upstream target SHA: `b21f0e7a98c05a570cc227e2ec62a2e29c3a7225`.
- Merge base: `04483f4ce5694d471e471583d4ca286908d7c8b7`.
- Worktree before preflight simulation: clean.
- Temporary merge simulation: removed after use; main worktree remains clean.
- Recommendation: ready to continue because the user explicitly requested the full merge release, with high-risk preservation notes carried forward.

## Preflight conflict inventory
The temporary merge simulation returned conflicts in:

- `codex-rs/core/src/compact_remote.rs`
- `codex-rs/core/src/config/mod.rs`
- `codex-rs/tui/src/app/event_dispatch.rs`
- `codex-rs/tui/src/chatwidget/tests/slash_commands.rs`
- `codex-rs/tui/src/slash_command.rs`

## Upstream overlap summary
Compact-relevant upstream touches include:

- `codex-rs/core/config.schema.json`
- `codex-rs/core/src/compact.rs`
- `codex-rs/core/src/compact_remote.rs`
- `codex-rs/core/src/compact_remote_v2.rs`
- `codex-rs/core/src/config/config_tests.rs`
- `codex-rs/core/src/config/mod.rs`
- `codex-rs/core/src/session/turn.rs`

Model-picker and slash-command upstream touches include:

- `codex-rs/tui/src/app/event_dispatch.rs`
- `codex-rs/tui/src/app_event.rs`
- `codex-rs/tui/src/bottom_pane/slash_commands.rs`
- `codex-rs/tui/src/chatwidget/slash_dispatch.rs`
- `codex-rs/tui/src/chatwidget/tests/slash_commands.rs`
- `codex-rs/tui/src/slash_command.rs`

Broad unrelated upstream changes also touched code-mode, app-server, core skills, context/token budget, exec-server, sandboxing, thread-store, and protocol files.

## Preservation risks
Carry these `docs/compact-fix/ChangeLog.md` behavior groups into merge resolution:

- Preserve `remote_compact` config parsing, validation bounds, defaults, effective config resolution, config tests, and `codex-rs/core/config.schema.json` generation.
- Preserve the shared remote-first fallback wrapper as the single policy owner for auto and manual compact routing.
- Preserve compact-only fast service tier behavior, including API-key auth request-shape behavior.
- Preserve V1 compact transport boundaries: explicit retry settings, zero hidden transport retries, configured timeout, TCP keepalive, headers/proxy/CA/cookie behavior, and warning categories.
- Preserve ordinary Responses retry behavior by keeping compact-specific retry policy out of generic endpoint defaults.
- Preserve V2 policy parity through the shared wrapper without hidden stream retries inflating visible attempts.
- Preserve compact integration tests, parity tests, config tests, and snapshots as source artifacts when request shape, warning text, fallback text, or config behavior changes.
- Preserve metadata-based `.mod` release version behavior through `CODEX_CLI_RELEASE_VERSION`.
- Preserve `/model` as session-only and `/modelp` as persistent, including nested model, reasoning, and Plan-mode picker paths.
- Preserve the Simple Power plan trail and the compact-fix changelog.

## Release metadata contract
- Expected `upstreamhash.txt`: `b21f0e7a98c05a570cc227e2ec62a2e29c3a7225`
- Expected `modversion.txt`: `1`
- Latest stable upstream release source checked during planning: `0.141.0`.
- Base series: `0.141`.
- Upstream short: `b21f0`.
- Expected release version: `0.141.b21f0.1.mod`.
- Version contract: `<latest-upstream-major>.<latest-upstream-minor>.<first5-upstreamhash>.<modversion>.mod`.
- Build handoff: `version="${base_series}.${upstream_short}.${mod_version}.mod"` and `CODEX_CLI_RELEASE_VERSION="${version}" cargo build -p codex-cli --release`.
- Do not derive the suffix from final `HEAD`, `git rev-parse --short`, or the upstream SemVer patch component.

## Execution checklist
0. Plan artifact gate:
   - Resolve release-plan reviewer findings.
   - Commit this plan and its release notes before the real merge so `git status --short` is clean for merge preservation.
1. Merge preservation:
   - Reconfirm branch, clean worktree, upstream remote, upstream target SHA, and metadata expectations.
   - Run `git fetch upstream main` and confirm `git rev-parse upstream/main` is still `b21f0e7a98c05a570cc227e2ec62a2e29c3a7225`.
   - Read `docs/compact-fix/ChangeLog.md` again and treat it as the source of truth for every conflict resolution.
   - Validate expected metadata shapes before merge: `upstreamhash.txt` value is one full 40-character lowercase hex SHA line and `modversion.txt` value is one positive decimal integer line.
   - Run the real `git merge upstream/main`.
   - Use `merge-conflict-worker` after the real conflict inventory and before applying conflict resolutions.
   - Resolve the five known conflicts by preserving the compact-fix behavior groups above.
   - Update `upstreamhash.txt` and `modversion.txt` to the expected values.
   - Run required non-test maintenance only. If Rust dependencies changed, run dependency lock maintenance.
   - Run `cd codex-rs && just fmt` after code changes.
   - Use `compact-preservation-reviewer` on the final merge diff before reporting merge completion.
   - Do not run tests unless explicitly requested.
   - Do not run Bazel unless explicitly requested.
2. Build verification:
   - Recompute latest stable upstream release and metadata version.
   - Stop if the recomputed latest stable upstream release differs from `0.141.0`, unless the coordinator approves the resulting new version.
   - Build with `CODEX_CLI_RELEASE_VERSION="${version}" cargo build -p codex-cli --release` from `codex-rs`.
   - Copy `codex-rs/target/release/codex` to repository-root `codex`.
   - Verify `./codex --version` is exactly `codex-cli 0.141.b21f0.1.mod`.
   - Use `build-verifier` after the artifact is copied and before publish handoff.
3. Publish:
   - Recompute and revalidate version from checked-in metadata.
   - Stop if the recomputed latest stable upstream release differs from `0.141.0`, unless the coordinator approves the resulting new version.
   - Confirm no local tag, remote tag, or GitHub release exists for `0.141.b21f0.1.mod`.
   - Confirm repository-root `codex` exists, is executable, and reports the expected version.
   - Use `release-packaging-reviewer` before creating any tag or GitHub release.
   - Tag final HEAD with annotated tag `0.141.b21f0.1.mod`.
   - Push `refs/tags/0.141.b21f0.1.mod` to `origin`.
   - Create the GitHub release in `garyfpga/codex-compact-fix` with title `0.141.b21f0.1.mod`.
   - Upload repository-root artifact `codex`.
   - If `git tag`, `git push`, or `gh release create` partially succeeds and a later step fails, inspect and report local tag, remote tag, and GitHub release state before any retry or recovery.

## Test and Bazel decisions
- Tests: not run unless explicitly requested.
- Bazel: not used; using Cargo release build only.

## Publish details
- Publish repository: `garyfpga/codex-compact-fix`.
- Artifact path and uploaded asset name: `codex`.
- Tag and title: `0.141.b21f0.1.mod`.
- Release notes source: `docs/mod-refresh/plans/2026-06-22-upstream-b21f0-release-notes.md`.
- Remote tag precheck during planning: no matching remote tag found.
- GitHub release precheck during planning: release not found.

## Stop conditions
Stop and ask for direction if:

- Upstream target SHA differs from `b21f0e7a98c05a570cc227e2ec62a2e29c3a7225`.
- A conflict exposes a compact-fix behavior choice not surfaced in this plan.
- `upstreamhash.txt` or `modversion.txt` is missing, malformed, dirty after staging expectations, or divergent from this plan.
- The recomputed latest stable upstream release differs from `0.141.0` without explicit coordinator approval for the resulting new version.
- Build output is missing or `./codex --version` is not exactly `codex-cli 0.141.b21f0.1.mod`.
- Release notes, publish repository, tag, artifact path, or existing release state becomes ambiguous.
- `git tag` or `git push` succeeds but a later publish step fails; report local tag, remote tag, and GitHub release state before any retry or recovery.
