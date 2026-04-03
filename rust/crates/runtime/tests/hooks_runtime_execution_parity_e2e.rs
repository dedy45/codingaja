use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use runtime::{
    ApiClient, ApiRequest, AssistantEvent, ContentBlock, ConversationRuntime, PermissionMode,
    PermissionPolicy, RuntimeError, RuntimeFeatureConfig, RuntimeHookConfig, Session,
    StaticToolExecutor,
};

struct ToolThenDoneApiClient {
    calls: usize,
}

impl ApiClient for ToolThenDoneApiClient {
    fn stream(&mut self, request: ApiRequest) -> Result<Vec<AssistantEvent>, RuntimeError> {
        self.calls += 1;
        match self.calls {
            1 => Ok(vec![
                AssistantEvent::ToolUse {
                    id: "tool-1".to_string(),
                    name: "echo".to_string(),
                    input: "payload".to_string(),
                },
                AssistantEvent::MessageStop,
            ]),
            2 => {
                assert!(
                    request.messages.iter().any(|message| {
                        matches!(
                            message.blocks.first(),
                            Some(ContentBlock::ToolResult { .. })
                        )
                    }),
                    "second request should include a tool result",
                );
                Ok(vec![
                    AssistantEvent::TextDelta("done".to_string()),
                    AssistantEvent::MessageStop,
                ])
            }
            _ => Err(RuntimeError::new("unexpected extra API call")),
        }
    }
}

#[test]
fn parity_e2e_hooks_pre_tool_use_can_block_execution_in_conversation_loop() {
    let tool_calls = Arc::new(AtomicUsize::new(0));
    let tool_calls_for_executor = Arc::clone(&tool_calls);

    let mut runtime = ConversationRuntime::new_with_features(
        Session::new(),
        ToolThenDoneApiClient { calls: 0 },
        StaticToolExecutor::new().register("echo", move |_input| {
            tool_calls_for_executor.fetch_add(1, Ordering::SeqCst);
            Ok("tool-output".to_string())
        }),
        PermissionPolicy::new(PermissionMode::DangerFullAccess),
        vec!["system".to_string()],
        RuntimeFeatureConfig::default().with_hooks(RuntimeHookConfig::new(
            vec![hook_deny_command("blocked by pre hook")],
            Vec::new(),
        )),
    );

    let summary = runtime
        .run_turn("run hook flow", None)
        .expect("conversation should continue after pre-hook denial");

    assert_eq!(tool_calls.load(Ordering::SeqCst), 0);
    assert_eq!(summary.tool_results.len(), 1);
    let (is_error, output) = first_tool_result(&summary);
    assert!(is_error);
    assert!(
        output.contains("blocked by pre hook") || output.contains("denied tool"),
        "unexpected tool result output: {output:?}",
    );
}

#[test]
fn parity_e2e_hooks_post_tool_use_appends_feedback_to_tool_result() {
    let tool_calls = Arc::new(AtomicUsize::new(0));
    let tool_calls_for_executor = Arc::clone(&tool_calls);

    let mut runtime = ConversationRuntime::new_with_features(
        Session::new(),
        ToolThenDoneApiClient { calls: 0 },
        StaticToolExecutor::new().register("echo", move |_input| {
            tool_calls_for_executor.fetch_add(1, Ordering::SeqCst);
            Ok("tool-output".to_string())
        }),
        PermissionPolicy::new(PermissionMode::DangerFullAccess),
        vec!["system".to_string()],
        RuntimeFeatureConfig::default().with_hooks(RuntimeHookConfig::new(
            vec![hook_echo_command("pre hook ran")],
            vec![hook_echo_command("post hook ran")],
        )),
    );

    let summary = runtime
        .run_turn("run hook flow", None)
        .expect("conversation should succeed");

    assert_eq!(tool_calls.load(Ordering::SeqCst), 1);
    let (is_error, output) = first_tool_result(&summary);
    assert!(!is_error);
    assert!(output.contains("tool-output"));
    assert!(output.contains("pre hook ran"));
    assert!(output.contains("post hook ran"));
}

#[test]
fn parity_e2e_hooks_post_tool_use_can_flip_tool_result_to_error() {
    let tool_calls = Arc::new(AtomicUsize::new(0));
    let tool_calls_for_executor = Arc::clone(&tool_calls);

    let mut runtime = ConversationRuntime::new_with_features(
        Session::new(),
        ToolThenDoneApiClient { calls: 0 },
        StaticToolExecutor::new().register("echo", move |_input| {
            tool_calls_for_executor.fetch_add(1, Ordering::SeqCst);
            Ok("tool-output".to_string())
        }),
        PermissionPolicy::new(PermissionMode::DangerFullAccess),
        vec!["system".to_string()],
        RuntimeFeatureConfig::default().with_hooks(RuntimeHookConfig::new(
            Vec::new(),
            vec![hook_deny_command("blocked by post hook")],
        )),
    );

    let summary = runtime
        .run_turn("run hook flow", None)
        .expect("conversation should continue after post-hook denial");

    assert_eq!(tool_calls.load(Ordering::SeqCst), 1);
    let (is_error, output) = first_tool_result(&summary);
    assert!(is_error);
    assert!(output.contains("blocked by post hook") || output.contains("denied"));
}

#[test]
fn parity_e2e_hooks_pre_tool_use_can_rewrite_tool_input_before_execution() {
    let captured_input: Arc<std::sync::Mutex<Vec<String>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
    let captured_input_for_executor = Arc::clone(&captured_input);

    let mut runtime = ConversationRuntime::new_with_features(
        Session::new(),
        ToolThenDoneApiClient { calls: 0 },
        StaticToolExecutor::new().register("echo", move |input| {
            captured_input_for_executor
                .lock()
                .expect("input lock")
                .push(input.to_string());
            Ok(format!("echoed:{input}"))
        }),
        PermissionPolicy::new(PermissionMode::DangerFullAccess),
        vec!["system".to_string()],
        RuntimeFeatureConfig::default().with_hooks(RuntimeHookConfig::new(
            vec![hook_rewrite_input_command("rewritten-input")],
            Vec::new(),
        )),
    );

    let summary = runtime
        .run_turn("run hook rewrite", None)
        .expect("conversation should succeed");

    let (is_error, output) = first_tool_result(&summary);
    assert!(!is_error);
    assert!(output.contains("echoed:rewritten-input"));

    let captured = captured_input.lock().expect("input lock");
    assert_eq!(captured.as_slice(), ["rewritten-input"]);
}

#[test]
fn parity_e2e_hooks_post_tool_use_can_mutate_result_and_error_flag() {
    let mut runtime = ConversationRuntime::new_with_features(
        Session::new(),
        ToolThenDoneApiClient { calls: 0 },
        StaticToolExecutor::new().register("echo", |_input| Ok("tool-output".to_string())),
        PermissionPolicy::new(PermissionMode::DangerFullAccess),
        vec!["system".to_string()],
        RuntimeFeatureConfig::default().with_hooks(RuntimeHookConfig::new(
            Vec::new(),
            vec![hook_mutate_output_command("mutated-output", true)],
        )),
    );

    let summary = runtime
        .run_turn("run hook mutate", None)
        .expect("conversation should continue after mutation");

    let (is_error, output) = first_tool_result(&summary);
    assert!(is_error);
    assert!(output.contains("mutated-output"));
}

fn first_tool_result(summary: &runtime::TurnSummary) -> (bool, String) {
    let Some(message) = summary.tool_results.first() else {
        panic!("expected at least one tool result message");
    };
    let Some(ContentBlock::ToolResult {
        is_error, output, ..
    }) = message.blocks.first()
    else {
        panic!("expected tool result block");
    };
    (*is_error, output.clone())
}

#[cfg(windows)]
fn hook_echo_command(message: &str) -> String {
    format!("echo {message}")
}

#[cfg(not(windows))]
fn hook_echo_command(message: &str) -> String {
    format!("printf '{message}'")
}

#[cfg(windows)]
fn hook_deny_command(message: &str) -> String {
    format!("echo {message} & exit /b 2")
}

#[cfg(not(windows))]
fn hook_deny_command(message: &str) -> String {
    format!("printf '{message}'; exit 2")
}

#[cfg(windows)]
fn hook_rewrite_input_command(new_input: &str) -> String {
    format!("echo HOOK_REWRITE_INPUT:{new_input}")
}

#[cfg(not(windows))]
fn hook_rewrite_input_command(new_input: &str) -> String {
    format!("printf 'HOOK_REWRITE_INPUT:{new_input}'")
}

#[cfg(windows)]
fn hook_mutate_output_command(new_output: &str, is_error: bool) -> String {
    let flag = if is_error { "true" } else { "false" };
    format!("echo HOOK_REWRITE_OUTPUT:{new_output} & echo HOOK_OVERRIDE_IS_ERROR:{flag}")
}

#[cfg(not(windows))]
fn hook_mutate_output_command(new_output: &str, is_error: bool) -> String {
    let flag = if is_error { "true" } else { "false" };
    format!("printf 'HOOK_REWRITE_OUTPUT:{new_output}\\nHOOK_OVERRIDE_IS_ERROR:{flag}'")
}
