# Mod Refresh Plan: upstream cfead

## Objective
Refresh the compact-fix fork from `upstream/main`, preserve the fork-local compact and model-picker behavior, build the Linux CLI artifact, and publish the `.mod` release.

## Fresh preflight
- Source: current-session `$mod-refresh-full-release` preflight on 2026-06-30.
- Current branch: `pub/mod-refresh-2026-06-24`.
- Release source HEAD before merge: `d3862c88c66a35456f44392dbe53129dd6d4aee0`.
- Upstream ref: `upstream/main`.
- Upstream target SHA: `cfead68e5d3984b247cf0758e3e53b19165de848`.
- Merge base: `283bc4cf011047314b4804c0f1ccd06e4f6a95c5`.
- Worktree before preflight simulation: clean.
- Temporary merge simulation: removed after use; main worktree remains clean.
- Recommendation: ready to continue because the user explicitly requested the full merge release and the high-risk preservation conflicts were surfaced by preflight.
- Preflight reviewer: continuation approved; the four conflicts are expected compact-preservation work, not unresolved blockers.
- Full-release handoff: continue into `$mod-refresh-release` with the metadata, no-test, no-Bazel, and preservation risks recorded in this plan.

## Preflight report
### Fetch
- `git fetch upstream main:refs/remotes/upstream/main`: passed, with only an X11 forwarding warning.
- Upstream target SHA from `git rev-parse upstream/main`: `cfead68e5d3984b247cf0758e3e53b19165de848`.
- Merge base from `git merge-base HEAD upstream/main`: `283bc4cf011047314b4804c0f1ccd06e4f6a95c5`.
- Fetch concern: none.

### Worktree
- `git status --short`: clean before merge simulation.
- Current branch: `pub/mod-refresh-2026-06-24`.

### Merge simulation
- Simulation command: `git merge --no-commit --no-ff upstream/main` in a temporary detached worktree from HEAD `d3862c88c66a35456f44392dbe53129dd6d4aee0`.
- Simulation result: conflicts.
- Conflicted files:
  - `codex-rs/core/src/client.rs`
  - `codex-rs/core/src/compact_remote.rs`
  - `codex-rs/core/src/compact_token_budget.rs`
  - `codex-rs/core/tests/suite/compact_remote.rs`
- Cleanup: merge aborted and temporary worktree removed; `git worktree list --porcelain` shows only the main checkout.

### Upstream touched files
- Compact-relevant upstream touches:
  - `codex-rs/config/src/types.rs`
  - `codex-rs/core/config.schema.json`
  - `codex-rs/core/src/client.rs`
  - `codex-rs/core/src/compact_remote.rs`
  - `codex-rs/core/src/compact_token_budget.rs`
  - `codex-rs/core/src/config/mod.rs`
  - `codex-rs/core/src/session/turn.rs`
  - `codex-rs/core/tests/suite/compact_remote.rs`
- Model-picker, slash-command, and TUI display upstream touches:
  - `codex-rs/tui/src/app/event_dispatch.rs`
  - `codex-rs/tui/src/app_event.rs`
  - `codex-rs/tui/src/chatwidget/model_popups.rs`
  - `codex-rs/tui/src/chatwidget/status_surfaces.rs`
- Broad unrelated upstream changes also touched app-server, exec-server, code-mode, plugins, MCP, SDK, CI, protocol, sandboxing, thread-store, state, dependency, and generated schema files.

### Compact impact
- Risk level: high, because upstream directly overlaps compact runtime, compact token-budget, client transport, compact tests, config/schema, TUI status surfaces, and model picker paths.
- The conflicts are expected merge-preservation work. They do not block continuation because the preflight surfaced them and the ChangeLog identifies the intended fork behavior.
- Preservation source of truth: `docs/compact-fix/ChangeLog.md`.

### Recommendation
- Ready to continue if explicitly requested. The user requested `$mod-refresh-full-release` in this session.

### Continuation gate
- Release mutation proceeds through this `$mod-refresh-release` plan with upstream target SHA `cfead68e5d3984b247cf0758e3e53b19165de848`.
- Tests: not run unless explicitly requested.
- Bazel: not used; using Cargo release build only.

## Preservation risks
Carry these `docs/compact-fix/ChangeLog.md` behavior groups into merge resolution:

- Preserve `remote_compact` config parsing, validation bounds, defaults, effective config resolution, config tests, and `codex-rs/core/config.schema.json` generation.
- Preserve the shared remote-first fallback wrapper as the single policy owner for auto and manual compact routing.
- Preserve compact-only fast service tier behavior, including API-key auth request-shape behavior and no effect on ordinary sampling traffic.
- Preserve auto and manual compact routing through the shared wrapper, including V2 selection when the feature flag is enabled and local-only behavior when the provider lacks remote compaction.
- Preserve compact transport boundaries: explicit V1 retry settings, zero hidden transport retries, bounded visible attempts, configured timeout, TCP keepalive, normal client headers, proxy, CA, and cookie behavior.
- Preserve ordinary Responses retry behavior by keeping compact-specific retry policy out of generic non-compact endpoint defaults.
- Preserve V1 visible attempt counts, timeout wording, fallback warnings, failure categories, fallback warning counts, and clean-history restore behavior.
- Preserve V2 policy parity through the shared wrapper without hidden stream retries inflating visible attempts.
- Preserve compact integration tests, parity tests, config tests, and snapshots as source artifacts when request shape, warning text, fallback text, or config behavior changes.
- Preserve metadata-based `.mod` release version behavior through `CODEX_CLI_RELEASE_VERSION`.
- Preserve `/model` as session-only and `/modelp` as persistent, including nested model, reasoning, and Plan-mode picker paths.
- Preserve the Simple Power plan trail and the compact-fix changelog.

## Release metadata contract
- Expected `upstreamhash.txt`: `cfead68e5d3984b247cf0758e3e53b19165de848`
- Expected `modversion.txt`: `1`
- Latest stable upstream release source checked during planning: `0.142.4` (`rust-v0.142.4`).
- Base series: `0.142`.
- Upstream short: `cfead`.
- Expected metadata suffix: `cfead.1.mod`.
- Expected release version: `0.142.cfead.1.mod`.
- Version contract: `<latest-upstream-major>.<latest-upstream-minor>.<first5-upstreamhash>.<modversion>.mod`.
- Computed release version handoff: `version="${base_series}.${upstream_short}.${mod_version}.mod"` must be passed into the build as `CODEX_CLI_RELEASE_VERSION="${version}"`.
- Build handoff: `CODEX_CLI_RELEASE_VERSION="${version}" cargo build -p codex-cli --release`.
- Do not derive the suffix from final `HEAD`, `git rev-parse --short`, or the upstream SemVer patch component.

## Execution checklist
0. Plan artifact gate:
   - Review this plan with `release-plan-reviewer` using the same model as the main agent and `reasoning_effort = high`.
   - Review the full-release handoff with `full-release-chain-reviewer` using the same model as the main agent and `reasoning_effort = high`.
   - Resolve release-plan reviewer findings by adding an explicit post-merge metadata gate, concrete `gh --repo garyfpga/codex-compact-fix` publish commands, same-model/high-effort reviewer wording, and a post-plan-commit revalidation gate.
   - Commit this plan and its release notes before the real merge so `git status --short` is clean for merge preservation.
   - After committing this plan and its release notes, record the new pre-merge HEAD and verify the only delta from `d3862c88c66a35456f44392dbe53129dd6d4aee0` is the two planning files. Stop and refresh preflight if any other source, metadata, artifact, or release-state change appears.
1. Merge preservation:
   - Reconfirm branch, clean worktree, upstream remote, upstream target SHA, and metadata expectations.
   - Run `git fetch upstream main` and confirm `git rev-parse upstream/main` is still `cfead68e5d3984b247cf0758e3e53b19165de848`.
   - Read `docs/compact-fix/ChangeLog.md` again and treat it as the source of truth for every conflict resolution.
   - Validate expected metadata shapes before merge: `upstreamhash.txt` value is one full 40-character lowercase hex SHA line and `modversion.txt` value is one positive decimal integer line.
   - Run the real `git merge upstream/main`.
   - Use `merge-conflict-worker` after the real conflict inventory and before applying conflict resolutions.
   - Resolve the four known conflicts by preserving the compact-fix behavior groups above.
   - Update `upstreamhash.txt` and `modversion.txt` to the expected values.
   - After merge preservation and before build or publish, verify actual metadata files exactly match this plan:
     - `upstreamhash.txt`: `cfead68e5d3984b247cf0758e3e53b19165de848`
     - `modversion.txt`: `1`
     - Stop if either file is missing, malformed, dirty after staging expectations, or divergent from this plan.
   - Run required non-test maintenance only.
   - If Rust dependency changes survive the merge, run and record `just bazel-lock-update` and `just bazel-lock-check` from the repository root. This is dependency lock maintenance only and does not authorize Bazel build or test.
   - Run `cd codex-rs && just fmt` after code changes.
   - Use `compact-preservation-reviewer` on the final merge diff before reporting merge completion.
   - Do not run tests unless explicitly requested.
   - Do not run Bazel build or Bazel test unless explicitly requested.
2. Build verification:
   - Recompute latest stable upstream release and metadata version.
   - Build with `CODEX_CLI_RELEASE_VERSION="${version}" cargo build -p codex-cli --release` from `codex-rs`.
   - Copy `codex-rs/target/release/codex` to repository-root `codex`.
   - Verify `./codex --version` is exactly `codex-cli ${version}`.
   - Use `build-verifier` after the artifact is copied and before publish handoff.
3. Publish:
   - Recompute and revalidate version from checked-in metadata.
   - Confirm no local tag exists for `${version}` with `git rev-parse -q --verify "refs/tags/${version}"`.
   - Confirm no remote tag exists for `${version}` with `git ls-remote --exit-code --tags origin "refs/tags/${version}"`, treating exit code `2` as the expected missing state and any other nonzero result as ambiguous.
   - Confirm no GitHub release exists for `${version}` with `gh release view "${version}" --repo garyfpga/codex-compact-fix`, treating only a clear not-found response as the expected missing state.
   - Confirm repository-root `codex` exists, is executable, and reports the expected version.
   - Use `release-packaging-reviewer` before creating any tag or GitHub release.
   - Tag final HEAD with `git tag -a "${version}" "${final_commit}" -m "${version}"`.
   - Push `refs/tags/${version}` to `origin` with `git push origin "refs/tags/${version}"`.
   - Create the GitHub release and upload the repository-root artifact with:
     `gh release create "${version}" "codex" --repo garyfpga/codex-compact-fix --verify-tag --title "${version}" --notes-file docs/mod-refresh/plans/2026-06-30-upstream-cfead-release-notes.md`.
   - If `git tag`, `git push`, or `gh release create` partially succeeds and a later step fails, inspect and report local tag, remote tag, and GitHub release state before any retry or recovery.

## Test and Bazel decisions
- Tests: not run unless explicitly requested.
- Bazel: not used; using Cargo release build only.

## Merge preservation results
- Real merge commit: `8d7444f066160b5b46eba560fc46ff9a4167680b`.
- Upstream merged: `upstream/main` at `cfead68e5d3984b247cf0758e3e53b19165de848`.
- Merge command: `git merge upstream/main`.
- Conflicts resolved:
  - `codex-rs/core/src/client.rs`
  - `codex-rs/core/src/compact_remote.rs`
  - `codex-rs/core/src/compact_token_budget.rs`
  - `codex-rs/core/tests/suite/compact_remote.rs`
- Merge-conflict worker reviewed the conflict set and recommended preserving both fork-local compact retry/fallback behavior and upstream auth/step-context additions. It also flagged `AuthMode::BedrockApiKey` as adjacent to the fork's API-key service-tier omission contract.
- Preservation summary:
  - `client.rs` keeps the fork-local `ResponsesStreamRetryPolicy`, exact compact retry policy, compact TCP keepalive client builder, and compact request timeout fields while preserving upstream agent-identity telemetry, auth-mode, and `reasoning_effort_for_request` handling.
  - `compact_remote.rs` keeps V1 visible attempt loops, zero hidden compact retries, configured timeout, TCP keepalive, fallback warnings, and request metadata while using upstream `codex_protocol::auth::AuthMode`.
  - `compact_remote_v2.rs` and `compact_service_tier.rs` were aligned to upstream `codex_protocol::auth::AuthMode`.
  - `compact_remote.rs`, `compact_remote_v2.rs`, and `compact_service_tier.rs` now treat `AuthMode::BedrockApiKey` like `AuthMode::ApiKey` for remote compact service-tier omission, matching upstream's API-key auth semantics and the fork's API-key compact request-shape contract.
  - `compact_token_budget.rs` preserves `run_manual_compact_task_after_turn_started` for the shared remote-first fallback wrapper while adopting upstream `StepContext` world-state capture.
  - `remote_compact_fallback.rs` routes token-budget auto compaction through the existing `StepContext` so the upstream token-budget API and fork-local remote-first token-budget fallback both remain coherent.
  - `compact_remote.rs` test source keeps the fork fallback sentinels and upstream agent-identity fixture constants.
  - `client_tests.rs` was updated with explicit compact timeout, TCP keepalive, and no-hidden-retry settings for `CompactConversationRequestSettings`.
- Metadata after merge preservation:
  - `upstreamhash.txt`: `cfead68e5d3984b247cf0758e3e53b19165de848`
  - `modversion.txt`: `1`
- Conflict checks:
  - `git diff --name-only --diff-filter=U`: no output.
  - `rg -n "<<<<<<<|=======|>>>>>>>"` on resolved compact files and metadata files: no matches.
  - `git diff --check`: no output.
- Maintenance commands run after resolving conflicts:
  - `cd codex-rs && just fmt`: passed.
  - `cd codex-rs && just write-config-schema`: initially exposed a `StepContext` merge adaptation error in `remote_compact_fallback.rs`; after fixing that, passed with existing non-fatal `codex-api` dead-code warnings.
  - `just bazel-lock-update`: passed as dependency lock maintenance only, with Bazel crate-annotation warnings.
  - `just bazel-lock-check`: passed as dependency lock maintenance only.
  - `cd codex-rs && just fix -p codex-core`: initially exposed a `client_tests.rs` initializer missing compact request settings; after fixing that, passed with existing non-fatal `codex-api` dead-code warnings and known compact remote `too_many_arguments` warnings.
  - `cd codex-rs && just fmt`: passed after the scoped fix command.
- Tests: not run unless explicitly requested.
- Bazel: no Bazel build or test was run; Bazel was used only for dependency lock maintenance commands required by dependency changes.

## Build details
- Build command: `CODEX_CLI_RELEASE_VERSION="${version}" cargo build -p codex-cli --release` from `codex-rs`.
- Expected Linux CLI binary path: `codex-rs/target/release/codex`.
- Expected copied artifact path: `codex`.
- Expected version output: `codex-cli ${version}`.

## Build verification results
- Metadata version recomputed before build:
  - Latest stable upstream release: `0.142.4`
  - Base series: `0.142`
  - Upstream SHA: `cfead68e5d3984b247cf0758e3e53b19165de848`
  - Mod version: `1`
  - Upstream short: `cfead`
  - Computed version: `0.142.cfead.1.mod`
- Formatting command before build: `just fmt` from `codex-rs`; passed.
- Cargo release build command: `CODEX_CLI_RELEASE_VERSION="0.142.cfead.1.mod" cargo build -p codex-cli --release` from `codex-rs`; passed in 11m 39s.
- Build warnings did not fail the release build:
  - `codex-api`: two existing dead-code warnings for encoded stream helpers.
  - `codex-app-server`: one existing `unused_mut` warning.
- Source binary path: `codex-rs/target/release/codex`.
- Copy command: `cp -p codex-rs/target/release/codex codex && chmod +x codex`.
- Repository-root artifact path: `codex`.
- Artifact mode: `755`.
- Artifact size: `1316796640` bytes.
- Artifact SHA-256: `3fc3595bf52cb87b3d3bf2cdec05ae476c2547b58e0610482ab23991aadaa4d8`.
- Artifact version check: `./codex --version` returned exactly `codex-cli 0.142.cfead.1.mod`.
- Build-verifier result: no findings. It verified the metadata-derived version, formatting command, Cargo release command, skipped tests, no Bazel build/test usage, source binary path, root artifact, executable bit, size, SHA-256, and exact `./codex --version` output.
- After recording the build verifier result, the same release build command was rerun on publish HEAD `5d76e420aa31a65dc559dd4e253d0ffbf08a10d2`; it passed in 1.90s with the same non-fatal warnings. The repository-root artifact was refreshed and still reported:
  - `./codex --version`: `codex-cli 0.142.cfead.1.mod`
  - Mode: `755`
  - Size: `1316796640` bytes
  - SHA-256: `3fc3595bf52cb87b3d3bf2cdec05ae476c2547b58e0610482ab23991aadaa4d8`
- Tests: not run unless explicitly requested.
- Bazel: no Bazel build or test was run; release validation used the Cargo release build path only.

## Publish details
- Publish repository: `garyfpga/codex-compact-fix`.
- `gh` repository selection must be explicit: use `--repo garyfpga/codex-compact-fix` for release commands because implicit `gh` repo detection in this checkout resolves to `openai/codex`.
- Artifact path and uploaded asset name: `codex`.
- Tag and title: `${version}`.
- Release notes source: `docs/mod-refresh/plans/2026-06-30-upstream-cfead-release-notes.md`.

## Stop conditions
Stop and ask for direction if:

- Upstream target SHA differs from `cfead68e5d3984b247cf0758e3e53b19165de848`.
- A conflict exposes a compact-fix behavior choice not surfaced in this plan.
- `upstreamhash.txt` or `modversion.txt` is missing, malformed, dirty after staging expectations, or divergent from this plan.
- Build output is missing or `./codex --version` is not exactly `codex-cli ${version}`.
- Release notes, publish repository, tag, artifact path, or existing release state becomes ambiguous.
- `git tag` or `git push` succeeds but a later publish step fails; report local tag, remote tag, and GitHub release state before any retry or recovery.
