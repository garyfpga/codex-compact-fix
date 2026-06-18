# Compact Fix Changelog Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `simplepower:subagent-driven-development` for aggregate parallel implementation. Dispatch all non-conflicting `sp-impl` file-edit workers whose coordination needs are satisfied by the approved Interface Contract, run the quick verifier after all workers finish, commit the quick-verified implementation, then run one REVIEW-tier review+fix agent before final verification and final commit.

**Goal:** Create `docs/compact-fix/ChangeLog.md` as the entry point for future agents merging upstream while preserving this fork's compact-fix behavior.

**Design Summary:** The approved design documents the fork-local delta from upstream baseline `f42780109c..HEAD`, grouped by behavior instead of commit chronology. The changelog must explain what this fork modified, why those changes exist, where the current code anchors are, and what future agents should preserve when pulling newer upstream `main`. The document must also include a compact changed-file inventory for coverage.

**Architecture:** This is a documentation-only change with one generated entry-point file. The Interface Contract fixes the baseline, required sections, and verification shape so one `sp-impl` worker can write the changelog without touching code or tests.

**Tech Stack:** Git, Markdown, repository-local Rust source references, and shell verification commands.

**Model Allocation:** FAST/NORMAL/BEST/REVIEW tiers are assigned below. Resolve each tier by explicit user override, quoted assignment in project root AGENTS.md, process environment variable, then built-in default. The project root AGENTS.md lookup reads only `<repo>/AGENTS.md`, not nested AGENTS.md files or repo-wide grep. FAST defaults to `SIMPLEPOWER_FAST_MODEL` (`gpt-5.3-codex-spark-high` when unset), NORMAL defaults to `SIMPLEPOWER_NORMAL_MODEL` (`gpt-5.4-mini-high` when unset), BEST defaults to `SIMPLEPOWER_BEST_MODEL` (`gpt-5.5-high` when unset), and REVIEW defaults to `SIMPLEPOWER_REVIEW_MODEL` (`gpt-5.5-xhigh` when unset). The plan reviewer is a REVIEW-tier plan reviewer, and the final review+fix agent is a REVIEW-tier review+fix agent. The quick verifier uses the FAST tier by default, resolving to `model="gpt-5.3-codex-spark"` and `reasoning_effort="high"` unless `SIMPLEPOWER_FAST_MODEL` is overridden.

**Commit Policy:** The coordinator commits after the reviewed plan, allocation, and immediate current-session execution receive combined approval, after all file edits and quick verification complete before final review, and after final review/fix plus final verification. Workers, plan reviewers, quick verifiers, and review+fix agents must not commit. No per-task commits. Coordinator-owned temporary scratch refs under `refs/simplepower/scratch/<run-id>/...` may be created only as local review diff anchors; they are not accepted history commits, not pushed, not merged, not rebased, and must be cleaned up after successful checkpoints or reported for manual cleanup on blockers or failed checkpoints.

---

## Interface Contract

1. **Baseline contract**
   - The changelog documents the intentional fork delta from upstream merge-base commit `f42780109c` to current `HEAD`.
   - The current `HEAD` at planning time is `b722574da3`, the merge commit titled `merge: upstream main with compact preservation`.
   - Do not compare against the moving local `upstream/main` pointer when writing the changelog, because that currently includes later upstream work outside this fork's last synced baseline.

2. **Document path and role**
   - Create `docs/compact-fix/ChangeLog.md`.
   - The document is an entry point for future agents that pull newer upstream `main`, resolve overlaps, preserve this fork's early compact features, and release a new binary.
   - The document must be self-contained enough that a future agent can start from it before inspecting the code.

3. **Required changelog sections**
   - `# Compact Fix ChangeLog`
   - `## Purpose`
   - `## Baseline`
   - `## Preservation Checklist`
   - `## Behavior Changes`
   - `## Changed File Inventory`
   - `## Future Upstream Merge Procedure`

4. **Behavior-entry contract**
   - Each behavior entry under `## Behavior Changes` must include:
     - `Files`
     - `Current anchors`
     - `What changed`
     - `Why`
     - `Future merge notes`
   - Current anchors must use existing file paths and line numbers from the current worktree.
   - The line anchors are current-maintenance pointers, not immutable permalinks.

5. **Behavior groups**
   - Cover these groups:
     - `remote_compact` config schema and validation
     - Shared remote-first fallback policy for V1 and V2
     - Fast service tier override for compaction only
     - V1 remote compact retry, timeout, and TCP keepalive plumbing
     - V2 remote compact policy parity
     - Auto and manual compact call-site routing
     - API/client retry and TCP keepalive support touched by compact
     - Integration tests and snapshots that preserve behavior
     - Display-only TUI version label `0.139.0+gary`
     - Simple Power plan history as the rationale trail

6. **Changed-file inventory contract**
   - The inventory must list every file from `git diff --name-status f42780109c..HEAD`.
   - The inventory should preserve status letters such as `A` and `M`.
   - The new changelog file itself is not part of that baseline inventory until after it is committed; do not add it to the inventory.

7. **Verification contract**
   - Verification is documentation-focused.
   - Do not run Rust tests for this docs-only change.
   - Required checks:
     - `docs/compact-fix/ChangeLog.md` exists.
     - The baseline inventory in the changelog matches `git diff --name-status f42780109c..HEAD`.
     - The changelog contains no unresolved placeholder markers.

## File Ownership

| File | Owner task | Change type | Responsibility | Parallel safety notes |
| --- | --- | --- | --- | --- |
| `docs/simplepower/plans/2026-06-18-compact-fix-changelog.md` | Coordinator | create | Authoritative implementation plan for this documentation change. | Coordinator-owned; not edited by implementation worker unless the plan review loop requires revision. |
| `docs/compact-fix/ChangeLog.md` | Task 1 | create | Behavior-oriented changelog and future upstream merge guide for the fork delta `f42780109c..HEAD`. | Exclusive to Task 1. |

## Implementation Tasks

### Task 1: Create Compact Fix Changelog

**Goal:** Create `docs/compact-fix/ChangeLog.md` as a durable behavior-preservation map for the fork-local compact changes.

**Contract inputs:** Interface Contract entries 1 through 7.

**Serialization required:** No. The task has a single exclusive write scope and no generated prerequisite.

**Write scope:**
- `docs/compact-fix/ChangeLog.md`

**Parallel:** Yes, compatible with no other implementation tasks because there is only one docs task.

**Risk:** Low. This is documentation-only, but it must be accurate enough to guide future upstream merges.

**Model tier:** NORMAL, resolved to `model="gpt-5.4-mini"`, `reasoning_effort="xhigh"`.

**Worker role:** `sp-impl`

**Outputs and file-level responsibilities:**
- Create the `docs/compact-fix/` directory if needed.
- Write `ChangeLog.md` using the required section structure.
- Group changes by behavior, not by commit.
- Include current file and line-number anchors for each behavior group.
- Include the complete baseline changed-file inventory from `git diff --name-status f42780109c..HEAD`.
- Include future merge guidance that tells agents to preserve the checklist and update anchors after resolving upstream changes.

**Implementation steps:**
1. Inspect the current line anchors with focused commands such as:
   ```bash
   cd /home/gary/git/codex-compact-fix
   rg -n "RemoteCompactConfigToml|remote_compact|max_attempts|attempt_timeout|tcp_keepalive_interval_ms|RemoteCompactVersion|run_remote_first|resolve_remote_first_compact_service_tiers|RemoteCompactionRunSettings|RemoteCompactionV2RunSettings|CODEX_CLI_DISPLAY_VERSION|CodexVersion|0\\.139\\.0\\+gary" codex-rs docs/simplepower/plans
   ```
2. Generate the baseline inventory:
   ```bash
   cd /home/gary/git/codex-compact-fix
   git diff --name-status f42780109c..HEAD
   ```
3. Create `docs/compact-fix/ChangeLog.md` with the approved behavior-oriented structure.
4. For each behavior entry, write concise rationale that explains why the fork carries the change ahead of upstream and what future merge agents must preserve.
5. Keep the document plain Markdown with no generated HTML or external assets.

**Verification commands:**
```bash
cd /home/gary/git/codex-compact-fix && timeout 30s test -f docs/compact-fix/ChangeLog.md
cd /home/gary/git/codex-compact-fix && timeout 30s sh -c '! rg -n "T[B]D|T[O]DO|fill[ ]in[ ]details|implement[ ]later" docs/compact-fix/ChangeLog.md'
cd /home/gary/git/codex-compact-fix && timeout 30s sh -c 'tmp=$(mktemp); git diff --name-status f42780109c..HEAD > "$tmp"; sed -n "/^## Changed File Inventory$/,/^## Future Upstream Merge Procedure$/p" docs/compact-fix/ChangeLog.md | rg "^[AMDRC]\t" > "$tmp.doc"; diff -u "$tmp" "$tmp.doc"'
```

Expected result: the changelog exists, contains no placeholders, and its changed-file inventory exactly matches `git diff --name-status f42780109c..HEAD`.

**Completion report requirements:** Changed files, baseline commit used, behavior groups covered, verification commands run, command results, and unresolved risks.

## Model Allocation

| Stage | Role | Model tier | Resolved model | Reasoning effort | Reason |
| --- | --- | --- | --- | --- | --- |
| Plan review | REVIEW-tier plan reviewer | REVIEW | `gpt-5.5` | `xhigh` | Validate the plan structure, file ownership, verification, and Simple Power execution contract. |
| Task 1 | `sp-impl` docs worker | NORMAL | `gpt-5.4-mini` | `xhigh` | Localized documentation creation requiring careful code-anchor accuracy. |
| Quick verification | FAST-tier quick verifier | FAST | `gpt-5.3-codex-spark` | `high` | Mechanical checks for file existence, placeholder absence, and inventory consistency. |
| Final review/fix | REVIEW-tier review+fix agent | REVIEW | `gpt-5.5` | `xhigh` | Final review of the changelog for accuracy, completeness, and future-merge usefulness. |

Resolved tier sources for this plan: project root `AGENTS.md` does not define quoted `SIMPLEPOWER_*_MODEL` assignments; process environment sets `SIMPLEPOWER_FAST_MODEL=gpt-5.3-codex-spark-high`, `SIMPLEPOWER_NORMAL_MODEL=gpt-5.4-mini-xhigh`, `SIMPLEPOWER_BEST_MODEL=gpt-5.5-xhigh`, and `SIMPLEPOWER_REVIEW_MODEL=gpt-5.5-xhigh`.

## Plan Review

Before the first plan review, the coordinator creates `refs/simplepower/scratch/<run-id>/plan-review/before` for this plan file. The REVIEW-tier plan reviewer reviews this plan against the approved brainstorming design, Interface Contract, File Ownership, task allocation, model allocation, verification, review/fix policy, commit checkpoints, scratch-ref workflow, and current-session auto-dispatch requirements.

The REVIEW-tier plan reviewer must perform the assigned plan review directly in its current worker. It must not invoke Simple Power skills, spawn subagents, run Codex CLI, restart execution, or reroute the workflow.

If the reviewer reports blocking issues, the coordinator revises only this plan file, reruns focused self-review for the changed categories, creates `refs/simplepower/scratch/<run-id>/plan-review/after-<n>`, and sends the same reviewer a concrete diff command. For the first revision, use:

```bash
git diff refs/simplepower/scratch/<run-id>/plan-review/before refs/simplepower/scratch/<run-id>/plan-review/after-1 -- docs/simplepower/plans/2026-06-18-compact-fix-changelog.md
```

The reviewer loop stays open until the plan is approved, the user explicitly redirects, or an unrecoverable blocker is reported. After reviewer approval, ask the user for combined approval of the reviewed plan, model/task allocation, and immediate current-session execution. The accepted plan checkpoint happens only after that combined approval.

## Aggregate Parallel Dispatch Guidance

There is one implementation worker, so aggregate parallel dispatch contains only Task 1. The worker may inspect any repository file needed for documentation accuracy but may only create or edit `docs/compact-fix/ChangeLog.md`. The quick verifier runs after Task 1 completes and before the coordinator creates the quick-verified implementation checkpoint.

## Quick Verification

Before dispatching the quick verifier, create `refs/simplepower/scratch/<run-id>/quick-verifier/before` for:

```text
docs/compact-fix/ChangeLog.md
```

The quick verifier may make only tiny typo-level fixes found by these checks:

```bash
cd /home/gary/git/codex-compact-fix && timeout 30s test -f docs/compact-fix/ChangeLog.md
cd /home/gary/git/codex-compact-fix && timeout 30s sh -c '! rg -n "T[B]D|T[O]DO|fill[ ]in[ ]details|implement[ ]later" docs/compact-fix/ChangeLog.md'
cd /home/gary/git/codex-compact-fix && timeout 30s sh -c 'tmp=$(mktemp); git diff --name-status f42780109c..HEAD > "$tmp"; sed -n "/^## Changed File Inventory$/,/^## Future Upstream Merge Procedure$/p" docs/compact-fix/ChangeLog.md | rg "^[AMDRC]\t" > "$tmp.doc"; diff -u "$tmp" "$tmp.doc"'
```

Expected result: the changelog exists, has no placeholder text, and the inventory block matches the baseline diff exactly. Failure means the implementation is not ready for the quick-verified implementation checkpoint.

If the quick verifier makes any allowed typo-level edits, the coordinator creates `refs/simplepower/scratch/<run-id>/quick-verifier/after` before the quick-verified implementation checkpoint and inspects or hands off:

```bash
git diff refs/simplepower/scratch/<run-id>/quick-verifier/before refs/simplepower/scratch/<run-id>/quick-verifier/after -- docs/compact-fix/ChangeLog.md
```

If the quick verifier makes no file changes, omit `quick-verifier/after`. After the quick-verified implementation checkpoint succeeds, delete the `quick-verifier` scratch refs. If the checkpoint fails or the workflow stops before that checkpoint, preserve the refs and report the manual cleanup command from the Scratch Ref Workflow section.

## Final Review And Fix

After the quick-verified implementation checkpoint, dispatch one REVIEW-tier review+fix agent. Before dispatch, create `refs/simplepower/scratch/<run-id>/review-fix/before` for:

```text
docs/compact-fix/ChangeLog.md
```

The REVIEW-tier review+fix agent must perform the assigned review and fixes directly in its current worker. It must not invoke Simple Power skills, spawn subagents, run Codex CLI, restart execution, or reroute the workflow.

The review must check that sampled file and line-number anchors in `docs/compact-fix/ChangeLog.md` resolve in the current worktree and that each behavior entry explains both what changed and why it must be preserved during future upstream merges.

If the review+fix agent edits files, create `refs/simplepower/scratch/<run-id>/review-fix/after` and inspect or hand off:

```bash
git diff refs/simplepower/scratch/<run-id>/review-fix/before refs/simplepower/scratch/<run-id>/review-fix/after -- docs/compact-fix/ChangeLog.md
```

The review+fix agent must report changed files, commands run, results, remaining risks, and any unresolved deviation that requires user approval. It must not commit.

## Final Verification

Run final verification only after the REVIEW-tier review+fix agent completes:

```bash
cd /home/gary/git/codex-compact-fix && timeout 30s test -f docs/compact-fix/ChangeLog.md
cd /home/gary/git/codex-compact-fix && timeout 30s sh -c '! rg -n "T[B]D|T[O]DO|fill[ ]in[ ]details|implement[ ]later" docs/compact-fix/ChangeLog.md'
cd /home/gary/git/codex-compact-fix && timeout 30s sh -c 'tmp=$(mktemp); git diff --name-status f42780109c..HEAD > "$tmp"; sed -n "/^## Changed File Inventory$/,/^## Future Upstream Merge Procedure$/p" docs/compact-fix/ChangeLog.md | rg "^[AMDRC]\t" > "$tmp.doc"; diff -u "$tmp" "$tmp.doc"'
```

Expected result: the changelog exists, contains no placeholders, and its inventory exactly matches the baseline diff. Failure means the final checkpoint must not be created until the issue is fixed or the user approves a plan change.

At final reporting, run:

```bash
git for-each-ref --format='%(refname)' "refs/simplepower/scratch/<run-id>"
```

Expected result after successful cleanup: no refs are printed. If the workflow stops because of user direction, blocker, or failed checkpoint commit, preserve remaining scratch refs and report this manual cleanup command:

```bash
git for-each-ref --format='%(refname)' "refs/simplepower/scratch/<run-id>" | while read -r ref; do git update-ref -d "$ref"; done
```

## Commit Checkpoints

1. **Accepted plan checkpoint:** After the user gives combined approval for the reviewed plan, model/task allocation, and immediate current-session execution, and before invoking `simplepower:subagent-driven-development`.
2. **Quick-verified implementation checkpoint:** After the `sp-impl` file edit completes and quick verification passes.
3. **Final checkpoint:** After the REVIEW-tier review+fix agent completes and final verification passes.

Workers, plan reviewers, quick verifiers, and review+fix agents must not commit. Scratch refs are coordinator-owned local review anchors only and must be deleted after successful checkpoints or preserved and reported for manual cleanup if the workflow stops or a checkpoint commit fails.

## Scratch Ref Workflow

Use run id format `YYYYMMDD-HHMMSS-<short-head>`, for example `20260618-120000-b722574`. All scratch refs for this run live under:

```text
refs/simplepower/scratch/<run-id>/
```

Create `plan-review/before` before the first plan review for this plan file. If the plan is revised after review feedback, create `plan-review/after-<n>` and send the same reviewer this diff command:

```bash
git diff refs/simplepower/scratch/<run-id>/plan-review/before refs/simplepower/scratch/<run-id>/plan-review/after-1 -- docs/simplepower/plans/2026-06-18-compact-fix-changelog.md
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

After combined approval, the coordinator creates the accepted plan checkpoint commit that includes this plan file, deletes the successful `plan-review` scratch refs, then immediately invokes `simplepower:subagent-driven-development` in the current session with:

```text
Execute `docs/simplepower/plans/2026-06-18-compact-fix-changelog.md` with aggregate parallel implementation from the approved Interface Contract. Use the approved FAST/NORMAL/BEST/REVIEW model allocation. Dispatch the `sp-impl` docs worker for `docs/compact-fix/ChangeLog.md`, run the quick FAST-tier verifier with the approved documentation checks and timeouts after the worker finishes, commit the quick-verified implementation, then run one REVIEW-tier review+fix agent, final verification, and final commit.
```
