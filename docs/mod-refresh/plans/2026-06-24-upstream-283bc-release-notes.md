Mod refresh release for upstream `283bc4cf011047314b4804c0f1ccd06e4f6a95c5`.

- Refreshes the compact-fix fork to the pinned upstream commit above.
- Preserves fork-local remote compact configuration, retry, fallback, service-tier, V1/V2 parity, and `.mod` version display behavior.
- Preserves session-only `/model` and persistent `/modelp` behavior.
- Builds and uploads the Linux `codex` CLI artifact.
- Tests were not run for this release unless explicitly requested; release validation uses the Cargo release build and `./codex --version`.
- Bazel was not used; this release uses the Cargo release build path only.
