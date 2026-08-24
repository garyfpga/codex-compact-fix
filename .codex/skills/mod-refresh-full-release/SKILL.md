---
name: mod-refresh-full-release
description: Refresh the minimal Codex fork from upstream/main, preserve its trust-all contract, build the optimized Linux CLI, and publish a commit-derived GitHub release. Use only after the user explicitly authorizes the complete refresh-and-release mutation.
---

# Minimal Fork Full Release

Refresh and publish `main-fork-2026-08-24` as one guarded workflow. The user's full-release request authorizes the fetch/merge, focused maintenance, branch push, tag push, and GitHub release described here. It does not authorize changing the fork contract, deleting ambiguous remote state, or retrying a partially successful publication.

## Fork Contract

Preserve only these fork-local items across upstream merges:

- `dangerously_trust_all_projects`: unknown projects load project-local `.codex` layers; when permissions are otherwise unspecified, runtime permissions default to danger-full-access; when approvals are otherwise unspecified, approval policy defaults to `never`. Explicit project trust, approval, sandbox/permission settings, and managed constraints remain authoritative.
- `CODEX_CLI_RELEASE_VERSION` compile-time version overrides in the CLI and TUI, with `CARGO_PKG_VERSION` fallback for ordinary builds.
- This skill, its UI metadata, the accepted Simple Power execution record, and `.vscode/settings.json`.

Do not restore historical compact, retry, timer, model, multi-agent, `upstreamhash.txt`, `modversion.txt`, `.mod` counter, or compact-preservation behavior.

## Preconditions

Before mutation, require all of the following:

- Current branch is exactly `main-fork-2026-08-24`.
- Tracked worktree and index are clean. Repository-root `codex` may be ignored.
- `upstream` fetches `openai/codex`; `origin` pushes `garyfpga/codex-compact-fix`.
- `gh auth status` succeeds and `gh repo view garyfpga/codex-compact-fix` resolves the intended repository.
- No in-progress merge, rebase, cherry-pick, or revert exists.

Stop on any mismatch. Do not stash unrelated changes, switch branches, reset files, or reinterpret another branch as the release source.

## Refresh Upstream

Fetch current upstream with:

```bash
git fetch upstream main:refs/remotes/upstream/main
```

Record the fetched SHA. If it is already an ancestor of `HEAD`, record a no-op refresh. Otherwise start a non-fast-forward, no-commit merge:

```bash
git merge --no-commit --no-ff upstream/main
```

Resolve only conflicts whose correct result unambiguously preserves the Fork Contract. If upstream changed the meaning or location of a contract item, stop with the conflicting paths and decision instead of choosing a new behavior.

Before committing a real merge, regenerate and verify the focused surface:

```bash
cd codex-rs
just write-config-schema
just fmt
just fmt-check
just test -p codex-core unknown_project_layer_enabled_when_trust_all
just test -p codex-core explicit_untrusted_project_layer_disabled_when_trust_all
just test -p codex-core dangerously_trust_all_projects_permission_defaults_and_precedence
```

Do not run the full workspace suite or Bazel. Review the merge diff for old-feature reintroduction, then commit the merge. Require a clean tracked worktree again. The resulting clean `HEAD` is the release source.

## Derive the Version

Query the latest stable upstream release at execution time:

```bash
fork_latest_release="$(gh release list --repo openai/codex --exclude-drafts --exclude-pre-releases --limit 1 --json name,tagName --jq '.[0] | (.name // .tagName)')"
```

Strip an optional `rust-v` or `v`, require exact `0.X.Y`, then remove only the leading `0.`. Let `fork_source_sha` be the full lowercase `git rev-parse HEAD`; let `fork_source_short` be exactly its first five characters. Compute:

```text
fork_version = X.Y.<fork_source_short>
```

Require `fork_version` to match `^[0-9]+\.[0-9]+\.[0-9a-f]{5}$`. For official `rust-v0.149.1` and source `abcde...`, the result is `149.1.abcde`.

Before building, require that `${fork_version}` is absent as a local tag, an origin tag, and a GitHub release. Also inspect `refs/heads/main-fork-2026-08-24` on origin: it may be absent or an ancestor of the local source, but it must not diverge.

## Build and Verify

From `codex-rs`, use upstream's optimized Cargo release profile and build only the CLI:

```bash
CODEX_CLI_RELEASE_VERSION="${fork_version}" cargo build -p codex-cli --release
```

Do not use a custom profile or Bazel. Copy `target/release/codex` to repository-root `codex`, preserving executable permissions. Record source and destination paths, size, file type, and SHA-256. Require exact output:

```text
codex-cli <fork_version>
```

Stop if the artifact is absent, not executable, has an unexpected platform/type, or reports any other version.

## Publish

Push the source branch first:

```bash
git push -u origin HEAD:refs/heads/main-fork-2026-08-24
```

Create an annotated tag at the exact source SHA and push that tag:

```bash
git tag -a "${fork_version}" "${fork_source_sha}" -m "${fork_version}"
git push origin "refs/tags/${fork_version}"
```

Create the release in `garyfpga/codex-compact-fix` with exact tag/title `${fork_version}`, upload repository-root `codex` under its stable asset name, and use concise notes naming the official upstream base plus full fork source SHA.

If branch push, tag creation/push, or release creation partially succeeds, inspect local tag, remote branch/tag, and GitHub release state. Report the exact partial state and stop. Never delete, move, reuse, overwrite, or blindly retry a tag/release without fresh user approval.

## Completion

Report the upstream SHA, whether refresh merged or was a no-op, source SHA, official base release, derived version/tag, focused checks, build command, artifact checksum/type/version, branch push, release URL, and any partial state. Return the verified repository-root `codex` path to the invoking workflow for separately authorized deployment.
