# Yolo Default And Compact Timer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `simplepower:subagent-driven-development` for aggregate parallel implementation. Dispatch all non-conflicting `sp-impl` file-edit workers whose coordination needs are satisfied by the approved Interface Contract, run the quick non-test verifier after all workers finish, commit the quick-verified implementation, then run one REVIEW-tier review+fix agent before final non-test verification and final commit. After the final feature checkpoint succeeds, the coordinator must invoke `$mod-refresh-full-release` as a separate post-final release workflow with a fresh current-session preflight.

**Branch:** `feat/yolo-default-compact-timer`

**Goal:** Add a top-level config flag that trusts unknown projects and defaults them to full-access/yolo permissions, add a compact service-tier status when compact is already on fast/priority, add a TUI-only compact timer that shows compact start time, live elapsed time, finish time, and duration, then run the full mod refresh release flow.

**Design Summary:** The approved design uses one explicit dangerous top-level config flag, `dangerously_trust_all_projects = true`. When enabled, unknown projects are treated as trusted without writing `[projects]` entries, and Codex defaults to full access only when permission settings are otherwise unset. Explicit user or managed configuration such as `[projects]`, `approval_policy`, `sandbox_mode`, `default_permissions`, permission-profile overrides, and requirements constraints remain higher priority. Compact timing is TUI-only: use existing app-server `ItemStarted.started_at_ms` and `ItemCompleted.completed_at_ms` payloads for `ContextCompaction` items to render one active compact row that refreshes elapsed time once per second, then a completed row with start, finish, and duration. Preserve the existing compact-only fast/priority service-tier override; add one explicit status when compact does not switch tiers because the original service tier is already fast/priority. After feature implementation is committed and reviewed, invoke `$mod-refresh-full-release`; that release workflow must first run a fresh `$mod-refresh-preflight`, carry forward the no-tests/no-Bazel release policy, and stop if preflight or release gates are blocked.

**User Override:** The user explicitly said “no need to test.” This plan does not authorize `just test`, `cargo test`, `cargo insta`, or snapshot update/acceptance commands. Final reporting must state that tests and snapshots were skipped by request. Required non-test maintenance still applies where relevant: `just fmt`, `just fix -p ...`, `git diff --check`, and `just write-config-schema`.

**Architecture:** Config behavior stays in the existing config loading and permission resolution path, with `ConfigToml` carrying the new flag and `ConfigBuilder` applying it while resolving `active_project`, implicit built-in permissions, and default approval policy. The compact service-tier status is a narrow change in the existing remote-first compact wrapper and must not alter compact request routing or service-tier selection. The compact timer stays in the TUI history-cell and ChatWidget lifecycle layer, using existing item lifecycle notifications so core compact execution and protocol contracts are not widened. Release execution is deliberately outside implementation worker ownership: after the final feature checkpoint, `$mod-refresh-full-release` owns its fresh preflight, release-run plan, merge preservation, Cargo release build, tagging, and publishing gates.

**Tech Stack:** Rust, `serde`, `schemars`, `chrono`, ratatui history cells, Codex app-server item notifications, `just fmt`, `just fix`, `just write-config-schema`, `git diff --check`, `$mod-refresh-full-release`.

**Model Allocation:** FAST/NORMAL/BEST/REVIEW tiers are assigned below. Resolve each tier by explicit user override, quoted assignment in project root AGENTS.md, process environment variable, then built-in default. The project root AGENTS.md lookup reads only `<repo>/AGENTS.md`, not nested AGENTS.md files or repo-wide grep. FAST defaults to `SIMPLEPOWER_FAST_MODEL` (`gpt-5.3-codex-spark-high` when unset), NORMAL defaults to `SIMPLEPOWER_NORMAL_MODEL` (`gpt-5.4-mini-high` when unset), BEST defaults to `SIMPLEPOWER_BEST_MODEL` (`gpt-5.5-high` when unset), and REVIEW defaults to `SIMPLEPOWER_REVIEW_MODEL` (`gpt-5.5-xhigh` when unset). For this run, process environment resolves FAST to `model="gpt-5.3-codex-spark"`, `reasoning_effort="high"`; NORMAL to `model="gpt-5.4-mini"`, `reasoning_effort="xhigh"`; BEST to `model="gpt-5.5"`, `reasoning_effort="xhigh"`; REVIEW to `model="gpt-5.5"`, `reasoning_effort="xhigh"`. The plan reviewer is a REVIEW-tier plan reviewer, and the final review+fix agent is a REVIEW-tier review+fix agent. The quick verifier uses the FAST tier by default.

**Commit Policy:** The coordinator commits after the reviewed plan, allocation, and immediate current-session execution receive combined approval, after all file edits and quick non-test verification complete before final review, and after final review/fix plus final non-test verification. These are exactly three Simple Power feature checkpoints. Workers, plan reviewers, quick verifiers, and review+fix agents must not commit. No per-task commits. Coordinator-owned temporary scratch refs under `refs/simplepower/scratch/<run-id>/...` may be created only as local review diff anchors; they are not accepted history commits, not pushed, not merged, not rebased, and must be cleaned up after successful checkpoints or reported for manual cleanup on blockers or failed checkpoints. After the final feature checkpoint and scratch cleanup, `$mod-refresh-full-release` is a separate release workflow with its own skill-defined plan, merge/build/publish gates, and release commits/tags; those are not additional Simple Power feature checkpoints.

---

## Interface Contract

1. `ConfigToml::dangerously_trust_all_projects` is a new top-level user config field loaded from `~/.codex/config.toml` as `dangerously_trust_all_projects = true`. Omitted or `false` means current upstream behavior.
2. When `dangerously_trust_all_projects` is enabled and no `[projects]` entry matches the cwd or resolved git root, `Config::load` resolves `active_project` as `ProjectConfig { trust_level: Some(TrustLevel::Trusted) }` without writing anything to `config.toml`.
3. Explicit project entries remain authoritative. If `[projects."<path>"].trust_level = "untrusted"` matches the cwd or repo root, the project remains untrusted even when `dangerously_trust_all_projects = true`.
4. The flag supplies full-access defaults only when no explicit permission surface is set. The implicit full-access condition is true only when all of these are true: the flag is enabled, no `permission_profile` override was supplied, `resolve_permission_config_syntax(...)` returns `None`, `effective_permission_selection.selected_profile_id` is `None`, and `effective_permission_selection.requirements_force_profile_selection` is `false`.
5. Under the implicit full-access condition, the permission resolver selects the built-in `:danger-full-access` profile (`BUILT_IN_DANGER_FULL_ACCESS_PROFILE`) instead of the normal trusted-project default, and the approval resolver defaults to `AskForApproval::Never`.
6. Existing constraints still apply. If requirements reject `AskForApproval::Never`, the existing constrained fallback path chooses the required approval default. If requirements force permission-profile selection, the new flag does not override that forced selection.
7. Explicit `approval_policy`, `sandbox_mode`, `default_permissions`, CLI/session overrides, or a supplied `permission_profile` override keep their existing priority and prevent the new flag from forcing full-access defaults.
8. The existing compact-only fast/priority service-tier override remains the source of truth. Do not edit `codex-rs/core/src/compact_service_tier.rs`, `codex-rs/core/src/compact_remote.rs`, `codex-rs/core/src/compact_remote_v2.rs`, `codex-rs/core/src/compact.rs`, or `codex-rs/core/src/tasks/compact.rs` for service-tier routing. The existing behavior resolves `priority` for compact work when supported, uses it for remote compact and local fallback compact requests, and leaves later normal sampling on the original `turn_context.config.service_tier`.
9. In `codex-rs/core/src/remote_compact_fallback.rs`, preserve the existing start/finish warning messages when `emit_service_tier_status` is true, meaning compact temporarily switches from the original tier to `priority`.
10. In `codex-rs/core/src/remote_compact_fallback.rs`, add an already-fast status when the resolved compact tier is `ServiceTier::Fast.request_value()` and `turn_context.config.service_tier.as_deref() == Some(ServiceTier::Fast.request_value())`. Emit exactly one warning before the first remote compact attempt:
    `Compact operations are already using fast service tier (priority); no service tier change needed.`
    Do not emit the existing “finished; normal requests are using ... again” service-tier restoration message for this already-fast case because no tier switch occurred.
11. The already-fast status is emitted as `EventMsg::Warning(WarningEvent { message })`, matching the existing compact service-tier status surface, so the TUI renders it as a colored warning cell. It must not be recorded into model-visible conversation history.
12. TUI compact timing uses existing app-server notifications:
    - `ItemStartedNotification { item: ThreadItem::ContextCompaction { id }, started_at_ms, ... }`
    - `ItemCompletedNotification { item: ThreadItem::ContextCompaction { id }, completed_at_ms, ... }`
13. The compact timer must run for every live `ContextCompaction` lifecycle, including when service tier changes, when no service tier change occurs because the original tier is already fast/priority, and when no service-tier status is emitted because `priority` is unsupported.
14. The compact active row text is a single active history cell, not repeated transcript messages. Required live text shape: `Compacting context · started HH:MM:SS · elapsed <duration>`.
15. The completed compact row text is one committed history cell. Required completed text shape when both times are known: `Context compacted · started HH:MM:SS · finished HH:MM:SS · took <duration>`. If live completion lacks a start time, render `Context compacted · finished HH:MM:SS`. Replay or legacy data with no timestamps may keep `Context compacted`.
16. `HH:MM:SS` uses local time via the TUI crate's existing `chrono` dependency. Durations use the existing `crate::status_indicator_widget::fmt_elapsed_compact` helper.
17. The active compact row refreshes no more frequently than once per second while running. The transcript overlay must also see a changing `transcript_animation_tick()` while the compact row is active.
18. `EventMsg::ContextCompacted(ContextCompactedEvent {})` remains unchanged. No core compact execution, rollout format, or protocol item lifecycle contract change is required by the TUI timer.
19. Non-test command contract:
    - Formatting: `cd codex-rs && timeout 120s just fmt`
    - Schema generation after `ConfigToml` changes: `cd codex-rs && timeout 120s just write-config-schema`
    - Typecheck/build without tests: `cd codex-rs && timeout 600s cargo check -p codex-core -p codex-tui`
    - Scoped fix/lint before finalization: `cd codex-rs && timeout 300s just fix -p codex-core` and `cd codex-rs && timeout 300s just fix -p codex-tui`
    - Whitespace check: `timeout 30s git diff --check -- <approved-files>`
    - Tests and snapshots: intentionally skipped by explicit user request.
20. Post-final release contract:
    - `$mod-refresh-full-release` runs only after the final feature checkpoint commit succeeds and the feature worktree is clean.
    - `$mod-refresh-full-release` must run a fresh current-session `$mod-refresh-preflight` first and must not mutate the worktree, merge, build, tag, publish, or invoke `$mod-refresh-release` if preflight is blocked or reports unresolved blockers.
    - The release handoff must record `Tests: not run unless explicitly requested` and `Bazel: not used; using Cargo release build only`.
    - The release handoff must record the preflight `upstream target SHA` from `git rev-parse upstream/main`, expected `upstreamhash.txt` as that full SHA, expected `modversion.txt` as `1` unless explicitly approved otherwise, expected artifact path `codex`, current branch, release target, and compact-fix preservation risks.
    - Stop and ask for direction if preflight, merge preservation, build, metadata validation, artifact naming, release tag, release notes source, publish destination, or existing release state is ambiguous or blocked.

## File Ownership

| File | Owner task | Change type | Responsibility | Parallel safety notes |
|---|---|---:|---|---|
| `docs/simplepower/plans/2026-06-18-yolo-default-and-compact-timer.md` | Coordinator planning | create | Authoritative Simple Power implementation plan. | Coordinator-owned; implementation workers must not edit. |
| `codex-rs/config/src/config_toml.rs` | Config defaults task | modify | Add the `dangerously_trust_all_projects` `ConfigToml` field with serde/schema comments. | No overlap with other parallel file-edit tasks. |
| `codex-rs/core/src/config/mod.rs` | Config defaults task | modify | Apply trust-all-projects fallback, implicit `:danger-full-access` selection, and implicit `AskForApproval::Never` default. | No overlap with other parallel file-edit tasks. |
| `codex-rs/core/src/remote_compact_fallback.rs` | Compact service-tier status task | modify | Add the already-fast compact status message without changing compact routing or request construction. | No overlap with config or TUI timer tasks. |
| `codex-rs/tui/src/history_cell/compact.rs` | TUI compact timer task | create | Implement active/completed compact history cell, local-time formatting, duration formatting, and animation tick. | New file; no overlap. |
| `codex-rs/tui/src/history_cell/mod.rs` | TUI compact timer task | modify | Register/export the new compact history cell module and constructors. | Owned only by TUI compact timer task. |
| `codex-rs/tui/src/chatwidget/protocol.rs` | TUI compact timer task | modify | Route `ContextCompaction` item started/completed notifications with lifecycle timestamps. | Owned only by TUI compact timer task. |
| `codex-rs/tui/src/chatwidget/tool_lifecycle.rs` | TUI compact timer task | modify | Add context-compaction start/completion handlers and active-cell completion logic. | Owned only by TUI compact timer task. |
| `codex-rs/tui/src/chatwidget/replay.rs` | TUI compact timer task | modify | Preserve replay behavior with completed compact rendering and no duplicate live compact row. | Owned only by TUI compact timer task. |
| `codex-rs/tui/src/thread_transcript.rs` | TUI compact timer task | modify | Keep transcript fallback text coherent for `ContextCompaction` items. | Owned only by TUI compact timer task. |
| `codex-rs/tui/src/app/agent_status_feed.rs` | TUI compact timer task | modify | Keep status-feed compact summary coherent if lifecycle items surface there. | Owned only by TUI compact timer task. |
| `codex-rs/core/config.schema.json` | Schema task | generated | Regenerate schema after the new `ConfigToml` field. | Serialized after Config defaults task because the field must exist before generation. |
| `docs/mod-refresh/plans/YYYY-MM-DD-<short-topic>.md` | `$mod-refresh-full-release` release workflow | create | Release-run plan created by the release skill after final feature checkpoint and fresh preflight. | Outside implementation worker ownership; governed by release skill gates. |
| `upstreamhash.txt` | `$mod-refresh-full-release` release workflow | modify | Store the preflight upstream target SHA as one full 40-character lowercase hex SHA line. | Outside implementation worker ownership; governed by release skill gates. |
| `modversion.txt` | `$mod-refresh-full-release` release workflow | modify | Store release mod version, expected `1` unless explicitly approved otherwise. | Outside implementation worker ownership; governed by release skill gates. |
| `codex` | `$mod-refresh-full-release` release workflow | generated | Repository-root Linux CLI artifact built by the release workflow. | Outside implementation worker ownership; governed by release skill gates. |

## Implementation Tasks

### Task 1: Config defaults

- **Goal:** Add `dangerously_trust_all_projects` and make it trust unknown projects plus select full-access/yolo defaults only when permission config is otherwise unset.
- **Contract inputs:** Interface Contract entries 1 through 7 and 19.
- **Serialization required:** No. The Interface Contract defines the config behavior, and the task has no write overlap with other parallel implementation tasks.
- **Write scope:** `codex-rs/config/src/config_toml.rs`, `codex-rs/core/src/config/mod.rs`
- **Parallel:** Yes, compatible with Compact service-tier status task and TUI compact timer task.
- **Risk:** High, because this changes security-sensitive defaults and must preserve explicit policy precedence.
- **Model tier:** BEST, resolved model `gpt-5.5`, reasoning effort `xhigh`.
- **Worker role:** `sp-impl`
- **Outputs and file-level responsibilities:** New top-level config field; active project fallback; implicit built-in `:danger-full-access` default selection; implicit approval default of `AskForApproval::Never`.
- **Implementation steps:**
  1. In `codex-rs/config/src/config_toml.rs`, add `pub dangerously_trust_all_projects: Option<bool>` near existing trust/permission-related top-level fields, with a doc comment that warns it treats unknown projects as trusted and defaults to full access when permissions are unset.
  2. In `codex-rs/core/src/config/mod.rs`, compute `let dangerously_trust_all_projects = cfg.dangerously_trust_all_projects.unwrap_or(false);` during config load after the top-level TOML is available.
  3. Replace the unmatched-project fallback so it becomes `ProjectConfig { trust_level: Some(TrustLevel::Trusted) }` only when `dangerously_trust_all_projects` is true; otherwise keep `trust_level: None`.
  4. After `effective_permission_selection` and `permission_config_syntax` are known, compute `implicit_danger_full_access` using the exact Interface Contract condition.
  5. In the profile-active branch, when no profile was explicitly selected, choose `BUILT_IN_DANGER_FULL_ACCESS_PROFILE` if `implicit_danger_full_access` is true; otherwise keep the existing default built-in profile selection.
  6. In default approval resolution, choose `AskForApproval::Never` if `implicit_danger_full_access` is true; otherwise keep existing trusted/untrusted/default behavior.
  7. Preserve constrained approval fallback and requirements behavior; do not bypass managed constraints.
- **Verification commands:** `timeout 30s git diff --check -- codex-rs/config/src/config_toml.rs codex-rs/core/src/config/mod.rs`
- **Completion report requirements:** Changed files, exact defaulting logic used, whitespace-check result, and any requirements/constraint behavior that could not be verified locally because tests were skipped by request.

### Task 2: Compact service-tier already-fast status

- **Goal:** Show a colored compact status when compact is already on fast/priority, without changing compact service-tier routing.
- **Contract inputs:** Interface Contract entries 8 through 11 and 19; existing changelog contract in `docs/compact-fix/ChangeLog.md` section “Fast service tier override for compaction only”.
- **Serialization required:** No. The task writes only `codex-rs/core/src/remote_compact_fallback.rs` and relies on existing compact-tier resolver behavior.
- **Write scope:** `codex-rs/core/src/remote_compact_fallback.rs`
- **Parallel:** Yes, compatible with Config defaults task and TUI compact timer task.
- **Risk:** Medium, because it changes user-visible compact warnings but not request routing.
- **Model tier:** NORMAL, resolved model `gpt-5.4-mini`, reasoning effort `xhigh`.
- **Worker role:** `sp-impl`
- **Outputs and file-level responsibilities:** One new warning branch for the already-fast case; unchanged compact request service-tier selection and unchanged restoration messages for actual tier switches.
- **Implementation steps:**
  1. In `run_remote_first_compact`, keep the existing `emit_service_tier_status` switch-detection branch intact.
  2. Add a separate boolean for the already-fast case: the resolved `remote_service_tier_override` is `Some(ServiceTier::Fast.request_value())` and `turn_context.config.service_tier.as_deref() == Some(ServiceTier::Fast.request_value())`.
  3. Before the first remote attempt, if already-fast is true and `emit_service_tier_status` is false, call `emit_compact_service_tier_status` with exactly `Compact operations are already using fast service tier (priority); no service tier change needed.`
  4. Do not emit the existing finished/restored message for the already-fast branch.
  5. Do not edit `compact_service_tier.rs`, `compact_remote.rs`, `compact_remote_v2.rs`, `compact.rs`, or `tasks/compact.rs`.
- **Verification commands:** `timeout 30s git diff --check -- codex-rs/core/src/remote_compact_fallback.rs`
- **Completion report requirements:** Changed file, exact message added, confirmation that routing files were not edited, whitespace-check result, and note that tests were skipped by request.

### Task 3: TUI compact timer

- **Goal:** Render active and completed compact lifecycle rows with start time, live elapsed time, finish time, and duration for every live compact lifecycle, including the already-fast service-tier case.
- **Contract inputs:** Interface Contract entries 12 through 19.
- **Serialization required:** No. The task writes only TUI files and uses existing app-server lifecycle notification fields.
- **Write scope:** `codex-rs/tui/src/history_cell/compact.rs`, `codex-rs/tui/src/history_cell/mod.rs`, `codex-rs/tui/src/chatwidget/protocol.rs`, `codex-rs/tui/src/chatwidget/tool_lifecycle.rs`, `codex-rs/tui/src/chatwidget/replay.rs`, `codex-rs/tui/src/thread_transcript.rs`, `codex-rs/tui/src/app/agent_status_feed.rs`
- **Parallel:** Yes, compatible with Config defaults task and Compact service-tier status task.
- **Risk:** High, because it changes user-visible TUI lifecycle rendering and must avoid duplicate compact messages.
- **Model tier:** BEST, resolved model `gpt-5.5`, reasoning effort `xhigh`.
- **Worker role:** `sp-impl`
- **Outputs and file-level responsibilities:** New compact history cell and routing of context-compaction item lifecycle notifications through active/completed rendering.
- **Implementation steps:**
  1. Create `codex-rs/tui/src/history_cell/compact.rs` with a `CompactHistoryCell` that stores `id: String`, `started_at_ms: Option<i64>`, `completed_at_ms: Option<i64>`, and a local `Instant` for active elapsed fallback.
  2. Format timestamps with `chrono::Local.timestamp_millis_opt(ms).single().map(|dt| dt.format("%H:%M:%S").to_string())`; invalid timestamps omit that part rather than panicking.
  3. Format active elapsed and completed duration with `crate::status_indicator_widget::fmt_elapsed_compact`.
  4. Implement `HistoryCell` so active display/raw lines follow `Compacting context · started HH:MM:SS · elapsed <duration>`, completed display/raw lines follow the Interface Contract, and replay/legacy missing timestamp cases degrade to `Context compacted`.
  5. Implement `transcript_animation_tick()` for active cells using elapsed seconds and return `None` for completed cells.
  6. Export constructors from `history_cell/mod.rs`, for example `new_active_context_compaction(id, started_at_ms)` and `new_completed_context_compaction(id, started_at_ms, completed_at_ms)`.
  7. In `chatwidget/protocol.rs`, route `ItemStartedNotification` with `ThreadItem::ContextCompaction { id }` to a new start handler, passing `started_at_ms`. Route `ItemCompletedNotification` with `ThreadItem::ContextCompaction { id }` to a new completion handler, passing `completed_at_ms`, instead of only falling through to generic completed replay handling.
  8. In `chatwidget/tool_lifecycle.rs`, implement start behavior by flushing streamed answer/active state as appropriate, setting the compact active cell, marking visible turn activity, bumping active-cell revision, and requesting redraw.
  9. Implement completion behavior by completing the matching active compact cell if present; if absent, add a completed compact cell with unknown start time. Do not also add the old generic `Context compacted` message for the same live lifecycle.
  10. Keep replay behavior in `chatwidget/replay.rs` compatible: replayed `ContextCompaction` without lifecycle timestamps may add the legacy completed compact cell.
  11. Update `thread_transcript.rs` and `app/agent_status_feed.rs` only if necessary for wording consistency; do not change model-visible history.
- **Verification commands:** `timeout 30s git diff --check -- codex-rs/tui/src/history_cell/compact.rs codex-rs/tui/src/history_cell/mod.rs codex-rs/tui/src/chatwidget/protocol.rs codex-rs/tui/src/chatwidget/tool_lifecycle.rs codex-rs/tui/src/chatwidget/replay.rs codex-rs/tui/src/thread_transcript.rs codex-rs/tui/src/app/agent_status_feed.rs`
- **Completion report requirements:** Changed files, active/completed rendering behavior, duplicate-message prevention notes, whitespace-check result, and note that tests/snapshots were skipped by request.

### Task 4: Schema generation

- **Goal:** Regenerate `codex-rs/core/config.schema.json` for the new config field.
- **Contract inputs:** Interface Contract entries 1 and 19.
- **Serialization required:** Yes. The generated schema requires the config field from Task 1 to exist before `just write-config-schema` can succeed.
- **Write scope:** `codex-rs/core/config.schema.json`
- **Parallel:** No, must run after Config defaults task.
- **Risk:** Low, because it is generated output.
- **Model tier:** FAST, resolved model `gpt-5.3-codex-spark`, reasoning effort `high`.
- **Worker role:** `sp-impl`
- **Outputs and file-level responsibilities:** Updated JSON schema containing `dangerously_trust_all_projects`.
- **Implementation steps:**
  1. Run `cd codex-rs && timeout 120s just write-config-schema`.
  2. Confirm `codex-rs/core/config.schema.json` contains `dangerously_trust_all_projects`.
  3. Do not hand-edit generated schema except to resolve generation failure under coordinator direction.
- **Verification commands:** `cd codex-rs && timeout 120s just write-config-schema`
- **Completion report requirements:** Command run, schema update confirmation, and any generator failure details.

## Model Allocation

| Stage | Role | Model tier | Resolved model | Reasoning effort | Reason |
|---|---|---:|---|---|---|
| Plan review | REVIEW-tier plan reviewer | REVIEW | `gpt-5.5` | `xhigh` | Required by `simplepower:writing-plans`; checks security-sensitive and UI-visible plan completeness. |
| Implementation | Config defaults task | BEST | `gpt-5.5` | `xhigh` | Security-sensitive config and permission default behavior. |
| Implementation | Compact service-tier status task | NORMAL | `gpt-5.4-mini` | `xhigh` | Localized user-visible warning branch with no routing changes. |
| Implementation | TUI compact timer task | BEST | `gpt-5.5` | `xhigh` | User-visible lifecycle rendering with active timer behavior and duplicate-row risk. |
| Implementation | Schema generation task | FAST | `gpt-5.3-codex-spark` | `high` | Mechanical generated artifact after config field lands. |
| Quick verification | FAST-tier non-test verifier | FAST | `gpt-5.3-codex-spark` | `high` | Runs formatting, schema, fix, and whitespace checks only. |
| Final review/fix | REVIEW-tier review+fix agent | REVIEW | `gpt-5.5` | `xhigh` | Required whole-implementation review and fixes before final non-test verification. |
| Post-final release | `$mod-refresh-full-release` coordinator workflow | REVIEW | `gpt-5.5` | `xhigh` | Runs fresh preflight, merge preservation, Cargo release build, tag, and publish after final feature checkpoint. Release-skill subagents use same model as main agent with `reasoning_effort="high"` per skill policy. |

## Plan Review

- Coordinator self-review checks the Design Summary, Interface Contract, File Ownership, task allocation, aggregate parallel readiness, model allocation, review allocation, commit policy, scratch refs, user test-skip override, and approved path enforcement before dispatching the plan reviewer.
- Scratch run id for this planning session uses the `YYYYMMDD-HHMMSS-<short-head>` shape. The current candidate run id is `20260618-083432-1fd87c2466`.
- Before first review, the coordinator creates `refs/simplepower/scratch/<run-id>/plan-review/before` for this plan file using the temporary-index scratch-ref pattern.
- Dispatch one REVIEW-tier plan reviewer using `skills/writing-plans/plan-document-reviewer-prompt.md`, with this saved plan path, approved design context, scratch run id, and `plan-review/before` ref.
- If the reviewer reports blocking issues, the coordinator edits this plan, reruns focused self-review for changed categories, creates `plan-review/after-<n>`, and sends the same reviewer the concrete diff command:
  - First revision: `git diff refs/simplepower/scratch/<run-id>/plan-review/before refs/simplepower/scratch/<run-id>/plan-review/after-1 -- docs/simplepower/plans/2026-06-18-yolo-default-and-compact-timer.md`
  - Later revisions compare the last `after-<n>` ref to the new `after-<n+1>` ref.
- The plan reviewer performs the review directly in the current worker. It must not run Codex CLI, spawn subagents, invoke Simple Power skills, restart execution, or reroute the workflow.

## Quick Verification

- Run after all file-edit workers finish and before the quick-verified implementation checkpoint.
- Before dispatching the quick verifier, the coordinator creates `refs/simplepower/scratch/<run-id>/quick-verifier/before` for all approved implementation files.
- The quick verifier uses FAST tier by default: `model="gpt-5.3-codex-spark"`, `reasoning_effort="high"`.
- The quick verifier may fix only tiny typo-level issues discovered while running commands. It must report behavior changes, structural edits, test rewrites, public interface changes, or unclear issues instead of fixing them.
- Quick non-test commands:
  - `cd codex-rs && timeout 120s just fmt`
  - `cd codex-rs && timeout 120s just write-config-schema`
  - `cd codex-rs && timeout 600s cargo check -p codex-core -p codex-tui`
  - `cd codex-rs && timeout 300s just fix -p codex-core`
  - `cd codex-rs && timeout 300s just fix -p codex-tui`
  - `timeout 30s git diff --check -- codex-rs/config/src/config_toml.rs codex-rs/core/src/config/mod.rs codex-rs/core/src/remote_compact_fallback.rs codex-rs/tui/src/history_cell/compact.rs codex-rs/tui/src/history_cell/mod.rs codex-rs/tui/src/chatwidget/protocol.rs codex-rs/tui/src/chatwidget/tool_lifecycle.rs codex-rs/tui/src/chatwidget/replay.rs codex-rs/tui/src/thread_transcript.rs codex-rs/tui/src/app/agent_status_feed.rs codex-rs/core/config.schema.json`
- Do not run `just test`, `cargo test`, `cargo insta`, or snapshot acceptance commands.
- If the quick verifier makes tiny fixes, the coordinator creates `refs/simplepower/scratch/<run-id>/quick-verifier/after` and inspects:
  - `git diff refs/simplepower/scratch/<run-id>/quick-verifier/before refs/simplepower/scratch/<run-id>/quick-verifier/after -- <approved-files>`

## Final Review And Fix

- After the quick-verified implementation checkpoint, dispatch one REVIEW-tier review+fix agent using `model="gpt-5.5"` and `reasoning_effort="xhigh"`.
- Before dispatching review+fix, the coordinator creates `refs/simplepower/scratch/<run-id>/review-fix/before` for all approved implementation files.
- The review+fix agent reviews the whole implementation against the accepted plan, Interface Contract, File Ownership, approved path enforcement, aggregate parallel dispatch semantics, the no-tests user override, and verification requirements.
- The review+fix agent may edit files within the approved file ownership to fix issues. It must report changed files, commands run, results, remaining risks, and unresolved deviations requiring user approval.
- The review+fix agent performs the assigned review and fixes directly in the current worker. It must not run Codex CLI, spawn subagents, invoke Simple Power skills, restart execution, reroute the workflow, or commit.
- If review+fix edits files, the coordinator creates `refs/simplepower/scratch/<run-id>/review-fix/after` and inspects:
  - `git diff refs/simplepower/scratch/<run-id>/review-fix/before refs/simplepower/scratch/<run-id>/review-fix/after -- <approved-files>`

## Commit Checkpoints

Exactly three future coordinator checkpoint commits are authorized:

1. Accepted plan checkpoint: after the REVIEW-tier plan reviewer approves and the user gives combined approval for the reviewed plan, model/task allocation, and immediate current-session execution. This checkpoint happens before invoking `simplepower:subagent-driven-development`.
2. Quick-verified implementation checkpoint: after all `sp-impl` file edits complete and quick non-test verification passes.
3. Final checkpoint: after the REVIEW-tier review+fix agent completes and final non-test verification passes.

Workers, plan reviewers, quick verifiers, review+fix agents, and individual tasks must not commit. Scratch refs under `refs/simplepower/scratch/<run-id>/...` are coordinator-owned local review anchors only. They are not branches, accepted checkpoint commits, pushed, merged, or rebased, and they do not alter the exactly-three-checkpoint policy.

The three checkpoints above complete the feature implementation workflow. After the final feature checkpoint succeeds and phase scratch refs are cleaned up, the coordinator invokes `$mod-refresh-full-release` as a separate release workflow. Release workflow commits, tags, and publish actions are governed by the mod refresh release skills and are not extra Simple Power feature checkpoints.

After the accepted plan checkpoint succeeds, delete `plan-review` scratch refs:

```bash
git for-each-ref --format='%(refname)' "refs/simplepower/scratch/<run-id>/plan-review" | while read -r ref; do git update-ref -d "$ref"; done
```

After the quick-verified implementation checkpoint succeeds, delete `quick-verifier` scratch refs. After the final checkpoint succeeds, delete `review-fix` scratch refs. If the workflow stops because of user direction, a blocker, or a failed checkpoint commit, preserve remaining refs and report:

```bash
git for-each-ref --format='%(refname)' "refs/simplepower/scratch/<run-id>" | while read -r ref; do git update-ref -d "$ref"; done
```

## Current-Session Auto-Dispatch

After the REVIEW-tier plan reviewer approves, ask the user for one combined approval covering:

- The reviewed plan
- The model/task allocation
- Immediate current-session execution

If the user requests changes, update this plan, rerun focused self-review for the changed categories, create the next `plan-review/after-<n>` scratch ref, and send the revised plan back to the same reviewer with the concrete scratch-ref diff command. Do not create the accepted plan checkpoint until the user gives combined approval.

After combined approval, the coordinator creates the accepted plan checkpoint commit, deletes the successful `plan-review` scratch refs, then immediately invokes `simplepower:subagent-driven-development` in the current session with:

```text
Execute `docs/simplepower/plans/2026-06-18-yolo-default-and-compact-timer.md` with aggregate parallel implementation from the approved Interface Contract. Use the approved FAST/NORMAL/BEST/REVIEW model allocation. Dispatch all non-conflicting `sp-impl` file-edit workers whose coordination needs are satisfied by their Contract inputs, run the quick FAST-tier non-test verifier with formatting/schema/fix/diff-check commands and timeouts after all workers finish, commit the quick-verified implementation, then run one REVIEW-tier review+fix agent, final non-test verification, and final commit. Do not run `just test`, `cargo test`, `cargo insta`, or snapshot acceptance commands.
```

After the final feature checkpoint succeeds, immediately invoke `$mod-refresh-full-release` in the current session with this handoff:

```text
Run `$mod-refresh-full-release` for the finalized feature branch. Start with a fresh current-session `$mod-refresh-preflight`; do not mutate, merge, build, tag, or publish if preflight reports blockers. Carry forward: Tests not run unless explicitly requested; Bazel not used, Cargo release build only. Expected artifact path is repository-root `codex`. Expected upstreamhash.txt is the preflight upstream target SHA. Expected modversion.txt is `1` unless a different positive integer is explicitly approved. Preserve compact-fix behavior from docs/compact-fix/ChangeLog.md, including compact-only priority service-tier behavior and the new compact timer/status changes.
```

## Verification

Final non-test verification runs only after the REVIEW-tier review+fix agent has completed. The coordinator performs the final checkpoint only after these commands pass:

- `cd codex-rs && timeout 120s just fmt` - expected result: formatting succeeds; failure means generated or edited Rust needs correction.
- `cd codex-rs && timeout 120s just write-config-schema` - expected result: `codex-rs/core/config.schema.json` is current; failure means schema generation or config type shape is broken.
- `cd codex-rs && timeout 600s cargo check -p codex-core -p codex-tui` - expected result: changed Rust crates typecheck without running tests; failure means compile errors remain.
- `cd codex-rs && timeout 300s just fix -p codex-core` - expected result: scoped lints/fixes pass for core/config changes; failure means clippy or fixable lint issues remain.
- `cd codex-rs && timeout 300s just fix -p codex-tui` - expected result: scoped lints/fixes pass for TUI changes; failure means clippy or fixable lint issues remain.
- `timeout 30s git diff --check -- codex-rs/config/src/config_toml.rs codex-rs/core/src/config/mod.rs codex-rs/core/src/remote_compact_fallback.rs codex-rs/tui/src/history_cell/compact.rs codex-rs/tui/src/history_cell/mod.rs codex-rs/tui/src/chatwidget/protocol.rs codex-rs/tui/src/chatwidget/tool_lifecycle.rs codex-rs/tui/src/chatwidget/replay.rs codex-rs/tui/src/thread_transcript.rs codex-rs/tui/src/app/agent_status_feed.rs codex-rs/core/config.schema.json` - expected result: no whitespace errors.

Do not run `just test`, `cargo test`, `cargo insta`, or snapshot acceptance commands unless the user gives fresh explicit approval. Final reporting must state that tests and snapshots were skipped by explicit user request.

Final reporting must include this cleanup check:

```bash
git for-each-ref --format='%(refname)' "refs/simplepower/scratch/<run-id>"
```

If the final checkpoint succeeds, no scratch refs for this run should remain after phase cleanup. If the workflow stops because of user direction, a blocker, or a failed checkpoint commit, preserve remaining scratch refs and report the manual cleanup command from the Commit Checkpoints section.

## Post-Final Release

After final feature verification, the final feature checkpoint commit, and scratch-ref cleanup, invoke `$mod-refresh-full-release` immediately in the current session. That release workflow must:

- Run a fresh `$mod-refresh-preflight` first.
- Stop before mutation if preflight is blocked or stale.
- Record `Tests: not run unless explicitly requested`.
- Record `Bazel: not used; using Cargo release build only`.
- Use the preflight `upstream target SHA` as the expected `upstreamhash.txt`.
- Use expected `modversion.txt` value `1` unless the coordinator records a different explicit approved positive decimal integer.
- Chain `$mod-refresh-release`, `$mod-refresh-merge-preserve`, `$mod-refresh-build`, and `$mod-refresh-publish` only through the release skill gates.
- Finish with the release summary required by `$mod-refresh-full-release`.

## Approved Path Enforcement

This accepted implementation plan is authoritative after combined approval. It does not authorize backup routes, scope reduction, docs-only substitutes, stub substitutes, test execution despite the user override, skipped review, or execution-route changes. If implementation discovers that the approved path is blocked, unsafe, underspecified, or mismatched with the codebase, stop, report the exact mismatch and current status, and ask the user before changing approach.
