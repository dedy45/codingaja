use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use runtime::{
    ApiClient, ApiRequest, AssistantEvent, ConversationRuntime, PermissionMode, PermissionPolicy,
    RuntimeError, Session, StaticToolExecutor, TokenUsage,
};

struct CountingSingleReplyApiClient {
    calls: Arc<AtomicUsize>,
}

impl ApiClient for CountingSingleReplyApiClient {
    fn stream(&mut self, _request: ApiRequest) -> Result<Vec<AssistantEvent>, RuntimeError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(vec![
            AssistantEvent::TextDelta("done".to_string()),
            AssistantEvent::Usage(TokenUsage {
                input_tokens: 5,
                output_tokens: 3,
                cache_creation_input_tokens: 0,
                cache_read_input_tokens: 0,
            }),
            AssistantEvent::MessageStop,
        ])
    }
}

#[test]
fn parity_e2e_runtime_single_turn_without_tools_is_single_submission() {
    let calls = Arc::new(AtomicUsize::new(0));
    let api_client = CountingSingleReplyApiClient {
        calls: Arc::clone(&calls),
    };
    let mut runtime = ConversationRuntime::new(
        Session::new(),
        api_client,
        StaticToolExecutor::new(),
        PermissionPolicy::new(PermissionMode::DangerFullAccess),
        vec!["system".to_string()],
    );

    let summary = runtime
        .run_turn("hello", None)
        .expect("single-turn reply should succeed");

    assert_eq!(summary.iterations, 1);
    assert_eq!(summary.assistant_messages.len(), 1);
    assert!(summary.tool_results.is_empty());
    assert_eq!(summary.usage.total_tokens(), 8);
    assert_eq!(runtime.session().messages.len(), 2);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

struct LoopingToolApiClient {
    calls: Arc<AtomicUsize>,
}

impl ApiClient for LoopingToolApiClient {
    fn stream(&mut self, _request: ApiRequest) -> Result<Vec<AssistantEvent>, RuntimeError> {
        let call_index = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
        Ok(vec![
            AssistantEvent::ToolUse {
                id: format!("tool-{call_index}"),
                name: "echo".to_string(),
                input: format!("payload-{call_index}"),
            },
            AssistantEvent::MessageStop,
        ])
    }
}

#[test]
fn parity_e2e_runtime_enforces_max_iterations_deterministically() {
    let calls = Arc::new(AtomicUsize::new(0));
    let api_client = LoopingToolApiClient {
        calls: Arc::clone(&calls),
    };
    let mut runtime = ConversationRuntime::new(
        Session::new(),
        api_client,
        StaticToolExecutor::new().register("echo", |input| Ok(input.to_string())),
        PermissionPolicy::new(PermissionMode::DangerFullAccess),
        vec!["system".to_string()],
    )
    .with_max_iterations(1);

    let error = runtime
        .run_turn("loop", None)
        .expect_err("second loop iteration should be blocked by max_iterations");

    assert!(error
        .to_string()
        .contains("maximum number of iterations"));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(runtime.session().messages.len(), 3);
}

struct UsageSequenceApiClient {
    calls: Arc<AtomicUsize>,
}

impl ApiClient for UsageSequenceApiClient {
    fn stream(&mut self, _request: ApiRequest) -> Result<Vec<AssistantEvent>, RuntimeError> {
        let call_index = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
        let usage = match call_index {
            1 => TokenUsage {
                input_tokens: 7,
                output_tokens: 5,
                cache_creation_input_tokens: 1,
                cache_read_input_tokens: 0,
            },
            2 => TokenUsage {
                input_tokens: 3,
                output_tokens: 2,
                cache_creation_input_tokens: 0,
                cache_read_input_tokens: 4,
            },
            _ => return Err(RuntimeError::new("unexpected extra API call")),
        };

        Ok(vec![
            AssistantEvent::TextDelta(format!("turn-{call_index}")),
            AssistantEvent::Usage(usage),
            AssistantEvent::MessageStop,
        ])
    }
}

#[test]
fn parity_e2e_runtime_usage_totals_match_turn_summaries() {
    let calls = Arc::new(AtomicUsize::new(0));
    let api_client = UsageSequenceApiClient {
        calls: Arc::clone(&calls),
    };
    let mut runtime = ConversationRuntime::new(
        Session::new(),
        api_client,
        StaticToolExecutor::new(),
        PermissionPolicy::new(PermissionMode::DangerFullAccess),
        vec!["system".to_string()],
    );

    let first = runtime.run_turn("first", None).expect("first turn");
    assert_eq!(first.usage.total_tokens(), 13);
    assert_eq!(runtime.usage().cumulative_usage().total_tokens(), 13);

    let second = runtime.run_turn("second", None).expect("second turn");
    assert_eq!(second.usage.total_tokens(), 22);
    assert_eq!(runtime.usage().cumulative_usage().total_tokens(), 22);
    assert_eq!(runtime.usage().turns(), 2);

    assert_eq!(calls.load(Ordering::SeqCst), 2);
}
