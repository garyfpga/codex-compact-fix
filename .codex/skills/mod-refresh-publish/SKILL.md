---
name: "mod-refresh-publish"
description: "Publish the final mod GitHub release by deriving the version from the final release commit SHA, tagging that commit, creating the GitHub release, and uploading the Linux binary. Use only when invoked by $mod-refresh-release, invoked by $mod-release-current, or explicitly asked to tag and publish by direct publish request."
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

Compute the release suffix from the first five characters of the final release commit SHA:

```bash
final_commit="$(git rev-parse HEAD)"
xxxxx="$(git rev-parse --short=5 HEAD)"
version="0.139.${xxxxx}.mod"
```

Use the exact `version` value for:

- The Git tag.
- The GitHub release title.
- The uploaded binary name.

The expected Linux binary name is:

```text
codex-0.139.xxxxx.mod-linux
```

Replace `xxxxx` with the computed suffix. If the build artifact in the repo root has a different name, copy or rename it before publishing so the uploaded artifact includes the same version string:

```bash
cp <repo-root-binary> "codex-${version}-linux"
```

Keep the original build artifact only if the release plan or user explicitly requires it.

## Safety Checks

Run these checks before creating any tag or release:

```bash
git status --short
git rev-parse --verify HEAD
git rev-parse -q --verify "refs/tags/${version}"
git ls-remote --tags origin "refs/tags/${version}"
gh release view "${version}"
test -f "codex-${version}-linux"
```

Stop if:

- A local tag, remote tag, or GitHub release already exists for `version`.
- The binary is missing from the repository root.
- The artifact name does not include the computed `version`.
- The release notes are unclear, stale, or not approved.
- The `gh` repository target is unclear.
- The publish command would overwrite or attach to unclear existing state.

Treat a failed `gh release view "${version}"` because the release does not exist as the expected state. Treat any successful view, authentication error, permission error, or ambiguous result as a stop condition until clarified.

## Reviewer Gate

Before running `git tag` or `gh release create`, use a `release-packaging-reviewer` subagent with the same model as main agent and `reasoning_effort = high`.

Ask the reviewer to check:

- The computed commit SHA, suffix, version, tag, and artifact name.
- The build artifact is in the repository root and matches `codex-${version}-linux`.
- The local tag, remote tag, and GitHub release do not already exist.
- The release notes and GitHub repository target are explicit.
- The publish commands will not overwrite unclear state.

Resolve reviewer findings before publishing. If no subagent facility is available in the current environment, stop and report that the reviewer gate cannot be completed.

## Publish Commands

After all checks and the reviewer gate pass, create the tag on the final release commit:

```bash
git tag "${version}"
```

Create the GitHub release and upload the binary:

```bash
gh release create "${version}" "codex-${version}-linux" --title "${version}" --notes "<notes>"
```

Use the approved release notes in place of `<notes>`. Preserve shell quoting so multiline or punctuation-heavy notes are passed exactly.

If `git tag` succeeds but `gh release create` fails, do not retry blindly. Inspect whether the tag or release exists locally or remotely, report the partial state, and ask for explicit recovery instructions.

## Self-Referential Version Caveat

The version includes the final commit SHA. Do not commit source text, docs, generated metadata, or other tracked files that embed `0.139.xxxxx.mod` into the same commit whose SHA is used to compute `xxxxx`; that is self-referential and changes the SHA being embedded.

If a tracked file must mention the final SHA-derived version, do it in a later commit or in release notes outside the final source commit. For this publish step, prefer naming the binary after the final commit is fixed, then tag and publish that commit.

## Completion Report

Report:

- Final commit SHA.
- Computed version/tag.
- Uploaded artifact path/name.
- GitHub release URL, if `gh` provides one.
- Any partial state or manual follow-up required.
