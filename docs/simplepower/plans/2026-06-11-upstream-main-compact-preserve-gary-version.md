# Upstream Main Merge And Compact Preservation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `simplepower:subagent-driven-development` for aggregate parallel implementation. Dispatch all non-conflicting `sp-impl` file-edit workers whose coordination needs are satisfied by the approved Interface Contract, run the quick verifier after all workers finish, commit the quick-verified implementation, then run one REVIEW-tier review+fix agent before final verification and final commit.

**Goal:** Merge `upstream/main` into this branch, preserve the branch-local compact behavior and config knobs, make the status line show a display-only `0.139.0+gary` version label, then rebuild and deploy the resulting Rust binary to the known remote hosts.

**Design Summary:** The approved path is a safe upstream merge, not a rebase. Create a backup branch first, merge `upstream/main`, and resolve only the concrete overlap areas that matter for compact: remote-first fast-tier routing, local fallback on remote failure, `remote_compact` config parsing/validation/schema, and V2 default routing. Preserve the branch-local remote-compaction policy while accepting upstream’s new default V2 behavior. Separately, add a display-only TUI version label so the status line shows `0.139.0+gary` without changing package metadata or external version checks. After the code is merged and formatted, rebuild `codex-cli` with Rust only, copy the binary locally, and deploy it to the remote hosts from the last simple-power plan.

**Architecture:** Treat the upstream merge itself as a serialized coordinator-owned repository operation, then split post-merge reconciliation into two bounded concerns. The compact path stays centered on the existing `core` orchestration layer and config model, while the version-label change stays inside the TUI display layer. That split keeps compact policy reconciliation isolated from the UI-only version display after the merge result exists, so workers can operate on disjoint files without stepping on each other.

**Tech Stack:** Git, Rust, cargo, the `codex-core`/`codex-tui` crates, insta snapshot tests, and SSH/SCP deployment to the remote hosts.

**Model Allocation:** FAST/NORMAL/BEST/REVIEW tiers are assigned below. Resolve each tier by explicit user override, quoted assignment in project root AGENTS.md, process environment variable, then built-in default. The project root AGENTS.md lookup reads only `<repo>/AGENTS.md`, not nested AGENTS.md files or repo-wide grep. FAST defaults to `SIMPLEPOWER_FAST_MODEL` (`gpt-5.3-codex-spark-high` when unset), NORMAL defaults to `SIMPLEPOWER_NORMAL_MODEL` (`gpt-5.4-mini-high` when unset), BEST defaults to `SIMPLEPOWER_BEST_MODEL` (`gpt-5.5-high` when unset), and REVIEW defaults to `SIMPLEPOWER_REVIEW_MODEL` (`gpt-5.5-xhigh` when unset). The plan reviewer is a REVIEW-tier plan reviewer, and the final review+fix agent is a REVIEW-tier review+fix agent. The quick verifier uses the FAST tier by default, resolving to `model="gpt-5.3-codex-spark"` and `reasoning_effort="high"` unless `SIMPLEPOWER_FAST_MODEL` is overridden.

**Commit Policy:** The coordinator commits after the reviewed plan, allocation, and immediate current-session execution receive combined approval, after all file edits and quick verification complete before final review, and after final review/fix plus final verification. Workers, plan reviewers, quick verifiers, and review+fix agents must not commit. No per-task commits. Coordinator-owned temporary scratch refs under `refs/simplepower/scratch/<run-id>/...` may be created only as local review diff anchors; they are not accepted history commits, not pushed, not merged, not rebased, and must be cleaned up after successful checkpoints or reported for manual cleanup on blockers or failed checkpoints.

**Reviewer Non-Recursion Rule:** Plan reviewers, quick verifiers, and review+fix agents must perform their assigned review, verification, or fix directly in their current worker. They must not invoke Simple Power skills, spawn subagents, run Codex CLI, restart execution, or reroute the workflow.

---

## Interface Contract

1. **Upstream merge contract**
   - Current branch: `feature/v1-remote-compact-config`.
   - Upstream target: `upstream/main`.
   - Safety branch to create first: `backup/before-upstream-main-2026-06-11`.
   - The merge must preserve the branch-local compact policy while adopting upstream’s new compact-related commits, including the default V2 remote compact behavior already present in upstream.

2. **Compact behavior contract**
   - The following branch-local behaviors must remain true after the merge:
     - fast-tier service selection for remote-first compact when supported
     - local fallback after remote compact failure
     - `remote_compact.max_attempts`
     - `remote_compact.attempt_timeout_sec`
   - The compact code paths that already embody these behaviors are:
     - `codex-rs/core/src/compact_service_tier.rs`
     - `codex-rs/core/src/remote_compact_fallback.rs`
     - `codex-rs/core/src/compact_remote.rs`
     - `codex-rs/core/src/compact_remote_v2.rs`
     - `codex-rs/core/src/session/turn.rs`
     - `codex-rs/core/src/tasks/compact.rs`
     - `codex-rs/core/src/config/mod.rs`
     - `codex-rs/config/src/config_toml.rs`
     - `codex-rs/config/src/types.rs`
     - `codex-rs/core/config.schema.json`
   - `RemoteCompactVersion::V2` stays the default path when the feature flag is enabled; the merge must not regress that behavior.

3. **Display-only version contract**
   - The status line version label must be display-only and must not change `CARGO_PKG_VERSION`, package metadata, or external update/version checks.
   - The TUI should expose a display constant such as `CODEX_CLI_DISPLAY_VERSION` whose value is `0.139.0+gary`.
   - `StatusLineItem::CodexVersion` must render the display-only value.
   - `StatusSurfacePreviewItem::CodexVersion` must preview the same display-only value.
   - The display version needs snapshot coverage in `codex-rs/tui/src/chatwidget/tests/status_surface_previews.rs`.

4. **Deployment contract**
   - The release binary path is `codex-rs/target/release/codex`.
   - The local copy target is `/home/gary/codex`.
   - The remote host list from the last simple-power plan is:
     - `fpga01`
     - `axel`
     - `office`
     - `backup`
     - `desk`
   - Deployment must kill any running `codex` process on each host.
   - Deployment must remove known `install.sh` managed standalone binary paths only, preserving `~/.codex/config.toml`, auth, and other user state.
   - Deployment must copy the new binary to `~/.local/bin/codex`.

## File Ownership

| File | Owner task | Change type | Responsibility | Parallel safety notes |
| --- | --- | --- | --- | --- |
| `docs/simplepower/plans/2026-06-11-upstream-main-compact-preserve-gary-version.md` | Coordinator | create | Authoritative plan and execution contract for the merge, display-label update, build, and deployment. | Coordinator-owned only. |
| `git merge upstream/main` repo-wide result | Coordinator | generated | Create `backup/before-upstream-main-2026-06-11`, fetch `upstream/main`, and run the repository-wide merge before any worker starts. | Serialized coordinator-owned operation; no `sp-impl` worker runs during the merge. |
| `codex-rs/core/src/config/mod.rs` | Task 1 | modify | Resolve upstream config merge conflicts while preserving `remote_compact` parsing and validation. | Exclusive to Task 1. |
| `codex-rs/core/src/lib.rs` | Task 1 | modify | Resolve module/export merge overlap if needed by upstream `core` changes. | Exclusive to Task 1. |
| `codex-rs/core/src/compact_service_tier.rs` | Task 1 | modify | Preserve branch-local fast-tier compact routing while accepting upstream compact changes. | Exclusive to Task 1. |
| `codex-rs/core/src/remote_compact_fallback.rs` | Task 1 | modify | Preserve local-fallback behavior and version-aware remote-first routing. | Exclusive to Task 1. |
| `codex-rs/config/src/config_toml.rs` | Task 1 | modify | Preserve `remote_compact` TOML fields and upstream config parsing changes. | Exclusive to Task 1. |
| `codex-rs/config/src/types.rs` | Task 1 | modify | Preserve `RemoteCompactConfigToml` and related validation ranges. | Exclusive to Task 1. |
| `codex-rs/core/config.schema.json` | Task 1 | modify | Regenerate or reconcile schema changes for preserved config fields. | Exclusive to Task 1. |
| `codex-rs/core/src/compact.rs` | Task 1 | modify | Resolve upstream compact orchestration overlap without regressing branch-local behavior. | Exclusive to Task 1. |
| `codex-rs/core/src/compact_remote.rs` | Task 1 | modify | Preserve V1 remote compact behavior and branch-local fallback semantics. | Exclusive to Task 1. |
| `codex-rs/core/src/compact_remote_v2.rs` | Task 1 | modify | Preserve V2 remote compact behavior while accepting upstream default changes. | Exclusive to Task 1. |
| `codex-rs/core/src/session/turn.rs` | Task 1 | modify | Preserve auto-compact routing and V2 default selection. | Exclusive to Task 1. |
| `codex-rs/core/src/tasks/compact.rs` | Task 1 | modify | Preserve manual `/compact` routing through the shared remote-first wrapper. | Exclusive to Task 1. |
| `codex-rs/tui/src/version.rs` | Task 2 | modify | Add the display-only version label constant. | Exclusive to Task 2. |
| `codex-rs/tui/src/chatwidget/status_surfaces.rs` | Task 2 | modify | Route the status-line version item to the display-only label. | Exclusive to Task 2. |
| `codex-rs/tui/src/bottom_pane/status_surface_preview.rs` | Task 2 | modify | Update the status-line preview placeholder for the version item. | Exclusive to Task 2. |
| `codex-rs/tui/src/chatwidget/tests/status_surface_previews.rs` | Task 2 | modify | Add snapshot coverage for the status-line version label. | Exclusive to Task 2. |
| `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__status_surface_previews_codex_version.snap` | Task 2 | generated | Snapshot for the version-label preview test. | Generated by Task 2. |

## Coordinator Pre-Implementation Merge Step

The coordinator performs this step after the accepted plan checkpoint and before dispatching any `sp-impl` workers. No worker may run while this step is active because `git merge upstream/main` can modify any tracked file, the index, and conflict state.

Commands:

```bash
cd /home/gary/git/codex-compact-fix
git branch backup/before-upstream-main-2026-06-11
git fetch upstream main
git merge upstream/main
```

Expected result: `upstream/main` is merged into `feature/v1-remote-compact-config`, or the coordinator resolves merge conflicts before worker dispatch. If merge conflicts are outside the known compact or TUI version areas, the coordinator owns the conflict triage and must either resolve them directly when they are straightforward upstream acceptance conflicts or stop and report the exact file list if the accepted plan needs a wider implementation scope.

Config-schema rule: if the merge or conflict resolution changes `ConfigToml`, `RemoteCompactConfigToml`, or nested config types, the implementation must run `just write-config-schema` before final build verification. Because this plan explicitly preserves `remote_compact` config fields and `codex-rs/core/config.schema.json`, the default path is to run `just write-config-schema` after post-merge reconciliation.

## Implementation Tasks

### Task 1: Reconcile Compact Behavior After The Coordinator Merge

**Goal:** Inspect the coordinator-owned upstream merge result and keep the branch-local compact policy intact.

**Contract inputs:** Interface Contract entries 1 and 2.

**Serialization required:** Yes. This task must start only after the coordinator-owned `git merge upstream/main` step completes because the merge is a repository-wide index/worktree mutation. After the merge result exists, this task can run in parallel with Task 2 because the write scopes do not overlap.

**Write scope:**
- `codex-rs/core/src/config/mod.rs`
- `codex-rs/core/src/lib.rs`
- `codex-rs/core/src/compact_service_tier.rs`
- `codex-rs/core/src/remote_compact_fallback.rs`
- `codex-rs/config/src/config_toml.rs`
- `codex-rs/config/src/types.rs`
- `codex-rs/core/config.schema.json`
- `codex-rs/core/src/compact.rs`
- `codex-rs/core/src/compact_remote.rs`
- `codex-rs/core/src/compact_remote_v2.rs`
- `codex-rs/core/src/session/turn.rs`
- `codex-rs/core/src/tasks/compact.rs`

**Parallel:** Yes, compatible with Task 2 only after the coordinator pre-implementation merge step is complete.

**Risk:** High, because the merge spans the compact orchestration and config surface and must preserve both branch-local policy and upstream default V2 behavior.

**Model tier:** BEST, resolved to `model="gpt-5.5"`, `reasoning_effort="xhigh"`.

**Worker role:** `sp-impl`.

**Outputs and file-level responsibilities:**
- Reconcile upstream’s compact-related commits into the branch without losing the local fast-tier compact policy.
- Keep local fallback when remote compact fails.
- Keep `remote_compact.max_attempts`, `attempt_timeout_sec`, and the schema/validation backing them.
- Preserve upstream’s V2-default routing behavior where the feature flag is enabled.

**Implementation steps:**
1. Inspect the post-merge diffs and conflict resolutions in the listed files.
2. Resolve any remaining compact/config conflicts in the listed files, preferring the branch-local compact policy and config support where it does not contradict upstream’s new default V2 behavior.
3. Make sure `core/src/config/mod.rs`, `config/src/config_toml.rs`, `config/src/types.rs`, and `core/config.schema.json` still accept and validate the requested `remote_compact` settings.
4. Run `just write-config-schema` if the merge or this task changes `ConfigToml`, `RemoteCompactConfigToml`, nested config types, or the schema output.
5. Verify that the compact call sites still route through the shared remote-first wrapper and that no merge step dropped the local fallback path.

**Verification commands:**
```bash
cd /home/gary/git/codex-compact-fix/codex-rs && timeout 60s git diff --check
cd /home/gary/git/codex-compact-fix/codex-rs && timeout 600s just write-config-schema
cd /home/gary/git/codex-compact-fix/codex-rs && timeout 1200s cargo check -p codex-cli -p codex-tui -p codex-core
```

Expected result: the merge result is clean, config schema generation is current, preserved compact/config behavior still typechecks, and no conflict markers remain.

**Completion report requirements:** Changed files, merge outcome, compact behaviors preserved, commands run, results, unresolved risks.

### Task 2: Add The Display-Only Status-Line Version Label

**Goal:** Make the status line show `0.139.0+gary` without changing package metadata or external version checks.

**Contract inputs:** Interface Contract entry 3.

**Serialization required:** No within the post-merge worker phase. This task owns only TUI display and snapshot files and starts only after the coordinator pre-implementation merge step is complete.

**Write scope:**
- `codex-rs/tui/src/version.rs`
- `codex-rs/tui/src/chatwidget/status_surfaces.rs`
- `codex-rs/tui/src/bottom_pane/status_surface_preview.rs`
- `codex-rs/tui/src/chatwidget/tests/status_surface_previews.rs`
- `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__status_surface_previews_codex_version.snap`

**Parallel:** Yes, compatible with Task 1.

**Risk:** Medium, because the change is small but user-visible and needs snapshot coverage.

**Model tier:** NORMAL, resolved to `model="gpt-5.4-mini"`, `reasoning_effort="xhigh"`.

**Worker role:** `sp-impl`.

**Outputs and file-level responsibilities:**
- Add a display-only constant for the status-line version label.
- Route `StatusLineItem::CodexVersion` to that display-only value.
- Keep `CARGO_PKG_VERSION`-based behavior out of the status-line display path.
- Update the preview placeholder and add snapshot coverage for the new label.

**Implementation steps:**
1. Add a display-only version constant in `tui/src/version.rs`.
2. Update `StatusLineItem::CodexVersion` rendering in `tui/src/chatwidget/status_surfaces.rs` to use the display-only constant.
3. Update the preview placeholder in `tui/src/bottom_pane/status_surface_preview.rs`.
4. Add a focused preview test in `tui/src/chatwidget/tests/status_surface_previews.rs` that renders the version item.
5. Accept or update the corresponding insta snapshot.

**Verification commands:**
```bash
cd /home/gary/git/codex-compact-fix/codex-rs && timeout 1200s just test -p codex-tui status_surface_previews
```

Expected result: the focused TUI preview test passes and the new snapshot captures the display-only `0.139.0+gary` label.

**Completion report requirements:** Changed files, exact display constant used, snapshot coverage added, commands run, results, unresolved risks.

## Model Allocation

| Stage | Role | Model tier | Resolved model | Reasoning effort | Reason |
| --- | --- | --- | --- | --- | --- |
| Plan review | REVIEW-tier plan reviewer | REVIEW | `gpt-5.5` | `xhigh` | Plan review must validate merge safety, file ownership, contract clarity, and the three-checkpoint policy. |
| Coordinator pre-implementation merge | Coordinator | BEST | `gpt-5.5` | `xhigh` | The upstream merge is a repository-wide index/worktree mutation and must be serialized before worker dispatch. |
| Task 1 | `sp-impl` compact reconciliation worker | BEST | `gpt-5.5` | `xhigh` | Post-merge compact reconciliation touches the compact and config surfaces and is behavior-shaping. |
| Task 2 | `sp-impl` TUI display worker | NORMAL | `gpt-5.4-mini` | `xhigh` | Localized UI/version-label work with snapshot coverage. |
| Quick verification | FAST-tier quick verifier | FAST | `gpt-5.3-codex-spark` | `high` | Run formatting, schema generation, focused snapshot tests, and the release build after all workers finish. |
| Final review/fix | REVIEW-tier review+fix agent | REVIEW | `gpt-5.5` | `xhigh` | Final whole-change review and fix before the release build and deployment. |

Resolved tier sources for this plan: project root `AGENTS.md` does not define quoted `SIMPLEPOWER_*_MODEL` assignments; process environment sets `SIMPLEPOWER_FAST_MODEL=gpt-5.3-codex-spark-high`, `SIMPLEPOWER_NORMAL_MODEL=gpt-5.4-mini-xhigh`, `SIMPLEPOWER_BEST_MODEL=gpt-5.5-xhigh`, and `SIMPLEPOWER_REVIEW_MODEL=gpt-5.5-xhigh`.

## Aggregate Parallel Dispatch Guidance

The coordinator first runs the serialized pre-implementation merge step and resolves any repository-wide merge state. After the merge result is clean enough for scoped file edits, dispatch Task 1 and Task 2 together because their write scopes do not overlap. The compact reconciliation worker owns the compact/config surface; the TUI worker owns the display-only version label and snapshot. Do not let either worker edit outside its write scope. If the merge exposes an unexpected overlap, stop and report the exact file instead of widening the write scope silently.

## Quick Verification

Before dispatching the quick verifier, create `refs/simplepower/scratch/<run-id>/quick-verifier/before` for these repo-tracked implementation files:

```text
codex-rs/core/src/config/mod.rs
codex-rs/core/src/lib.rs
codex-rs/core/src/compact_service_tier.rs
codex-rs/core/src/remote_compact_fallback.rs
codex-rs/config/src/config_toml.rs
codex-rs/config/src/types.rs
codex-rs/core/config.schema.json
codex-rs/core/src/compact.rs
codex-rs/core/src/compact_remote.rs
codex-rs/core/src/compact_remote_v2.rs
codex-rs/core/src/session/turn.rs
codex-rs/core/src/tasks/compact.rs
codex-rs/tui/src/version.rs
codex-rs/tui/src/chatwidget/status_surfaces.rs
codex-rs/tui/src/bottom_pane/status_surface_preview.rs
codex-rs/tui/src/chatwidget/tests/status_surface_previews.rs
codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__status_surface_previews_codex_version.snap
```

The quick verifier may run and, if necessary, make only tiny typo-level fixes discovered by these commands:

```bash
cd /home/gary/git/codex-compact-fix/codex-rs && timeout 600s just fmt
cd /home/gary/git/codex-compact-fix/codex-rs && timeout 600s just write-config-schema
cd /home/gary/git/codex-compact-fix/codex-rs && timeout 1200s just test -p codex-tui status_surface_previews
cd /home/gary/git/codex-compact-fix/codex-rs && timeout 3600s cargo build --release -p codex-cli
test -x /home/gary/git/codex-compact-fix/codex-rs/target/release/codex
```

Expected result: formatting completes, config schema generation is current, the focused status-line snapshot test passes, and the release Codex binary exists. Failure means the implementation is not ready for the quick-verified implementation checkpoint.

## Final Review And Fix

After the quick-verified implementation checkpoint, dispatch one REVIEW-tier review+fix agent. Before dispatch, create `refs/simplepower/scratch/<run-id>/review-fix/before` for the approved implementation file list. The review+fix agent reviews the whole implementation against the accepted plan, file ownership, compact-preservation contract, display-only version contract, build requirements, and deployment requirements.

The REVIEW-tier review+fix agent must perform the assigned review and fixes directly in its current worker. It must not invoke Simple Power skills, spawn subagents, run Codex CLI, restart execution, or reroute the workflow.

If the review+fix agent edits files, create `refs/simplepower/scratch/<run-id>/review-fix/after` and inspect or hand off:

```bash
git diff refs/simplepower/scratch/<run-id>/review-fix/before refs/simplepower/scratch/<run-id>/review-fix/after -- codex-rs/core/src/config/mod.rs codex-rs/core/src/lib.rs codex-rs/core/src/compact_service_tier.rs codex-rs/core/src/remote_compact_fallback.rs codex-rs/config/src/config_toml.rs codex-rs/config/src/types.rs codex-rs/core/config.schema.json codex-rs/core/src/compact.rs codex-rs/core/src/compact_remote.rs codex-rs/core/src/compact_remote_v2.rs codex-rs/core/src/session/turn.rs codex-rs/core/src/tasks/compact.rs codex-rs/tui/src/version.rs codex-rs/tui/src/chatwidget/status_surfaces.rs codex-rs/tui/src/bottom_pane/status_surface_preview.rs codex-rs/tui/src/chatwidget/tests/status_surface_previews.rs codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__status_surface_previews_codex_version.snap
```

The review+fix agent must report changed files, commands run, results, remaining risks, and any unresolved deviation that requires user approval. It must not commit.

## Final Verification

Run final verification only after the REVIEW-tier review+fix agent completes:

```bash
set -euo pipefail
cd /home/gary/git/codex-compact-fix/codex-rs && timeout 600s just fmt
cd /home/gary/git/codex-compact-fix/codex-rs && timeout 600s just write-config-schema
cd /home/gary/git/codex-compact-fix/codex-rs && timeout 1200s just test -p codex-tui status_surface_previews
cd /home/gary/git/codex-compact-fix/codex-rs && timeout 3600s cargo build --release -p codex-cli
test -x /home/gary/git/codex-compact-fix/codex-rs/target/release/codex
cp /home/gary/git/codex-compact-fix/codex-rs/target/release/codex /home/gary/codex
for host in fpga01 axel office backup desk; do
  ssh "$host" 'killall codex || true; rm -rf ~/.codex/packages/standalone ~/.codex/bin/codex; mkdir -p ~/.local/bin'
  scp /home/gary/git/codex-compact-fix/codex-rs/target/release/codex "$host:~/.local/bin/codex"
  ssh "$host" 'test -x ~/.local/bin/codex'
done
```

Expected result: formatting completes, config schema generation is current, the focused status-line snapshot test passes, the release binary exists, the local copy succeeds, each remote host has an executable `~/.local/bin/codex`, and the deployment loop stops on the first non-recoverable host failure. Failure means the final checkpoint must not be created until the command failure is resolved or the user approves a plan change.

## Commit Checkpoints

1. **Accepted plan checkpoint:** After the user gives combined approval for the reviewed plan, model/task allocation, and immediate current-session execution, and before invoking `simplepower:subagent-driven-development`.
2. **Quick-verified implementation checkpoint:** After all `sp-impl` file edits complete and quick verification passes.
3. **Final checkpoint:** After the REVIEW-tier review+fix agent completes and final verification passes.

Workers, plan reviewers, quick verifiers, and review+fix agents must not commit. Scratch refs are coordinator-owned local review anchors only and must be deleted after successful checkpoints or preserved and reported for manual cleanup if the workflow stops or a checkpoint commit fails.

## Scratch Ref Workflow

Use run id format `YYYYMMDD-HHMMSS-<short-head>`, for example `20260611-120000-79f2799`. All scratch refs for this run live under:

```text
refs/simplepower/scratch/<run-id>/
```

Create `plan-review/before` before the first plan review for this plan file. If the plan is revised after review feedback, create `plan-review/after-<n>` and send the same reviewer this diff command:

```bash
git diff refs/simplepower/scratch/<run-id>/plan-review/before refs/simplepower/scratch/<run-id>/plan-review/after-1 -- docs/simplepower/plans/2026-06-11-upstream-main-compact-preserve-gary-version.md
```

After the accepted plan checkpoint succeeds, delete:

```bash
git for-each-ref --format='%(refname)' "refs/simplepower/scratch/<run-id>/plan-review" | while read -r ref; do git update-ref -d "$ref"; done
```

Use the same phase cleanup pattern for `quick-verifier` after the quick-verified implementation checkpoint and `review-fix` after the final checkpoint. If the workflow stops because of user direction, blocker, or failed checkpoint commit, preserve remaining scratch refs and report the refs with:

```bash
git for-each-ref --format='%(refname)' "refs/simplepower/scratch/<run-id>"
```

Also report this manual cleanup command for the user to run later if desired; do not run it automatically on blockers or failed checkpoints:

```bash
git for-each-ref --format='%(refname)' "refs/simplepower/scratch/<run-id>" | while read -r ref; do git update-ref -d "$ref"; done
```

After the final checkpoint succeeds and the `review-fix` phase refs are deleted, run the final cleanup check:

```bash
git for-each-ref --format='%(refname)' "refs/simplepower/scratch/<run-id>"
```

Expected result: no refs are printed for a fully successful run. If refs remain, report them with the manual cleanup command above.

## Current-Session Auto-Dispatch

After the REVIEW-tier plan reviewer approves, ask the user for one combined approval covering:

- the reviewed plan
- the model/task allocation
- immediate current-session execution

After combined approval, the coordinator creates the accepted plan checkpoint commit that includes this plan file, deletes the successful `plan-review` scratch refs, runs the serialized coordinator pre-implementation merge step, then immediately invokes `simplepower:subagent-driven-development` in the current session with:

```text
Execute `docs/simplepower/plans/2026-06-11-upstream-main-compact-preserve-gary-version.md` with aggregate parallel implementation from the approved Interface Contract after the coordinator-owned upstream merge step has completed. Use the approved FAST/NORMAL/BEST/REVIEW model allocation. Dispatch all non-conflicting `sp-impl` file-edit workers whose coordination needs are satisfied by their Contract inputs, run the quick FAST-tier verifier with the approved format, config-schema, focused test, and release-build commands after all workers finish, commit the quick-verified implementation, then run one REVIEW-tier review+fix agent, final verification, local copy, fail-fast remote deployment, and final commit.
```
