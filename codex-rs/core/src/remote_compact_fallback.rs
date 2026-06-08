use std::sync::Arc;

use crate::compact::InitialContextInjection;
use crate::compact::LocalCompactRunSettings;
use crate::compact::PreCompactHookPolicy;
use crate::compact::run_compact_task_after_turn_started_with_settings;
use crate::compact::run_inline_auto_compact_task_with_pre_hook_policy_and_settings;
use crate::compact::send_compact_turn_started;
use crate::compact_remote::RemoteCompactionRunSettings;
use crate::compact_remote::run_remote_compact_task_for_mode;
use crate::compact_service_tier::V1RemoteFirstCompactServiceTier;
use crate::compact_service_tier::resolve_v1_remote_first_compact_service_tiers;
use crate::context_manager::ContextManager;
use crate::session::session::Session;
use crate::session::turn_context::TurnContext;
use crate::tasks::emit_compact_metric;
use codex_analytics::CompactionPhase;
use codex_analytics::CompactionReason;
use codex_analytics::CompactionTrigger;
use codex_protocol::config_types::ServiceTier;
use codex_protocol::error::CodexErr;
use codex_protocol::error::Result as CodexResult;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::WarningEvent;
use codex_protocol::user_input::UserInput;

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
    let compact_service_tiers = resolve_v1_remote_first_compact_service_tiers(&sess, &turn_context);
    let original_service_tier = turn_context
        .config
        .service_tier
        .as_deref()
        .unwrap_or("default")
        .to_string();
    let emit_service_tier_status = compact_service_tiers
        .remote_service_tier_override
        .as_deref()
        .is_some_and(|service_tier| {
            service_tier == ServiceTier::Fast.request_value()
                && turn_context.config.service_tier.as_deref() != Some(service_tier)
        });
    let clean_history = sess.clone_history().await;
    if emit_service_tier_status {
        emit_compact_service_tier_status(
            &sess,
            &turn_context,
            format!(
                "Compact operations are using fast service tier (priority); normal requests will return to {original_service_tier} afterward."
            ),
        )
        .await;
    }

    match run_remote_attempt(&sess, &turn_context, kind, &compact_service_tiers).await {
        Ok(()) => {
            if emit_service_tier_status {
                emit_compact_service_tier_status(
                    &sess,
                    &turn_context,
                    format!(
                        "Compact operations finished; normal requests are using {original_service_tier} service tier again."
                    ),
                )
                .await;
            }
            return Ok(());
        }
        Err(CodexErr::Interrupted) => return Err(CodexErr::Interrupted),
        Err(CodexErr::TurnAborted) => return Err(CodexErr::TurnAborted),
        Err(_) => {}
    }

    restore_clean_history(&sess, &clean_history).await;
    emit_fallback_warning(&sess, &turn_context).await;
    emit_compact_metric(&sess.services.session_telemetry, "local", kind.is_manual());
    let result = run_local_fallback(
        Arc::clone(&sess),
        Arc::clone(&turn_context),
        kind,
        &compact_service_tiers,
    )
    .await;
    if emit_service_tier_status
        && !matches!(&result, Err(CodexErr::Interrupted | CodexErr::TurnAborted))
    {
        emit_compact_service_tier_status(
            &sess,
            &turn_context,
            format!(
                "Compact operations finished; normal requests are using {original_service_tier} service tier again."
            ),
        )
        .await;
    }
    result
}

async fn run_remote_attempt(
    sess: &Arc<Session>,
    turn_context: &Arc<TurnContext>,
    kind: V1RemoteCompactKind,
    compact_service_tiers: &V1RemoteFirstCompactServiceTier,
) -> CodexResult<()> {
    let (initial_context_injection, trigger, reason, phase) = kind.remote_args();
    run_remote_compact_task_for_mode(
        sess,
        turn_context,
        initial_context_injection,
        trigger,
        reason,
        phase,
        RemoteCompactionRunSettings {
            service_tier_override: compact_service_tiers.remote_service_tier_override.clone(),
        },
    )
    .await
}

async fn run_local_fallback(
    sess: Arc<Session>,
    turn_context: Arc<TurnContext>,
    kind: V1RemoteCompactKind,
    compact_service_tiers: &V1RemoteFirstCompactServiceTier,
) -> CodexResult<()> {
    let local_run_settings = LocalCompactRunSettings {
        service_tier_override: compact_service_tiers
            .local_fallback_service_tier_override
            .clone(),
    };
    match kind {
        V1RemoteCompactKind::Auto {
            initial_context_injection,
            reason,
            phase,
        } => {
            run_inline_auto_compact_task_with_pre_hook_policy_and_settings(
                sess,
                turn_context,
                initial_context_injection,
                reason,
                phase,
                PreCompactHookPolicy::SkipAlreadyRan,
                local_run_settings,
            )
            .await
        }
        V1RemoteCompactKind::Manual => {
            let input = vec![UserInput::Text {
                text: turn_context.compact_prompt().to_string(),
                // Compaction prompt is synthesized; no UI element ranges to preserve.
                text_elements: Vec::new(),
            }];
            run_compact_task_after_turn_started_with_settings(
                sess,
                turn_context,
                input,
                PreCompactHookPolicy::SkipAlreadyRan,
                local_run_settings,
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

async fn emit_compact_service_tier_status(
    sess: &Session,
    turn_context: &TurnContext,
    message: String,
) {
    sess.send_event(turn_context, EventMsg::Warning(WarningEvent { message }))
        .await;
}

async fn emit_fallback_warning(sess: &Session, turn_context: &TurnContext) {
    let max_attempts = turn_context.config.remote_compact.max_attempts;
    let message = format!(
        "Remote compact failed after {max_attempts} attempts; falling back to local compact."
    );
    sess.send_event(turn_context, EventMsg::Warning(WarningEvent { message }))
        .await;
}
