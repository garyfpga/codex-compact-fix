---
name: "mod-refresh-publish"
description: "Publish the final mod GitHub release by deriving the base series from the latest stable upstream Codex release, reading checked-in upstreamhash.txt and modversion.txt for the mod version, tagging the final commit, creating the GitHub release, and uploading repo-root codex. Use only when invoked by $mod-refresh-release, invoked by $mod-release-current, or explicitly asked to tag and publish by direct publish request."
---

# Mod Refresh Publish

## Purpose

Use this skill only for the final publish step of a mod release. The final release commit can come from either a completed upstream refresh merge or a current-HEAD feature release path. It tags the final release commit, creates the matching GitHub release, and uploads the built binary.

This skill mutates remote release state. Stop on ambiguity rather than guessing.

## Required Inputs

Require all of the following before publishing:

- A final release commit checked out as `HEAD`, from either a completed upstream refresh merge or a current-HEAD feature release path.
- A verified build artifact in the repository root for that final `HEAD`.
- Checked-in repository-root `upstreamhash.txt` and `modversion.txt` metadata files.
- Approved release notes or a release-plan entry that clearly states the notes to publish.
- A confirmed GitHub repository target for `gh release create`.

Do not continue if the release path, preflight/build provenance, artifact verification, release notes, metadata files, or publish destination is ambiguous. Do not continue if the working tree has uncommitted tracked changes that could affect the final release commit or checked-in metadata. If `upstreamhash.txt` or `modversion.txt` changes after computing the version, recompute and revalidate the version.

## Version Contract

Compute the release base from GitHub's latest non-draft, non-prerelease `openai/codex` release every time. Do not hardcode a base series such as `0.139`.

Use `gh release list` against the upstream repository, strip `rust-v` or `v`, require a SemVer `x.y.z` release name, and use the first two SemVer components as the mod release series. Read the checked-in repository-root metadata files, validate their exact shapes, derive `upstream_short` from the first five characters of `upstreamhash.txt`, and compute the release version:

```bash
final_commit="$(git rev-parse HEAD)"
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

Do not use the final release commit SHA, `git rev-parse --short`, or the upstream SemVer patch component as version inputs. The release suffix comes only from checked-in `upstreamhash.txt` and `modversion.txt`.

Use the exact `version` value for:

- The annotated Git tag.
- The pushed tag ref.
- The GitHub release tag argument.
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
git ls-files --error-unmatch upstreamhash.txt modversion.txt >/dev/null
git diff --quiet -- upstreamhash.txt modversion.txt
git diff --cached --quiet -- upstreamhash.txt modversion.txt
perl -0ne 'exit(/\A[0-9a-f]{40}\n\z/ ? 0 : 1)' upstreamhash.txt
perl -0ne 'exit(/\A[1-9][0-9]*\n\z/ ? 0 : 1)' modversion.txt
if git rev-parse -q --verify "refs/tags/${version}" >/dev/null; then
  echo "local tag already exists: ${version}" >&2
  exit 1
fi
if git ls-remote --exit-code --tags origin "refs/tags/${version}" >/dev/null; then
  echo "remote tag already exists: ${version}" >&2
  exit 1
fi
if gh release view "${version}" --repo garyfpga/codex-compact-fix >/dev/null 2>&1; then
  echo "release already exists: ${version}" >&2
  exit 1
fi
test -f codex
test -x codex
```

Stop if:

- `upstreamhash.txt` or `modversion.txt` is absent, untracked, dirty, staged but uncommitted, or fails exact shape validation.
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

- The computed commit SHA, upstream SHA, mod version, upstream short value, version, tag, and artifact path.
- The latest stable upstream release was checked and used as the base series.
- `upstreamhash.txt` and `modversion.txt` are checked in, clean, and valid.
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

## Completion Report

Report:

- Final commit SHA.
- Upstream SHA from `upstreamhash.txt`.
- Mod version from `modversion.txt`.
- Computed version/tag.
- Latest upstream stable release used for the base series.
- Uploaded artifact path/name.
- GitHub release URL, if `gh` provides one.
- Any partial state or manual follow-up required.
