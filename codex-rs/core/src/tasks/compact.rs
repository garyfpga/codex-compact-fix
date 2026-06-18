use std::sync::Arc;

use super::SessionTask;
use super::SessionTaskContext;
use super::emit_compact_metric;
use crate::remote_compact_fallback::RemoteCompactVersion;
use crate::remote_compact_fallback::run_remote_first_manual_compact;
use crate::session::TurnInput;
use crate::session::turn_context::TurnContext;
use crate::state::TaskKind;
use codex_protocol::user_input::UserInput;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Copy, Default)]
pub(crate) struct CompactTask;

impl SessionTask for CompactTask {
    fn kind(&self) -> TaskKind {
        TaskKind::Compact
    }

    fn span_name(&self) -> &'static str {
        "session_task.compact"
    }

    async fn run(
        self: Arc<Self>,
        session: Arc<SessionTaskContext>,
        ctx: Arc<TurnContext>,
        _input: Vec<TurnInput>,
        _cancellation_token: CancellationToken,
    ) -> Option<String> {
        let session = session.clone_session();
        let _ = if crate::compact::should_use_remote_compact_task(ctx.provider.info()) {
            let version = if ctx
                .config
                .features
                .enabled(codex_features::Feature::RemoteCompactionV2)
            {
                RemoteCompactVersion::V2
            } else {
                RemoteCompactVersion::V1
            };
            run_remote_first_manual_compact(session.clone(), ctx, version).await
        } else {
            emit_compact_metric(
                &session.services.session_telemetry,
                "local",
                /*manual*/ true,
            );
            let input = vec![UserInput::Text {
                text: ctx
                    .config
                    .compact_prompt
                    .as_deref()
                    .unwrap_or(crate::compact::SUMMARIZATION_PROMPT)
                    .to_string(),
                // Compaction prompt is synthesized; no UI element ranges to preserve.
                text_elements: Vec::new(),
            }];
            crate::compact::run_compact_task(session.clone(), ctx, input).await
        };
        None
    }
}
