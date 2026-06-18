/// The current Codex CLI version as embedded at compile time.
pub const CODEX_CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Display-only version label used by the TUI status line and related previews.
///
/// This intentionally differs from `CODEX_CLI_VERSION` so UI copy can change
/// without affecting package metadata or version checks.
pub const CODEX_CLI_DISPLAY_VERSION: &str = "0.141.0+gary";
