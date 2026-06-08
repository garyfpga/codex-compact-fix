use std::sync::Arc;
use std::time::Duration;

use crate::compact::InitialContextInjection;
use crate::compact::PreCompactHookPolicy;
use crate::compact::run_compact_task_after_turn_started;
use crate::compact::run_inline_auto_compact_task_with_pre_hook_policy;
use crate::compact::send_compact_turn_started;
use crate::compact_remote::RemoteCompactionFailureMode;
use crate::compact_remote::run_remote_compact_task_for_mode;
use crate::context_manager::ContextManager;
use crate::session::session::Session;
use crate::session::turn_context::TurnContext;
use crate::tasks::emit_compact_metric;
use codex_analytics::CompactionPhase;
use codex_analytics::CompactionReason;
use codex_analytics::CompactionTrigger;
use codex_protocol::error::CodexErr;
use codex_protocol::error::Result as CodexResult;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::WarningEvent;
use codex_protocol::user_input::UserInput;

pub(crate) const REMOTE_COMPACT_TOTAL_ATTEMPTS: u64 = 3;
pub(crate) const REMOTE_COMPACT_ATTEMPT_TIMEOUT: Duration = Duration::from_secs(180);

const REMOTE_COMPACT_FALLBACK_WARNING: &str =
    "Remote compact failed after 3 attempts; falling back to local compact.";

pub(crate) async fn run_v1_remote_first_auto_compact(
    sess: &Arc<Session>,
    turn_context: &Arc<TurnContext>,
    initial_context_injection: InitialContextInjection,
    reason: CompactionReason,
    phase: CompactionPhase,
) -> CodexResult<()> {
    run_v1_remote_first_compact(
        Arc::clone(sess),
        Arc::clone(turn_context),
        V1RemoteCompactKind::Auto {
            initial_context_injection,
            reason,
            phase,
        },
    )
    .await
}

pub(crate) async fn run_v1_remote_first_manual_compact(
    sess: Arc<Session>,
    turn_context: Arc<TurnContext>,
) -> CodexResult<()> {
    send_compact_turn_started(&sess, &turn_context).await;
    run_v1_remote_first_compact(sess, turn_context, V1RemoteCompactKind::Manual).await
}

#[derive(Clone, Copy)]
enum V1RemoteCompactKind {
    Auto {
        initial_context_injection: InitialContextInjection,
        reason: CompactionReason,
        phase: CompactionPhase,
    },
    Manual,
}

impl V1RemoteCompactKind {
    fn is_manual(self) -> bool {
        matches!(self, V1RemoteCompactKind::Manual)
    }

    fn remote_args(
        self,
    ) -> (
        InitialContextInjection,
        CompactionTrigger,
        CompactionReason,
        CompactionPhase,
    ) {
        match self {
            V1RemoteCompactKind::Auto {
                initial_context_injection,
                reason,
                phase,
            } => (
                initial_context_injection,
                CompactionTrigger::Auto,
                reason,
                phase,
            ),
            V1RemoteCompactKind::Manual => (
                InitialContextInjection::DoNotInject,
                CompactionTrigger::Manual,
                CompactionReason::UserRequested,
                CompactionPhase::StandaloneTurn,
            ),
        }
    }
}

async fn run_v1_remote_first_compact(
    sess: Arc<Session>,
    turn_context: Arc<TurnContext>,
    kind: V1RemoteCompactKind,
) -> CodexResult<()> {
    let clean_history = sess.clone_history().await;
    match run_remote_attempt(&sess, &turn_context, kind).await {
        Ok(()) => return Ok(()),
        Err(CodexErr::Interrupted) => return Err(CodexErr::Interrupted),
        Err(CodexErr::TurnAborted) => return Err(CodexErr::TurnAborted),
        Err(_) => {}
    }

    restore_clean_history(&sess, &clean_history).await;
    emit_fallback_warning(&sess, &turn_context).await;
    emit_compact_metric(&sess.services.session_telemetry, "local", kind.is_manual());
    run_local_fallback(sess, turn_context, kind).await
}

async fn run_remote_attempt(
    sess: &Arc<Session>,
    turn_context: &Arc<TurnContext>,
    kind: V1RemoteCompactKind,
) -> CodexResult<()> {
    let (initial_context_injection, trigger, reason, phase) = kind.remote_args();
    run_remote_compact_task_for_mode(
        sess,
        turn_context,
        initial_context_injection,
        trigger,
        reason,
        phase,
        RemoteCompactionFailureMode::FallbackToLocal,
    )
    .await
}

async fn run_local_fallback(
    sess: Arc<Session>,
    turn_context: Arc<TurnContext>,
    kind: V1RemoteCompactKind,
) -> CodexResult<()> {
    match kind {
        V1RemoteCompactKind::Auto {
            initial_context_injection,
            reason,
            phase,
        } => {
            run_inline_auto_compact_task_with_pre_hook_policy(
                sess,
                turn_context,
                initial_context_injection,
                reason,
                phase,
                PreCompactHookPolicy::SkipAlreadyRan,
            )
            .await
        }
        V1RemoteCompactKind::Manual => {
            let input = vec![UserInput::Text {
                text: turn_context.compact_prompt().to_string(),
                // Compaction prompt is synthesized; no UI element ranges to preserve.
                text_elements: Vec::new(),
            }];
            run_compact_task_after_turn_started(
                sess,
                turn_context,
                input,
                PreCompactHookPolicy::SkipAlreadyRan,
            )
            .await
        }
    }
}

async fn restore_clean_history(sess: &Session, clean_history: &ContextManager) {
    sess.replace_history(
        clean_history.raw_items().to_vec(),
        clean_history.reference_context_item(),
    )
    .await;
}

async fn emit_fallback_warning(sess: &Session, turn_context: &TurnContext) {
    sess.send_event(
        turn_context,
        EventMsg::Warning(WarningEvent {
            message: REMOTE_COMPACT_FALLBACK_WARNING.to_string(),
        }),
    )
    .await;
}
