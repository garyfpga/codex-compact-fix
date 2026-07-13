use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;

use crate::Prompt;
use crate::compact::CompactionAnalyticsAttempt;
use crate::compact::CompactionAnalyticsDetails;
use crate::compact::InitialContextInjection;
use crate::compact::build_compaction_initial_context;
use crate::compact::compaction_status_from_result;
use crate::compact::insert_initial_context_before_last_real_user_or_summary;
use crate::compact_model_fallback::record_model_fallback;
use crate::context::world_state::WorldState;
use crate::context_manager::ContextManager;
use crate::hook_runtime::PostCompactHookOutcome;
use crate::hook_runtime::PreCompactHookOutcome;
use crate::hook_runtime::run_post_compact_hooks;
use crate::hook_runtime::run_pre_compact_hooks;
use crate::responses_metadata::CompactionTurnMetadata;
use crate::session::session::Session;
use crate::session::step_context::StepContext;
use crate::session::turn_context::TurnContext;
use codex_analytics::CompactionImplementation;
use codex_analytics::CompactionPhase;
use codex_analytics::CompactionReason;
use codex_analytics::CompactionTrigger;
use codex_api::RetryOn;
use codex_api::RetryPolicy;
use codex_protocol::error::CodexErr;
use codex_protocol::error::Result as CodexResult;
use codex_protocol::items::ContextCompactionItem;
use codex_protocol::items::TurnItem;
use codex_protocol::models::BaseInstructions;
use codex_protocol::models::FunctionCallOutputBody;
use codex_protocol::models::FunctionCallOutputPayload;
use codex_protocol::models::ResponseItem;
use codex_protocol::protocol::CompactedItem;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::WarningEvent;
use codex_rollout_trace::CompactionCheckpointTracePayload;
use tracing::error;

#[path = "compact_remote_request.rs"]
mod request;
use request::RemoteCompactAttempt;
use request::run_remote_compact_attempt;

const CONTEXT_WINDOW_TRUNCATED_OUTPUT_MESSAGE: &str =
    "Output exceeded the available model context and was truncated";

#[derive(Clone, Debug)]
pub(crate) struct RemoteCompactionRunSettings {
    pub(crate) service_tier_override: Option<String>,
    pub(crate) max_attempts: u64,
    pub(crate) attempt_timeout: Duration,
}

pub(crate) struct RemoteCompactAttemptBudget {
    total_attempts: u64,
    next_attempt: u64,
}

impl RemoteCompactAttemptBudget {
    pub(crate) fn new(total_attempts: u64) -> Self {
        Self {
            total_attempts,
            next_attempt: 1,
        }
    }

    pub(crate) fn take(&mut self) -> Option<(u64, u64)> {
        if self.next_attempt > self.total_attempts {
            return None;
        }
        let attempt_number = self.next_attempt;
        self.next_attempt += 1;
        Some((attempt_number, self.total_attempts))
    }

    pub(crate) fn has_remaining(&self) -> bool {
        self.next_attempt <= self.total_attempts
    }
}

pub(crate) fn no_hidden_remote_compact_retry_policy() -> RetryPolicy {
    RetryPolicy {
        max_attempts: 0,
        base_delay: Duration::ZERO,
        retry_on: RetryOn {
            retry_429: false,
            retry_5xx: false,
            retry_transport: false,
        },
    }
}

pub(crate) async fn run_remote_compact_task_for_mode(
    sess: &Arc<Session>,
    step_context: &Arc<StepContext>,
    fallback_step_context: Option<&Arc<StepContext>>,
    turn_state: Option<Arc<OnceLock<String>>>,
    initial_context_injection: InitialContextInjection,
    trigger: CompactionTrigger,
    reason: CompactionReason,
    phase: CompactionPhase,
    settings: RemoteCompactionRunSettings,
) -> CodexResult<()> {
    let turn_context = &step_context.turn;
    let compaction_metadata = CompactionTurnMetadata::new(
        trigger,
        reason,
        CompactionImplementation::ResponsesCompact,
        phase,
    );
    let mut analytics_details = CompactionAnalyticsDetails {
        active_context_tokens_before: Some(sess.get_total_token_usage().await),
        ..Default::default()
    };
    let attempt = CompactionAnalyticsAttempt::begin(
        sess.as_ref(),
        turn_context.as_ref(),
        trigger,
        reason,
        CompactionImplementation::ResponsesCompact,
        phase,
    )
    .await;
    let pre_compact_outcome = run_pre_compact_hooks(sess, turn_context, trigger).await;
    match pre_compact_outcome {
        PreCompactHookOutcome::Continue => {}
        PreCompactHookOutcome::Stopped => {
            let error = CodexErr::TurnAborted;
            attempt
                .track(
                    sess.as_ref(),
                    codex_analytics::CompactionStatus::Interrupted,
                    Some(&error),
                    analytics_details,
                )
                .await;
            return Err(error);
        }
    }
    let result = run_remote_compact_task_inner_impl(
        sess,
        step_context,
        fallback_step_context,
        turn_state,
        initial_context_injection,
        compaction_metadata,
        &mut analytics_details,
        &settings,
    )
    .await;
    let status = compaction_status_from_result(&result);
    let codex_error = result.as_ref().err();
    if result.is_ok() {
        let post_compact_outcome = run_post_compact_hooks(sess, turn_context, trigger).await;
        if let PostCompactHookOutcome::Stopped = post_compact_outcome {
            attempt
                .track(sess.as_ref(), status, codex_error, analytics_details)
                .await;
            return Err(CodexErr::TurnAborted);
        }
    }
    attempt
        .track(sess.as_ref(), status, codex_error, analytics_details)
        .await;
    result?;
    Ok(())
}

async fn run_remote_compact_task_inner_impl(
    sess: &Arc<Session>,
    step_context: &Arc<StepContext>,
    fallback_step_context: Option<&Arc<StepContext>>,
    turn_state: Option<Arc<OnceLock<String>>>,
    initial_context_injection: InitialContextInjection,
    compaction_metadata: CompactionTurnMetadata,
    analytics_details: &mut CompactionAnalyticsDetails,
    settings: &RemoteCompactionRunSettings,
) -> CodexResult<()> {
    let turn_context = &step_context.turn;
    let context_compaction_item = ContextCompactionItem::new();
    let compaction_id = context_compaction_item.id.clone();
    // Use the UI compaction item ID as the trace compaction ID so protocol lifecycle events,
    // endpoint attempts, and the installed history checkpoint all have one join key.
    let compaction_trace = sess.services.rollout_thread_trace.compaction_trace_context(
        turn_context.sub_id.as_str(),
        compaction_id.as_str(),
        turn_context.model_info.slug.as_str(),
        turn_context.provider.info().name.as_str(),
    );
    let compaction_item = TurnItem::ContextCompaction(context_compaction_item);
    sess.emit_turn_item_started(turn_context, &compaction_item)
        .await;
    let mut attempt_budget = RemoteCompactAttemptBudget::new(settings.max_attempts);
    let attempt = run_remote_compact_attempt(
        sess,
        step_context,
        turn_state.clone(),
        &compaction_trace,
        compaction_metadata,
        analytics_details,
        settings,
        &mut attempt_budget,
        fallback_step_context.is_some(),
    )
    .await;
    let (attempt, compaction_turn_context) = match attempt {
        Ok(attempt) => (attempt, turn_context),
        Err(error) => {
            let Some(fallback_step_context) =
                fallback_step_context.filter(|_| attempt_budget.has_remaining())
            else {
                return Err(error);
            };
            if !matches!(&error, CodexErr::InvalidRequest(_)) {
                return Err(error);
            }
            let fallback_turn_context = &fallback_step_context.turn;
            let fallback_compaction_trace =
                sess.services.rollout_thread_trace.compaction_trace_context(
                    fallback_turn_context.sub_id.as_str(),
                    compaction_id.as_str(),
                    fallback_turn_context.model_info.slug.as_str(),
                    fallback_turn_context.provider.info().name.as_str(),
                );
            let fallback_result = run_remote_compact_attempt(
                sess,
                fallback_step_context,
                turn_state,
                &fallback_compaction_trace,
                compaction_metadata,
                analytics_details,
                settings,
                &mut attempt_budget,
                false,
            )
            .await;
            record_model_fallback(
                &sess.services.session_telemetry,
                turn_context.model_info.slug.as_str(),
                fallback_turn_context.model_info.slug.as_str(),
                compaction_metadata.reason(),
                compaction_metadata.implementation(),
                fallback_result.as_ref().err(),
            );
            match fallback_result {
                Ok(attempt) => (attempt, fallback_turn_context),
                Err(err @ (CodexErr::Interrupted | CodexErr::TurnAborted)) => return Err(err),
                Err(_) => return Err(error),
            }
        }
    };
    let RemoteCompactAttempt {
        new_history,
        trace_input_history,
    } = attempt;
    let (new_window_number, new_window_ids) = sess.advance_auto_compact_window().await;
    let (new_history, world_state_baseline) = process_compacted_history(
        sess.as_ref(),
        compaction_turn_context.as_ref(),
        new_history,
        &initial_context_injection,
    )
    .await;

    let reference_context_item = match initial_context_injection {
        InitialContextInjection::DoNotInject => None,
        InitialContextInjection::BeforeLastUserMessage(_) => {
            Some(compaction_turn_context.to_turn_context_item())
        }
    };
    let compacted_item = CompactedItem {
        message: String::new(),
        replacement_history: Some(new_history.clone()),
        window_number: Some(new_window_number),
        first_window_id: Some(new_window_ids.first_window_id.to_string()),
        previous_window_id: new_window_ids.previous_window_id.map(|id| id.to_string()),
        window_id: Some(new_window_ids.window_id.to_string()),
    };
    // Install is the semantic boundary where the compact endpoint's output becomes live
    // thread history. Keep it distinct from the later inference request so the reducer can
    // still represent repeated developer/context prefix items exactly as the model saw them.
    compaction_trace.record_installed(&CompactionCheckpointTracePayload {
        input_history: &trace_input_history,
        replacement_history: &new_history,
    });
    sess.replace_compacted_history(
        compaction_turn_context.as_ref(),
        new_history,
        reference_context_item,
        world_state_baseline,
        compacted_item,
    )
    .await;
    sess.recompute_token_usage(compaction_turn_context).await;

    sess.emit_turn_item_completed(compaction_turn_context, compaction_item)
        .await;
    Ok(())
}

pub(crate) async fn log_remote_compaction_request_failure(
    sess: &Session,
    turn_context: &TurnContext,
    prompt: &Prompt,
    err: &CodexErr,
) {
    let active_context_tokens = sess.get_total_token_usage().await;
    let estimated_tokens_after_last_model_generated_item = sess
        .estimated_tokens_after_last_model_generated_item()
        .await;
    error!(
        turn_id = %turn_context.sub_id,
        active_context_tokens,
        estimated_tokens_after_last_model_generated_item,
        model_context_window_tokens = ?turn_context.model_context_window(),
        failing_compaction_request_input_items = prompt.input.len(),
        failing_compaction_request_instructions_bytes = prompt.base_instructions.text.len(),
        compact_error = %err,
        "remote compaction failed"
    );
}

pub(crate) async fn send_remote_compaction_attempt_warning(
    sess: &Session,
    turn_context: &TurnContext,
    version_label: &str,
    attempt_number: u64,
    total_attempts: u64,
    attempt_timeout: Duration,
    err: &CodexErr,
) {
    let message = remote_compaction_attempt_warning_message(
        version_label,
        attempt_number,
        total_attempts,
        attempt_timeout,
        err,
    );
    sess.send_event(turn_context, EventMsg::Warning(WarningEvent { message }))
        .await;
}

fn remote_compaction_attempt_warning_message(
    version_label: &str,
    attempt_number: u64,
    total_attempts: u64,
    attempt_timeout: Duration,
    err: &CodexErr,
) -> String {
    let action = if attempt_number < total_attempts {
        "; retrying remote compact."
    } else {
        "."
    };
    match remote_compaction_attempt_warning_kind(err) {
        RemoteCompactionAttemptWarningKind::Timeout => {
            let seconds = attempt_timeout.as_secs();
            format!(
                "{version_label} remote compact attempt {attempt_number}/{total_attempts} timed out after {seconds}s{action}"
            )
        }
        RemoteCompactionAttemptWarningKind::UnexpectedHttp => {
            format!(
                "{version_label} remote compact attempt {attempt_number}/{total_attempts} got unexpected HTTP response: {err}{action}"
            )
        }
        RemoteCompactionAttemptWarningKind::TransportOrStream => {
            format!(
                "{version_label} remote compact attempt {attempt_number}/{total_attempts} failed with transport or stream error: {err}{action}"
            )
        }
        RemoteCompactionAttemptWarningKind::ProtocolBodyParse => {
            let detail = if let CodexErr::Stream(message, _) = err {
                message.clone()
            } else {
                err.to_string()
            };
            format!(
                "{version_label} remote compact attempt {attempt_number}/{total_attempts} failed to parse remote compact response: {detail}{action}"
            )
        }
        RemoteCompactionAttemptWarningKind::Other => {
            format!(
                "{version_label} remote compact attempt {attempt_number}/{total_attempts} failed: {err}{action}"
            )
        }
    }
}

#[derive(Clone, Copy)]
enum RemoteCompactionAttemptWarningKind {
    Timeout,
    UnexpectedHttp,
    TransportOrStream,
    ProtocolBodyParse,
    Other,
}

fn remote_compaction_attempt_warning_kind(err: &CodexErr) -> RemoteCompactionAttemptWarningKind {
    match err {
        CodexErr::RequestTimeout | CodexErr::Timeout => RemoteCompactionAttemptWarningKind::Timeout,
        CodexErr::UnexpectedStatus(_) => RemoteCompactionAttemptWarningKind::UnexpectedHttp,
        CodexErr::Stream(message, _) => {
            if message.contains(" at line ") && message.contains(" column ") {
                RemoteCompactionAttemptWarningKind::ProtocolBodyParse
            } else {
                RemoteCompactionAttemptWarningKind::TransportOrStream
            }
        }
        CodexErr::ConnectionFailed(_) | CodexErr::ResponseStreamFailed(_) => {
            RemoteCompactionAttemptWarningKind::TransportOrStream
        }
        CodexErr::Json(_) => RemoteCompactionAttemptWarningKind::ProtocolBodyParse,
        CodexErr::TurnAborted
        | CodexErr::SessionBudgetExceeded
        | CodexErr::ContextWindowExceeded
        | CodexErr::ThreadNotFound(_)
        | CodexErr::AgentLimitReached { .. }
        | CodexErr::SessionConfiguredNotFirstEvent
        | CodexErr::Spawn
        | CodexErr::Interrupted
        | CodexErr::InvalidRequest(_)
        | CodexErr::InvalidImageRequest()
        | CodexErr::UsageLimitReached(_)
        | CodexErr::ServerOverloaded
        | CodexErr::CyberPolicy { .. }
        | CodexErr::QuotaExceeded
        | CodexErr::UsageNotIncluded
        | CodexErr::InternalServerError
        | CodexErr::RetryLimit(_)
        | CodexErr::InternalAgentDied
        | CodexErr::Sandbox(_)
        | CodexErr::LandlockSandboxExecutableNotProvided
        | CodexErr::UnsupportedOperation(_)
        | CodexErr::RefreshTokenFailed(_)
        | CodexErr::Fatal(_)
        | CodexErr::Io(_)
        | CodexErr::TokioJoin(_)
        | CodexErr::EnvVar(_) => RemoteCompactionAttemptWarningKind::Other,
        #[cfg(target_os = "linux")]
        CodexErr::LandlockRuleset(_) | CodexErr::LandlockPathFd(_) => {
            RemoteCompactionAttemptWarningKind::Other
        }
    }
}

pub(crate) async fn process_compacted_history(
    sess: &Session,
    turn_context: &TurnContext,
    mut compacted_history: Vec<ResponseItem>,
    initial_context_injection: &InitialContextInjection,
) -> (Vec<ResponseItem>, Option<Arc<WorldState>>) {
    // Mid-turn compaction is the only path that must inject initial context above the last user
    // message in the replacement history. Pre-turn compaction instead injects context after the
    // compaction item, but mid-turn compaction keeps the compaction item last for model training.
    let (initial_context, world_state_baseline) =
        build_compaction_initial_context(sess, turn_context, initial_context_injection).await;

    compacted_history.retain(should_keep_compacted_history_item);
    (
        insert_initial_context_before_last_real_user_or_summary(compacted_history, initial_context),
        world_state_baseline,
    )
}

/// Returns whether an item from remote compaction output should be preserved.
///
/// Called while processing the model-provided compacted transcript, before we
/// append fresh canonical context from the current session.
///
/// We drop:
/// - `developer` messages because remote output can include stale/duplicated
///   instruction content.
/// - non-user-content `user` messages (session prefix/instruction wrappers),
///   while preserving real user messages and persisted hook prompts.
///
/// This intentionally keeps:
/// - `assistant` messages (future remote compaction models may emit them)
/// - `user`-role warnings that parse as `TurnItem::UserMessage` and compaction-generated summary
///   messages. Legacy warning fragments are filtered by `parse_turn_item` before they reach this
///   check.
pub(crate) fn should_keep_compacted_history_item(item: &ResponseItem) -> bool {
    match item {
        ResponseItem::Message { role, .. } if role == "developer" => false,
        ResponseItem::Message { role, .. } if role == "user" => {
            matches!(
                crate::event_mapping::parse_turn_item(item),
                Some(TurnItem::UserMessage(_) | TurnItem::HookPrompt(_))
            )
        }
        ResponseItem::Message { role, .. } if role == "assistant" => true,
        ResponseItem::Message { .. } => false,
        ResponseItem::AgentMessage { .. } => true,
        ResponseItem::Compaction { .. } | ResponseItem::ContextCompaction { .. } => true,
        ResponseItem::CompactionTrigger { .. } => false,
        ResponseItem::AdditionalTools { .. }
        | ResponseItem::Reasoning { .. }
        | ResponseItem::LocalShellCall { .. }
        | ResponseItem::FunctionCall { .. }
        | ResponseItem::ToolSearchCall { .. }
        | ResponseItem::FunctionCallOutput { .. }
        | ResponseItem::ToolSearchOutput { .. }
        | ResponseItem::CustomToolCall { .. }
        | ResponseItem::CustomToolCallOutput { .. }
        | ResponseItem::WebSearchCall { .. }
        | ResponseItem::ImageGenerationCall { .. }
        | ResponseItem::Other => false,
    }
}

pub(crate) fn trim_function_call_history_to_fit_context_window(
    history: &mut ContextManager,
    turn_context: &TurnContext,
    base_instructions: &BaseInstructions,
) -> (usize, i64) {
    let Some(context_window) = turn_context.model_context_window() else {
        return (0, 0);
    };
    let mut rewritten_outputs = 0usize;
    let mut estimated_deleted_tokens = 0i64;
    let item_count = history.raw_items().len();

    for index in (0..item_count).rev() {
        let Some(estimated_tokens_before) =
            history.estimate_token_count_with_base_instructions(base_instructions)
        else {
            break;
        };
        if estimated_tokens_before <= context_window {
            break;
        }
        let Some(rewritten_item) = history
            .raw_items()
            .get(index)
            .and_then(rewritten_output_for_context_window)
        else {
            break;
        };
        let mut items = history.raw_items().to_vec();
        items[index] = rewritten_item;
        history.replace(items);
        let estimated_tokens_after = history
            .estimate_token_count_with_base_instructions(base_instructions)
            .unwrap_or_default();
        rewritten_outputs += 1;
        estimated_deleted_tokens = estimated_deleted_tokens
            .saturating_add(estimated_tokens_before.saturating_sub(estimated_tokens_after));
    }

    (rewritten_outputs, estimated_deleted_tokens)
}

fn rewritten_output_for_context_window(item: &ResponseItem) -> Option<ResponseItem> {
    Some(match item {
        ResponseItem::FunctionCallOutput {
            id,
            call_id,
            output,
            internal_chat_message_metadata_passthrough: metadata,
        } => ResponseItem::FunctionCallOutput {
            id: id.clone(),
            call_id: call_id.clone(),
            output: truncated_output_payload(output),
            internal_chat_message_metadata_passthrough: metadata.clone(),
        },
        ResponseItem::CustomToolCallOutput {
            id,
            call_id,
            name,
            output,
            internal_chat_message_metadata_passthrough: metadata,
        } => ResponseItem::CustomToolCallOutput {
            id: id.clone(),
            call_id: call_id.clone(),
            name: name.clone(),
            output: truncated_output_payload(output),
            internal_chat_message_metadata_passthrough: metadata.clone(),
        },
        ResponseItem::ToolSearchOutput {
            id,
            call_id,
            status,
            execution,
            internal_chat_message_metadata_passthrough: metadata,
            ..
        } => ResponseItem::ToolSearchOutput {
            id: id.clone(),
            call_id: call_id.clone(),
            status: status.clone(),
            execution: execution.clone(),
            tools: Vec::new(),
            internal_chat_message_metadata_passthrough: metadata.clone(),
        },
        _ => return None,
    })
}

fn truncated_output_payload(output: &FunctionCallOutputPayload) -> FunctionCallOutputPayload {
    FunctionCallOutputPayload {
        body: FunctionCallOutputBody::Text(CONTEXT_WINDOW_TRUNCATED_OUTPUT_MESSAGE.to_string()),
        success: output.success,
    }
}
