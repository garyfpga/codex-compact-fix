Mod refresh release for upstream `cfead68e5d3984b247cf0758e3e53b19165de848`.

- Refreshes the compact-fix fork to the pinned upstream commit above.
- Preserves fork-local remote compact configuration, retry, fallback, service-tier, V1/V2 parity, and `.mod` version display behavior.
- Preserves session-only `/model` and persistent `/modelp` behavior.
- Builds and uploads the Linux `codex` CLI artifact.
- Tests were not run for this release unless explicitly requested; release validation uses the Cargo release build and `./codex --version`.
- Bazel was not used; this release uses the Cargo release build path only.
