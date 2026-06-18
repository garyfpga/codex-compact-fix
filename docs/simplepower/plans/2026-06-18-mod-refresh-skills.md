# Mod Refresh Skills Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `simplepower:subagent-driven-development` for aggregate parallel implementation. Dispatch all non-conflicting `sp-impl` file-edit workers whose coordination needs are satisfied by the approved Interface Contract, run the quick verifier with the approved skill validation commands after all workers finish, commit the quick-verified implementation, then run one REVIEW-tier review+fix agent before final verification and final commit.

**Goal:** Create repo-local `mod-refresh-*` Codex skills that preflight upstream changes, preserve compact-fix mods while merging upstream, build a Linux `.mod` binary, and publish a GitHub release.

**Design Summary:** The approved design replaces a full Simple Power feature-development loop with a focused repo maintenance chain, implemented on feature branch `feature/mod-refresh-skills`. `mod-refresh-preflight` is the safe default entry point: fetch `upstream/main`, perform non-mutating merge/conflict and compact-impact analysis against `docs/compact-fix/ChangeLog.md`, then either report only or continue to `mod-refresh-release` when explicitly requested. `mod-refresh-release` is the mutation entry point and chains `mod-refresh-merge-preserve`, `mod-refresh-build`, and `mod-refresh-publish`. If the user wants new features, they should invoke `simplepower:brainstorming` separately and explicitly ask that brainstorming run `mod-refresh-preflight` during context exploration. Future release-run plans produced by these skills are saved under `docs/mod-refresh/plans/`, while this implementation plan stays in `docs/simplepower/plans/`.

**Architecture:** Implement five independent repo-local skills under `.codex/skills/`, each with concise `SKILL.md` instructions and `agents/openai.yaml` UI metadata. The Interface Contract defines the skill names, trigger boundaries, chaining protocol, preserved compact-fix contract, subagent policy, and release artifact contract so each skill can be authored independently after scaffold generation.

**Tech Stack:** Codex local skills, Markdown skill instructions, YAML agent metadata, Git/GitHub CLI workflow guidance, Rust/Cargo build commands for the Codex CLI binary.

**Model Allocation:** FAST/NORMAL/BEST/REVIEW tiers are assigned below. Resolve each tier by explicit user override, quoted assignment in project root AGENTS.md, process environment variable, then built-in default. The project root AGENTS.md lookup reads only `<repo>/AGENTS.md`, not nested AGENTS.md files or repo-wide grep. FAST defaults to `SIMPLEPOWER_FAST_MODEL` (`gpt-5.3-codex-spark-high` when unset), NORMAL defaults to `SIMPLEPOWER_NORMAL_MODEL` (`gpt-5.4-mini-high` when unset), BEST defaults to `SIMPLEPOWER_BEST_MODEL` (`gpt-5.5-high` when unset), and REVIEW defaults to `SIMPLEPOWER_REVIEW_MODEL` (`gpt-5.5-xhigh` when unset). The plan reviewer is a REVIEW-tier plan reviewer, and the final review+fix agent is a REVIEW-tier review+fix agent. The quick verifier uses the FAST tier by default, resolving to `model="gpt-5.3-codex-spark"` and `reasoning_effort="high"` unless `SIMPLEPOWER_FAST_MODEL` is overridden.

Resolved for this run from the process environment: FAST `gpt-5.3-codex-spark-high` resolves to `model="gpt-5.3-codex-spark"`, `reasoning_effort="high"`; NORMAL `gpt-5.4-mini-xhigh` resolves to `model="gpt-5.4-mini"`, `reasoning_effort="xhigh"`; BEST `gpt-5.5-xhigh` resolves to `model="gpt-5.5"`, `reasoning_effort="xhigh"`; REVIEW `gpt-5.5-xhigh` resolves to `model="gpt-5.5"`, `reasoning_effort="xhigh"`.

**Commit Policy:** All three coordinator checkpoint commits happen on feature branch `feature/mod-refresh-skills`. The coordinator commits after the reviewed plan, allocation, and immediate current-session execution receive combined approval, after all file edits and quick verification complete before final review, and after final review/fix plus final verification. Workers, plan reviewers, quick verifiers, and review+fix agents must not commit. No per-task commits. Coordinator-owned temporary scratch refs under `refs/simplepower/scratch/<run-id>/...` may be created only as local review diff anchors; they are not accepted history commits, not pushed, not merged, not rebased, and must be cleaned up after successful checkpoints or reported for manual cleanup on blockers or failed checkpoints.

---

## Interface Contract

1. Skill folders:
   - `.codex/skills/mod-refresh-preflight/`
   - `.codex/skills/mod-refresh-release/`
   - `.codex/skills/mod-refresh-merge-preserve/`
   - `.codex/skills/mod-refresh-build/`
   - `.codex/skills/mod-refresh-publish/`

2. Every skill has exactly:
   - `SKILL.md`
   - `agents/openai.yaml`

3. Skill descriptions must include concrete trigger language:
   - `mod-refresh-preflight`: use when asked to check whether pulling upstream Codex changes may conflict with or affect the fork-local compact-fix mods.
   - `mod-refresh-release`: use when asked to actually merge upstream, preserve mods, build the Linux CLI binary, tag, and publish a mod release.
   - `mod-refresh-merge-preserve`: use only when invoked by `mod-refresh-release` or explicitly asked to perform the upstream merge and compact-fix preservation step.
   - `mod-refresh-build`: use only when invoked by `mod-refresh-release` or explicitly asked to build the Linux Codex CLI `.mod` binary after a mod-refresh merge.
   - `mod-refresh-publish`: use only when invoked by `mod-refresh-release` or explicitly asked to tag and publish the mod-refresh GitHub release.

4. `mod-refresh-preflight` behavior:
   - Fetch `upstream/main`.
   - Require a clean worktree before merge simulation unless the user explicitly asks for a dirty-tree analysis.
   - Use a non-mutating path, preferably a temporary worktree or disposable branch, to simulate merging `upstream/main`.
   - Analyze likely conflicts and upstream file changes against `docs/compact-fix/ChangeLog.md`.
   - Report a compact-impact summary grouped by preservation areas from the ChangeLog.
   - Default to report-only.
   - Continue to `mod-refresh-release` only when the user explicitly requests release continuation.
   - If the user wants new features, tell them to use `simplepower:brainstorming` and explicitly ask brainstorming to run `mod-refresh-preflight` during context exploration.

5. `mod-refresh-release` behavior:
   - Require a fresh `mod-refresh-preflight` report from the current session or run/ask to run preflight first.
   - Save the release-run plan and notes under `docs/mod-refresh/plans/YYYY-MM-DD-<short-topic>.md`.
   - Chain `mod-refresh-merge-preserve`, then `mod-refresh-build`, then `mod-refresh-publish`.
   - Stop for user approval before publishing if merge preservation, build verification, or artifact naming is unclear.

6. `mod-refresh-merge-preserve` behavior:
   - Merge `upstream/main` into the current branch.
   - Resolve conflicts while preserving all relevant behavior listed in `docs/compact-fix/ChangeLog.md`.
   - Audit final diff against the ChangeLog preservation checklist before build.
   - If preserving an existing mod requires a behavior choice not discovered during preflight, stop and report the missed risk before making a product decision.

7. `mod-refresh-build` behavior:
   - Run no tests by default.
   - Do not use Bazel by default.
   - Build only the Linux CLI binary, using `cargo build -p codex-cli --release` from `codex-rs` unless the repo has a more direct checked-in command at execution time.
   - Run `just fmt` from `codex-rs` after code changes and before the build, per repo instructions.
   - Copy the resulting CLI binary to the repo root.

8. `mod-refresh-publish` behavior:
   - Compute `xxxxx` from the first five characters of the final release commit SHA.
   - Use version and tag `0.139.xxxxx.mod`.
   - Name the repo-root binary with the same version, for example `codex-0.139.xxxxx.mod-linux`.
   - Create a GitHub release for the tag and upload the binary.
   - Do not try to commit source text that embeds the final commit SHA-derived version into the same commit, because that is self-referential. The tag, release title, and artifact name carry the exact SHA-derived version.

9. Future skill subagent policy:
   - Each `mod-refresh-*` skill must explicitly specify subagents as `model = same model as main agent` and `reasoning_effort = high`.
   - This satisfies the repo AGENTS.md rule and avoids accidental lower-effort release automation.
   - Where a skill has multiple subagents, dispatch in parallel only when their write or analysis scopes do not overlap.

10. Future skill subagent names:
    - `mod-refresh-preflight`: `preflight-git-analyzer`, `compact-impact-analyzer`, `release-risk-reviewer`.
    - `mod-refresh-release`: `release-plan-reviewer`.
    - `mod-refresh-merge-preserve`: `merge-conflict-worker`, `compact-preservation-reviewer`.
    - `mod-refresh-build`: `build-verifier`.
    - `mod-refresh-publish`: `release-packaging-reviewer`.

11. Agent metadata contract:
    - `agents/openai.yaml` uses quoted string values.
    - `interface.default_prompt` must explicitly mention the matching `$mod-refresh-*` skill.
    - Keep short descriptions between 25 and 64 characters.

12. Validation contract:
    - Validate every skill folder with `/home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py`.
    - No Rust source is changed by this plan, so do not run `just fmt`, `just test`, or Rust builds for implementation verification.

13. Branch contract:
    - Execute this implementation on `feature/mod-refresh-skills`.
    - Before the accepted plan checkpoint, verify `git branch --show-current` prints `feature/mod-refresh-skills`.
    - Do not commit these skill files directly on `main`.

## File Ownership

| File | Owner task | Change type | Responsibility | Parallel safety notes |
| --- | --- | --- | --- | --- |
| `docs/simplepower/plans/2026-06-18-mod-refresh-skills.md` | Coordinator | create | Authoritative implementation plan | Coordinator only |
| `.codex/skills/mod-refresh-preflight/SKILL.md` | Author preflight skill | create | Preflight workflow instructions | Independent after scaffold |
| `.codex/skills/mod-refresh-preflight/agents/openai.yaml` | Author preflight skill | create | Preflight UI metadata | Independent after scaffold |
| `.codex/skills/mod-refresh-release/SKILL.md` | Author release orchestrator skill | create | Release orchestration instructions | Independent after scaffold |
| `.codex/skills/mod-refresh-release/agents/openai.yaml` | Author release orchestrator skill | create | Release UI metadata | Independent after scaffold |
| `.codex/skills/mod-refresh-merge-preserve/SKILL.md` | Author merge preserve skill | create | Merge and preservation instructions | Independent after scaffold |
| `.codex/skills/mod-refresh-merge-preserve/agents/openai.yaml` | Author merge preserve skill | create | Merge UI metadata | Independent after scaffold |
| `.codex/skills/mod-refresh-build/SKILL.md` | Author build skill | create | Build and artifact-copy instructions | Independent after scaffold |
| `.codex/skills/mod-refresh-build/agents/openai.yaml` | Author build skill | create | Build UI metadata | Independent after scaffold |
| `.codex/skills/mod-refresh-publish/SKILL.md` | Author publish skill | create | Tag, release, and upload instructions | Independent after scaffold |
| `.codex/skills/mod-refresh-publish/agents/openai.yaml` | Author publish skill | create | Publish UI metadata | Independent after scaffold |

## Visual Aids

```text
mod-refresh-preflight
  |-- report only
  `-- explicit continue --> mod-refresh-release
                            |--> mod-refresh-merge-preserve
                            |--> mod-refresh-build
                            `--> mod-refresh-publish

new feature path:
simplepower:brainstorming
  |-- explicitly run mod-refresh-preflight during context exploration
  `-- normal Simple Power implementation
```

## Implementation Tasks

### Task 1: Scaffold Skill Folders

Goal: Use the system skill-creator initializer to create the five skill folders with required files.

Contract inputs: Interface Contract entries 1, 2, 11, and 12.

Serialization required: Yes. Concrete reason: the skill folders and generated files must exist before independent workers can author their assigned skill contents.

Write scope:
- `.codex/skills/mod-refresh-preflight/SKILL.md`
- `.codex/skills/mod-refresh-preflight/agents/openai.yaml`
- `.codex/skills/mod-refresh-release/SKILL.md`
- `.codex/skills/mod-refresh-release/agents/openai.yaml`
- `.codex/skills/mod-refresh-merge-preserve/SKILL.md`
- `.codex/skills/mod-refresh-merge-preserve/agents/openai.yaml`
- `.codex/skills/mod-refresh-build/SKILL.md`
- `.codex/skills/mod-refresh-build/agents/openai.yaml`
- `.codex/skills/mod-refresh-publish/SKILL.md`
- `.codex/skills/mod-refresh-publish/agents/openai.yaml`

Parallel: No.

Risk: Low. This is mechanical scaffolding.

Model tier: FAST, resolved `model="gpt-5.3-codex-spark"`, `reasoning_effort="high"`.

Worker role: `sp-impl`.

Outputs and file-level responsibilities: Create the five folders and generated starter files only; later workers own final content.

Implementation steps:
1. Run these commands from the repo root:

   ```bash
   python3 /home/gary/.codex/skills/.system/skill-creator/scripts/init_skill.py mod-refresh-preflight --path .codex/skills --interface 'display_name=Mod Refresh Preflight' --interface 'short_description=Check upstream risk before a mod refresh' --interface 'default_prompt=Use $mod-refresh-preflight to check upstream merge risk before refreshing this fork.'
   python3 /home/gary/.codex/skills/.system/skill-creator/scripts/init_skill.py mod-refresh-release --path .codex/skills --interface 'display_name=Mod Refresh Release' --interface 'short_description=Merge upstream and publish a mod release' --interface 'default_prompt=Use $mod-refresh-release to merge upstream, preserve local mods, build the Linux binary, and publish a release.'
   python3 /home/gary/.codex/skills/.system/skill-creator/scripts/init_skill.py mod-refresh-merge-preserve --path .codex/skills --interface 'display_name=Mod Refresh Merge Preserve' --interface 'short_description=Merge upstream while preserving local mods' --interface 'default_prompt=Use $mod-refresh-merge-preserve to merge upstream and preserve this fork compact-fix behavior.'
   python3 /home/gary/.codex/skills/.system/skill-creator/scripts/init_skill.py mod-refresh-build --path .codex/skills --interface 'display_name=Mod Refresh Build' --interface 'short_description=Build the Linux Codex mod binary' --interface 'default_prompt=Use $mod-refresh-build to build the Linux Codex CLI binary for a mod refresh release.'
   python3 /home/gary/.codex/skills/.system/skill-creator/scripts/init_skill.py mod-refresh-publish --path .codex/skills --interface 'display_name=Mod Refresh Publish' --interface 'short_description=Tag and publish a Codex mod release' --interface 'default_prompt=Use $mod-refresh-publish to tag the mod refresh commit and publish the GitHub release.'
   ```

2. Confirm the ten files in the write scope exist.

Verification commands:
- `timeout 30s test -f .codex/skills/mod-refresh-preflight/SKILL.md -a -f .codex/skills/mod-refresh-publish/agents/openai.yaml`

Completion report requirements: list created folders, command results, and any initializer warnings.

### Task 2: Author Preflight Skill

Goal: Write `mod-refresh-preflight` so it safely fetches upstream, performs non-mutating merge/conflict and compact-impact analysis, and either reports only or explicitly continues to release.

Contract inputs: Interface Contract entries 3, 4, 9, 10, 11, and 12; `docs/compact-fix/ChangeLog.md` is the compact preservation source of truth.

Serialization required: Yes. Concrete reason: runs after Task 1 scaffold exists.

Write scope:
- `.codex/skills/mod-refresh-preflight/SKILL.md`
- `.codex/skills/mod-refresh-preflight/agents/openai.yaml`

Parallel: Yes, compatible with Tasks 3, 4, 5, and 6 after Task 1.

Risk: Medium. This skill controls whether mutation starts and must keep report-only as the default.

Model tier: BEST, resolved `model="gpt-5.5"`, `reasoning_effort="xhigh"`.

Worker role: `sp-impl`.

Outputs and file-level responsibilities: Final frontmatter, workflow body, subagent policy, report format, continuation gate, and matching metadata.

Implementation steps:
1. Replace the scaffolded `SKILL.md` with concise instructions covering:
   - Required fetch of `upstream/main`.
   - Clean-worktree check.
   - Temporary-worktree or disposable-branch merge simulation.
   - Conflict summary.
   - Upstream touched-files summary.
   - ChangeLog compact-impact mapping.
   - Report-only default.
   - Explicit continuation to `mod-refresh-release`.
   - New-feature guidance to use `simplepower:brainstorming` and explicitly request preflight during context exploration.
   - Subagents `preflight-git-analyzer`, `compact-impact-analyzer`, and `release-risk-reviewer`, each using same model as main agent and high effort.
2. Ensure `agents/openai.yaml` matches the skill and uses `$mod-refresh-preflight` in `default_prompt`.

Verification commands:
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-preflight`

Completion report requirements: summarize final trigger description, branch behavior, subagent definitions, and validation result.

### Task 3: Author Release Orchestrator Skill

Goal: Write `mod-refresh-release` as the mutation entry point that requires fresh preflight, saves release-run plans under `docs/mod-refresh/plans/`, and chains merge, build, and publish skills.

Contract inputs: Interface Contract entries 3, 5, 9, 10, 11, and 12.

Serialization required: Yes. Concrete reason: runs after Task 1 scaffold exists.

Write scope:
- `.codex/skills/mod-refresh-release/SKILL.md`
- `.codex/skills/mod-refresh-release/agents/openai.yaml`

Parallel: Yes, compatible with Tasks 2, 4, 5, and 6 after Task 1.

Risk: Medium. This skill gates mutating release work and must require preflight freshness.

Model tier: BEST, resolved `model="gpt-5.5"`, `reasoning_effort="xhigh"`.

Worker role: `sp-impl`.

Outputs and file-level responsibilities: Final orchestrator instructions, preflight requirement, plan path contract, chain order, stop conditions, and matching metadata.

Implementation steps:
1. Replace `SKILL.md` with instructions covering:
   - Require fresh `mod-refresh-preflight` report or run/ask for one first.
   - Save release-run plan to `docs/mod-refresh/plans/YYYY-MM-DD-<short-topic>.md`.
   - Chain `mod-refresh-merge-preserve`, `mod-refresh-build`, and `mod-refresh-publish` in that order.
   - Stop before publishing if merge preservation, build verification, or artifact naming is unclear.
   - Use `release-plan-reviewer` with same model as main agent and high effort.
2. Ensure `agents/openai.yaml` matches the skill and uses `$mod-refresh-release` in `default_prompt`.

Verification commands:
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-release`

Completion report requirements: summarize preflight gate, chain order, plan path, and validation result.

### Task 4: Author Merge Preserve Skill

Goal: Write `mod-refresh-merge-preserve` so it performs the real upstream merge while preserving the compact-fix contract.

Contract inputs: Interface Contract entries 3, 6, 9, 10, 11, and 12; `docs/compact-fix/ChangeLog.md` is the preservation source of truth.

Serialization required: Yes. Concrete reason: runs after Task 1 scaffold exists.

Write scope:
- `.codex/skills/mod-refresh-merge-preserve/SKILL.md`
- `.codex/skills/mod-refresh-merge-preserve/agents/openai.yaml`

Parallel: Yes, compatible with Tasks 2, 3, 5, and 6 after Task 1.

Risk: High. The workflow resolves real merge conflicts and preserves fork behavior.

Model tier: BEST, resolved `model="gpt-5.5"`, `reasoning_effort="xhigh"`.

Worker role: `sp-impl`.

Outputs and file-level responsibilities: Final merge instructions, conflict handling, preservation audit checklist, missed-risk stop condition, and matching metadata.

Implementation steps:
1. Replace `SKILL.md` with instructions covering:
   - Confirm expected branch and clean state before mutating.
   - Merge `upstream/main`.
   - Resolve conflicts with `docs/compact-fix/ChangeLog.md` as durable behavior map.
   - Preserve compact config, fallback policy, service tier behavior, call-site routing, transport retry boundaries, V1/V2 policy, tests/snapshots, TUI version label, and Simple Power plan trail when touched.
   - Stop and report if a preservation choice was not surfaced during preflight.
   - Use `merge-conflict-worker` and `compact-preservation-reviewer`, each same model as main agent and high effort.
2. Ensure `agents/openai.yaml` matches the skill and uses `$mod-refresh-merge-preserve` in `default_prompt`.

Verification commands:
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-merge-preserve`

Completion report requirements: summarize merge safeguards, preservation audit requirements, subagents, and validation result.

### Task 5: Author Build Skill

Goal: Write `mod-refresh-build` so it builds only the Linux Codex CLI binary and places the artifact in the repo root.

Contract inputs: Interface Contract entries 3, 7, 9, 10, 11, and 12.

Serialization required: Yes. Concrete reason: runs after Task 1 scaffold exists.

Write scope:
- `.codex/skills/mod-refresh-build/SKILL.md`
- `.codex/skills/mod-refresh-build/agents/openai.yaml`

Parallel: Yes, compatible with Tasks 2, 3, 4, and 6 after Task 1.

Risk: Medium. The skill intentionally bypasses the repo's normal test/Bazel path for this release workflow.

Model tier: BEST, resolved `model="gpt-5.5"`, `reasoning_effort="xhigh"`.

Worker role: `sp-impl`.

Outputs and file-level responsibilities: Final build instructions, no-test/no-Bazel default, formatting requirement, artifact-copy contract, and matching metadata.

Implementation steps:
1. Replace `SKILL.md` with instructions covering:
   - Run `just fmt` in `codex-rs` after code changes.
   - Run `cargo build -p codex-cli --release` from `codex-rs`.
   - Do not run tests by default.
   - Do not use Bazel by default.
   - Locate the built CLI binary and copy it to repo root.
   - Use `build-verifier` with same model as main agent and high effort.
2. Ensure `agents/openai.yaml` matches the skill and uses `$mod-refresh-build` in `default_prompt`.

Verification commands:
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-build`

Completion report requirements: summarize build command, artifact contract, subagent, and validation result.

### Task 6: Author Publish Skill

Goal: Write `mod-refresh-publish` so it computes the SHA-derived mod version, tags the final commit, creates the GitHub release, and uploads the binary.

Contract inputs: Interface Contract entries 3, 8, 9, 10, 11, and 12.

Serialization required: Yes. Concrete reason: runs after Task 1 scaffold exists.

Write scope:
- `.codex/skills/mod-refresh-publish/SKILL.md`
- `.codex/skills/mod-refresh-publish/agents/openai.yaml`

Parallel: Yes, compatible with Tasks 2, 3, 4, and 5 after Task 1.

Risk: High. This skill publishes tags and GitHub releases.

Model tier: BEST, resolved `model="gpt-5.5"`, `reasoning_effort="xhigh"`.

Worker role: `sp-impl`.

Outputs and file-level responsibilities: Final publish instructions, version/tag/artifact contract, GitHub release checks, self-reference warning, and matching metadata.

Implementation steps:
1. Replace `SKILL.md` with instructions covering:
   - Require build artifact in repo root.
   - Determine final release commit and `xxxxx="$(git rev-parse --short=5 HEAD)"`.
   - Use `0.139.xxxxx.mod` for tag and release title.
   - Rename/copy binary to include the same version if needed.
   - Run `git tag 0.139.xxxxx.mod`.
   - Use `gh release create 0.139.xxxxx.mod <binary> --title 0.139.xxxxx.mod --notes <notes>`.
   - Stop if the tag exists, the binary is missing, or the release command would overwrite unclear state.
   - Explain the self-referential source-version caveat.
   - Use `release-packaging-reviewer` with same model as main agent and high effort.
2. Ensure `agents/openai.yaml` matches the skill and uses `$mod-refresh-publish` in `default_prompt`.

Verification commands:
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-publish`

Completion report requirements: summarize version contract, publish safeguards, subagent, and validation result.

### Task 7: Validate Skill Set

Goal: Validate all five skills and inspect the aggregate triggering surface for consistency.

Contract inputs: Interface Contract entries 1 through 12.

Serialization required: Yes. Concrete reason: runs after Tasks 2 through 6 complete because it validates their final files.

Write scope: none unless fixing typo-level validation failures in files owned by Tasks 2 through 6 after coordinator confirms they are implied-scope corrections.

Parallel: No.

Risk: Low. This is validation and consistency review.

Model tier: FAST, resolved `model="gpt-5.3-codex-spark"`, `reasoning_effort="high"`.

Worker role: `sp-impl`.

Outputs and file-level responsibilities: Validation report only, plus typo-level fixes when allowed by coordinator.

Implementation steps:
1. Run validation for each skill:

   ```bash
   python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-preflight
   python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-release
   python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-merge-preserve
   python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-build
   python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-publish
   ```

2. Inspect trigger descriptions and default prompts:

   ```bash
   rg '^(name:|description:|  display_name:|  short_description:|  default_prompt:)' .codex/skills/mod-refresh-*/SKILL.md .codex/skills/mod-refresh-*/agents/openai.yaml
   ```

Verification commands:
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-preflight`
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-release`
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-merge-preserve`
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-build`
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-publish`

Completion report requirements: list validation results, consistency findings, and any unresolved concerns.

## Model Allocation

| Stage | Role | Model tier | Resolved model | Reasoning effort | Reason |
| --- | --- | --- | --- | --- | --- |
| Plan review | Plan reviewer | REVIEW | `gpt-5.5` | `xhigh` | Check the multi-skill workflow plan and release-safety constraints |
| Implementation | Task 1 scaffold worker | FAST | `gpt-5.3-codex-spark` | `high` | Mechanical initializer commands |
| Implementation | Task 2 preflight skill worker | BEST | `gpt-5.5` | `xhigh` | Behavior-shaping gate for safe vs mutating paths |
| Implementation | Task 3 release skill worker | BEST | `gpt-5.5` | `xhigh` | Orchestrates the mutating chain |
| Implementation | Task 4 merge preserve skill worker | BEST | `gpt-5.5` | `xhigh` | Encodes conflict resolution and compact preservation policy |
| Implementation | Task 5 build skill worker | BEST | `gpt-5.5` | `xhigh` | Encodes intentional bypass of normal test/Bazel release checks |
| Implementation | Task 6 publish skill worker | BEST | `gpt-5.5` | `xhigh` | Encodes tag and GitHub release publishing safeguards |
| Implementation | Task 7 validation worker | FAST | `gpt-5.3-codex-spark` | `high` | Runs deterministic validation and consistency checks |
| Quick verification | Quick verifier | FAST | `gpt-5.3-codex-spark` | `high` | Validate generated skill folders and metadata |
| Final review/fix | Review+fix agent | REVIEW | `gpt-5.5` | `xhigh` | Review all generated skills against the approved plan |

Canonical tier resolution for this plan:
- FAST: `SIMPLEPOWER_FAST_MODEL="gpt-5.3-codex-spark-high"` -> `model="gpt-5.3-codex-spark"`, `reasoning_effort="high"`.
- NORMAL: `SIMPLEPOWER_NORMAL_MODEL="gpt-5.4-mini-xhigh"` -> `model="gpt-5.4-mini"`, `reasoning_effort="xhigh"`.
- BEST: `SIMPLEPOWER_BEST_MODEL="gpt-5.5-xhigh"` -> `model="gpt-5.5"`, `reasoning_effort="xhigh"`.
- REVIEW: `SIMPLEPOWER_REVIEW_MODEL="gpt-5.5-xhigh"` -> `model="gpt-5.5"`, `reasoning_effort="xhigh"`.

## Plan Review

Self-review checklist:
- Design Summary captures the approved `mod-refresh-*` chain, report-only preflight default, optional release continuation, new-feature path through `simplepower:brainstorming`, and future plan directory.
- Interface Contract defines concrete file paths, skill trigger descriptions, behavior guarantees, subagent names, model/effort policy, artifact/version contract, and validation commands.
- File Ownership assigns every generated skill file to exactly one authoring task after scaffold.
- Task allocation maps every requirement to scaffold, five independent skill-authoring tasks, and aggregate validation.
- Aggregate parallel readiness is explicit: Tasks 2 through 6 can run in parallel after Task 1 because their write scopes do not overlap and their coordination needs are in the Interface Contract.
- Model allocation uses resolved environment values: FAST `gpt-5.3-codex-spark-high`, NORMAL `gpt-5.4-mini-xhigh`, BEST `gpt-5.5-xhigh`, REVIEW `gpt-5.5-xhigh`.
- Branch contract requires all checkpoint commits on `feature/mod-refresh-skills`.
- Commit policy has exactly three coordinator checkpoints.
- Scratch refs are local review anchors only.
- Verification commands are concrete and use `timeout`.
- Approved path enforcement does not authorize skipped validation, docs-only substitutes, or alternate deliverables.

Before first review, the coordinator creates `refs/simplepower/scratch/<run-id>/plan-review/before` for this plan file using the temporary-index pattern. Then dispatch one REVIEW-tier plan reviewer using `/home/gary/.codex/simplepower/skills/writing-plans/plan-document-reviewer-prompt.md`, the approved design summary, this plan path, the scratch run id, and the `plan-review/before` ref.

The REVIEW-tier plan reviewer must perform the assigned review directly in the current worker. Do not run Codex CLI. Do not spawn subagents. Do not invoke Simple Power skills. Do not restart execution. Do not reroute the workflow.

If the reviewer reports issues, update this plan, rerun focused self-review for changed categories, create `plan-review/after-<n>`, and send the same reviewer the concrete diff command:

```bash
git diff refs/simplepower/scratch/<run-id>/plan-review/before refs/simplepower/scratch/<run-id>/plan-review/after-<n> -- docs/simplepower/plans/2026-06-18-mod-refresh-skills.md
```

After reviewer approval, ask the user for combined approval of the reviewed plan, model/task allocation, and immediate current-session execution. Do not create the accepted plan checkpoint until the user gives that combined approval.

## Quick Verification

The quick verifier runs after all file-edit workers complete and before the quick-verified implementation checkpoint. It uses FAST: `model="gpt-5.3-codex-spark"`, `reasoning_effort="high"`.

Commands:
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-preflight`
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-release`
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-merge-preserve`
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-build`
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-publish`
- `timeout 30s rg 'same model as main agent' .codex/skills/mod-refresh-*/SKILL.md`
- `timeout 30s rg 'reasoning_effort = high|high effort' .codex/skills/mod-refresh-*/SKILL.md`
- `timeout 30s rg '\\$mod-refresh-(preflight|release|merge-preserve|build|publish)' .codex/skills/mod-refresh-*/agents/openai.yaml`
- `timeout 30s rg 'preflight-git-analyzer|compact-impact-analyzer|release-risk-reviewer|release-plan-reviewer|merge-conflict-worker|compact-preservation-reviewer|build-verifier|release-packaging-reviewer' .codex/skills/mod-refresh-*/SKILL.md`

Expected result: all validation commands pass and every skill explicitly encodes same-model/high subagent policy. Failure means the generated skill set is not ready for final review.

Before dispatching the quick verifier, create `refs/simplepower/scratch/<run-id>/quick-verifier/before` for the approved implementation file list. If the quick verifier makes typo-level fixes, create `quick-verifier/after` and inspect:

```bash
git diff refs/simplepower/scratch/<run-id>/quick-verifier/before refs/simplepower/scratch/<run-id>/quick-verifier/after -- .codex/skills/mod-refresh-preflight/SKILL.md .codex/skills/mod-refresh-preflight/agents/openai.yaml .codex/skills/mod-refresh-release/SKILL.md .codex/skills/mod-refresh-release/agents/openai.yaml .codex/skills/mod-refresh-merge-preserve/SKILL.md .codex/skills/mod-refresh-merge-preserve/agents/openai.yaml .codex/skills/mod-refresh-build/SKILL.md .codex/skills/mod-refresh-build/agents/openai.yaml .codex/skills/mod-refresh-publish/SKILL.md .codex/skills/mod-refresh-publish/agents/openai.yaml
```

## Final Review And Fix

After the quick-verified implementation checkpoint, dispatch one REVIEW-tier review+fix agent with `model="gpt-5.5"` and `reasoning_effort="xhigh"`. The agent reviews all five skill folders against this plan, the approved design, skill-creator guidance, trigger correctness, subagent policy, release safety, and validation results. It may fix files within the approved write scope and must not commit.

The REVIEW-tier review+fix agent must perform the assigned review and fixes directly in the current worker. Do not run Codex CLI. Do not spawn subagents. Do not invoke Simple Power skills. Do not restart execution. Do not reroute the workflow.

Before dispatching the review+fix agent, create `refs/simplepower/scratch/<run-id>/review-fix/before` for the approved implementation file list. If review+fix edits files, create `review-fix/after` and inspect:

```bash
git diff refs/simplepower/scratch/<run-id>/review-fix/before refs/simplepower/scratch/<run-id>/review-fix/after -- .codex/skills/mod-refresh-preflight/SKILL.md .codex/skills/mod-refresh-preflight/agents/openai.yaml .codex/skills/mod-refresh-release/SKILL.md .codex/skills/mod-refresh-release/agents/openai.yaml .codex/skills/mod-refresh-merge-preserve/SKILL.md .codex/skills/mod-refresh-merge-preserve/agents/openai.yaml .codex/skills/mod-refresh-build/SKILL.md .codex/skills/mod-refresh-build/agents/openai.yaml .codex/skills/mod-refresh-publish/SKILL.md .codex/skills/mod-refresh-publish/agents/openai.yaml
```

## Commit Checkpoints

1. Accepted plan checkpoint: on `feature/mod-refresh-skills`, after user combined approval for this reviewed plan, model/task allocation, and immediate current-session execution, and before invoking `simplepower:subagent-driven-development`.
2. Quick-verified implementation checkpoint: on `feature/mod-refresh-skills`, after Tasks 1 through 7 complete and quick verification passes.
3. Final checkpoint: on `feature/mod-refresh-skills`, after the REVIEW-tier review+fix agent completes and final verification passes.

Workers, plan reviewers, quick verifiers, and review+fix agents must not commit. Scratch refs under `refs/simplepower/scratch/<run-id>/` are coordinator-owned local diff anchors only.

## Current-Session Auto-Dispatch

After plan-review approval, ask the user for combined approval that covers:
- This reviewed plan.
- The model/task allocation.
- Immediate current-session execution on `feature/mod-refresh-skills`.

After combined approval, verify `git branch --show-current` prints `feature/mod-refresh-skills`, create the accepted plan checkpoint commit on that branch, delete successful `plan-review` scratch refs, and immediately invoke `simplepower:subagent-driven-development` with:

```text
Execute `docs/simplepower/plans/2026-06-18-mod-refresh-skills.md` on branch `feature/mod-refresh-skills` with aggregate parallel implementation from the approved Interface Contract. Use the approved FAST/NORMAL/BEST/REVIEW model allocation. Dispatch all non-conflicting `sp-impl` file-edit workers whose coordination needs are satisfied by their Contract inputs, run the quick FAST-tier verifier with the approved skill validation commands and timeouts after all workers finish, commit the quick-verified implementation, then run one REVIEW-tier review+fix agent, final verification, and final commit.
```

## Verification

Final verification commands:
- `timeout 30s test "$(git branch --show-current)" = feature/mod-refresh-skills`
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-preflight`
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-release`
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-merge-preserve`
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-build`
- `timeout 30s python3 /home/gary/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/mod-refresh-publish`
- `timeout 30s rg 'docs/mod-refresh/plans' .codex/skills/mod-refresh-release/SKILL.md`
- `timeout 30s rg '0\\.139\\.xxxxx\\.mod|0\\.139\\.\\$\\{xxxxx\\}\\.mod|0\\.139\\.\\{xxxxx\\}\\.mod' .codex/skills/mod-refresh-publish/SKILL.md`
- `timeout 30s rg 'docs/compact-fix/ChangeLog.md' .codex/skills/mod-refresh-preflight/SKILL.md .codex/skills/mod-refresh-merge-preserve/SKILL.md`
- `timeout 30s rg 'same model as main agent' .codex/skills/mod-refresh-*/SKILL.md`
- `timeout 30s rg '\\$mod-refresh-(preflight|release|merge-preserve|build|publish)' .codex/skills/mod-refresh-*/agents/openai.yaml`
- `timeout 30s rg 'preflight-git-analyzer|compact-impact-analyzer|release-risk-reviewer|release-plan-reviewer|merge-conflict-worker|compact-preservation-reviewer|build-verifier|release-packaging-reviewer' .codex/skills/mod-refresh-*/SKILL.md`

Expected result: all commands pass. Failure means the local skills are not complete enough to rely on for release automation.

No Rust code is changed, so do not run `just fmt`, `just test`, `cargo build`, or Bazel for implementation verification. The coordinator performs the final checkpoint only after the REVIEW-tier review+fix agent has completed and all final commands pass.

Final reporting includes:
- Created skill folders and files.
- Current branch and checkpoint commit branch.
- Validation commands and results.
- The accepted plan commit, quick-verified implementation commit, and final commit.
- Any model escalation, serialization exception, or scratch-ref issue.
- Scratch cleanup check:

  ```bash
  git for-each-ref --format='%(refname)' "refs/simplepower/scratch/<run-id>"
  ```

If the workflow stops because of user direction, a blocker, or a failed checkpoint commit, preserve remaining scratch refs and report:

```bash
git for-each-ref --format='%(refname)' "refs/simplepower/scratch/<run-id>" | while read -r ref; do git update-ref -d "$ref"; done
```
