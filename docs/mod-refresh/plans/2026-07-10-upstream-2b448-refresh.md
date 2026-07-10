# Mod Refresh Release Plan: upstream 2b448 refresh

Date: 2026-07-10

## Objective

Merge `upstream/main` at `2b44896c5ad653a1dcfc537f8bdc37767744ed09` into
`pub/mod-refresh-2026-06-24`, preserve the fork-local compact and model-selection
behavior recorded in `docs/compact-fix/ChangeLog.md`, build the Linux Codex CLI,
and publish the metadata-based `.mod` release to `garyfpga/codex-compact-fix`.
Reusing the stale-named release branch is intentional; it is clean and 16 commits
ahead of its origin tracking branch at preflight time.

## Fresh Preflight

Source: current-session `$mod-refresh-full-release` request and
`$mod-refresh-preflight` run on 2026-07-10.

- Recommendation: ready to continue; reviewer PASS.
- Current branch: `pub/mod-refresh-2026-06-24`
- Current HEAD: `7fe50ccb8c2bd103de726d641028e0b21078e6e7`
- Upstream ref: `upstream/main`
- Upstream target SHA: `2b44896c5ad653a1dcfc537f8bdc37767744ed09`
- Merge base: `da4c8ca57d40b074bdc1b5b1218851100150c56b`
- Worktree: clean before this plan was created.
- Upstream scope: 772 files, 31,715 additions, and 9,460 deletions.
- Merge simulation: conflicts in seven surfaced preservation areas:
  `codex-rs/core/src/client.rs`, `codex-rs/core/src/compact_remote.rs`,
  `codex-rs/core/src/compact_remote_v2.rs`,
  `codex-rs/core/src/session/turn.rs`,
  `codex-rs/login/src/auth/default_client.rs`,
  `codex-rs/login/src/auth/default_client_tests.rs`, and
  `codex-rs/tui/src/chatwidget/model_popups.rs`.
- Preflight reviewer: continuation justified with high but bounded preservation
  risk; no unsurfaced behavior choice or other blocker found.

## Release Decisions

- Tests: not run unless explicitly requested
- Bazel: not used; using Cargo release build only
- Release source: final post-merge `HEAD` on `pub/mod-refresh-2026-06-24`
- Expected artifact path: repository-root `codex`
- Expected build source: `codex-rs/target/release/codex`
- Expected `upstreamhash.txt`: `2b44896c5ad653a1dcfc537f8bdc37767744ed09`
- Expected `modversion.txt`: `1`
- Latest stable upstream release at planning time: `0.144.1`; revalidate before
  build and publish.
- Expected version contract:
  `<latest-upstream-major>.<latest-upstream-minor>.<first5-upstreamhash>.<modversion>.mod`
- Provisional candidate version/tag: `0.144.2b448.1.mod`. This is not approved
  for build or publish until the latest stable upstream release and all metadata
  are recomputed and revalidated immediately before both stages.
- Build handoff:
  `CODEX_CLI_RELEASE_VERSION="${version}" cargo build -p codex-cli --release`
- The release suffix comes only from `upstreamhash.txt` and `modversion.txt`, not
  final `HEAD`, `git rev-parse --short`, or the upstream SemVer patch component.
- Publish repository: `garyfpga/codex-compact-fix`
- `origin` was verified during planning to resolve to
  `git@github.com:garyfpga/codex-compact-fix.git` for fetch and push.
- Tag and release title: `${version}`
- Release notes source:
  `docs/mod-refresh/plans/2026-07-10-upstream-2b448-release-notes.md`
- Release notes approval: the notes content is part of this user-invoked full
  release plan and must receive the release-plan PASS before the initial plan
  commit and packaging reviewer PASS before publish. Record the packaging PASS in
  the coordinator handoff and final report without another tracked plan edit.
- Upload target and stable asset name: repository-root `codex`

## Ordered Checklist

1. Review this plan for stale assumptions, missing gates, and publish ambiguity.
2. Verify the plan and notes are the only preflight worktree delta, then commit
   them before the real merge.
3. Record the plan commit as the expected merge-start HEAD. Reconfirm the branch,
   clean `git status --short`, remotes, merge base, and unchanged upstream target
   SHA. Stop if the branch, upstream target, or state differs; the historical
   preflight HEAD remains `7fe50ccb8c2bd103de726d641028e0b21078e6e7`.
4. Run the real merge, inventory its actual conflicts, and use a
   `merge-conflict-worker` with the same model and high reasoning effort. Stop on
   any expanded or unsurfaced conflict. Resolve the seven simulated conflicts only
   if the real inventory matches and the ChangeLog settles their behavior.
5. Update `upstreamhash.txt` to the expected full SHA and `modversion.txt` to `1`.
6. Run required non-test maintenance only: `just fmt`, schema/snapshot generation
   when required by intentional changes, and dependency lock maintenance when
   dependency files changed.
7. Use a `compact-preservation-reviewer` with the same model and high reasoning
   effort on the final merge diff. Record skipped tests/Bazel and commit the
   completed merge, metadata, and result notes only after a PASS.
8. Revalidate the latest stable `openai/codex` release and compute `${version}`
   from its major/minor series plus the checked-in metadata.
9. From `codex-rs`, build only `codex-cli` in Cargo release mode with
   `CODEX_CLI_RELEASE_VERSION="${version}"`.
10. Return to the repository root, copy `codex-rs/target/release/codex` to
    repository-root `codex`, preserve its executable bit, and verify
    `./codex --version` is exactly `codex-cli ${version}`.
11. Use a `build-verifier` with the same model and high reasoning effort to check
    the command, cwd, embedded version, artifact path/size/executable bit/source,
    skipped-test/Bazel decisions, and exact version output.
12. Record build verification in this plan and commit it. Because that changes
    final `HEAD`, rebuild with the same embedded `${version}` and reverify the
    artifact from that clean final publish commit, refresh repository-root `codex`,
    and rerun the build verifier. Make no further tracked edits before tagging.
13. Confirm the final commit, metadata cleanliness, artifact provenance, release
    notes, publish target, and absence of local/remote tag and GitHub release.
14. Recompute/revalidate `${version}` again, run the packaging reviewer gate,
    create and push the annotated tag, then
    create the GitHub release with the approved notes and `codex` asset.

## Preservation Risks

- Upstream split remote compact logic into new request, attempt, and model-fallback
  modules. Preserve the shared remote-first fork wrapper as the policy owner while
  retaining upstream's previous-model rejection retry with the selected model.
  Preserve its request history cleanup, warning category, and visible attempt
  accounting. Stop if integrating it creates an ambiguous attempt-budget, warning,
  or fallback-history choice not settled by the ChangeLog and upstream contract.
- Preserve upstream selected-model fallback only when it does not violate the
  fork's bounded visible attempt contract; otherwise stop for direction.
- Preserve V1 and V2 visible attempt budgets, configured timeouts, warning labels,
  clean-history local fallback, and the prohibition on hidden retries that inflate
  visible attempts.
- Preserve compact-only fast service-tier selection and API-key omission without
  changing ordinary sampling.
- Upstream migrated HTTP consumers to `codex-http-client`. Preserve normal
  headers, proxy, custom CA, Cloudflare cookie, TCP keepalive, and explicit compact
  retry boundaries without retaining obsolete client construction.
- Preserve auto/manual compact routing, V2 feature selection, local-only provider
  behavior, pre-compact hook semantics, and upstream session-turn changes.
- Preserve `remote_compact` config defaults, bounds, effective resolution, schema,
  and source tests.
- Preserve compact integration tests, parity tests, config tests, and snapshots as
  source artifacts. Tests remain skipped by release policy.
- Preserve upstream dynamic skill-catalog and tool-snapshot parity in remote
  compact request bodies while retaining the fork's intended V1/V2 differences.
- Preserve `/model` as session-only and `/modelp` as persistent through quick,
  all-model, reasoning, and Plan-mode picker paths, including snapshots.
- Preserve `/modelp` slash-command visibility, prefix filtering, and ordering
  after service-tier pseudo-commands.
- Preserve release-version display through `CODEX_CLI_RELEASE_VERSION`, with
  Cargo package fallback for local builds.
- Preserve `docs/compact-fix/ChangeLog.md`, the Simple Power plan trail, and the
  exact metadata file contracts.

## Maintenance Policy

Allowed: `cd codex-rs && just fmt`, required schema/snapshot maintenance,
dependency lock maintenance, and the Cargo release build for `codex-cli`.

Not allowed without an explicit user request: `just test`, `cargo test`, Bazel
build/test commands, focused test commands, or full upstream test suites.

Because the upstream merge changes Rust dependency manifests and lockfiles, if
those dependency changes survive conflict resolution run exactly
`just bazel-lock-update` from the repository root and record its result. This is
dependency lock maintenance only and may invoke Bazel only for lock maintenance;
it does not authorize or count as a Bazel release build or test path. The release
policy remains exactly: Bazel: not used; using Cargo release build only.

## Metadata Gates

Before both build and publish, from the repository root require the metadata files
to be tracked and clean, then validate their exact contents:

```bash
git ls-files --error-unmatch upstreamhash.txt modversion.txt >/dev/null
git diff --quiet -- upstreamhash.txt modversion.txt
git diff --cached --quiet -- upstreamhash.txt modversion.txt
perl -0ne 'exit(/\A[0-9a-f]{40}\n\z/ ? 0 : 1)' upstreamhash.txt
perl -0ne 'exit(/\A[1-9][0-9]*\n\z/ ? 0 : 1)' modversion.txt
test "$(sed -n '1p' upstreamhash.txt)" = \
  "2b44896c5ad653a1dcfc537f8bdc37767744ed09"
test "$(sed -n '1p' modversion.txt)" = "1"
```

## Publish Process

Recompute `${version}` independently before build and again before publish from
the latest stable upstream release plus checked-in metadata. Run this block from
the repository root. Only after the build-results commit, clean final-HEAD rebuild,
metadata checks, no-tracked-edits check, release-notes approval, and packaging
reviewer PASS, set the final commit and publish:

```bash
final_commit="$(git rev-parse HEAD)"
git tag -a "${version}" "${final_commit}" -m "${version}"
git push origin "refs/tags/${version}"
gh release create "${version}" "codex" \
  --repo garyfpga/codex-compact-fix \
  --verify-tag \
  --title "${version}" \
  --notes-file "$(git rev-parse --show-toplevel)/docs/mod-refresh/plans/2026-07-10-upstream-2b448-release-notes.md"
```

## Stop Conditions

Stop for direction if the upstream SHA, branch, or preflight assumptions diverge;
the merge reveals an unsurfaced compact/model behavior choice; preservation
review is incomplete; metadata is malformed or divergent; required maintenance,
formatting, build, or exact version verification fails; release notes, artifact,
tag, or publish repository becomes ambiguous; or a tag/release operation partially
succeeds and a later step fails.

## Execution Log

- Fresh preflight: PASS; ready to continue.
- Full-release preflight reviewer: PASS.
- Full-release chain reviewer: PASS.
- Release-plan reviewer: PASS; release-notes content approved for packaging review.
- Plan and release-notes commit: `2f6bbf2633eb961498069556e2a4075390e3f997`.
- Merge-start HEAD: `2f6bbf2633eb961498069556e2a4075390e3f997`.
- Real merge target: `upstream/main` at
  `2b44896c5ad653a1dcfc537f8bdc37767744ed09`; its conflict inventory exactly
  matched the seven simulated paths.
- Merge conflict worker: PASS; upstream request/attempt modules retained, with
  one global visible attempt budget across previous-model and selected-model
  requests and the fork timeout, retry, service-tier, and fallback policies.
- Conflict resolution: upstream route-aware HTTP client and raw-auth logging
  architecture retained; compact keepalive support was adapted to that client
  factory. `/model` and `/modelp` persistence semantics were retained while
  incorporating upstream Ultra reasoning concurrency warnings.
- Metadata: `upstreamhash.txt` updated to
  `2b44896c5ad653a1dcfc537f8bdc37767744ed09`; `modversion.txt` remains `1`.
- Dependency lock maintenance: repository-root `just bazel-lock-update` completed
  successfully. It invoked Bazel only to refresh `MODULE.bazel.lock`; no Bazel
  build or test was run.
- Formatting: `cd codex-rs && just fmt` completed successfully.
- Compact-preservation reviewer: PASS; one global request budget, exact retry and
  transport boundaries, selected-model fallback, clean local fallback, dynamic
  tool snapshots, compact-only service tier, `/model`/`/modelp`, config/schema,
  and metadata contracts were preserved by inspection.
- Merge commit: `6462e9f90974eca6301fcd43b68f2ddf4b510fe9`.
- Pre-build version computation: latest stable upstream `0.144.1`; base series
  `0.144`; upstream SHA `2b44896c5ad653a1dcfc537f8bdc37767744ed09`;
  upstream short `2b448`; mod version `1`; release version
  `0.144.2b448.1.mod`.
- Tests: not run unless explicitly requested.
- Bazel: not used; using Cargo release build only.
- First release build: FAILED with exit code 101. The exact versioned command was
  `cd codex-rs && CODEX_CLI_RELEASE_VERSION="0.144.2b448.1.mod" cargo build -p codex-cli --release`.
  `codex-core` reported three post-merge API mismatches in
  `core/src/session/turn.rs`: obsolete `turn_context` arguments to
  `finalize_non_tool_response_item` and `handle_non_tool_response_item`, and a
  missing `HttpClientFactory` argument to `models_manager.refresh_if_new_etag`.
  No artifact was copied, tagged, pushed, or published. Release flow stopped for
  explicit recovery direction as required by the build-failure gate.
- Recovery direction: user explicitly approved fixing the three compile errors
  and resuming the release chain.
- Build-failure fix reviewer: PASS. The three minimal call-site fixes match
  `upstream/main`: remove obsolete turn-context arguments and pass the effective
  turn config's `HttpClientFactory` to the model ETag refresh.
- Recovery formatting: `cd codex-rs && just fmt` completed successfully.
- Second release build: FAILED with exit code 101 after the three approved fixes
  compiled. `codex-core` reported an unhandled upstream
  `ResponseEvent::ReasoningSummaryDone` variant in the exhaustive event match in
  `core/src/session/turn.rs`; the compiler also reported unused parameters around
  that stale merged event-processing path. No artifact was copied, tagged,
  pushed, or published. Release flow stopped again for explicit recovery
  direction.
- Second recovery direction: user explicitly approved continuing.
- Reasoning-event fix reviewer: PASS. The earlier whole-file `ours` resolution of
  `session/turn.rs` had also dropped upstream request-scoped MCP tool snapshots,
  streamed item-ID repair, hook lifecycle recording, and sequential-cutoff
  reasoning summaries. The file was restored wholesale from `upstream/main`, then
  only the fork's shared remote-compaction imports and `run_auto_compact` routing
  were reapplied. Its remaining diff from upstream is confined to those intended
  compact-routing changes.
- Second recovery formatting: `cd codex-rs && just fmt` completed successfully.
