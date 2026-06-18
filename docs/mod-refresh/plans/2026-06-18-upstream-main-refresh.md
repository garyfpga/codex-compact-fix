# Mod Refresh Release Plan: upstream/main refresh

Date: 2026-06-18

## Objective

Refresh the `refresh1` compact-fix fork branch with `upstream/main`, preserve the fork-local compact behavior recorded in `docs/compact-fix/ChangeLog.md`, build the Linux Codex CLI release artifact, then publish a `.mod` GitHub release.

## Fresh Preflight

Source: current-session `$mod-refresh-full-release` preflight run after the user request on 2026-06-18.

Summary:

- Current branch: `refresh1`
- Current branch upstream: `origin/refresh1`
- Current HEAD: `83144de0a6bbc992410e9a03b0b5532c7b756913`
- Upstream ref fetched: `upstream/main`
- Upstream target SHA: `2c7802e7cf3ad53733ca9fb603f270debcca280f`
- Merge base: `f42780109c6646463f74eb8c8cf484437fedcca3`
- Worktree before preflight: clean
- Upstream touched paths: 1248
- Merge simulation: conflicted in the surfaced compact/API/client/session/task preservation files listed below
- Preflight recommendation: ready to continue if explicitly requested; request was `$mod-refresh-full-release`
- Reviewer result: PASS, with high preservation risk and required stop on any unsurfaced compact behavior choice

Simulated conflict files:

- `codex-rs/codex-api/src/endpoint/compact.rs`
- `codex-rs/codex-api/src/endpoint/session.rs`
- `codex-rs/core/src/client.rs`
- `codex-rs/core/src/compact.rs`
- `codex-rs/core/src/compact_remote.rs`
- `codex-rs/core/src/compact_remote_v2.rs`
- `codex-rs/core/src/session/turn.rs`
- `codex-rs/core/src/tasks/compact.rs`

## Release Decisions

- Tests: not run unless explicitly requested
- Bazel: not used; using Cargo release build only
- Release source: final post-merge `HEAD` on `refresh1`
- Expected release tag: computed after the final release commit as `0.139.<short5>.mod`
- Expected artifact name: `codex-0.139.<short5>.mod-linux`, where `<short5>` is the first five characters of the final release commit SHA
- Expected build source binary: `codex-rs/target/release/codex`
- Expected repo-root artifact path before publish: `codex-0.139.<short5>.mod-linux`
- GitHub publish target: `garyfpga/codex-compact-fix`
- Git tag flow: create an annotated tag on the final release commit SHA, push that tag to `origin`, then create the GitHub release with `--verify-tag`
- GitHub publish command: `gh release create "${version}" "${artifact}" --repo garyfpga/codex-compact-fix --verify-tag --title "${version}" --notes-file "${notes_file}"`
- Release notes source: the notes in this plan under `Release Notes`

## Release Notes

Upstream refresh of the compact-fix fork to `openai/codex` `upstream/main` at `2c7802e7cf3ad53733ca9fb603f270debcca280f`, preserving the fork-local remote compact behavior and `0.139.0+gary` TUI display label.

## Ordered Checklist

1. Reconfirm branch, clean worktree, remotes, and fetch `upstream/main`.
   - Before the real merge, verify `upstream/main` still equals `2c7802e7cf3ad53733ca9fb603f270debcca280f`, `HEAD` still equals `83144de0a6bbc992410e9a03b0b5532c7b756913`, and the merge base is still `f42780109c6646463f74eb8c8cf484437fedcca3`.
   - Run `git status --short --untracked-files=all` and allow only `?? docs/mod-refresh/plans/2026-06-18-upstream-main-refresh.md` as expected pre-merge dirtiness.
2. Merge `upstream/main` into `refresh1`.
3. Resolve compact conflicts while preserving `docs/compact-fix/ChangeLog.md`.
4. Run only required non-test maintenance checks.
5. Record skipped tests and skipped Bazel decisions.
6. Run `cd codex-rs && just fmt` before building if code changed.
7. Run `cd codex-rs && cargo build -p codex-cli --release`.
8. Compute the final release version from the final release commit.
9. Create a release notes file from the exact `Release Notes` block in this plan.
10. Copy `codex-rs/target/release/codex` to `codex-0.139.<short5>.mod-linux` in the repo root and preserve executable permissions.
11. Verify the artifact path, size, executable bit, SHA-256, source binary path, version output, and version-matching name.
12. Confirm no local tag, remote tag, or GitHub release exists for the computed version.
13. Create the annotated git tag on the final release commit SHA, push that tag to `origin`, and create the GitHub release against `garyfpga/codex-compact-fix` with `--verify-tag`.

## Preservation Risks

Carry these `docs/compact-fix/ChangeLog.md` groups through merge review:

- `remote_compact` config schema, validation bounds, defaults, effective config, tests, and `codex-rs/core/config.schema.json`
- Shared remote-first fallback wrapper as the single policy owner for auto and manual compact routing
- Compact-only fast service tier behavior, including API-key auth omission of the remote tier override
- Auto and manual compact routing through the shared wrapper, V2 feature-flag selection, and local-only provider behavior
- Compact transport boundaries: explicit retry settings, zero hidden V1 transport retries, configured timeout, TCP keepalive, and normal client headers/proxy/CA/cookie behavior
- Ordinary Responses retry behavior unchanged by compact-specific retry policy
- V1 warning categories, attempt counts, timeout wording, fallback warnings, fallback warning counts, and clean-history restore behavior
- V2 policy parity with version-specific warnings, visible attempt budget, timeout semantics, request-shape parity where intended, and no hidden stream retries that inflate visible attempts
- Compact integration tests, parity tests, config tests, and snapshots preserved as source artifacts
- TUI display-only version label remains `0.139.0+gary` and is not routed back to `CARGO_PKG_VERSION`
- Simple Power plan history under `docs/simplepower/plans/` remains intact

Additional adjacent risk:

- Upstream changed broader config loading, requirements, thread config, and auth-keyring files. Recheck `remote_compact` schema/config behavior after conflict resolution.
- Upstream changed `Cargo.toml` and `Cargo.lock`; after merge, explicitly check final dependency-file changes and run required dependency lock maintenance if dependency files changed.

## Maintenance Policy

Allowed without explicit test approval:

- `cd codex-rs && just fmt`
- Schema generation if config API changes require it
- Snapshot review or acceptance only for intentionally changed generated UI/text artifacts
- Dependency lock maintenance if dependencies changed
- Cargo release build for `codex-cli`

Dependency-file gate:

```bash
git diff --name-only 83144de0a6bbc992410e9a03b0b5532c7b756913..HEAD -- Cargo.toml Cargo.lock 'codex-rs/**/Cargo.toml' 'codex-rs/Cargo.lock'
```

If this reports dependency-file changes, run these non-test maintenance commands from the repository root and record the results:

```bash
just bazel-lock-update
just bazel-lock-check
```

These commands are dependency lock maintenance only; they do not authorize Bazel as the release build or test path.

Not allowed unless explicitly requested:

- `just test`
- `cargo test`
- Bazel build commands
- Bazel tests
- Full upstream test suites
- Focused upstream test commands

## Stop Conditions

Stop before continuing and ask for direction if:

- Worktree state diverges from the preflight assumptions before merge.
- `upstream/main`, `HEAD`, or merge base no longer match the preflight values before merge.
- Pre-merge `git status --short --untracked-files=all` shows anything other than the expected release plan file.
- Merge preservation exposes a compact behavior choice not surfaced by preflight.
- Conflict resolution cannot clearly preserve the ChangeLog behavior.
- Build fails or `codex-rs/target/release/codex` is missing.
- The artifact name does not include the final computed version.
- Local tag, remote tag, or GitHub release already exists for the computed version.
- Release notes, publish destination, or artifact destination becomes ambiguous.
- `git tag`, `git push origin "refs/tags/${version}"`, or `gh release create` partially succeeds and a later publish step fails.
- Tag deletion, release deletion, artifact overwrite, or publish retry is needed after a partial failure.

## Required Command Details

Pre-merge stale-assumption checks:

```bash
git fetch upstream main
test "$(git rev-parse upstream/main)" = "2c7802e7cf3ad53733ca9fb603f270debcca280f"
test "$(git rev-parse HEAD)" = "83144de0a6bbc992410e9a03b0b5532c7b756913"
test "$(git merge-base HEAD upstream/main)" = "f42780109c6646463f74eb8c8cf484437fedcca3"
git status --short --untracked-files=all
```

Only acceptable pre-merge status output:

```text
?? docs/mod-refresh/plans/2026-06-18-upstream-main-refresh.md
```

Artifact and notes commands:

```bash
final_sha="$(git rev-parse HEAD)"
short5="$(git rev-parse HEAD | cut -c1-5)"
version="0.139.${short5}.mod"
artifact="codex-${version}-linux"
notes_file="$(mktemp)"
cat >"${notes_file}" <<'NOTES'
Upstream refresh of the compact-fix fork to `openai/codex` `upstream/main` at `2c7802e7cf3ad53733ca9fb603f270debcca280f`, preserving the fork-local remote compact behavior and `0.139.0+gary` TUI display label.
NOTES

cp codex-rs/target/release/codex "${artifact}"
chmod +x "${artifact}"
stat -c '%n %s %A' "${artifact}"
sha256sum "${artifact}"
"./${artifact}" --version
```

Publish commands:

```bash
git tag -a "${version}" "${final_sha}" -m "${version}"
git push origin "refs/tags/${version}"
gh release create "${version}" "${artifact}" \
  --repo garyfpga/codex-compact-fix \
  --verify-tag \
  --title "${version}" \
  --notes-file "${notes_file}"
```

If `git tag`, `git push`, or `gh release create` partially succeeds and a later publish step fails, stop and ask for explicit recovery direction before deleting tags, deleting releases, overwriting artifacts, or retrying publish.

## Execution Log

- Preflight: completed; reviewer PASS.
- Plan review: initial reviewer BLOCK; plan amended for explicit final-SHA tag flow, stale-ref gates, expected plan-file dirtiness, dependency maintenance commands, and artifact/notes commands.
- Plan review: amended plan reviewer PASS.
- Merge preservation: real `git merge upstream/main` started from `refresh1`; conflicts matched preflight in compact/API/client/session/task files.
- Merge preservation: resolved by preserving the shared remote-first fallback wrapper, explicit V1/V2 visible attempt policy, compact-only service-tier override, upstream `CodexResponsesMetadata` plumbing, and V1 turn-state passthrough.
- Merge preservation: `merge-conflict-worker` found no blocking unsurfaced behavior choice.
- Maintenance: `cd codex-rs && just fmt` completed successfully.
- Maintenance: dependency files changed; `just bazel-lock-update` completed successfully.
- Maintenance: `just bazel-lock-check` completed successfully. This was dependency lock maintenance, not a Bazel build/test path.
- Merge preservation review: `compact-preservation-reviewer` PASS.
- Tests: not run unless explicitly requested.
- Bazel: not used; using Cargo release build only.
- Build: pending.
- Publish: pending.
