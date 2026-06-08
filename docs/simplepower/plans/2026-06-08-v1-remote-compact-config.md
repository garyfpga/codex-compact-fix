# V1 Remote Compact Config Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `simplepower:subagent-driven-development` for aggregate parallel implementation. Dispatch all non-conflicting `sp-impl` file-edit workers whose coordination needs are satisfied by the approved Interface Contract, run the quick verifier after all workers finish, commit the quick-verified implementation, then run one REVIEW-tier review+fix agent before final verification and final commit.

**Goal:** Make V1 remote compact report more diagnostic failures, use `[remote_compact]` settings for attempts, per-attempt timeout, and TCP keepalive, then build and deploy the release binary as requested.

**Design Summary:** This change applies only to the V1 `/responses/compact` path used when `remote_compaction_v2` is disabled. Add top-level config:
```toml
[remote_compact]
max_attempts = 3
attempt_timeout_sec = 180
tcp_keepalive_interval_ms = 1000
```
Absent fields use those same defaults. V1 remote compact warnings must distinguish timeout from unexpected HTTP response, transport/network/build error, and stream/protocol/body parse failure, with bounded response details for bot-check or Cloudflare-like replies. V2 routing, retry, timeout, and failure behavior stay unchanged. Implementation must start on a new feature branch. After implementation, update `/home/gary/.codex/config.toml` with the new block, run no test command, run a Rust release compile, copy the binary to `/home/gary/codex`, and kill remote `codex` processes before copying the binary to `fpga01`, `axel`, `office`, `backup`, and `desk`.

**Architecture:** Add a small `RemoteCompactConfigToml` input shape and an effective `RemoteCompactConfig` in `codex-core` so `TurnContext` can provide resolved values to V1 compact code. Thread the effective settings through `CompactConversationRequestSettings` to the V1 compact request, and add a compact-specific reqwest client construction path that preserves the normal Codex headers, proxy policy, ChatGPT Cloudflare cookie store, and custom CA handling while applying the configured TCP keepalive interval. The Interface Contract below gives each worker exact APIs and file boundaries so config, transport, runtime warning behavior, and tests can be implemented in aggregate parallel.

**Tech Stack:** Rust async/Tokio, `codex-core`, `codex-config`, `codex-login`, `codex-api` request plumbing, reqwest 0.12 TCP keepalive builder options, existing `EventMsg::Warning`, existing core integration tests.

**Model Allocation:** FAST/NORMAL/BEST/REVIEW tiers are assigned below. Resolve each tier by explicit user override, quoted assignment in project root AGENTS.md, process environment variable, then built-in default. The project root AGENTS.md lookup reads only `<repo>/AGENTS.md`, not nested AGENTS.md files or repo-wide grep. FAST defaults to `SIMPLEPOWER_FAST_MODEL` (`gpt-5.3-codex-spark-high` when unset), NORMAL defaults to `SIMPLEPOWER_NORMAL_MODEL` (`gpt-5.4-mini-high` when unset), BEST defaults to `SIMPLEPOWER_BEST_MODEL` (`gpt-5.5-high` when unset), and REVIEW defaults to `SIMPLEPOWER_REVIEW_MODEL` (`gpt-5.5-xhigh` when unset). The plan reviewer is a REVIEW-tier plan reviewer, and the final review+fix agent is a REVIEW-tier review+fix agent. The quick verifier uses the FAST tier by default, resolving to `model="gpt-5.3-codex-spark"` and `reasoning_effort="high"` unless `SIMPLEPOWER_FAST_MODEL` is overridden.

Resolved tiers for this run from process environment, with no current-request model override and no quoted model assignments found in project-root `AGENTS.md`: FAST = `model="gpt-5.3-codex-spark"`, `reasoning_effort="high"`; NORMAL = `model="gpt-5.4-mini"`, `reasoning_effort="xhigh"`; BEST = `model="gpt-5.5"`, `reasoning_effort="xhigh"`; REVIEW = `model="gpt-5.5"`, `reasoning_effort="xhigh"`.

**Commit Policy:** The coordinator commits after the reviewed plan, allocation, and immediate current-session execution receive combined approval, after all file edits and quick verification complete before final review, and after final review/fix plus final verification. Workers, plan reviewers, quick verifiers, and review+fix agents must not commit. No per-task commits. Coordinator-owned temporary scratch refs under `refs/simplepower/scratch/<run-id>/...` may be created only as local review diff anchors; they are not accepted history commits, not pushed, not merged, not rebased, and must be cleaned up after successful checkpoints or reported for manual cleanup on blockers or failed checkpoints.

---

## Interface Contract

1. Branch and checkpoint contract:
   - After plan review approval and user combined approval, before the accepted plan checkpoint commit, the coordinator creates a new feature branch from the current `fix/remote-compact-timeout-fallback` branch state:
     ```bash
     git switch -c feature/v1-remote-compact-config
     ```
   - The accepted plan checkpoint commit, quick-verified implementation checkpoint commit, and final checkpoint commit all occur on `feature/v1-remote-compact-config`.
   - Workers, plan reviewers, quick verifiers, and review+fix agents must not create branches or commits.

2. Config TOML contract:
   - Add `RemoteCompactConfigToml` as a serializable/deserializable TOML shape with `JsonSchema` and `#[schemars(deny_unknown_fields)]`.
   - Add this exact top-level optional field to `ConfigToml`:
     ```rust
     #[serde(default)]
     pub remote_compact: Option<RemoteCompactConfigToml>,
     ```
   - TOML keys:
     - `remote_compact.max_attempts: Option<i64>`
     - `remote_compact.attempt_timeout_sec: Option<i64>`
     - `remote_compact.tcp_keepalive_interval_ms: Option<i64>`
   - Defaults when the table or field is absent:
     - `max_attempts = 3`
     - `attempt_timeout_sec = 180`
     - `tcp_keepalive_interval_ms = 1000`
   - Validation:
     - `remote_compact.max_attempts` must be at least `1` and at most `20`.
     - `remote_compact.attempt_timeout_sec` must be at least `1` and at most `3600`.
     - `remote_compact.tcp_keepalive_interval_ms` must be at least `1` and at most `60000`.
   - Config build errors use `std::io::ErrorKind::InvalidInput` and exact message shape:
     - `remote_compact.max_attempts must be at least 1`
     - `remote_compact.max_attempts must be at most 20`
     - `remote_compact.attempt_timeout_sec must be at least 1`
     - `remote_compact.attempt_timeout_sec must be at most 3600`
     - `remote_compact.tcp_keepalive_interval_ms must be at least 1`
     - `remote_compact.tcp_keepalive_interval_ms must be at most 60000`
   - Run `cd codex-rs && timeout 600s just write-config-schema` after changing `ConfigToml`, and include the generated `codex-rs/core/config.schema.json`.

3. Effective config contract:
   - Add an effective config type in `codex-rs/core/src/config/mod.rs`:
     ```rust
     #[derive(Debug, Clone, Copy, PartialEq, Eq)]
     pub struct RemoteCompactConfig {
         pub max_attempts: u64,
         pub attempt_timeout: std::time::Duration,
         pub tcp_keepalive_interval: std::time::Duration,
     }
     ```
   - `Config` gains:
     ```rust
     pub remote_compact: RemoteCompactConfig,
     ```
   - `ConfigBuilder` resolves `cfg.remote_compact` into `Config::remote_compact` before constructing `Config`.
   - All tests and struct literals updated by the implementation must use the effective default unless a test intentionally overrides a field.

4. Compact request settings and keepalive contract:
   - `CompactConversationRequestSettings` in `codex-rs/core/src/client.rs` gains:
     ```rust
     pub(crate) tcp_keepalive_interval: Duration,
     ```
   - The V1 compact request uses `settings.request_timeout` as the per-attempt timeout and builds the `ReqwestTransport` with a compact-specific reqwest client that applies `settings.tcp_keepalive_interval`.
   - Add a public helper in `codex-rs/login/src/auth/default_client.rs`:
     ```rust
     pub fn build_reqwest_client_with_tcp_keepalive_interval(
         tcp_keepalive_interval: Duration,
     ) -> reqwest::Client
     ```
   - The helper must preserve the same default headers, sandbox proxy policy, ChatGPT Cloudflare cookie store, and custom CA handling as `build_reqwest_client()`. It must set:
     ```rust
     builder = builder
         .tcp_keepalive(Some(tcp_keepalive_interval))
         .tcp_keepalive_interval(Some(tcp_keepalive_interval));
     ```
   - If the custom client builder fails, it logs the failure and returns the same kind of fallback client as `build_reqwest_client()`, with the keepalive options applied to that fallback builder when possible.
   - Do not change ordinary Codex HTTP traffic, streaming Responses requests, WebSocket traffic, V2 remote compaction, memories, images, search, or non-compact endpoints.

5. V1 runtime contract:
   - `codex-rs/core/src/compact_remote.rs` stops importing hardcoded attempt and timeout constants from `remote_compact_fallback`.
   - `run_remote_compaction_request_v1` reads:
     - `let total_attempts = turn_context.config.remote_compact.max_attempts;`
     - `let attempt_timeout = turn_context.config.remote_compact.attempt_timeout;`
     - `let tcp_keepalive_interval = turn_context.config.remote_compact.tcp_keepalive_interval;`
   - Each visible V1 attempt still maps to exactly one `/responses/compact` HTTP request by keeping the existing explicit retry policy with `max_attempts: 0` and retry flags disabled.
   - `CompactConversationRequestSettings` passed to the V1 compact endpoint uses the effective timeout and keepalive interval.
   - `remote_compact_fallback.rs` fallback warning uses the configured attempt count:
     `Remote compact failed after {max_attempts} attempts; falling back to local compact.`
   - Existing clean-history fallback behavior remains unchanged: failed remote attempts must not leave failed remote artifacts in local fallback model-visible history.
   - `CodexErr::Interrupted` and `CodexErr::TurnAborted` still abort and do not fall back.

6. V1 warning reporting contract:
   - Failed V1 attempts still emit `EventMsg::Warning(WarningEvent { message })`.
   - Timeout warning:
     `Remote compact attempt {attempt_number}/{total_attempts} timed out after {seconds}s{action}`
   - Unexpected HTTP warning for `CodexErr::UnexpectedStatus(err)`:
     `Remote compact attempt {attempt_number}/{total_attempts} got unexpected HTTP response: {err}{action}`
   - Transport/network/build or retryable stream warning for `CodexErr::Stream(message, _)`, `CodexErr::ConnectionFailed(_)`, or `CodexErr::ResponseStreamFailed(_)`:
     `Remote compact attempt {attempt_number}/{total_attempts} failed with transport or stream error: {err}{action}`
   - Protocol/body parse warning for `CodexErr::Json(_)`:
     `Remote compact attempt {attempt_number}/{total_attempts} failed to parse remote compact response: {err}{action}`
   - Other non-timeout warning:
     `Remote compact attempt {attempt_number}/{total_attempts} failed: {err}{action}`
   - `action` remains `; retrying remote compact.` when another attempt remains, otherwise `.`.
   - Unexpected HTTP details are bounded by the existing `UnexpectedResponseError` display implementation, which already caps body display and includes status, URL, request id, `cf-ray`, and identity error metadata when present.

7. Personal config update contract:
   - After source code supports the new config and before release compile/deploy, update `/home/gary/.codex/config.toml`.
   - If no `[remote_compact]` table exists, append exactly:
     ```toml
     [remote_compact]
     max_attempts = 3
     attempt_timeout_sec = 180
     tcp_keepalive_interval_ms = 1000
     ```
   - If `[remote_compact]` already exists, update or add those three keys in that table and preserve unrelated keys.
   - `/home/gary/.codex/config.toml` is user-local state, is not part of git, and must not be included in checkpoint commits.

8. Verification, release compile, and deployment contract:
   - Do not run `just test`, `cargo test`, `cargo nextest`, or any other test command in this execution. This follows the user's approved "Rust compile without test" instruction.
   - Required formatting/schema/lint-fix commands:
     ```bash
     cd codex-rs && timeout 600s just fmt
     cd codex-rs && timeout 600s just write-config-schema
     cd codex-rs && timeout 1800s just fix -p codex-core -p codex-config -p codex-login
     ```
   - Required release compile without tests:
     ```bash
     cd codex-rs && timeout 3600s cargo build --release -p codex-cli
     test -x codex-rs/target/release/codex
     ```
   - Required local copy:
     ```bash
     cp codex-rs/target/release/codex /home/gary/codex
     ```
   - Required remote deployment, after local release binary exists:
     ```bash
     for host in fpga01 axel office backup desk; do
       ssh "$host" 'killall codex || true; mkdir -p ~/.local/bin' &&
       scp codex-rs/target/release/codex "$host:~/.local/bin/codex"
     done
     ```
   - If any deployment command fails, stop and report the host and command failure instead of using another destination or skipping a host.

## File Ownership

| File | Owner task | Change type | Responsibility | Parallel safety notes |
|------|------------|-------------|----------------|-----------------------|
| `docs/simplepower/plans/2026-06-08-v1-remote-compact-config.md` | Coordinator planning | create | Authoritative implementation plan. | Coordinator-owned; not edited by implementation workers unless reviewer asks for plan revision before combined approval. |
| `codex-rs/config/src/types.rs` | Task A: Remote compact config model | modify | Define `RemoteCompactConfigToml` input type. | Parallel with Tasks B, C, and D through Interface Contract entries 2 and 3. |
| `codex-rs/config/src/config_toml.rs` | Task A: Remote compact config model | modify | Add top-level `remote_compact` field to `ConfigToml`. | Parallel with Tasks B, C, and D through Interface Contract entries 2 and 3. |
| `codex-rs/core/src/config/mod.rs` | Task A: Remote compact config model | modify | Add effective `RemoteCompactConfig`, defaults, validation, and `Config` field wiring. | Parallel with Tasks B, C, and D through Interface Contract entries 2 and 3. |
| `codex-rs/core/src/config/config_tests.rs` | Task A: Remote compact config model | modify | Add config default, override, and invalid value coverage. | Parallel with Tasks B, C, and D through Interface Contract entries 2 and 3. |
| `codex-rs/core/config.schema.json` | Task A: Remote compact config model | generated | Regenerate JSON schema after `ConfigToml` changes. | Generated by Task A only. |
| `codex-rs/login/src/auth/default_client.rs` | Task B: Compact keepalive HTTP client | modify | Add compact-specific reqwest client helper with TCP keepalive. | Parallel with Tasks A, C, and D through Interface Contract entry 4. |
| `codex-rs/login/src/auth/default_client_tests.rs` | Task B: Compact keepalive HTTP client | modify | Add focused construction/fallback coverage where practical. | Parallel with Tasks A, C, and D through Interface Contract entry 4. |
| `codex-rs/core/src/client.rs` | Task B: Compact keepalive HTTP client | modify | Thread `tcp_keepalive_interval` through compact request settings and V1 compact request transport. | Parallel with Tasks A, C, and D through Interface Contract entry 4. |
| `codex-rs/core/src/compact_remote.rs` | Task C: V1 runtime settings and warnings | modify | Use configured attempts/timeout/keepalive and add failure-category warning text. | Parallel with Tasks A, B, and D through Interface Contract entries 5 and 6. |
| `codex-rs/core/src/remote_compact_fallback.rs` | Task C: V1 runtime settings and warnings | modify | Remove hardcoded fallback warning count and use configured attempt count. | Parallel with Tasks A, B, and D through Interface Contract entries 5 and 6. |
| `codex-rs/core/tests/suite/compact_remote.rs` | Task D: V1 remote compact integration coverage | modify | Add/update V1 fallback tests for configured attempts, timeout wording, unexpected response details, and clean local fallback history. | Parallel with Tasks A, B, and C because tests target the approved Interface Contract. |
| `/home/gary/.codex/config.toml` | Task E: Personal config update | modify | Add or update the `[remote_compact]` block with approved values. | Serialization required because user-local config should be changed only after source support exists. Not committed. |

## Visual Aids

```text
V1 manual /compact or auto compact
        |
        v
remote_compaction_v2 enabled?
        | yes -> existing V2 path unchanged
        | no
        v
read Config.remote_compact
        |
        v
for attempt in 1..=max_attempts:
  POST /responses/compact
    timeout = attempt_timeout_sec
    TCP keepalive interval = tcp_keepalive_interval_ms
    hidden HTTP retries = 0
        |
        +-- success -> install remote compact history
        |
        +-- failure -> warning with category and bounded details
        |
        v
after all attempts fail:
  restore clean pre-remote history
  warn fallback using configured max_attempts
  run existing local compact fallback
```

## Implementation Tasks

### Task A: Remote Compact Config Model

- Goal: Add the `[remote_compact]` TOML input shape, effective defaults, validation, and schema output.
- Contract inputs: Interface Contract entries 2, 3, and 8.
- Serialization required: No. The Interface Contract defines the public config shape and defaults.
- Write scope:
  - `codex-rs/config/src/types.rs`
  - `codex-rs/config/src/config_toml.rs`
  - `codex-rs/core/src/config/mod.rs`
  - `codex-rs/core/src/config/config_tests.rs`
  - `codex-rs/core/config.schema.json`
- Parallel: Yes, compatible with Tasks B, C, and D.
- Risk: Medium, because config shape changes affect config loading and schema.
- Model tier: BEST, resolved `model="gpt-5.5"`, `reasoning_effort="xhigh"`.
- Worker role: `sp-impl`.
- Outputs and file-level responsibilities:
  - `RemoteCompactConfigToml` in `codex-rs/config/src/types.rs`.
  - `ConfigToml.remote_compact` in `codex-rs/config/src/config_toml.rs`.
  - Effective `RemoteCompactConfig`, defaults, validation, and `Config.remote_compact` in `codex-rs/core/src/config/mod.rs`.
  - Tests in `codex-rs/core/src/config/config_tests.rs` that cover absent/default, override, zero, negative, and too-large values.
  - Regenerated `codex-rs/core/config.schema.json`.
- Implementation steps:
  1. Add `RemoteCompactConfigToml` with the three `Option<i64>` fields named in Interface Contract entry 2.
  2. Import and add `remote_compact: Option<RemoteCompactConfigToml>` to `ConfigToml`.
  3. Add default constants in `codex-rs/core/src/config/mod.rs`: `3`, `180`, and `1000`, plus hard caps `20`, `3600`, and `60000`.
  4. Add `RemoteCompactConfig` effective type and a resolver that returns defaults when values are absent.
  5. Validate values after resolution or before constructing `Config`, using exact error messages from Interface Contract entry 2.
  6. Thread `remote_compact` into the `Config` struct construction.
  7. Add focused config tests near existing compact or numeric config tests.
  8. Run `cd codex-rs && timeout 600s just write-config-schema`.
- Verification commands:
  - `cd codex-rs && timeout 600s just write-config-schema`
  - `cd codex-rs && timeout 600s cargo check -p codex-config -p codex-core`
- Completion report requirements: changed files, whether schema was regenerated, commands run and results, and any config validation risk.

### Task B: Compact Keepalive HTTP Client

- Goal: Add a compact-specific reqwest client helper with TCP keepalive and thread the keepalive duration through V1 compact request settings.
- Contract inputs: Interface Contract entry 4.
- Serialization required: No. The helper signature and settings field are defined by the Interface Contract.
- Write scope:
  - `codex-rs/login/src/auth/default_client.rs`
  - `codex-rs/login/src/auth/default_client_tests.rs`
  - `codex-rs/core/src/client.rs`
- Parallel: Yes, compatible with Tasks A, C, and D.
- Risk: High, because transport construction must preserve existing default headers, proxy, cookie store, and custom CA behavior.
- Model tier: BEST, resolved `model="gpt-5.5"`, `reasoning_effort="xhigh"`.
- Worker role: `sp-impl`.
- Outputs and file-level responsibilities:
  - `build_reqwest_client_with_tcp_keepalive_interval(Duration) -> reqwest::Client`.
  - `CompactConversationRequestSettings.tcp_keepalive_interval`.
  - V1 compact `ReqwestTransport` constructed from the new helper.
- Implementation steps:
  1. Refactor the shared body of `try_build_reqwest_client()` only as needed so ordinary default client behavior remains identical.
  2. Add the new keepalive helper and ensure fallback client construction applies keepalive options when possible.
  3. Update `codex-rs/core/src/client.rs` to import and use the helper only inside `compact_conversation_history`.
  4. Add `tcp_keepalive_interval` to `CompactConversationRequestSettings` and update all struct literals.
  5. Add a focused test in `default_client_tests.rs` that at least exercises construction of the keepalive helper without panicking; if builder internals cannot be inspected, report that limitation.
- Verification commands:
  - `cd codex-rs && timeout 600s cargo check -p codex-login -p codex-core`
- Completion report requirements: changed files, commands run and results, and whether keepalive behavior is directly inspectable in tests.

### Task C: V1 Runtime Settings And Warning Categories

- Goal: Replace hardcoded V1 attempts/timeout with effective config and add categorized V1 failure warnings.
- Contract inputs: Interface Contract entries 5 and 6.
- Serialization required: No. Runtime APIs and warning strings are defined by the Interface Contract.
- Write scope:
  - `codex-rs/core/src/compact_remote.rs`
  - `codex-rs/core/src/remote_compact_fallback.rs`
- Parallel: Yes, compatible with Tasks A, B, and D.
- Risk: High, because this changes user-visible warning behavior and retry loop control.
- Model tier: BEST, resolved `model="gpt-5.5"`, `reasoning_effort="xhigh"`.
- Worker role: `sp-impl`.
- Outputs and file-level responsibilities:
  - V1 request loop uses configured `max_attempts`, `attempt_timeout`, and `tcp_keepalive_interval`.
  - Existing zero-hidden-retry behavior remains intact.
  - Warning messages match Interface Contract entry 6.
  - Fallback warning uses configured attempt count.
- Implementation steps:
  1. Remove dependency on `REMOTE_COMPACT_TOTAL_ATTEMPTS` and `REMOTE_COMPACT_ATTEMPT_TIMEOUT` from `compact_remote.rs`.
  2. Read effective remote compact settings from `turn_context.config.remote_compact`.
  3. Pass `attempt_timeout` and `tcp_keepalive_interval` into `CompactConversationRequestSettings`.
  4. Replace `send_remote_compaction_attempt_warning` message construction with a small exhaustive classifier over relevant `CodexErr` variants. Avoid wildcard arms where practical.
  5. Update `remote_compact_fallback.rs` constants and fallback warning so the count is dynamic and still uses the exact message shape.
  6. Preserve interruption and clean-history fallback behavior.
- Verification commands:
  - `cd codex-rs && timeout 600s cargo check -p codex-core`
- Completion report requirements: changed files, commands run and results, exact warning categories implemented, and any residual categorization ambiguity.

### Task D: V1 Remote Compact Integration Coverage

- Goal: Add or update focused integration coverage for the configured V1 behavior and warning text.
- Contract inputs: Interface Contract entries 2, 5, 6, and 8.
- Serialization required: No. Tests target the approved Interface Contract while implementation workers create APIs.
- Write scope:
  - `codex-rs/core/tests/suite/compact_remote.rs`
- Parallel: Yes, compatible with Tasks A, B, and C.
- Risk: Medium, because tests must coordinate with existing remote compact harness behavior.
- Model tier: NORMAL, resolved `model="gpt-5.4-mini"`, `reasoning_effort="xhigh"`.
- Worker role: `sp-impl`.
- Outputs and file-level responsibilities:
  - Test that configured `max_attempts = 2` produces exactly two V1 `/responses/compact` requests and fallback warning says `2 attempts`.
  - Test that configured `attempt_timeout_sec` appears in timeout warning text.
  - Test that unexpected HTTP response warning includes the categorized prefix and useful bounded response details such as status or `cf-ray` when mocked headers provide it.
  - Preserve clean local fallback history assertions for remote failure text and warning text.
- Implementation steps:
  1. Reuse existing `compact_remote.rs` helpers such as `collect_warnings_until_turn_complete`, compact endpoint mocks, and request assertions.
  2. Configure `harness` with `config.remote_compact` effective overrides once Task A API exists.
  3. Use existing mock response helpers for invalid JSON/protocol responses where possible.
  4. For unexpected HTTP response, mount a `/responses/compact` response with non-success status, `cf-ray`, and a short body that resembles an interstitial or bot-check response.
  5. Do not add tests for V2.
- Verification commands:
  - `cd codex-rs && timeout 600s cargo check -p codex-core`
  - Do not run `just test`, `cargo test`, or `cargo nextest`.
- Completion report requirements: changed files, commands run and results, which behaviors are covered, and any test gap caused by the no-test-run constraint.

### Task E: Personal Config Update

- Goal: Add the approved `[remote_compact]` block to `/home/gary/.codex/config.toml`.
- Contract inputs: Interface Contract entry 7.
- Serialization required: Yes. This must run after source support exists so the active config file does not contain unsupported keys while old code is still active.
- Write scope:
  - `/home/gary/.codex/config.toml`
- Parallel: No. It must run after Tasks A through D have landed in the worktree.
- Risk: Low, because this is a small user-local config edit outside git.
- Model tier: FAST, resolved `model="gpt-5.3-codex-spark"`, `reasoning_effort="high"`.
- Worker role: `sp-impl` only if dispatched after Tasks A through D; coordinator may also perform this exact edit after implementation support exists.
- Outputs and file-level responsibilities:
  - `/home/gary/.codex/config.toml` contains the approved `remote_compact` values.
  - The file is not added to git.
- Implementation steps:
  1. Read `/home/gary/.codex/config.toml`.
  2. If no `[remote_compact]` table exists, append the exact block from Interface Contract entry 7.
  3. If the table exists, update/add the three approved keys and preserve unrelated config.
  4. Report whether the table was appended or updated.
- Verification commands:
  - `timeout 30s rg -n "^\\[remote_compact\\]|^max_attempts = 3$|^attempt_timeout_sec = 180$|^tcp_keepalive_interval_ms = 1000$" /home/gary/.codex/config.toml`
- Completion report requirements: whether config was appended or updated, command result, and confirmation that the file is outside git.

## Model Allocation

| Stage | Role | Model tier | Resolved model | Reasoning effort | Reason |
|-------|------|------------|----------------|------------------|--------|
| Plan review | REVIEW-tier plan document reviewer | REVIEW | `gpt-5.5` | `xhigh` | Required by writing-plans; validates the full authoritative plan. |
| Task A | `sp-impl` config model worker | BEST | `gpt-5.5` | `xhigh` | Config shape, validation, effective runtime values, and generated schema are cross-cutting. |
| Task B | `sp-impl` transport worker | BEST | `gpt-5.5` | `xhigh` | HTTP client construction is behavior-shaping and must preserve custom CA/proxy/cookie behavior. |
| Task C | `sp-impl` runtime worker | BEST | `gpt-5.5` | `xhigh` | Retry loop control and user-visible warning behavior are high-impact. |
| Task D | `sp-impl` integration-test worker | NORMAL | `gpt-5.4-mini` | `xhigh` | Test edits are localized and target the approved contract. |
| Task E | `sp-impl` or coordinator user-config edit | FAST | `gpt-5.3-codex-spark` | `high` | Static user-local config edit after source support exists. |
| Quick verifier | FAST-tier quick verifier | FAST | `gpt-5.3-codex-spark` | `high` | Runs bounded formatting/schema/check commands and may fix only tiny typo-level issues. |
| Final review+fix | REVIEW-tier review+fix agent | REVIEW | `gpt-5.5` | `xhigh` | Required final whole-change review and fixes before final verification. |

## Plan Review

Self-review checklist result before dispatch:
- Design Summary: Captures V1-only scope, config keys/defaults, diagnostic warnings, keepalive, branch, no-test release compile, local copy, and remote deployment.
- Interface Contract: Defines concrete APIs, config shape, validation, warning strings, branch command, generated schema command, personal config update, and deployment commands.
- File Ownership: Every planned repo and user-local file has exactly one owner; parallel tasks have no overlapping write scopes.
- Task allocation: Every task has Contract inputs, Serialization required, write scope, verification commands, and completion requirements.
- Aggregate parallel readiness: Tasks A through D can run in parallel from the Interface Contract; Task E is serialized with a concrete user-local runtime reason.
- Visual aids: Included inline text flow supports the written contract and does not replace it.
- Model allocation: FAST/NORMAL/BEST/REVIEW tiers resolved from process environment after checking only project-root `AGENTS.md`.
- Review allocation: Includes one REVIEW-tier plan reviewer and one REVIEW-tier final review+fix agent.
- Commit policy: Exactly three coordinator checkpoint commits; no non-coordinator commits.
- Scratch refs: Uses coordinator-only refs under `refs/simplepower/scratch/<run-id>/` with phase cleanup and blocker preservation.
- Verification: Commands are concrete, use `timeout`, and intentionally omit tests per user instruction.
- Approved path enforcement: No alternate routes, skipped review, skipped release compile, skipped deployment, placeholder implementation, or docs-only substitute is authorized.

Before first plan review, coordinator creates:
```bash
SP_RUN_ID="${SP_RUN_ID:-$(date -u +%Y%m%d-%H%M%S)-$(git rev-parse --short HEAD)}"
SP_SCRATCH_PREFIX="refs/simplepower/scratch/$SP_RUN_ID"
SP_REF="$SP_SCRATCH_PREFIX/plan-review/before"
SP_TMP_INDEX="$(mktemp)"
GIT_INDEX_FILE="$SP_TMP_INDEX" git read-tree HEAD
GIT_INDEX_FILE="$SP_TMP_INDEX" git add -- docs/simplepower/plans/2026-06-08-v1-remote-compact-config.md
SP_TREE="$(GIT_INDEX_FILE="$SP_TMP_INDEX" git write-tree)"
SP_COMMIT="$(printf '%s\n' "simplepower scratch $SP_RUN_ID plan-review/before" | git commit-tree "$SP_TREE" -p HEAD)"
git update-ref "$SP_REF" "$SP_COMMIT"
rm -f "$SP_TMP_INDEX"
```

If the reviewer reports issues, coordinator edits this plan, creates `plan-review/after-<n>` with the same temporary-index pattern for this plan file, and returns the concrete diff command to the same reviewer:
```bash
git diff refs/simplepower/scratch/<run-id>/plan-review/before refs/simplepower/scratch/<run-id>/plan-review/after-1 -- docs/simplepower/plans/2026-06-08-v1-remote-compact-config.md
```

For later revisions, compare the previous `after` ref to the new `after` ref. Close the reviewer only after approval, unrecoverable interruption, or explicit user direction. If a scratch ref is missing, stop before relying on the diff anchor.

After reviewer approval, ask the user for one combined approval covering:
- the reviewed plan,
- the model/task allocation,
- immediate current-session execution.

After combined approval, the coordinator creates `feature/v1-remote-compact-config`, creates the accepted plan checkpoint commit on that branch, deletes `plan-review` scratch refs, and immediately invokes `simplepower:subagent-driven-development`.

## Quick Verification

Quick verification runs after Tasks A through E complete and before the quick-verified implementation checkpoint.

Before dispatching the quick verifier, coordinator creates `refs/simplepower/scratch/<run-id>/quick-verifier/before` for the approved implementation file list:
```text
codex-rs/config/src/types.rs
codex-rs/config/src/config_toml.rs
codex-rs/core/src/config/mod.rs
codex-rs/core/src/config/config_tests.rs
codex-rs/core/config.schema.json
codex-rs/login/src/auth/default_client.rs
codex-rs/login/src/auth/default_client_tests.rs
codex-rs/core/src/client.rs
codex-rs/core/src/compact_remote.rs
codex-rs/core/src/remote_compact_fallback.rs
codex-rs/core/tests/suite/compact_remote.rs
```

Quick verifier commands:
```bash
cd codex-rs && timeout 600s just fmt
cd codex-rs && timeout 600s just write-config-schema
cd codex-rs && timeout 900s cargo check -p codex-core -p codex-config -p codex-login
```

Expected result: formatting/schema generation complete and `cargo check` passes. Failure means implementation is not coherent enough for checkpoint or final review.

The quick verifier may fix only tiny typo-level errors discovered while running these commands. It must report behavior changes, structural edits, test rewrites, public interface changes, or unclear issues instead of fixing them. If it edits files, coordinator creates `quick-verifier/after` and inspects:
```bash
git diff refs/simplepower/scratch/<run-id>/quick-verifier/before refs/simplepower/scratch/<run-id>/quick-verifier/after -- <approved-files>
```

After quick verification passes, coordinator creates the quick-verified implementation checkpoint commit and deletes the `quick-verifier` scratch refs. If the checkpoint fails or the workflow stops, preserve refs and report the cleanup command from the Scratch Ref Review Anchors section of the writing-plans skill.

## Final Review And Fix

After the quick-verified implementation checkpoint, dispatch one REVIEW-tier review+fix agent with `model="gpt-5.5"` and `reasoning_effort="xhigh"`.

Before dispatch, coordinator creates `refs/simplepower/scratch/<run-id>/review-fix/before` for the same approved implementation file list used by quick verification.

The review+fix agent must:
- Review the implementation against this accepted plan, file ownership, approved path enforcement, aggregate parallel dispatch assumptions, and verification commands.
- Perform the assigned review and fixes directly in the current worker.
- Not run Codex CLI.
- Not spawn subagents.
- Not invoke Simple Power skills.
- Not restart execution.
- Not reroute the workflow.
- Not commit.
- Edit only approved implementation files if it fixes issues.
- Report changed files, commands run, results, remaining risks, and unresolved deviations requiring user approval.

If review+fix edits files, coordinator creates `review-fix/after` and inspects:
```bash
git diff refs/simplepower/scratch/<run-id>/review-fix/before refs/simplepower/scratch/<run-id>/review-fix/after -- <approved-files>
```

After review+fix and final verification pass, coordinator creates the final checkpoint commit and deletes `review-fix` scratch refs. If final checkpoint fails or workflow stops, preserve refs and report the manual cleanup command.

## Commit Checkpoints

Exactly three coordinator checkpoint commits are authorized:

1. Accepted plan checkpoint:
   - Occurs after REVIEW-tier plan reviewer approval and user combined approval.
   - Coordinator first runs `git switch -c feature/v1-remote-compact-config`.
   - Commit includes this plan file.
   - Happens before invoking `simplepower:subagent-driven-development`.

2. Quick-verified implementation checkpoint:
   - Occurs after all file-edit tasks complete and quick verification passes.
   - Commit includes repo files only.
   - Does not include `/home/gary/.codex/config.toml`.

3. Final checkpoint:
   - Occurs after REVIEW-tier review+fix completes and final verification plus deployment commands pass.
   - Commit includes repo files only.
   - Does not include `/home/gary/.codex/config.toml`.

Workers, plan reviewers, quick verifiers, review+fix agents, and individual tasks must not commit. Scratch refs are coordinator-owned local review anchors only and do not count as checkpoint commits.

## Current-Session Auto-Dispatch

After plan reviewer approval, ask the user for one combined approval for the reviewed plan, model/task allocation, and immediate current-session execution.

After combined approval, the coordinator creates the feature branch, accepted plan checkpoint commit, deletes successful `plan-review` scratch refs, then immediately invokes `simplepower:subagent-driven-development` in the current session with:

```text
Execute `docs/simplepower/plans/2026-06-08-v1-remote-compact-config.md` with aggregate parallel implementation from the approved Interface Contract. Use the approved FAST/NORMAL/BEST/REVIEW model allocation. Dispatch all non-conflicting `sp-impl` file-edit workers whose coordination needs are satisfied by their Contract inputs, run the quick FAST-tier verifier with lint/build checks and timeouts after all workers finish, commit the quick-verified implementation, then run one REVIEW-tier review+fix agent, final verification, deployment, and final commit. Do not run tests; use the approved Rust release compile command instead.
```

Do not create a separate implementation JSON artifact. Do not offer alternate execution routes.

## Verification

Final verification and deployment commands run after REVIEW-tier review+fix completes and before the final checkpoint commit.

1. Formatting:
   ```bash
   cd codex-rs && timeout 600s just fmt
   ```
   Expected result: exits `0`. Failure means formatting must be fixed before final checkpoint.

2. Config schema:
   ```bash
   cd codex-rs && timeout 600s just write-config-schema
   ```
   Expected result: exits `0` and generated schema is current. Failure means config/schema drift must be fixed before final checkpoint.

3. Lint/fix:
   ```bash
   cd codex-rs && timeout 1800s just fix -p codex-core -p codex-config -p codex-login
   ```
   Expected result: exits `0`. Do not rerun tests after this command. Failure means Clippy or fixable lint issues remain.

4. Rust release compile without tests:
   ```bash
   cd codex-rs && timeout 3600s cargo build --release -p codex-cli
   test -x codex-rs/target/release/codex
   ```
   Expected result: exits `0` and release binary exists. Failure means the release binary is not ready for copy/deploy.

5. Local copy:
   ```bash
   cp codex-rs/target/release/codex /home/gary/codex
   ```
   Expected result: exits `0`. Failure means local requested copy is incomplete.

6. Remote deployment:
   ```bash
   for host in fpga01 axel office backup desk; do
     ssh "$host" 'killall codex || true; mkdir -p ~/.local/bin' &&
     scp codex-rs/target/release/codex "$host:~/.local/bin/codex"
   done
   ```
   Expected result: exits `0` for all hosts. Failure means deployment is incomplete; stop and report the failed host/command.

7. Scratch-ref cleanup check:
   ```bash
   git for-each-ref --format='%(refname)' "refs/simplepower/scratch/<run-id>"
   ```
   Expected result after successful final checkpoint and cleanup: no refs remain for this run. If workflow stopped because of user direction, blocker, or failed checkpoint commit, preserve remaining refs and report:
   ```bash
   git for-each-ref --format='%(refname)' "refs/simplepower/scratch/<run-id>" | while read -r ref; do git update-ref -d "$ref"; done
   ```

No test commands are part of quick or final verification for this plan because the approved user instruction is to run Rust release compile without tests.

## Approved Path Enforcement

This accepted plan is authoritative once approved. Do not authorize backup routes, scope reduction, docs-only substitutes, placeholder implementations, skipped review, skipped release compile, skipped local copy, skipped remote host, alternate deployment destinations, test substitution, or execution-route changes without fresh explicit user approval at the moment the deviation is needed.

If the approved path is blocked, unsafe, underspecified, or mismatched with the codebase during execution, stop, report the exact mismatch and current status, and ask the user before changing approach.
