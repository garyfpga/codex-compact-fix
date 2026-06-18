---
name: "mod-refresh-preflight"
description: "Use this skill when asked to check whether pulling upstream Codex changes may conflict with or affect fork-local compact-fix mods before a mod refresh."
---

# Mod Refresh Preflight

Run this skill before refreshing the compact-fix fork against upstream Codex. It fetches `upstream/main`, performs only non-mutating analysis, maps upstream changes against `docs/compact-fix/ChangeLog.md`, and reports whether release work should proceed.

## Default Stance

- Report-only is the default. Do not merge into the user's branch, resolve conflicts, build, tag, publish, or invoke `mod-refresh-release` unless the user explicitly asks to continue after seeing the preflight report.
- Treat `docs/compact-fix/ChangeLog.md` as the compact preservation source of truth. Start impact analysis there before reading implementation code.
- For new feature work, tell the user to use `simplepower:brainstorming` and explicitly ask brainstorming to run `mod-refresh-preflight` during context exploration.

## Workflow

1. Fetch upstream:
   - Run `git fetch upstream main:refs/remotes/upstream/main`.
   - If `upstream` is missing or the fetch fails, stop and report the blocker.

2. Check the worktree:
   - Run `git status --short`.
   - Require a clean worktree unless the user explicitly asked for dirty-tree analysis.
   - If dirty and not explicitly allowed, stop before merge simulation and report the changed paths.

3. Summarize upstream file changes:
   - Identify the merge base with `git merge-base HEAD upstream/main`.
   - Summarize upstream-touched files with `git diff --name-status <merge-base>..upstream/main`.
   - Highlight files and directories that overlap compact-fix ChangeLog entries.

4. Simulate the merge without mutating the user's branch:
   - Prefer a temporary worktree: create it from `HEAD`, run `git merge --no-commit --no-ff upstream/main` inside it, collect results, abort any in-progress merge, and remove the worktree.
   - A disposable branch is acceptable only when a temporary worktree is impractical. Delete it before finishing.
   - Never leave merge state, generated files, or branch switches behind in the user's working tree.

5. Analyze conflicts and compact impact:
   - Summarize conflicted files, conflict types, and likely owners.
   - Map upstream-touched and conflicted files to the compact behavior groups in `docs/compact-fix/ChangeLog.md`.
   - Call out direct hits, adjacent-risk files, test/snapshot/schema implications, and any areas with no apparent compact impact.

6. Report and gate continuation:
   - End with a report using the format below.
   - If the user explicitly asks to proceed after the report, hand off to `mod-refresh-release`.
   - If the report shows unresolved blockers, ask for direction instead of continuing.

## Subagent Policy

When using subagents for this skill, use the same model as main agent and `reasoning_effort = high`.

- `preflight-git-analyzer`: Fetch state, clean-worktree status, upstream file summary, and merge-simulation conflict inventory.
- `compact-impact-analyzer`: Map upstream changes and conflicts to `docs/compact-fix/ChangeLog.md` preservation groups.
- `release-risk-reviewer`: Review the report for mutation risk, missing blockers, and whether release continuation is justified.

## Report Format

Use concise sections:

- `Fetch`: upstream ref fetched, merge base, and any fetch concerns.
- `Worktree`: clean or dirty, with dirty paths if relevant.
- `Merge simulation`: clean merge or conflict summary, including conflicted files.
- `Upstream touched files`: grouped by compact-relevant areas and unrelated areas.
- `Compact impact`: ChangeLog behavior groups affected, risk level, and preservation notes.
- `Recommendation`: `report-only`, `ready to continue if explicitly requested`, or `blocked`.
- `Continuation gate`: state that `mod-refresh-release` will only run on explicit user request.
