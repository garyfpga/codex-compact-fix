# Minimal Upstream Fork Release Implementation Plan

**Goal:** Rebuild the fork from current `upstream/main`, retain only the approved trust-all behavior, release/version infrastructure, and VS Code customization, publish a commit-derived optimized Linux binary, and deploy it safely to the approved destinations.

## Design Summary

Create local branch `main-fork-2026-08-24` directly from execution-time `upstream/main`; leave the existing `main` ref unchanged. Carry forward the already staged `.vscode/settings.json` customization as an editor-only exception. Manually forward-port `dangerously_trust_all_projects` instead of cherry-picking mixed historical commits. When true, it must trust otherwise unknown projects early enough to load project-local `.codex` layers and, only when approval and permission settings are otherwise unspecified, select danger-full-access with approvals disabled. Explicit project trust, approval policy, sandbox mode, default/named permissions, managed configuration, and requirements constraints stay authoritative. Omitted or false preserves upstream behavior.

Keep only one streamlined `.codex/skills/mod-refresh-full-release` skill. Future releases fetch and merge upstream, preserve this minimal contract, run focused checks, build `codex-cli` with upstream's optimized Cargo `release` profile, and publish from a clean source commit. For official release `rust-v0.149.1` and source commit prefix `abcde`, the binary version, tag, and release title are exactly `149.1.abcde`. The base is always the execution-time latest stable official release with its leading `0.` removed; the suffix is exactly the first five lowercase hexadecimal characters of the source commit. Do not carry `upstreamhash.txt`, `modversion.txt`, a `.mod` counter, historical compact preservation rules, or checked-in release-run logs.

After publication, copy the verified artifact locally to `/tmp/codex-${fork_version}` without changing local `~/.local/bin/codex`. Preflight all five SSH aliases before remote mutation. Then deploy atomically in the approved order `backup`, `fpga01`, `office`, `axel`, `desk`: upload beside the target, verify checksum, kill current-login-user processes with `pkill -x codex`, retain the existing target as a timestamped backup, atomically rename the new executable, and verify it. Treat both `backup` and `office` as required operations even though they currently resolve to `gary@focus`. A failed global preflight changes no remote. A per-host install failure restores that host and stops later hosts.

This is one cohesive package because code behavior, version provenance, release mutation, and deployment all depend on the same committed source and verified artifact. Specialized delegation would add coordination risk without a separable write package. The user selected the main-agent route and minimal focused tests; no full workspace suite and no Bazel commands are authorized.

Observable success is: the new branch is based on current upstream and pushed to `origin`; only the approved fork delta remains; focused trust tests pass; the built/tagged/published artifact reports the commit-derived version; `/tmp` holds the same artifact; and all five SSH aliases report the same checksum and version while retaining prior targets as backups.

## Implementation Route

**Main agent.** The code, tests, release skill, source commit, release, and deployments form one ordered provenance chain. The main agent owns the entire current-session run; there are no grouped workers, implementation subagents, mandatory reviewer subagents, or quick-verifier subagents.

## Exact Files

Tracked files created, modified, or generated:

- `docs/simplepower/plans/2026-08-24-minimal-upstream-fork-release.md` — authoritative plan and coordinator-owned execution record.
- `.vscode/settings.json` — retain the user's staged editor-color customization.
- `.codex/skills/mod-refresh-full-release/SKILL.md` — create the single minimal-fork fetch/merge/build/publish workflow.
- `.codex/skills/mod-refresh-full-release/agents/openai.yaml` — create matching skill discovery metadata.
- `codex-rs/config/src/config_toml.rs` — add the top-level optional dangerous setting.
- `codex-rs/config/src/loader/mod.rs` — apply the setting during early project-layer trust decisions.
- `codex-rs/core/src/config/mod.rs` — apply trusted-project and implicit danger-full-access defaults with explicit/managed precedence.
- `codex-rs/core/src/config/config_loader_tests.rs` — cover early project-layer loading and explicit-untrusted precedence.
- `codex-rs/core/src/config/config_tests.rs` — cover permission/approval defaults and explicit/managed precedence.
- `codex-rs/core/config.schema.json` — regenerate from `ConfigToml`.
- `codex-rs/cli/src/main.rs` — use `CODEX_CLI_RELEASE_VERSION` when embedded, otherwise `CARGO_PKG_VERSION`.
- `codex-rs/tui/src/version.rs` — use the same release-version fallback contract for TUI version surfaces.

Generated or deployed artifacts, where `fork_version` is defined from the clean source commit during execution and `fork_deploy_stamp` is one UTC `YYYYMMDDTHHMMSSZ` value computed before deployment:

- `codex-rs/target/release/codex` — Cargo build output.
- `codex` — ignored repository-root release asset copied from the Cargo output.
- `/tmp/codex-${fork_version}` — verified local copy; do not modify local `~/.local/bin/codex` directly.
- On each approved SSH alias: `~/.local/bin/.codex-${fork_version}.incoming`, `~/.local/bin/codex`, and, when a previous target exists, `~/.local/bin/codex.backup.${fork_deploy_stamp}`.

No other tracked files may be changed. No snapshot change is expected because ordinary builds retain the upstream `CARGO_PKG_VERSION`; if a snapshot changes, stop and request a design decision instead of accepting it.

## Implementation Steps

1. **Create the accepted-plan baseline without changing `main`.**
   - Confirm the current branch is `main`, record its SHA, and inventory the staged `.vscode/settings.json` plus this plan.
   - Preserve both paths in a clearly named stash with untracked-file inclusion. Do not drop that stash during the run.
   - Fetch exactly `upstream main:refs/remotes/upstream/main`; record `git rev-parse upstream/main`.
   - Stop if local branch `main-fork-2026-08-24` already exists or the fetch cannot establish a current upstream SHA.
   - Create and switch to `main-fork-2026-08-24` at `upstream/main`, apply the safety stash with its index state, and unstage `.vscode/settings.json` while preserving its working-tree content.
   - Confirm `git rev-parse main` still equals the recorded original SHA.
   - After combined plan approval, commit only this accepted plan as the accepted-plan checkpoint. The staged VS Code change remains uncommitted for the implementation source commit.

2. **Forward-port the trust-all behavior manually.**
   - Add `pub dangerously_trust_all_projects: Option<bool>` to `ConfigToml` beside trust/permission settings, with a warning doc comment describing unknown-project trust and implicit full access.
   - Parse the option in the early project trust configuration in `config/src/loader/mod.rs`. Add the resolved Boolean to `ProjectTrustContext`. After all explicit cwd/project-root/repo-root trust lookups fail, return `Trusted` only when the Boolean is true; explicit `Untrusted` and `Trusted` matches return before this fallback.
   - In `core/src/config/mod.rs`, resolve the flag with `false` as the default. Use it for the unmatched active-project fallback.
   - Define implicit danger-full-access selection only when trust-all is true and no explicit approval override, `approval_policy`, legacy sandbox/permission syntax, selected default permission profile, or requirements-forced profile applies. Select the existing built-in danger-full-access profile and `AskForApproval::Never` only in that case.
   - Keep upstream managed requirements and constraints in the existing resolution pipeline so they can reject or replace disallowed defaults.

3. **Add focused behavioral coverage and regenerate the schema.**
   - In `config_loader_tests.rs`, add `unknown_project_layer_enabled_when_trust_all` and `explicit_untrusted_project_layer_disabled_when_trust_all`. Assert complete project-layer/effective-config results where practical.
   - In `config_tests.rs`, add `dangerously_trust_all_projects_permission_defaults_and_precedence`. Use a case table covering omitted, false, true with otherwise-unset permissions, explicit approval, explicit sandbox/default permissions, and managed requirements; compare complete resolved permission/approval result objects rather than individual fields where supported.
   - Run `just write-config-schema` from `codex-rs` and confirm the schema contains the new Boolean-or-null top-level field.
   - Do not add tests for the statically defined version constant and do not run the full suite.

4. **Add minimal release-version plumbing.**
   - In CLI and TUI version sources, define compile-time constants with `option_env!("CODEX_CLI_RELEASE_VERSION")` and fall back to `env!("CARGO_PKG_VERSION")`.
   - Bind Clap's CLI version and the TUI display version to those constants. Ordinary upstream-style builds therefore remain unchanged; release builds report the approved derived version.

5. **Create the streamlined full-release skill.**
   - Create only `mod-refresh-full-release/SKILL.md` and its `agents/openai.yaml`; do not copy the old chain of compact-specific helper skills.
   - Require branch `main-fork-2026-08-24`, clean tracked state, `origin` = `garyfpga/codex-compact-fix`, `upstream` = `openai/codex`, authenticated `gh`, and absence of ambiguous local/remote release state.
   - Fetch `upstream/main`. Merge it when it is not already an ancestor of `HEAD`; treat an already-contained upstream commit as a no-op. On conflicts or a changed minimal contract, stop for explicit user direction.
   - Require the focused tests, schema check, and formatting check after a real merge. Require a committed clean source before version derivation.
   - Query the latest non-draft, non-prerelease `openai/codex` release. Require tag/name shape `rust-v0.X.Y` or `0.X.Y`, remove only `rust-v`/`v` and the leading `0.`, and obtain `X.Y`.
   - Set `fork_source_sha` to the full clean `HEAD`, `fork_source_short` to its first five characters, and `fork_version` to `X.Y.${fork_source_short}`. Require exact shape `^[0-9]+\.[0-9]+\.[0-9a-f]{5}$`.
   - Build only `codex-cli` with `CODEX_CLI_RELEASE_VERSION="${fork_version}" cargo build -p codex-cli --release`; do not use Bazel or a custom Cargo profile.
   - Copy `target/release/codex` to repository-root `codex`, preserve executable mode, and require exact output `codex-cli ${fork_version}`.
   - Verify the origin branch, exact tag, and GitHub release do not already exist in conflicting form. Push the branch, create/push annotated tag `${fork_version}` at `${fork_source_sha}`, and publish repository-root `codex` to `garyfpga/codex-compact-fix` with the exact tag/title and concise notes naming the official base and source SHA.
   - If a remote operation partially succeeds, inspect and report the exact branch/tag/release state; do not retry or delete remote state without fresh approval.

6. **Format, run focused verification, and create the required source commit.**
   - Run `just fmt` from `codex-rs` after all tracked code and skill edits.
   - Run the mandatory Quick Verification commands below through the main agent and repair only approved in-scope failures.
   - Review the complete diff against `upstream/main`, confirming no old fork behavior or unrelated path entered the branch and the safety stash still exists.
   - Create one objective technical-prerequisite source commit containing the VS Code setting, trust feature, tests/schema, version plumbing, simplified skill, and accepted plan state. A clean committed source SHA is required before the approved version/build/tag operation. Record its SHA as `fork_source_sha`; this is the commit whose first five characters enter the binary version and whose tag identifies the binary source.

7. **Run the simplified full release.**
   - Re-read and invoke the newly created `$mod-refresh-full-release` skill from the clean source commit.
   - Require the skill's upstream refresh/no-op result, build command, embedded version, branch push, tag push, GitHub release URL, and uploaded `codex` asset before deployment.
   - Do not update tracked release logs after the source commit; execution evidence belongs in this plan's later `Execution Summary`.

8. **Create and verify the local `/tmp` copy.**
   - Copy repository-root `codex` atomically to `/tmp/codex-${fork_version}` as mode `0755`; do not follow or replace local `~/.local/bin/codex`.
   - Record `sha256sum`, `file`, and exact version output for both repository-root and `/tmp` paths and require equality.

9. **Preflight every remote before mutation.**
   - For each alias in `backup fpga01 office axel desk`, use non-interactive SSH to record resolved hostname/user and require Linux x86_64, a writable existing `~/.local/bin`, `pkill`, `pgrep`, `sha256sum`, and the interpreter/runtime path required by the built artifact.
   - Record the existing target kind, link destination when applicable, checksum when it is a regular executable, and current version when runnable.
   - Abort the entire remote phase without uploads, kills, backups, or installs if any alias fails. Do not deduplicate `backup` and `office`.

10. **Deploy sequentially with per-host rollback.**
    - Compute one `fork_deploy_stamp` in UTC. For each alias in the approved order, upload to `~/.local/bin/.codex-${fork_version}.incoming`, set mode `0755`, and verify its checksum before killing processes.
    - Run `pkill -x codex` as the SSH login user; accept the no-process status and poll with `pgrep -x codex` until none remain. Do not use `sudo` and do not target other users.
    - Move an existing `~/.local/bin/codex` path itself, including a symlink, to `~/.local/bin/codex.backup.${fork_deploy_stamp}`. Atomically rename the incoming file to `~/.local/bin/codex`.
    - Require mode `0755`, the local release checksum, and exact `codex-cli ${fork_version}` output. Do not restart Codex.
    - If install verification fails, move the failed new target aside, restore that alias's backup, verify restoration where runnable, report the exact state, and stop before the next alias.

11. **Review, verify, update the execution record, and close the run.**
    - The main agent reviews the complete accepted-plan-to-working-state diff plus committed source diff, release provenance, and deployment evidence. Apply only approved in-scope fixes; any source-changing fix invalidates the artifact and requires rebuilding/releasing under a newly derived source version after fresh user approval because the existing published tag cannot be repurposed.
    - Run the first Final Verification pass below.
    - Refresh `## Execution Summary` in this file with status/outcome, key changes, verification overview, notable findings/fixes/deviations, branch, the tagged source SHA, pre-summary HEAD/worktree state, release URL/version/checksum, deployment results/backups, safety stash reference, and unresolved follow-ups.
    - Rerun the terminal Final Verification commands without further file edits.
    - Create the final reviewed/verified completion checkpoint commit containing the execution-summary update. The final branch commit is intentionally later than the tagged source commit; the tag continues to identify the exact binary source. Push the updated branch, verify the remote branch SHA, and report the final containing SHA. Do not create an empty commit.

## Risks

- **Dirty starting state:** The staged VS Code customization and untracked plan could be lost or absorbed into `main`. Preserve both in a named stash, apply without dropping, and prove the `main` ref is unchanged.
- **Historical feature leakage:** The old trust commits mix compact/TUI behavior, and the old release chain preserves many obsolete mods. Port behavior manually and audit the full diff against fresh upstream.
- **Security precedence regression:** Trust-all is intentionally dangerous but must not override explicit or managed restrictions. Cover early loading and permission precedence with focused integration tests.
- **Self-referential versioning:** A commit cannot contain a version derived from its own future SHA. Embed the already-known clean source SHA through the build environment, tag that source commit, and place later evidence only in the final execution-summary commit.
- **Official-version ambiguity or tag collision:** Strictly validate the stable release string and derived version; stop before mutation if any local tag, remote tag, or release already conflicts.
- **Partial GitHub publication:** Branch, tag, and release operations can partially succeed. Inspect and report exact remote state; no blind retries, deletes, or tag reuse.
- **Binary compatibility:** The Cargo release artifact is a Linux x86_64 executable with a runtime interpreter requirement. Preflight every alias before remote mutation.
- **Duplicate endpoint aliases:** `backup` and `office` currently resolve to `focus`, but the user explicitly requires both. Execute and report both; do not silently deduplicate.
- **Killing active sessions:** Limit `pkill -x codex` to the SSH login user, verify exit, and never use `sudo`.
- **Fleet partial deployment:** Complete global preflight first, deploy sequentially, restore the current host on failure, and stop later hosts.

## Quick Verification

Resolved `skip_quick_verifier = true`; executor: **Main agent**. Rust commands use `/usr/bin/time -p` and are allowed to finish without a terminating timeout, per repository instructions.

Run after all implementation edits and before the source commit:

```bash
timeout 30s git diff --check
cd codex-rs && /usr/bin/time -p just fmt-check
cd codex-rs && /usr/bin/time -p just test -p codex-core unknown_project_layer_enabled_when_trust_all
cd codex-rs && /usr/bin/time -p just test -p codex-core explicit_untrusted_project_layer_disabled_when_trust_all
cd codex-rs && /usr/bin/time -p just test -p codex-core dangerously_trust_all_projects_permission_defaults_and_precedence
timeout 30s rg -n 'dangerously_trust_all_projects|CODEX_CLI_RELEASE_VERSION' codex-rs/config/src/config_toml.rs codex-rs/config/src/loader/mod.rs codex-rs/core/src/config/mod.rs codex-rs/core/src/config/config_loader_tests.rs codex-rs/core/src/config/config_tests.rs codex-rs/core/config.schema.json codex-rs/cli/src/main.rs codex-rs/tui/src/version.rs
```

Expected: clean diff syntax; formatting passes; all three named tests pass; the config field, early trust path, runtime permission path, generated schema, and both version surfaces are present. The main agent diagnoses failures, makes only approved in-scope repairs, and reruns affected commands. No quick-verifier subagent or scratch refs are used.

## Final Verification

First pass after publication and deployment, then the same safe read-only terminal pass after the execution-summary edit:

```bash
timeout 30s git diff --check
cd codex-rs && /usr/bin/time -p just fmt-check
cd codex-rs && /usr/bin/time -p just test -p codex-core unknown_project_layer_enabled_when_trust_all
cd codex-rs && /usr/bin/time -p just test -p codex-core explicit_untrusted_project_layer_disabled_when_trust_all
cd codex-rs && /usr/bin/time -p just test -p codex-core dangerously_trust_all_projects_permission_defaults_and_precedence
timeout 30s bash -lc 'test "$(git branch --show-current)" = main-fork-2026-08-24 && test "$(git merge-base upstream/main HEAD)" = "$(git rev-parse upstream/main)"'
timeout 30s bash -lc 'fork_tag=$(git tag --points-at "${FORK_SOURCE_SHA}" | rg "^[0-9]+\.[0-9]+\.[0-9a-f]{5}$") && test -n "$fork_tag" && test "$(./codex --version)" = "codex-cli $fork_tag" && test "$(/tmp/codex-${fork_tag} --version)" = "codex-cli $fork_tag" && cmp -s codex "/tmp/codex-${fork_tag}"'
timeout 30s gh release view "${FORK_VERSION}" --repo garyfpga/codex-compact-fix --json tagName,assets,url
```

`FORK_SOURCE_SHA` and `FORK_VERSION` are execution-record values exported from the completed release step before these commands. For each alias, also run this read-only verification with the same exported values:

```bash
timeout 30s ssh "$FORK_HOST" "test \"\$(~/.local/bin/codex --version)\" = 'codex-cli ${FORK_VERSION}' && printf '%s  %s\n' '${FORK_SHA256}' ~/.local/bin/codex | sha256sum -c -"
```

Expected: focused tests and formatting pass; branch/upstream relationship is correct; the exact source commit is tagged; repository-root, `/tmp`, GitHub asset metadata, and all aliases agree on version/checksum. The main agent must inspect the full diff from the accepted-plan checkpoint, the fresh-upstream delta, GitHub state, and remote results before declaring success. After the final completion commit and branch push, perform one last read-only check that `git ls-remote origin refs/heads/main-fork-2026-08-24` equals the local final SHA. Do not rerun publishing or deployment commands as verification.

## Execution Record

This file, `docs/simplepower/plans/2026-08-24-minimal-upstream-fork-release.md`, is the coordinator-owned execution record. After the first Final Verification pass, update `## Execution Summary` with a concise current snapshot: status/outcome, key changes, verification overview, notable findings/fixes/deviations, observed branch, tagged source SHA, pre-summary HEAD and worktree state, release version/URL/checksum, per-alias deployment and backup results, safety stash reference, and unresolved follow-ups. Exclude raw logs and unrelated audits. If a later material finding occurs during this active run, refresh the snapshot, append a phase- or date-labeled follow-up entry, and rerun affected checks plus terminal verification. The final handoff reports the SHA containing the summary because a file cannot record its own containing commit without self-reference.

## Checkpoint Conditions

There are exactly two mandatory coordinator checkpoint types:

1. **Accepted plan checkpoint:** After combined user approval of this plan, its `Main agent` route, immediate current-session execution, the accepted-plan commit, the final reviewed/verified completion commit, and bounded in-scope coordinator execution commits, create `main-fork-2026-08-24` from current `upstream/main`, carry this plan onto it, and commit the accepted plan before implementation edits. Combined approval authorizes this checkpoint without another prompt.
2. **Final reviewed/verified completion checkpoint:** After implementation, main-agent Quick Verification, main-agent final diff review and in-scope fixes, the first Final Verification pass, execution-summary update, and unchanged terminal verification pass, commit remaining in-scope summary changes as the newest final commit and push the branch. Do not create an empty commit.

Conditional execution commits do not add checkpoint types. This plan already expects one objective technical-prerequisite source commit because the approved build/version/tag step requires a clean committed source SHA. A separate summary commit is expected because it records publication and deployment facts after the tagged source build. Any newly discovered execution commit is allowed only when an objective approved command requires committed state or a later finding requires refreshing the execution summary; convenience and history-shaping commits do not qualify. Combined approval covers only coordinator-owned, in-scope commits during this active run and expires at final handoff. Fresh approval is required for any scope, strategy, route, or verification change.

## Execution Summary

Status: Accepted on 2026-08-24. The user approved the `Main agent` route, immediate current-session execution, both mandatory checkpoint types, and the bounded in-scope execution commits described above.
