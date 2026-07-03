# Mod Refresh Plan: upstream da4c8

## Objective
Refresh the compact-fix fork from `upstream/main`, preserve the fork-local compact and model-picker behavior, build the Linux CLI artifact, and publish the `.mod` release.

## Fresh preflight
- Source: current-session `$mod-refresh-full-release` preflight on 2026-07-03.
- Current branch: `pub/mod-refresh-2026-06-24`.
- Release source HEAD before merge: `6f7530ce539ed2b5da72caa37b8c9a5a036e37a9`.
- Upstream ref: `upstream/main`.
- Upstream target SHA: `da4c8ca57d40b074bdc1b5b1218851100150c56b`.
- Merge base: `cfead68e5d3984b247cf0758e3e53b19165de848`.
- Worktree before preflight simulation: clean.
- Temporary merge simulation: clean merge; merge aborted and temporary worktree removed.
- Recommendation: ready to continue because the user explicitly requested `$mod-refresh-full-release`, the simulated merge is clean, and the compact-fix risks are surfaced below.
- Preflight reviewer: found no merge blocker, but corrected the release suffix from `da4c.1.mod` to `da4c8.1.mod`. This plan uses the corrected first-five-character metadata suffix.
- Full-release handoff: continue into `$mod-refresh-release` with the metadata, no-test, no-Bazel, and preservation risks recorded in this plan.

## Preflight report
### Fetch
- `git fetch upstream main:refs/remotes/upstream/main`: passed; `upstream/main` advanced from `cfead68e5d` to `da4c8ca57d`.
- Upstream target SHA from `git rev-parse upstream/main`: `da4c8ca57d40b074bdc1b5b1218851100150c56b`.
- Merge base from `git merge-base HEAD upstream/main`: `cfead68e5d3984b247cf0758e3e53b19165de848`.
- Fetch concern: none.

### Worktree
- `git status --short`: clean before merge simulation.
- Current branch: `pub/mod-refresh-2026-06-24`.

### Merge simulation
- Simulation command: `git merge --no-commit --no-ff upstream/main` in a temporary detached worktree from HEAD `6f7530ce539ed2b5da72caa37b8c9a5a036e37a9`.
- Simulation result: clean automatic merge, exit 0.
- Conflicted files: none.
- Cleanup: merge aborted and temporary worktree removed.

### Upstream touched files
- Compact-relevant or compact-adjacent upstream touches:
  - `codex-rs/core/config.schema.json`
  - `codex-rs/core/src/client.rs`
  - `codex-rs/core/src/config/config_tests.rs`
  - `codex-rs/core/src/config/mod.rs`
  - `codex-rs/core/src/lib.rs`
  - `codex-rs/protocol/src/config_types.rs`
  - `codex-rs/protocol/src/protocol.rs`
  - generated app-server protocol schema files
  - `codex-rs/Cargo.toml`
  - `codex-rs/Cargo.lock`
  - `MODULE.bazel.lock`
- Narrow compact-path search found no upstream changes to:
  - `codex-rs/core/src/compact.rs`
  - `codex-rs/core/src/compact_remote.rs`
  - `codex-rs/core/src/compact_remote_v2.rs`
  - `codex-rs/core/src/remote_compact_fallback.rs`
  - `codex-rs/core/src/compact_service_tier.rs`
  - `codex-rs/codex-api/src/endpoint/compact.rs`
  - `codex-rs/codex-api/src/endpoint/session.rs`
  - `codex-rs/core/src/responses_retry.rs`
  - `codex-rs/tui/src/version.rs`
  - `codex-rs/tui/src/chatwidget/model_popups.rs`
  - `codex-rs/tui/src/slash_command.rs`
  - `codex-rs/tui/src/chatwidget/slash_dispatch.rs`
  - `codex-rs/tui/tooltips.txt`
- Broad unrelated upstream changes also touched app-server logging tests, exec-server relay/websocket liveness code, Bedrock model metadata, telemetry, safety notice wording, docs, audit and deny config, and TUI safety notice snapshots.

### Compact impact
- Risk level: moderate. The merge simulation is clean and no compact runtime modules are directly changed, but upstream touches shared config, client transport, protocol, generated schema, and dependency lockfiles.
- Config/schema overlap adds `features.multi_agent_v2.multi_agent_mode_hint_text`; preserve `remote_compact` config parsing, validation bounds, defaults, tests, and schema.
- Client overlap changes websocket incremental input matching and TTFT telemetry; preserve compact transport boundaries, compact-specific retry policy, TCP keepalive behavior, and ordinary Responses retry isolation.
- Protocol/app-server generated schema overlap changes `MultiAgentMode` shape for custom hint text; preserve session history compatibility and avoid regressing `/model` and `/modelp` behavior if adjacent paths are affected.
- Dependency overlap bumps `quick-xml` and related lock/audit files; run required dependency lock maintenance after merge if dependency changes survive the merge.
- Preservation source of truth: `docs/compact-fix/ChangeLog.md`.

### Recommendation
- Ready to continue if explicitly requested. The user requested `$mod-refresh-full-release` in this session.

### Continuation gate
- Release mutation proceeds through this `$mod-refresh-release` plan with upstream target SHA `da4c8ca57d40b074bdc1b5b1218851100150c56b`.
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
- Expected `upstreamhash.txt`: `da4c8ca57d40b074bdc1b5b1218851100150c56b`
- Expected `modversion.txt`: `1`
- Latest stable upstream release source checked during planning: `0.142.5`.
- Base series: `0.142`.
- Upstream short: `da4c8`.
- Expected metadata suffix: `da4c8.1.mod`.
- Expected release version: `0.142.da4c8.1.mod`.
- Version contract: `<latest-upstream-major>.<latest-upstream-minor>.<first5-upstreamhash>.<modversion>.mod`.
- Computed release version handoff: `version="${base_series}.${upstream_short}.${mod_version}.mod"` must be passed into the build as `CODEX_CLI_RELEASE_VERSION="${version}"`.
- Build handoff: `CODEX_CLI_RELEASE_VERSION="${version}" cargo build -p codex-cli --release`.
- Do not derive the suffix from final `HEAD`, `git rev-parse --short`, or the upstream SemVer patch component.

## Execution checklist
0. Plan artifact gate:
   - Review this plan with `release-plan-reviewer` using the same model as the main agent and `reasoning_effort = high`.
   - Review the full-release handoff with `full-release-chain-reviewer` using the same model as the main agent and `reasoning_effort = high`.
   - Resolve reviewer findings before mutating.
   - Commit this plan and its release notes before the real merge so `git status --short` is clean for merge preservation.
   - After committing this plan and its release notes, verify the only delta from `6f7530ce539ed2b5da72caa37b8c9a5a036e37a9` is the two planning files. Stop and refresh preflight if any other source, metadata, artifact, or release-state change appears.
1. Merge preservation:
   - Reconfirm branch, clean worktree, upstream remote, upstream target SHA, and metadata expectations.
   - Run `git fetch upstream main` and confirm `git rev-parse upstream/main` is still `da4c8ca57d40b074bdc1b5b1218851100150c56b`.
   - Read `docs/compact-fix/ChangeLog.md` again and treat it as the source of truth for every overlap.
   - Validate expected metadata shapes before merge: `upstreamhash.txt` value is one full 40-character lowercase hex SHA line and `modversion.txt` value is one positive decimal integer line.
   - Run the real `git merge upstream/main`.
   - Inventory conflicts with `git status --short` and `git diff --name-only --diff-filter=U`.
   - Use `merge-conflict-worker` after the real conflict inventory. If there are no conflicts, ask it to review the clean merge diff for compact-preservation risks.
   - Resolve any conflicts by preserving the compact-fix behavior groups above.
   - Update `upstreamhash.txt` and `modversion.txt` to the expected values.
   - After merge preservation and before build or publish, verify actual metadata files exactly match this plan:
     - `upstreamhash.txt`: `da4c8ca57d40b074bdc1b5b1218851100150c56b`
     - `modversion.txt`: `1`
     - Stop if either file is missing, malformed, dirty after staging expectations, or divergent from this plan.
   - Run required non-test maintenance only.
   - If Rust dependency changes survive the merge, run and record `just bazel-lock-update` from the repository root as dependency lock maintenance required by repository policy. This is not a Bazel build, Bazel test, or release build path, and it does not change the recorded `Bazel: not used; using Cargo release build only` decision.
   - Run `cd codex-rs && just fmt` after code changes.
   - Use `compact-preservation-reviewer` on the final merge diff before reporting merge completion.
   - Stage and commit the completed merge, metadata, formatting, and maintenance results before build handoff.
   - Verify `git status --short` is clean after the merge preservation commit.
   - Set `final_commit="$(git rev-parse HEAD)"` only after the clean post-merge source commit exists. Stop if metadata, source, schema, or lockfile changes remain uncommitted.
   - Do not run tests unless explicitly requested.
   - Do not run Bazel build or Bazel test unless explicitly requested.
2. Build verification:
   - From the repository root, recompute latest stable upstream release and metadata version, and stop if the result is not exactly `0.142.da4c8.1.mod`.
   - Build with `CODEX_CLI_RELEASE_VERSION="${version}" cargo build -p codex-cli --release` from `codex-rs`.
   - Return to the repository root with `cd "$(git rev-parse --show-toplevel)"`.
   - Copy `codex-rs/target/release/codex` to repository-root `codex`.
   - Verify `./codex --version` is exactly `codex-cli ${version}`.
   - Record artifact mode, size, SHA-256, and version output in this plan.
   - Use `build-verifier` after the artifact is copied and before publish handoff.
   - Stage and commit build verification notes and any intended artifact refresh before publish handoff.
   - Verify `git status --short` is clean except for the repository-root `codex` artifact if that artifact is intentionally untracked in this checkout; publish must still verify the artifact path directly.
   - Refresh `final_commit="$(git rev-parse HEAD)"` after the final committed release source state is clean.
3. Publish:
   - From the repository root, recompute and revalidate version from checked-in metadata, and stop if the result is not exactly `0.142.da4c8.1.mod`.
   - Confirm no local tag exists for `${version}` with `git rev-parse -q --verify "refs/tags/${version}"`.
   - Confirm no remote tag exists for `${version}` with `git ls-remote --exit-code --tags origin "refs/tags/${version}"`, treating exit code `2` as the expected missing state and any other nonzero result as ambiguous.
   - Confirm no GitHub release exists for `${version}` with `gh release view "${version}" --repo garyfpga/codex-compact-fix`, treating only a clear not-found response as the expected missing state.
   - Confirm repository-root `codex` exists, is executable, and reports the expected version from the repository root.
   - Use `release-packaging-reviewer` before creating any tag or GitHub release.
   - Tag final HEAD with `git tag -a "${version}" "${final_commit}" -m "${version}"`.
   - Push `refs/tags/${version}` to `origin` with `git push origin "refs/tags/${version}"`.
   - Create the GitHub release and upload the repository-root artifact with:
     `gh release create "${version}" "$(git rev-parse --show-toplevel)/codex" --repo garyfpga/codex-compact-fix --verify-tag --title "${version}" --notes-file "$(git rev-parse --show-toplevel)/docs/mod-refresh/plans/2026-07-03-upstream-da4c8-release-notes.md"`.
   - If `git tag`, `git push`, or `gh release create` partially succeeds and a later step fails, inspect and report local tag, remote tag, and GitHub release state before any retry or recovery.

## Test and Bazel decisions
- Tests: not run unless explicitly requested.
- Bazel: not used; using Cargo release build only.

## Merge preservation results
- Real merge commit: `fa5b27421cc0ad89c60621db0d3ccccbdc5f4dab`.
- Upstream merged: `upstream/main` at `da4c8ca57d40b074bdc1b5b1218851100150c56b`.
- Merge command: `git merge upstream/main`.
- Conflicts: none.
- Merge worker result: no missed compact-fix behavior choice; no required source edit beyond metadata, formatting, and dependency lock maintenance.
- Preservation summary:
  - Compact runtime modules, compact endpoints, `responses_retry.rs`, TUI version display, and `/model`/`/modelp` routing were not directly touched by upstream.
  - `remote_compact` config/schema/tests remain intact.
  - The shared remote-first fallback remains the compact policy owner.
  - V1/V2 retry, timeout, service-tier, and TCP keepalive paths are unchanged.
  - Upstream protocol/schema changes around `MultiAgentMode` match the preflight risk and do not create a compact-preservation blocker.
- Metadata after merge preservation:
  - `upstreamhash.txt`: `da4c8ca57d40b074bdc1b5b1218851100150c56b`
  - `modversion.txt`: `1`
- Conflict checks:
  - `git diff --name-only --diff-filter=U`: no output.
  - `git diff --check`: no output.
- Maintenance commands run after merge:
  - `just bazel-lock-update`: passed as dependency lock maintenance only, with non-fatal crate-annotation warnings.
  - `cd codex-rs && just fmt`: passed.
- Tests: not run unless explicitly requested.
- Bazel: no Bazel build or test was run; Bazel was used only for dependency lock maintenance required by dependency changes.

## Build details
- Build command: `CODEX_CLI_RELEASE_VERSION="${version}" cargo build -p codex-cli --release` from `codex-rs`.
- Expected Linux CLI binary path: `codex-rs/target/release/codex`.
- Expected copied artifact path: `codex`.
- Expected version output: `codex-cli ${version}`.

## Publish details
- Publish repository: `garyfpga/codex-compact-fix`.
- `gh` repository selection must be explicit: use `--repo garyfpga/codex-compact-fix` for release commands because implicit `gh` repo detection in this checkout may resolve to `openai/codex`.
- Artifact path and uploaded asset name: `codex`.
- Tag and title: `${version}`.
- Release notes source: `docs/mod-refresh/plans/2026-07-03-upstream-da4c8-release-notes.md`.

## Stop conditions
Stop and ask for direction if:

- Upstream target SHA differs from `da4c8ca57d40b074bdc1b5b1218851100150c56b`.
- A conflict or clean-merge overlap exposes a compact-fix behavior choice not surfaced in this plan.
- `upstreamhash.txt` or `modversion.txt` is missing, malformed, dirty after staging expectations, or divergent from this plan.
- Build output is missing or `./codex --version` is not exactly `codex-cli ${version}`.
- Release notes, publish repository, tag, artifact path, or existing release state becomes ambiguous.
- `git tag` or `git push` succeeds but a later publish step fails; report local tag, remote tag, and GitHub release state before any retry or recovery.
