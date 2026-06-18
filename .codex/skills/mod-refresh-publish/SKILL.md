---
name: "mod-refresh-publish"
description: "Publish the final mod GitHub release by deriving the base series from the latest stable upstream Codex release and the suffix from final HEAD, tagging that commit, creating the GitHub release, and uploading repo-root codex. Use only when invoked by $mod-refresh-release, invoked by $mod-release-current, or explicitly asked to tag and publish by direct publish request."
---

# Mod Refresh Publish

## Purpose

Use this skill only for the final publish step of a mod release. The final release commit can come from either a completed upstream refresh merge or a current-HEAD feature release path. It tags the final release commit, creates the matching GitHub release, and uploads the built binary.

This skill mutates remote release state. Stop on ambiguity rather than guessing.

## Required Inputs

Require all of the following before publishing:

- A final release commit checked out as `HEAD`, from either a completed upstream refresh merge or a current-HEAD feature release path.
- A verified build artifact in the repository root for that final `HEAD`.
- Approved release notes or a release-plan entry that clearly states the notes to publish.
- A confirmed GitHub repository target for `gh release create`.

Do not continue if the release path, preflight/build provenance, artifact verification, release notes, or publish destination is ambiguous. Do not continue if the working tree has uncommitted tracked changes that could affect the final release commit. If another commit is created after computing the version, recompute the version from the new `HEAD`.

## Version Contract

Compute the release base from GitHub's latest non-prerelease `openai/codex` release every time. Do not hardcode a base series such as `0.139`.

Use `gh release view` against the upstream repository, strip `rust-v` or `v`, require a SemVer release name, and use the first two SemVer components as the mod release series:

```bash
final_commit="$(git rev-parse HEAD)"
xxxxx="$(git rev-parse HEAD | cut -c1-5)"
latest_release="$(gh release view --repo openai/codex --json name,tagName,isPrerelease,isDraft --jq 'select(.isDraft == false and .isPrerelease == false) | (.name // .tagName)')"
base_semver="${latest_release#rust-v}"
base_semver="${base_semver#v}"
case "${base_semver}" in
  [0-9]*.[0-9]*.[0-9]*) ;;
  *) echo "latest upstream release is not SemVer: ${latest_release}" >&2; exit 1 ;;
esac
base_series="$(printf '%s\n' "${base_semver}" | awk -F. '{print $1 "." $2}')"
version="${base_series}.${xxxxx}.mod"
```

Do not use `git rev-parse --short=5 HEAD` for `xxxxx`: Git may print more than five characters to keep the abbreviation unique, but this release contract requires the literal first five characters.

Use the exact `version` value for:

- The Git tag.
- The GitHub release title.
- Release notes and final reporting.

The expected Linux binary path is repository-root `codex`:

```text
codex
```

Upload `codex` as the GitHub release asset. The tag and release title carry the version; the asset name stays stable across releases.

If the repository has a display-only TUI version label, verify that it matches the latest upstream SemVer release with the fork suffix before publishing. For example, latest `0.141.0` requires `0.141.0+gary` unless the user explicitly approved a different display label:

```bash
expected_display_version="${base_semver}+gary"
rg -n "CODEX_CLI_DISPLAY_VERSION.*${expected_display_version}" codex-rs/tui/src/version.rs
```

## Safety Checks

Run these checks before creating any tag or release:

```bash
git status --short
git rev-parse --verify HEAD
git rev-parse -q --verify "refs/tags/${version}"
git ls-remote --tags origin "refs/tags/${version}"
if gh release view "${version}" --repo garyfpga/codex-compact-fix >/dev/null 2>&1; then
  echo "release already exists: ${version}" >&2
  exit 1
fi
test -f codex
test -x codex
```

Stop if:

- A local tag, remote tag, or GitHub release already exists for `version`.
- The binary is missing from the repository root.
- The repository-root binary is not exactly `codex`, unless the user explicitly requested a different artifact path.
- The display-only TUI version label is stale relative to the latest upstream stable SemVer release.
- The release notes are unclear, stale, or not approved.
- The `gh` repository target is unclear.
- The publish command would overwrite or attach to unclear existing state.

Treat a failed `gh release view "${version}"` because the release does not exist as the expected state. Treat any successful view, authentication error, permission error, or ambiguous result as a stop condition until clarified.

## Reviewer Gate

Before running `git tag` or `gh release create`, use a `release-packaging-reviewer` subagent with the same model as main agent and `reasoning_effort = high`.

Ask the reviewer to check:

- The computed commit SHA, suffix, version, tag, and artifact path.
- The latest stable upstream release was checked and used as the base series.
- The build artifact is repository-root `codex`, executable, and built from final `HEAD`.
- The TUI display label matches the latest upstream SemVer release plus `+gary`, unless the user explicitly approved a different display label.
- The local tag, remote tag, and GitHub release do not already exist.
- The release notes and GitHub repository target are explicit.
- The publish commands will not overwrite unclear state.

Resolve reviewer findings before publishing. If no subagent facility is available in the current environment, stop and report that the reviewer gate cannot be completed.

## Publish Commands

After all checks and the reviewer gate pass, create the tag on the final release commit:

```bash
git tag -a "${version}" "${final_commit}" -m "${version}"
git push origin "refs/tags/${version}"
```

Create the GitHub release and upload the binary:

```bash
gh release create "${version}" "codex" \
  --repo garyfpga/codex-compact-fix \
  --verify-tag \
  --title "${version}" \
  --notes-file "${notes_file}"
```

Use the approved release notes file in place of `${notes_file}`. Preserve shell quoting so multiline or punctuation-heavy notes are passed exactly.

If `git tag`, `git push`, or `gh release create` partially succeeds and a later step fails, do not retry blindly. Inspect whether the tag or release exists locally or remotely, report the partial state, and ask for explicit recovery instructions.

## Self-Referential Version Caveat

The version includes the final commit SHA. Do not commit source text, docs, generated metadata, or other tracked files that embed `base.xxxxx.mod` into the same commit whose SHA is used to compute `xxxxx`; that is self-referential and changes the SHA being embedded.

If a tracked file must mention the final SHA-derived version, do it in a later commit or in release notes outside the final source commit. For this publish step, prefer naming the binary after the final commit is fixed, then tag and publish that commit.

## Completion Report

Report:

- Final commit SHA.
- Computed version/tag.
- Latest upstream stable release used for the base series.
- Uploaded artifact path/name.
- GitHub release URL, if `gh` provides one.
- Any partial state or manual follow-up required.
