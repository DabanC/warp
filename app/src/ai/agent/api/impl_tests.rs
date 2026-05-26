use std::sync::Arc;

use futures::executor::block_on;
use futures_util::StreamExt;
use mockito::{Matcher, Server};

use super::{
    api_keys_with_warp_credit_fallback_setting, get_supported_cli_agent_tools, get_supported_tools,
};
use crate::ai::agent::api::RequestParams;
use crate::ai::blocklist::SessionContext;
use crate::ai::llms::LLMId;
use crate::server::server_api::ServerApiProvider;
use crate::terminal::model::session::SessionType;
use warp_core::features::FeatureFlag;
use warp_core::HostId;
use warp_multi_agent_api as api;

fn request_params_with_ask_user_question_enabled(ask_user_question_enabled: bool) -> RequestParams {
    let model = LLMId::from("test-model");

    RequestParams {
        input: vec![],
        conversation_token: None,
        forked_from_conversation_token: None,
        ambient_agent_task_id: None,
        tasks: vec![],
        existing_suggestions: None,
        metadata: None,
        session_context: SessionContext::new_for_test(),
        model: model.clone(),
        coding_model: model.clone(),
        cli_agent_model: model.clone(),
        computer_use_model: model,
        is_memory_enabled: false,
        warp_drive_context_enabled: false,
        context_window_limit: None,
        mcp_context: None,
        planning_enabled: true,
        should_redact_secrets: false,
        api_keys: None,
        custom_model_providers: None,
        allow_use_of_warp_credits: false,
        autonomy_level: api::AutonomyLevel::Supervised,
        isolation_level: api::IsolationLevel::None,
        web_search_enabled: false,
        computer_use_enabled: false,
        ask_user_question_enabled,
        research_agent_enabled: false,
        orchestration_enabled: false,
        supported_tools_override: None,
        parent_agent_id: None,
        agent_name: None,
    }
}

fn request_params_for_remote(host_id: Option<HostId>) -> RequestParams {
    let mut params = request_params_with_ask_user_question_enabled(false);
    params.session_context =
        SessionContext::new_with_session_type_for_test(Some(SessionType::WarpifiedRemote {
            host_id,
        }));
    params
}

#[test]
fn api_keys_with_warp_credit_fallback_setting_returns_none_without_keys_or_fallback() {
    let api_keys = api_keys_with_warp_credit_fallback_setting(None, false);

    assert!(api_keys.is_none());
}

#[test]
fn api_keys_with_warp_credit_fallback_setting_creates_fallback_only_api_keys() {
    let api_keys = api_keys_with_warp_credit_fallback_setting(None, true)
        .expect("fallback setting should create ApiKeys");

    assert!(api_keys.allow_use_of_warp_credits);
    assert!(api_keys.anthropic.is_empty());
    assert!(api_keys.openai.is_empty());
    assert!(api_keys.google.is_empty());
    assert!(api_keys.open_router.is_empty());
    assert!(api_keys.aws_credentials.is_none());
}

#[test]
fn api_keys_with_warp_credit_fallback_setting_preserves_existing_keys() {
    let api_keys = api_keys_with_warp_credit_fallback_setting(
        Some(api::request::settings::ApiKeys {
            anthropic: "anthropic-key".to_string(),
            openai: String::new(),
            google: String::new(),
            open_router: String::new(),
            allow_use_of_warp_credits: false,
            aws_credentials: None,
        }),
        true,
    )
    .expect("existing ApiKeys should be preserved");

    assert_eq!(api_keys.anthropic, "anthropic-key");
    assert!(api_keys.allow_use_of_warp_credits);
}
#[test]
fn supported_tools_omits_ask_user_question_when_disabled() {
    let params = request_params_with_ask_user_question_enabled(false);
    let supported_tools = get_supported_tools(&params);

    assert!(!supported_tools.contains(&api::ToolType::AskUserQuestion));
}

#[test]
fn supported_tools_includes_ask_user_question_when_enabled_and_feature_flag_is_enabled() {
    if !FeatureFlag::AskUserQuestion.is_enabled() {
        return;
    }

    let params = request_params_with_ask_user_question_enabled(true);
    let supported_tools = get_supported_tools(&params);

    assert!(supported_tools.contains(&api::ToolType::AskUserQuestion));
}

#[test]
fn supported_tools_include_upload_artifact_when_feature_flag_is_enabled() {
    let _flag = FeatureFlag::ArtifactCommand.override_enabled(true);
    let params = request_params_with_ask_user_question_enabled(false);
    let supported_tools = get_supported_tools(&params);

    assert!(supported_tools.contains(&api::ToolType::UploadFileArtifact));
}

#[test]
fn supported_tools_omit_upload_artifact_when_feature_flag_is_disabled() {
    let _flag = FeatureFlag::ArtifactCommand.override_enabled(false);
    let params = request_params_with_ask_user_question_enabled(false);
    let supported_tools = get_supported_tools(&params);

    assert!(!supported_tools.contains(&api::ToolType::UploadFileArtifact));
}

#[test]
fn remote_supported_tools_include_search_codebase_when_connected_and_feature_flag_is_enabled() {
    let _flag = FeatureFlag::RemoteCodebaseIndexing.override_enabled(true);
    let params = request_params_for_remote(Some(HostId::new("host".to_string())));
    let supported_tools = get_supported_tools(&params);
    let supported_cli_agent_tools = get_supported_cli_agent_tools(&params);

    assert!(supported_tools.contains(&api::ToolType::SearchCodebase));
    assert!(supported_cli_agent_tools.contains(&api::ToolType::SearchCodebase));
}
#[test]
fn remote_supported_tools_omit_search_codebase_when_feature_flag_is_disabled() {
    let _flag = FeatureFlag::RemoteCodebaseIndexing.override_enabled(false);
    let params = request_params_for_remote(Some(HostId::new("host".to_string())));
    let supported_tools = get_supported_tools(&params);
    let supported_cli_agent_tools = get_supported_cli_agent_tools(&params);

    assert!(!supported_tools.contains(&api::ToolType::SearchCodebase));
    assert!(!supported_cli_agent_tools.contains(&api::ToolType::SearchCodebase));
}

#[test]
fn remote_supported_tools_omit_search_codebase_when_remote_is_not_connected() {
    let _flag = FeatureFlag::RemoteCodebaseIndexing.override_enabled(true);
    let params = request_params_for_remote(None);
    let supported_tools = get_supported_tools(&params);
    let supported_cli_agent_tools = get_supported_cli_agent_tools(&params);

    assert!(!supported_tools.contains(&api::ToolType::SearchCodebase));
    assert!(!supported_cli_agent_tools.contains(&api::ToolType::SearchCodebase));
}


#[test]
fn local_only_custom_provider_model_resolution_matches_selected_config_key() {
    let params = request_params_with_custom_provider_model("cfg-selected");

    let model = super::resolve_custom_provider_model(&params)
        .expect("selected custom model config key should resolve");

    assert_eq!(model.base_url, "https://custom.example/v1");
    assert_eq!(model.api_key, "custom-key");
    assert_eq!(model.model_slug, "llama-local");
}

#[test]
fn local_only_custom_provider_model_resolution_ignores_non_selected_models() {
    let params = request_params_with_custom_provider_model("not-configured");

    assert!(super::resolve_custom_provider_model(&params).is_none());
}

fn request_params_with_custom_provider_model(selected_model: &str) -> RequestParams {
    let mut params = request_params_with_ask_user_question_enabled(false);
    params.model = LLMId::from(selected_model);
    params.custom_model_providers = Some(api::request::settings::CustomModelProviders {
        providers: vec![
            api::request::settings::custom_model_providers::CustomModelProvider {
                base_url: "https://other.example/v1".to_string(),
                api_key: "other-key".to_string(),
                models: vec![api::request::settings::custom_model_providers::CustomModel {
                    slug: "ignored-model".to_string(),
                    config_key: "cfg-other".to_string(),
                }],
            },
            api::request::settings::custom_model_providers::CustomModelProvider {
                base_url: "https://custom.example/v1".to_string(),
                api_key: "custom-key".to_string(),
                models: vec![api::request::settings::custom_model_providers::CustomModel {
                    slug: "llama-local".to_string(),
                    config_key: "cfg-selected".to_string(),
                }],
            },
        ],
    });
    params
}


#[test]
fn local_only_custom_provider_route_posts_to_configured_endpoint_without_warp_auth() {
    block_on(async {
        let mut server = Server::new();
        let mock = server
            .mock("POST", "/v1/chat/completions")
            .match_header("authorization", "Bearer custom-key")
            .match_header("content-type", Matcher::Regex("application/json.*".to_string()))
            .match_body(Matcher::PartialJson(serde_json::json!({
                "model": "llama-local",
                "messages": [
                    {"role": "user", "content": "列出当前目录"}
                ]
            })))
            .with_status(200)
            .with_body(r#"{
                "choices": [
                    {"message": {"role": "assistant", "content": "可以使用 `ls`。"}}
                ]
            }"#)
            .create();

        let mut params = request_params_with_custom_provider_model("cfg-selected");
        params.custom_model_providers.as_mut().unwrap().providers[1].base_url = server.url();
        params.input = vec![crate::ai::agent::AIAgentInput::UserQuery {
            query: "列出当前目录".to_string(),
            context: Arc::from([]),
            static_query_type: None,
            referenced_attachments: Default::default(),
            user_query_mode: crate::ai::agent::UserQueryMode::default(),
            running_command: None,
            intended_agent: None,
        }];
        params.tasks = vec![api::Task {
            id: "task-1".to_string(),
            description: String::new(),
            dependencies: None,
            messages: vec![],
            summary: String::new(),
            server_data: String::new(),
        }];

        let server_api = ServerApiProvider::new_for_test().get();
        let (_tx, cancellation_rx) = futures::channel::oneshot::channel();
        let mut stream = super::generate_multi_agent_output(server_api, params, cancellation_rx)
            .await
            .unwrap();

        let mut events = Vec::new();
        while let Some(event) = stream.next().await {
            events.push(event.unwrap());
        }

        mock.assert();
        assert!(matches!(
            events.first().and_then(|event| event.r#type.as_ref()),
            Some(api::response_event::Type::Init(_))
        ));
        assert!(events.iter().any(|event| matches!(
            event.r#type.as_ref(),
            Some(api::response_event::Type::ClientActions(actions))
                if actions.actions.iter().any(|action| matches!(
                    action.action.as_ref(),
                    Some(api::client_action::Action::AddMessagesToTask(add))
                        if add.messages.iter().any(|message| matches!(
                            message.message.as_ref(),
                            Some(api::message::Message::AgentOutput(output))
                                if output.text.contains("可以使用")
                        ))
                ))
        )));
        assert!(matches!(
            events.last().and_then(|event| event.r#type.as_ref()),
            Some(api::response_event::Type::Finished(finished))
                if matches!(finished.reason, Some(api::response_event::stream_finished::Reason::Done(_)))
        ));
    });
}

#[test]
fn local_only_non_custom_model_returns_local_configuration_message_without_warp_auth() {
    block_on(async {
        let mut params = request_params_with_custom_provider_model("not-configured");
        params.input = vec![crate::ai::agent::AIAgentInput::UserQuery {
            query: "列出当前目录".to_string(),
            context: Arc::from([]),
            static_query_type: None,
            referenced_attachments: Default::default(),
            user_query_mode: crate::ai::agent::UserQueryMode::default(),
            running_command: None,
            intended_agent: None,
        }];
        params.tasks = vec![api::Task {
            id: "task-1".to_string(),
            description: String::new(),
            dependencies: None,
            messages: vec![],
            summary: String::new(),
            server_data: String::new(),
        }];

        let server_api = ServerApiProvider::new_for_test().get();
        let (_tx, cancellation_rx) = futures::channel::oneshot::channel();
        let mut stream = super::generate_multi_agent_output(server_api, params, cancellation_rx)
            .await
            .unwrap();

        let mut events = Vec::new();
        while let Some(event) = stream.next().await {
            events.push(event.unwrap());
        }

        assert!(events.iter().any(|event| matches!(
            event.r#type.as_ref(),
            Some(api::response_event::Type::ClientActions(actions))
                if actions.actions.iter().any(|action| matches!(
                    action.action.as_ref(),
                    Some(api::client_action::Action::AddMessagesToTask(add))
                        if add.messages.iter().any(|message| matches!(
                            message.message.as_ref(),
                            Some(api::message::Message::AgentOutput(output))
                                if output.text.contains("custom AI model")
                                    && !output.text.contains("missing authentication credentials")
                        ))
                ))
        )));
        assert!(matches!(
            events.last().and_then(|event| event.r#type.as_ref()),
            Some(api::response_event::Type::Finished(finished))
                if matches!(finished.reason, Some(api::response_event::stream_finished::Reason::Done(_)))
        ));
    });
}
