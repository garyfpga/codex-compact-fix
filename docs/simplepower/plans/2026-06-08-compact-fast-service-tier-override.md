# Compact Fast Service Tier Override Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `simplepower:subagent-driven-development` for aggregate parallel implementation. Dispatch all non-conflicting `sp-impl` file-edit workers whose coordination needs are satisfied by the approved Interface Contract, run the quick verifier after all workers finish, commit the quick-verified implementation, then run one REVIEW-tier review+fix agent before final verification and final commit.

**Goal:** Make V1 remote-first compaction use the fast service tier for compact work, including backup local fallback, then return normal sampling to the original service tier before building and deploying the release binary.

**Design Summary:** Implement the approved scoped compact override instead of mutating persisted config or session settings. When V1 remote-first compaction runs and `remote_compaction_v2` is disabled, the coordinator resolves a compact-only service tier of `priority` when the active model supports it and the request auth mode can use service tiers. All V1 `/responses/compact` attempts use that compact tier, and if remote attempts fall back to local compaction, the backup local compact request uses the same compact tier. The original `TurnContext` is not mutated, so the following normal sampling request naturally uses the original tier. If `priority` is unsupported, compact keeps the original effective tier. V2 remote compaction stays unchanged. After source verification, compile the release binary, copy it to `/home/gary/codex`, and deploy it to `fpga01`, `axel`, `office`, `backup`, and `desk`.

**Architecture:** Add compact-tier resolution and named compact run settings in `codex-core`, then thread the resolved compact tier through V1 remote compact and local fallback without changing session config. The Interface Contract below names the exact APIs and behavior so code plumbing and integration tests can run in aggregate parallel against the same contract.

**Tech Stack:** Rust async/Tokio, `codex-core`, `codex-protocol` service tier types, Responses `/responses/compact` and `/responses` request builders, existing core integration tests, Cargo release build.

**Model Allocation:** FAST/NORMAL/BEST/REVIEW tiers are assigned below. Resolve each tier by explicit user override, quoted assignment in project root AGENTS.md, process environment variable, then built-in default. The project root AGENTS.md lookup reads only `<repo>/AGENTS.md`, not nested AGENTS.md files or repo-wide grep. FAST defaults to `SIMPLEPOWER_FAST_MODEL` (`gpt-5.3-codex-spark-high` when unset), NORMAL defaults to `SIMPLEPOWER_NORMAL_MODEL` (`gpt-5.4-mini-high` when unset), BEST defaults to `SIMPLEPOWER_BEST_MODEL` (`gpt-5.5-high` when unset), and REVIEW defaults to `SIMPLEPOWER_REVIEW_MODEL` (`gpt-5.5-xhigh` when unset). The plan reviewer is a REVIEW-tier plan reviewer, and the final review+fix agent is a REVIEW-tier review+fix agent. The quick verifier uses the FAST tier by default, resolving to `model="gpt-5.3-codex-spark"` and `reasoning_effort="high"` unless `SIMPLEPOWER_FAST_MODEL` is overridden.

Resolved tiers for this run from process environment, with no current-request model override and no quoted model assignments found in project-root `AGENTS.md`: FAST = `model="gpt-5.3-codex-spark"`, `reasoning_effort="high"`; NORMAL = `model="gpt-5.4-mini"`, `reasoning_effort="xhigh"`; BEST = `model="gpt-5.5"`, `reasoning_effort="xhigh"`; REVIEW = `model="gpt-5.5"`, `reasoning_effort="xhigh"`.

**Commit Policy:** The coordinator commits after the reviewed plan, allocation, and immediate current-session execution receive combined approval, after all file edits and quick verification complete before final review, and after final review/fix plus final verification. Workers, plan reviewers, quick verifiers, and review+fix agents must not commit. No per-task commits. Coordinator-owned temporary scratch refs under `refs/simplepower/scratch/<run-id>/...` may be created only as local review diff anchors; they are not accepted history commits, not pushed, not merged, not rebased, and must be cleaned up after successful checkpoints or reported for manual cleanup on blockers or failed checkpoints.

---

## Interface Contract

1. Scope and routing contract:
   - Applies only to the V1 remote-first compact path reached through `codex-rs/core/src/remote_compact_fallback.rs`.
   - `remote_compaction_v2` routing and behavior remain unchanged. Do not edit `codex-rs/core/src/compact_remote_v2.rs` or `codex-rs/core/src/responses_retry.rs`.
   - Direct local compact paths that do not go through V1 remote-first fallback keep their current service-tier behavior.
   - Existing V1 remote fallback behavior remains unchanged except for compact request service tier: configured attempts, configured timeout, configured TCP keepalive, visible warnings, clean-history restore, and fallback-to-local semantics stay intact.

2. Compact tier resolution contract:
   - Add new file `codex-rs/core/src/compact_service_tier.rs` and register it in `codex-rs/core/src/lib.rs`.
   - The module exposes a crate-private result type and resolver:
     ```rust
     #[derive(Clone, Debug, Eq, PartialEq)]
     pub(crate) struct V1RemoteFirstCompactServiceTier {
         pub(crate) remote_service_tier_override: Option<String>,
         pub(crate) local_fallback_service_tier_override: Option<String>,
     }

     pub(crate) fn resolve_v1_remote_first_compact_service_tiers(
         sess: &Session,
         turn_context: &TurnContext,
     ) -> V1RemoteFirstCompactServiceTier
     ```
   - The resolver returns `Some(ServiceTier::Fast.request_value().to_string())`, i.e. `Some("priority".to_string())`, in both fields when all of these are true:
     - `sess.services.auth_manager.auth_mode() != Some(AuthMode::ApiKey)`.
     - `turn_context.model_info.supports_service_tier(ServiceTier::Fast.request_value())` is true.
   - If auth mode is API key, `remote_service_tier_override` is `None` so V1 remote compact preserves its existing API-key behavior of omitting service tier, while `local_fallback_service_tier_override` is `turn_context.config.service_tier.clone()` so backup local compact preserves existing local compact semantics.
   - If auth mode is not API key and the model does not support `priority`, both fields are `turn_context.config.service_tier.clone()`.
   - This preserves API-key remote compact behavior and avoids emitting unsupported service tiers for models that do not advertise `priority`.

3. Remote compact API contract:
   - In `codex-rs/core/src/compact_remote.rs`, replace the positional `failure_mode: RemoteCompactionFailureMode` parameter on `run_remote_compact_task_for_mode(...)` with a named settings struct:
     ```rust
     #[derive(Clone, Debug)]
     pub(crate) struct RemoteCompactionRunSettings {
         pub(crate) failure_mode: RemoteCompactionFailureMode,
         pub(crate) service_tier_override: Option<String>,
     }
     ```
   - Existing terminal remote callers pass `RemoteCompactionRunSettings { failure_mode: RemoteCompactionFailureMode::TerminalError, service_tier_override: None }`.
   - The V1 remote-first fallback coordinator passes `RemoteCompactionRunSettings { failure_mode: RemoteCompactionFailureMode::FallbackToLocal, service_tier_override: compact_service_tiers.remote_service_tier_override.clone() }`.
   - `run_remote_compaction_request_v1(...)` uses `settings.service_tier_override.clone()` when present. If the override is absent, it preserves the existing behavior: API-key auth sends no remote compact service tier, and non-API-key auth uses `turn_context.config.service_tier.clone()`.
   - The `service_tier_override` affects only request payload construction. It must not mutate `turn_context.config`, session configuration, config files, rollout session metadata, or future turns.

4. Local fallback compact API contract:
   - In `codex-rs/core/src/compact.rs`, add a named local compact settings struct:
     ```rust
     #[derive(Clone, Debug, Default)]
     pub(crate) struct LocalCompactRunSettings {
         pub(crate) service_tier_override: Option<String>,
     }
     ```
   - Existing direct local callers keep calling the current public crate functions, which delegate with `LocalCompactRunSettings::default()`.
   - Add or refactor crate-private entrypoints so `remote_compact_fallback.rs` can run manual and auto local fallback with `LocalCompactRunSettings { service_tier_override: compact_service_tiers.local_fallback_service_tier_override.clone() }` and without duplicate `TurnStarted` events or duplicate pre-compact hooks.
   - `drain_to_completed(...)` uses `settings.service_tier_override.clone().or_else(|| turn_context.config.service_tier.clone())` for the local compact sampling request.
   - After local compact completes or fails, normal turn sampling still uses `turn_context.config.service_tier.clone()` through existing code paths.

5. V1 coordinator contract:
   - `remote_compact_fallback.rs` resolves compact tiers once before the first remote attempt by calling `resolve_v1_remote_first_compact_service_tiers(&sess, &turn_context)`.
   - `remote_service_tier_override` is passed to remote V1 attempts and `local_fallback_service_tier_override` is passed to local fallback if fallback runs.
   - The pre-remote clean-history snapshot remains taken before remote attempts, and local fallback still restores that snapshot before running.
   - When the resolved compact tier differs from `turn_context.config.service_tier`, emit this user-visible protocol status before the first remote compact attempt:
     `Compact operations are using fast service tier (priority); normal requests will return to <original> afterward.`
   - After remote compact succeeds, or after local fallback compact returns success or non-interruption failure, emit this user-visible protocol status:
     `Compact operations finished; normal requests are using <original> service tier again.`
   - `<original>` is `turn_context.config.service_tier.as_deref().unwrap_or("default")`.
   - Emit these statuses as `EventMsg::Warning(WarningEvent { message })` because the current protocol has no neutral informational event for this path. The messages are protocol-only and must not be recorded into model-visible conversation history.
   - If `CodexErr::Interrupted` or `CodexErr::TurnAborted` aborts before fallback, preserve existing abort behavior and do not emit the finished status.
   - Unsupported fast tier quietly falls back to original effective tier and emits no service-tier status messages because no tier switch occurred.

6. Test contract:
   - Tests live in `codex-rs/core/tests/suite/compact_remote.rs`.
   - Add `ModelServiceTier` import if needed and configure the active test model to advertise `priority` using `TestCodexBuilder::with_model_info_override(...)`.
   - Add integration coverage for these behaviors:
     - With ChatGPT auth, original effective service tier absent or standard/default, V1 remote compact request body includes `"service_tier": "priority"` while the normal `/responses` request after successful compact omits service tier or uses the original tier.
     - With ChatGPT auth and V1 remote attempts failing into backup local compact, all remote compact request bodies include `"service_tier": "priority"`, the local fallback `/responses` compact request includes `"service_tier": "priority"`, and the following normal `/responses` request returns to the original tier.
     - With a model that does not support `priority`, V1 remote-first compact keeps the original effective tier and does not emit `"service_tier": "priority"`.
     - When a tier switch occurs, the user-visible start and finished status messages are emitted; those status messages are absent from remote compact input, backup local compact input, and following normal request input.
   - Existing API-key service-tier omission behavior for remote compact remains covered or is updated only if the new helper requires assertion wording changes. Do not intentionally add `priority` to API-key remote compact requests.
   - Do not add V2 tests for this feature because V2 is out of scope.

7. Verification, release compile, and deployment contract:
   - Required formatting command after code edits:
     ```bash
     cd codex-rs && timeout 600s just fmt
     ```
   - Required lint/fix command before finalizing code changes:
     ```bash
     cd codex-rs && timeout 1800s just fix -p codex-core
     ```
   - Required focused test command:
     ```bash
     cd codex-rs && timeout 1800s just test -p codex-core compact_remote
     ```
   - Required release compile after final verification:
     ```bash
     (cd codex-rs && timeout 3600s cargo build --release -p codex-cli)
     test -x /home/gary/git/codex-compact-fix/codex-rs/target/release/codex
     ```
   - Required local copy after release binary exists:
     ```bash
     cp /home/gary/git/codex-compact-fix/codex-rs/target/release/codex /home/gary/codex
     ```
   - Required remote deployment, after local release binary exists:
     ```bash
     for host in fpga01 axel office backup desk; do
       ssh "$host" 'killall codex || true; mkdir -p ~/.local/bin' &&
       scp /home/gary/git/codex-compact-fix/codex-rs/target/release/codex "$host:~/.local/bin/codex"
     done
     ```
   - If any release or deployment command fails, stop and report the exact command and host if applicable. Do not skip a host or switch destinations without fresh user approval.

## File Ownership

| File | Owner task | Change type | Responsibility | Parallel safety notes |
|------|------------|-------------|----------------|-----------------------|
| `docs/simplepower/plans/2026-06-08-compact-fast-service-tier-override.md` | Coordinator planning | create | Authoritative implementation plan. | Coordinator-owned; not edited by implementation workers unless reviewer asks for plan revision before combined approval. |
| `codex-rs/core/src/compact_service_tier.rs` | Task A: Compact service tier plumbing | create | Resolve the compact-only fast/priority tier for V1 remote-first compaction. | Parallel with Task B through Interface Contract entries 2 through 6. |
| `codex-rs/core/src/lib.rs` | Task A: Compact service tier plumbing | modify | Register the new `compact_service_tier` module. | Parallel with Task B because tests do not edit this file. |
| `codex-rs/core/src/compact_remote.rs` | Task A: Compact service tier plumbing | modify | Add named remote compact run settings and apply optional service-tier override to V1 `/responses/compact` requests. | Parallel with Task B because tests do not edit this file. |
| `codex-rs/core/src/compact.rs` | Task A: Compact service tier plumbing | modify | Add named local compact run settings and apply optional service-tier override to local fallback compact streaming requests. | Parallel with Task B because tests do not edit this file. |
| `codex-rs/core/src/remote_compact_fallback.rs` | Task A: Compact service tier plumbing | modify | Resolve compact tier once, emit compact service-tier status messages, and pass the compact tier through both remote attempts and local fallback. | Parallel with Task B because tests do not edit this file. |
| `codex-rs/core/tests/suite/compact_remote.rs` | Task B: Compact service tier integration tests | modify | Add V1 compact service-tier override coverage for remote success, remote-to-local fallback, and unsupported fast tier. | Parallel with Task A through Interface Contract entries 2 through 6. |
| `codex-rs/core/tests/suite/compact_remote_parity.rs` | Task B: Compact service tier integration tests | modify | Implied-scope correction: the approved `just test -p codex-core compact_remote` verification command runs this compact parity module, so update parity normalization for the intentional V1 compact service-tier delta. | Parallel with Task A because tests do not edit implementation files. |

## Visual Aids

The visual aid below is supporting material only. If it conflicts with the Interface Contract, File Ownership, Implementation Tasks, Model Allocation, Verification, or approved path enforcement text, the written plan sections are authoritative.

```text
V1 remote-first compact
        |
        v
resolve compact tier once
  - non-API auth + model supports priority -> remote/local "priority"
  - API-key auth -> remote existing behavior, local original effective tier
  - unsupported priority -> remote/local original effective tier
        |
        v
remote /responses/compact attempt(s)
  service_tier = compact tier
        |
        +--> success
        |      |
        |      v
        |   next normal /responses request
        |   service_tier = original TurnContext tier
        |
        +--> remote exhausted
               |
               v
            restore clean history
               |
               v
            backup local compact /responses request
            service_tier = compact tier
               |
               v
            next normal /responses request
            service_tier = original TurnContext tier
```

## Implementation Tasks

### Task A: Compact Service Tier Plumbing

- **Goal:** Thread a compact-only fast/priority service tier through V1 remote compact and backup local fallback without mutating session or config state, and emit visible status messages for the temporary tier switch.
- **Contract inputs:** Interface Contract entries 1 through 5 and 7; approved brainstorming decision to implement option 1, scoped compact tier override.
- **Serialization required:** No. The Interface Contract defines the public crate APIs, file names, and expected behavior needed by Task B.
- **Write scope:**
  - `codex-rs/core/src/compact_service_tier.rs`
  - `codex-rs/core/src/lib.rs`
  - `codex-rs/core/src/compact_remote.rs`
  - `codex-rs/core/src/compact.rs`
  - `codex-rs/core/src/remote_compact_fallback.rs`
- **Parallel:** Yes, compatible with Task B.
- **Risk:** High, because this changes request construction across remote and fallback local compaction while preserving post-compact sampling behavior.
- **Model tier:** BEST, resolved to `model="gpt-5.5"`, `reasoning_effort="xhigh"`.
- **Worker role:** `sp-impl`.
- **Outputs and file-level responsibilities:** A compact service-tier resolver, named run settings for remote and local compact paths, V1 remote-first fallback wiring, and unchanged V2/direct-local behavior.
- **Implementation steps:**
  1. Add `codex-rs/core/src/compact_service_tier.rs` with `V1RemoteFirstCompactServiceTier` and `resolve_v1_remote_first_compact_service_tiers(sess: &Session, turn_context: &TurnContext) -> V1RemoteFirstCompactServiceTier` exactly as defined in Interface Contract entry 2.
  2. Register `mod compact_service_tier;` in `codex-rs/core/src/lib.rs`.
  3. In `compact_remote.rs`, define `RemoteCompactionRunSettings` and change `run_remote_compact_task_for_mode(...)` to accept that struct instead of the standalone `RemoteCompactionFailureMode` argument.
  4. Update `run_inline_remote_auto_compact_task(...)` and `run_remote_compact_task(...)` to pass `RemoteCompactionRunSettings { failure_mode: RemoteCompactionFailureMode::TerminalError, service_tier_override: None }`.
  5. Thread `RemoteCompactionRunSettings` into `run_remote_compact_task_inner_impl(...)` and `run_remote_compaction_request_v1(...)`.
  6. In V1 remote request construction, compute the request service tier from `settings.service_tier_override.clone()` when present; otherwise preserve the existing auth-aware behavior.
  7. In `compact.rs`, define `LocalCompactRunSettings`, keep existing public crate entrypoints delegating with `LocalCompactRunSettings::default()`, and add/refactor crate-private fallback entrypoints that accept `LocalCompactRunSettings`.
  8. In `drain_to_completed(...)`, pass `settings.service_tier_override.clone().or_else(|| turn_context.config.service_tier.clone())` to `client_session.stream(...)`.
  9. In `remote_compact_fallback.rs`, resolve `compact_service_tiers` once before `run_remote_attempt(...)`, pass `remote_service_tier_override` to `RemoteCompactionRunSettings`, and pass `local_fallback_service_tier_override` to local fallback through `LocalCompactRunSettings`.
  10. In `remote_compact_fallback.rs`, emit the exact start and finished status messages from Interface Contract entry 5 only when the compact tier differs from the original effective tier. Keep them protocol-only via `EventMsg::Warning`.
  11. Do not edit `compact_remote_v2.rs`, `responses_retry.rs`, config schema, or user config files.
- **Verification commands:**
  - `timeout 30s git diff --check -- codex-rs/core/src/compact_service_tier.rs codex-rs/core/src/lib.rs codex-rs/core/src/compact_remote.rs codex-rs/core/src/compact.rs codex-rs/core/src/remote_compact_fallback.rs` from repo root; expected result: no whitespace errors.
- **Completion report requirements:** Report changed files, the exact compact-tier resolution behavior, the status message behavior, how original service tier restoration is guaranteed, whether V2/direct-local behavior changed, commands run, command results, and unresolved risks.

### Task B: Compact Service Tier Integration Tests

- **Goal:** Add integration coverage proving V1 compact uses fast/priority during remote and backup local compact while normal sampling returns to the original tier, with visible status messages that stay out of model-visible history.
- **Contract inputs:** Interface Contract entries 1 through 7; approved design requirement that fallback local compact also uses the compact tier.
- **Serialization required:** No. The tests target the approved APIs and behavior in the Interface Contract and own only test files.
- **Write scope:**
  - `codex-rs/core/tests/suite/compact_remote.rs`
  - `codex-rs/core/tests/suite/compact_remote_parity.rs`
- **Parallel:** Yes, compatible with Task A.
- **Risk:** Medium, because tests exercise cross-request behavior but are isolated to the existing remote compact integration suite.
- **Model tier:** NORMAL, resolved to `model="gpt-5.4-mini"`, `reasoning_effort="xhigh"`.
- **Worker role:** `sp-impl`.
- **Outputs and file-level responsibilities:** Integration tests for successful remote compact tier override, failed remote-to-local fallback tier override, and unsupported fast-tier preservation.
- **Implementation steps:**
  1. Import `codex_protocol::openai_models::ModelServiceTier` if needed.
  2. Add a local test helper, or inline builder setup, that uses `with_model_info_override("gpt-5.4", |model_info| { model_info.service_tiers = vec![ModelServiceTier { id: ServiceTier::Fast.request_value().to_string(), name: "fast".to_string(), description: "Fast processing.".to_string() }]; })`.
  3. Add or update a ChatGPT-auth V1 remote compact success test so the normal pre/post `/responses` request has the original tier and the `/responses/compact` request has `"service_tier": "priority"` even when the original effective tier is absent or standard/default.
  4. Add or update a ChatGPT-auth remote failure fallback test that mounts failing compact responses followed by local compact and normal sampling, then asserts every remote compact request body has `"service_tier": "priority"`, the local fallback compact `/responses` request has `"service_tier": "priority"`, and the following normal `/responses` request returns to the original tier.
  5. Add a model-without-fast support assertion showing compact does not request `"priority"` when the active model does not advertise that service tier.
  6. Assert the exact start and finished status messages from Interface Contract entry 5 are emitted when compact uses `priority`, and assert those message strings are not present in remote compact input, backup local compact input, or following normal request input.
  7. In `compact_remote_parity.rs`, ignore intentional V1/V2 compact request `service_tier` differences when comparing request-shape parity; keep the dedicated API-key assertions that check the legacy and V2 service-tier behaviors explicitly.
  8. Keep or update API-key remote compact assertions so they continue to expect no forced `priority` service tier.
  9. Do not add V2 service-tier tests.
- **Verification commands:**
  - `timeout 30s git diff --check -- codex-rs/core/tests/suite/compact_remote.rs` from repo root; expected result: no whitespace errors.
- **Completion report requirements:** Report changed test file, new or updated test names, service-tier and status-message behaviors asserted, commands run, command results, and unresolved risks.

## Model Allocation

| Stage | Role | Model tier | Resolved model | Reasoning effort | Reason |
|-------|------|------------|----------------|------------------|--------|
| Plan review | Plan document reviewer | REVIEW | `gpt-5.5` | xhigh | Required by `simplepower:writing-plans`; reviews plan completeness before user approval. |
| Implementation Task A | `sp-impl` | BEST | `gpt-5.5` | xhigh | Behavior-shaping compact request plumbing across remote and local fallback paths. |
| Implementation Task B | `sp-impl` | NORMAL | `gpt-5.4-mini` | xhigh | Focused integration test changes in an existing suite against a concrete Interface Contract. |
| Quick verification | Quick verifier | FAST | `gpt-5.3-codex-spark` | high | Runs formatting, focused lint/fix, and focused tests after implementation workers complete. |
| Final review/fix | Review+fix agent | REVIEW | `gpt-5.5` | xhigh | Required single final reviewer for the whole implementation before final verification. |

## Verification

Approved path enforcement: the accepted implementation plan is authoritative. Do not use backup routes beyond the explicitly planned local compact fallback, scope reduction, docs-only substitutes, stub or placeholder implementations, skipped verification, skipped review, execution-route changes, or alternate implementation strategies unless the user gives fresh explicit approval at the moment the deviation is needed. If implementation is blocked or mismatched with the accepted plan, stop and ask the user before changing approach.

Reviewer non-recursion: the REVIEW-tier plan reviewer and REVIEW-tier final review+fix agent must perform their assigned review directly in the current worker. They must not run Codex CLI, spawn subagents, invoke Simple Power skills, restart execution, reroute the workflow, or delegate the assigned review.

Current-session auto-dispatch: after the plan reviewer approves, the coordinator asks the user for one combined approval covering the reviewed plan, the model/task allocation, and immediate current-session execution. Only after that combined approval does the coordinator create the accepted-plan checkpoint commit. Immediately after that checkpoint succeeds, the coordinator invokes `simplepower:subagent-driven-development` in the current session with this instruction:

```text
Execute `docs/simplepower/plans/2026-06-08-compact-fast-service-tier-override.md` with aggregate parallel implementation from the approved Interface Contract. Use the approved FAST/NORMAL/BEST/REVIEW model allocation. Dispatch all non-conflicting `sp-impl` file-edit workers whose coordination needs are satisfied by their Contract inputs, run the quick FAST-tier verifier with lint/build/tests and timeouts after all workers finish, commit the quick-verified implementation, then run one REVIEW-tier review+fix agent, final verification, final commit, release compile, and deployment.
```

Scratch ref review anchors are coordinator-owned local artifacts under `refs/simplepower/scratch/<run-id>/`, where `<run-id>` has format `YYYYMMDD-HHMMSS-<short-head>`. Scratch refs are not branches, accepted checkpoint commits, pushed, merged, rebased, or created by workers/reviewers. They do not change the exactly-three-checkpoint commit policy.

Plan-review refs use `refs/simplepower/scratch/<run-id>/plan-review/before` before first review and `refs/simplepower/scratch/<run-id>/plan-review/after-<n>` after coordinator revisions. Revised-plan review loops provide the same reviewer a concrete diff command, for example:

```bash
git diff refs/simplepower/scratch/<run-id>/plan-review/before refs/simplepower/scratch/<run-id>/plan-review/after-1 -- docs/simplepower/plans/2026-06-08-compact-fast-service-tier-override.md
```

Quick-verifier refs use `refs/simplepower/scratch/<run-id>/quick-verifier/before` and, only when tiny verifier fixes changed files, `refs/simplepower/scratch/<run-id>/quick-verifier/after`. Review+fix refs use `refs/simplepower/scratch/<run-id>/review-fix/before` and, only when review+fix changed files, `refs/simplepower/scratch/<run-id>/review-fix/after`. Before the next accepted checkpoint, the coordinator inspects or hands off the relevant diff command:

```bash
git diff refs/simplepower/scratch/<run-id>/<phase>/before refs/simplepower/scratch/<run-id>/<phase>/after -- <approved-files>
```

After a successful accepted checkpoint for a phase, delete that phase's scratch refs. If the workflow stops because of user direction, a blocker, or a failed checkpoint commit, preserve remaining scratch refs and report the manual cleanup command instead of deleting them.

Quick verifier scope: the quick verifier may fix only tiny typo-level issues discovered while running the quick checks. It must report behavior changes, structural edits, test rewrites, public interface changes, or unclear issues to the coordinator instead of fixing them.

Quick verification after all implementation workers finish:

```bash
(cd codex-rs && timeout 600s just fmt)
(cd codex-rs && timeout 1800s just fix -p codex-core)
(cd codex-rs && timeout 1800s just test -p codex-core compact_remote)
```

Expected result: formatting completes, scoped fix completes, and the focused core remote compact tests pass. Failure means the coordinator must stop before the quick-verified implementation checkpoint unless the quick verifier made only tiny typo-level fixes within approved scope and the commands then pass.

Final verification after the REVIEW-tier review+fix agent completes:

```bash
(cd codex-rs && timeout 600s just fmt)
(cd codex-rs && timeout 1800s just fix -p codex-core)
(cd codex-rs && timeout 1800s just test -p codex-core compact_remote)
```

Expected result: all commands pass. The coordinator performs the final checkpoint only after the REVIEW-tier review+fix agent has completed and these commands pass. Do not run the complete `just test` suite without fresh user approval because common/core full-suite coverage is broader and can be slow.

Release compile and deployment after final checkpoint:

```bash
(cd codex-rs && timeout 3600s cargo build --release -p codex-cli)
test -x /home/gary/git/codex-compact-fix/codex-rs/target/release/codex
cp /home/gary/git/codex-compact-fix/codex-rs/target/release/codex /home/gary/codex
for host in fpga01 axel office backup desk; do
  ssh "$host" 'killall codex || true; mkdir -p ~/.local/bin' &&
  scp /home/gary/git/codex-compact-fix/codex-rs/target/release/codex "$host:~/.local/bin/codex"
done
```

Expected result: release binary exists, local copy lands at `/home/gary/codex`, and each remote host receives `~/.local/bin/codex` after old remote `codex` processes are killed. Any failure stops the workflow and is reported with the exact command and host.

Final reporting must include a cleanup check for any remaining scratch refs from the run:

```bash
git for-each-ref --format='%(refname)' "refs/simplepower/scratch/<run-id>"
```

If the final checkpoint succeeds, no scratch refs for that run should remain after phase cleanup. If the workflow stops because of user direction, a blocker, or a failed checkpoint commit, preserve remaining scratch refs and report this manual cleanup command:

```bash
git for-each-ref --format='%(refname)' "refs/simplepower/scratch/<run-id>" | while read -r ref; do git update-ref -d "$ref"; done
```
