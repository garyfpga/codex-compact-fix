# Mod Version And Trust-All Fixes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `simplepower:subagent-driven-development` for aggregate parallel implementation. Dispatch all non-conflicting `sp-impl` file-edit workers whose coordination needs are satisfied by the approved Interface Contract, run the quick verifier after all workers finish, commit the quick-verified implementation, then run one REVIEW-tier review+fix agent before final verification and final commit.

**Goal:** Make the generated `codex` binary and TUI display the metadata-based `.mod` release version, and make `dangerously_trust_all_projects = true` trust project-local `.codex` layers early enough to avoid disabled-project warnings.

**Design Summary:** The approved design fixes two regressions from the last Simple Power implementation. The version flow uses one build-time environment contract, `CODEX_CLI_RELEASE_VERSION`, whose value is computed by the mod refresh release skills from `upstreamhash.txt`, `modversion.txt`, and the latest stable upstream release series, for example `0.141.c7329.1.mod`; both `./codex --version` and TUI status/display surfaces must use that value when present, while local development builds fall back to `env!("CARGO_PKG_VERSION")`. The trust flow moves `dangerously_trust_all_projects` into the config-layer trust decision so unmatched projects are considered trusted before project-local config, hooks, and exec policies are loaded; explicit trusted or untrusted `[projects]` entries remain authoritative. The user explicitly approved no local Rust test runs and no binary generation in this implementation pass; use formatting, static assertions, diff inspection, and review instead.

**Architecture:** Keep the release version source of truth in the existing release metadata files and release skills, not in a new checked-in generated version file. Rust exposes a small compile-time version constant from the `CODEX_CLI_RELEASE_VERSION` environment variable with a Cargo package-version fallback. Config loading extends the existing `ProjectTrustContext` so layer enablement and later `active_project` resolution agree before the TUI warning code sees disabled project layers.

**Tech Stack:** Rust, Clap derive, Codex config loader, TUI status snapshots, Markdown Simple Power skill documents, shell release commands.

**Model Allocation:** FAST/NORMAL/BEST/REVIEW tiers are assigned below. Resolve each tier by explicit user override, quoted assignment in project root AGENTS.md, process environment variable, then built-in default. The project root AGENTS.md lookup reads only `<repo>/AGENTS.md`, not nested AGENTS.md files or repo-wide grep. FAST defaults to `SIMPLEPOWER_FAST_MODEL` (`gpt-5.3-codex-spark-high` when unset), NORMAL defaults to `SIMPLEPOWER_NORMAL_MODEL` (`gpt-5.4-mini-high` when unset), BEST defaults to `SIMPLEPOWER_BEST_MODEL` (`gpt-5.5-high` when unset), and REVIEW defaults to `SIMPLEPOWER_REVIEW_MODEL` (`gpt-5.5-xhigh` when unset). The plan reviewer is a REVIEW-tier plan reviewer, and the final review+fix agent is a REVIEW-tier review+fix agent. The quick verifier uses the FAST tier by default, resolving to `model="gpt-5.3-codex-spark"` and `reasoning_effort="high"` unless `SIMPLEPOWER_FAST_MODEL` is overridden.

**Commit Policy:** The coordinator commits after the reviewed plan, allocation, and immediate current-session execution receive combined approval, after all file edits and quick verification complete before final review, and after final review/fix plus final verification. Workers, plan reviewers, quick verifiers, and review+fix agents must not commit. No per-task commits. Coordinator-owned temporary scratch refs under `refs/simplepower/scratch/<run-id>/...` may be created only as local review diff anchors; they are not accepted history commits, not pushed, not merged, not rebased, and must be cleaned up after successful checkpoints or reported for manual cleanup on blockers or failed checkpoints.

---

## Interface Contract

1. **Release version environment contract**
   - Environment variable name: `CODEX_CLI_RELEASE_VERSION`.
   - Value shape: `<latest-upstream-major>.<latest-upstream-minor>.<first5-upstreamhash>.<modversion>.mod`, for example `0.141.c7329.1.mod`.
   - The value is computed by release skills from:
     - latest stable upstream release base series from `gh release list --repo openai/codex --exclude-drafts --exclude-pre-releases --limit 1`;
     - repository-root `upstreamhash.txt`, exactly one full lowercase 40-character SHA line;
     - repository-root `modversion.txt`, exactly one positive decimal integer line.
   - The value must not come from final `HEAD`, `git rev-parse --short`, the upstream SemVer patch component, or hardcoded `+gary` labels.
   - The release build command must pass the variable to Cargo, for example:
     ```bash
     CODEX_CLI_RELEASE_VERSION="${version}" cargo build -p codex-cli --release
     ```

2. **Rust version constants**
   - `codex-rs/tui/src/version.rs` must define `CODEX_CLI_VERSION` as:
     ```rust
     pub const CODEX_CLI_VERSION: &str = match option_env!("CODEX_CLI_RELEASE_VERSION") {
         Some(version) => version,
         None => env!("CARGO_PKG_VERSION"),
     };
     ```
   - `CODEX_CLI_DISPLAY_VERSION` may remain for existing TUI call sites, but it must be an alias to `CODEX_CLI_VERSION`, not a hardcoded `0.141.0+gary` or other release-specific literal.
   - `codex-rs/cli/src/main.rs` must provide the same compile-time fallback expression for the Clap version value and wire `MultitoolCli` to that constant so `./codex --version` reports the `.mod` value in release builds and Cargo package version in local builds.

3. **Version user-visible behavior**
   - Release build with `CODEX_CLI_RELEASE_VERSION=0.141.c7329.1.mod`: `./codex --version` reports `codex-cli 0.141.c7329.1.mod`, and TUI status/preview surfaces show `0.141.c7329.1.mod`.
   - Local build without `CODEX_CLI_RELEASE_VERSION`: `./codex --version` and TUI status/preview surfaces use `env!("CARGO_PKG_VERSION")`; in this workspace that is currently `0.0.0`.
   - The TUI snapshot for `status_surface_previews_codex_version` must match the local-build fallback unless the test harness is explicitly changed to compile with `CODEX_CLI_RELEASE_VERSION`. This plan does not require such a harness change.

4. **Trust-all config-layer contract**
   - `dangerously_trust_all_projects = true` in merged user/profile/runtime config means unmatched projects are treated as trusted during project-layer loading.
   - Explicit `[projects."<path>"].trust_level = "trusted"` and `trust_level = "untrusted"` entries remain authoritative and are checked before the trust-all fallback.
   - When trust-all applies, project-local `.codex` config, hooks, and exec policies must load as enabled layers, and `ConfigLayerEntry.disabled_reason` must be `None`.
   - Because disabled project layers are what drive the TUI/app-server startup warning, trust-all must prevent the warning by preventing the disabled layer state, not by filtering warning display.

5. **No local Rust test or build execution in this run**
   - Implementation workers may add or adjust Rust tests and snapshots as source artifacts.
   - Implementation workers, quick verifier, final review+fix agent, and coordinator must not run `just test`, `cargo test`, `cargo insta`, `cargo build`, release build commands, or generate the `codex` binary unless the user gives fresh explicit approval.
   - Allowed local verification commands in this run are formatting (`just fmt` in `codex-rs`), static `rg` checks, shell text assertions, diff inspection, and review.

## File Ownership

| File | Owner task | Change type | Responsibility | Parallel safety notes |
|------|------------|-------------|----------------|-----------------------|
| `codex-rs/cli/src/main.rs` | Version surfaces | modify | Add `CODEX_CLI_RELEASE_VERSION` fallback constant and wire Clap version output to it. | Exclusive to Version surfaces. |
| `codex-rs/tui/src/version.rs` | Version surfaces | modify | Replace hardcoded `0.141.0+gary` display label with the release-env-aware version constant and display alias. | Exclusive to Version surfaces. |
| `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__status_surface_previews_codex_version.snap` | Version surfaces | modify | Update expected local-build status/terminal-title version output. | Exclusive to Version surfaces. |
| `codex-rs/config/src/loader/mod.rs` | Trust-all loader | modify | Add trust-all fallback to `ProjectTrustContext` and project-layer trust decisions. | Exclusive to Trust-all loader. |
| `codex-rs/core/src/config/config_loader_tests.rs` | Trust-all loader | modify | Add/adjust source tests for trusted project-layer behavior under trust-all and explicit untrusted precedence. | Exclusive to Trust-all loader. |
| `.codex/skills/mod-refresh-build/SKILL.md` | Release skill version contract | modify | Compute release metadata version before build and pass `CODEX_CLI_RELEASE_VERSION` into Cargo. | Exclusive to Release skill version contract. |
| `.codex/skills/mod-refresh-publish/SKILL.md` | Release skill version contract | modify | Replace the `+gary` display gate with binary/TUI `.mod` version verification. | Exclusive to Release skill version contract. |
| `.codex/skills/mod-refresh-merge-preserve/SKILL.md` | Release skill version contract | modify | Preserve metadata-based version display wording in future merge refreshes. | Exclusive to Release skill version contract. |
| `.codex/skills/mod-refresh-release/SKILL.md` | Release skill version contract | modify | Ensure release plans record that the computed `.mod` version must be passed into the build. | Exclusive to Release skill version contract. |
| `.codex/skills/mod-release-current/SKILL.md` | Release skill version contract | modify | Ensure current-HEAD release handoff expects the build skill to embed the checked-in metadata version. | Exclusive to Release skill version contract. |
| `docs/compact-fix/ChangeLog.md` | Preservation docs | modify | Replace `0.141.0+gary` preservation wording with metadata-based `.mod` version display behavior. | Exclusive to Preservation docs; compatible with code and skill edits. |

## Implementation Tasks

### Version surfaces

- **Goal:** Make `./codex --version` and TUI version display use `CODEX_CLI_RELEASE_VERSION` in release builds with Cargo package-version fallback.
- **Contract inputs:** Interface Contract entries 1, 2, 3, and 5.
- **Serialization required:** No.
- **Write scope:** `codex-rs/cli/src/main.rs`, `codex-rs/tui/src/version.rs`, `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__status_surface_previews_codex_version.snap`.
- **Parallel:** Yes, compatible with Trust-all loader, Release skill version contract, and Preservation docs.
- **Risk:** Medium, because it changes user-visible CLI and TUI version output but is localized.
- **Model tier:** NORMAL, resolved to `model="gpt-5.4-mini"` and `reasoning_effort="xhigh"`.
- **Worker role:** `sp-impl`.
- **Outputs and file-level responsibilities:** Updated CLI Clap version source, TUI version constants, and local-build snapshot expectation.
- **Implementation steps:**
  1. In `codex-rs/cli/src/main.rs`, add a private constant near `MultitoolCli`:
     ```rust
     const CODEX_CLI_VERSION: &str = match option_env!("CODEX_CLI_RELEASE_VERSION") {
         Some(version) => version,
         None => env!("CARGO_PKG_VERSION"),
     };
     ```
  2. Change the `MultitoolCli` derive attribute from `version` to `version = CODEX_CLI_VERSION` while preserving existing `author`, `bin_name`, and usage attributes.
  3. In `codex-rs/tui/src/version.rs`, define `CODEX_CLI_VERSION` with the same `option_env!` fallback expression.
  4. Keep `CODEX_CLI_DISPLAY_VERSION` only as an alias:
     ```rust
     pub const CODEX_CLI_DISPLAY_VERSION: &str = CODEX_CLI_VERSION;
     ```
     Do not leave any hardcoded `0.141.0+gary` literal.
  5. Update `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__status_surface_previews_codex_version.snap` to the local fallback currently produced by `env!("CARGO_PKG_VERSION")`, expected `0.0.0` for both `status line` and `terminal title`.
- **Verification commands the worker should run:**
  - `timeout 30s rg -n "CODEX_CLI_RELEASE_VERSION|CODEX_CLI_DISPLAY_VERSION|0\\.141\\.0\\+gary" codex-rs/cli/src/main.rs codex-rs/tui/src/version.rs codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__status_surface_previews_codex_version.snap`
  - `timeout 30s bash -lc '! rg -n "0\\.141\\.0\\+gary" codex-rs/cli/src/main.rs codex-rs/tui/src/version.rs codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__status_surface_previews_codex_version.snap'`
  - Do not run Rust tests, snapshot commands, Cargo build commands, or generate `codex`.
- **Completion report requirements:** Changed files, exact version constants used, snapshot expected value, verification commands and results, and any uncertainty about Clap attribute syntax.

### Trust-all loader

- **Goal:** Make `dangerously_trust_all_projects = true` affect project-layer trust decisions before `.codex` layers are disabled.
- **Contract inputs:** Interface Contract entries 4 and 5.
- **Serialization required:** No.
- **Write scope:** `codex-rs/config/src/loader/mod.rs`, `codex-rs/core/src/config/config_loader_tests.rs`.
- **Parallel:** Yes, compatible with Version surfaces, Release skill version contract, and Preservation docs.
- **Risk:** High, because this changes config trust behavior and must preserve explicit untrusted precedence.
- **Model tier:** BEST, resolved to `model="gpt-5.5"` and `reasoning_effort="xhigh"`.
- **Worker role:** `sp-impl`.
- **Outputs and file-level responsibilities:** Loader trust-all fallback and source tests documenting enabled/disabled project-layer behavior.
- **Implementation steps:**
  1. In `codex-rs/config/src/loader/mod.rs`, extend `ProjectTrustConfigToml` with:
     ```rust
     dangerously_trust_all_projects: Option<bool>,
     ```
  2. Add `dangerously_trust_all_projects: bool` to `ProjectTrustContext`.
  3. In `project_trust_context`, after parsing `ProjectTrustConfigToml`, store `dangerously_trust_all_projects.unwrap_or(false)` in the context.
  4. In `ProjectTrustContext::decision_for_dir`, keep the existing exact-dir, project-root, and repo-root explicit trust lookups first. After those lookups and before returning `trust_level: None`, add a trust-all fallback that returns `trust_level: Some(TrustLevel::Trusted)` with the same fallback `trust_key` the unknown-project path already uses.
  5. Do not change `disabled_reason_for_decision`; trust-all should work because the decision is trusted.
  6. In `codex-rs/core/src/config/config_loader_tests.rs`, add or adjust tests so source coverage documents:
     - unknown project with `.codex/config.toml` and user config `dangerously_trust_all_projects = true` loads the project layer with `disabled_reason == None` and project config present in effective config;
     - explicit `[projects."<project>"].trust_level = "untrusted"` still disables the project layer even when `dangerously_trust_all_projects = true`;
     - `.codex` folder without `config.toml` is not disabled when trust-all is true.
  7. Use `pretty_assertions::assert_eq` if the touched test module already imports it or local style supports it; otherwise follow existing file-local assertion style.
- **Verification commands the worker should run:**
  - `timeout 30s rg -n "dangerously_trust_all_projects|ProjectTrustContext|disabled_reason" codex-rs/config/src/loader/mod.rs codex-rs/core/src/config/config_loader_tests.rs`
  - `timeout 30s bash -lc '! rg -n "dangerously_trust_all_projects.*TODO|TODO.*dangerously_trust_all_projects" codex-rs/config/src/loader/mod.rs codex-rs/core/src/config/config_loader_tests.rs'`
  - Do not run Rust tests.
- **Completion report requirements:** Changed files, explicit precedence behavior, tests added/adjusted but not run, verification commands and results, and remaining risks.

### Release skill version contract

- **Goal:** Update mod refresh/current release skills so the computed `.mod` version is embedded into the built binary and verified before publish.
- **Contract inputs:** Interface Contract entries 1, 3, and 5.
- **Serialization required:** No.
- **Write scope:** `.codex/skills/mod-refresh-build/SKILL.md`, `.codex/skills/mod-refresh-publish/SKILL.md`, `.codex/skills/mod-refresh-merge-preserve/SKILL.md`, `.codex/skills/mod-refresh-release/SKILL.md`, `.codex/skills/mod-release-current/SKILL.md`.
- **Parallel:** Yes, compatible with Version surfaces, Trust-all loader, and Preservation docs.
- **Risk:** High, because release skill wording controls future build and publish behavior.
- **Model tier:** BEST, resolved to `model="gpt-5.5"` and `reasoning_effort="xhigh"`.
- **Worker role:** `sp-impl`.
- **Outputs and file-level responsibilities:** Skill docs compute/pass `CODEX_CLI_RELEASE_VERSION`, remove stale `+gary` gates, and verify binary/TUI version consistency with the metadata release version.
- **Implementation steps:**
  1. In `.codex/skills/mod-refresh-build/SKILL.md`, before the Cargo build command, add the metadata-version computation from the publish skill: latest stable upstream SemVer base series, exact validation of `upstreamhash.txt` and `modversion.txt`, `upstream_short="$(cut -c1-5)"`, and `version="${base_series}.${upstream_short}.${mod_version}.mod"`.
  2. Change the build command example to pass `CODEX_CLI_RELEASE_VERSION="${version}" cargo build -p codex-cli --release`.
  3. Update build verification/completion text to record the computed version and require the copied artifact's `./codex --version` output to match `codex-cli ${version}` when the build actually runs in a release workflow.
  4. In `.codex/skills/mod-refresh-publish/SKILL.md`, remove the `expected_display_version="${base_semver}+gary"` gate and replace it with checks that the repository-root artifact exists, is executable, and reports `codex-cli ${version}` from `./codex --version`.
  5. In the publish reviewer checklist, replace the TUI display label `+gary` item with a `.mod` metadata-version item.
  6. In `.codex/skills/mod-refresh-release/SKILL.md`, add to the release plan contents and execution checklist that the computed `.mod` version must be passed to the build through `CODEX_CLI_RELEASE_VERSION`.
  7. In `.codex/skills/mod-release-current/SKILL.md`, add to the handoff/reviewer checks that the build embeds the checked-in metadata version with `CODEX_CLI_RELEASE_VERSION`.
  8. In `.codex/skills/mod-refresh-merge-preserve/SKILL.md`, update preservation wording from display-only `+gary` to metadata-based `.mod` display/version behavior.
  9. Do not add wording that allows deriving the version from final `HEAD`, `git rev-parse --short`, or the upstream SemVer patch component.
- **Verification commands the worker should run:**
  - `timeout 30s rg -n "CODEX_CLI_RELEASE_VERSION|version=\"\\$\\{base_series\\}\\.\\$\\{upstream_short\\}\\.\\$\\{mod_version\\}\\.mod\"|\\./codex --version|0\\.141\\.0\\+gary|\\+gary" .codex/skills/mod-refresh-build/SKILL.md .codex/skills/mod-refresh-publish/SKILL.md .codex/skills/mod-refresh-merge-preserve/SKILL.md .codex/skills/mod-refresh-release/SKILL.md .codex/skills/mod-release-current/SKILL.md`
  - `timeout 30s bash -lc '! rg -n "expected_display_version|0\\.141\\.0\\+gary|\\+gary" .codex/skills/mod-refresh-build/SKILL.md .codex/skills/mod-refresh-publish/SKILL.md .codex/skills/mod-refresh-merge-preserve/SKILL.md .codex/skills/mod-refresh-release/SKILL.md .codex/skills/mod-release-current/SKILL.md'`
- **Completion report requirements:** Changed files, exact release-version command contract, removed stale gates, verification commands and results, and remaining release-process risks.

### Preservation docs

- **Goal:** Update the compact-fix changelog so future merge preservation follows metadata-based `.mod` version display instead of the obsolete `+gary` label.
- **Contract inputs:** Interface Contract entries 1, 2, and 3.
- **Serialization required:** No.
- **Write scope:** `docs/compact-fix/ChangeLog.md`.
- **Parallel:** Yes, compatible with Version surfaces, Trust-all loader, and Release skill version contract.
- **Risk:** Medium, because this documentation guides future merge preservation but does not alter runtime behavior.
- **Model tier:** NORMAL, resolved to `model="gpt-5.4-mini"` and `reasoning_effort="xhigh"`.
- **Worker role:** `sp-impl`.
- **Outputs and file-level responsibilities:** Changelog wording no longer preserves `0.141.0+gary`; it preserves metadata-based `.mod` version display and release env embedding.
- **Implementation steps:**
  1. Replace references that describe the preserved TUI version label as `0.141.0+gary` with wording that describes `CODEX_CLI_RELEASE_VERSION` and the metadata-based `.mod` version.
  2. Update the preservation checklist item so future merges preserve binary/TUI `.mod` version consistency.
  3. Keep historical plan references intact unless the sentence claims the current expected behavior is `+gary`.
- **Verification commands the worker should run:**
  - `timeout 30s rg -n "CODEX_CLI_RELEASE_VERSION|metadata-based|0\\.141\\.0\\+gary|\\+gary" docs/compact-fix/ChangeLog.md`
  - `timeout 30s bash -lc '! rg -n "current.*\\+gary|preserve.*\\+gary|0\\.141\\.0\\+gary" docs/compact-fix/ChangeLog.md'`
- **Completion report requirements:** Changed sections, wording summary, verification commands and results, and any historical references intentionally left unchanged.

## Model Allocation

| Stage | Role | Model tier | Resolved model | Reasoning effort | Reason |
|-------|------|------------|----------------|------------------|--------|
| Plan review | REVIEW-tier plan reviewer | REVIEW | `gpt-5.5` | `xhigh` | Required by writing-plans; verifies plan completeness and aggregate parallel readiness. |
| Version surfaces | `sp-impl` | NORMAL | `gpt-5.4-mini` | `xhigh` | Localized Rust and snapshot update with moderate user-visible risk. |
| Trust-all loader | `sp-impl` | BEST | `gpt-5.5` | `xhigh` | Behavior-shaping config trust logic with security-sensitive precedence. |
| Release skill version contract | `sp-impl` | BEST | `gpt-5.5` | `xhigh` | High-risk release process wording controls future build and publish behavior. |
| Preservation docs | `sp-impl` | NORMAL | `gpt-5.4-mini` | `xhigh` | Focused documentation update that guides future preservation. |
| Quick verifier | FAST-tier verifier | FAST | `gpt-5.3-codex-spark` | `high` | Static checks and formatting only; no Rust tests or builds per user approval. |
| Final review+fix | REVIEW-tier review+fix agent | REVIEW | `gpt-5.5` | `xhigh` | Required whole-change review and allowed in-scope fixes before final verification. |

Resolved model sources for this plan:
- Project root `AGENTS.md` was checked only at `<repo>/AGENTS.md`; it did not define quoted Simple Power model-tier assignments.
- Process environment provided `SIMPLEPOWER_FAST_MODEL=gpt-5.3-codex-spark-high`, `SIMPLEPOWER_NORMAL_MODEL=gpt-5.4-mini-xhigh`, `SIMPLEPOWER_BEST_MODEL=gpt-5.5-xhigh`, and `SIMPLEPOWER_REVIEW_MODEL=gpt-5.5-xhigh`.

## Plan Review

Self-review checklist result:
- Design Summary captures the approved brainstorming decisions: both CLI and TUI use metadata `.mod` release versions, trust-all applies to project-layer loading, explicit untrusted remains authoritative, no binary generation, and no local Rust tests.
- Interface Contract is before File Ownership and gives concrete env var, Rust constants, release command, trust-layer behavior, and verification constraints.
- File Ownership lists every planned modified file exactly once with no parallel write collisions.
- Every implementation task has Contract inputs, Serialization required, exact write scope, model tier, verification commands, and completion reporting.
- Aggregate parallel dispatch is ready because all file-edit tasks have non-overlapping write scopes and rely on the Interface Contract.
- Visual aids are omitted because they would not reduce ambiguity for these code and skill-doc changes.
- Model allocation uses FAST/NORMAL/BEST/REVIEW, resolves by the required precedence, and uses REVIEW for plan review and final review+fix.
- Commit policy has exactly three coordinator checkpoints and no worker commits.
- Scratch-ref guidance is included below for plan review, quick verification, review+fix, cleanup, and blocker preservation.
- Verification commands are concrete and use `timeout`, while honoring the user-approved no-Rust-test/no-build constraint.
- Approved path enforcement does not authorize fallback implementations, skipped review, skipped formatting, docs-only substitutes, or execution-route changes.

Before first review, the coordinator creates `refs/simplepower/scratch/<run-id>/plan-review/before` for this saved plan file using the temporary-index pattern. The run id format is `YYYYMMDD-HHMMSS-<short-head>`, for example `20260618-170000-fa1cdc6`.

Reviewer dispatch prompt must use `skills/writing-plans/plan-document-reviewer-prompt.md` with:
- Plan path: `docs/simplepower/plans/2026-06-18-mod-version-and-trust-all-fixes.md`
- Approved brainstorming design context: both CLI and TUI version surfaces use metadata `.mod` version from `CODEX_CLI_RELEASE_VERSION`; trust-all applies in project-layer loader; explicit untrusted remains authoritative; no local Rust tests or binary generation; verification is format/static checks/review.
- Scratch run id and `plan-review/before` ref.

If the reviewer reports issues, the coordinator edits only the plan, creates `refs/simplepower/scratch/<run-id>/plan-review/after-<n>`, and sends the same reviewer this diff command:

```bash
git diff refs/simplepower/scratch/<run-id>/plan-review/before refs/simplepower/scratch/<run-id>/plan-review/after-<n> -- docs/simplepower/plans/2026-06-18-mod-version-and-trust-all-fixes.md
```

For subsequent revisions, compare the previous `after-<n>` ref to the new one:

```bash
git diff refs/simplepower/scratch/<run-id>/plan-review/after-<n> refs/simplepower/scratch/<run-id>/plan-review/after-<n+1> -- docs/simplepower/plans/2026-06-18-mod-version-and-trust-all-fixes.md
```

The REVIEW-tier plan reviewer must perform the review directly in the current worker. Do not run Codex CLI. Do not spawn subagents. Do not invoke Simple Power skills. Do not restart execution. Do not reroute the workflow.

## Quick Verification

Quick verification runs after all file-edit workers complete and before the quick-verified implementation checkpoint. The quick verifier uses FAST by default, resolved here to `model="gpt-5.3-codex-spark"` and `reasoning_effort="high"`.

Before dispatching the quick verifier, the coordinator creates `refs/simplepower/scratch/<run-id>/quick-verifier/before` for all approved implementation files. If the quick verifier makes tiny typo-level fixes, the coordinator creates `refs/simplepower/scratch/<run-id>/quick-verifier/after` and inspects or hands off:

```bash
git diff refs/simplepower/scratch/<run-id>/quick-verifier/before refs/simplepower/scratch/<run-id>/quick-verifier/after -- codex-rs/cli/src/main.rs codex-rs/tui/src/version.rs codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__status_surface_previews_codex_version.snap codex-rs/config/src/loader/mod.rs codex-rs/core/src/config/config_loader_tests.rs .codex/skills/mod-refresh-build/SKILL.md .codex/skills/mod-refresh-publish/SKILL.md .codex/skills/mod-refresh-merge-preserve/SKILL.md .codex/skills/mod-refresh-release/SKILL.md .codex/skills/mod-release-current/SKILL.md docs/compact-fix/ChangeLog.md
```

Quick verifier may fix only tiny typo-level errors. Any behavior change, structural edit, test rewrite, public interface change, or unclear issue must be reported to the coordinator instead of fixed.

Quick verification commands:

```bash
timeout 120s bash -lc 'cd codex-rs && just fmt'
timeout 30s bash -lc '! rg -n "0\\.141\\.0\\+gary|expected_display_version|\\+gary" codex-rs/cli/src/main.rs codex-rs/tui/src/version.rs .codex/skills/mod-refresh-build/SKILL.md .codex/skills/mod-refresh-publish/SKILL.md .codex/skills/mod-refresh-merge-preserve/SKILL.md .codex/skills/mod-refresh-release/SKILL.md .codex/skills/mod-release-current/SKILL.md docs/compact-fix/ChangeLog.md'
timeout 30s rg -n "CODEX_CLI_RELEASE_VERSION|CODEX_CLI_VERSION|version = CODEX_CLI_VERSION|dangerously_trust_all_projects|ProjectTrustContext|\\./codex --version" codex-rs/cli/src/main.rs codex-rs/tui/src/version.rs codex-rs/config/src/loader/mod.rs codex-rs/core/src/config/config_loader_tests.rs .codex/skills/mod-refresh-build/SKILL.md .codex/skills/mod-refresh-publish/SKILL.md .codex/skills/mod-refresh-release/SKILL.md .codex/skills/mod-release-current/SKILL.md
timeout 30s rg -n "CODEX_CLI_RELEASE_VERSION|metadata-based|\\.mod" docs/compact-fix/ChangeLog.md
timeout 30s bash -lc 'upstream_sha="$(tr -d "[:space:]" < upstreamhash.txt)"; mod_version="$(tr -d "[:space:]" < modversion.txt)"; case "$upstream_sha" in [0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f]) ;; *) exit 1 ;; esac; case "$mod_version" in ""|0|*[!0-9]*) exit 1 ;; esac; test "0.141.$(printf "%s" "$upstream_sha" | cut -c1-5).${mod_version}.mod" = "0.141.c7329.1.mod"'
```

Expected result: commands exit 0. Failure means formatting changed files, stale `+gary` gates remain, the version/trust contract is not visible in source, or release metadata shape is not as expected. Do not run Rust tests or build commands.

## Final Review And Fix

After the quick-verified implementation checkpoint, dispatch exactly one REVIEW-tier review+fix agent using `model="gpt-5.5"` and `reasoning_effort="xhigh"`. The agent reviews and may fix only files in the approved File Ownership table.

Before dispatching review+fix, the coordinator creates `refs/simplepower/scratch/<run-id>/review-fix/before` for all approved implementation files. If review+fix edits files, the coordinator creates `refs/simplepower/scratch/<run-id>/review-fix/after` and inspects or hands off:

```bash
git diff refs/simplepower/scratch/<run-id>/review-fix/before refs/simplepower/scratch/<run-id>/review-fix/after -- codex-rs/cli/src/main.rs codex-rs/tui/src/version.rs codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__status_surface_previews_codex_version.snap codex-rs/config/src/loader/mod.rs codex-rs/core/src/config/config_loader_tests.rs .codex/skills/mod-refresh-build/SKILL.md .codex/skills/mod-refresh-publish/SKILL.md .codex/skills/mod-refresh-merge-preserve/SKILL.md .codex/skills/mod-refresh-release/SKILL.md .codex/skills/mod-release-current/SKILL.md docs/compact-fix/ChangeLog.md
```

The REVIEW-tier review+fix agent must perform the review and fixes directly in the current worker. Do not run Codex CLI. Do not spawn subagents. Do not invoke Simple Power skills. Do not restart execution. Do not reroute the workflow. It must not commit.

Review focus:
- `CODEX_CLI_RELEASE_VERSION` is the only release build-time version override.
- CLI `--version` and TUI display share the same release-env-aware version behavior.
- No stale `0.141.0+gary` or `+gary` release gate remains in active code or skills.
- Trust-all applies before project layers are disabled, and explicit untrusted still wins.
- Tests/snapshots were updated as source artifacts but not run.
- Skill docs do not derive release versions from final `HEAD`, `git rev-parse --short`, or upstream SemVer patch.

## Commit Checkpoints

Exactly three future coordinator checkpoint commits are allowed:

1. **Accepted plan checkpoint:** after the REVIEW-tier plan reviewer approves and the user gives combined approval for the reviewed plan, model/task allocation, and immediate current-session execution; before invoking `simplepower:subagent-driven-development`.
2. **Quick-verified implementation checkpoint:** after all `sp-impl` file edits complete and quick verification passes.
3. **Final checkpoint:** after the REVIEW-tier review+fix agent completes and final verification passes.

Workers, plan reviewers, quick verifiers, review+fix agents, and individual tasks must not commit. Do not include worker-owned commits or per-task commits.

Scratch refs are coordinator-owned local review anchors only. They live under `refs/simplepower/scratch/<run-id>/`, are not branches, are not pushed, are not merged or rebased, and do not count as checkpoint commits. Delete each phase's scratch refs after that phase's accepted checkpoint succeeds. If a checkpoint fails, a blocker stops execution, or the user stops the workflow, preserve scratch refs and report:

```bash
git for-each-ref --format='%(refname)' "refs/simplepower/scratch/<run-id>" | while read -r ref; do git update-ref -d "$ref"; done
```

That command is only the manual cleanup command to report for later use; do not run it while the workflow is blocked or before the user directs cleanup.

## Current-Session Auto-Dispatch

After the plan reviewer approves, ask the user for one combined approval that covers:
- the reviewed plan;
- the model/task allocation;
- immediate current-session execution.

If the user requests changes, update this plan, rerun focused self-review for changed categories, create the next `plan-review/after-<n>` scratch ref, and send the revised plan back to the same reviewer with the concrete scratch-ref diff command.

After combined approval, the coordinator creates the accepted plan checkpoint commit, deletes successful `plan-review` scratch refs, then immediately invokes `simplepower:subagent-driven-development` in the current session with:

```text
Execute `docs/simplepower/plans/2026-06-18-mod-version-and-trust-all-fixes.md` with aggregate parallel implementation from the approved Interface Contract. Use the approved FAST/NORMAL/BEST/REVIEW model allocation. Dispatch all non-conflicting `sp-impl` file-edit workers whose coordination needs are satisfied by their Contract inputs, run the quick FAST-tier verifier with format/static checks/review and timeouts after all workers finish, commit the quick-verified implementation, then run one REVIEW-tier review+fix agent, final verification, and final commit.
```

For this plan, verification is constrained by Interface Contract entry 5: run formatting and static checks only; do not run Rust tests, Cargo builds, release builds, or binary generation without fresh explicit user approval.

## Verification

Final verification runs after the REVIEW-tier review+fix agent completes and before the final checkpoint. Use the same no-Rust-test/no-build constraint approved by the user.

Final commands:

```bash
timeout 120s bash -lc 'cd codex-rs && just fmt'
timeout 30s bash -lc '! rg -n "0\\.141\\.0\\+gary|expected_display_version|\\+gary" codex-rs/cli/src/main.rs codex-rs/tui/src/version.rs .codex/skills/mod-refresh-build/SKILL.md .codex/skills/mod-refresh-publish/SKILL.md .codex/skills/mod-refresh-merge-preserve/SKILL.md .codex/skills/mod-refresh-release/SKILL.md .codex/skills/mod-release-current/SKILL.md docs/compact-fix/ChangeLog.md'
timeout 30s rg -n "CODEX_CLI_RELEASE_VERSION|version = CODEX_CLI_VERSION|pub const CODEX_CLI_DISPLAY_VERSION: &str = CODEX_CLI_VERSION|dangerously_trust_all_projects|trust_level: Some\\(TrustLevel::Trusted\\)|\\./codex --version" codex-rs/cli/src/main.rs codex-rs/tui/src/version.rs codex-rs/config/src/loader/mod.rs codex-rs/core/src/config/config_loader_tests.rs .codex/skills/mod-refresh-build/SKILL.md .codex/skills/mod-refresh-publish/SKILL.md .codex/skills/mod-refresh-release/SKILL.md .codex/skills/mod-release-current/SKILL.md
timeout 30s rg -n "CODEX_CLI_RELEASE_VERSION|metadata-based|\\.mod" docs/compact-fix/ChangeLog.md
timeout 30s bash -lc 'upstream_sha="$(tr -d "[:space:]" < upstreamhash.txt)"; mod_version="$(tr -d "[:space:]" < modversion.txt)"; test "0.141.$(printf "%s" "$upstream_sha" | cut -c1-5).${mod_version}.mod" = "0.141.c7329.1.mod"'
```

Expected result: formatting succeeds; static checks find the new release/trust contracts and no stale `+gary` gates; release metadata computes the expected current example version. Failure means the implementation is incomplete or formatting changed files that need inspection.

The coordinator performs the final checkpoint only after the REVIEW-tier review+fix agent has completed and these final commands pass.

After the final checkpoint succeeds and the coordinator deletes successful `review-fix` scratch refs, final reporting must run:

```bash
timeout 30s git for-each-ref --format='%(refname)' "refs/simplepower/scratch/<run-id>"
```

Expected result: no refs remain for the run. If refs remain after successful cleanup, report them and the manual cleanup command. If the workflow stops because of user direction, a blocker, or a failed checkpoint, preserve remaining scratch refs and report the manual cleanup command without running it.
