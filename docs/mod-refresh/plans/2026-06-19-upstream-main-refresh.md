# Mod Refresh Release Plan: upstream/main refresh

Date: 2026-06-19

## Objective

Merge `upstream/main` into `feat/yolo-default-compact-timer`, preserve the fork-local compact behavior recorded in `docs/compact-fix/ChangeLog.md`, build the Linux Codex CLI artifact, then publish a metadata-based `.mod` GitHub release.

## Fresh Preflight

Source: current-session `$mod-refresh-full-release` preflight run after the user's request on 2026-06-19.

Summary:

- Current branch: `feat/yolo-default-compact-timer`
- Current HEAD: `0bed05d0e652089129296bc56326afaf93f1feca`
- Upstream ref fetched: `upstream/main`
- Upstream target SHA: `04483f4ce5694d471e471583d4ca286908d7c8b7`
- Merge base: `c73296a0f095e72dbb646909c613ae09c9459c3a`
- Worktree before preflight: clean
- Upstream touched paths: 429
- Merge simulation: conflicts in the compact preservation files listed below
- Preflight recommendation: ready to continue if explicitly requested; request was `$mod-refresh-full-release`
- Full-release preflight reviewer result: continuation justified, no blockers, with required stale-assumption checks and compact preservation warnings

Simulated conflict files:

- `codex-rs/codex-api/src/endpoint/compact.rs`
- `codex-rs/core/config.schema.json`
- `codex-rs/core/src/compact_remote.rs`
- `codex-rs/core/src/compact_remote_v2.rs`
- `codex-rs/core/src/config/mod.rs`
- `codex-rs/core/src/tasks/compact.rs`
- `codex-rs/core/src/tasks/regular.rs`

Direct compact-risk overlap files:

- `codex-rs/codex-api/src/endpoint/compact.rs`
- `codex-rs/codex-api/src/lib.rs`
- `codex-rs/config/src/config_toml.rs`
- `codex-rs/core/config.schema.json`
- `codex-rs/core/src/client.rs`
- `codex-rs/core/src/compact.rs`
- `codex-rs/core/src/compact_remote.rs`
- `codex-rs/core/src/compact_remote_v2.rs`
- `codex-rs/core/src/config/config_tests.rs`
- `codex-rs/core/src/config/mod.rs`
- `codex-rs/core/src/lib.rs`
- `codex-rs/core/src/session/turn.rs`
- `codex-rs/core/src/tasks/compact.rs`
- `codex-rs/core/src/tasks/regular.rs`
- `codex-rs/core/tests/suite/compact_remote.rs`

## Release Decisions

- Tests: not run unless explicitly requested
- Bazel: not used; using Cargo release build only
- Release source: final post-merge `HEAD` on `feat/yolo-default-compact-timer`
- Expected artifact path: repository-root `codex`
- Expected build source binary: `codex-rs/target/release/codex`
- Latest stable upstream release source: `openai/codex` latest non-draft, non-prerelease GitHub release; current pre-plan check returned `0.141.0`
- Expected base series from current upstream release check: `0.141`
- Expected `upstreamhash.txt`: `04483f4ce5694d471e471583d4ca286908d7c8b7`
- Expected `modversion.txt`: `1`
- Expected release version contract: `<latest-upstream-major>.<latest-upstream-minor>.04483.1.mod`
- Current computed release version handoff: `version="0.141.04483.1.mod"`
- Build handoff: pass `CODEX_CLI_RELEASE_VERSION="${version}"` into `cargo build -p codex-cli --release`
- Release suffix source: `<first5-upstreamhash>.<modversion>.mod`; do not derive the suffix from final `HEAD`, `git rev-parse --short`, or the upstream SemVer patch component
- GitHub publish target: `garyfpga/codex-compact-fix`
- Git tag flow: create an annotated tag named `${version}` on the final release commit SHA, push that tag to `origin`, then create the GitHub release with `--verify-tag`
- GitHub publish command: `gh release create "${version}" "codex" --repo garyfpga/codex-compact-fix --verify-tag --title "${version}" --notes-file "${notes_file}"`
- Release notes source: the notes in this plan under `Release Notes`

## Release Notes

Refresh to `openai/codex` `upstream/main` at `04483f4ce5694d471e471583d4ca286908d7c8b7`, preserving the compact-fix fork behavior and the metadata-based `.mod` version display.

## Ordered Checklist

1. Reconfirm branch, clean worktree, remotes, and fetch `upstream/main`.
2. Before the real merge, verify `upstream/main` still equals `04483f4ce5694d471e471583d4ca286908d7c8b7`, `HEAD` still equals `0bed05d0e652089129296bc56326afaf93f1feca`, and the merge base is still `c73296a0f095e72dbb646909c613ae09c9459c3a`.
3. Run `git status --short --untracked-files=all` and allow only this release plan file as expected pre-merge dirtiness.
4. Merge `upstream/main` into `feat/yolo-default-compact-timer`.
5. Resolve conflicts while preserving `docs/compact-fix/ChangeLog.md`.
6. Update `upstreamhash.txt` to `04483f4ce5694d471e471583d4ca286908d7c8b7` and `modversion.txt` to `1`.
7. Verify both metadata files exactly match the expected values and shape.
8. Run only required non-test maintenance checks.
9. Record skipped tests and skipped Bazel decisions.
10. Commit the merge, metadata, and release-plan updates.
11. Run `cd codex-rs && just fmt` before building if code changed after the merge commit.
12. Compute the release version from latest stable upstream Codex release plus checked-in `upstreamhash.txt` and `modversion.txt`.
13. Run `cd codex-rs && CODEX_CLI_RELEASE_VERSION="${version}" cargo build -p codex-cli --release`.
14. Copy `codex-rs/target/release/codex` to repository-root `codex` and preserve executable permissions.
15. Verify artifact path, size, executable bit, source binary path, and `./codex --version` output exactly `codex-cli ${version}`.
16. Create a release notes file from the exact `Release Notes` block in this plan.
17. Confirm no local tag, remote tag, or GitHub release exists for the computed version.
18. Run the release packaging reviewer gate.
19. Create the annotated git tag on the final release commit SHA, push that tag to `origin`, and create the GitHub release against `garyfpga/codex-compact-fix` with `--verify-tag`.

## Preservation Risks

Carry these `docs/compact-fix/ChangeLog.md` groups through merge review:

- Preserve `remote_compact` config parsing, validation bounds, defaults, effective config, tests, and `codex-rs/core/config.schema.json` generation.
- Preserve the shared remote-first fallback wrapper as the single policy owner for auto and manual compact routing.
- Preserve compact-only fast service tier behavior, including API-key auth omission of the remote tier override.
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

- Upstream changed config and schema surfaces for orchestrator skills, rollout budget, current-time reminder, web search indexed mode, and realtime config removal. Recheck `remote_compact` schema/config behavior after conflict resolution.
- Upstream changed session/task return shapes, `TurnAborted` propagation, window numbering, token budget hooks, and history replacement signatures. Preserve compact routing through the shared fallback wrapper while adapting to upstream control flow.
- Upstream changed compact request item-id handling, compaction trigger shape, rollout budget accounting, and compacted-history APIs. Preserve explicit V1 attempt counts, no hidden retries, timeout wording, TCP keepalive, and V2 parity.
- Upstream changed `codex-rs/Cargo.lock`; after merge, check final dependency-file changes and run required dependency lock maintenance if dependency files changed.
- Group 9 TUI `.mod` version display files and Group 10 Simple Power plan history were not directly touched by the preflight delta, but must remain intact.

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
git diff --name-only 0bed05d0e652089129296bc56326afaf93f1feca..HEAD -- Cargo.toml Cargo.lock 'codex-rs/**/Cargo.toml' 'codex-rs/Cargo.lock'
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
- Merge preservation exposes a compact behavior choice not surfaced by preflight.
- Conflict resolution cannot clearly preserve the ChangeLog behavior.
- `upstreamhash.txt` or `modversion.txt` is missing, malformed, or divergent from this plan after the merge.
- Build fails or `codex-rs/target/release/codex` is missing.
- Repository-root artifact `codex` is missing, not executable, or reports a version other than `codex-cli ${version}`.
- Local tag, remote tag, or GitHub release already exists for the computed version.
- Release notes, publish destination, or artifact destination becomes ambiguous.
- `git tag`, `git push origin "refs/tags/${version}"`, or `gh release create` partially succeeds and a later publish step fails.
- Tag deletion, release deletion, artifact overwrite, or publish retry is needed after a partial failure.

## Required Command Details

Pre-merge stale-assumption checks:

```bash
git fetch upstream main
test "$(git rev-parse upstream/main)" = "04483f4ce5694d471e471583d4ca286908d7c8b7"
test "$(git rev-parse HEAD)" = "0bed05d0e652089129296bc56326afaf93f1feca"
test "$(git merge-base HEAD upstream/main)" = "c73296a0f095e72dbb646909c613ae09c9459c3a"
git status --short --untracked-files=all
```

Only acceptable pre-merge status output:

```text
?? docs/mod-refresh/plans/2026-06-19-upstream-main-refresh.md
```

Version computation:

```bash
final_commit="$(git rev-parse HEAD)"
latest_release="$(gh release list --repo openai/codex --exclude-drafts --exclude-pre-releases --limit 1 --json name,tagName --jq '.[0] | (.name // .tagName)')"
test -n "${latest_release}" || { echo "no stable upstream release found" >&2; exit 1; }
base_semver="${latest_release#rust-v}"
base_semver="${base_semver#v}"
printf '%s\n' "${base_semver}" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'
base_series="$(printf '%s\n' "${base_semver}" | awk -F. '{print $1 "." $2}')"
upstream_sha="$(sed -n '1p' upstreamhash.txt)"
mod_version="$(sed -n '1p' modversion.txt)"
upstream_short="$(printf '%s' "${upstream_sha}" | cut -c1-5)"
version="${base_series}.${upstream_short}.${mod_version}.mod"
```

Build and artifact commands:

```bash
cd codex-rs
CODEX_CLI_RELEASE_VERSION="${version}" cargo build -p codex-cli --release
cd ..
cp codex-rs/target/release/codex codex
chmod +x codex
stat -c '%n %s %A' codex codex-rs/target/release/codex
sha256sum codex codex-rs/target/release/codex
./codex --version
```

Artifact verification must produce exactly:

```text
codex-cli 0.141.04483.1.mod
```

Release notes:

```bash
notes_file="$(mktemp)"
cat >"${notes_file}" <<'NOTES'
Refresh to `openai/codex` `upstream/main` at `04483f4ce5694d471e471583d4ca286908d7c8b7`, preserving the compact-fix fork behavior and the metadata-based `.mod` version display.
NOTES
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

- Preflight: completed after release request at `0bed05d0e652089129296bc56326afaf93f1feca`.
- Upstream target SHA: `04483f4ce5694d471e471583d4ca286908d7c8b7`.
- Preflight reviewer: approved continuation to `$mod-refresh-release`.
- Tests: not run unless explicitly requested.
- Bazel: not used; using Cargo release build only.
- Plan review: PASS.
- Merge preservation: real `git merge upstream/main` started from `feat/yolo-default-compact-timer`; conflicts matched preflight in compact endpoint, config/schema, V1/V2 remote compact, and compact/regular task files.
- Merge preservation: resolved by preserving the shared remote-first fallback wrapper, explicit V1/V2 visible attempt policy, compact-only service-tier override, upstream window-number bookkeeping, upstream rollout-budget usage recording, and upstream `SessionTaskResult` propagation.
- Maintenance: `cd codex-rs && just write-config-schema` completed successfully.
- Maintenance: `cd codex-rs && just fmt` completed successfully.
- Maintenance: dependency files changed; `just bazel-lock-update` completed successfully. This was dependency lock maintenance, not a Bazel build/test path.
- Maintenance: `just bazel-lock-check` completed successfully. This was dependency lock maintenance, not a Bazel build/test path.
- Tests: not run unless explicitly requested.
- Bazel: not used; using Cargo release build only.
- Merge preservation review: initial reviewer BLOCK; V2 remote compact still inherited provider stream retries through the normal Responses streaming path.
- Merge preservation fix: added explicit stream retry-policy plumbing and routed V2 remote compact through the shared no-hidden-retry compact policy. Updated the V2 fallback source test to enable provider stream retries while expecting exactly the configured visible attempt count and warning count. Tests were not run, per release policy.
- Merge preservation review: second reviewer BLOCK; V2 remote compact disabled provider stream retries, but still inherited ordinary 401 auth recovery as a hidden retry path inside a visible attempt.
- Merge preservation fix: changed the explicit compact stream policy to disable unauthorized recovery for both Responses HTTP and WebSocket stream setup. Added a V2 unauthorized stream-open fallback regression test that expects exactly one `/responses` request per visible compact attempt. Tests were not run, per release policy.
- Merge preservation review: third reviewer BLOCK; Responses API policy-aware streaming path called the provider-default endpoint session wrapper instead of the policy-aware wrapper, which would not compile with the new retry-policy argument.
- Merge preservation fix: changed `ResponsesClient::stream_encoded_with_policy` to call `EndpointSession::stream_encoded_json_with_policy`. Ran `cd codex-rs && just fmt` again. Tests were not run, per release policy.
- Merge preservation review: PASS after API retry-policy plumbing fix.
- Build: first `CODEX_CLI_RELEASE_VERSION=0.141.04483.1.mod cargo build -p codex-cli --release` failed in `codex-core` because the Responses API stream path borrowed `client_setup.api_provider` after moving it into the API client.
- Build fix: resolved the stream retry policy before moving `client_setup.api_provider` into `ApiResponsesClient`.
- Build: pending.
- Publish: pending.
