Mod refresh release for upstream `da4c8ca57d40b074bdc1b5b1218851100150c56b`.

- Refreshes the compact-fix fork to the pinned upstream commit above.
- Preserves fork-local remote compact configuration, retry, fallback, service-tier, V1/V2 parity, and `.mod` version display behavior.
- Preserves session-only `/model` and persistent `/modelp` behavior.
- Includes upstream multi-agent hint configuration, websocket liveness/incremental fixes, telemetry updates, Bedrock metadata fixes, quick-xml dependency updates, and safety notice wording updates.
- Builds and uploads the Linux `codex` CLI artifact.
- Tests were not run for this release unless explicitly requested; release validation uses the Cargo release build and `./codex --version`.
- Bazel was not used; this release uses the Cargo release build path only.
