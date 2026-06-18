use std::sync::Arc;

use crate::client::ModelClientSession;
use crate::compact::InitialContextInjection;
use crate::compact::LocalCompactRunSettings;
use crate::compact::PreCompactHookPolicy;
use crate::compact::run_compact_task_after_turn_started_with_settings;
use crate::compact::run_inline_auto_compact_task_with_pre_hook_policy_and_settings;
use crate::compact::send_compact_turn_started;
use crate::compact_remote;
use crate::compact_remote::RemoteCompactionRunSettings;
use crate::compact_remote_v2;
use crate::compact_remote_v2::RemoteCompactionV2RunSettings;
use crate::compact_service_tier::RemoteFirstCompactServiceTier;
use crate::compact_service_tier::resolve_remote_first_compact_service_tiers;
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RemoteCompactVersion {
    V1,
    V2,
}

impl RemoteCompactVersion {
    fn display_name(self) -> &'static str {
        match self {
            RemoteCompactVersion::V1 => "V1",
            RemoteCompactVersion::V2 => "V2",
        }
    }

    fn metric_label(self) -> &'static str {
        match self {
            RemoteCompactVersion::V1 => "remote",
            RemoteCompactVersion::V2 => "remote_v2",
        }
    }
}

pub(crate) async fn run_remote_first_auto_compact(
    sess: &Arc<Session>,
    turn_context: &Arc<TurnContext>,
    client_session: &mut ModelClientSession,
    initial_context_injection: InitialContextInjection,
    reason: CompactionReason,
    phase: CompactionPhase,
    version: RemoteCompactVersion,
) -> CodexResult<()> {
    Box::pin(run_remote_first_compact(
        Arc::clone(sess),
        Arc::clone(turn_context),
        Some(client_session),
        RemoteCompactKind::Auto {
            initial_context_injection,
            reason,
            phase,
        },
        version,
    ))
    .await
}

pub(crate) async fn run_remote_first_manual_compact(
    sess: Arc<Session>,
    turn_context: Arc<TurnContext>,
    version: RemoteCompactVersion,
) -> CodexResult<()> {
    send_compact_turn_started(&sess, &turn_context).await;
    Box::pin(run_remote_first_compact(
        sess,
        turn_context,
        None,
        RemoteCompactKind::Manual,
        version,
    ))
    .await
}

#[derive(Clone, Copy)]
enum RemoteCompactKind {
    Auto {
        initial_context_injection: InitialContextInjection,
        reason: CompactionReason,
        phase: CompactionPhase,
    },
    Manual,
}

impl RemoteCompactKind {
    fn is_manual(self) -> bool {
        matches!(self, RemoteCompactKind::Manual)
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
            RemoteCompactKind::Auto {
                initial_context_injection,
                reason,
                phase,
            } => (
                initial_context_injection,
                CompactionTrigger::Auto,
                reason,
                phase,
            ),
            RemoteCompactKind::Manual => (
                InitialContextInjection::DoNotInject,
                CompactionTrigger::Manual,
                CompactionReason::UserRequested,
                CompactionPhase::StandaloneTurn,
            ),
        }
    }
}

async fn run_remote_first_compact(
    sess: Arc<Session>,
    turn_context: Arc<TurnContext>,
    client_session: Option<&mut ModelClientSession>,
    kind: RemoteCompactKind,
    version: RemoteCompactVersion,
) -> CodexResult<()> {
    let compact_service_tiers = resolve_remote_first_compact_service_tiers(&sess, &turn_context);
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

    emit_compact_metric(
        &sess.services.session_telemetry,
        version.metric_label(),
        kind.is_manual(),
    );

    match run_remote_attempt(
        &sess,
        &turn_context,
        client_session,
        kind,
        version,
        &compact_service_tiers,
    )
    .await
    {
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

    emit_fallback_warning(&sess, &turn_context, version).await;
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
    client_session: Option<&mut ModelClientSession>,
    kind: RemoteCompactKind,
    version: RemoteCompactVersion,
    compact_service_tiers: &RemoteFirstCompactServiceTier,
) -> CodexResult<()> {
    let (initial_context_injection, trigger, reason, phase) = kind.remote_args();
    let max_attempts = turn_context.config.remote_compact.max_attempts;
    let attempt_timeout = turn_context.config.remote_compact.attempt_timeout;
    match version {
        RemoteCompactVersion::V1 => {
            let turn_state = client_session
                .as_ref()
                .map(|client_session| client_session.turn_state());
            compact_remote::run_remote_compact_task_for_mode(
                sess,
                turn_context,
                turn_state,
                initial_context_injection,
                trigger,
                reason,
                phase,
                RemoteCompactionRunSettings {
                    service_tier_override: compact_service_tiers
                        .remote_service_tier_override
                        .clone(),
                    max_attempts,
                    attempt_timeout,
                },
            )
            .await
        }
        RemoteCompactVersion::V2 => {
            compact_remote_v2::run_remote_compact_task_for_mode(
                sess,
                turn_context,
                client_session,
                initial_context_injection,
                trigger,
                reason,
                phase,
                RemoteCompactionV2RunSettings {
                    service_tier_override: compact_service_tiers
                        .remote_service_tier_override
                        .clone(),
                    max_attempts,
                    attempt_timeout,
                },
            )
            .await
        }
    }
}

async fn run_local_fallback(
    sess: Arc<Session>,
    turn_context: Arc<TurnContext>,
    kind: RemoteCompactKind,
    compact_service_tiers: &RemoteFirstCompactServiceTier,
) -> CodexResult<()> {
    let local_run_settings = LocalCompactRunSettings {
        service_tier_override: compact_service_tiers
            .local_fallback_service_tier_override
            .clone(),
    };
    match kind {
        RemoteCompactKind::Auto {
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
        RemoteCompactKind::Manual => {
            let input = vec![UserInput::Text {
                text: turn_context
                    .config
                    .compact_prompt
                    .as_deref()
                    .unwrap_or(crate::compact::SUMMARIZATION_PROMPT)
                    .to_string(),
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

async fn emit_compact_service_tier_status(
    sess: &Session,
    turn_context: &TurnContext,
    message: String,
) {
    sess.send_event(turn_context, EventMsg::Warning(WarningEvent { message }))
        .await;
}

async fn emit_fallback_warning(
    sess: &Session,
    turn_context: &TurnContext,
    version: RemoteCompactVersion,
) {
    let max_attempts = turn_context.config.remote_compact.max_attempts;
    let version = version.display_name();
    let message = format!(
        "{version} remote compact failed after {max_attempts} attempts; falling back to local compact."
    );
    sess.send_event(turn_context, EventMsg::Warning(WarningEvent { message }))
        .await;
}
