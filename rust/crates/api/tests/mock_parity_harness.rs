use std::collections::HashMap;
use std::ffi::OsString;
use std::sync::Arc;
use std::sync::{Mutex as StdMutex, OnceLock};

use api::{
    AuthSource, InputContentBlock, InputMessage, MessageRequest, OutputContentBlock, ProviderClient,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

#[tokio::test]
async fn deterministic_mock_harness_keeps_provider_regression_tests_offline() {
    let _lock = env_lock();

    let claw_harness = DeterministicMockHarness::start(vec![
        MockResponse::json(
            "200 OK",
            "{\"id\":\"msg_parity\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"text\",\"text\":\"parity-ok\"}],\"model\":\"claude-sonnet-4-6\",\"stop_reason\":\"end_turn\",\"stop_sequence\":null,\"usage\":{\"input_tokens\":4,\"output_tokens\":2}}",
        ),
    ])
    .await;

    let claw_client = ProviderClient::from_model_with_default_auth(
        "claude-sonnet-4-6",
        Some(AuthSource::ApiKey("test-key".to_string())),
    )
    .expect("claw provider client should be constructed");
    let claw_client = match claw_client {
        ProviderClient::ClawApi(client) => {
            ProviderClient::ClawApi(client.with_base_url(claw_harness.base_url()))
        }
        other => panic!("expected ClawApi provider, got {other:?}"),
    };

    let claw_response = claw_client
        .send_message(&sample_request("claude-sonnet-4-6"))
        .await
        .expect("claw request should succeed against local harness");

    let claw_requests = claw_harness.captured_requests().await;
    let claw_request = claw_requests
        .first()
        .expect("local harness should capture claw request");
    assert_eq!(claw_request.path, "/v1/messages");
    assert_eq!(
        claw_request.headers.get("x-api-key").map(String::as_str),
        Some("test-key")
    );

    let _xai_api_key = ScopedEnvVar::set("XAI_API_KEY", "xai-test-key");
    let xai_harness = DeterministicMockHarness::start(vec![MockResponse::json(
        "200 OK",
        "{\"id\":\"chatcmpl_parity\",\"model\":\"grok-3\",\"choices\":[{\"message\":{\"role\":\"assistant\",\"content\":\"parity-ok\",\"tool_calls\":[]},\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":4,\"completion_tokens\":2}}",
    )])
    .await;
    let _xai_base_url = ScopedEnvVar::set("XAI_BASE_URL", xai_harness.base_url());

    let xai_client = ProviderClient::from_model("grok")
        .expect("xAI provider client should be constructed from model alias");
    let xai_response = xai_client
        .send_message(&sample_request("grok-3"))
        .await
        .expect("xAI request should succeed against local harness");

    let xai_requests = xai_harness.captured_requests().await;
    let xai_request = xai_requests
        .first()
        .expect("local harness should capture xAI request");
    assert_eq!(xai_request.path, "/chat/completions");
    assert_eq!(
        xai_request.headers.get("authorization").map(String::as_str),
        Some("Bearer xai-test-key")
    );

    assert_eq!(first_text(&claw_response.content), Some("parity-ok"));
    assert_eq!(first_text(&xai_response.content), Some("parity-ok"));
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MockResponse {
    status: &'static str,
    content_type: &'static str,
    body: &'static str,
}

impl MockResponse {
    fn json(status: &'static str, body: &'static str) -> Self {
        Self {
            status,
            content_type: "application/json",
            body,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CapturedRequest {
    path: String,
    headers: HashMap<String, String>,
}

struct DeterministicMockHarness {
    base_url: String,
    captured: Arc<Mutex<Vec<CapturedRequest>>>,
    join_handle: tokio::task::JoinHandle<()>,
}

impl DeterministicMockHarness {
    async fn start(script: Vec<MockResponse>) -> Self {
        let captured = Arc::new(Mutex::new(Vec::<CapturedRequest>::new()));
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("listener should bind");
        let address = listener.local_addr().expect("listener addr");
        let captured_for_task = Arc::clone(&captured);

        let join_handle = tokio::spawn(async move {
            for response in script {
                let (mut socket, _) = listener.accept().await.expect("accept");
                let mut buffer = Vec::new();
                let mut header_end = None;
                loop {
                    let mut chunk = [0_u8; 1024];
                    let read = socket.read(&mut chunk).await.expect("read request");
                    if read == 0 {
                        break;
                    }
                    buffer.extend_from_slice(&chunk[..read]);
                    if let Some(position) = find_header_end(&buffer) {
                        header_end = Some(position);
                        break;
                    }
                }

                let header_end = header_end.expect("headers should exist");
                let (header_bytes, remaining) = buffer.split_at(header_end);
                let header_text = String::from_utf8(header_bytes.to_vec()).expect("utf8 headers");
                let mut lines = header_text.split("\r\n");
                let request_line = lines.next().expect("request line");
                let path = request_line
                    .split_whitespace()
                    .nth(1)
                    .expect("path")
                    .to_string();
                let mut headers = HashMap::new();
                let mut content_length = 0_usize;
                for line in lines {
                    if line.is_empty() {
                        continue;
                    }
                    let (name, value) = line.split_once(':').expect("header");
                    let value = value.trim().to_string();
                    if name.eq_ignore_ascii_case("content-length") {
                        content_length = value.parse().expect("content length");
                    }
                    headers.insert(name.to_ascii_lowercase(), value);
                }

                let mut body = remaining[4..].to_vec();
                while body.len() < content_length {
                    let mut chunk = vec![0_u8; content_length - body.len()];
                    let read = socket.read(&mut chunk).await.expect("read body");
                    if read == 0 {
                        break;
                    }
                    body.extend_from_slice(&chunk[..read]);
                }

                captured_for_task
                    .lock()
                    .await
                    .push(CapturedRequest { path, headers });

                let response_text = format!(
                    "HTTP/1.1 {}\r\ncontent-type: {}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                    response.status,
                    response.content_type,
                    response.body.len(),
                    response.body
                );
                socket
                    .write_all(response_text.as_bytes())
                    .await
                    .expect("write response");
            }
        });

        Self {
            base_url: format!("http://{address}"),
            captured,
            join_handle,
        }
    }

    fn base_url(&self) -> String {
        self.base_url.clone()
    }

    async fn captured_requests(&self) -> Vec<CapturedRequest> {
        self.captured.lock().await.clone()
    }
}

impl Drop for DeterministicMockHarness {
    fn drop(&mut self) {
        self.join_handle.abort();
    }
}

fn sample_request(model: &str) -> MessageRequest {
    MessageRequest {
        model: model.to_string(),
        max_tokens: 64,
        messages: vec![InputMessage {
            role: "user".to_string(),
            content: vec![InputContentBlock::Text {
                text: "Say parity-ok".to_string(),
            }],
        }],
        system: None,
        tools: None,
        tool_choice: None,
        stream: false,
    }
}

fn find_header_end(bytes: &[u8]) -> Option<usize> {
    bytes.windows(4).position(|window| window == b"\r\n\r\n")
}

fn first_text(blocks: &[OutputContentBlock]) -> Option<&str> {
    blocks.iter().find_map(|block| match block {
        OutputContentBlock::Text { text } => Some(text.as_str()),
        _ => None,
    })
}

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<StdMutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| StdMutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

struct ScopedEnvVar {
    key: &'static str,
    previous: Option<OsString>,
}

impl ScopedEnvVar {
    fn set(key: &'static str, value: impl AsRef<std::ffi::OsStr>) -> Self {
        let previous = std::env::var_os(key);
        std::env::set_var(key, value);
        Self { key, previous }
    }
}

impl Drop for ScopedEnvVar {
    fn drop(&mut self) {
        match &self.previous {
            Some(value) => std::env::set_var(self.key, value),
            None => std::env::remove_var(self.key),
        }
    }
}
