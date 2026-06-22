# Modelp Persistent Model Command Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `simplepower:subagent-driven-development` for aggregate parallel implementation. Dispatch all non-conflicting `sp-impl` file-edit workers whose coordination needs are satisfied by the approved Interface Contract, run the quick verifier after all workers finish, commit the quick-verified implementation, then run one REVIEW-tier review+fix agent before final verification and final commit.

**Goal:** Make `/model` change the active model without writing `config.toml`, and add visible `/modelp` as the persistent version that saves the selected model and reasoning effort.

**Design Summary:** The approved design adds a visible `/modelp` command that duplicates the existing `/model` picker but preserves the current config-writing behavior. `/model` becomes temporary: model and reasoning selections update the active runtime/thread settings only and must not emit config persistence events, including Plan-mode reasoning override persistence. `/modelp` carries a persistence mode through the quick model picker, the all-models picker, reasoning selection, and Plan-mode scope prompt so each selection path has one clear rule. Success means `/model` emits `UpdateModel` and `UpdateReasoningEffort` only, `/modelp` emits those runtime updates plus `PersistModelSelection` where the existing persistent flow did, and user-visible command lists show `/modelp`.

**Architecture:** Add a crate-local persistence-mode contract for model picker events, then route both slash commands into the same popup implementation with different mode values. The Interface Contract below lets the runtime worker add the mode and slash command while the test worker writes assertions and snapshots against the approved event/API shape without waiting for implementation details.

**Tech Stack:** Rust, Codex TUI, ratatui selection popups, `AppEvent`, `just`, `insta` snapshots.

**Model Allocation:** FAST/NORMAL/BEST/REVIEW tiers are assigned below. Resolve each tier by explicit user override, quoted assignment in project root AGENTS.md, process environment variable, then built-in default. The project root AGENTS.md lookup reads only `<repo>/AGENTS.md`, not nested AGENTS.md files or repo-wide grep. FAST defaults to `SIMPLEPOWER_FAST_MODEL` (`gpt-5.3-codex-spark-high` when unset), NORMAL defaults to `SIMPLEPOWER_NORMAL_MODEL` (`gpt-5.4-mini-high` when unset), BEST defaults to `SIMPLEPOWER_BEST_MODEL` (`gpt-5.5-high` when unset), and REVIEW defaults to `SIMPLEPOWER_REVIEW_MODEL` (`gpt-5.5-xhigh` when unset). The plan reviewer is a REVIEW-tier plan reviewer, and the final review+fix agent is a REVIEW-tier review+fix agent. The quick verifier uses the FAST tier by default, resolving to `model="gpt-5.3-codex-spark"` and `reasoning_effort="high"` unless `SIMPLEPOWER_FAST_MODEL` is overridden.

**Commit Policy:** The coordinator commits after the reviewed plan, allocation, and immediate current-session execution receive combined approval, after all file edits and quick verification complete before final review, and after final review/fix plus final verification. Workers, plan reviewers, quick verifiers, and review+fix agents must not commit. No per-task commits. Coordinator-owned temporary scratch refs under `refs/simplepower/scratch/<run-id>/...` may be created only as local review diff anchors; they are not accepted history commits, not pushed, not merged, not rebased, and must be cleaned up after successful checkpoints or reported for manual cleanup on blockers or failed checkpoints.

---

## Interface Contract

1. `SlashCommand::Model` remains serialized as `model`, stays visible, remains unavailable during active tasks and side conversations, and opens the model picker in temporary mode.
2. `SlashCommand::ModelPersistent` is added, serialized as `modelp`, visible in slash popup/autocomplete, unavailable during active tasks and side conversations, and opens the same model picker in persistent mode.
3. Add a crate-local model picker persistence type with exact variants `Temporary` and `Persist`. Its meaning is:
   - `Temporary`: selection actions may emit `AppEvent::UpdateModel` and `AppEvent::UpdateReasoningEffort`; they must not emit `AppEvent::PersistModelSelection` or `AppEvent::PersistPlanModeReasoningEffort`.
   - `Persist`: selection actions emit the same runtime update events and also emit `AppEvent::PersistModelSelection` for selections that currently save defaults. Existing Plan-mode all-modes behavior may also persist the Plan-mode reasoning override.
4. `AppEvent::OpenReasoningPopup`, `AppEvent::OpenAllModelsPopup`, and `AppEvent::OpenPlanReasoningScopePrompt` carry the persistence mode so nested popups preserve whether they were launched by `/model` or `/modelp`.
5. `ChatWidget::open_model_popup()` keeps the temporary behavior for existing call sites. Add a persistent wrapper such as `ChatWidget::open_model_popup_persistent()` for `/modelp`. Internal helper names are flexible if all event behavior in this contract is preserved.
6. The all-models picker and reasoning picker are behaviorally identical in temporary and persistent modes except for persistence event emission.
7. In Plan mode, temporary `/model` must not open or select an option that writes config. If the existing scope prompt is shown for temporary mode, its actions must be temporary-only. If the implementation skips the scope prompt in temporary mode, it must still update active runtime model/reasoning correctly and emit no persistence events.
8. `tooltips.txt` must explain the distinction briefly: `/model` switches for the current session, `/modelp` saves the default.
9. Service-tier pseudo-commands in `codex-rs/tui/src/bottom_pane/slash_commands.rs` remain inserted immediately after `/model` when enabled. `/modelp` is a normal visible built-in command; with service tiers disabled it appears immediately after `/model` in built-in presentation order, and with service tiers enabled it appears after the inserted service-tier commands.
10. Testing uses existing TUI helper patterns in `codex-rs/tui/src/chatwidget/tests/*`, existing slash command tests in `codex-rs/tui/src/slash_command.rs`, bottom-pane command filtering tests, and existing insta snapshot workflow. Snapshot updates are generated artifacts and must be accepted only for intentional command-list UI changes.

## File Ownership

| File | Owner task | Change type | Responsibility | Parallel safety notes |
|---|---|---:|---|---|
| `docs/simplepower/plans/2026-06-22-modelp-persistent-model-command.md` | Coordinator planning | create | Authoritative approved implementation plan | Coordinator-owned only; workers must not edit |
| `codex-rs/tui/src/slash_command.rs` | Task 1 runtime implementation | modify | Add visible `/modelp` slash command metadata, parsing, availability, and focused unit assertions if colocated | Runtime worker only |
| `codex-rs/tui/src/app_event.rs` | Task 1 runtime implementation | modify | Add model picker persistence type and carry it on nested model popup app events | Runtime worker only |
| `codex-rs/tui/src/chatwidget/slash_dispatch.rs` | Task 1 runtime implementation | modify | Dispatch `/model` as temporary and `/modelp` as persistent; keep queue-drain/task-running behavior aligned | Runtime worker only |
| `codex-rs/tui/src/chatwidget/model_popups.rs` | Task 1 runtime implementation | modify | Thread persistence mode through model, all-models, reasoning, and Plan-mode scope popup actions | Runtime worker only |
| `codex-rs/tui/src/app/event_dispatch.rs` | Task 1 runtime implementation | modify | Pass persistence mode from app events back into chat widget popup handlers | Runtime worker only |
| `codex-rs/tui/tooltips.txt` | Task 1 runtime implementation | modify | Update tooltip wording for `/model` and `/modelp` | Runtime worker only |
| `codex-rs/tui/src/bottom_pane/slash_commands.rs` | Task 2 tests and snapshots | modify | Update or add tests for `/modelp` visibility and service-tier insertion ordering | Test worker only |
| `codex-rs/tui/src/bottom_pane/command_popup.rs` | Task 2 tests and snapshots | modify | Update exact command filtering expectations for `/m` and `/mo` after `/modelp` becomes visible | Test worker only |
| `codex-rs/tui/src/chatwidget/tests/popups_and_settings.rs` | Task 2 tests and snapshots | modify | Assert temporary picker paths do not persist and persistent picker paths do persist | Test worker only |
| `codex-rs/tui/src/chatwidget/tests/plan_mode.rs` | Task 2 tests and snapshots | modify | Assert Plan-mode temporary path emits no config persistence and persistent path keeps intended save behavior | Test worker only |
| `codex-rs/tui/src/chatwidget/tests/slash_commands.rs` | Task 2 tests and snapshots | modify | Assert `/modelp` is disabled/queued consistently with `/model` where relevant | Test worker only |
| `codex-rs/tui/src/bottom_pane/snapshots/codex_tui__bottom_pane__command_popup__tests__command_popup_default_items.snap` | Task 2 tests and snapshots | generated | Accept intentional visible command-list change for `/modelp` | Generated by test worker after review |
| `codex-rs/tui/src/bottom_pane/snapshots/codex_tui__bottom_pane__chat_composer__tests__slash_popup_mo.snap` | Task 2 tests and snapshots | generated | Accept intentional autocomplete/suggestion change if `/modelp` appears for `/mo` | Generated by test worker after review |

## Implementation Tasks

### Task 1: Runtime Slash Command And Picker Persistence

**Goal:** Add `/modelp` and make model picker selections respect temporary vs persistent mode across direct and nested popup flows.

**Contract inputs:** Interface Contract entries 1-10; approved design says `/model` no longer writes `config.toml`, `/modelp` duplicates `/model` and persists; repository Rust/TUI conventions from root `AGENTS.md`.

**Serialization required:** No. This task owns runtime files only and can proceed in parallel with Task 2 because tests target the Interface Contract.

**Write scope:** `codex-rs/tui/src/slash_command.rs`, `codex-rs/tui/src/app_event.rs`, `codex-rs/tui/src/chatwidget/slash_dispatch.rs`, `codex-rs/tui/src/chatwidget/model_popups.rs`, `codex-rs/tui/src/app/event_dispatch.rs`, `codex-rs/tui/tooltips.txt`.

**Parallel:** Yes, compatible with Task 2.

**Risk:** Medium. The behavior is localized but touches nested event routing and Plan-mode model selection semantics.

**Model tier:** BEST, resolved `model="gpt-5.5"`, `reasoning_effort="xhigh"`.

**Worker role:** `sp-impl`.

**Outputs and file-level responsibilities:**
- `SlashCommand::ModelPersistent` parses from `modelp`, is visible, and has a user-visible description that distinguishes saving defaults.
- `AppEvent` nested popup variants carry persistence mode without adding ambiguous positional booleans.
- `/model` dispatch uses temporary mode; `/modelp` dispatch uses persistent mode.
- Temporary selection paths do not emit config persistence events.
- Persistent selection paths preserve the current save behavior.

**Implementation steps:**
1. In `codex-rs/tui/src/app_event.rs`, add a crate-local enum, for example `ModelSelectionPersistence { Temporary, Persist }`, near the model popup event variants. Derive at least `Debug`, `Clone`, `Copy`, `PartialEq`, and `Eq`.
2. Update `OpenReasoningPopup`, `OpenPlanReasoningScopePrompt`, and `OpenAllModelsPopup` to include `persistence: ModelSelectionPersistence`.
3. In `codex-rs/tui/src/slash_command.rs`, add `ModelPersistent` immediately after `Model` in presentation order with `#[strum(serialize = "modelp")]`; add description, task-availability handling, and any exact parsing/availability unit assertions needed in the existing test module.
4. In `codex-rs/tui/src/chatwidget/slash_dispatch.rs`, dispatch `SlashCommand::Model` to temporary popup opening and `SlashCommand::ModelPersistent` to persistent popup opening. Include `ModelPersistent` in the same unavailable/queue-drain categories as `Model`.
5. In `codex-rs/tui/src/chatwidget/model_popups.rs`, keep `open_model_popup()` as the temporary public wrapper and add a persistent wrapper for `/modelp`. Refactor internal popup builders to accept `ModelSelectionPersistence` and pass it to all nested event actions.
6. In model selection actions, centralize selection application so `Temporary` emits only runtime update events and `Persist` emits runtime updates plus `PersistModelSelection`.
7. In Plan-mode scope handling, ensure temporary mode writes no config. Either avoid the scope prompt in temporary mode or include temporary-only actions. Persistent mode should preserve the existing Plan-only and all-modes persistence semantics.
8. In `codex-rs/tui/src/app/event_dispatch.rs`, destructure the new event fields and pass persistence into the corresponding chat widget methods.
9. Update `codex-rs/tui/tooltips.txt` to mention `/model` as current-session switching and `/modelp` as saving the default.

**Verification commands:**
- `cd codex-rs && timeout 120s just fmt` - expected: formatting completes.
- `cd codex-rs && timeout 900s just test -p codex-tui` - expected: runtime code compiles and focused TUI tests pass or fail only because Task 2 has not yet landed.

**Completion report requirements:** Report changed files, exact event semantics implemented for temporary and persistent modes, commands run with results, and any unresolved Plan-mode behavior risk.

### Task 2: Tests And Snapshot Updates

**Goal:** Add/adjust TUI tests and accepted snapshots so `/model` non-persistence and visible `/modelp` persistence are covered.

**Contract inputs:** Interface Contract entries 1-10; Task 1 will provide the approved `ModelSelectionPersistence` event field and `SlashCommand::ModelPersistent`; existing tests may dispatch commands or open popups directly using those names.

**Serialization required:** No. This task owns only test/snapshot files and may write tests against the Interface Contract while Task 1 edits runtime files.

**Write scope:** `codex-rs/tui/src/bottom_pane/slash_commands.rs`, `codex-rs/tui/src/bottom_pane/command_popup.rs`, `codex-rs/tui/src/chatwidget/tests/popups_and_settings.rs`, `codex-rs/tui/src/chatwidget/tests/plan_mode.rs`, `codex-rs/tui/src/chatwidget/tests/slash_commands.rs`, `codex-rs/tui/src/bottom_pane/snapshots/codex_tui__bottom_pane__command_popup__tests__command_popup_default_items.snap`, `codex-rs/tui/src/bottom_pane/snapshots/codex_tui__bottom_pane__chat_composer__tests__slash_popup_mo.snap`.

**Parallel:** Yes, compatible with Task 1.

**Risk:** Medium. Tests are localized but need to reflect behavior changes without weakening existing coverage.

**Model tier:** NORMAL, resolved `model="gpt-5.4-mini"`, `reasoning_effort="xhigh"`.

**Worker role:** `sp-impl`.

**Outputs and file-level responsibilities:**
- Event tests prove `/model` temporary selections emit no `PersistModelSelection`.
- Event tests prove `/modelp` persistent selections emit `PersistModelSelection`.
- Reasoning-level selection tests cover both temporary and persistent behavior.
- Plan-mode tests cover no config persistence for temporary mode and existing save behavior for persistent mode.
- Slash command tests cover `/modelp` visibility/dispatch availability and task-running disabled behavior consistent with `/model`.
- Bottom-pane tests cover exact `/m` filtering and service-tier insertion ordering after `/modelp` becomes visible.
- Command popup/autocomplete snapshots are updated and accepted only for intentional `/modelp` visibility changes.

**Implementation steps:**
1. Update existing expectations in `popups_and_settings.rs` that currently assume reasoning popup selections always persist; split or duplicate tests so temporary and persistent paths are explicit.
2. Add a persistent-mode model selection test that dispatches or opens `/modelp`, selects a model/reasoning option, and asserts the event stream includes `PersistModelSelection { model, effort }`.
3. Add a temporary-mode assertion that selecting via `/model` or `open_model_popup()` updates runtime events but all received events do not match `PersistModelSelection { .. }` or `PersistPlanModeReasoningEffort(_)`.
4. Update `plan_mode.rs` tests affected by new event fields and add coverage for temporary no-persist behavior in Plan mode.
5. Update `slash_commands.rs` to cover `/modelp` in the same command-disabled or queued-menu behavior as `/model`.
6. Update bottom-pane command tests so exact `/m` filtering includes `/modelp` in the intended order and service-tier pseudo-commands remain immediately after `/model` when enabled.
7. Run the focused snapshot-producing tests through `just test -p codex-tui`; inspect pending snapshots with `cargo insta pending-snapshots -p codex-tui`; accept only the command-list/autocomplete snapshots listed in File Ownership when their only meaningful change is adding `/modelp`.

**Verification commands:**
- `cd codex-rs && timeout 120s just fmt` - expected: formatting completes.
- `cd codex-rs && timeout 900s just test -p codex-tui` - expected: TUI tests pass after Task 1 is integrated.
- `cd codex-rs && timeout 120s cargo insta pending-snapshots -p codex-tui` - expected: no pending snapshots remain.

**Completion report requirements:** Report changed test and snapshot files, commands run with results, accepted snapshot names, and any coverage gaps.

## Model Allocation

| Stage | Role | Model tier | Resolved model | Reasoning effort | Reason |
|---|---|---:|---|---|---|
| Plan review | REVIEW-tier plan reviewer | REVIEW | `gpt-5.5` | `xhigh` | Required by writing-plans; environment override sets `SIMPLEPOWER_REVIEW_MODEL=gpt-5.5-xhigh` |
| Task 1 runtime implementation | `sp-impl` | BEST | `gpt-5.5` | `xhigh` | Behavior-shaping event routing with Plan-mode persistence semantics; environment override sets `SIMPLEPOWER_BEST_MODEL=gpt-5.5-xhigh` |
| Task 2 tests and snapshots | `sp-impl` | NORMAL | `gpt-5.4-mini` | `xhigh` | Routine localized TUI test and snapshot changes; environment override sets `SIMPLEPOWER_NORMAL_MODEL=gpt-5.4-mini-xhigh` |
| Quick verification | FAST-tier verifier | FAST | `gpt-5.3-codex-spark` | `high` | Required quick verifier tier; environment override sets `SIMPLEPOWER_FAST_MODEL=gpt-5.3-codex-spark-high` |
| Final review and fix | REVIEW-tier review+fix agent | REVIEW | `gpt-5.5` | `xhigh` | Required whole-change review/fix after quick verification; environment override sets `SIMPLEPOWER_REVIEW_MODEL=gpt-5.5-xhigh` |

## Plan Review

Self-review checklist status before reviewer dispatch:
- Design Summary captures the approved `/model` temporary and `/modelp` persistent design, constraints, success criteria, and the Plan-mode no-write nuance.
- Interface Contract is concrete and appears before File Ownership.
- File Ownership assigns every expected runtime, test, snapshot, tooltip, and plan file to exactly one owner; parallel workers do not share write scopes.
- Implementation tasks include Contract inputs and Serialization required fields.
- Aggregate parallel dispatch is planned for the runtime worker and test worker because the Interface Contract supplies the shared event/API shape.
- Visual aids are omitted because no diagram would reduce implementation ambiguity.
- Model allocation uses FAST/NORMAL/BEST/REVIEW and documents the environment-resolved model tiers.
- Review allocation includes one REVIEW-tier plan reviewer and one REVIEW-tier final review+fix agent.
- Commit policy has exactly three future coordinator checkpoints and forbids worker/reviewer commits.
- Scratch refs are coordinator-only local anchors under `refs/simplepower/scratch/<run-id>/` and do not alter checkpoint count.
- Verification commands are concrete and use `timeout`.
- Approved path enforcement does not authorize alternate routes, skipped checks, stubs, or docs-only substitutes.

Before first review, the coordinator creates `refs/simplepower/scratch/<run-id>/plan-review/before` for this saved plan file using the temporary-index pattern. If the reviewer reports issues, the coordinator edits this plan, reruns focused self-review for changed categories, creates `plan-review/after-<n>`, and sends the same reviewer this exact diff command, adjusted for the current run id and revision:

```bash
git diff refs/simplepower/scratch/<run-id>/plan-review/before refs/simplepower/scratch/<run-id>/plan-review/after-1 -- docs/simplepower/plans/2026-06-22-modelp-persistent-model-command.md
```

For later revisions, compare the previous `after-<n>` ref to the next `after-<n+1>` ref. If a needed scratch ref is missing, stop the review loop before relying on the missing anchor. The REVIEW-tier plan reviewer must perform the assigned review directly in the current worker. It must not run Codex CLI, spawn subagents, invoke Simple Power skills, restart execution, or reroute the workflow.

Every scratch `<run-id>` used by this plan must use the exact format `YYYYMMDD-HHMMSS-<short-head>`, for example `20260622-015947-865e8f4911`.

After the plan reviewer approves, the coordinator asks the user for one combined approval covering the reviewed plan, model/task allocation, and immediate current-session execution. The accepted plan checkpoint commit happens only after that combined approval, and workers/reviewers must not create it.

## Quick Verification

Before quick verification, the coordinator creates `refs/simplepower/scratch/<run-id>/quick-verifier/before` for all implementation files listed in File Ownership except the plan file. The quick verifier uses FAST (`model="gpt-5.3-codex-spark"`, `reasoning_effort="high"`) and may fix only tiny typo-level errors discovered while running quick checks. It must report behavior changes, structural edits, test rewrites, public interface changes, or unclear issues to the coordinator instead of fixing them.

Quick verification commands, run after all file-edit workers finish and before the quick-verified implementation checkpoint:

| Command | When | Expected result | Failure means |
|---|---|---|---|
| `cd codex-rs && timeout 120s just fmt` | First quick check after worker edits | Formatting completes | Formatting tooling failed or source has syntax that blocks rustfmt |
| `cd codex-rs && timeout 900s just test -p codex-tui` | After formatting | Focused TUI tests pass | Runtime/test behavior is not coherent enough for checkpoint |
| `cd codex-rs && timeout 120s cargo insta pending-snapshots -p codex-tui` | After focused tests | No pending snapshots remain | Snapshot updates were generated but not reviewed/accepted |

If the quick verifier makes typo-level fixes, the coordinator creates `refs/simplepower/scratch/<run-id>/quick-verifier/after` and inspects or hands off:

```bash
git diff refs/simplepower/scratch/<run-id>/quick-verifier/before refs/simplepower/scratch/<run-id>/quick-verifier/after -- codex-rs/tui/src/slash_command.rs codex-rs/tui/src/app_event.rs codex-rs/tui/src/chatwidget/slash_dispatch.rs codex-rs/tui/src/chatwidget/model_popups.rs codex-rs/tui/src/app/event_dispatch.rs codex-rs/tui/tooltips.txt codex-rs/tui/src/bottom_pane/slash_commands.rs codex-rs/tui/src/bottom_pane/command_popup.rs codex-rs/tui/src/chatwidget/tests/popups_and_settings.rs codex-rs/tui/src/chatwidget/tests/plan_mode.rs codex-rs/tui/src/chatwidget/tests/slash_commands.rs codex-rs/tui/src/bottom_pane/snapshots/codex_tui__bottom_pane__command_popup__tests__command_popup_default_items.snap codex-rs/tui/src/bottom_pane/snapshots/codex_tui__bottom_pane__chat_composer__tests__slash_popup_mo.snap
```

After the quick-verified implementation checkpoint succeeds, delete that run's `quick-verifier` scratch refs. If the checkpoint fails or the workflow stops before the checkpoint, preserve refs and report the manual cleanup command.

## Final Review And Fix

After the quick-verified implementation checkpoint, dispatch exactly one REVIEW-tier review+fix agent using `model="gpt-5.5"` and `reasoning_effort="xhigh"`. Before dispatch, the coordinator creates `refs/simplepower/scratch/<run-id>/review-fix/before` for all implementation files listed in File Ownership except the plan file.

The review+fix agent reviews and fixes the whole implementation against this accepted plan, file ownership, approved path enforcement, aggregate parallel dispatch semantics, event behavior, Plan-mode persistence behavior, and verification requirements. It may edit files within the approved File Ownership list when fixing issues it finds. It must report changed files, commands run, results, remaining risks, and unresolved deviations requiring user approval. It must not commit, run Codex CLI, spawn subagents, invoke Simple Power skills, restart execution, or reroute the workflow.

If the review+fix agent edits files, the coordinator creates `refs/simplepower/scratch/<run-id>/review-fix/after` and inspects or hands off:

```bash
git diff refs/simplepower/scratch/<run-id>/review-fix/before refs/simplepower/scratch/<run-id>/review-fix/after -- codex-rs/tui/src/slash_command.rs codex-rs/tui/src/app_event.rs codex-rs/tui/src/chatwidget/slash_dispatch.rs codex-rs/tui/src/chatwidget/model_popups.rs codex-rs/tui/src/app/event_dispatch.rs codex-rs/tui/tooltips.txt codex-rs/tui/src/bottom_pane/slash_commands.rs codex-rs/tui/src/bottom_pane/command_popup.rs codex-rs/tui/src/chatwidget/tests/popups_and_settings.rs codex-rs/tui/src/chatwidget/tests/plan_mode.rs codex-rs/tui/src/chatwidget/tests/slash_commands.rs codex-rs/tui/src/bottom_pane/snapshots/codex_tui__bottom_pane__command_popup__tests__command_popup_default_items.snap codex-rs/tui/src/bottom_pane/snapshots/codex_tui__bottom_pane__chat_composer__tests__slash_popup_mo.snap
```

After the final checkpoint succeeds, delete that run's `review-fix` scratch refs. If the checkpoint fails or the workflow stops before the checkpoint, preserve refs and report the manual cleanup command.

## Commit Checkpoints

1. Accepted plan checkpoint: after the plan reviewer approves and the user gives combined approval for the reviewed plan, model/task allocation, and immediate current-session execution; before invoking `simplepower:subagent-driven-development`.
2. Quick-verified implementation checkpoint: after all `sp-impl` file edits complete and quick verification passes.
3. Final checkpoint: after the REVIEW-tier review+fix agent completes and final verification passes.

Workers, plan reviewers, quick verifiers, review+fix agents, and individual tasks must not commit. Scratch refs are the only temporary review anchors. They are coordinator-owned, local-only, and not accepted checkpoint commits. Delete phase scratch refs after the successful checkpoint for their phase, or preserve and report manual cleanup if the workflow stops or the checkpoint commit fails.

## Current-Session Auto-Dispatch

The saved plan is the execution artifact. Do not write a project-local implementation JSON artifact. Do not run routing heuristics or offer alternate execution routes.

After the plan reviewer approves, ask the user for one combined approval covering:
- The reviewed plan.
- The model/task allocation.
- Immediate current-session execution.

If the user requests changes, update this plan, rerun focused self-review checks for changed categories, create the next `plan-review/after-<n>` scratch ref, and send the revised plan back to the same reviewer with the concrete scratch-ref diff command. Do not create the accepted plan checkpoint until the user gives combined approval.

After combined approval, the coordinator creates the accepted plan checkpoint commit, deletes the successful `plan-review` scratch refs, then immediately invokes `simplepower:subagent-driven-development` in the current session with this instruction:

```text
Execute `docs/simplepower/plans/2026-06-22-modelp-persistent-model-command.md` with aggregate parallel implementation from the approved Interface Contract. Use the approved FAST/NORMAL/BEST/REVIEW model allocation. Dispatch all non-conflicting `sp-impl` file-edit workers whose coordination needs are satisfied by their Contract inputs, run the quick FAST-tier verifier with lint/build/tests and timeouts after all workers finish, commit the quick-verified implementation, then run one REVIEW-tier review+fix agent, final verification, and final commit.
```

## Verification

Final verification runs after the REVIEW-tier review+fix agent completes and before the final checkpoint:

| Command | When | Expected result | Failure means |
|---|---|---|---|
| `cd codex-rs && timeout 120s just fmt` | First final check after review+fix | Formatting completes | Source formatting or syntax is invalid |
| `cd codex-rs && timeout 900s just test -p codex-tui` | After final formatting | Focused TUI tests pass | Implementation does not satisfy runtime/test expectations |
| `cd codex-rs && timeout 120s cargo insta pending-snapshots -p codex-tui` | After focused tests | No pending snapshots remain | Snapshot changes remain unreviewed or unaccepted |
| `cd codex-rs && timeout 900s just fix -p codex-tui` | Last local lint/fix check before final report | Clippy/fix pass completes; do not rerun tests afterward per repo guidance | Linter found unresolved issues or fix command failed |

The coordinator performs the final checkpoint only after the REVIEW-tier review+fix agent has completed and the final commands pass. Final reporting must include this cleanup check for the run id:

```bash
git for-each-ref --format='%(refname)' "refs/simplepower/scratch/<run-id>"
```

If the final checkpoint succeeds, no scratch refs for that run should remain after phase cleanup. If the workflow stops because of user direction, a blocker, or a failed checkpoint commit, preserve remaining scratch refs and report:

```bash
git for-each-ref --format='%(refname)' "refs/simplepower/scratch/<run-id>" | while read -r ref; do git update-ref -d "$ref"; done
```
