use std::path::Path;

use serde_json::{json, Value};

const ACTION_CAPABLE_FEATURES: &[&str] = &[
    "apps",
    "auth_elicitation",
    "browser_use",
    "browser_use_external",
    "browser_use_full_cdp_access",
    "code_mode_host",
    "computer_use",
    "goals",
    "guardian_approval",
    "hooks",
    "image_generation",
    "in_app_browser",
    "memories",
    "multi_agent",
    "plugin_sharing",
    "plugins",
    "remote_plugin",
    "shell_snapshot",
    "shell_tool",
    "skill_mcp_dependency_install",
    "tool_call_mcp_elicitation",
    "tool_suggest",
    "unified_exec",
    "workspace_dependencies",
];

pub fn app_server_args() -> Vec<String> {
    let mut args = vec![
        "app-server".to_string(),
        "--strict-config".to_string(),
        "-c".to_string(),
        "web_search=\"disabled\"".to_string(),
        "-c".to_string(),
        "cli_auth_credentials_store=\"keyring\"".to_string(),
        "-c".to_string(),
        "analytics.enabled=false".to_string(),
        "-c".to_string(),
        "mcp_servers={}".to_string(),
    ];
    for feature in ACTION_CAPABLE_FEATURES {
        args.push("-c".to_string());
        args.push(format!("features.{feature}=false"));
    }
    args.extend(["--listen".to_string(), "stdio://".to_string()]);
    args
}

pub fn initialize_params(version: &str) -> Value {
    json!({
        "clientInfo": {
            "name": "pebble",
            "title": "Pebble",
            "version": version
        },
        "capabilities": {
            "experimentalApi": false,
            "requestAttestation": false
        }
    })
}

pub fn thread_start_params(
    model: &str,
    cwd: &Path,
    base_instructions: &str,
    developer_instructions: &str,
) -> Value {
    json!({
        "model": model,
        "cwd": cwd,
        "approvalPolicy": "never",
        "sandbox": "read-only",
        "baseInstructions": base_instructions,
        "developerInstructions": developer_instructions,
        "ephemeral": true,
        "config": {
            "web_search": "disabled",
            "mcp_servers": {},
            "features": disabled_feature_config()
        }
    })
}

pub fn turn_start_params(thread_id: &str, input: Vec<Value>, model: &str, effort: &str) -> Value {
    json!({
        "threadId": thread_id,
        "input": input,
        "approvalPolicy": "never",
        "model": model,
        "effort": effort,
        "summary": "none"
    })
}

fn disabled_feature_config() -> Value {
    Value::Object(
        ACTION_CAPABLE_FEATURES
            .iter()
            .map(|feature| ((*feature).to_string(), Value::Bool(false)))
            .collect(),
    )
}

pub fn allowed_stream_item_type(item_type: &str) -> bool {
    matches!(item_type, "userMessage" | "agentMessage" | "reasoning")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use serde_json::json;

    use super::{
        allowed_stream_item_type, app_server_args, initialize_params, thread_start_params,
        turn_start_params, ACTION_CAPABLE_FEATURES,
    };

    #[test]
    fn disables_every_action_capable_codex_feature_and_external_context() {
        let args = app_server_args();
        for feature in ACTION_CAPABLE_FEATURES {
            assert!(
                args.contains(&format!("features.{feature}=false")),
                "{feature} must be disabled before app-server startup"
            );
        }
        assert!(args.contains(&"web_search=\"disabled\"".to_string()));
        assert!(args.contains(&"mcp_servers={}".to_string()));

        let thread = thread_start_params(
            "gpt-test",
            Path::new("/private/pebble"),
            "base",
            "developer",
        );
        assert_eq!(thread["sandbox"], "read-only");
        assert_eq!(thread["approvalPolicy"], "never");
        assert_eq!(thread["ephemeral"], true);
        assert_eq!(thread["config"]["web_search"], "disabled");
        assert_eq!(thread["config"]["mcp_servers"], json!({}));
        for feature in ACTION_CAPABLE_FEATURES {
            assert_eq!(thread["config"]["features"][feature], false);
        }
        assert!(thread.get("environments").is_none());

        let turn = turn_start_params(
            "thread-1",
            vec![json!({"type": "text", "text": "visible only"})],
            "gpt-test",
            "medium",
        );
        assert_eq!(turn["approvalPolicy"], "never");
        assert!(turn.get("environments").is_none());
    }

    #[test]
    fn keeps_experimental_protocol_capabilities_disabled() {
        let params = initialize_params("0.0.0");
        assert_eq!(params["capabilities"]["experimentalApi"], false);
        assert_eq!(params["capabilities"]["requestAttestation"], false);
    }

    #[test]
    fn rejects_every_action_capable_stream_item() {
        for item_type in [
            "commandExecution",
            "fileChange",
            "mcpToolCall",
            "dynamicToolCall",
            "webSearch",
            "imageGeneration",
        ] {
            assert!(!allowed_stream_item_type(item_type));
        }
        assert!(allowed_stream_item_type("agentMessage"));
    }
}
