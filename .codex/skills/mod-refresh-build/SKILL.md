---
name: "mod-refresh-build"
description: "Build the Linux Codex CLI .mod release artifact at repo-root codex for a mod release. Use when invoked by $mod-refresh-release or $mod-release-current, or when the user explicitly asks to build the Linux Codex CLI .mod binary."
---

# Mod Refresh Build

## Purpose

Build only the Linux Codex CLI binary for a mod release, verify the build-focused result, and place the release artifact at repository-root `codex` for the publish step. This skill may be invoked by `$mod-refresh-release` or `$mod-release-current`.

## Workflow

1. Confirm the repository is already in the release source state: post-merge for `$mod-refresh-release`, or the intended current `HEAD` for `$mod-release-current`. If invoked by `$mod-refresh-release`, read the release plan for the expected artifact path and record build results there when possible. If invoked by `$mod-release-current`, read the current-release handoff or run notes for the expected artifact path and record build results there when possible.
2. If code changed during the release run, run formatting before building:

   ```bash
   cd codex-rs
   just fmt
   ```

3. Compute the release metadata version before building. Use GitHub's latest non-draft, non-prerelease `openai/codex` release for the base series, read the checked-in repository-root metadata files, validate their exact shapes, derive `upstream_short` from the first five characters of `upstreamhash.txt`, and compute `version`:

   ```bash
   latest_release="$(gh release list --repo openai/codex --exclude-drafts --exclude-pre-releases --limit 1 --json name,tagName --jq '.[0] | (.name // .tagName)')"
   test -n "${latest_release}" || { echo "no stable upstream release found" >&2; exit 1; }
   base_semver="${latest_release#rust-v}"
   base_semver="${base_semver#v}"
   printf '%s\n' "${base_semver}" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$' || {
     echo "latest upstream release is not SemVer x.y.z: ${latest_release}" >&2
     exit 1
   }
   base_series="$(printf '%s\n' "${base_semver}" | awk -F. '{print $1 "." $2}')"
   git ls-files --error-unmatch upstreamhash.txt modversion.txt >/dev/null
   perl -0ne 'exit(/\A[0-9a-f]{40}\n\z/ ? 0 : 1)' upstreamhash.txt || {
     echo "upstreamhash.txt must contain exactly one full lowercase upstream SHA line" >&2
     exit 1
   }
   perl -0ne 'exit(/\A[1-9][0-9]*\n\z/ ? 0 : 1)' modversion.txt || {
     echo "modversion.txt must contain exactly one positive decimal integer line" >&2
     exit 1
   }
   upstream_sha="$(sed -n '1p' upstreamhash.txt)"
   mod_version="$(sed -n '1p' modversion.txt)"
   upstream_short="$(printf '%s' "${upstream_sha}" | cut -c1-5)"
   version="${base_series}.${upstream_short}.${mod_version}.mod"
   ```

   Do not use the final release commit SHA, `git rev-parse --short`, or the upstream SemVer patch component as version inputs. Record the latest upstream release, `upstream_sha`, `mod_version`, `upstream_short`, and computed `version` in the release plan or handoff notes.

4. Build the Linux CLI binary from `codex-rs` with the computed release version embedded:

   ```bash
   cd codex-rs
   CODEX_CLI_RELEASE_VERSION="${version}" cargo build -p codex-cli --release
   ```

   Use this Cargo command by default. Only use a more direct checked-in command if the repository has one at execution time and it clearly builds the same Linux Codex CLI binary without adding tests, Bazel, or unrelated targets.

5. Record `Tests: not run unless explicitly requested`. Do not run tests by default.
6. Record `Bazel: not used; using Cargo release build only`. Do not use Bazel by default.
7. If dependency lock maintenance is needed because dependencies changed, perform only that maintenance. It does not change the release build path and it does not authorize Bazel as a release build or test path.
8. Locate the built CLI binary. The expected default path is:

   ```text
   codex-rs/target/release/codex
   ```

   If that file is absent, inspect the release build output or Cargo metadata to locate the executable produced by the `codex-cli` package. Do not build additional packages to find it.

9. Copy the resulting CLI binary to repository-root `codex` unless the user explicitly requests a different artifact path. The default publish artifact path is exactly:

   ```text
   codex
   ```

10. Preserve executable permissions on `codex`. If needed, run `chmod +x codex`.
11. Verify the copied artifact reports the embedded release version:

   ```bash
   ./codex --version
   ```

   The output must be exactly `codex-cli ${version}`. Stop if the artifact is missing, not executable, cannot report its version, or reports any other version.

## Verification

After copying the artifact, use a `build-verifier` subagent with the same model as main agent and `reasoning_effort = high`. Ask it to verify, from the repository state and command output, that:

- `just fmt` ran from `codex-rs` after code changes when applicable.
- The computed release version was recorded and the build command targeted only `codex-cli` in release mode with `CODEX_CLI_RELEASE_VERSION="${version}"`.
- `Tests: not run unless explicitly requested` is recorded and no tests ran unless explicitly requested.
- `Bazel: not used; using Cargo release build only` is recorded and no Bazel commands ran unless explicitly requested.
- The Linux CLI binary was located and copied to repository-root `codex`.
- The copied artifact path, size, executable bit, and source binary path are recorded.
- The copied artifact's `./codex --version` output is exactly `codex-cli ${version}`.

Resolve any verifier findings before handing off to publish.

## Completion

Report the computed release version, build command, formatting command if run, the recorded tests/Bazel decisions, skipped tests/Bazel status, source binary path, repo-root artifact path, `./codex --version` output, verifier result, and any concerns. If invoked by `$mod-refresh-release` or `$mod-release-current`, update the release plan with the same details before returning control.
