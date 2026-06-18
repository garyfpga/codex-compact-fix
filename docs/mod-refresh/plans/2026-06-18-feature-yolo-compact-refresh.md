# Mod Refresh Release Plan: feature yolo compact refresh

Date: 2026-06-18

## Objective

Merge `upstream/main` into `feat/yolo-default-compact-timer`, preserve the fork-local compact behavior recorded in `docs/compact-fix/ChangeLog.md`, preserve the new trust-all-projects and TUI compact timer feature work, build the Linux Codex CLI artifact, then publish a metadata-based `.mod` GitHub release.

## Fresh Preflight

Source: current-session `$mod-refresh-full-release` preflight run after the user's feature and release request on 2026-06-18.

Summary:

- Current branch: `feat/yolo-default-compact-timer`
- Current HEAD: `7ca701b2f9428018aaf9a22bc545f732a2c82a6b`
- Upstream ref fetched: `upstream/main`
- Upstream target SHA: `c73296a0f095e72dbb646909c613ae09c9459c3a`
- Merge base: `2c7802e7cf3ad53733ca9fb603f270debcca280f`
- Worktree before preflight: clean
- Upstream commit: `c73296a0f0 [codex] Pass plugin namespace into skill loading (#28608)`
- Upstream touched paths: 11 plugin/skill loader and executor files
- Merge simulation: clean merge in a temporary worktree, no conflicts
- Preflight recommendation: ready to continue if explicitly requested; request was `$mod-refresh-full-release`
- Preflight reviewer result: approved to continue

Upstream touched files:

- `codex-rs/core-plugins/src/loader.rs`
- `codex-rs/core-plugins/src/manager.rs`
- `codex-rs/core-plugins/src/manager_tests.rs`
- `codex-rs/core-skills/src/loader.rs`
- `codex-rs/core-skills/src/loader_tests.rs`
- `codex-rs/core-skills/src/service.rs`
- `codex-rs/core-skills/src/service_tests.rs`
- `codex-rs/ext/skills/src/provider/executor.rs`
- `codex-rs/ext/skills/tests/executor_file_system_authority.rs`
- `codex-rs/plugin/src/load_outcome.rs`
- `codex-rs/utils/plugins/src/lib.rs`

## Release Decisions

- Tests: not run unless explicitly requested
- Bazel: not used; using Cargo release build only
- Release source: final post-merge `HEAD` on `feat/yolo-default-compact-timer`
- Expected artifact path: repository-root `codex`
- Expected build source binary: `codex-rs/target/release/codex`
- Expected `upstreamhash.txt`: `c73296a0f095e72dbb646909c613ae09c9459c3a`
- Expected `modversion.txt`: `1`
- Release version contract: `<latest-upstream-major>.<latest-upstream-minor>.c7329.1.mod`
- Release suffix source: `<first5-upstreamhash>.<modversion>.mod`; do not derive the suffix from final `HEAD`, `git rev-parse --short`, or the upstream SemVer patch component
- GitHub publish target: `garyfpga/codex-compact-fix`
- Git tag flow: create an annotated tag on the final release commit SHA, push that tag to `origin`, then create the GitHub release with `--verify-tag`
- GitHub publish command: `gh release create "${version}" "codex" --repo garyfpga/codex-compact-fix --verify-tag --title "${version}" --notes-file "${notes_file}"`
- Release notes source: the notes in this plan under `Release Notes`

## Release Notes

Refresh to `openai/codex` `upstream/main` at `c73296a0f095e72dbb646909c613ae09c9459c3a`, preserving the compact-fix fork behavior and adding `dangerously_trust_all_projects` plus the TUI compact timer/status updates.

## Ordered Checklist

1. Reconfirm branch, clean worktree, remotes, and fetch `upstream/main`.
2. Before the real merge, verify `upstream/main` still equals `c73296a0f095e72dbb646909c613ae09c9459c3a`, `HEAD` still equals `7ca701b2f9428018aaf9a22bc545f732a2c82a6b`, and the merge base is still `2c7802e7cf3ad53733ca9fb603f270debcca280f`.
3. Run `git status --short --untracked-files=all` and allow only this release plan file as expected pre-merge dirtiness.
4. Merge `upstream/main` into `feat/yolo-default-compact-timer`.
5. If conflicts appear, resolve while preserving `docs/compact-fix/ChangeLog.md` and the feature plan contract in `docs/simplepower/plans/2026-06-18-yolo-default-and-compact-timer.md`.
6. Update `upstreamhash.txt` to `c73296a0f095e72dbb646909c613ae09c9459c3a` and `modversion.txt` to `1`.
7. Verify both metadata files exactly match the expected values and shape.
8. Run only required non-test maintenance checks.
9. Record skipped tests and skipped Bazel decisions.
10. Commit the merge, metadata, and release-plan updates.
11. Run `cd codex-rs && just fmt` before building if code changed after the merge commit.
12. Run `cd codex-rs && cargo build -p codex-cli --release`.
13. Copy `codex-rs/target/release/codex` to repository-root `codex` and preserve executable permissions.
14. Verify artifact path, size, executable bit, source binary path, and version output.
15. Compute the final release version from the latest stable upstream Codex release and checked-in metadata files.
16. Create a release notes file from the exact `Release Notes` block in this plan.
17. Confirm no local tag, remote tag, or GitHub release exists for the computed version.
18. Run the release packaging reviewer gate.
19. Create the annotated git tag on the final release commit SHA, push that tag to `origin`, and create the GitHub release against `garyfpga/codex-compact-fix` with `--verify-tag`.

## Preservation Risks

Carry these `docs/compact-fix/ChangeLog.md` groups through merge review:

- Preserve `remote_compact` config parsing, validation bounds, defaults, effective config, tests, and `codex-rs/core/config.schema.json` generation.
- Preserve the shared remote-first fallback wrapper as the single policy owner for auto and manual compact routing.
- Preserve compact-only fast service tier behavior, including API-key auth omission of the remote tier override and the new already-fast status message.
- Preserve auto and manual compact routing through the shared wrapper, V2 feature-flag selection, and local-only provider behavior.
- Preserve compact transport boundaries: explicit retry settings, zero hidden V1 transport retries, configured timeout, TCP keepalive, and normal client headers/proxy/CA/cookie behavior.
- Preserve ordinary Responses retry behavior unchanged by compact-specific retry policy.
- Preserve V1 warning categories, attempt counts, timeout wording, fallback warnings, fallback warning counts, and clean-history restore behavior.
- Preserve V2 policy parity with version-specific warnings, visible attempt budget, timeout semantics, request-shape parity where intended, and no hidden stream retries that inflate visible attempts.
- Preserve compact integration tests, parity tests, config tests, and snapshots as source artifacts. Do not run tests unless explicitly requested.
- Preserve the TUI display-only version label and do not route display surfaces back to `CARGO_PKG_VERSION`.
- Preserve the Simple Power plan history under `docs/simplepower/plans/`.
- Preserve `upstreamhash.txt` and `modversion.txt` as release metadata.

Additional current-run risks:

- Upstream changed plugin/skill loading and executor scope only. This is not a direct compact hit, but it is adjacent to Simple Power skill execution, so keep the new plan files and skill-driven branch rationale intact.
- The feature branch changed config loading and TUI compact lifecycle display. If upstream merge or formatting touches those files, recheck `dangerously_trust_all_projects` precedence and compact timer messages.

## Maintenance Policy

Allowed without explicit test approval:

- `cd codex-rs && just fmt`
- `cd codex-rs && just write-config-schema` if config schema generation is required
- Snapshot review or acceptance only for intentionally changed generated UI/text artifacts
- Dependency lock maintenance if dependencies changed
- Cargo release build for `codex-cli`

Not allowed unless explicitly requested:

- `just test`
- `cargo test`
- Bazel build commands
- Bazel tests
- Full upstream test suites
- Focused upstream test commands

Dependency-file gate:

```bash
git diff --name-only 7ca701b2f9428018aaf9a22bc545f732a2c82a6b..HEAD -- Cargo.toml Cargo.lock 'codex-rs/**/Cargo.toml' 'codex-rs/Cargo.lock'
```

If this reports dependency-file changes, run these non-test maintenance commands from the repository root and record the results:

```bash
just bazel-lock-update
just bazel-lock-check
```

These commands are dependency lock maintenance only; they do not authorize Bazel as the release build or test path.

## Stop Conditions

Stop before continuing and ask for direction if:

- Worktree state diverges from the preflight assumptions before merge.
- `upstream/main`, `HEAD`, or merge base no longer match the preflight values before merge.
- Pre-merge `git status --short --untracked-files=all` shows anything other than the expected release plan file.
- Merge preservation exposes a compact or feature behavior choice not surfaced by preflight.
- Conflict resolution cannot clearly preserve the ChangeLog behavior or the accepted feature plan behavior.
- `upstreamhash.txt` or `modversion.txt` is missing, malformed, or divergent from this plan after the merge.
- Build fails or `codex-rs/target/release/codex` is missing.
- Repository-root artifact `codex` is missing or not executable.
- Local tag, remote tag, or GitHub release already exists for the computed version.
- Release notes, publish destination, or artifact destination becomes ambiguous.
- `git tag`, `git push origin "refs/tags/${version}"`, or `gh release create` partially succeeds and a later publish step fails.
- Tag deletion, release deletion, artifact overwrite, or publish retry is needed after a partial failure.

## Required Command Details

Pre-merge stale-assumption checks:

```bash
git fetch upstream main
test "$(git rev-parse upstream/main)" = "c73296a0f095e72dbb646909c613ae09c9459c3a"
test "$(git rev-parse HEAD)" = "7ca701b2f9428018aaf9a22bc545f732a2c82a6b"
test "$(git merge-base HEAD upstream/main)" = "2c7802e7cf3ad53733ca9fb603f270debcca280f"
git status --short --untracked-files=all
```

Only acceptable pre-merge status output:

```text
?? docs/mod-refresh/plans/2026-06-18-feature-yolo-compact-refresh.md
```

Artifact and notes commands:

```bash
notes_file="$(mktemp)"
cat >"${notes_file}" <<'NOTES'
Refresh to `openai/codex` `upstream/main` at `c73296a0f095e72dbb646909c613ae09c9459c3a`, preserving the compact-fix fork behavior and adding `dangerously_trust_all_projects` plus the TUI compact timer/status updates.
NOTES

cp codex-rs/target/release/codex codex
chmod +x codex
stat -c '%n %s %A' codex
sha256sum codex
./codex --version
```

Version computation:

```bash
final_commit="$(git rev-parse HEAD)"
latest_release="$(gh release list --repo openai/codex --exclude-drafts --exclude-pre-releases --limit 1 --json name,tagName --jq '.[0] | (.name // .tagName)')"
base_semver="${latest_release#rust-v}"
base_semver="${base_semver#v}"
base_series="$(printf '%s\n' "${base_semver}" | awk -F. '{print $1 "." $2}')"
upstream_sha="$(sed -n '1p' upstreamhash.txt)"
mod_version="$(sed -n '1p' modversion.txt)"
upstream_short="$(printf '%s' "${upstream_sha}" | cut -c1-5)"
version="${base_series}.${upstream_short}.${mod_version}.mod"
```

Publish commands:

```bash
git tag -a "${version}" "${final_commit}" -m "${version}"
git push origin "refs/tags/${version}"
gh release create "${version}" "codex" \
  --repo garyfpga/codex-compact-fix \
  --verify-tag \
  --title "${version}" \
  --notes-file "${notes_file}"
```

If `git tag`, `git push`, or `gh release create` partially succeeds and a later publish step fails, stop and ask for explicit recovery direction before deleting tags, deleting releases, overwriting artifacts, or retrying publish.

## Execution Log

- Preflight: completed after feature checkpoint `7ca701b2f9428018aaf9a22bc545f732a2c82a6b`.
- Preflight reviewer: approved continuation to `$mod-refresh-release`.
- Tests: not run unless explicitly requested.
- Bazel: not used; using Cargo release build only.
- Plan review: PASS.
- Merge preservation: `git merge upstream/main` completed cleanly with no conflicts.
- Metadata: `upstreamhash.txt` updated to `c73296a0f095e72dbb646909c613ae09c9459c3a`; `modversion.txt` remains `1`.
- Maintenance: `cd codex-rs && timeout 120s just fmt` completed successfully.
- Maintenance: dependency-file gate reported no dependency-file changes; no Bazel lock maintenance needed.
- Verification: `cd codex-rs && timeout 600s cargo check -p codex-core -p codex-tui -p codex-core-skills -p codex-core-plugins -p codex-plugin` completed successfully.
- Verification: `git diff --check` over metadata, release plan, and upstream-touched files completed successfully.
- Merge preservation review: PASS.
- Build: pending.
- Publish: pending.
