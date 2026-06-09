# V2 Remote Compact Policy Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `simplepower:subagent-driven-development` for aggregate parallel implementation. Dispatch all non-conflicting `sp-impl` file-edit workers whose coordination needs are satisfied by the approved Interface Contract, run the quick verifier after all workers finish, commit the quick-verified implementation, then run one REVIEW-tier review+fix agent before final verification and final commit.

**Goal:** Make `features.remote_compaction_v2 = true` use the same configured remote-first compact policy as V1, including bounded attempts, per-attempt timeout, fast service tier handling, local fallback, distinct V1/V2 logging, and local config enablement.

**Design Summary:** The approved design generalizes the existing V1 remote-first compact wrapper instead of duplicating policy inside `compact_remote_v2.rs`. When `[features] remote_compaction_v2 = true` exists in loaded `config.toml`, auto and manual compact choose V2 remote compaction, honor `[remote_compact] max_attempts = 3` and `attempt_timeout_sec = 180`, use fast service tier for compaction when supported, restore the original tier afterward, fall back to local compact after all visible attempts fail, and log/warn distinctly for V1 vs V2 attempts. The approved execution path updates `/home/gary/.codex/config.toml`, runs formatting and release compile, then copies the resulting binary locally and to the configured hosts; it does not require the normal release pipeline commands such as clippy, `just test`, or schema generation.

**Architecture:** Keep V1 and V2 compaction modules responsible for their bounded remote attempt loops, hooks, analytics, request execution, and path-specific history processing, while a version-aware remote-first wrapper owns version selection, fast-tier policy, clean-history restore, and local fallback. The Interface Contract below gives workers exact method shapes, settings, warning semantics, and file boundaries so implementation and test changes can be dispatched in aggregate parallel where write scopes do not overlap.

**Tech Stack:** Rust, Tokio async, Codex core session/turn orchestration, Responses API streaming, `/v1/responses/compact`, `codex-core` integration test harness, TOML config.

**Model Allocation:** FAST/NORMAL/BEST/REVIEW tiers are assigned below. Resolve each tier by explicit user override, quoted assignment in project root AGENTS.md, process environment variable, then built-in default. The project root AGENTS.md lookup reads only `<repo>/AGENTS.md`, not nested AGENTS.md files or repo-wide grep. FAST defaults to `SIMPLEPOWER_FAST_MODEL` (`gpt-5.3-codex-spark-high` when unset), NORMAL defaults to `SIMPLEPOWER_NORMAL_MODEL` (`gpt-5.4-mini-high` when unset), BEST defaults to `SIMPLEPOWER_BEST_MODEL` (`gpt-5.5-high` when unset), and REVIEW defaults to `SIMPLEPOWER_REVIEW_MODEL` (`gpt-5.5-xhigh` when unset). The plan reviewer is a REVIEW-tier plan reviewer, and the final review+fix agent is a REVIEW-tier review+fix agent. The quick verifier uses the FAST tier by default, resolving to `model="gpt-5.3-codex-spark"` and `reasoning_effort="high"` unless `SIMPLEPOWER_FAST_MODEL` is overridden.

**Commit Policy:** The coordinator commits after the reviewed plan, allocation, and immediate current-session execution receive combined approval, after all file edits and quick verification complete before final review, and after final review/fix plus final verification. Workers, plan reviewers, quick verifiers, and review+fix agents must not commit. No per-task commits. Coordinator-owned temporary scratch refs under `refs/simplepower/scratch/<run-id>/...` may be created only as local review diff anchors; they are not accepted history commits, not pushed, not merged, not rebased, and must be cleaned up after successful checkpoints or reported for manual cleanup on blockers or failed checkpoints.

---

## Interface Contract

1. **Remote compact version selection**
   - Define a crate-visible enum in `codex-rs/core/src/remote_compact_fallback.rs`:
     ```rust
     pub(crate) enum RemoteCompactVersion {
         V1,
         V2,
     }
     ```
   - Public entrypoints from `remote_compact_fallback.rs` must include:
     ```rust
     pub(crate) async fn run_remote_first_auto_compact(
         sess: &Arc<Session>,
         turn_context: &Arc<TurnContext>,
         client_session: &mut ModelClientSession,
         initial_context_injection: InitialContextInjection,
         reason: CompactionReason,
         phase: CompactionPhase,
         version: RemoteCompactVersion,
     ) -> CodexResult<()>;

     pub(crate) async fn run_remote_first_manual_compact(
         sess: Arc<Session>,
         turn_context: Arc<TurnContext>,
         version: RemoteCompactVersion,
     ) -> CodexResult<()>;
     ```
   - `RemoteCompactVersion` must not be exposed outside `codex-core`.
   - Existing V1-named public entrypoints may remain as thin compatibility wrappers only if that minimizes call-site churn.

2. **V1 and V2 remote task APIs**
   - `codex-rs/core/src/compact_remote.rs` owns V1 hooks, analytics, bounded visible attempts, `/v1/responses/compact` request execution, and V1 history installation through this exact API:
     ```rust
     #[derive(Clone, Debug)]
     pub(crate) struct RemoteCompactionRunSettings {
         pub(crate) service_tier_override: Option<String>,
         pub(crate) max_attempts: u64,
         pub(crate) attempt_timeout: Duration,
     }

     pub(crate) async fn run_remote_compact_task_for_mode(
         sess: &Arc<Session>,
         turn_context: &Arc<TurnContext>,
         initial_context_injection: InitialContextInjection,
         trigger: CompactionTrigger,
         reason: CompactionReason,
         phase: CompactionPhase,
         settings: RemoteCompactionRunSettings,
     ) -> CodexResult<()>;
     ```
   - `codex-rs/core/src/compact_remote_v2.rs` owns V2 hooks, analytics, bounded visible attempts, Responses stream execution, compaction output collection, and V2 history installation through this exact API:
     ```rust
     #[derive(Clone, Debug)]
     pub(crate) struct RemoteCompactionV2RunSettings {
         pub(crate) service_tier_override: Option<String>,
         pub(crate) max_attempts: u64,
         pub(crate) attempt_timeout: Duration,
     }

     pub(crate) async fn run_remote_compact_task_for_mode(
         sess: &Arc<Session>,
         turn_context: &Arc<TurnContext>,
         client_session: Option<&mut ModelClientSession>,
         initial_context_injection: InitialContextInjection,
         trigger: CompactionTrigger,
         reason: CompactionReason,
         phase: CompactionPhase,
         settings: RemoteCompactionV2RunSettings,
     ) -> CodexResult<()>;
     ```
   - V2 manual compaction passes `client_session: None` and creates its own model client session internally. V2 auto compaction passes `Some(client_session)` and must use the active `ModelClientSession` supplied by `session/turn.rs`.
   - Both APIs run pre-compact hooks once per remote compact task, before their visible attempt loop. Both APIs run post-compact hooks only after successful remote compaction. The wrapper must not run pre-compact or post-compact hooks for remote attempts.
   - Both APIs return remote failure errors to the wrapper without performing local fallback. V2 must not send a terminal `EventMsg::Error` before returning an error that the wrapper will handle with local fallback.
   - V2 hidden stream retry budget must not add extra visible attempts. Under this approved design, `remote_compact.max_attempts = 3` means exactly three V2 remote attempts total before local fallback.

3. **Remote compact settings**
   - `turn_context.config.remote_compact.max_attempts` and `turn_context.config.remote_compact.attempt_timeout` are authoritative for both V1 and V2 and are passed into the version-specific run settings by the wrapper.
   - V1 keeps using `tcp_keepalive_interval`; V2 does not need TCP keepalive unless the current streaming API already supports it.
   - V2 per-attempt timeout should wrap the whole stream request and output collection. Timeout errors should use `CodexErr::RequestTimeout` so existing remote compact warning classification can identify timeout attempts.

4. **Fast service tier policy**
   - Rename or generalize `V1RemoteFirstCompactServiceTier` and `resolve_v1_remote_first_compact_service_tiers` in `codex-rs/core/src/compact_service_tier.rs` so both V1 and V2 use the same logic.
   - API-key auth keeps remote attempts without `service_tier`; ChatGPT auth uses `priority` when the selected model advertises `ServiceTier::Fast`.
   - Local fallback uses the same fast-tier override selected for remote compaction when available.
   - Normal post-compact sampling must return to the original configured service tier.

5. **Fallback, hooks, and history policy**
   - For both V1 and V2, the wrapper snapshots clean history before remote attempts.
   - If all visible remote attempts fail, restore the clean history before local fallback.
   - `CodexErr::Interrupted` and `CodexErr::TurnAborted` return immediately and must not fall back locally.
   - Version-specific remote modules run pre-compact hooks once before their visible attempt loop. The wrapper does not run remote pre-compact hooks.
   - Local fallback uses `PreCompactHookPolicy::SkipAlreadyRan`, matching the existing V1 behavior after a remote module has already run pre-compact hooks.
   - Version-specific remote modules run post-compact hooks only after successful remote compaction. Local fallback runs its existing local post-compact behavior.

6. **Distinct V1/V2 logging and user-visible failure text**
   - Each remote attempt must emit a `tracing::info!` log with version, attempt number, total attempts, and turn id. Message text must distinguish "V1 remote compact" from "V2 remote compact".
   - Failure warning text must also include the version, for example `V2 remote compact attempt 1/3 timed out after 180s...`.
   - The final fallback warning must include the version, for example `V2 remote compact failed after 3 attempts; falling back to local compact.`
   - Existing fast-tier status messages may keep their current wording.

7. **Call-site behavior**
   - `codex-rs/core/src/session/turn.rs` selects `RemoteCompactVersion::V2` when `turn_context.features.enabled(Feature::RemoteCompactionV2)` is true, otherwise `RemoteCompactVersion::V1`.
   - `codex-rs/core/src/tasks/compact.rs` applies the same selection for manual `/compact`.
   - Metrics continue to use `remote_v2` for V2 remote attempts, `remote` for V1 remote attempts, and `local` for local fallback.

8. **Config file update**
   - `/home/gary/.codex/config.toml` must end with an enabled feature entry under the existing `[features]` table:
     ```toml
     remote_compaction_v2 = true
     ```
   - The same config must include:
     ```toml
     [remote_compact]
     max_attempts = 3
     attempt_timeout_sec = 180
     ```
   - Preserve unrelated user config entries and table ordering as much as practical.

9. **Approved execution commands**
   - Required formatting command after source edits:
     ```bash
     cd /home/gary/git/codex-compact-fix/codex-rs && timeout 600s just fmt
     ```
   - Required release compile:
     ```bash
     cd /home/gary/git/codex-compact-fix/codex-rs && timeout 3600s cargo build --release -p codex-cli
     test -x /home/gary/git/codex-compact-fix/codex-rs/target/release/codex
     ```
   - Required local copy:
     ```bash
     cp /home/gary/git/codex-compact-fix/codex-rs/target/release/codex /home/gary/codex
     ```
   - Required remote copy:
     ```bash
     for host in fpga01 axel office backup desk; do
       ssh "$host" 'killall codex || true; mkdir -p ~/.local/bin' &&
       scp /home/gary/git/codex-compact-fix/codex-rs/target/release/codex "$host:~/.local/bin/codex"
     done
     ```
   - Do not make `just fix`, `cargo clippy`, `just test`, `cargo test`, `cargo nextest`, or schema generation required commands for this approved run.

## File Ownership

| File | Owner task | Change type | Responsibility | Parallel safety notes |
| --- | --- | --- | --- | --- |
| `docs/simplepower/plans/2026-06-09-v2-remote-compact-policy.md` | Coordinator | create | Authoritative implementation plan and execution contract. | Coordinator-owned only; workers do not edit. |
| `codex-rs/core/src/compact_service_tier.rs` | Task 1 | modify | Generalize V1-only service tier naming and API for V1/V2 remote-first compact. | Exclusive to Task 1. |
| `codex-rs/core/src/remote_compact_fallback.rs` | Task 1 | modify | Version-aware remote-first orchestration, visible attempts, fallback, status/warnings, metrics dispatch. | Exclusive to Task 1. |
| `codex-rs/core/src/compact_remote.rs` | Task 2 | modify | Keep V1 remote attempt compatible with the generalized wrapper and add distinct V1 attempt logging/warning support. | Exclusive to Task 2; depends on Interface Contract for wrapper settings shape, not on Task 1's uncommitted code. |
| `codex-rs/core/src/compact_remote_v2.rs` | Task 2 | modify | Add V2 single-attempt settings, timeout handling, service tier override, and disable hidden visible retries. | Exclusive to Task 2. |
| `codex-rs/core/src/session/turn.rs` | Task 3 | modify | Route auto compact through the version-aware wrapper while preserving active `ModelClientSession` reuse. | Exclusive to Task 3. |
| `codex-rs/core/src/tasks/compact.rs` | Task 3 | modify | Route manual `/compact` through the version-aware wrapper and select V1/V2 consistently. | Exclusive to Task 3. |
| `codex-rs/core/tests/suite/compact_remote.rs` | Task 4 | modify | Add/update focused integration coverage for V2 max attempts, timeout/fallback, fast tier, and distinct warnings/log-visible text. | Exclusive to Task 4; tests target Interface Contract entries. |
| `/home/gary/.codex/config.toml` | Task 5 | modify | Enable `remote_compaction_v2` and set `[remote_compact] max attempts and timeout. | Exclusive to Task 5; outside repo, preserve user entries. |

## Visual Aids

```text
auto/manual compact
  |
  v
provider supports remote compact?
  | no
  v
local compact

  | yes
  v
select version from features.remote_compaction_v2
  |
  v
remote-first wrapper
  |-- resolve fast service tier
  |-- snapshot clean history
  |-- call selected remote module once
  |     |-- V1 module: hooks + attempt 1..=max_attempts against /v1/responses/compact
  |     `-- V2 module: hooks + attempt 1..=max_attempts via Responses stream
  |-- success: install compacted history, restore normal service tier status
  `-- all attempts fail: restore clean history, local fallback, restore normal service tier status
```

## Implementation Tasks

### Task 1: Generalize Remote-First Orchestration

**Goal:** Convert the V1-only remote-first fallback wrapper into a V1/V2 policy owner.

**Contract inputs:** Interface Contract entries 1, 3, 4, 5, 6, 7, and 9.

**Serialization required:** No. This task owns wrapper and service-tier files; other workers can target the approved interface.

**Write scope:**
- `codex-rs/core/src/compact_service_tier.rs`
- `codex-rs/core/src/remote_compact_fallback.rs`

**Parallel:** Yes, compatible with Task 2, Task 3, Task 4, and Task 5.

**Risk:** High, because this controls remote/local fallback semantics and hook/error behavior.

**Model tier:** BEST, resolved to `model="gpt-5.5"`, `reasoning_effort="xhigh"`.

**Worker role:** `sp-impl`.

**Outputs and file-level responsibilities:**
- Generalized service tier struct and resolver, with non-V1 names.
- `RemoteCompactVersion` and version-aware remote-first entrypoints.
- Wrapper passes `remote_compact.max_attempts`, `remote_compact.attempt_timeout`, and service tier override to the selected remote module.
- Wrapper-level final fallback warning text includes the selected remote compact version.
- Preserve current V1 fallback behavior while leaving per-attempt logs and warnings to Task 2.

**Implementation steps:**
1. In `compact_service_tier.rs`, rename `V1RemoteFirstCompactServiceTier` to `RemoteFirstCompactServiceTier` and `resolve_v1_remote_first_compact_service_tiers` to `resolve_remote_first_compact_service_tiers`.
2. In `remote_compact_fallback.rs`, add `RemoteCompactVersion` and update the remote-first functions to accept a version.
3. Replace `V1RemoteCompactKind` naming with a version-neutral kind only if needed for readability; keep the existing enum if renaming would create churn.
4. Keep visible remote attempt loops inside the selected remote module. The wrapper calls one V1 or V2 remote task, then falls back locally only if that remote task returns a non-interrupting error after its bounded attempts.
5. Ensure `Interrupted` and `TurnAborted` return immediately without local fallback.
6. Ensure local fallback restores clean history first and uses `PreCompactHookPolicy::SkipAlreadyRan`.
7. Emit metrics using `remote` for V1, `remote_v2` for V2, and `local` for fallback.

**Verification commands:**
```bash
cd /home/gary/git/codex-compact-fix/codex-rs && timeout 600s cargo check -p codex-core
```
Expected result: `codex-core` typechecks. If dependency edits outside the write scope are needed, stop and report the mismatch.

**Completion report requirements:** Changed files, summary of wrapper API, how V1 behavior was preserved, how V2 max attempts are enforced, commands run, command results, unresolved risks.

### Task 2: Adapt V1/V2 Remote Attempt Modules

**Goal:** Make V1 and V2 compaction modules conform to the wrapper contract while preserving their path-specific request and history logic.

**Contract inputs:** Interface Contract entries 1, 2, 3, 4, 6, and 7.

**Serialization required:** No. This task owns attempt modules and can implement against the approved wrapper contract.

**Write scope:**
- `codex-rs/core/src/compact_remote.rs`
- `codex-rs/core/src/compact_remote_v2.rs`

**Parallel:** Yes, compatible with Task 1, Task 3, Task 4, and Task 5.

**Risk:** High, because V2 streaming compaction must preserve active client session handling, history installation, token accounting, and tracing.

**Model tier:** BEST, resolved to `model="gpt-5.5"`, `reasoning_effort="xhigh"`.

**Worker role:** `sp-impl`.

**Outputs and file-level responsibilities:**
- V1 API exactly matches Interface Contract entry 2 and accepts `RemoteCompactionRunSettings`.
- V2 API exactly matches Interface Contract entry 2 and accepts `RemoteCompactionV2RunSettings`.
- V2 no longer performs hidden stream retries that increase visible attempts beyond `remote_compact.max_attempts`.
- V2 timeout maps to `CodexErr::RequestTimeout`.
- V2 stream requests use the override service tier when supplied, otherwise existing configured service tier behavior.
- V1 and V2 modules each run pre-hooks once before their bounded remote attempt loop and post-hooks only after successful remote compaction.
- V1 and V2 modules own per-attempt `tracing::info!` logs and per-attempt failure warnings because the attempt loops live in these modules.

**Implementation steps:**
1. In `compact_remote.rs`, update `RemoteCompactionRunSettings` to include `max_attempts: u64` and `attempt_timeout: Duration`; use `settings.max_attempts` and `settings.attempt_timeout` instead of reading those values inside the request loop. Keep `turn_context.config.remote_compact.tcp_keepalive_interval` for V1 keepalive.
2. In `compact_remote_v2.rs`, add the exact settings struct from Interface Contract entry 2:
   ```rust
   pub(crate) struct RemoteCompactionV2RunSettings {
       pub(crate) service_tier_override: Option<String>,
       pub(crate) max_attempts: u64,
       pub(crate) attempt_timeout: Duration,
   }
   ```
3. Pass `settings.service_tier_override.clone().or_else(|| turn_context.config.service_tier.clone())` to `client_session.stream(...)`.
4. Add a V2 loop `for attempt_number in 1..=settings.max_attempts` around the stream request and output collection. The final returned error after the loop is what triggers wrapper local fallback.
5. Wrap each V2 stream creation and `collect_compaction_output(stream)` in `tokio::time::timeout(settings.attempt_timeout, async { ... })`, mapping elapsed timeout to `CodexErr::RequestTimeout`.
6. Remove or bypass `handle_retryable_response_stream_error` for this remote compact path so only the visible `max_attempts` loop retries.
7. Add per-attempt `tracing::info!` logs in both V1 and V2 modules with version, attempt number, total attempts, and turn id.
8. Add or update per-attempt failure warnings in both V1 and V2 modules so warning text includes `V1 remote compact` or `V2 remote compact` and the configured attempt count.
9. Preserve `run_pre_compact_hooks` and `run_post_compact_hooks` inside the V1/V2 modules, with pre-hooks running once before the loop and post-hooks running only after successful remote compaction.
10. Preserve compaction analytics, rollout trace, `process_compacted_history`, `replace_compacted_history`, and token recomputation behavior.

**Verification commands:**
```bash
cd /home/gary/git/codex-compact-fix/codex-rs && timeout 600s cargo check -p codex-core
```
Expected result: `codex-core` typechecks. If the new attempt API requires wrapper code not yet present, report the expected temporary compile mismatch rather than editing Task 1 files.

**Completion report requirements:** Changed files, V2 retry/timeout behavior, service tier behavior, active `ModelClientSession` handling, commands run, command results, unresolved risks.

### Task 3: Route Auto And Manual Compact Through Version-Aware Wrapper

**Goal:** Update compact call sites to choose V1 or V2 once and enter the shared remote-first policy.

**Contract inputs:** Interface Contract entries 1, 4, 5, 7, and 9.

**Serialization required:** No. This task owns call sites and can code against the approved wrapper API.

**Write scope:**
- `codex-rs/core/src/session/turn.rs`
- `codex-rs/core/src/tasks/compact.rs`

**Parallel:** Yes, compatible with Task 1, Task 2, Task 4, and Task 5.

**Risk:** Medium, because call-site changes are small but must preserve active auto compact client session reuse.

**Model tier:** NORMAL, resolved to `model="gpt-5.4-mini"`, `reasoning_effort="xhigh"`.

**Worker role:** `sp-impl`.

**Outputs and file-level responsibilities:**
- Auto compact routes through `run_remote_first_auto_compact(..., RemoteCompactVersion::V1 | V2)`.
- Manual compact routes through `run_remote_first_manual_compact(..., RemoteCompactVersion::V1 | V2)`.
- Local-only branch remains unchanged for providers without remote compaction support.
- Metrics are not double-emitted at call sites if the wrapper owns remote/local metrics.

**Implementation steps:**
1. In `session/turn.rs`, replace direct V2 call and V1 fallback call with version selection:
   ```rust
   let version = if turn_context.features.enabled(Feature::RemoteCompactionV2) {
       RemoteCompactVersion::V2
   } else {
       RemoteCompactVersion::V1
   };
   ```
2. Pass the active `client_session` into the auto wrapper for both versions.
3. In `tasks/compact.rs`, apply the same feature-based version selection for manual compact.
4. Remove stale imports of direct V1/V2 remote compact functions if no longer needed.
5. Keep local compaction behavior unchanged when `should_use_remote_compact_task(...)` is false.

**Verification commands:**
```bash
cd /home/gary/git/codex-compact-fix/codex-rs && timeout 600s cargo check -p codex-core
```
Expected result: `codex-core` typechecks once Task 1 and Task 2 APIs are present. If APIs are not present yet, report the missing symbol names.

**Completion report requirements:** Changed files, selected wrapper entrypoints, active client session preservation, commands run, command results, unresolved risks.

### Task 4: Add Focused Integration Coverage

**Goal:** Add source-level coverage for the approved V2 remote-first behavior in the existing compact integration suite.

**Contract inputs:** Interface Contract entries 1, 2, 3, 4, 5, 6, 7, and existing test helpers in `codex-rs/core/tests/suite/compact_remote.rs`.

**Serialization required:** No. Tests target the approved interface and own only the test file.

**Write scope:**
- `codex-rs/core/tests/suite/compact_remote.rs`

**Parallel:** Yes, compatible with Task 1, Task 2, Task 3, and Task 5.

**Risk:** Medium, because integration tests must model SSE V2 compaction and local fallback request ordering correctly.

**Model tier:** NORMAL, resolved to `model="gpt-5.4-mini"`, `reasoning_effort="xhigh"`.

**Worker role:** `sp-impl`.

**Outputs and file-level responsibilities:**
- V2 success test remains or is updated to prove V2 still reuses compaction trigger and advertises beta feature.
- New V2 failure test verifies exactly configured `max_attempts` visible remote attempts before local fallback.
- New or updated V2 fast-tier test verifies remote V2 compact and local fallback use `priority` when supported, and post-fallback normal sampling returns to original tier.
- Warning assertions distinguish V1 from V2 text where failures occur.

**Implementation steps:**
1. Reuse existing helpers like `remote_compact_config`, `compact_service_tier_switch_messages`, `collect_warnings_until_turn_complete`, and `assert_local_fallback_compact_request_is_clean`.
2. Add a V2 fallback test with `config.features.enable(Feature::RemoteCompactionV2)` and `config.remote_compact = remote_compact_config(2, 180, 1000)` or equivalent.
3. Mount V2 remote compaction failures through Responses SSE mocks, not `/v1/responses/compact` mocks, then mount a local fallback SSE summary.
4. Assert exactly `max_attempts` V2 remote requests and that no `/v1/responses/compact` request is used in the V2 path.
5. Assert warning text includes `V2 remote compact` and the configured attempt count.
6. Add fast-tier assertions matching existing V1 expectations where practical.

**Verification commands:**
```bash
cd /home/gary/git/codex-compact-fix/codex-rs && timeout 600s cargo check -p codex-core
```
Expected result: `codex-core` typechecks. The approved current-session execution path does not require running the integration tests.

**Completion report requirements:** Changed tests, scenarios covered, mocks used, commands run, command results, unresolved risks.

### Task 5: Update Local User Config

**Goal:** Enable V2 remote compaction and set the requested remote compact policy in `/home/gary/.codex/config.toml`.

**Contract inputs:** Interface Contract entry 8.

**Serialization required:** No. This task owns only the user config file.

**Write scope:**
- `/home/gary/.codex/config.toml`

**Parallel:** Yes, compatible with Task 1, Task 2, Task 3, and Task 4.

**Risk:** Low, because this is a narrow TOML edit, but it affects the user's active Codex sessions after restart.

**Model tier:** FAST, resolved to `model="gpt-5.3-codex-spark"`, `reasoning_effort="high"`.

**Worker role:** `sp-impl`.

**Outputs and file-level responsibilities:**
- Existing `[features]` includes `remote_compaction_v2 = true`.
- Config includes `[remote_compact] max_attempts = 3` and `attempt_timeout_sec = 180`.
- No unrelated config entries are removed or reordered unnecessarily.

**Implementation steps:**
1. Read `/home/gary/.codex/config.toml`.
2. Add `remote_compaction_v2 = true` under the existing `[features]` table.
3. Add or update `[remote_compact]` with:
   ```toml
   max_attempts = 3
   attempt_timeout_sec = 180
   ```
4. Preserve all other user settings.

**Verification commands:**
```bash
timeout 30s rg -n "remote_compaction_v2|\\[remote_compact\\]|max_attempts|attempt_timeout_sec" /home/gary/.codex/config.toml
```
Expected result: all four requested config entries are present.

**Completion report requirements:** Changed file, exact config keys added or updated, command run, command result, unresolved risks.

## Model Allocation

| Stage | Role | Model tier | Resolved model | Reasoning effort | Reason |
| --- | --- | --- | --- | --- | --- |
| Plan review | REVIEW-tier plan reviewer | REVIEW | `gpt-5.5` | `xhigh` | Plan review must validate cross-task contract, scratch refs, approved path, and execution policy. |
| Task 1 | `sp-impl` orchestration worker | BEST | `gpt-5.5` | `xhigh` | Remote-first fallback orchestration is broad, behavior-shaping, and high-risk. |
| Task 2 | `sp-impl` remote attempt worker | BEST | `gpt-5.5` | `xhigh` | V2 streaming attempt behavior is subtle and hard to verify. |
| Task 3 | `sp-impl` call-site worker | NORMAL | `gpt-5.4-mini` | `xhigh` | Localized call-site routing using the approved wrapper contract. |
| Task 4 | `sp-impl` integration test worker | NORMAL | `gpt-5.4-mini` | `xhigh` | Focused test edits with existing helpers, but nontrivial request ordering. |
| Task 5 | `sp-impl` config worker | FAST | `gpt-5.3-codex-spark` | `high` | Mechanical TOML update. |
| Quick verification | FAST-tier quick verifier | FAST | `gpt-5.3-codex-spark` | `high` | Run formatting and release compile commands, with only tiny typo-level fixes allowed. |
| Final review/fix | REVIEW-tier review+fix agent | REVIEW | `gpt-5.5` | `xhigh` | Whole-change correctness review and fix against the accepted plan. |

Resolved tier sources for this plan: project root `AGENTS.md` does not define quoted `SIMPLEPOWER_*_MODEL` assignments; process environment provides `SIMPLEPOWER_FAST_MODEL=gpt-5.3-codex-spark-high`, `SIMPLEPOWER_NORMAL_MODEL=gpt-5.4-mini-xhigh`, `SIMPLEPOWER_BEST_MODEL=gpt-5.5-xhigh`, and `SIMPLEPOWER_REVIEW_MODEL=gpt-5.5-xhigh`.

## Aggregate Parallel Dispatch Guidance

Dispatch Tasks 1-5 together because the Interface Contract defines the shared APIs, settings shapes, warning semantics, and config contract. The coordinator should expect temporary compile mismatches while workers are in flight; integration happens after all workers return.

Do not dispatch two workers to the same file. If any worker discovers a required edit outside its write scope, it must report the needed file and stop instead of editing it. The coordinator decides whether the plan needs revision.

## Quick Verification

Before dispatching the quick verifier, create `refs/simplepower/scratch/<run-id>/quick-verifier/before` for these repo-tracked implementation files:

```text
codex-rs/core/src/compact_service_tier.rs
codex-rs/core/src/remote_compact_fallback.rs
codex-rs/core/src/compact_remote.rs
codex-rs/core/src/compact_remote_v2.rs
codex-rs/core/src/session/turn.rs
codex-rs/core/src/tasks/compact.rs
codex-rs/core/tests/suite/compact_remote.rs
```

`/home/gary/.codex/config.toml` is outside the repository and must not be included in scratch refs; verify it separately with the config `rg` command below.

The quick verifier may run and, if necessary, make only tiny typo-level fixes discovered by these commands:

```bash
cd /home/gary/git/codex-compact-fix/codex-rs && timeout 600s just fmt
cd /home/gary/git/codex-compact-fix/codex-rs && timeout 3600s cargo build --release -p codex-cli
test -x /home/gary/git/codex-compact-fix/codex-rs/target/release/codex
```

Expected result: formatting completes and the release Codex binary exists. Failure means implementation is not ready for the quick-verified implementation checkpoint. If quick verification changes files, create `refs/simplepower/scratch/<run-id>/quick-verifier/after` and inspect or hand off:

```bash
git diff refs/simplepower/scratch/<run-id>/quick-verifier/before refs/simplepower/scratch/<run-id>/quick-verifier/after -- codex-rs/core/src/compact_service_tier.rs codex-rs/core/src/remote_compact_fallback.rs codex-rs/core/src/compact_remote.rs codex-rs/core/src/compact_remote_v2.rs codex-rs/core/src/session/turn.rs codex-rs/core/src/tasks/compact.rs codex-rs/core/tests/suite/compact_remote.rs
```

The approved current-session path does not require `just fix`, `cargo clippy`, `just test`, `cargo test`, `cargo nextest`, or schema generation.

## Final Review And Fix

After the quick-verified implementation checkpoint, dispatch one REVIEW-tier review+fix agent. Before dispatch, create `refs/simplepower/scratch/<run-id>/review-fix/before` for the approved implementation file list. The review+fix agent reviews the whole change against the accepted plan, Interface Contract, file ownership, visible attempt semantics, service tier behavior, fallback behavior, config update, and release compile/copy requirements.

If the review+fix agent edits files, create `refs/simplepower/scratch/<run-id>/review-fix/after` and inspect or hand off:

```bash
git diff refs/simplepower/scratch/<run-id>/review-fix/before refs/simplepower/scratch/<run-id>/review-fix/after -- codex-rs/core/src/compact_service_tier.rs codex-rs/core/src/remote_compact_fallback.rs codex-rs/core/src/compact_remote.rs codex-rs/core/src/compact_remote_v2.rs codex-rs/core/src/session/turn.rs codex-rs/core/src/tasks/compact.rs codex-rs/core/tests/suite/compact_remote.rs
```

The review+fix agent must report changed files, commands run, results, remaining risks, and any unresolved deviation that requires user approval. It must not commit.

## Final Verification

Run final verification only after the REVIEW-tier review+fix agent completes:

```bash
cd /home/gary/git/codex-compact-fix/codex-rs && timeout 600s just fmt
cd /home/gary/git/codex-compact-fix/codex-rs && timeout 3600s cargo build --release -p codex-cli
test -x /home/gary/git/codex-compact-fix/codex-rs/target/release/codex
timeout 30s rg -n "remote_compaction_v2|\\[remote_compact\\]|max_attempts|attempt_timeout_sec" /home/gary/.codex/config.toml
cp /home/gary/git/codex-compact-fix/codex-rs/target/release/codex /home/gary/codex
for host in fpga01 axel office backup desk; do
  ssh "$host" 'killall codex || true; mkdir -p ~/.local/bin' &&
  scp /home/gary/git/codex-compact-fix/codex-rs/target/release/codex "$host:~/.local/bin/codex"
done
git for-each-ref --format='%(refname)' "refs/simplepower/scratch/<run-id>"
```

Expected result: formatting completes, release binary exists, config entries are present, local copy succeeds, each remote host receives the binary, and no scratch refs remain after successful phase cleanup. Failure means the final checkpoint must not be created until the command failure is resolved or the user approves a plan change.

## Commit Checkpoints

1. **Accepted plan checkpoint:** After the user gives combined approval for the reviewed plan, model/task allocation, and immediate current-session execution, and before invoking `simplepower:subagent-driven-development`.
2. **Quick-verified implementation checkpoint:** After all `sp-impl` file edits complete and quick verification passes.
3. **Final checkpoint:** After the REVIEW-tier review+fix agent completes and final verification passes.

Workers, plan reviewers, quick verifiers, and review+fix agents must not commit. Scratch refs are coordinator-owned local review anchors only and must be deleted after successful checkpoints or preserved and reported for manual cleanup if the workflow stops or a checkpoint commit fails.

## Scratch Ref Workflow

Use run id format `YYYYMMDD-HHMMSS-<short-head>`, for example `20260609-120000-69ded0c`. All scratch refs for this run live under:

```text
refs/simplepower/scratch/<run-id>/
```

Create `plan-review/before` before the first plan review for this plan file. If the plan is revised after review feedback, create `plan-review/after-<n>` and send the same reviewer this diff command:

```bash
git diff refs/simplepower/scratch/<run-id>/plan-review/before refs/simplepower/scratch/<run-id>/plan-review/after-1 -- docs/simplepower/plans/2026-06-09-v2-remote-compact-policy.md
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

## Current-Session Auto-Dispatch

After the REVIEW-tier plan reviewer approves, ask the user for one combined approval covering:

- the reviewed plan
- the model/task allocation
- immediate current-session execution

After combined approval, the coordinator creates the accepted plan checkpoint commit, deletes the successful `plan-review` scratch refs, then immediately invokes `simplepower:subagent-driven-development` in the current session with:

```text
Execute `docs/simplepower/plans/2026-06-09-v2-remote-compact-policy.md` with aggregate parallel implementation from the approved Interface Contract. Use the approved FAST/NORMAL/BEST/REVIEW model allocation. Dispatch all non-conflicting `sp-impl` file-edit workers whose coordination needs are satisfied by their Contract inputs, run the quick FAST-tier verifier with the approved format and release-build commands after all workers finish, commit the quick-verified implementation, then run one REVIEW-tier review+fix agent, final verification, binary copy/deploy, and final commit.
```
