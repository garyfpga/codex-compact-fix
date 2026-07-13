# Multi-Agent Version Override Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use simplepower:subagent-driven-development for aggregate parallel implementation. Dispatch all non-conflicting sp-impl file-edit workers whose coordination needs are satisfied by the approved Interface Contract, run the quick verifier after all workers finish, commit the quick-verified implementation, then run one REVIEW-tier review+fix agent before final verification and final commit.

**Goal:** Add a fork-native multi_agent_version_override config key that selects V1, V2, or disabled behavior for new roots and spawned children while preserving resumed session contracts.

**Design Summary:** Add the top-level enum-valued TOML key multi_agent_version_override = "v1", accepting disabled, v1, and v2. Newly constructed roots, forks, and children resolve config override, then copied/persisted history, inherited parent version, model-catalog metadata, and feature fallback. Resumed existing threads retain their persisted or legacy V1 version. Resolve after model metadata is loaded so a remote catalog cannot beat the explicit override. Update schema and targeted tests, create a dedicated branch before implementation, and add the V1 line to /home/gary/.codex/config.toml. Do not modify the bundled catalog at codex-rs/models-manager/models.json; the runtime override must remain effective when remote metadata replaces bundled metadata.

**Architecture:** ConfigToml owns the user-facing enum and Config carries the effective merged value. Codex::spawn performs the definitive resolution after ModelsManager::get_model_info, initializes the session OnceLock, and persists the selection in new thread metadata. ThreadManager uses a history-only form before session construction so child capacity, residency, and fork bookkeeping agree with the eventual version.

**Tech Stack:** Rust workspace, Serde/TOML, schemars-generated config.schema.json, Codex ModelsManager, JSONL/thread metadata, just, and codex-core integration tests with mocked Responses/model-catalog endpoints.

**Model Allocation:** FAST/NORMAL/BEST/REVIEW tiers are independent of use_subagent. Resolve them from built-in defaults, then /home/gary/.codex/simplepower.toml, repository simplepower.toml, non-empty SIMPLEPOWER_*_MODEL environment values, and explicit current-session instructions; validate every present TOML source before overlaying it. Do not read model assignments from AGENTS.md. Final reasoning-effort suffixes must be one of low, medium, high, xhigh, max, or ultra. The validated values for this session are FAST gpt-5.3-codex-spark/xhigh, NORMAL gpt-5.4/high, BEST gpt-5.5/xhigh, and REVIEW gpt-5.5/xhigh. The plan reviewer and final review+fix agent use REVIEW; the quick verifier uses FAST; mandatory tiers are independent of use_subagent.

**Commit Policy:** The coordinator commits after reviewed-plan approval and combined user approval, after implementation and quick verification, and after final review/fix and final verification. Workers and reviewers never commit. No per-task commits. Coordinator-only scratch refs under refs/simplepower/scratch/<run-id>/ are local review anchors, not accepted history commits, and are cleaned after successful checkpoints or reported for manual cleanup on blockers.

---

## Interface Contract

This contract is authoritative for all tasks and enables non-conflicting
aggregate dispatch.

1. The shared enum remains codex_protocol::protocol::MultiAgentVersion with
   Disabled, V1, and V2. Its existing Serde/schema form accepts the strings
   disabled, v1, and v2. No protocol/API enum is added.

2. codex_config::config_toml::ConfigToml gains this global-only field:

    pub multi_agent_version_override: Option<MultiAgentVersion>

   The existing deny-unknown-fields behavior and generated schema reject
   unknown values. No profile-scoped field is added.

3. codex_core::config::Config gains the corresponding effective field:

    pub multi_agent_version_override: Option<MultiAgentVersion>

   Config loading copies it from ConfigToml. Config-lock serialization retains
   it through ConfigToml automatically.

4. The definitive resolver in codex-rs/core/src/session/mod.rs has the shape:

    pub(crate) fn resolve_multi_agent_version(
        conversation_history: &InitialHistory,
        inherited_multi_agent_version: Option<MultiAgentVersion>,
        config_override: Option<MultiAgentVersion>,
        model_multi_agent_version: Option<MultiAgentVersion>,
        feature_fallback: MultiAgentVersion,
    ) -> MultiAgentVersion

   For InitialHistory::Resumed, return persisted history version, then
   inherited version, then legacy V1; ignore a current config override,
   catalog metadata, and feature fallback. For New, Cleared, and Forked,
   return config_override, copied/persisted history version, inherited parent
   version, model_multi_agent_version, then feature_fallback.

5. The history-only helper has the shape:

    pub(crate) fn resolve_multi_agent_version_from_history(
        conversation_history: &InitialHistory,
        inherited_multi_agent_version: Option<MultiAgentVersion>,
        config_override: Option<MultiAgentVersion>,
    ) -> Option<MultiAgentVersion>

   It applies the same resumed/new distinction but has no catalog or feature
   input. It returns None only when a new construction has no early selection,
   preserving the chance for catalog metadata to win later.

6. Codex::spawn calls the definitive resolver after
   models_manager.get_model_info(...). The selected value is passed to
   Session::new, initializes its OnceLock, and is used by new thread metadata.

7. ThreadManager::initial_multi_agent_version_for_spawn calls the history-only
   helper with config.multi_agent_version_override. Its result feeds early
   child capacity/residency and fork-marker decisions. Direct fork-marker
   paths must apply the same override rule.

8. Session::resolve_multi_agent_version_for_model and the startup preview path
   preserve an initialized session version. If the lock is unset, config
   override precedes catalog metadata and feature fallback.

9. Acceptance behavior:

   - A fresh Sol-like model whose catalog says V2 selects and stores V1 under
     the V1 override and exposes the V1 tool surface.
   - A child created under the override stores V1 even when child metadata says
     V2.
   - A resumed V2 rollout remains V2 when current config says V1.
   - disabled suppresses both multi-agent surfaces for new sessions; v2 selects
     the existing V2 surface.
   - Removing the override restores current catalog/feature behavior.

10. The schema command is timeout 240s just write-config-schema from the
    repository root and updates the checked-in config schema.

## File Ownership

| File | Owner task | Change type | Responsibility | Parallel safety notes |
|---|---|---|---|---|
| docs/simplepower/plans/2026-07-13-multi-agent-version-override.md | Coordinator | create | Authoritative implementation plan | No implementation worker edits it |
| codex-rs/config/src/config_toml.rs | Task 1 | modify | Top-level optional enum field | Disjoint from runtime files |
| codex-rs/core/src/config/mod.rs | Task 1 | modify | Effective Config field and loading | Disjoint from runtime files |
| codex-rs/core/src/config/config_tests.rs | Task 1 | modify | Config parsing/default/error coverage | Disjoint test file |
| codex-rs/core/config.schema.json | Task 1 | generated | Checked-in schema output | Task 1 exclusively owns it |
| codex-rs/core/src/session/mod.rs | Task 2 | modify | Definitive/history-only resolvers and session fallback | Task 2 exclusively owns runtime |
| codex-rs/core/src/session/turn_context.rs | Task 2 | modify | Preview and turn selection fallback | Task 2 exclusively owns runtime |
| codex-rs/core/src/thread_manager.rs | Task 2 | modify | Spawn/fork early version selection | Task 2 exclusively owns runtime |
| codex-rs/core/src/session/tests.rs | Task 2 | modify | Precedence and resumed compatibility tests | Disjoint from integration test |
| codex-rs/core/tests/suite/model_runtime_selectors.rs | Task 3 | modify | Sol root/child integration test | Disjoint suite file |
| /home/gary/.codex/config.toml | Task 4 | modify | User-requested V1 setting | Serialized after repository verification |

## Implementation Tasks

### Task 1 — Add typed config and schema

**Goal:** Make multi_agent_version_override a strict top-level config key and
regenerate the checked-in schema.

**Contract inputs:** Interface Contract items 1–3 and 10; global-only scope;
accepted values disabled, v1, and v2.

**Serialization required:** No. This task owns its source and generated files.

**Write scope:** codex-rs/config/src/config_toml.rs,
codex-rs/core/src/config/mod.rs, codex-rs/core/src/config/config_tests.rs,
codex-rs/core/config.schema.json.

**Parallel:** Yes, with Tasks 2 and 3. Task 4 is later and serialized.

**Risk:** Medium; typed config, Config construction, and generated schema are
compile-coupled.

**Model tier:** NORMAL, model gpt-5.4, effort high.

**Worker role:** sp-impl.

**Steps and outputs:**

1. Import the existing MultiAgentVersion type and add the optional field near
   model-selection settings with documentation of precedence and resumed
   compatibility.
2. Add the matching Config field and assign cfg.multi_agent_version_override in
   the Config constructor.
3. Add isolated temporary-home tests for all three values, absent None, and an
   invalid string rejected by strict loading. Use whole-value assertions.
4. Run:

    timeout 240s just write-config-schema
    timeout 300s just test -p codex-config
    timeout 300s just test -p codex-core config_tests

   The schema must contain the three enum values and both test commands must
   pass.

**Completion report:** Changed paths, commands/results, generated property, and
any unresolved compile or test risk.

### Task 2 — Implement session and thread precedence

**Goal:** Make construction, child spawning, fork bookkeeping, model selection,
and preview honor the approved precedence while preserving resumed contracts.

**Contract inputs:** Interface Contract items 4–8; Config field shape from Task
1; ModelInfo::multi_agent_version is the catalog input.

**Serialization required:** No. The contract supplies all cross-task shapes.

**Write scope:** codex-rs/core/src/session/mod.rs,
codex-rs/core/src/session/turn_context.rs,
codex-rs/core/src/thread_manager.rs,
codex-rs/core/src/session/tests.rs.

**Parallel:** Yes, with Tasks 1 and 3.

**Risk:** High; behavior crosses session construction, child capacity,
residency, forks, model metadata, and compatibility paths.

**Model tier:** BEST, model gpt-5.5, effort xhigh.

**Worker role:** sp-impl.

**Steps and outputs:**

1. Add the definitive and history-only resolver functions with the exact
   resumed/new rules in the Interface Contract. Treat Forked as new so its
   explicit override wins over copied history.
2. In Codex::spawn, after get_model_info, pass config override, inherited
   version, model metadata, and feature fallback to the definitive resolver.
   Pass the result to Session::new so new thread metadata and OnceLock agree.
3. Update Session::resolve_multi_agent_version_for_model and the Preview branch
   in turn_context.rs so an unset lock uses the override before catalog and
   feature fallback, while an initialized lock is sticky.
4. Update ThreadManager::initial_multi_agent_version_for_spawn and direct
   InterruptedTurnHistoryMarker fork paths to use override-aware early
   resolution. Do not inject feature fallback before catalog lookup.
5. Add session tests for fresh override over V2 catalog/feature, child override
   over inherited V2, history/inherited ordering without override, catalog over
   feature fallback, disabled, and resumed V2 remaining V2 under V1 config.
6. Run:

    timeout 360s just test -p codex-core resolve_multi_agent_version
    timeout 360s just test -p codex-core session

   Both must pass; precedence or compatibility failures are reported rather
   than worked around.

**Completion report:** Runtime/test paths, final resolver order, resumed branch,
commands/results, and any untested internal path.

### Task 3 — Add Sol root and child integration coverage

**Goal:** Prove the user-visible behavior against a catalog advertising V2,
including a child created through the V1 tool surface.

**Contract inputs:** Interface Contract items 1, 6, and 9; existing helpers in
model_runtime_selectors.rs and core_test_support responses/TestCodexBuilder.

**Serialization required:** No. The integration test targets the fixed contract
and owns a disjoint file.

**Write scope:** codex-rs/core/tests/suite/model_runtime_selectors.rs.

**Parallel:** Yes, with Tasks 1 and 2.

**Risk:** Medium; mocked discovery, asynchronous spawn, and tool-surface
assertions must line up.

**Model tier:** NORMAL, model gpt-5.4, effort high.

**Worker role:** sp-impl.

**Steps and outputs:**

1. Reuse remote_model, mount_models_once, mount_sse_once_match, and
   ev_function_call_with_namespace patterns already in the suite.
2. Configure a Sol-like model whose catalog metadata is V2 and set
   config.multi_agent_version_override to V1.
3. Assert the fresh root response contains multi_agent_v1 and not the V2
   namespace. Return a V1 spawn_agent call, provide child and parent follow-up
   responses, then retrieve the child through
   test.thread_manager.get_thread(thread_id).await? and assert its complete
   version is Some(MultiAgentVersion::V1).
4. Run:

    timeout 420s just test -p codex-core model_runtime_selectors

   The test must pass while metadata remains V2.

**Completion report:** Test name/assertions, commands/results, and whether the
child was observed through the live manager or rollout metadata.

### Task 4 — Apply the requested home configuration

**Goal:** Enable V1 in the user's local fork after repository implementation
and verification.

**Contract inputs:** Interface Contract items 1–3 and 9; exact line
multi_agent_version_override = "v1"; preserve all unrelated content.

**Serialization required:** Yes. Apply this external setting after repository
tests and make no other home-config edit.

**Write scope:** /home/gary/.codex/config.toml.

**Parallel:** No; run after Tasks 1–3 and repository verification.

**Risk:** Low but user-visible; duplicate or nested TOML would change startup.

**Model tier:** FAST, model gpt-5.3-codex-spark, effort xhigh.

**Worker role:** sp-impl.

**Steps and outputs:**

1. Insert exactly one root-level multi_agent_version_override = "v1" line.
2. Verify:

    timeout 30s rg -n '^multi_agent_version_override = "v1"$' /home/gary/.codex/config.toml
    timeout 30s rg -n '^multi_agent_version_override = ' /home/gary/.codex/config.toml

   The first command returns one line and the second shows no duplicate.
   Config parsing remains covered by isolated repository tests.

**Completion report:** Exact line count, confirmation that no other home lines
changed, and both verification results. Do not commit the home file.

## Model Allocation

| Stage | Role | Tier | Resolved model | Effort | Reason |
|---|---|---|---|---|---|
| Task 1 | sp-impl | NORMAL | gpt-5.4 | high | Localized typed config and schema work |
| Task 2 | sp-impl | BEST | gpt-5.5 | xhigh | Cross-cutting, behavior-shaping runtime precedence |
| Task 3 | sp-impl | NORMAL | gpt-5.4 | high | Established integration helpers with async assertions |
| Task 4 | sp-impl | FAST | gpt-5.3-codex-spark | xhigh | Exact one-line external edit |
| Plan review | worker | REVIEW | gpt-5.5 | xhigh | Independent contract/ownership/verification review |
| Quick verification | worker | FAST | gpt-5.3-codex-spark | xhigh | Bounded format/schema/type/test checks |
| Final review/fix | worker | REVIEW | gpt-5.5 | xhigh | Whole-diff plan compliance and in-scope fixes |

## Plan Review

The coordinator self-reviews this plan for approved-design fidelity, exact
Interface Contract before ownership, unique file ownership, non-conflicting
Tasks 1–3, serialized Task 4, model overlays, timeout-bounded commands, and
exactly three checkpoints.

Record a UTC run id as YYYYMMDD-HHMMSS-<short-head>, for example
20260713-120000-a57cb2d4. Scratch refs are only under
refs/simplepower/scratch/<run-id>/ and are coordinator-owned local review
anchors.

Before review, create plan-review/before with a temporary index:

    SP_RUN_ID=$(date -u +%Y%m%d-%H%M%S)-$(git rev-parse --short HEAD)
    SP_PREFIX=refs/simplepower/scratch/$SP_RUN_ID
    SP_TMP_INDEX=$(mktemp)
    GIT_INDEX_FILE=$SP_TMP_INDEX git read-tree HEAD
    GIT_INDEX_FILE=$SP_TMP_INDEX git add -- docs/simplepower/plans/2026-07-13-multi-agent-version-override.md
    SP_TREE=$(GIT_INDEX_FILE=$SP_TMP_INDEX git write-tree)
    SP_COMMIT=$(printf '%s\n' "simplepower scratch $SP_RUN_ID plan-review/before" | git commit-tree "$SP_TREE" -p HEAD)
    git update-ref "$SP_PREFIX/plan-review/before" "$SP_COMMIT"
    rm -f "$SP_TMP_INDEX"

Read /home/gary/.codex/simplepower/skills/writing-plans/plan-document-reviewer-prompt.md
and dispatch one REVIEW worker with model gpt-5.5, effort xhigh, and
fork_turns="none". The self-contained prompt includes the approved design,
plan path, run id, before ref, read-only/no-edit/no-commit/no-subagent
constraints, and required review categories: contract completeness, ownership,
parallel readiness, model allocation, scratch refs, checkpoints, and bounded
verification. Keep the reviewer open through issue loops.

If issues are found, edit only this plan, rerun focused self-review, create
plan-review/after-1 (or the next number), and send the reviewer:

    git diff refs/simplepower/scratch/<run-id>/plan-review/before refs/simplepower/scratch/<run-id>/plan-review/after-1 -- docs/simplepower/plans/2026-07-13-multi-agent-version-override.md

Later revisions compare the prior after ref to the new one. Never rely on a
missing anchor. After approval, ask the user for combined approval of the
reviewed plan, allocation, and immediate execution. Do not create the accepted
plan checkpoint before that approval.

## Quick Verification

After Tasks 1–3 complete, the coordinator runs timeout 180s just fmt from the
repository root, then creates quick-verifier/before for the approved
implementation file list using the temporary-index pattern.

Dispatch the FAST verifier with fork_turns="none" and a self-contained prompt.
It runs:

    timeout 60s just fmt-check
    timeout 240s just write-config-schema
    timeout 300s just test -p codex-config
    timeout 420s just test -p codex-core resolve_multi_agent_version
    timeout 420s just test -p codex-core model_runtime_selectors

Expected results are clean formatting, current enum schema, and passing config,
resolver, and Sol/child tests. The verifier may fix only a typo-level issue. Any
behavioral, structural, API, or unclear issue is reported. If it edits, create
quick-verifier/after and inspect:

    git diff refs/simplepower/scratch/<run-id>/quick-verifier/before refs/simplepower/scratch/<run-id>/quick-verifier/after -- <approved-files>

Delete quick-verifier refs after the quick-verified checkpoint succeeds.
Otherwise preserve them and report:

    git for-each-ref --format='%(refname)' "refs/simplepower/scratch/<run-id>" | while read -r ref; do git update-ref -d "$ref"; done

## Final Review And Fix

After the quick-verified implementation checkpoint, create
review-fix/before for the approved implementation files and dispatch exactly
one REVIEW worker with model gpt-5.5, effort xhigh, and fork_turns="none".
Require whole-diff review against this plan, the interface contract, ownership,
precedence, resumed behavior, child behavior, schema, tests, formatting, and
home configuration. It may edit only approved owned files and must not commit,
spawn, invoke skills, or reroute execution.

If it edits, create review-fix/after and inspect:

    git diff refs/simplepower/scratch/<run-id>/review-fix/before refs/simplepower/scratch/<run-id>/review-fix/after -- <approved-files>

The report lists changed files, commands/results, remaining risks, and any
deviation requiring fresh approval. Delete phase refs only after final success;
preserve and report cleanup on a blocker or failed checkpoint.

## Commit Checkpoints

Exactly three future coordinator checkpoints are authorized:

1. Accepted plan checkpoint: after plan-review approval and combined user
   approval. Create feature/multi-agent-version-override from clean main, commit
   the plan, delete successful plan-review refs, and immediately invoke
   simplepower:subagent-driven-development.
2. Quick-verified implementation checkpoint: after all sp-impl edits and FAST
   verification pass. Repository implementation/schema/tests are committed;
   the home config remains outside repository history.
3. Final checkpoint: after REVIEW review/fix, final verification, and exact
   home-config line-count confirmation. Workers never create checkpoints.

## Current-Session Auto-Dispatch

After plan-review approval, ask the user for one combined approval covering the
reviewed plan, model/task allocation, and immediate current-session execution.
If changes are requested, revise this plan, create the next plan-review ref,
send the exact scratch-ref diff to the same reviewer, and ask again. Do not
create the accepted checkpoint without combined approval.

After approval, create the branch and accepted plan checkpoint, delete
successful plan-review refs, then immediately invoke
simplepower:subagent-driven-development with:

    Execute docs/simplepower/plans/2026-07-13-multi-agent-version-override.md with aggregate parallel implementation from the approved Interface Contract. Use the approved FAST/NORMAL/BEST/REVIEW allocation. Dispatch all non-conflicting sp-impl workers, run the quick FAST verifier with timeout-bounded checks, commit the quick-verified implementation, then run one REVIEW review+fix worker, final verification, and final commit.

Every future implementation, verifier, and review dispatch passes exactly
fork_turns="none" and includes a self-contained task, scope, constraints,
contract inputs, outputs, and exact commands. No alternate route, scope
reduction, skipped check, or unapproved substitute is authorized.

## Verification

After review/fix and before the final checkpoint, run:

    timeout 60s just fmt-check
    timeout 240s just write-config-schema
    timeout 300s just test -p codex-config
    timeout 420s just test -p codex-core resolve_multi_agent_version
    timeout 420s just test -p codex-core model_runtime_selectors
    timeout 600s just test -p codex-core

Targeted commands pass before the complete codex-core suite. The complete suite
requires the user approval specified by the repository instructions; if absent,
stop before that command and ask. Any failure blocks the final checkpoint until
the approved implementation is corrected or a changed path is approved.

Verify the external setting:

    grep -Fxq 'multi_agent_version_override = "v1"' /home/gary/.codex/config.toml
    test "$(grep -Fxc 'multi_agent_version_override = "v1"' /home/gary/.codex/config.toml)" -eq 1

After successful cleanup, run:

    git for-each-ref --format='%(refname)' "refs/simplepower/scratch/<run-id>"

It must print no refs for the run. If execution stops because of user direction,
a blocker, or failed checkpoint, preserve remaining refs and report the manual
cleanup command above.

The coordinator creates the final checkpoint only after the REVIEW worker
completes, final commands pass, schema is current, resumed compatibility and
fresh Sol/child behavior are covered, and the home setting appears exactly once.
