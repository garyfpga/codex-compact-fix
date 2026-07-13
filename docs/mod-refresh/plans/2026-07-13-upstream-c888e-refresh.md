# Mod Refresh Release Plan: upstream c888e refresh

Date: 2026-07-13

## Objective

Merge `upstream/main` at `c888e8e75a9f0e90ce7d5517f8b9540832cbbf76` into
`main`, preserve the fork-local compact and model-selection behavior recorded in
`docs/compact-fix/ChangeLog.md`, build the Linux Codex CLI, and publish the
metadata-based `.mod` release to `garyfpga/codex-compact-fix`.

## Fresh Preflight

Source: current-session `$mod-refresh-full-release` request and
`$mod-refresh-preflight` run on 2026-07-13.

- Recommendation: ready to continue.
- Current branch: `main`
- Current HEAD: `8446de7c64f3f2cd72432f3c4911d488c38ce828`
- Upstream ref: `upstream/main`
- Upstream target SHA: `c888e8e75a9f0e90ce7d5517f8b9540832cbbf76`
- Merge base: `2b44896c5ad653a1dcfc537f8bdc37767744ed09`
- Worktree: clean before this plan was created.
- Upstream scope: 303 files, 10,185 insertions, and 5,431 deletions.
- Merge simulation: clean, with no conflicts.
- Direct preservation overlaps: `codex-rs/config/src/config_toml.rs`,
  `codex-rs/core/config.schema.json`, `codex-rs/core/src/client.rs`,
  `codex-rs/core/src/compact_remote.rs`, `codex-rs/core/src/config/mod.rs`,
  `codex-rs/core/src/lib.rs`, `codex-rs/core/src/session/turn.rs`, and
  `codex-rs/tui/src/chatwidget/tests/popups_and_settings.rs`.
- No unsurfaced behavior choice or blocker was found during preflight.

## Release Decisions

- Tests: not run unless explicitly requested
- Bazel: not used; using Cargo release build only
- Release source: final post-merge `HEAD` on `main`
- Expected artifact path: repository-root `codex`
- Expected build source: `codex-rs/target/release/codex`
- Expected `upstreamhash.txt`: `c888e8e75a9f0e90ce7d5517f8b9540832cbbf76`
- Expected `modversion.txt`: `1`
- Latest stable upstream release at planning time: `0.144.1` from GitHub's
  latest non-draft, non-prerelease `openai/codex` release; revalidate before
  build and publish.
- Expected version contract:
  `<latest-upstream-major>.<latest-upstream-minor>.<first5-upstreamhash>.<modversion>.mod`
- Provisional candidate version/tag: `0.144.c888e.1.mod`. Recompute from the
  latest stable upstream release and checked-in metadata before build and publish.
- Build handoff:
  `CODEX_CLI_RELEASE_VERSION="${version}" cargo build -p codex-cli --release`
- The release suffix comes only from `upstreamhash.txt` and `modversion.txt`, not
  final `HEAD`, `git rev-parse --short`, or the upstream SemVer patch component.
- Publish repository: `garyfpga/codex-compact-fix`
- Confirmed `origin`: `git@github.com:garyfpga/codex-compact-fix.git`.
- Tag and release title: `${version}`
- Release notes source:
  `docs/mod-refresh/plans/2026-07-13-upstream-c888e-release-notes.md`
- Release notes approval: the notes are part of this user-invoked full-release
  plan and require the release-plan reviewer PASS and packaging reviewer PASS.
- Upload target and stable asset name: repository-root `codex`

## Ordered Checklist

1. Review this plan and release notes for stale assumptions, missing gates,
   artifact ambiguity, and unsafe publishing.
2. Commit the reviewed plan and release notes, then reconfirm `main`, a clean
   worktree, remotes, and the unchanged upstream target SHA.
3. Merge `upstream/main` and inspect all direct semantic overlaps even though the
   simulation was conflict-free.
4. Update `upstreamhash.txt` to the expected SHA and `modversion.txt` to `1`.
5. Run required non-test maintenance only: `just fmt`, schema/snapshot generation
   when required, and dependency lock maintenance when dependency files changed.
6. Run compact-preservation review against the final merge diff, record skipped
   tests/Bazel, and commit the completed merge and metadata only after PASS.
7. Revalidate the latest stable upstream release and compute `${version}` from
   its major/minor series plus the checked-in metadata.
8. Build only `codex-cli` in Cargo release mode from `codex-rs` with
   `CODEX_CLI_RELEASE_VERSION="${version}"`.
9. Copy `codex-rs/target/release/codex` to repository-root `codex`, preserve its
   executable bit, and require `./codex --version` to equal
   `codex-cli ${version}`.
10. Run the build-verifier gate and record its result. If recording results changes
    final `HEAD`, rebuild and reverify from the final clean publish commit.
11. Recompute `${version}`, run every publish safety check and the packaging
    reviewer gate, create and push the annotated tag, then create the GitHub
    release with the approved notes and `codex` asset.

## Preservation Risks

- Preserve `remote_compact` config parsing, defaults, validation bounds, effective
  resolution, and schema despite upstream config and schema changes.
- Preserve the shared remote-first wrapper as the single policy owner for auto
  and manual compact routing, including V2 selection and local-only behavior.
- Preserve compact-only fast service-tier selection and API-key omission without
  changing ordinary sampling.
- Preserve V1 explicit retry settings, zero hidden retries, bounded visible
  attempts, timeout wording, TCP keepalive, normal headers/proxy/CA/cookie
  behavior, warning categories, and clean local fallback history.
- Preserve V2 policy parity, attempt accounting, warning labels, and request-shape
  parity where intended.
- Audit upstream changes in `client.rs`, `compact_remote.rs`, and
  `session/turn.rs` for semantic drift even though Git merges them automatically.
- Preserve compact integration/config/parity tests and snapshots as source
  artifacts; tests remain skipped under this release policy.
- Preserve `/model` as session-only and `/modelp` as persistent through nested
  picker paths and the touched popup tests.
- Preserve release display via `CODEX_CLI_RELEASE_VERSION`, the compact-fix
  ChangeLog, Simple Power plan history, and exact metadata contracts.

## Maintenance Policy

Allowed: `cd codex-rs && just fmt`, required schema/snapshot maintenance,
dependency lock maintenance, and the Cargo release build for `codex-cli`.

Not allowed without an explicit user request: `just test`, `cargo test`, Bazel
build/test commands, focused test commands, or full upstream test suites.

Because upstream changes `codex-rs/Cargo.toml`, `codex-rs/Cargo.lock`, and
`MODULE.bazel.lock`, run repository-root `just bazel-lock-update` if those
dependency changes survive the merge. This is dependency lock maintenance only;
it does not authorize Bazel as the release build or test path.

## Metadata Gates

Before build and publish, require both metadata files to be tracked and clean,
validate their exact one-line shapes, and require these values:

```text
upstreamhash.txt = c888e8e75a9f0e90ce7d5517f8b9540832cbbf76
modversion.txt = 1
```

## Publish Process

Recompute `${version}` independently before build and publish. After the final
clean-HEAD artifact verification and packaging reviewer PASS:

```bash
final_commit="$(git rev-parse HEAD)"
git tag -a "${version}" "${final_commit}" -m "${version}"
git push origin "refs/tags/${version}"
gh release create "${version}" "codex" \
  --repo garyfpga/codex-compact-fix \
  --verify-tag \
  --title "${version}" \
  --notes-file "$(git rev-parse --show-toplevel)/docs/mod-refresh/plans/2026-07-13-upstream-c888e-release-notes.md"
```

## Stop Conditions

Stop for direction if the branch, upstream SHA, or preflight assumptions diverge;
the merge reveals an unsurfaced behavior choice; preservation review is
incomplete; metadata, maintenance, formatting, build, or version verification
fails; release notes, artifact, tag, or repository become ambiguous; or a
tag/release operation partially succeeds and a later step fails.

## Execution Log

- Fresh preflight: PASS; ready to continue.
- Release-plan reviewer: PASS; no missing gates, stale assumptions, artifact
  ambiguity, metadata/version-contract issue, or unsafe publish step found.
- Tests: not run unless explicitly requested.
- Bazel: not used; using Cargo release build only.
