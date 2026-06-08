# Remote Compact Timeout Fallback Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `simplepower:subagent-driven-development` for aggregate parallel implementation. Dispatch all non-conflicting `sp-impl` file-edit workers whose coordination needs are satisfied by the approved Interface Contract, run the quick verifier after all workers finish, commit the quick-verified implementation, then run one REVIEW-tier review+fix agent before final verification and final commit.

**Goal:** Make V1 remote compaction try exactly 3 visible 3-minute attempts, then fall back to local compaction without putting failed remote compact artifacts into the local compact model history.

**Design Summary:** V1 remote compact (`/responses/compact`) remains the preferred path when the provider supports remote compaction and `remote_compaction_v2` is not enabled. V1 remote compact attempts are capped at 3 total attempts, each with a 180-second attempt budget. Every failed V1 remote attempt, including each timeout, emits a visible protocol warning, and exhaustion emits a visible fallback warning before local compact starts. Local fallback must use the same model-visible session history that existed before the first V1 remote attempt: no failed remote request content, partial remote output, remote error text, fallback warning text, failed `ContextCompaction` lifecycle item, or other failed remote artifacts may be recorded into conversation history. Explicit interruption and pre-compact hook aborts still abort instead of falling back. `remote_compaction_v2` is intentionally out of scope for this branch and keeps its current routing, retry, timeout, and failure behavior when the feature is enabled. The lower-level HTTP/HTTPS audit found no reqwest global request/connect/read timeout below 3 minutes; the default reqwest client has no global request timeout, the compact endpoint sets per-request timeouts, and WebSocket connect timeout is 15 seconds but only covers handshake setup.

**Architecture:** Add a small V1 remote-compact fallback coordinator in `codex-core` that owns V1 remote-first/manual and V1 remote-first/auto routing and centralizes user-visible fallback warnings. Refactor only the V1 remote compact helper to support side-effect-light fallback mode: failed attempts may warn and trace, but only successful remote compaction may replace session history or emit completed compaction lifecycle state. The Interface Contract below defines the exact helper APIs and behavior so timeout plumbing, remote helper refactors, routing, and tests can be worked in parallel without sharing uncommitted implementation details.

**Tech Stack:** Rust async/Tokio, `codex-core`, `codex-api`, `codex-client`, reqwest transport, Responses HTTP/SSE/WebSocket APIs, existing `EventMsg::Warning` and `EventMsg::StreamError` protocol events, existing core integration test suite.

**Model Allocation:** FAST/NORMAL/BEST/REVIEW tiers are assigned below. Resolve each tier by explicit user override, quoted assignment in project root AGENTS.md, process environment variable, then built-in default. The project root AGENTS.md lookup reads only `<repo>/AGENTS.md`, not nested AGENTS.md files or repo-wide grep. FAST defaults to `SIMPLEPOWER_FAST_MODEL` (`gpt-5.3-codex-spark-high` when unset), NORMAL defaults to `SIMPLEPOWER_NORMAL_MODEL` (`gpt-5.4-mini-high` when unset), BEST defaults to `SIMPLEPOWER_BEST_MODEL` (`gpt-5.5-high` when unset), and REVIEW defaults to `SIMPLEPOWER_REVIEW_MODEL` (`gpt-5.5-xhigh` when unset). The plan reviewer is a REVIEW-tier plan reviewer, and the final review+fix agent is a REVIEW-tier review+fix agent. The quick verifier uses the FAST tier by default, resolving to `model="gpt-5.3-codex-spark"` and `reasoning_effort="high"` unless `SIMPLEPOWER_FAST_MODEL` is overridden.

Resolved tiers for this run from environment, with no current-request override and no quoted model assignments read from project root `AGENTS.md`: FAST = `model="gpt-5.3-codex-spark"`, `reasoning_effort="high"`; NORMAL = `model="gpt-5.4-mini"`, `reasoning_effort="xhigh"`; BEST = `model="gpt-5.5"`, `reasoning_effort="xhigh"`; REVIEW = `model="gpt-5.5"`, `reasoning_effort="xhigh"`.

**Commit Policy:** The coordinator commits after the reviewed plan, allocation, and immediate current-session execution receive combined approval, after all file edits and quick verification complete before final review, and after final review/fix plus final verification. Workers, plan reviewers, quick verifiers, and review+fix agents must not commit. No per-task commits. Coordinator-owned temporary scratch refs under `refs/simplepower/scratch/<run-id>/...` may be created only as local review diff anchors; they are not accepted history commits, not pushed, not merged, not rebased, and must be cleaned up after successful checkpoints or reported for manual cleanup on blockers or failed checkpoints.

---

## Interface Contract

1. `remote_compact_fallback` module contract:
   - New file: `codex-rs/core/src/remote_compact_fallback.rs`.
   - Constants:
     - `pub(crate) const REMOTE_COMPACT_TOTAL_ATTEMPTS: u64 = 3;`
     - `pub(crate) const REMOTE_COMPACT_ATTEMPT_TIMEOUT: Duration = Duration::from_secs(180);`
   - Public crate APIs:
     - `pub(crate) async fn run_v1_remote_first_auto_compact(sess: &Arc<Session>, turn_context: &Arc<TurnContext>, initial_context_injection: InitialContextInjection, reason: CompactionReason, phase: CompactionPhase) -> CodexResult<()>`
     - `pub(crate) async fn run_v1_remote_first_manual_compact(sess: Arc<Session>, turn_context: Arc<TurnContext>) -> CodexResult<()>`
   - Routing behavior:
     - Callers in `session/turn.rs` and `tasks/compact.rs` keep the existing direct local path when `should_use_remote_compact_task(turn_context.provider.info())` is false.
     - Callers in `session/turn.rs` and `tasks/compact.rs` keep the existing V2 path when `Feature::RemoteCompactionV2` is enabled; V2 behavior is unchanged and out of scope.
     - Callers use this module only when remote is supported and V2 is disabled, then emit `"remote"` for the V1 remote attempt path and call the V1 fallback-mode helper.
     - On remote success, return success and do not run local compact.
     - On `CodexErr::Interrupted` or `CodexErr::TurnAborted`, return the error and do not run local compact.
     - On any other remote error after the remote helper exhausts 3 total attempts, emit one `EventMsg::Warning` whose message includes `Remote compact failed after 3 attempts; falling back to local compact`, then emit `"local"` and run the existing local compact path.
   - Clean-history guarantee:
     - The coordinator must clone the session history before the first remote attempt.
     - The local fallback path must use history equivalent to that pre-remote snapshot.
     - Remote failure warnings are protocol events only; they must not be added with `record_conversation_items`, must not be included in `Prompt.input`, and must not be included in `CompactedItem.replacement_history`.
     - Failed remote attempts must not call `replace_compacted_history`, must not call local `record_conversation_items`, and must not leave a failed `ContextCompaction` item in conversation history.

2. V1 remote compact helper contract:
   - File: `codex-rs/core/src/compact_remote.rs`.
   - New public crate enum, defined only in this file:
     - `pub(crate) enum RemoteCompactionFailureMode { TerminalError, FallbackToLocal }`
   - New or refactored public crate API:
     - `pub(crate) async fn run_remote_compact_task_for_mode(sess: &Arc<Session>, turn_context: &Arc<TurnContext>, initial_context_injection: InitialContextInjection, trigger: CompactionTrigger, reason: CompactionReason, phase: CompactionPhase, failure_mode: RemoteCompactionFailureMode) -> CodexResult<()>`
   - `TerminalError` preserves current direct remote behavior for callers that intentionally want a terminal remote error.
   - `FallbackToLocal` must not send `EventMsg::Error`, must not call `track_turn_codex_error`, and must not record a failed `ContextCompaction` lifecycle item. It may emit `EventMsg::Warning` for failed attempts and may write tracing/analytics.
   - V1 remote request attempts are exactly 3 total calls to `/responses/compact`; each request receives `REMOTE_COMPACT_ATTEMPT_TIMEOUT`.
   - V1 must not multiply attempts by the provider default `request_max_retries`; the compact endpoint call used by V1 fallback mode must run with zero hidden transport retries, and the visible V1 remote helper owns the 3-attempt loop.

3. V2 exclusion contract:
   - Files intentionally not edited for V2: `codex-rs/core/src/compact_remote_v2.rs` and `codex-rs/core/src/responses_retry.rs`.
   - When `Feature::RemoteCompactionV2` is enabled, `session/turn.rs` and `tasks/compact.rs` must continue to call the existing V2 functions with current behavior.
   - This branch does not change V2 retry count, V2 timeout behavior, V2 WebSocket-to-HTTP fallback behavior, V2 tests, or V2 snapshots.

4. Compact endpoint retry/timeout contract:
   - Files: `codex-rs/codex-api/src/endpoint/session.rs`, `codex-rs/codex-api/src/endpoint/compact.rs`, and `codex-rs/core/src/client.rs`.
   - `EndpointSession` exposes `execute_with_policy(...)`, a request execution method that accepts an explicit `codex_client::RetryPolicy`. Existing `execute_with(...)` continues to call `execute_with_policy(...)` with `self.provider.retry.to_policy()`.
   - `codex-rs/codex-api/src/lib.rs` re-exports `codex_client::RetryPolicy` as `codex_api::RetryPolicy` and `codex_client::RetryOn` as `codex_api::RetryOn` so `codex-core` can name the existing retry types without adding a new direct dependency.
   - `CompactClient` exposes `compact_input_with_policy(...)`, which accepts `request_timeout: Duration` and `retry_policy: codex_api::RetryPolicy`. Existing compact methods delegate to it with the provider default policy.
   - `CompactConversationRequestSettings` in `codex-rs/core/src/client.rs` gains these exact fields:
     - `pub(crate) request_timeout: Duration`
     - `pub(crate) retry_policy: codex_api::RetryPolicy`
   - V1 remote compaction sets `request_timeout` to `REMOTE_COMPACT_ATTEMPT_TIMEOUT` and sets `retry_policy.max_attempts` to `0`, with retry-on flags false or otherwise ineffective, so each visible V1 attempt maps to exactly one `/responses/compact` HTTP request.
   - Existing non-compact Responses, memories, images, and search endpoints keep their current provider retry and timeout behavior.

5. Warning/reporting contract:
   - Failed remote attempts emit `EventMsg::Warning(WarningEvent { message })`.
   - Timeout message format must include the attempt number, total attempt count, and 180-second budget, for example: `Remote compact attempt 1/3 timed out after 180s; retrying remote compact.`
   - Non-timeout failure message format must include the attempt number, total attempt count, and error text, for example: `Remote compact attempt 2/3 failed: <error>; retrying remote compact.`
   - Final remote failure message must include: `Remote compact failed after 3 attempts; falling back to local compact.`
   - These warnings are visible protocol events and may be persisted in rollout, but must not be model-visible conversation history.

6. Tests and fixtures contract:
   - Core integration tests in `codex-rs/core/tests/suite/compact_remote.rs` cover:
     - V1 auto remote compact failure sends 3 remote compact requests, emits warnings for each failure/timeout, then performs local compact and continues the agent loop.
     - V1 manual `/compact` remote failure sends 3 remote compact requests, emits the fallback warning, then performs local compact.
     - The post-fallback local compact request body does not contain remote failure text, timeout warning text, fallback warning text, failed remote compact output, or failed remote `ContextCompaction` content.
   - Existing failure tests that asserted terminal remote error behavior must be updated to the fallback behavior.
   - Existing V2 tests remain unchanged because V2 behavior is out of scope.
   - Snapshot fixtures under `codex-rs/core/tests/suite/snapshots/` may be updated only where the changed fallback behavior intentionally changes request-shape snapshots.

7. Verification and deployment command contract:
   - User explicitly requested no test runs during this execution. Workers may author/update tests, but quick and final verification must not run `just test` or `cargo test`.
   - Required formatting command after code edits: `timeout 600s just fmt` from repo root.
   - Required lint/fix command before finalizing code changes: `timeout 1800s just fix -p codex-api -p codex-core` from repo root.
   - Required release build after implementation/final verification: `timeout 3600s just build-for-release` from repo root.
   - Release binary discovery command after build: `find bazel-bin -type f -name codex -perm -111 | sort`.
   - Deployment target hosts: `fpga01`, `axel`, `backup`, `desk`, `office`.
   - Deployment command shape after selecting the release binary path: `for host in fpga01 axel backup desk office; do ssh "$host" 'mkdir -p ~/.local/bin' && scp <release-codex-binary> "$host:~/.local/bin/codex"; done`.

## File Ownership

| File | Owner task | Change type | Responsibility | Parallel safety notes |
|------|------------|-------------|----------------|-----------------------|
| `codex-rs/codex-api/src/lib.rs` | Task A: Compact endpoint retry controls | modify | Re-export `codex_client::RetryPolicy` and `codex_client::RetryOn` for use by `codex-core` without a new direct dependency. | Implied-scope omission correction after Task A blocker; parallel with Tasks B, C, and D because no other task edits this file. |
| `codex-rs/codex-api/src/endpoint/session.rs` | Task A: Compact endpoint retry controls | modify | Add explicit retry-policy execution path while preserving existing default retry behavior. | Parallel with Tasks B, C, and D because no other task edits this file. |
| `codex-rs/codex-api/src/endpoint/compact.rs` | Task A: Compact endpoint retry controls | modify | Add compact call variant accepting explicit timeout and retry policy. | Parallel with Tasks B, C, and D because no other task edits this file. |
| `codex-rs/core/src/client.rs` | Task A: Compact endpoint retry controls | modify | Thread compact-specific timeout/retry controls through `CompactConversationRequestSettings` and V1 compact endpoint call. | Parallel with Tasks B, C, and D because no other task edits this file. |
| `codex-rs/core/src/compact_remote.rs` | Task B: Side-effect-light remote helpers | modify | Add V1 fallback-mode remote helper, 3 visible attempts, warning reporting, no hidden retries, and clean failure behavior. | Parallel with Tasks A, C, and D through Interface Contract entries 2, 4, and 5. |
| `codex-rs/core/src/remote_compact_fallback.rs` | Task C: V1 remote-first fallback routing | create | Implement V1 remote-first coordinator, local fallback warning, clean-history guard, and manual/auto entrypoints. | Parallel with Tasks A, B, and D through Interface Contract entries 1 through 5. |
| `codex-rs/core/src/lib.rs` | Task C: Remote-first fallback routing | modify | Register the new `remote_compact_fallback` module. | Parallel with Tasks A, B, and D because no other task edits this file. |
| `codex-rs/core/src/compact.rs` | Task C: Remote-first fallback routing | modify | Expose a local compaction entrypoint usable by the fallback coordinator without duplicate manual `TurnStarted` events and without remote artifacts. | Parallel with Tasks A, B, and D because no other task edits this file. |
| `codex-rs/core/src/session/turn.rs` | Task C: V1 remote-first fallback routing | modify | Replace the V1 auto remote/local branch with the new V1 remote-first fallback coordinator while leaving the V2 branch unchanged. | Parallel with Tasks A, B, and D because no other task edits this file. |
| `codex-rs/core/src/tasks/compact.rs` | Task C: V1 remote-first fallback routing | modify | Replace the V1 manual `/compact` remote/local branch with the new V1 remote-first fallback coordinator while leaving the V2 branch unchanged. | Parallel with Tasks A, B, and D because no other task edits this file. |
| `codex-rs/core/tests/suite/compact_remote.rs` | Task D: V1 remote compact fallback tests | modify | Update/add integration tests for V1 fallback, warning visibility, exact attempt counts, and clean local fallback history. | Parallel with Tasks A, B, and C through Interface Contract entries 1 through 6. |
| `codex-rs/core/tests/suite/snapshots/all__suite__compact_remote__remote_pre_turn_compaction_failure_shapes.snap` | Task D: Remote compact fallback tests | modify | Update intentional request-shape snapshot if the pre-turn remote failure path now falls back locally. | Parallel with Tasks A, B, and C because only Task D owns snapshots. |

## Visual Aids

```text
Manual /compact or auto compact
        |
        v
remote supported?
        | no
        v
local compact
        |
        v
done

remote supported? yes
        |
        v
remote_compaction_v2 enabled?
        | yes
        v
existing V2 path unchanged
        |
        v
done

remote_compaction_v2 enabled? no
        |
        v
V1 /responses/compact fallback path
        |
        v
attempt 1, max 180s -> warn on failure/timeout
        |
        v
attempt 2, max 180s -> warn on failure/timeout
        |
        v
attempt 3, max 180s -> warn on failure/timeout
        |
        +--> success: install remote compacted history
        |
        +--> final failure: warn fallback, run local compact from pre-remote history
```

## Implementation Tasks

### Task A: Compact endpoint retry controls

- **Goal:** Give `/responses/compact` an explicit compact-specific timeout and zero-hidden-retry call path so V1 remote compact can own exactly 3 visible attempts.
- **Contract inputs:** Interface Contract entries 2, 4, and 7; approved design detail that each remote attempt is capped at 180 seconds and there are exactly 3 visible attempts total.
- **Serialization required:** No. The Interface Contract defines the API shape Task B will call, and Task A owns all files it edits.
- **Write scope:** `codex-rs/codex-api/src/lib.rs`, `codex-rs/codex-api/src/endpoint/session.rs`, `codex-rs/codex-api/src/endpoint/compact.rs`, `codex-rs/core/src/client.rs`.
- **Parallel:** Yes, compatible with Tasks B, C, and D.
- **Risk:** Medium because it touches shared API transport code, but the changes are localized and must preserve default behavior for existing callers.
- **Model tier:** NORMAL, resolved to `model="gpt-5.4-mini"`, `reasoning_effort="xhigh"`.
- **Worker role:** `sp-impl`.
- **Outputs and file-level responsibilities:** A self-documenting compact request settings path that lets callers pass `REMOTE_COMPACT_ATTEMPT_TIMEOUT` and a retry policy equivalent to zero hidden retries; no behavior change for non-compact endpoints.
- **Implementation steps:**
  - In `codex-rs/codex-api/src/endpoint/session.rs`, add `execute_with_policy(...)`, an execution helper that accepts an explicit `codex_client::RetryPolicy`. Keep `execute_with(...)` as a wrapper that passes `self.provider.retry.to_policy()`.
  - In `codex-rs/codex-api/src/lib.rs`, re-export `codex_client::RetryPolicy` as `codex_api::RetryPolicy` and `codex_client::RetryOn` as `codex_api::RetryOn`.
  - In `codex-rs/codex-api/src/endpoint/compact.rs`, add `compact_input_with_policy(...)`, a compact method that accepts `request_timeout: Duration` and `retry_policy: codex_api::RetryPolicy`. Existing compact methods delegate to the new method with the provider default policy.
  - In `codex-rs/core/src/client.rs`, add exact fields `request_timeout: Duration` and `retry_policy: codex_api::RetryPolicy` to `CompactConversationRequestSettings` so V1 remote compact can set request timeout and retry policy without positional booleans or ambiguous `Option`s.
  - Replace the current `COMPACT_REQUEST_TIMEOUT_IDLE_MULTIPLIER` behavior for remote compact callers with the explicit compact timeout supplied by settings. Remove the multiplier constant if it becomes unused.
  - Ensure `run_with_request_telemetry` still wraps the compact request so telemetry is preserved.
- **Verification commands:**
  - `timeout 30s git diff --check -- codex-rs/codex-api/src/lib.rs codex-rs/codex-api/src/endpoint/session.rs codex-rs/codex-api/src/endpoint/compact.rs codex-rs/core/src/client.rs` from repo root; expected result: no whitespace errors. Do not run mutating repo-wide commands from the worker.
- **Completion report requirements:** Report changed files, the exact compact retry API added, whether any non-compact caller changed behavior, commands run, command results, and unresolved risks.

### Task B: Side-effect-light remote helpers

- **Goal:** Refactor V1 remote compaction so fallback-mode failures are visible but do not become terminal turn errors or model-visible history, and so V1 remote attempts are exactly 3 total attempts with 180-second per-attempt caps.
- **Contract inputs:** Interface Contract entries 1 through 5; approved design detail that explicit interruption and pre-compact hook abort do not fall back local.
- **Serialization required:** No. The Interface Contract defines the settings and coordinator APIs Task B can rely on, and Task B owns the V1 remote helper file it edits.
- **Write scope:** `codex-rs/core/src/compact_remote.rs`.
- **Parallel:** Yes, compatible with Tasks A, C, and D.
- **Risk:** High because this owns the remote failure semantics, retry budget, lifecycle event timing, and clean-history guarantee.
- **Model tier:** BEST, resolved to `model="gpt-5.5"`, `reasoning_effort="xhigh"`.
- **Worker role:** `sp-impl`.
- **Outputs and file-level responsibilities:** Fallback-mode remote helper API for V1, visible warning emission for every failed/timeout attempt, no terminal error emission in fallback mode, and no failed remote compaction lifecycle artifact in model-visible history.
- **Implementation steps:**
  - In `compact_remote.rs`, introduce `RemoteCompactionFailureMode` as an enum, not a bool. Implement the fallback-mode V1 helper from Interface Contract entry 2.
  - Move V1 request execution into a visible 3-attempt loop. Each failed attempt emits a warning through `EventMsg::Warning`; timeout messages must include `timed out after 180s`.
  - Ensure the V1 compact endpoint call uses the Task A zero-hidden-retry path so provider `request_max_retries` does not multiply attempts.
  - In fallback mode, do not call `sess.track_turn_codex_error` and do not send `EventMsg::Error` when all remote attempts fail. Return the last remote error to the coordinator.
  - Delay or suppress remote compaction lifecycle item emission in fallback mode so a failed remote attempt does not leave a failed `ContextCompaction` artifact. Successful remote compaction must still install the replacement history and emit completed lifecycle state.
  - Do not edit `compact_remote_v2.rs` or `responses_retry.rs`; V2 behavior is out of scope and remains unchanged.
- **Verification commands:**
  - `timeout 30s git diff --check -- codex-rs/core/src/compact_remote.rs` from repo root; expected result: no whitespace errors. Do not run mutating repo-wide commands from the worker.
- **Completion report requirements:** Report changed files, exact V1 attempt-count semantics, warning message formats, how hidden retries were prevented, commands run, command results, and unresolved risks.

### Task C: V1 remote-first fallback routing

- **Goal:** Route V1 manual and V1 auto compaction through a single remote-first coordinator that falls back to local compaction after remote exhaustion while keeping local fallback history clean.
- **Contract inputs:** Interface Contract entries 1 through 5 and 7; approved branch name `fix/remote-compact-timeout-fallback`.
- **Serialization required:** No. The Interface Contract defines remote helper APIs and compact endpoint settings Task C can call before Tasks A and B finish.
- **Write scope:** `codex-rs/core/src/remote_compact_fallback.rs`, `codex-rs/core/src/lib.rs`, `codex-rs/core/src/compact.rs`, `codex-rs/core/src/session/turn.rs`, `codex-rs/core/src/tasks/compact.rs`.
- **Parallel:** Yes, compatible with Tasks A, B, and D.
- **Risk:** High because this changes user-facing compaction routing and must avoid duplicate `TurnStarted`, duplicate hooks, or polluted local fallback context.
- **Model tier:** BEST, resolved to `model="gpt-5.5"`, `reasoning_effort="xhigh"`.
- **Worker role:** `sp-impl`.
- **Outputs and file-level responsibilities:** New V1 fallback coordinator module, V1 manual and V1 auto routing updates, local compact entrypoint suitable for fallback, and clean-history guard logic.
- **Implementation steps:**
  - Add `mod remote_compact_fallback;` to `codex-rs/core/src/lib.rs`.
  - Implement `remote_compact_fallback.rs` with the constants and entrypoints from Interface Contract entry 1.
  - In `compact.rs`, expose a crate-private local compact entrypoint the coordinator can call after it already handled manual `TurnStarted`, without creating duplicate manual start events. Preserve existing `run_inline_auto_compact_task` and `run_compact_task` behavior for direct local callers.
  - In `session/turn.rs`, keep the existing V2 branch unchanged and replace only the V1 auto remote branch with a call to `remote_compact_fallback::run_v1_remote_first_auto_compact(...)`.
  - In `tasks/compact.rs`, keep the existing V2 branch unchanged and replace only the V1 manual remote branch with `remote_compact_fallback::run_v1_remote_first_manual_compact(...)`.
  - Clone the session history before the first remote attempt. Prefer side-effect-light remote helpers so no restore is needed; if a defensive restore is necessary, restore only in-memory context before local fallback and do not persist failed remote items as conversation items.
  - Ensure fallback warning emission happens before local compact starts and remains protocol-only.
  - Keep `should_use_remote_compact_task(provider)` as provider capability logic; do not force local compact globally like `fix/force-local-compact`.
- **Verification commands:**
  - `timeout 30s git diff --check -- codex-rs/core/src/remote_compact_fallback.rs codex-rs/core/src/lib.rs codex-rs/core/src/compact.rs codex-rs/core/src/session/turn.rs codex-rs/core/src/tasks/compact.rs` from repo root; expected result: no whitespace errors. Do not run mutating repo-wide commands from the worker.
- **Completion report requirements:** Report changed files, exact manual and auto routing behavior, how duplicate start/hook behavior was avoided, how clean-history fallback is enforced, commands run, command results, and unresolved risks.

### Task D: Remote compact fallback tests

- **Goal:** Update the remote compact integration coverage to describe the new fallback behavior and guard against polluted local fallback history.
- **Contract inputs:** Interface Contract entry 6 and approved design details that tests may be authored but not run during this execution because the user requested no test runs.
- **Serialization required:** No. The Interface Contract defines the expected APIs and behavior, and Task D owns all test/snapshot files it edits.
- **Write scope:** `codex-rs/core/tests/suite/compact_remote.rs`, `codex-rs/core/tests/suite/snapshots/all__suite__compact_remote__remote_pre_turn_compaction_failure_shapes.snap`.
- **Parallel:** Yes, compatible with Tasks A, B, and C.
- **Risk:** Medium because tests target cross-cutting behavior, but write scope is isolated to tests and snapshots.
- **Model tier:** NORMAL, resolved to `model="gpt-5.4-mini"`, `reasoning_effort="xhigh"`.
- **Worker role:** `sp-impl`.
- **Outputs and file-level responsibilities:** Updated failure tests that expect fallback instead of terminal remote errors, new assertions for 3 attempts and warning visibility, and clean local fallback model-history assertions.
- **Implementation steps:**
  - Update `auto_remote_compact_failure_stops_agent_loop` or replace it with a test that mounts three failing remote compact responses, verifies three `/responses/compact` requests, waits for fallback warning events, then verifies the next normal sampling request happens after local compact.
  - Update the manual remote compact failure test near the existing manual remote failure assertions to expect fallback local compact instead of terminal `EventMsg::Error`.
  - Do not add or update V2 tests; V2 behavior is out of scope and remains unchanged.
  - Add assertions that the local compact request after remote failure does not contain remote failure messages, timeout warning text, fallback warning text, failed remote output, or failed remote `ContextCompaction` content.
  - Update the owned snapshot only if the intentionally changed behavior affects the existing pre-turn remote failure request-shape snapshot.
  - Do not run `just test` or `cargo test`; note in the completion report that tests were authored but intentionally not run at the user's request.
- **Verification commands:**
  - `timeout 30s git diff --check -- codex-rs/core/tests/suite/compact_remote.rs codex-rs/core/tests/suite/snapshots/all__suite__compact_remote__remote_pre_turn_compaction_failure_shapes.snap` from repo root; expected result: no whitespace errors.
  - Do not run test commands or mutating repo-wide commands. User explicitly requested no test runs, and formatting/lint/fix runs after integration in quick verification.
- **Completion report requirements:** Report changed test/snapshot files, new or renamed test names, the behaviors asserted, confirmation that tests were not run by user request, commands run, command results, and unresolved risks.

## Model Allocation

| Stage | Role | Model tier | Resolved model | Reasoning effort | Reason |
|-------|------|------------|----------------|------------------|--------|
| Plan review | Plan document reviewer | REVIEW | `gpt-5.5` | xhigh | Required by `simplepower:writing-plans`; checks plan completeness before approval. |
| Implementation Task A | `sp-impl` | NORMAL | `gpt-5.4-mini` | xhigh | Localized retry/timeout plumbing with clear API contract. |
| Implementation Task B | `sp-impl` | BEST | `gpt-5.5` | xhigh | High-risk behavior-shaping remote retry/failure semantics and clean-history guarantee. |
| Implementation Task C | `sp-impl` | BEST | `gpt-5.5` | xhigh | Cross-cutting routing changes for manual and auto compact. |
| Implementation Task D | `sp-impl` | NORMAL | `gpt-5.4-mini` | xhigh | Integration test updates against an explicit Interface Contract. |
| Quick verification | Quick verifier | FAST | `gpt-5.3-codex-spark` | high | Runs formatting/lint/build checks and may fix only tiny typo-level issues. |
| Final review/fix | Review+fix agent | REVIEW | `gpt-5.5` | xhigh | Required single final reviewer for the whole implementation before final verification. |

## Plan Review

Self-review checklist result:

- Design Summary: Captures the approved V1 remote-first design, 3 total attempts, 180-second timeout, visible warnings, local fallback, V2 exclusion, and no failed remote history in local compact.
- Interface Contract: Lists concrete APIs, filenames, command contracts, data shapes, behavior guarantees, and cross-task assumptions before File Ownership.
- File ownership: Every implied file is assigned to exactly one task; parallel tasks have non-overlapping write scopes.
- Task allocation: Every requirement maps to a task, every task has Contract inputs, and every task includes Serialization required.
- Aggregate parallel readiness: Tasks A, B, C, and D can dispatch in aggregate parallel because their coordination needs are captured by the Interface Contract and write scopes do not overlap.
- Visual aids: Present and consistent with the authoritative written sections.
- Model allocation: FAST/NORMAL/BEST/REVIEW choices match risk; model resolution precedence and resolved models are explicit; plan reviewer and final review+fix use REVIEW; quick verifier uses FAST.
- Review allocation: Exactly one REVIEW-tier review+fix agent is planned after quick verification.
- Commit policy: Exactly three future coordinator checkpoints are present; non-coordinator roles must not commit.
- Scratch refs: Coordinator-only scratch refs use `refs/simplepower/scratch/<run-id>/` and include creation, diff handoff, cleanup, preservation, and final cleanup check guidance.
- Verification: Quick and final commands are concrete and use `timeout`; tests are not run because the user explicitly requested no test runs.
- Approved path enforcement: The plan does not authorize route changes, skipped review, skipped formatting/lint/build verification, scope reduction, or placeholder implementation.

Before first review, the coordinator creates `refs/simplepower/scratch/<run-id>/plan-review/before` for this plan file using the temporary-index pattern. The initial review prompt must include the saved plan path, approved brainstorming design context, scratch run id, and `plan-review/before` ref.

If the reviewer reports blocking issues, the coordinator fixes this plan, reruns the focused self-review checks for changed categories, creates `refs/simplepower/scratch/<run-id>/plan-review/after-<n>`, and sends the same reviewer this diff command:

```bash
git diff refs/simplepower/scratch/<run-id>/plan-review/before refs/simplepower/scratch/<run-id>/plan-review/after-<n> -- docs/simplepower/plans/2026-06-08-remote-compact-timeout-fallback.md
```

For further revisions, compare the last `after-<n>` ref to the next `after-<n+1>` ref. If a needed scratch ref is missing, stop the review loop before relying on the missing anchor.

The REVIEW-tier plan reviewer must perform the assigned review directly in the current worker. Do not run Codex CLI. Do not spawn subagents. Do not invoke Simple Power skills. Do not restart execution. Do not reroute the workflow.

After the plan reviewer approves, ask the user for combined approval of the reviewed plan, model/task allocation, and immediate current-session execution. The accepted plan checkpoint commit happens only after that combined approval. Workers and reviewers must not create this commit.

## Quick Verification

The quick verifier runs after all file-edit workers complete and before the coordinator creates the quick-verified implementation checkpoint.

Before dispatching the quick verifier, the coordinator creates `refs/simplepower/scratch/<run-id>/quick-verifier/before` for the approved implementation file list:

```bash
codex-rs/codex-api/src/endpoint/session.rs
codex-rs/codex-api/src/endpoint/compact.rs
codex-rs/core/src/client.rs
codex-rs/core/src/compact_remote.rs
codex-rs/core/src/remote_compact_fallback.rs
codex-rs/core/src/lib.rs
codex-rs/core/src/compact.rs
codex-rs/core/src/session/turn.rs
codex-rs/core/src/tasks/compact.rs
codex-rs/core/tests/suite/compact_remote.rs
codex-rs/core/tests/suite/snapshots/all__suite__compact_remote__remote_pre_turn_compaction_failure_shapes.snap
```

Quick verification commands:

- `timeout 600s just fmt` from repo root after all file edits. Expected result: formatting succeeds and no unrelated files are reformatted.
- `timeout 1800s just fix -p codex-api -p codex-core` from repo root after formatting. Expected result: clippy fix completes for both changed crates without errors.
- `timeout 3600s just build-for-release` from repo root after `just fix`. Expected result: Bazel release build succeeds and produces a `codex` binary under `bazel-bin`.

No test command is run in quick verification because the user explicitly requested no test runs. This is an approved exception to the usual local test command; Task D still updates test coverage for later CI/manual runs.

The quick verifier may fix only tiny typo-level errors discovered while running quick checks. Any behavior change, structural edit, test rewrite, public interface change, or unclear issue must be reported to the coordinator instead of fixed by the quick verifier. If tiny fixes are made, the coordinator creates `refs/simplepower/scratch/<run-id>/quick-verifier/after` and inspects:

```bash
git diff refs/simplepower/scratch/<run-id>/quick-verifier/before refs/simplepower/scratch/<run-id>/quick-verifier/after -- <approved-files>
```

After the quick-verified implementation checkpoint succeeds, delete that run's `quick-verifier` scratch refs. If the checkpoint fails or the workflow stops before the checkpoint, preserve the refs and report the manual cleanup command.

## Final Review And Fix

After the coordinator checkpoint for the quick-verified implementation, dispatch one REVIEW-tier review+fix agent. That agent reviews and fixes the whole implementation against the accepted plan, file ownership, approved path enforcement, aggregate parallel dispatch semantics, and verification requirements.

Before dispatching the review+fix agent, the coordinator creates `refs/simplepower/scratch/<run-id>/review-fix/before` for the approved implementation file list. If the review+fix agent edits files, the coordinator creates `refs/simplepower/scratch/<run-id>/review-fix/after` after those edits and before final verification, then inspects or hands off:

```bash
git diff refs/simplepower/scratch/<run-id>/review-fix/before refs/simplepower/scratch/<run-id>/review-fix/after -- <approved-files>
```

The review+fix agent may edit files within the plan's approved file ownership when fixing issues it finds. It must report changed files, commands run, results, remaining risks, and any unresolved deviations that require user approval. It must not commit.

The REVIEW-tier review+fix agent must perform the assigned review and fixes directly in the current worker. Do not run Codex CLI. Do not spawn subagents. Do not invoke Simple Power skills. Do not restart execution. Do not reroute the workflow. If no file changes happen during review+fix, omit the `review-fix/after` ref.

After the final checkpoint succeeds, delete that run's `review-fix` scratch refs. If the checkpoint fails or the workflow stops before the checkpoint, preserve the refs and report the manual cleanup command.

## Commit Checkpoints

Exactly three future coordinator commit checkpoints are allowed:

1. Accepted plan checkpoint: after the REVIEW-tier plan reviewer approves and the user gives combined approval for the reviewed plan, model/task allocation, and immediate current-session execution, and before invoking `simplepower:subagent-driven-development`.
2. Quick-verified implementation checkpoint: after all `sp-impl` file edits complete and quick verification passes.
3. Final checkpoint: after the REVIEW-tier review+fix agent completes and final verification passes.

Workers, plan reviewers, quick verifiers, review+fix agents, and individual tasks must not commit. Do not include worker-owned commits or per-task commits.

Scratch refs are the only allowed temporary review anchors. They are coordinator-owned, local-only, and not accepted checkpoint commits. They must be deleted after the successful checkpoint for their phase or preserved and reported for manual cleanup if the workflow stops or the checkpoint commit fails.

## Current-Session Auto-Dispatch

The saved plan is the execution artifact. Do not write a project-local implementation JSON artifact. The current branch is `fix/remote-compact-timeout-fallback`; the coordinator must verify it is still on that branch before the accepted plan checkpoint.

After the plan reviewer approves, ask the user for one combined approval that covers:

- The reviewed plan.
- The model/task allocation.
- Immediate current-session execution.

If the user requests changes, update this plan, rerun focused self-review checks for changed categories, create the next `plan-review/after-<n>` scratch ref, and send the revised plan back to the same reviewer with the concrete scratch-ref `git diff` command when review approval must be refreshed. Do not create the accepted plan checkpoint until the user gives combined approval.

After combined approval, the coordinator creates the accepted plan checkpoint commit, deletes the successful `plan-review` scratch refs, then immediately invokes `simplepower:subagent-driven-development` in the current session with this instruction:

```text
Execute `docs/simplepower/plans/2026-06-08-remote-compact-timeout-fallback.md` with aggregate parallel implementation from the approved Interface Contract. Use the approved FAST/NORMAL/BEST/REVIEW model allocation. Dispatch all non-conflicting `sp-impl` file-edit workers whose coordination needs are satisfied by their Contract inputs, run the quick FAST-tier verifier with formatting, lint/fix, and release build commands and their timeouts after all workers finish, commit the quick-verified implementation, then run one REVIEW-tier review+fix agent, final verification, and final commit. Do not run tests; the user explicitly requested no test runs.
```

## Verification

Final verification runs only after the REVIEW-tier review+fix agent has completed.

Final verification commands:

- `timeout 600s just fmt` from repo root. Run after review+fix. Expected result: formatting succeeds.
- `timeout 1800s just fix -p codex-api -p codex-core` from repo root. Run after final formatting. Expected result: clippy fix completes for both changed crates without errors. Do not re-run tests after `fix`.
- `timeout 3600s just build-for-release` from repo root. Run after `just fix`. Expected result: Bazel release build succeeds.
- `find bazel-bin -type f -name codex -perm -111 | sort` from repo root. Run after release build. Expected result: at least one executable `codex` binary path is listed; select the release binary produced by `//codex-rs/cli:release_binaries`.

No `just test` or `cargo test` command is run in final verification because the user explicitly requested no test runs. If final verification fails, report the exact failing command and do not deploy.

After successful final verification and final checkpoint, deploy the release binary to the requested hosts:

```bash
for host in fpga01 axel backup desk office; do
  ssh "$host" 'mkdir -p ~/.local/bin' &&
    scp <release-codex-binary> "$host:~/.local/bin/codex"
done
```

Expected deployment result: each host receives the release binary at `~/.local/bin/codex`. Any SSH/SCP failure must be reported with the host name and failing command.

Final reporting must include:

- Current branch and final commit hash.
- Whether tests were intentionally not run by user request.
- Release build command and result.
- Release binary path copied.
- Per-host deployment result for `fpga01`, `axel`, `backup`, `desk`, and `office`.
- Cleanup check output for:

```bash
git for-each-ref --format='%(refname)' "refs/simplepower/scratch/<run-id>"
```

If the final checkpoint succeeds, no scratch refs for that run should remain after phase cleanup. If the workflow stops because of user direction, a blocker, or a failed checkpoint commit, preserve remaining scratch refs and report:

```bash
git for-each-ref --format='%(refname)' "refs/simplepower/scratch/<run-id>" | while read -r ref; do git update-ref -d "$ref"; done
```
