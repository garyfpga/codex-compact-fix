# Tighten Mod Refresh Release Skills Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `simplepower:subagent-driven-development` for aggregate parallel implementation. Dispatch all non-conflicting `sp-impl` file-edit workers whose coordination needs are satisfied by the approved Interface Contract, run the quick verifier after all workers finish, commit the quick-verified implementation, then run one REVIEW-tier review+fix agent before final verification and final commit.

**Goal:** Tighten the repo-local mod-refresh release skills so upstream refresh, current-HEAD release, tests, Bazel, and publish chaining have explicit entry points and defaults.

**Design Summary:** The approved design keeps `mod-refresh-preflight` safe and report-only, adds `mod-refresh-full-release` for the explicit full upstream-refresh-to-publish flow, and adds `mod-release-current` for publishing the current `HEAD` after feature work without fetching, preflighting, or merging upstream. Both mutating release paths record `Tests: not run unless explicitly requested` and `Bazel: not used; using Cargo release build only`. Non-test maintenance checks remain allowed, including formatting, schema or snapshot regeneration when relevant, dependency lock maintenance if dependencies changed, and the Cargo release build. Publishing still stops on ambiguity around notes, repository target, existing release state, artifact presence, dirty release state, or failed build.

**Architecture:** Implement two new repo-local skills under `.codex/skills/` and tighten five existing skill instruction files so the release chain has clear entry boundaries. The Interface Contract defines exact skill names, chain semantics, no-test/no-Bazel defaults, and file ownership so independent workers can update separate skill files in aggregate parallel without inventing behavior.

**Tech Stack:** Codex local skills, Markdown skill instructions, YAML agent metadata, Git/GitHub CLI workflow guidance, Rust/Cargo build commands for the Codex CLI binary.

**Model Allocation:** FAST/NORMAL/BEST/REVIEW tiers are assigned below. Resolve each tier by explicit user override, quoted assignment in project root AGENTS.md, process environment variable, then built-in default. The project root AGENTS.md lookup reads only `<repo>/AGENTS.md`, not nested AGENTS.md files or repo-wide grep. FAST defaults to `SIMPLEPOWER_FAST_MODEL` (`gpt-5.3-codex-spark-high` when unset), NORMAL defaults to `SIMPLEPOWER_NORMAL_MODEL` (`gpt-5.4-mini-high` when unset), BEST defaults to `SIMPLEPOWER_BEST_MODEL` (`gpt-5.5-high` when unset), and REVIEW defaults to `SIMPLEPOWER_REVIEW_MODEL` (`gpt-5.5-xhigh` when unset). The plan reviewer is a REVIEW-tier plan reviewer, and the final review+fix agent is a REVIEW-tier review+fix agent. The quick verifier uses the FAST tier by default, resolving to `model="gpt-5.3-codex-spark"` and `reasoning_effort="high"` unless `SIMPLEPOWER_FAST_MODEL` is overridden.

Resolved for this run from the process environment: FAST `gpt-5.3-codex-spark-high` resolves to `model="gpt-5.3-codex-spark"`, `reasoning_effort="high"`; NORMAL `gpt-5.4-mini-xhigh` resolves to `model="gpt-5.4-mini"`, `reasoning_effort="xhigh"`; BEST `gpt-5.5-xhigh` resolves to `model="gpt-5.5"`, `reasoning_effort="xhigh"`; REVIEW `gpt-5.5-xhigh` resolves to `model="gpt-5.5"`, `reasoning_effort="xhigh"`.

**Commit Policy:** The coordinator commits after the reviewed plan, allocation, and immediate current-session execution receive combined approval, after all file edits and quick verification complete before final review, and after final review/fix plus final verification. Workers, plan reviewers, quick verifiers, and review+fix agents must not commit. No per-task commits. Coordinator-owned temporary scratch refs under `refs/simplepower/scratch/<run-id>/...` may be created only as local review diff anchors; they are not accepted history commits, not pushed, not merged, not rebased, and must be cleaned up after successful checkpoints or reported for manual cleanup on blockers or failed checkpoints.

---

## Interface Contract

1. Skill folders:
   - Existing folders remain:
     - `.codex/skills/mod-refresh-preflight/`
     - `.codex/skills/mod-refresh-release/`
     - `.codex/skills/mod-refresh-merge-preserve/`
     - `.codex/skills/mod-refresh-build/`
     - `.codex/skills/mod-refresh-publish/`
   - New folders are created:
     - `.codex/skills/mod-refresh-full-release/`
     - `.codex/skills/mod-release-current/`

2. Every new skill folder has exactly:
   - `SKILL.md`
   - `agents/openai.yaml`

3. Trigger boundaries:
   - `mod-refresh-preflight`: report-only upstream risk analysis. It must not merge, build, tag, publish, invoke `mod-refresh-release`, invoke `mod-refresh-full-release`, or mutate release state.
   - `mod-refresh-full-release`: explicit full upstream-refresh release entry point. It runs fresh `mod-refresh-preflight`, and if preflight is not blocked, chains through `mod-refresh-release` to merge, build, tag, and publish.
   - `mod-release-current`: explicit current-HEAD release entry point for after feature work. It must not fetch upstream, run preflight, simulate or perform an upstream merge, or invoke `mod-refresh-release`. It runs `mod-refresh-build`, then `mod-refresh-publish`.
   - `mod-refresh-release`: remains the mutating upstream refresh orchestrator that requires a fresh current-session preflight handoff and chains merge preservation, build, and publish. It is normally invoked by `mod-refresh-full-release` or by a user who already has a fresh preflight.
   - `mod-refresh-merge-preserve`: performs only the upstream merge and compact-fix preservation step when invoked by `mod-refresh-release` or explicitly requested.
   - `mod-refresh-build`: builds the Linux Codex CLI `.mod` artifact from the current post-merge or current-feature state.
   - `mod-refresh-publish`: computes the SHA-derived version from final `HEAD`, tags, creates the GitHub release, and uploads the verified repo-root artifact. It may be invoked by `mod-refresh-release`, by `mod-release-current`, or by an explicit direct publish request.

4. Release-plan decision records:
   - Mutating release orchestrators must record these explicit defaults in their plan or handoff notes:
     - `Tests: not run unless explicitly requested`
     - `Bazel: not used; using Cargo release build only`
   - `mod-refresh-full-release` records these decisions before invoking `mod-refresh-release`.
   - `mod-release-current` records these decisions in its current-HEAD release notes or handoff before invoking `mod-refresh-build`.
   - `mod-refresh-release` records these decisions in `docs/mod-refresh/plans/YYYY-MM-DD-<short-topic>.md`.

5. Test policy:
   - Do not run `just test`, `cargo test`, Bazel tests, full upstream suites, or focused upstream test commands during mod-refresh release flows unless the user explicitly requests tests for that release.
   - Non-test maintenance checks remain allowed: `just fmt`, schema generation, snapshot review or acceptance when required by intentionally changed generated UI/text artifacts, dependency lock maintenance when dependencies changed, and release compilation.
   - If Rust dependencies changed, run `just bazel-lock-update` and `just bazel-lock-check` from the repository root. This is dependency lock maintenance, not approval to run Bazel as the release build or test path.

6. Bazel policy:
   - Bazel is not used for release build or test verification by default.
   - The default release build command remains:
     ```bash
     cd codex-rs
     cargo build -p codex-cli --release
     ```
   - Use Bazel build or Bazel test commands only when the user explicitly requests Bazel for that release.

7. Full-refresh data flow:
   ```text
   mod-refresh-full-release
     -> mod-refresh-preflight
     -> if ready, mod-refresh-release
     -> mod-refresh-merge-preserve
     -> mod-refresh-build
     -> mod-refresh-publish
   ```

8. Current-HEAD release data flow:
   ```text
   mod-release-current
     -> mod-refresh-build
     -> mod-refresh-publish
   ```

9. Stop conditions:
   - Preflight blocked or reports unresolved blockers: stop before mutation.
   - Merge preservation discovers an unsurfaced compact-fix behavior choice: stop and ask.
   - Build fails or artifact is missing: stop before publish.
   - Publish ambiguity or existing release state exists: stop before tag/upload.
   - If `git tag` succeeds but `gh release create` fails, report partial state and ask for explicit recovery direction.

10. Future skill subagent policy:
    - Each new or modified `mod-refresh-*` release skill must specify subagents as same model as main agent and `reasoning_effort = high`, except where a Simple Power plan or skill explicitly overrides dispatch settings.
    - New skill reviewer names:
      - `mod-refresh-full-release`: `full-release-preflight-reviewer` and `full-release-chain-reviewer`
      - `mod-release-current`: `current-release-packaging-reviewer`

11. Agent metadata contract:
    - New `agents/openai.yaml` files use quoted string values.
    - `interface.default_prompt` must explicitly mention the matching `$mod-refresh-full-release` or `$mod-release-current` skill.
    - Keep short descriptions between 25 and 64 characters.

12. Validation contract:
    - Validate every touched skill folder with `/home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py`.
    - No Rust source is changed by this plan, so do not run `just fmt`, `just test`, `cargo build`, or Bazel for implementation verification.

## File Ownership

| File | Owner task | Change type | Responsibility | Parallel safety notes |
| --- | --- | --- | --- | --- |
| `docs/simplepower/plans/2026-06-18-tighten-mod-refresh-release-skills.md` | Coordinator | create | Authoritative implementation plan | Coordinator only |
| `.codex/skills/mod-refresh-full-release/SKILL.md` | Task 1 | create | Full upstream-refresh release entry point | Independent new file |
| `.codex/skills/mod-refresh-full-release/agents/openai.yaml` | Task 1 | create | Full release UI metadata | Independent new file |
| `.codex/skills/mod-release-current/SKILL.md` | Task 2 | create | Current-HEAD build-and-publish entry point | Independent new file |
| `.codex/skills/mod-release-current/agents/openai.yaml` | Task 2 | create | Current release UI metadata | Independent new file |
| `.codex/skills/mod-refresh-preflight/SKILL.md` | Task 3 | modify | Preserve report-only semantics and direct users to full-release entry point | Independent existing file |
| `.codex/skills/mod-refresh-release/SKILL.md` | Task 4 | modify | Record no-tests/no-Bazel defaults and clarify full-release handoff | Independent existing file |
| `.codex/skills/mod-refresh-merge-preserve/SKILL.md` | Task 5 | modify | Remove default test-running guidance and allow only non-test maintenance checks unless tests are requested | Independent existing file |
| `.codex/skills/mod-refresh-build/SKILL.md` | Task 6 | modify | Make Cargo/no-Bazel/no-tests decision recording explicit for both release entry points | Independent existing file |
| `.codex/skills/mod-refresh-publish/SKILL.md` | Task 7 | modify | Allow invocation from current-HEAD release path and preserve publish safety checks | Independent existing file |
| `.codex/skills/mod-refresh-publish/agents/openai.yaml` | Task 7 | modify | Publish metadata mentions both refresh and current release paths | Independent existing file |

## Visual Aids

```text
safe analysis:
mod-refresh-preflight
  `-- report only, no mutation

full upstream refresh release:
mod-refresh-full-release
  |-- mod-refresh-preflight
  |-- mod-refresh-release
  |   |-- mod-refresh-merge-preserve
  |   |-- mod-refresh-build
  |   `-- mod-refresh-publish
  `-- published GitHub release

current HEAD release after feature work:
mod-release-current
  |-- mod-refresh-build
  |-- mod-refresh-publish
  `-- published GitHub release
```

## Implementation Tasks

### Task 1: Add Full Refresh Release Skill

Goal: Create `mod-refresh-full-release` as the explicit entry point that runs preflight and then publishes the full upstream refresh release when preflight is ready.

Contract inputs: Interface Contract entries 1, 2, 3, 4, 5, 6, 7, 9, 10, 11, and 12.

Serialization required: No. The Interface Contract defines the skill name, trigger boundary, chain semantics, and metadata contract; this task writes only new files.

Write scope:
- `.codex/skills/mod-refresh-full-release/SKILL.md`
- `.codex/skills/mod-refresh-full-release/agents/openai.yaml`

Parallel: Yes, compatible with Tasks 2, 3, 4, 5, 6, and 7.

Risk: Medium. This creates a mutating release entry point that must preserve safety gates while automating the intended chain.

Model tier: BEST, resolved `model="gpt-5.5"`, `reasoning_effort="xhigh"`.

Worker role: `sp-impl`.

Outputs and file-level responsibilities: Final frontmatter, workflow body, handoff semantics, stop conditions, subagent policy, completion report, and matching metadata.

Implementation steps:
1. Create `.codex/skills/mod-refresh-full-release/SKILL.md` with frontmatter:
   ```markdown
   ---
   name: "mod-refresh-full-release"
   description: "Run a complete mod refresh release by preflighting upstream, merging, preserving local mods, building the Linux artifact, tagging, and publishing. Use when the user explicitly asks for the full upstream refresh release flow."
   ---
   ```
2. The body must state:
   - This is the explicit full upstream refresh release entry point.
   - Run `$mod-refresh-preflight` first in the current session.
   - Do not mutate if preflight is blocked or reports unresolved blockers.
   - If preflight recommends ready to continue, invoke `$mod-refresh-release`.
   - Ensure release plan or handoff records `Tests: not run unless explicitly requested` and `Bazel: not used; using Cargo release build only`.
   - Do not run tests or Bazel build/test commands unless explicitly requested.
   - Use non-test maintenance checks only as allowed by Interface Contract entry 5.
   - Stop conditions match Interface Contract entry 9.
3. Add a subagent policy section that names `full-release-preflight-reviewer` and `full-release-chain-reviewer`, same model as main agent and `reasoning_effort = high`.
4. Create `.codex/skills/mod-refresh-full-release/agents/openai.yaml`:
   ```yaml
   interface:
     display_name: "Mod Refresh Full Release"
     short_description: "Preflight, merge, build, and publish"
     default_prompt: "Use $mod-refresh-full-release to preflight upstream, merge, preserve local mods, build, tag, and publish a full mod refresh release."
   ```

Verification commands:
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-full-release`

Completion report requirements: changed files, validation command result, chain summary, no-test/no-Bazel wording, and unresolved risks.

### Task 2: Add Current HEAD Release Skill

Goal: Create `mod-release-current` as the explicit entry point that builds current `HEAD` and publishes it without upstream fetch, preflight, or merge.

Contract inputs: Interface Contract entries 1, 2, 3, 4, 5, 6, 8, 9, 10, 11, and 12.

Serialization required: No. The Interface Contract defines the skill name, trigger boundary, chain semantics, and metadata contract; this task writes only new files.

Write scope:
- `.codex/skills/mod-release-current/SKILL.md`
- `.codex/skills/mod-release-current/agents/openai.yaml`

Parallel: Yes, compatible with Tasks 1, 3, 4, 5, 6, and 7.

Risk: Medium. This creates a publish path that intentionally bypasses upstream refresh, so the no-fetch/no-merge boundary must be explicit.

Model tier: BEST, resolved `model="gpt-5.5"`, `reasoning_effort="xhigh"`.

Worker role: `sp-impl`.

Outputs and file-level responsibilities: Final frontmatter, workflow body, no-upstream boundary, build/publish handoff, stop conditions, subagent policy, completion report, and matching metadata.

Implementation steps:
1. Create `.codex/skills/mod-release-current/SKILL.md` with frontmatter:
   ```markdown
   ---
   name: "mod-release-current"
   description: "Build and publish a mod release from the current HEAD without fetching upstream, running preflight, or merging. Use after feature work when the current commit is the intended release source."
   ---
   ```
2. The body must state:
   - This is the current-HEAD release entry point for after feature work.
   - Do not run `git fetch upstream`, `$mod-refresh-preflight`, merge simulation, or `git merge upstream/main`.
   - Confirm current `HEAD`, branch, and worktree state before building.
   - Record `Tests: not run unless explicitly requested` and `Bazel: not used; using Cargo release build only`.
   - Invoke `$mod-refresh-build`, then `$mod-refresh-publish`.
   - Do not run tests or Bazel build/test commands unless explicitly requested.
   - Use non-test maintenance checks only as allowed by Interface Contract entry 5.
   - Stop conditions match Interface Contract entry 9.
3. Add a subagent policy section that names `current-release-packaging-reviewer`, same model as main agent and `reasoning_effort = high`.
4. Create `.codex/skills/mod-release-current/agents/openai.yaml`:
   ```yaml
   interface:
     display_name: "Mod Release Current"
     short_description: "Build and publish current HEAD"
     default_prompt: "Use $mod-release-current to build the current HEAD Linux mod artifact, tag it, and publish the GitHub release without pulling upstream."
   ```

Verification commands:
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-release-current`

Completion report requirements: changed files, validation command result, no-upstream boundary summary, no-test/no-Bazel wording, and unresolved risks.

### Task 3: Tighten Preflight Report-Only Semantics

Goal: Update `mod-refresh-preflight` so it remains a safe analysis tool and points full release requests to `mod-refresh-full-release`.

Contract inputs: Interface Contract entries 3, 7, 9, and 12.

Serialization required: No. This task modifies only the preflight skill and relies on the Interface Contract for the new full-release skill name.

Write scope:
- `.codex/skills/mod-refresh-preflight/SKILL.md`

Parallel: Yes, compatible with Tasks 1, 2, 4, 5, 6, and 7.

Risk: Medium. This skill controls the safe default boundary and must not accidentally chain into mutation.

Model tier: BEST, resolved `model="gpt-5.5"`, `reasoning_effort="xhigh"`.

Worker role: `sp-impl`.

Outputs and file-level responsibilities: Updated default stance, workflow continuation wording, recommendation format, and report-only gate.

Implementation steps:
1. Replace wording that says preflight can hand off to `mod-refresh-release` after explicit continuation.
2. State that `mod-refresh-preflight` never invokes mutating release skills itself, even if the initial request includes release intent.
3. Add guidance: for a complete upstream refresh release, use `$mod-refresh-full-release`.
4. Keep non-mutating fetch, worktree check, merge simulation, and ChangeLog impact analysis intact.
5. Update the `Continuation gate` report wording so it says release mutation requires a separate `$mod-refresh-full-release` or `$mod-refresh-release` invocation with a fresh current-session preflight.

Verification commands:
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-preflight`
- `timeout 30s rg 'mod-refresh-full-release|report-only|must not|never invokes' .codex/skills/mod-refresh-preflight/SKILL.md`

Completion report requirements: changed file, validation command result, grep result, and summary of report-only boundary.

### Task 4: Tighten Release Orchestrator Decisions

Goal: Update `mod-refresh-release` so its release plans explicitly record the no-tests/no-Bazel decisions and recognize `mod-refresh-full-release` as the full-flow caller.

Contract inputs: Interface Contract entries 3, 4, 5, 6, 7, 9, and 12.

Serialization required: No. This task modifies only the release orchestrator skill and relies on the Interface Contract for shared policy wording.

Write scope:
- `.codex/skills/mod-refresh-release/SKILL.md`

Parallel: Yes, compatible with Tasks 1, 2, 3, 5, 6, and 7.

Risk: Medium. This skill writes release plans and chains publish, so ambiguous defaults can create expensive mistakes.

Model tier: BEST, resolved `model="gpt-5.5"`, `reasoning_effort="xhigh"`.

Worker role: `sp-impl`.

Outputs and file-level responsibilities: Updated purpose, required preflight handoff, release plan fields, execution chain notes, stop conditions, and completion wording.

Implementation steps:
1. Mention `$mod-refresh-full-release` as the preferred full upstream refresh entry point when the user wants preflight through publish in one request.
2. Keep the fresh current-session preflight requirement for `mod-refresh-release`.
3. In the release plan required fields, add:
   - `Tests: not run unless explicitly requested`
   - `Bazel: not used; using Cargo release build only`
4. Add policy wording that tests and Bazel build/test commands are not run unless explicitly requested.
5. Add non-test maintenance allowance wording from Interface Contract entry 5, including dependency lock maintenance if dependencies changed.

Verification commands:
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-release`
- `timeout 30s rg 'Tests: not run unless explicitly requested|Bazel: not used|mod-refresh-full-release|dependency lock' .codex/skills/mod-refresh-release/SKILL.md`

Completion report requirements: changed file, validation command result, grep result, and summary of release-plan defaults.

### Task 5: Tighten Merge Preserve Verification Policy

Goal: Update `mod-refresh-merge-preserve` so merge conflict resolution does not default to upstream tests, while still allowing non-test maintenance checks.

Contract inputs: Interface Contract entries 5, 6, 9, and 12.

Serialization required: No. This task modifies only the merge-preserve skill and relies on the Interface Contract for shared policy wording.

Write scope:
- `.codex/skills/mod-refresh-merge-preserve/SKILL.md`

Parallel: Yes, compatible with Tasks 1, 2, 3, 4, 6, and 7.

Risk: High. This is the current inconsistency that can cause accidental upstream test execution during release work.

Model tier: BEST, resolved `model="gpt-5.5"`, `reasoning_effort="xhigh"`.

Worker role: `sp-impl`.

Outputs and file-level responsibilities: Updated merge workflow verification step, preservation checklist wording, missed-risk examples, and final report wording.

Implementation steps:
1. Replace the current workflow step that says to run the smallest verification matching touched areas and follows repo `just test` guidance.
2. New wording must say:
   - After conflicts are resolved, run only non-test maintenance checks needed by touched files.
   - Allowed examples: `just fmt`, schema generation, snapshot review or acceptance when generated UI/text artifacts intentionally changed, dependency lock maintenance if dependencies changed.
   - Do not run `just test`, `cargo test`, Bazel tests, full upstream suites, or focused upstream tests unless explicitly requested.
   - Build verification happens in `$mod-refresh-build`.
3. Preserve the compact-fix checklist, but clarify that test and snapshot assets are preserved as source artifacts; test commands are not run by default.
4. Update the final report to include skipped-test and skipped-Bazel status when applicable.

Verification commands:
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-merge-preserve`
- `timeout 30s rg 'Do not run `just test`|Build verification happens|non-test maintenance|skipped-test|skipped-Bazel' .codex/skills/mod-refresh-merge-preserve/SKILL.md`

Completion report requirements: changed file, validation command result, grep result, and summary of removed default test behavior.

### Task 6: Tighten Build Decision Recording

Goal: Update `mod-refresh-build` so both release entry points get explicit Cargo/no-tests/no-Bazel recording.

Contract inputs: Interface Contract entries 3, 4, 5, 6, 8, 9, and 12.

Serialization required: No. This task modifies only the build skill and relies on the Interface Contract for shared policy wording.

Write scope:
- `.codex/skills/mod-refresh-build/SKILL.md`

Parallel: Yes, compatible with Tasks 1, 2, 3, 4, 5, and 7.

Risk: Medium. The build skill is already mostly aligned, but it must make the decision record explicit for both release paths.

Model tier: NORMAL, resolved `model="gpt-5.4-mini"`, `reasoning_effort="xhigh"`.

Worker role: `sp-impl`.

Outputs and file-level responsibilities: Updated purpose, workflow inputs, decision records, verifier checklist, and completion report wording.

Implementation steps:
1. State that the skill may be invoked by `$mod-refresh-release` or `$mod-release-current`.
2. Require recording:
   - `Tests: not run unless explicitly requested`
   - `Bazel: not used; using Cargo release build only`
3. Keep `cargo build -p codex-cli --release` as the default build command.
4. Keep the existing rule that tests and Bazel are not run by default.
5. Clarify that dependency lock maintenance is allowed only if dependencies changed; it does not authorize Bazel as the release build/test path.
6. Update the build verifier checklist to check both decision records.

Verification commands:
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-build`
- `timeout 30s rg 'mod-release-current|Tests: not run unless explicitly requested|Bazel: not used|cargo build -p codex-cli --release|dependency lock' .codex/skills/mod-refresh-build/SKILL.md`

Completion report requirements: changed file, validation command result, grep result, and summary of Cargo/no-Bazel/no-test decision recording.

### Task 7: Tighten Publish Invocation Boundary

Goal: Update `mod-refresh-publish` so it can be invoked by both upstream refresh releases and current-HEAD releases while preserving publish safety checks.

Contract inputs: Interface Contract entries 3, 8, 9, 11, and 12.

Serialization required: No. This task modifies only the publish skill and its metadata; the Interface Contract defines the new `mod-release-current` caller.

Write scope:
- `.codex/skills/mod-refresh-publish/SKILL.md`
- `.codex/skills/mod-refresh-publish/agents/openai.yaml`

Parallel: Yes, compatible with Tasks 1, 2, 3, 4, 5, and 6.

Risk: High. This skill mutates remote tag and GitHub release state.

Model tier: BEST, resolved `model="gpt-5.5"`, `reasoning_effort="xhigh"`.

Worker role: `sp-impl`.

Outputs and file-level responsibilities: Updated frontmatter description, purpose, required inputs, invocation wording, metadata prompt, and safety wording.

Implementation steps:
1. Update `.codex/skills/mod-refresh-publish/SKILL.md` frontmatter description so the skill may be invoked by `$mod-refresh-release`, by `$mod-release-current`, or by an explicit direct publish request.
2. In the purpose and required inputs sections, state that the final release commit can come from either a completed upstream refresh merge or from the current-HEAD feature release path.
3. Preserve all existing safety checks: final release commit at `HEAD`, verified repo-root artifact, approved release notes, confirmed GitHub repository target, no existing local tag, no existing remote tag, no existing GitHub release, and artifact name matching `codex-${version}-linux`.
4. Do not loosen self-referential version caveat or partial-failure handling.
5. Update `.codex/skills/mod-refresh-publish/agents/openai.yaml` default prompt to mention publishing either a mod refresh release or a current-HEAD mod release.

Verification commands:
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-publish`
- `timeout 30s rg 'mod-release-current|current-HEAD|mod refresh release|direct publish request' .codex/skills/mod-refresh-publish/SKILL.md .codex/skills/mod-refresh-publish/agents/openai.yaml`

Completion report requirements: changed files, validation command result, grep result, and summary of publish invocation boundaries.

## Model Allocation

| Stage | Role | Model tier | Resolved model | Reasoning effort | Reason |
| --- | --- | --- | --- | --- | --- |
| Plan review | Plan document reviewer | REVIEW | `gpt-5.5` | `xhigh` | Required REVIEW-tier plan review before combined approval |
| Implementation | Task 1 full refresh release skill worker | BEST | `gpt-5.5` | `xhigh` | New mutating release entry point with cross-skill chain semantics |
| Implementation | Task 2 current HEAD release skill worker | BEST | `gpt-5.5` | `xhigh` | New publish path with intentional no-upstream boundary |
| Implementation | Task 3 preflight tightening worker | BEST | `gpt-5.5` | `xhigh` | Safe report-only boundary is behavior-shaping |
| Implementation | Task 4 release orchestrator tightening worker | BEST | `gpt-5.5` | `xhigh` | Release plan and publish-chain defaults are high impact |
| Implementation | Task 5 merge-preserve verification tightening worker | BEST | `gpt-5.5` | `xhigh` | Removes accidental test execution from the conflict resolution path |
| Implementation | Task 6 build decision worker | NORMAL | `gpt-5.4-mini` | `xhigh` | Localized update to already-aligned build instructions |
| Implementation | Task 7 publish boundary worker | BEST | `gpt-5.5` | `xhigh` | Remote release mutation boundary must accept the new current-HEAD caller without loosening checks |
| Quick verification | Quick verifier | FAST | `gpt-5.3-codex-spark` | `high` | Static skill validation and targeted grep checks after all edits |
| Final review+fix | Review+fix agent | REVIEW | `gpt-5.5` | `xhigh` | Required full implementation review and fixes before final verification |

## Plan Review

Self-review checklist:
- Design Summary captures the approved two-entry-point design, report-only preflight, no-tests/no-Bazel defaults, allowed non-test checks, and publish stop conditions.
- Interface Contract lists exact skill names, filenames, command contracts, behavior guarantees, data flows, and cross-task assumptions before File Ownership.
- File Ownership assigns every created or modified file to exactly one task and reserves the plan file for the coordinator.
- Task allocation maps every approved requirement to a task with `Contract inputs`, `Serialization required`, exact write scopes, verification commands, and completion requirements.
- Aggregate parallel readiness is explicit: Tasks 1 through 7 have non-overlapping write scopes and can run together after the plan checkpoint.
- Visual Aids are inline, support the written contract, and do not introduce separate artifacts.
- Model allocation resolves FAST/NORMAL/BEST/REVIEW using process environment values after checking project root `AGENTS.md`; no nested `AGENTS.md` scan or repo-wide grep is used.
- Review allocation includes one REVIEW-tier plan reviewer and one REVIEW-tier final review+fix agent.
- Commit policy defines exactly three coordinator checkpoints and forbids worker, reviewer, verifier, and task commits.
- Scratch refs are coordinator-only local review anchors under `refs/simplepower/scratch/<run-id>/` with creation, revised-plan diff handoff, cleanup, blocker preservation, and final cleanup check guidance.
- Verification commands are concrete and use `timeout`.
- Approved path enforcement does not authorize route changes, skipped review, skipped validation, docs-only substitutes, or reduced deliverables.

Before first review, the coordinator creates `refs/simplepower/scratch/<run-id>/plan-review/before` for this saved plan file using the temporary-index pattern. The run id format is `YYYYMMDD-HHMMSS-<short-head>`. Revised plans after blocking issues create `plan-review/after-<n>` refs and are sent back to the same reviewer with one of these concrete diff commands:

```bash
git diff refs/simplepower/scratch/<run-id>/plan-review/before refs/simplepower/scratch/<run-id>/plan-review/after-1 -- docs/simplepower/plans/2026-06-18-tighten-mod-refresh-release-skills.md
git diff refs/simplepower/scratch/<run-id>/plan-review/after-1 refs/simplepower/scratch/<run-id>/plan-review/after-2 -- docs/simplepower/plans/2026-06-18-tighten-mod-refresh-release-skills.md
```

The REVIEW-tier plan reviewer must perform the assigned review directly in the current worker. Do not run Codex CLI. Do not spawn subagents. Do not invoke Simple Power skills. Do not restart execution. Do not reroute the workflow. Close the reviewer only after approval, unrecoverable interruption, or explicit user direction.

After reviewer approval, ask the user for combined approval of the reviewed plan, model/task allocation, and immediate current-session execution. The accepted plan checkpoint commit happens only after that combined approval. After that checkpoint succeeds, delete the `plan-review` scratch refs for the run. If the checkpoint fails or the workflow stops before checkpointing, preserve the refs and report this manual cleanup command:

```bash
git for-each-ref --format='%(refname)' "refs/simplepower/scratch/<run-id>" | while read -r ref; do git update-ref -d "$ref"; done
```

## Quick Verification

The quick verifier runs after all file-edit workers complete and before the coordinator creates the quick-verified implementation checkpoint. Before dispatching the quick verifier, the coordinator creates `refs/simplepower/scratch/<run-id>/quick-verifier/before` for the approved implementation file list:

```text
.codex/skills/mod-refresh-full-release/SKILL.md
.codex/skills/mod-refresh-full-release/agents/openai.yaml
.codex/skills/mod-release-current/SKILL.md
.codex/skills/mod-release-current/agents/openai.yaml
.codex/skills/mod-refresh-preflight/SKILL.md
.codex/skills/mod-refresh-release/SKILL.md
.codex/skills/mod-refresh-merge-preserve/SKILL.md
.codex/skills/mod-refresh-build/SKILL.md
.codex/skills/mod-refresh-publish/SKILL.md
.codex/skills/mod-refresh-publish/agents/openai.yaml
```

If the quick verifier makes tiny typo-level fixes, the coordinator creates `refs/simplepower/scratch/<run-id>/quick-verifier/after` and inspects or hands off this diff:

```bash
git diff refs/simplepower/scratch/<run-id>/quick-verifier/before refs/simplepower/scratch/<run-id>/quick-verifier/after -- .codex/skills/mod-refresh-full-release/SKILL.md .codex/skills/mod-refresh-full-release/agents/openai.yaml .codex/skills/mod-release-current/SKILL.md .codex/skills/mod-release-current/agents/openai.yaml .codex/skills/mod-refresh-preflight/SKILL.md .codex/skills/mod-refresh-release/SKILL.md .codex/skills/mod-refresh-merge-preserve/SKILL.md .codex/skills/mod-refresh-build/SKILL.md .codex/skills/mod-refresh-publish/SKILL.md .codex/skills/mod-refresh-publish/agents/openai.yaml
```

The quick verifier may fix only tiny typo-level errors discovered while running the quick checks. Any behavior change, structural edit, public interface change, or unclear issue must be reported to the coordinator instead of fixed.

Quick verification commands:
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-full-release`
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-release-current`
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-preflight`
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-release`
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-merge-preserve`
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-build`
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-publish`
- `timeout 30s rg 'Tests: not run unless explicitly requested|Bazel: not used; using Cargo release build only|mod-refresh-full-release|mod-release-current' .codex/skills/mod-refresh-full-release .codex/skills/mod-release-current .codex/skills/mod-refresh-preflight .codex/skills/mod-refresh-release .codex/skills/mod-refresh-merge-preserve .codex/skills/mod-refresh-build .codex/skills/mod-refresh-publish`
- `timeout 30s rg 'mod-release-current|current-HEAD|direct publish request' .codex/skills/mod-refresh-publish/SKILL.md .codex/skills/mod-refresh-publish/agents/openai.yaml`

Expected result: all commands pass. Failure means a skill folder is invalid or the required release-policy wording is absent.

After the quick-verified implementation checkpoint succeeds, delete that run's `quick-verifier` scratch refs. If the checkpoint fails or the workflow stops before checkpointing, preserve the refs and report the manual cleanup command from Plan Review.

## Final Review And Fix

After the quick-verified implementation checkpoint, dispatch one REVIEW-tier review+fix agent. That agent reviews and fixes the whole implementation against the accepted plan, file ownership, approved path enforcement, aggregate parallel dispatch semantics, and verification requirements.

Before dispatching the review+fix agent, the coordinator creates `refs/simplepower/scratch/<run-id>/review-fix/before` for the approved implementation file list. If the review+fix agent edits files, the coordinator creates `refs/simplepower/scratch/<run-id>/review-fix/after` after those edits and before final verification, then inspects or hands off this diff:

```bash
git diff refs/simplepower/scratch/<run-id>/review-fix/before refs/simplepower/scratch/<run-id>/review-fix/after -- .codex/skills/mod-refresh-full-release/SKILL.md .codex/skills/mod-refresh-full-release/agents/openai.yaml .codex/skills/mod-release-current/SKILL.md .codex/skills/mod-release-current/agents/openai.yaml .codex/skills/mod-refresh-preflight/SKILL.md .codex/skills/mod-refresh-release/SKILL.md .codex/skills/mod-refresh-merge-preserve/SKILL.md .codex/skills/mod-refresh-build/SKILL.md .codex/skills/mod-refresh-publish/SKILL.md .codex/skills/mod-refresh-publish/agents/openai.yaml
```

The review+fix agent may edit files within the plan's approved file ownership when fixing issues it finds. It must report changed files, commands run, results, remaining risks, and unresolved deviations that require user approval. It must not commit.

The REVIEW-tier review+fix agent must perform the assigned review and fixes directly in the current worker. Do not run Codex CLI. Do not spawn subagents. Do not invoke Simple Power skills. Do not restart execution. Do not reroute the workflow. If no file changes happen during review+fix, omit the `review-fix/after` ref.

## Commit Checkpoints

Exactly three future coordinator checkpoint commits are authorized:

1. Accepted plan checkpoint: after the user gives combined approval for the reviewed plan, model/task allocation, and immediate current-session execution, and before invoking `simplepower:subagent-driven-development`.
2. Quick-verified implementation checkpoint: after all `sp-impl` file edits complete and the quick verifier passes.
3. Final checkpoint: after the REVIEW-tier review+fix agent completes and final verification passes.

Workers, plan reviewers, quick verifiers, review+fix agents, and individual tasks must not commit. Scratch refs are coordinator-owned, local-only review anchors; they are not branches, accepted checkpoint commits, pushed, merged, or rebased. Successful phase checkpoints delete that phase's scratch refs. Blockers, user stops, and failed checkpoint commits preserve scratch refs and report the manual cleanup command from Plan Review.

## Current-Session Auto-Dispatch

After the plan reviewer approves, ask the user for one combined approval covering:
- The reviewed plan.
- The model/task allocation.
- Immediate current-session execution.

If the user requests changes, update the plan, rerun the focused self-review checks for the changed categories, create the next `plan-review/after-<n>` scratch ref, and send the revised plan back to the same reviewer with the concrete scratch-ref `git diff` command.

After combined approval, the coordinator creates the accepted plan checkpoint commit, deletes successful `plan-review` scratch refs, then immediately invokes `simplepower:subagent-driven-development` in the current session with this instruction:

```text
Execute `docs/simplepower/plans/2026-06-18-tighten-mod-refresh-release-skills.md` with aggregate parallel implementation from the approved Interface Contract. Use the approved FAST/NORMAL/BEST/REVIEW model allocation. Dispatch all non-conflicting `sp-impl` file-edit workers whose coordination needs are satisfied by their Contract inputs, run the quick FAST-tier verifier with the listed validation commands and timeouts after all workers finish, commit the quick-verified implementation, then run one REVIEW-tier review+fix agent, final verification, and final commit.
```

## Verification

Final verification runs after the REVIEW-tier review+fix agent completes and before the final checkpoint commit:

- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-full-release`
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-release-current`
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-preflight`
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-release`
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-merge-preserve`
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-build`
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-publish`
- `timeout 30s rg 'Tests: not run unless explicitly requested|Bazel: not used; using Cargo release build only|mod-refresh-full-release|mod-release-current' .codex/skills/mod-refresh-full-release .codex/skills/mod-release-current .codex/skills/mod-refresh-preflight .codex/skills/mod-refresh-release .codex/skills/mod-refresh-merge-preserve .codex/skills/mod-refresh-build .codex/skills/mod-refresh-publish`
- `timeout 30s rg 'mod-release-current|current-HEAD|direct publish request' .codex/skills/mod-refresh-publish/SKILL.md .codex/skills/mod-refresh-publish/agents/openai.yaml`
- `timeout 30s rg 'git fetch upstream|mod-refresh-preflight|git merge upstream/main' .codex/skills/mod-release-current/SKILL.md` must show these commands only in prohibition wording, not as workflow commands.

Expected result: all skill validation commands pass, required policy strings are present, and `mod-release-current` explicitly prohibits upstream fetch, preflight, and upstream merge.

No Rust code is changed, so do not run `just fmt`, `just test`, `cargo build`, or Bazel for implementation verification. The coordinator performs the final checkpoint only after the REVIEW-tier review+fix agent has completed and all final commands pass.

Final reporting includes this cleanup check:

```bash
git for-each-ref --format='%(refname)' "refs/simplepower/scratch/<run-id>"
```

If the final checkpoint succeeds, no scratch refs for that run should remain after phase cleanup. If the workflow stops because of user direction, a blocker, or a failed checkpoint commit, preserve remaining scratch refs and report this manual cleanup command:

```bash
git for-each-ref --format='%(refname)' "refs/simplepower/scratch/<run-id>" | while read -r ref; do git update-ref -d "$ref"; done
```
