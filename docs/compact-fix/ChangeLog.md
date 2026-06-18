# Compact Fix ChangeLog

## Purpose
This document is the durable behavior map for the fork-local compact changes that sit on top of upstream merge-base `f42780109c`. It records the intentional delta that was present at planning-time head `b722574da3` and gives future agents a stable starting point for upstream merges, conflict resolution, and release preparation.

Start here before reading implementation code. The goal is to preserve the fork's compact behavior, not to rediscover it from a moving `upstream/main` pointer.

## Baseline
The baseline inventory is the fork delta from `f42780109c` to `b722574da3`, using `git diff --name-status f42780109c..b722574da3` as requested by the plan. Do not substitute the moving `upstream/main` ref for this inventory.

The inventory below intentionally excludes `docs/compact-fix/ChangeLog.md` and `docs/simplepower/plans/2026-06-18-compact-fix-changelog.md`; those belong to this documentation run, not the upstream compact-fix delta.

## Preservation Checklist
- Keep `remote_compact` config parsing, validation, defaults, and schema generation aligned.
- Keep the compact-only fast service tier override limited to compact work, not normal sampling.
- Keep the shared remote-first fallback wrapper as the single policy owner for auto and manual compact routing.
- Keep compact transport explicit: no hidden retries for V1 compact, and no accidental widening of retry behavior for ordinary Responses calls.
- Keep V1 visible attempt counts, timeout wording, fallback warnings, and clean-history restore behavior stable.
- Keep V2 policy parity with the shared wrapper, including the version-specific attempt budget and warning labels.
- Keep the compact integration tests and snapshots current whenever request shape or fallback text changes.
- Keep the TUI version label display-only and aligned with the current mod release base, currently `0.141.0+gary`.
- Keep the Simple Power plan trail intact so future merge agents can read the rationale before changing code.

## Behavior Changes

### 1. `remote_compact` config schema and validation
**Files**
- `codex-rs/config/src/types.rs`
- `codex-rs/config/src/config_toml.rs`
- `codex-rs/core/src/config/mod.rs`
- `codex-rs/core/config.schema.json`
- `codex-rs/core/src/config/config_tests.rs`

**Current anchors**
- `codex-rs/config/src/types.rs:203-218`
- `codex-rs/config/src/config_toml.rs:230-234`
- `codex-rs/core/src/config/mod.rs:196-204, 698-699, 1109-1127, 2420-2465, 3134, 3548`
- `codex-rs/core/config.schema.json:2533-2560, 5164-5172`
- `codex-rs/core/src/config/config_tests.rs:9784-9906`

**What changed**
`remote_compact` became a first-class TOML section with `max_attempts`, `attempt_timeout_sec`, and `tcp_keepalive_interval_ms`, plus an effective `RemoteCompactConfig` in `codex-core`. The resolver enforces the documented ranges, resolves defaults, and threads the result into `Config::remote_compact`. The schema and tests were updated with the new shape.

**Why**
The fork needs these knobs to keep compact behavior deterministic through later upstream merges. If the config contract drifts, the compact policy will silently drift with it.

**Future merge notes**
- If `ConfigToml` or nested config types change, regenerate the schema before claiming the merge is done.
- Preserve the exact validation bounds and default values unless the fork intentionally changes the compact contract.
- Keep the config tests aligned with the runtime defaults and validation errors.

### 2. Shared remote-first fallback policy for V1 and V2
**Files**
- `codex-rs/core/src/remote_compact_fallback.rs`
- `codex-rs/core/src/compact.rs`
- `codex-rs/core/src/lib.rs`

**Current anchors**
- `codex-rs/core/src/remote_compact_fallback.rs:29-343`
- `codex-rs/core/src/compact.rs:67-74, 125-218, 692-703`
- `codex-rs/core/src/lib.rs:19-23`

**What changed**
The fork now centralizes remote-first compaction in one wrapper that chooses `RemoteCompactVersion::V1` or `RemoteCompactVersion::V2`, runs the version-specific remote attempt, and falls back to local compaction with the same cleanup and warning policy. `compact.rs` gained named local compact settings and a pre-hook policy so local fallback can reuse local compaction without duplicate turn-start or hook behavior.

**Why**
This keeps the compact policy in one place. Future merges are much safer when V1 and V2 share fallback, warning, telemetry, and clean-history behavior instead of carrying separate policy stacks.

**Future merge notes**
- Preserve `RemoteCompactVersion::V2` as the feature-flagged default when remote compaction V2 is enabled.
- Keep `PreCompactHookPolicy::SkipAlreadyRan` on local fallback after a remote attempt has already run the hooks.
- Do not split V1 and V2 back into separate remote-first fallback policies unless the fork intentionally changes the compact contract.

### 3. Fast service tier override for compaction only
**Files**
- `codex-rs/core/src/compact_service_tier.rs`
- `codex-rs/core/src/remote_compact_fallback.rs`
- `codex-rs/core/src/compact.rs`

**Current anchors**
- `codex-rs/core/src/compact_service_tier.rs:7-38`
- `codex-rs/core/src/remote_compact_fallback.rs:141-164, 182-219, 276-319`
- `codex-rs/core/src/compact.rs:125-218`

**What changed**
The compact path resolves a compact-only service tier once, prefers `fast` when the authenticated model supports it, preserves API-key behavior by omitting the remote tier override, and restores normal request behavior after compact work finishes. Local fallback keeps the same compact-tier decision so a failed remote compact does not silently downgrade the fallback compact request.

**Why**
The fork wants compaction to use priority capacity when available without changing ordinary sampling traffic. That separation is user-visible through status messages and must not be lost in upstream service-tier refactors.

**Future merge notes**
- Keep the fast-tier override limited to compact work only.
- Preserve API-key behavior that omits `service_tier` from remote compact request bodies.
- Keep the start/finish status messages tied to compact work so normal request tier behavior remains clear.

### 4. Auto and manual compact call-site routing
**Files**
- `codex-rs/core/src/remote_compact_fallback.rs`
- `codex-rs/core/src/session/turn.rs`
- `codex-rs/core/src/tasks/compact.rs`
- `codex-rs/core/src/tasks/regular.rs`

**Current anchors**
- `codex-rs/core/src/remote_compact_fallback.rs:51-88, 134-221`
- `codex-rs/core/src/session/turn.rs:896-910`
- `codex-rs/core/src/tasks/compact.rs:34-43`
- `codex-rs/core/src/tasks/regular.rs:36-81`

**What changed**
Auto compact in the session turn path and manual compact in the compact task now both enter the same remote-first wrapper when the provider supports remote compaction. The call sites choose V2 when the feature flag is enabled and V1 otherwise, while ordinary turn execution remains ordinary-turn behavior.

**Why**
Auto and manual compact must not diverge during future upstream merges. The fork relies on both call sites preserving the same version choice, fallback policy, and local compact cleanup.

**Future merge notes**
- Do not split auto/manual compact back into separate remote-first policy stacks unless the fork intentionally changes the contract.
- Keep local-only compact behavior unchanged when the provider does not support remote compaction.
- Keep `tasks/regular.rs` ordinary-turn semantics separate from compact routing.

### 5. API/client retry and TCP keepalive support touched by compact
**Files**
- `codex-rs/codex-api/src/endpoint/compact.rs`
- `codex-rs/codex-api/src/endpoint/session.rs`
- `codex-rs/codex-api/src/lib.rs`
- `codex-rs/core/src/client.rs`
- `codex-rs/core/src/responses_retry.rs`
- `codex-rs/login/src/auth/default_client.rs`
- `codex-rs/login/src/auth/default_client_tests.rs`

**Current anchors**
- `codex-rs/codex-api/src/endpoint/compact.rs:17-105`
- `codex-rs/codex-api/src/endpoint/session.rs:19-178`
- `codex-rs/codex-api/src/lib.rs:1-109`
- `codex-rs/core/src/client.rs:158-165, 441-531`
- `codex-rs/core/src/responses_retry.rs:14-88`
- `codex-rs/login/src/auth/default_client.rs:198-273`
- `codex-rs/login/src/auth/default_client_tests.rs:7-18, 41-96`

**What changed**
The compact endpoint now accepts explicit retry-policy plumbing across the `codex-api` boundary. `CompactConversationRequestSettings` carries the compact request timeout and retry policy, and the V1 compact path uses a keepalive-configured reqwest client that preserves the normal Codex headers, sandbox proxy policy, Cloudflare cookie store, and custom CA behavior. The shared response-retry helper stays generic; the compact-specific V2 special case was removed from that layer.

**Why**
Compact needs transport behavior that is explicit and isolated from the rest of the Responses stack. That keeps the fork's compact contract testable without changing ordinary endpoint defaults.

**Future merge notes**
- Keep non-compact endpoints on their existing provider retry path.
- Preserve `build_reqwest_client()` behavior for ordinary traffic.
- Keep compact retry policy explicit instead of re-inferring it from provider defaults.
- Do not reintroduce compact-specific branches into the generic response retry helper.

### 6. V1 remote compact retry, timeout, and TCP keepalive plumbing
**Files**
- `codex-rs/core/src/compact_remote.rs`
- `codex-rs/core/src/remote_compact_fallback.rs`

**Current anchors**
- `codex-rs/core/src/compact_remote.rs:47-343`
- `codex-rs/core/src/remote_compact_fallback.rs:133-274, 331-343`

**What changed**
V1 remote compact now consumes the resolved `max_attempts`, `attempt_timeout`, and `tcp_keepalive_interval` values from config. Each visible attempt uses zero hidden transport retries, so one visible attempt maps to one `/responses/compact` HTTP request. Failures are categorized into timeout, unexpected HTTP, transport or stream, protocol/body parse, and catch-all warnings, and the wrapper falls back only after exhausting the configured attempt budget.

**Why**
This is the fork-local behavior that must not regress: bounded visible attempts, explicit timeout reporting, and clean fallback history. It is the main user-visible compact contract.

**Future merge notes**
- Do not let hidden transport retries creep back into V1 compact.
- Keep the warning wording and categories aligned with the existing helper.
- Keep failed remote artifacts out of the local fallback history.
- Preserve the fallback warning count and the configured attempt count in user-visible messages.

### 7. V2 remote compact policy parity
**Files**
- `codex-rs/core/src/compact_remote_v2.rs`
- `codex-rs/core/src/remote_compact_fallback.rs`
- `codex-rs/core/src/session/turn.rs`
- `codex-rs/core/src/tasks/compact.rs`
- `codex-rs/core/tests/suite/compact_remote_parity.rs`

**Current anchors**
- `codex-rs/core/src/compact_remote_v2.rs:53-364`
- `codex-rs/core/src/remote_compact_fallback.rs:224-274`
- `codex-rs/core/src/session/turn.rs:896-910`
- `codex-rs/core/src/tasks/compact.rs:34-43`
- `codex-rs/core/tests/suite/compact_remote_parity.rs:149-259`

**What changed**
V2 now receives the same visible attempt budget, timeout, and compact-tier override policy as V1 through the shared wrapper, but keeps its own stream-based attempt loop and output collection. The parity tests ensure API-key auth still omits `service_tier`, that the request bodies stay aligned where they should, and that V2 continues to use its own version-specific warnings and fallback path.

**Why**
The fork's intent is policy parity, not implementation parity. V2 can use its own transport shape, but it must still honor the same visible compact contract the fork promised.

**Future merge notes**
- Preserve the exact visible attempt budget and timeout semantics.
- Do not let hidden V2 stream retries inflate the visible attempt count.
- Keep the V1 and V2 warning labels distinct so future merges can reason about the path that failed.

### 8. Integration tests and snapshots that preserve compact behavior
**Files**
- `codex-rs/core/src/config/config_tests.rs`
- `codex-rs/core/tests/suite/compact_remote.rs`
- `codex-rs/core/tests/suite/compact_remote_parity.rs`
- `codex-rs/core/tests/suite/snapshots/all__suite__compact_remote__remote_pre_turn_compaction_failure_shapes.snap`

**Current anchors**
- `codex-rs/core/src/config/config_tests.rs:9784-9906`
- `codex-rs/core/tests/suite/compact_remote.rs:145-155, 329-455, 541-560, 1260-1731, 2868-3482, 4663-4770`
- `codex-rs/core/tests/suite/compact_remote_parity.rs:149-259`
- `codex-rs/core/tests/suite/snapshots/all__suite__compact_remote__remote_pre_turn_compaction_failure_shapes.snap:1-25`

**What changed**
The config tests lock the new `remote_compact` defaults, overrides, and validation bounds. The compact integration suite now covers exact attempt counts, timeout wording, fallback cleanliness, unexpected HTTP body limits, and the pre-turn failure shape that now continues through local fallback. The parity tests keep the V1/V2 request shapes aligned where intended. The request-shape snapshot now shows the local fallback compact request and the post-fallback sampling request.

**Why**
These are the guardrails future merge agents need. Compact regressions tend to show up first as request-shape drift, warning-text drift, or a snapshot mismatch, so the tests and snapshots must stay in sync with the runtime contract.

**Future merge notes**
- Update the compact snapshots whenever a request shape or warning text changes intentionally.
- Rerun the focused compact tests after any merge conflict resolution in these files.
- Keep the parity test normalization aligned with intentional V1/V2 differences only.

### 9. Display-only TUI version label `0.141.0+gary`
**Files**
- `codex-rs/tui/src/version.rs`
- `codex-rs/tui/src/chatwidget/status_surfaces.rs`
- `codex-rs/tui/src/bottom_pane/status_surface_preview.rs`
- `codex-rs/tui/src/chatwidget/tests/status_surface_previews.rs`
- `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__status_surface_previews_codex_version.snap`

**Current anchors**
- `codex-rs/tui/src/version.rs:1-8`
- `codex-rs/tui/src/chatwidget/status_surfaces.rs:1-18, 625-625, 739-741`
- `codex-rs/tui/src/bottom_pane/status_surface_preview.rs:40-104, 58-58`
- `codex-rs/tui/src/chatwidget/tests/status_surface_previews.rs:152-162`
- `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__status_surface_previews_codex_version.snap:1-6`

**What changed**
`CODEX_CLI_DISPLAY_VERSION` now holds `0.141.0+gary`, and both the status line and terminal title preview route `CodexVersion` to that display-only constant. The preview placeholder and snapshot test follow the same label.

**Why**
The fork wants a visible version label that can differ from package metadata without altering release checks or external version behavior.

**Future merge notes**
- Never swap the TUI display path back to `CARGO_PKG_VERSION`.
- Keep the snapshot updated if the display label intentionally changes, including when the mod release base advances with upstream stable releases.
- Preserve the separation between display-only UI copy and package metadata.

### 10. Simple Power plan history as the rationale trail
**Files**
- `docs/simplepower/plans/2026-06-08-remote-compact-timeout-fallback.md`
- `docs/simplepower/plans/2026-06-08-compact-fast-service-tier-override.md`
- `docs/simplepower/plans/2026-06-08-v1-remote-compact-config.md`
- `docs/simplepower/plans/2026-06-09-v2-remote-compact-policy.md`
- `docs/simplepower/plans/2026-06-11-upstream-main-compact-preserve-gary-version.md`

**Current anchors**
- `docs/simplepower/plans/2026-06-08-remote-compact-timeout-fallback.md:1-18, 21-86, 157-201`
- `docs/simplepower/plans/2026-06-08-compact-fast-service-tier-override.md:1-18, 21-100, 185-248`
- `docs/simplepower/plans/2026-06-08-v1-remote-compact-config.md:1-18, 28-130, 218-306`
- `docs/simplepower/plans/2026-06-09-v2-remote-compact-policy.md:1-18, 21-124, 203-244`
- `docs/simplepower/plans/2026-06-11-upstream-main-compact-preserve-gary-version.md:1-18, 23-90, 111-208`

**What changed**
These plans form the rationale chain for the fork-local compact behavior: first the bounded V1 timeout/fallback policy, then the compact-only fast-tier override, then the V1 `remote_compact` config and TCP keepalive contract, then V2 policy parity, and finally the upstream merge and display-label preservation step.

**Why**
Future merge agents need the human-approved intent, not just the code, when deciding whether a conflict is a regression or a deliberate fork-local behavior.

**Future merge notes**
- Read these plans before resolving any later upstream merge that touches compact or the TUI version label.
- Update this section if the preserved behavior set changes in a new approved plan.

## Changed File Inventory
```text
M	codex-rs/codex-api/src/endpoint/compact.rs
M	codex-rs/codex-api/src/endpoint/session.rs
M	codex-rs/codex-api/src/lib.rs
M	codex-rs/config/src/config_toml.rs
M	codex-rs/config/src/types.rs
M	codex-rs/core/config.schema.json
M	codex-rs/core/src/client.rs
M	codex-rs/core/src/compact.rs
M	codex-rs/core/src/compact_remote.rs
M	codex-rs/core/src/compact_remote_v2.rs
A	codex-rs/core/src/compact_service_tier.rs
M	codex-rs/core/src/config/config_tests.rs
M	codex-rs/core/src/config/mod.rs
M	codex-rs/core/src/lib.rs
A	codex-rs/core/src/remote_compact_fallback.rs
M	codex-rs/core/src/responses_retry.rs
M	codex-rs/core/src/session/turn.rs
M	codex-rs/core/src/tasks/compact.rs
M	codex-rs/core/src/tasks/regular.rs
M	codex-rs/core/tests/suite/compact_remote.rs
M	codex-rs/core/tests/suite/compact_remote_parity.rs
M	codex-rs/core/tests/suite/snapshots/all__suite__compact_remote__remote_pre_turn_compaction_failure_shapes.snap
M	codex-rs/login/src/auth/default_client.rs
M	codex-rs/login/src/auth/default_client_tests.rs
M	codex-rs/tui/src/bottom_pane/status_surface_preview.rs
A	codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__status_surface_previews_codex_version.snap
M	codex-rs/tui/src/chatwidget/status_surfaces.rs
M	codex-rs/tui/src/chatwidget/tests/status_surface_previews.rs
M	codex-rs/tui/src/version.rs
A	docs/simplepower/plans/2026-06-08-compact-fast-service-tier-override.md
A	docs/simplepower/plans/2026-06-08-remote-compact-timeout-fallback.md
A	docs/simplepower/plans/2026-06-08-v1-remote-compact-config.md
A	docs/simplepower/plans/2026-06-09-v2-remote-compact-policy.md
A	docs/simplepower/plans/2026-06-11-upstream-main-compact-preserve-gary-version.md
```

## Future Upstream Merge Procedure
1. Re-read this changelog before touching any compact or TUI version files.
2. Re-run the baseline inventory command `git diff --name-status f42780109c..b722574da3` and compare it to the inventory section above.
3. Resolve overlaps in this order: config/schema, shared fallback routing, V1 runtime plumbing, V2 parity, tests/snapshots, TUI display label.
4. Preserve the compact-only fast-tier override and the display-only version label unless a new approved plan explicitly changes them.
5. If a merge changes any `ConfigToml` or `remote_compact` shape, regenerate the schema before claiming the merge is done.
6. If a merge changes request shape or warning text, update the compact tests and snapshots in the same change.
7. If a merge touches the TUI version label, update the display snapshot and keep the label display-only.
8. After the merge is stable, update this changelog so the next agent starts from the current fork delta instead of a stale summary.
