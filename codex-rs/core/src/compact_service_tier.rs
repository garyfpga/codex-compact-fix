use crate::session::session::Session;
use crate::session::turn_context::TurnContext;
use codex_protocol::auth::AuthMode;
use codex_protocol::config_types::ServiceTier;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RemoteFirstCompactServiceTier {
    pub(crate) remote_service_tier_override: Option<String>,
    pub(crate) local_fallback_service_tier_override: Option<String>,
}

pub(crate) fn resolve_remote_first_compact_service_tiers(
    sess: &Session,
    turn_context: &TurnContext,
) -> RemoteFirstCompactServiceTier {
    if matches!(
        sess.services.auth_manager.auth_mode(),
        Some(AuthMode::ApiKey | AuthMode::BedrockApiKey)
    ) {
        return RemoteFirstCompactServiceTier {
            remote_service_tier_override: None,
            local_fallback_service_tier_override: turn_context.config.service_tier.clone(),
        };
    }

    if turn_context
        .model_info
        .supports_service_tier(ServiceTier::Fast.request_value())
    {
        let fast_service_tier = Some(ServiceTier::Fast.request_value().to_string());
        return RemoteFirstCompactServiceTier {
            remote_service_tier_override: fast_service_tier.clone(),
            local_fallback_service_tier_override: fast_service_tier,
        };
    }

    RemoteFirstCompactServiceTier {
        remote_service_tier_override: turn_context.config.service_tier.clone(),
        local_fallback_service_tier_override: turn_context.config.service_tier.clone(),
    }
}
