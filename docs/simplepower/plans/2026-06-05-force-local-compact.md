# Force Local Compact Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `simplepower:subagent-driven-development` for aggregate parallel implementation. Dispatch all non-conflicting `sp-impl` file-edit workers whose coordination needs are satisfied by the approved Interface Contract, run the quick verifier after all workers finish, commit the quick-verified implementation, then run one REVIEW-tier review+fix agent before final verification and final commit.

**Goal:** Force codex-core compaction routing to use local compaction regardless of provider/model remote compaction capability.

**Design Summary:** The approved design keeps `ModelProviderInfo::supports_remote_compaction()` unchanged and changes only codex-core's compaction routing policy. Manual `/compact`, pre-turn auto compact, mid-turn auto compact, and model-downshift compact all route through `codex-rs/core/src/compact.rs::should_use_remote_compact_task`; forcing that wrapper to return `false` makes these flows use the existing local compaction path. The motivation to mention in commits is that forcing local compact can fix failures caused by remote compact. The user explicitly requested skipping all tests and compiling the Rust binary in release mode only.

**Architecture:** Keep provider capability metadata intact and make the core routing layer the single behavior override. The Interface Contract below pins the only public behavior change, so a single implementation worker can update the policy and the directly related assertion without coordinating across broader compaction modules.

**Tech Stack:** Rust workspace under `codex-rs`, codex-core compaction routing, `just` repository commands, Bazel release build via `just build-for-release`.

**Coordinator Precondition Completed:** The coordinator checked out `main`, pulled `origin/main` with `--ff-only`, confirmed it was already up to date, and created branch `fix/force-local-compact` before saving this plan.

**Model Allocation:** FAST/NORMAL/BEST/REVIEW tiers are assigned below. Resolve each tier by explicit user override, quoted assignment in project root AGENTS.md, process environment variable, then built-in default. The project root AGENTS.md lookup reads only `<repo>/AGENTS.md`, not nested AGENTS.md files or repo-wide grep. FAST defaults to `SIMPLEPOWER_FAST_MODEL` (`gpt-5.3-codex-spark-high` when unset), NORMAL defaults to `SIMPLEPOWER_NORMAL_MODEL` (`gpt-5.4-mini-high` when unset), BEST defaults to `SIMPLEPOWER_BEST_MODEL` (`gpt-5.5-high` when unset), and REVIEW defaults to `SIMPLEPOWER_REVIEW_MODEL` (`gpt-5.5-xhigh` when unset). The plan reviewer is a REVIEW-tier plan reviewer, and the final review+fix agent is a REVIEW-tier review+fix agent. The quick verifier uses the FAST tier by default, resolving to `model="gpt-5.3-codex-spark"` and `reasoning_effort="high"` unless `SIMPLEPOWER_FAST_MODEL` is overridden.

**Commit Policy:** The coordinator commits after the reviewed plan, allocation, and immediate current-session execution receive combined approval, after all file edits and quick verification complete before final review, and after final review/fix plus final verification. Workers, plan reviewers, quick verifiers, and review+fix agents must not commit. No per-task commits. Coordinator-owned temporary scratch refs under `refs/simplepower/scratch/<run-id>/...` may be created only as local review diff anchors; they are not accepted history commits, not pushed, not merged, not rebased, and must be cleaned up after successful checkpoints or reported for manual cleanup on blockers or failed checkpoint commits.

---

## Interface Contract

- `codex-rs/core/src/compact.rs::should_use_remote_compact_task(provider: &ModelProviderInfo) -> bool` remains the codex-core compaction routing API used by manual and automatic compaction call sites.
- `should_use_remote_compact_task` must always return `false` for every `ModelProviderInfo`, including OpenAI and Azure providers that still report `supports_remote_compaction() == true`.
- `codex-rs/model-provider-info/src/lib.rs::ModelProviderInfo::supports_remote_compaction()` and its tests must not be changed.
- Existing local compaction functions remain the implementation path: `run_compact_task` for manual compaction and `run_inline_auto_compact_task` for automatic compaction.
- Existing remote compaction modules, feature flags, telemetry labels, and `/responses/compact` client code remain present but are not selected by codex-core routing after this change.
- User-requested verification contract: skip all tests; run formatting and a release-only Rust binary build.
- Release build command: from repo root, run `timeout 3600s just build-for-release`; this invokes Bazel release binaries for `//codex-rs/cli:release_binaries`.

## File Ownership

| File | Owner task | Change type | Responsibility | Parallel safety notes |
| --- | --- | --- | --- | --- |
| `docs/simplepower/plans/2026-06-05-force-local-compact.md` | Coordinator | create | Authoritative Simple Power implementation plan. | Coordinator-owned; workers must not edit unless explicitly directed by coordinator. |
| `codex-rs/core/src/compact.rs` | Force local compaction routing | modify | Make `should_use_remote_compact_task` always return `false` without changing provider capability metadata. | Single implementation task owns this file. |
| `codex-rs/core/src/compact_tests.rs` | Force local compaction routing | modify | Update the direct codex-core routing assertion to match forced-local behavior while leaving model-provider capability tests alone. | Same task owns the coupled assertion update. |

## Implementation Tasks

### Task 1: Force local compaction routing

Goal: Make codex-core choose local compaction for all providers while preserving provider capability reporting.

Contract inputs: Interface Contract entries for `should_use_remote_compact_task`, unchanged `ModelProviderInfo::supports_remote_compaction()`, local compaction paths, and user-requested verification contract.

Serialization required: No. The task owns all implementation files and has no dependency on other workers.

Write scope:
- `codex-rs/core/src/compact.rs`
- `codex-rs/core/src/compact_tests.rs`

Parallel: Yes, compatible with no other file-edit tasks because this plan has only one implementation task.

Risk: Low. The change is localized to a policy wrapper used by existing manual and auto compaction call sites.

Model tier: NORMAL, resolved model `gpt-5.4-mini`, reasoning effort `xhigh`.

Worker role: `sp-impl`

Outputs and file-level responsibilities:
- In `compact.rs`, update `should_use_remote_compact_task` so it always returns `false`; avoid unused-parameter warnings.
- In `compact_tests.rs`, update the direct routing test so it asserts Azure remote-capable providers do not use remote compaction through codex-core routing. Do not edit `model-provider-info` capability tests.

Implementation steps:
1. Confirm the branch is `fix/force-local-compact` and the working tree contains only approved plan changes before code edits.
2. Edit `codex-rs/core/src/compact.rs`:
   ```rust
   pub(crate) fn should_use_remote_compact_task(_provider: &ModelProviderInfo) -> bool {
       false
   }
   ```
3. Edit `codex-rs/core/src/compact_tests.rs` by renaming or updating `should_use_remote_compact_task_for_azure_provider` so it asserts `!should_use_remote_compact_task(&provider)`.
4. Leave `codex-rs/model-provider-info/src/lib.rs` and `codex-rs/model-provider-info/src/model_provider_info_tests.rs` untouched.
5. Run `timeout 120s just fmt` from repo root after code edits.

Verification commands:
- `timeout 120s just fmt`
  Expected: formatting completes successfully. Failure means the source tree may not meet repository formatting requirements.
- `timeout 3600s just build-for-release`
  Expected: release binaries build successfully. Failure means the release binary is not compiled and the coordinator must report the exact blocker before changing build approach.

Completion report requirements: changed files, commands run, command results, confirmation that tests were skipped by explicit user request, and unresolved risks.

## Model Allocation

| Stage | Role | Model tier | Resolved model | Reasoning effort | Reason |
| --- | --- | --- | --- | --- | --- |
| Plan review | REVIEW-tier plan reviewer | REVIEW | `gpt-5.5` | `xhigh` | Required by `simplepower:writing-plans`; review the plan contract, ownership, allocation, and verification. |
| Implementation Task 1 | `sp-impl` | NORMAL | `gpt-5.4-mini` | `xhigh` | Routine low-risk localized Rust policy edit with one coupled assertion update. |
| Quick verification | FAST-tier quick verifier | FAST | `gpt-5.3-codex-spark` | `high` | Run formatting and release build checks without changing behavior. |
| Final review/fix | REVIEW-tier review+fix agent | REVIEW | `gpt-5.5` | `xhigh` | Required final review against the accepted plan and approved path before final verification. |

Resolved model tier sources:
- FAST from environment: `SIMPLEPOWER_FAST_MODEL=gpt-5.3-codex-spark-high`.
- NORMAL from environment: `SIMPLEPOWER_NORMAL_MODEL=gpt-5.4-mini-xhigh`.
- BEST from environment: `SIMPLEPOWER_BEST_MODEL=gpt-5.5-xhigh`.
- REVIEW from environment: `SIMPLEPOWER_REVIEW_MODEL=gpt-5.5-xhigh`.
- Project root `AGENTS.md` contains no quoted `SIMPLEPOWER_*_MODEL` assignments.

## Plan Review

Self-review checklist status:
- Design Summary captures the approved design, user constraints, success criteria, and commit-message rationale.
- Interface Contract defines the exact routing API, unchanged provider capability API, behavior guarantee, and verification contract.
- File Ownership assigns every planned file to exactly one owner.
- Task allocation maps all requirements to one implementation task with explicit Contract inputs.
- Aggregate parallel readiness is satisfied; only one file-edit worker exists, so no conflicting write scopes exist.
- Visual aids are omitted because this non-visual routing change does not benefit from diagrams.
- Model allocation resolves FAST/NORMAL/BEST/REVIEW by the required precedence and environment values.
- Review allocation includes one REVIEW-tier plan reviewer and one REVIEW-tier final review+fix agent.
- Reviewer non-recursion is explicit for both the plan reviewer and final review+fix agent.
- Commit policy defines exactly three coordinator checkpoints and no worker commits.
- Scratch refs use coordinator-only `refs/simplepower/scratch/<run-id>/...` anchors and phase cleanup.
- Verification commands are concrete and use `timeout`; tests are omitted only because the user explicitly requested skipping all tests.
- Approved path enforcement does not authorize alternate implementation or build routes.

Coordinator scratch-ref guidance:
- Run id format: `YYYYMMDD-HHMMSS-<short-head>`.
- Before first plan review, create `refs/simplepower/scratch/<run-id>/plan-review/before` for `docs/simplepower/plans/2026-06-05-force-local-compact.md`.
- If the plan is revised after review, create `plan-review/after-<n>` and hand the reviewer this diff command:
  ```bash
  git diff refs/simplepower/scratch/<run-id>/plan-review/before refs/simplepower/scratch/<run-id>/plan-review/after-<n> -- docs/simplepower/plans/2026-06-05-force-local-compact.md
  ```
- After the accepted plan checkpoint commit succeeds, delete `refs/simplepower/scratch/<run-id>/plan-review/*`.

## Quick Verification

Before quick verification, create `refs/simplepower/scratch/<run-id>/quick-verifier/before` for:
- `codex-rs/core/src/compact.rs`
- `codex-rs/core/src/compact_tests.rs`

Quick verifier commands:
- `timeout 120s just fmt`
  Expected result: success with no formatting errors.
- `timeout 3600s just build-for-release`
  Expected result: release binary build succeeds.

The quick verifier may fix only typo-level issues discovered while running these commands. Any behavior change, test rewrite, public interface change, or build-route change must be reported to the coordinator instead of fixed. If tiny fixes are made, create `refs/simplepower/scratch/<run-id>/quick-verifier/after` and inspect:

```bash
git diff refs/simplepower/scratch/<run-id>/quick-verifier/before refs/simplepower/scratch/<run-id>/quick-verifier/after -- codex-rs/core/src/compact.rs codex-rs/core/src/compact_tests.rs
```

After the quick-verified implementation checkpoint commit succeeds, delete `refs/simplepower/scratch/<run-id>/quick-verifier/*`.

## Final Review And Fix

After the quick-verified implementation checkpoint, dispatch one REVIEW-tier review+fix agent. The review+fix agent reviews:
- `codex-rs/core/src/compact.rs`
- `codex-rs/core/src/compact_tests.rs`
- The accepted plan constraints that provider capability metadata remains unchanged and tests are skipped by explicit user request.

Before review+fix, create `refs/simplepower/scratch/<run-id>/review-fix/before` for the implementation files. If review+fix edits files, create `refs/simplepower/scratch/<run-id>/review-fix/after` and inspect:

```bash
git diff refs/simplepower/scratch/<run-id>/review-fix/before refs/simplepower/scratch/<run-id>/review-fix/after -- codex-rs/core/src/compact.rs codex-rs/core/src/compact_tests.rs
```

The review+fix agent must not commit. After the final checkpoint succeeds, delete `refs/simplepower/scratch/<run-id>/review-fix/*`.

The REVIEW-tier review+fix agent must perform the assigned review and any approved fixes directly in the current worker. It must not run Codex CLI, spawn subagents, invoke Simple Power skills, restart execution, or reroute the workflow.

## Commit Checkpoints

1. Accepted plan checkpoint: after the user gives combined approval for the reviewed plan, model/task allocation, and immediate current-session execution, and before invoking `simplepower:subagent-driven-development`.
2. Quick-verified implementation checkpoint: after the implementation task completes and quick verification passes.
3. Final checkpoint: after the REVIEW-tier review+fix agent completes and final verification passes.

Commit message guidance:
- Include the rationale that forcing local compact can fix failures from remote compact.
- Do not mention test execution as if tests were run; explicitly note when tests were skipped by request in final reporting.

Workers, plan reviewers, quick verifiers, and review+fix agents must not commit. Scratch refs are local-only review anchors and must be cleaned up after successful phase checkpoints or preserved and reported if a blocker stops the workflow.

## Current-Session Auto-Dispatch

After the plan reviewer approves, ask the user for one combined approval covering:
- The reviewed plan.
- The model/task allocation.
- Immediate current-session execution.

After combined approval, the coordinator creates the accepted plan checkpoint commit, deletes successful `plan-review` scratch refs, and immediately invokes `simplepower:subagent-driven-development` with:

```text
Execute `docs/simplepower/plans/2026-06-05-force-local-compact.md` with aggregate parallel implementation from the approved Interface Contract. Use the approved FAST/NORMAL/BEST/REVIEW model allocation. Dispatch all non-conflicting `sp-impl` file-edit workers whose coordination needs are satisfied by their Contract inputs, run the quick FAST-tier verifier with the approved formatting and release-build commands after all workers finish, commit the quick-verified implementation, then run one REVIEW-tier review+fix agent, final verification, and final commit. Do not run tests; they are explicitly skipped by user request.
```

## Verification

Final verification commands:
- After implementation and quick verification, run `timeout 120s just fmt`.
  Expected result: success. Failure means formatting is not complete.
- After final review/fix, run `timeout 3600s just build-for-release`.
  Expected result: success. Failure means the release binary was not compiled, and the coordinator must report the blocker before changing build approach.

No test command is included because the user explicitly requested skipping all tests. The coordinator performs the final checkpoint only after the REVIEW-tier review+fix agent has completed and the final commands pass.

Final cleanup check:

```bash
git for-each-ref --format='%(refname)' "refs/simplepower/scratch/<run-id>"
```

If the final checkpoint succeeds, no scratch refs for the run should remain. If the workflow stops because of user direction, a blocker, or a failed checkpoint commit, preserve remaining refs and report this manual cleanup command:

```bash
git for-each-ref --format='%(refname)' "refs/simplepower/scratch/<run-id>" | while read -r ref; do git update-ref -d "$ref"; done
```
