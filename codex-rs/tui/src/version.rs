/// The current Codex CLI version as embedded at compile time.
pub const CODEX_CLI_VERSION: &str = match option_env!("CODEX_CLI_RELEASE_VERSION") {
    Some(version) => version,
    None => env!("CARGO_PKG_VERSION"),
};

/// Display-only version label used by the TUI status line and related previews.
pub const CODEX_CLI_DISPLAY_VERSION: &str = CODEX_CLI_VERSION;
