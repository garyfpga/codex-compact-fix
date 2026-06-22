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

## Merge preservation log
- Real merge command: `git merge upstream/main`.
- Upstream target confirmed before merge: `b21f0e7a98c05a570cc227e2ec62a2e29c3a7225`.
- Merge commit: `341b2c1160897679612506f0128787ecc487b85b`.
- Merge commit second parent: `b21f0e7a98c05a570cc227e2ec62a2e29c3a7225`.
- Conflict files matched preflight:
  - `codex-rs/core/src/compact_remote.rs`
  - `codex-rs/core/src/config/mod.rs`
  - `codex-rs/tui/src/app/event_dispatch.rs`
  - `codex-rs/tui/src/chatwidget/tests/slash_commands.rs`
  - `codex-rs/tui/src/slash_command.rs`
- Conflict resolution summary:
  - Preserved fork-local V1 `run_remote_compaction_request_v1(...)` so V1 compact keeps explicit retry, timeout, TCP keepalive, and no-hidden-retry behavior.
  - Accepted upstream `AutoCompactWindowIds` handling for compacted item window metadata.
  - Preserved `resolve_remote_compact_config(...)` and added upstream `resolve_token_budget_config(...)`; `Config::load` now resolves both.
  - Preserved persistence-aware model picker events and accepted upstream `SettingsSelectionClosed` / `SettingsSelectionSettled` queue-settling behavior.
  - Made `/modelp` follow upstream's queue-safe settings command behavior and added the missing `defer_input_until_settings_applied()` call after `open_model_popup_persistent()`.
  - Preserved `/model` session-only and `/modelp` persistent selection behavior.
- Adjacent merge integration fixes:
  - `codex-rs/core/src/session/turn.rs` now reads auto-compaction feature state from `turn_context.config.features`, matching this branch's `TurnContext` shape.
  - `codex-rs/thread-manager-sample/src/main.rs` initializes the fork-local `remote_compact` config field with `Default::default()`.
- Metadata:
  - `upstreamhash.txt` updated to `b21f0e7a98c05a570cc227e2ec62a2e29c3a7225`.
  - `modversion.txt` remains `1`.
- Maintenance commands:
  - `cd codex-rs && just fmt`: passed.
  - `cd codex-rs && just write-config-schema`: initially exposed the `turn_context.features` merge issue; passed after the fix.
  - `just bazel-lock-update`: passed with existing crate-annotation warnings.
  - `just bazel-lock-check`: passed.
  - `cd codex-rs && just fix`: initially required installing `libcap-devel`, then exposed the `thread-manager-sample` config issue; passed after fixes.
- Environment maintenance:
  - Installed `libcap-devel` with `sudo -n dnf install -y libcap-devel` so `codex-bwrap` can compile through pkg-config.
- Skipped-test status: no tests were explicitly requested, so no `just test` or `cargo test` command was run.
- Skipped-Bazel status: no Bazel build or Bazel test command was run; only lock maintenance ran because Rust dependency files changed.
- Stop-gate note:
  - After the real merge was staged and preservation review passed, local `upstream/main` advanced to `566f7bf6314cbf213de523a0268d8df89f93ef62`.
  - The merge commit's upstream parent remains the planned target `b21f0e7a98c05a570cc227e2ec62a2e29c3a7225`.
  - Checked metadata remains pinned to `b21f0e7a98c05a570cc227e2ec62a2e29c3a7225` / `1`.
  - Coordinator approved continuing pinned to `b21f0e7a98c05a570cc227e2ec62a2e29c3a7225`; the newer `566f7bf...` upstream tip is intentionally excluded from this release.

## Build verification log
- Latest stable upstream release recheck before build: `0.141.0`.
- Derived release version: `0.141.b21f0.1.mod`.
- Build command: `CODEX_CLI_RELEASE_VERSION="0.141.b21f0.1.mod" cargo build -p codex-cli --release` from `codex-rs`.
- Build result: passed; Cargo finished the `release` profile in `10m 06s`.
- Build warnings:
  - `codex-api`: unused `stream_encoded` and `stream_encoded_json_with` methods.
  - `codex-app-server`: unused `mut loader_overrides`.
- Artifact copy: `codex-rs/target/release/codex` to repository-root `codex`.
- Artifact version check: `./codex --version` returned `codex-cli 0.141.b21f0.1.mod`.
- Artifact size: `1.2G`.
- Artifact SHA-256: `c2f1619dbeb5f30199e2b32dcac0a95644fc66e772fc6cad417a0b76189c54e1`.
- Artifact git status: repository-root `codex` and `codex-rs/target/` are ignored.
- Skipped-test status: no tests were explicitly requested, so no `just test` or `cargo test` command was run.
- Skipped-Bazel status: no Bazel build or Bazel test command was run.

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
