# Mod Refresh Upstream Metadata Versioning Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `simplepower:subagent-driven-development` for aggregate parallel implementation. Dispatch all non-conflicting `sp-impl` file-edit workers whose coordination needs are satisfied by the approved Interface Contract, run the quick verifier after all workers finish, commit the quick-verified implementation, then run one REVIEW-tier review+fix agent before final verification and final commit.

**Goal:** Make mod release tags derive from checked-in upstream provenance metadata instead of final fork commit SHA, and initialize that metadata for the current refresh.

**Design Summary:** The approved design adds repository-root `upstreamhash.txt` and `modversion.txt` as the release metadata source of truth. `upstreamhash.txt` stores the full merged upstream commit SHA; `modversion.txt` stores a positive integer mod release sequence. The mod release version becomes `<latest-upstream-major>.<latest-upstream-minor>.<first5-upstreamhash>.<modversion>.mod`, and the Git tag, GitHub release tag, and GitHub release title use that exact value. The initial values are `2c7802e7cf3ad53733ca9fb603f270debcca280f` and `1`.

**Architecture:** The release process remains Markdown skill driven. The refresh skills maintain the two metadata files during upstream refreshes, while publish/current-release skills validate and consume them through one shared shell contract. The Interface Contract below fixes file formats, version computation, and tag/release behavior so independent workers can update different skill documents in parallel.

**Tech Stack:** Markdown skill files, root text metadata files, Git, GitHub CLI, POSIX shell snippets.

**Model Allocation:** FAST/NORMAL/BEST/REVIEW tiers are assigned below. Resolve each tier by explicit user override, quoted assignment in project root AGENTS.md, process environment variable, then built-in default. The project root AGENTS.md lookup reads only `<repo>/AGENTS.md`, not nested AGENTS.md files or repo-wide grep. FAST defaults to `SIMPLEPOWER_FAST_MODEL` (`gpt-5.3-codex-spark-high` when unset), NORMAL defaults to `SIMPLEPOWER_NORMAL_MODEL` (`gpt-5.4-mini-high` when unset), BEST defaults to `SIMPLEPOWER_BEST_MODEL` (`gpt-5.5-high` when unset), and REVIEW defaults to `SIMPLEPOWER_REVIEW_MODEL` (`gpt-5.5-xhigh` when unset). The plan reviewer is a REVIEW-tier plan reviewer, and the final review+fix agent is a REVIEW-tier review+fix agent. The quick verifier uses the FAST tier by default, resolving to `model="gpt-5.3-codex-spark"` and `reasoning_effort="high"` unless `SIMPLEPOWER_FAST_MODEL` is overridden.

**Commit Policy:** The coordinator commits after the reviewed plan, allocation, and immediate current-session execution receive combined approval, after all file edits and quick verification complete before final review, and after final review/fix plus final verification. Workers, plan reviewers, quick verifiers, and review+fix agents must not commit. No per-task commits. Coordinator-owned temporary scratch refs under `refs/simplepower/scratch/<run-id>/...` may be created only as local review diff anchors; they are not accepted history commits, not pushed, not merged, not rebased, and must be cleaned up after successful checkpoints or reported for manual cleanup on blockers or failed checkpoints.

---

## Interface Contract

- `upstreamhash.txt` is a repository-root file containing exactly one line: the full 40-character lowercase hexadecimal upstream commit SHA. Initial content is `2c7802e7cf3ad53733ca9fb603f270debcca280f\n`.
- `modversion.txt` is a repository-root file containing exactly one line: a positive decimal integer with no sign. Initial content is `1\n`.
- Version computation must use the latest non-draft, non-prerelease `openai/codex` release for the base series, strip leading `rust-v` or `v`, require SemVer `x.y.z`, and use only the first two SemVer components for `base_series`.
- Version computation must read `upstreamhash.txt` and `modversion.txt`, validate their exact shapes, set `upstream_short` to the first five characters of `upstreamhash.txt`, and set `version="${base_series}.${upstream_short}.${mod_version}.mod"`.
- Git tag, pushed tag ref, GitHub release tag argument, and GitHub release title must all use the exact computed `version`. The upload asset remains repository-root `codex`.
- `$mod-refresh-preflight` must report the fetched upstream target SHA explicitly as `git rev-parse upstream/main`.
- `$mod-refresh-merge-preserve` must update `upstreamhash.txt` to the actual merged upstream target SHA and set `modversion.txt` to `1` for a new upstream refresh unless the invoking handoff records a different explicit approved positive integer.
- `$mod-refresh-release` and `$mod-refresh-full-release` must carry the expected `upstreamhash.txt` and `modversion.txt` values in the release plan/handoff and stop if the actual files diverge before publish.
- `$mod-release-current` must not fetch upstream. It must use the already checked-in `upstreamhash.txt` and `modversion.txt`; callers are responsible for updating and committing those files before current-HEAD release publishing.
- The publish skill must no longer derive the release suffix from `HEAD`, `git rev-parse --short`, or the upstream SemVer patch component.

## File Ownership

| File | Owner task | Change type | Responsibility | Parallel safety notes |
|---|---|---:|---|---|
| `upstreamhash.txt` | Metadata initialization | create | Store the initial full upstream SHA `2c7802e7cf3ad53733ca9fb603f270debcca280f`. | Unique write scope; safe in parallel with skill text edits. |
| `modversion.txt` | Metadata initialization | create | Store the initial mod sequence `1`. | Unique write scope; safe in parallel with skill text edits. |
| `.codex/skills/mod-refresh-preflight/SKILL.md` | Refresh flow metadata maintenance | modify | Require/report the upstream target SHA that feeds `upstreamhash.txt`. | Same owner as related refresh-flow docs to avoid conflicting process wording. |
| `.codex/skills/mod-refresh-full-release/SKILL.md` | Refresh flow metadata maintenance | modify | Include metadata file values in handoff requirements and stop conditions. | Same owner as related refresh-flow docs. |
| `.codex/skills/mod-refresh-release/SKILL.md` | Refresh flow metadata maintenance | modify | Require release plans to record/check metadata file values before publish. | Same owner as related refresh-flow docs. |
| `.codex/skills/mod-refresh-merge-preserve/SKILL.md` | Refresh flow metadata maintenance | modify | Instruct real upstream merge flow to update both metadata files. | Same owner as related refresh-flow docs. |
| `.codex/skills/mod-refresh-publish/SKILL.md` | Publish/current version contract | modify | Replace final-HEAD suffix contract with metadata-file version derivation and validation. | Unique write scope; safe in parallel with refresh-flow docs. |
| `.codex/skills/mod-release-current/SKILL.md` | Publish/current version contract | modify | Require current releases to consume checked-in metadata files without fetching upstream. | Unique write scope; safe in parallel with refresh-flow docs. |

## Implementation Tasks

### Metadata initialization

- **Goal:** Create the two root metadata files with the approved initial values.
- **Contract inputs:** Interface Contract entries for `upstreamhash.txt`, `modversion.txt`, and initial values.
- **Serialization required:** No. The files are uniquely owned.
- **Write scope:** `upstreamhash.txt`, `modversion.txt`.
- **Parallel:** Yes, compatible with `Refresh flow metadata maintenance` and `Publish/current version contract`.
- **Risk:** Low; static root text files.
- **Model tier:** FAST, resolved model `gpt-5.3-codex-spark`, reasoning effort `high`.
- **Worker role:** `sp-impl`.
- **Outputs and file-level responsibilities:** Create `upstreamhash.txt` with exactly `2c7802e7cf3ad53733ca9fb603f270debcca280f\n`; create `modversion.txt` with exactly `1\n`.
- **Implementation steps:** Use `apply_patch` to add both files. Do not edit release skills from this task.
- **Verification commands:** `timeout 30s bash -lc 'test "$(cat upstreamhash.txt)" = "2c7802e7cf3ad53733ca9fb603f270debcca280f" && test "$(cat modversion.txt)" = "1"'`.
- **Completion report requirements:** List created files, command result, and any unexpected pre-existing metadata files.

### Refresh flow metadata maintenance

- **Goal:** Update upstream-refresh skill docs so refresh runs maintain and verify the metadata files.
- **Contract inputs:** Interface Contract entries for preflight target SHA reporting, refresh merge metadata updates, release handoff checks, and no final-HEAD suffix derivation.
- **Serialization required:** No. This task owns all refresh-flow skill files and does not edit publish/current files.
- **Write scope:** `.codex/skills/mod-refresh-preflight/SKILL.md`, `.codex/skills/mod-refresh-full-release/SKILL.md`, `.codex/skills/mod-refresh-release/SKILL.md`, `.codex/skills/mod-refresh-merge-preserve/SKILL.md`.
- **Parallel:** Yes, compatible with `Metadata initialization` and `Publish/current version contract`.
- **Risk:** Medium; this is process documentation that gates future mutating release runs.
- **Model tier:** BEST, resolved model `gpt-5.5`, reasoning effort `xhigh`.
- **Worker role:** `sp-impl`.
- **Outputs and file-level responsibilities:** Preflight reports `upstream/main` target SHA; full-release/release handoffs record expected metadata file values; merge-preserve updates `upstreamhash.txt` and resets or explicitly sets `modversion.txt`; stop conditions cover missing, malformed, or divergent metadata files.
- **Implementation steps:** Edit only the owned skill Markdown files. Keep subagent policy text aligned with existing same-model/high or Simple Power overrides. Do not introduce build, test, tag, or publish commands.
- **Verification commands:** `timeout 30s rg -n "upstreamhash.txt|modversion.txt|upstream target SHA" .codex/skills/mod-refresh-preflight/SKILL.md .codex/skills/mod-refresh-full-release/SKILL.md .codex/skills/mod-refresh-release/SKILL.md .codex/skills/mod-refresh-merge-preserve/SKILL.md`.
- **Completion report requirements:** List changed files, summarize the metadata maintenance behavior added to each skill, report verification command result, and flag any wording that still points to final `HEAD` as release suffix source.

### Publish/current version contract

- **Goal:** Update publish and current-release skill docs so all mod releases consume checked-in metadata files for the version.
- **Contract inputs:** Interface Contract entries for metadata file validation, version computation, same tag/release version, current release no-fetch behavior, and final-HEAD suffix removal.
- **Serialization required:** No. This task owns publish/current skill files and does not edit refresh-flow files.
- **Write scope:** `.codex/skills/mod-refresh-publish/SKILL.md`, `.codex/skills/mod-release-current/SKILL.md`.
- **Parallel:** Yes, compatible with `Metadata initialization` and `Refresh flow metadata maintenance`.
- **Risk:** High; this changes the release naming contract used immediately before remote tag/release mutation.
- **Model tier:** BEST, resolved model `gpt-5.5`, reasoning effort `xhigh`.
- **Worker role:** `sp-impl`.
- **Outputs and file-level responsibilities:** Publish reads and validates `upstreamhash.txt` and `modversion.txt`; computes `version="${base_series}.${upstream_short}.${mod_version}.mod"`; uses the same `version` for annotated Git tag, pushed tag, GitHub release tag, and title; keeps `codex` as the asset; removes final-HEAD self-referential suffix wording. Current-release requires checked-in metadata and keeps the no-upstream-fetch contract.
- **Implementation steps:** Replace the publish Version Contract snippet with the metadata-file snippet; update safety checks and reviewer checklist; update completion reporting to include upstream SHA and mod version; update `$mod-release-current` required state, handoff, reviewer gate, and stop conditions.
- **Verification commands:** `timeout 30s bash -lc 'upstream_sha="$(cat upstreamhash.txt 2>/dev/null || printf "%s" 2c7802e7cf3ad53733ca9fb603f270debcca280f)"; mod_version="$(cat modversion.txt 2>/dev/null || printf "%s" 1)"; upstream_short="$(printf "%s" "$upstream_sha" | cut -c1-5)"; version="0.141.${upstream_short}.${mod_version}.mod"; test "$version" = "0.141.2c780.1.mod"'` and `timeout 30s rg -n "upstreamhash.txt|modversion.txt|gh release create|git tag -a" .codex/skills/mod-refresh-publish/SKILL.md .codex/skills/mod-release-current/SKILL.md`.
- **Completion report requirements:** List changed files, quote the computed example version `0.141.2c780.1.mod`, report verification command results, and flag any remaining `HEAD` suffix wording.

## Model Allocation

| Stage | Role | Model tier | Resolved model | Reasoning effort | Reason |
|---|---|---|---|---|---|
| Plan review | REVIEW-tier plan reviewer | REVIEW | `gpt-5.5` | `xhigh` | Required by writing-plans; reviews authoritative execution plan. |
| Metadata initialization | `sp-impl` | FAST | `gpt-5.3-codex-spark` | `high` | Static creation of two simple text files. |
| Refresh flow metadata maintenance | `sp-impl` | BEST | `gpt-5.5` | `xhigh` | Cross-skill release process wording that affects future mutating refreshes. |
| Publish/current version contract | `sp-impl` | BEST | `gpt-5.5` | `xhigh` | High-risk release naming and remote publish contract. |
| Quick verification | FAST-tier quick verifier | FAST | `gpt-5.3-codex-spark` | `high` | Mechanical validation of metadata files and Markdown skill references. |
| Final review+fix | REVIEW-tier review+fix agent | REVIEW | `gpt-5.5` | `xhigh` | Required whole-implementation review before final verification. |

## Plan Review

- Self-review must confirm the Design Summary, Interface Contract, File Ownership, Implementation Tasks, Model Allocation, quick/final verification, approved-path enforcement, and exact three-checkpoint commit policy are present and consistent.
- Coordinator creates `refs/simplepower/scratch/<run-id>/plan-review/before` for `docs/simplepower/plans/2026-06-18-mod-refresh-upstream-metadata-versioning.md` before first reviewer dispatch. Run id format is `YYYYMMDD-HHMMSS-<short-head>`.
- Dispatch one REVIEW-tier plan reviewer using `/home/gary/.codex/simplepower/skills/writing-plans/plan-document-reviewer-prompt.md`. The reviewer must perform review directly, must not run Codex CLI, must not spawn subagents, must not invoke Simple Power skills, must not restart execution, and must not reroute the workflow.
- If the reviewer finds blocking issues, the coordinator edits only this plan, creates `refs/simplepower/scratch/<run-id>/plan-review/after-<n>`, and sends the same reviewer this diff command: `git diff refs/simplepower/scratch/<run-id>/plan-review/before refs/simplepower/scratch/<run-id>/plan-review/after-<n> -- docs/simplepower/plans/2026-06-18-mod-refresh-upstream-metadata-versioning.md`. Later revisions compare the prior `after-<n>` ref to the new one.
- After reviewer approval, ask the user for combined approval of the reviewed plan, model/task allocation, and immediate current-session execution. Do not create the accepted plan checkpoint until that combined approval is given.

## Quick Verification

- Runs after all `sp-impl` file-edit workers complete and before the quick-verified implementation checkpoint.
- Coordinator creates `refs/simplepower/scratch/<run-id>/quick-verifier/before` for all implementation-owned files before dispatch.
- Quick verifier uses FAST tier and may fix only tiny typo-level issues. It must report behavior changes, structural edits, test rewrites, public interface changes, or unclear issues instead of fixing them.
- Commands:
  - `timeout 30s bash -lc 'test "$(cat upstreamhash.txt)" = "2c7802e7cf3ad53733ca9fb603f270debcca280f" && test "$(cat modversion.txt)" = "1"'`
  - `timeout 30s bash -lc 'upstream_sha="$(tr -d "[:space:]" < upstreamhash.txt)"; mod_version="$(tr -d "[:space:]" < modversion.txt)"; case "$upstream_sha" in [0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f]) ;; *) exit 1 ;; esac; case "$mod_version" in ""|0|*[!0-9]*) exit 1 ;; esac; test "0.141.$(printf "%s" "$upstream_sha" | cut -c1-5).${mod_version}.mod" = "0.141.2c780.1.mod"'`
  - `timeout 30s rg -n "upstreamhash.txt|modversion.txt" .codex/skills/mod-refresh-preflight/SKILL.md .codex/skills/mod-refresh-full-release/SKILL.md .codex/skills/mod-refresh-release/SKILL.md .codex/skills/mod-refresh-merge-preserve/SKILL.md .codex/skills/mod-refresh-publish/SKILL.md .codex/skills/mod-release-current/SKILL.md`
- If the quick verifier edits files, coordinator creates `refs/simplepower/scratch/<run-id>/quick-verifier/after` and inspects `git diff refs/simplepower/scratch/<run-id>/quick-verifier/before refs/simplepower/scratch/<run-id>/quick-verifier/after -- upstreamhash.txt modversion.txt .codex/skills/mod-refresh-preflight/SKILL.md .codex/skills/mod-refresh-full-release/SKILL.md .codex/skills/mod-refresh-release/SKILL.md .codex/skills/mod-refresh-merge-preserve/SKILL.md .codex/skills/mod-refresh-publish/SKILL.md .codex/skills/mod-release-current/SKILL.md` before checkpoint.

## Final Review And Fix

- After the quick-verified implementation checkpoint, dispatch one REVIEW-tier review+fix agent over the full implementation.
- Coordinator creates `refs/simplepower/scratch/<run-id>/review-fix/before` for all implementation-owned files before dispatch.
- The review+fix agent may edit only the approved implementation-owned files to fix issues against this plan. It must not commit, run Codex CLI, spawn subagents, invoke Simple Power skills, restart execution, or reroute the workflow.
- If the review+fix agent edits files, coordinator creates `refs/simplepower/scratch/<run-id>/review-fix/after` and inspects `git diff refs/simplepower/scratch/<run-id>/review-fix/before refs/simplepower/scratch/<run-id>/review-fix/after -- upstreamhash.txt modversion.txt .codex/skills/mod-refresh-preflight/SKILL.md .codex/skills/mod-refresh-full-release/SKILL.md .codex/skills/mod-refresh-release/SKILL.md .codex/skills/mod-refresh-merge-preserve/SKILL.md .codex/skills/mod-refresh-publish/SKILL.md .codex/skills/mod-release-current/SKILL.md` before final verification.

## Commit Checkpoints

1. Accepted plan checkpoint: after the REVIEW-tier plan reviewer approves and the user gives combined approval for the reviewed plan, model/task allocation, and immediate current-session execution.
2. Quick-verified implementation checkpoint: after all `sp-impl` file edits complete and quick verification passes.
3. Final checkpoint: after the REVIEW-tier review+fix agent completes and final verification passes.

Workers, plan reviewers, quick verifiers, review+fix agents, and individual tasks must not commit. Scratch refs are coordinator-owned local review anchors only, live under `refs/simplepower/scratch/<run-id>/`, are not pushed or treated as accepted commits, and must be deleted after the successful checkpoint for their phase. If the workflow stops because of user direction, a blocker, or failed checkpoint commit, preserve remaining refs and report: `git for-each-ref --format='%(refname)' "refs/simplepower/scratch/<run-id>" | while read -r ref; do git update-ref -d "$ref"; done`.

## Current-Session Auto-Dispatch

After plan reviewer approval, ask for one combined approval covering the reviewed plan, model/task allocation, and immediate current-session execution. After combined approval, coordinator creates the accepted plan checkpoint commit, deletes successful `plan-review` scratch refs, and immediately invokes `simplepower:subagent-driven-development` in the current session with:

```text
Execute `docs/simplepower/plans/2026-06-18-mod-refresh-upstream-metadata-versioning.md` with aggregate parallel implementation from the approved Interface Contract. Use the approved FAST/NORMAL/BEST/REVIEW model allocation. Dispatch all non-conflicting `sp-impl` file-edit workers whose coordination needs are satisfied by their Contract inputs, run the quick FAST-tier verifier with lint/build/tests and timeouts after all workers finish, commit the quick-verified implementation, then run one REVIEW-tier review+fix agent, final verification, and final commit.
```

## Approved Path Enforcement

The accepted implementation plan is authoritative. Workers, verifiers, reviewers, and the coordinator must not switch to backup routes, reduce scope, substitute docs-only or placeholder work, skip verification, skip review, change the execution route, or publish/release anything outside the planned metadata-file contract. If any planned step is blocked, unsafe, mismatched with the repository, or impossible with the approved file ownership, stop and report the exact blocker, current file state, and needed decision before changing approach.

## Verification

- Final verification runs only after the REVIEW-tier review+fix agent completes.
- Commands:
  - `timeout 30s bash -lc 'test "$(cat upstreamhash.txt)" = "2c7802e7cf3ad53733ca9fb603f270debcca280f" && test "$(cat modversion.txt)" = "1"'`
  - `timeout 30s bash -lc 'upstream_sha="$(tr -d "[:space:]" < upstreamhash.txt)"; mod_version="$(tr -d "[:space:]" < modversion.txt)"; case "$upstream_sha" in [0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f]) ;; *) exit 1 ;; esac; case "$mod_version" in ""|0|*[!0-9]*) exit 1 ;; esac; test "0.141.$(printf "%s" "$upstream_sha" | cut -c1-5).${mod_version}.mod" = "0.141.2c780.1.mod"'`
  - `timeout 30s rg -n "upstreamhash.txt|modversion.txt|version=\"\\$\\{base_series\\}\\.\\$\\{upstream_short\\}\\.\\$\\{mod_version\\}\\.mod\"" .codex/skills/mod-refresh-publish/SKILL.md .codex/skills/mod-release-current/SKILL.md .codex/skills/mod-refresh-release/SKILL.md .codex/skills/mod-refresh-full-release/SKILL.md .codex/skills/mod-refresh-merge-preserve/SKILL.md .codex/skills/mod-refresh-preflight/SKILL.md`
- Expected result: all commands exit 0, proving metadata files have exact initial values, the derived version is `0.141.2c780.1.mod`, and the skill docs reference the new metadata contract.
- Failure means the implementation does not satisfy the approved versioning contract or metadata initialization and must be fixed before final checkpoint.
- Coordinator performs the final checkpoint only after the REVIEW-tier review+fix agent has completed and these final commands pass.
- Final reporting runs cleanup check: `git for-each-ref --format='%(refname)' "refs/simplepower/scratch/<run-id>"`. If the final checkpoint succeeds, no refs for the run should remain after phase cleanup; otherwise preserve refs and report the manual cleanup command from Commit Checkpoints.
