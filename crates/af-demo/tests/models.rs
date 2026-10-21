//! Local protocol fixtures, not evidence of real LLM inference.
use af_demo::{
    Config, End, Event, examples, execute_with_adapter,
    model::{
        AgentSettings, Function, ModelAdapter, ModelError, ModelInfo, OpenAiCompatible, Reply,
        ToolCall, endpoint, parse_reply, run_agent,
    },
    tools,
};
use serde_json::{Value, json};
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};

fn settings() -> AgentSettings {
    AgentSettings {
        model: "fixture-model".into(),
        system_prompt: "Fixture only".into(),
        allowed_tools: tools::NAMES.iter().map(|s| (*s).into()).collect(),
        tool_budget: 8,
        timeout_seconds: 10,
    }
}
fn final_reply(text: &str) -> Reply {
    Reply {
        role: "assistant".into(),
        content: Some(text.into()),
        tool_calls: vec![],
        reasoning_content: None,
    }
}
fn calls(names: &[(&str, &str)]) -> Reply {
    Reply {
        role: "assistant".into(),
        content: None,
        tool_calls: names
            .iter()
            .enumerate()
            .map(|(i, (name, args))| ToolCall {
                id: format!("call_{i}"),
                kind: "function".into(),
                function: Function {
                    name: (*name).into(),
                    arguments: (*args).into(),
                },
            })
            .collect(),
        reasoning_content: None,
    }
}
struct Script {
    replies: Mutex<VecDeque<Reply>>,
    requests: Mutex<Vec<Value>>,
}
impl Script {
    fn new(replies: Vec<Reply>) -> Self {
        Self {
            replies: Mutex::new(replies.into()),
            requests: Mutex::new(vec![]),
        }
    }
}
impl ModelAdapter for Script {
    fn info(&self) -> ModelInfo {
        ModelInfo {
            configured: true,
            default_model: "fixture-model".into(),
            endpoint: "fixture://not-a-real-model".into(),
        }
    }
    fn complete(&self, r: &Value, _: &AtomicBool, _: Instant) -> Result<Reply, ModelError> {
        self.requests.lock().unwrap().push(r.clone());
        self.replies
            .lock()
            .unwrap()
            .pop_front()
            .ok_or_else(|| ModelError::Failed("fixture exhausted".into()))
    }
    fn redact(&self, s: &str) -> String {
        s.replace("fixture-secret", "[REDACTED]")
    }
}
fn scripted(
    s: &Script,
    cfg: &AgentSettings,
    max: u32,
) -> (Result<String, ModelError>, Vec<String>) {
    let mut phases = vec![];
    let result = run_agent(
        s,
        cfg,
        max,
        "one two three",
        &AtomicBool::new(false),
        Instant::now() + Duration::from_secs(20),
        |p, _, _, _| phases.push(p.to_owned()),
    );
    (result, phases)
}

#[test]
fn model_selected_tool_executes_and_result_is_returned_with_correct_id() {
    let s = Script::new(vec![
        calls(&[("text_word_count", r#"{"text":"one two three"}"#)]),
        final_reply("fixture final"),
    ]);
    let (result, phases) = scripted(&s, &settings(), 3);
    assert_eq!(result, Ok("fixture final".into()));
    assert_eq!(
        phases,
        vec![
            "model_request",
            "model_response",
            "tool_requested",
            "tool_started",
            "tool_completed",
            "model_request",
            "model_response",
            "model_final_answer"
        ]
    );
    let requests = s.requests.lock().unwrap();
    let messages = requests[1]["messages"].as_array().unwrap();
    assert_eq!(messages[2]["role"], "assistant");
    assert_eq!(messages[2]["tool_calls"][0]["id"], "call_0");
    assert_eq!(messages[3]["role"], "tool");
    assert_eq!(messages[3]["tool_call_id"], "call_0");
    let result: Value = serde_json::from_str(messages[3]["content"].as_str().unwrap()).unwrap();
    assert_eq!(result["word_count"], 3);
    assert_eq!(requests[0]["tool_choice"], "auto");
}
#[test]
fn model_can_choose_final_without_tools() {
    let s = Script::new(vec![final_reply("direct")]);
    let (r, p) = scripted(&s, &settings(), 3);
    assert_eq!(r, Ok("direct".into()));
    assert!(!p.iter().any(|p| p.starts_with("tool_")));
}
#[test]
fn unknown_and_unallowed_tools_do_not_execute() {
    for name in ["shell", "text_transform"] {
        let mut cfg = settings();
        cfg.allowed_tools = vec!["text.word_count".into()];
        let s = Script::new(vec![calls(&[(name, r#"{"text":"x"}"#)])]);
        let (r, p) = scripted(&s, &cfg, 3);
        assert!(r.is_err());
        assert!(!p.contains(&"tool_started".into()));
    }
}
#[test]
fn invalid_argument_batch_is_atomic() {
    let s = Script::new(vec![calls(&[
        ("text_word_count", r#"{"text":"a b"}"#),
        ("text_transform", r#"{"text":"x","operation":"shell"}"#),
    ])]);
    let (r, p) = scripted(&s, &settings(), 3);
    assert!(r.is_err());
    assert!(!p.contains(&"tool_started".into()));
}
#[test]
fn budget_is_separate_from_request_limit() {
    let mut cfg = settings();
    cfg.tool_budget = 1;
    let s = Script::new(vec![calls(&[
        ("text_word_count", r#"{"text":"a"}"#),
        ("text_word_count", r#"{"text":"b"}"#),
    ])]);
    let (r, p) = scripted(&s, &cfg, 5);
    assert!(matches!(r,Err(ModelError::Failed(s)) if s.contains("budget")));
    assert!(!p.contains(&"tool_started".into()));
}
#[test]
fn duplicate_ids_are_rejected() {
    let mut reply = calls(&[
        ("text_word_count", r#"{"text":"a"}"#),
        ("text_word_count", r#"{"text":"b"}"#),
    ]);
    reply.tool_calls[1].id = reply.tool_calls[0].id.clone();
    let s = Script::new(vec![reply]);
    let (r, p) = scripted(&s, &settings(), 3);
    assert!(r.is_err());
    assert!(!p.contains(&"tool_started".into()));
}
#[test]
fn max_iterations_never_fakes_a_final_answer() {
    let s = Script::new(vec![calls(&[("text_word_count", r#"{"text":"a"}"#)])]);
    let (r, p) = scripted(&s, &settings(), 1);
    assert!(matches!(r,Err(ModelError::Failed(s)) if s.contains("iterations")));
    assert!(!p.contains(&"model_final_answer".into()));
}
#[test]
fn tool_catalogue_uses_valid_wire_names_and_closed_schemas() {
    let definitions = tools::definitions(&settings().allowed_tools);
    assert_eq!(definitions.len(), 3);
    for d in definitions {
        assert!(
            d["function"]["name"]
                .as_str()
                .unwrap()
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
        );
        assert_eq!(d["function"]["parameters"]["additionalProperties"], false);
    }
    assert_eq!(
        tools::definitions(&["text.find".into()])[0]["function"]["name"],
        "text_find"
    );
}
#[test]
fn tools_compute_real_results_and_unicode_positions() {
    assert_eq!(
        tools::execute("text_word_count", &json!({"text":"hello 世界\nthird"})).unwrap()["word_count"],
        3
    );
    assert_eq!(
        tools::execute(
            "text_transform",
            &json!({"text":"ß","operation":"uppercase"})
        )
        .unwrap()["text"],
        "SS"
    );
    let found = tools::execute("text_find", &json!({"text":"猫a猫","query":"猫"})).unwrap();
    assert_eq!(
        found["positions"][1],
        json!({"byte_index":4,"character_index":2})
    );
    assert_eq!(found["match_count"], 2);
}
#[test]
fn invalid_tools_parameters_fail_closed() {
    for (wire, args) in [
        ("shell", json!({})),
        ("text_word_count", json!({"text":3})),
        ("text_word_count", json!({"text":"a","extra":1})),
        ("text_transform", json!({"text":"a","operation":"exec"})),
        ("text_find", json!({"text":"abc","query":""})),
    ] {
        assert!(tools::execute(wire, &args).is_err());
    }
}
#[test]
fn find_positions_and_text_growth_are_bounded() {
    let r = tools::execute("text_find", &json!({"text":"a".repeat(500),"query":"a"})).unwrap();
    assert_eq!(r["positions"].as_array().unwrap().len(), 128);
    assert_eq!(r["positions_truncated"], true);
    assert!(
        tools::execute(
            "text_transform",
            &json!({"text":"ΐ".repeat(5000),"operation":"uppercase"})
        )
        .is_err()
    );
}
#[test]
fn compatible_response_handles_null_tool_calls_and_refuses_truncation() {
    assert_eq!(parse_reply(r#"{"choices":[{"finish_reason":"stop","message":{"role":"assistant","content":"ok","tool_calls":null}}]}"#).unwrap().content.as_deref(),Some("ok"));
    for body in [
        "bad",
        r#"{"choices":[]}"#,
        r#"{"choices":[{"finish_reason":"length","message":{"role":"assistant","content":"partial"}}]}"#,
        r#"{"choices":[{"finish_reason":"stop","message":{"role":"assistant","content":null}}]}"#,
    ] {
        assert!(parse_reply(body).is_err());
    }
}
#[test]
fn base_url_normalization_does_not_enable_arbitrary_protocols() {
    assert_eq!(
        endpoint("https://example.invalid/").unwrap(),
        "https://example.invalid/v1/chat/completions"
    );
    assert_eq!(
        endpoint("https://example.invalid/v1/").unwrap(),
        "https://example.invalid/v1/chat/completions"
    );
    for url in [
        "http://example.invalid",
        "file:///etc/passwd",
        "https://key@host.invalid",
        "https://x.invalid/?api_key=x",
        "https://x.invalid\nheader:bad",
    ] {
        assert!(endpoint(url).is_err());
    }
}
#[test]
fn model_credentials_and_urls_are_rejected_in_workflow_data() {
    let mut d = serde_json::to_value(&examples()[3]).unwrap();
    d["workflow"]["nodes"][1]["config"]["model"]["api_key"] = json!("fixture-secret");
    assert!(serde_json::from_value::<af_demo::Document>(d).is_err());
}
#[test]
fn trace_redaction_happens_before_preview_cutoff() {
    let mut d = examples().remove(3);
    if let Config::AgentLoop { model: Some(m), .. } = &mut d.workflow.nodes[1].config {
        m.model = "fixture-model".into();
    }
    let s = Script::new(vec![final_reply(&("x".repeat(507) + "fixture-secret"))]);
    let mut trace: Vec<Event> = vec![];
    let end = execute_with_adapter(
        &d.validate().unwrap(),
        &AtomicBool::new(false),
        Duration::ZERO,
        &s,
        |e| trace.push(e),
    );
    let raw = serde_json::to_string(&trace).unwrap();
    assert!(!raw.contains("fixture-s"));
    assert!(matches!(end,End::Completed(s) if s.ends_with("[REDACTED]")));
    assert!(trace.iter().any(|e| e.duration_ms.is_some()));
}
#[test]
fn real_mode_without_credentials_does_not_fall_back_to_offline() {
    let d = examples().remove(3);
    let end = af_demo::execute(
        &d.validate().unwrap(),
        &AtomicBool::new(false),
        Duration::ZERO,
        |_| {},
    );
    assert!(matches!(end,End::Failed(s) if s.contains("not configured")));
}

struct Fixture {
    url: String,
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
    requests: Arc<Mutex<Vec<Value>>>,
}
fn read_request(stream: &mut TcpStream) -> Value {
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    let mut data = vec![];
    let end = loop {
        let mut b = [0; 1024];
        let n = stream.read(&mut b).unwrap();
        assert!(n > 0);
        data.extend_from_slice(&b[..n]);
        assert!(data.len() < 150_000);
        if let Some(i) = data.windows(4).position(|w| w == b"\r\n\r\n") {
            break i + 4;
        }
    };
    let header = std::str::from_utf8(&data[..end]).unwrap();
    assert!(header.starts_with("POST /v1/chat/completions HTTP/1.1"));
    assert!(header.contains("Authorization: Bearer fixture-secret"));
    let len: usize = header
        .lines()
        .find_map(|l| {
            l.to_ascii_lowercase()
                .strip_prefix("content-length:")
                .map(|s| s.trim().parse().unwrap())
        })
        .unwrap();
    while data.len() < end + len {
        let mut b = [0; 4096];
        let n = stream.read(&mut b).unwrap();
        assert!(n > 0);
        data.extend_from_slice(&b[..n]);
    }
    serde_json::from_slice(&data[end..end + len]).unwrap()
}
impl Fixture {
    fn new(responses: Vec<(u16, String)>, delay: Duration) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        listener.set_nonblocking(true).unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let requests = Arc::new(Mutex::new(vec![]));
        let captured = Arc::clone(&requests);
        let thread = std::thread::spawn(move || {
            let mut responses: VecDeque<_> = responses.into();
            while !flag.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        stream
                            .set_write_timeout(Some(Duration::from_secs(2)))
                            .unwrap();
                        let request = read_request(&mut stream);
                        captured.lock().unwrap().push(request);
                        let begin = Instant::now();
                        while begin.elapsed() < delay && !flag.load(Ordering::Relaxed) {
                            std::thread::sleep(Duration::from_millis(5));
                        }
                        let (code, body) = responses
                            .pop_front()
                            .unwrap_or((500, "fixture exhausted".into()));
                        let _ = write!(
                            stream,
                            "HTTP/1.1 {code} Fixture\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{body}",
                            body.len()
                        );
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(5));
                    }
                    Err(e) => panic!("fixture: {e}"),
                }
            }
        });
        Self {
            url: format!("http://127.0.0.1:{port}"),
            stop,
            thread: Some(thread),
            requests,
        }
    }
    fn model(&self) -> OpenAiCompatible {
        OpenAiCompatible::new(&self.url, "fixture-secret".into(), "fixture-model".into()).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        self.thread.take().unwrap().join().unwrap();
    }
}
fn envelope(reply: Reply) -> String {
    let finish = if reply.tool_calls.is_empty() {
        "stop"
    } else {
        "tool_calls"
    };
    json!({"choices":[{"finish_reason":finish,"message":serde_json::to_value(reply).unwrap()}]})
        .to_string()
}
#[test]
fn actual_http_transport_round_trip_and_tool_feedback() {
    let fixture = Fixture::new(
        vec![
            (
                200,
                envelope(calls(&[(
                    "text_transform",
                    r#"{"text":"a\n\"b\" 世界","operation":"uppercase"}"#,
                )])),
            ),
            (200, envelope(final_reply("fixture result"))),
        ],
        Duration::ZERO,
    );
    let (r, _) = scripted_transport(&fixture, &settings(), &AtomicBool::new(false));
    assert_eq!(r, Ok("fixture result".into()));
    let requests = fixture.requests.lock().unwrap();
    assert_eq!(requests.len(), 2);
    let result: Value =
        serde_json::from_str(requests[1]["messages"][3]["content"].as_str().unwrap()).unwrap();
    assert_eq!(result["text"], "A\n\"B\" 世界");
    assert!(!requests[0].to_string().contains("fixture-secret"));
}
fn scripted_transport(
    f: &Fixture,
    cfg: &AgentSettings,
    cancel: &AtomicBool,
) -> (Result<String, ModelError>, Vec<String>) {
    let mut phases = vec![];
    let r = run_agent(
        &f.model(),
        cfg,
        3,
        "test",
        cancel,
        Instant::now() + Duration::from_secs(20),
        |p, _, _, _| phases.push(p.into()),
    );
    (r, phases)
}
#[test]
fn http_error_body_never_leaks_credentials() {
    let f = Fixture::new(
        vec![(
            401,
            "fixture-secret Authorization: Bearer fixture-secret".into(),
        )],
        Duration::ZERO,
    );
    let (r, p) = scripted_transport(&f, &settings(), &AtomicBool::new(false));
    assert!(
        matches!(r,Err(ModelError::Failed(s)) if s.contains("401")&&!s.contains("fixture-secret"))
    );
    assert!(!p.contains(&"model_final_answer".into()));
}
#[test]
fn malformed_http_response_and_oversize_fail() {
    for body in ["not json".into(), "x".repeat(200_000)] {
        let f = Fixture::new(vec![(200, body)], Duration::ZERO);
        let (r, _) = scripted_transport(&f, &settings(), &AtomicBool::new(false));
        assert!(r.is_err());
    }
}
#[test]
fn transport_times_out_and_reaps_child() {
    let f = Fixture::new(
        vec![(200, envelope(final_reply("too late")))],
        Duration::from_secs(3),
    );
    let mut cfg = settings();
    cfg.timeout_seconds = 1;
    let start = Instant::now();
    let (r, _) = scripted_transport(&f, &cfg, &AtomicBool::new(false));
    assert_eq!(r, Err(ModelError::Timeout));
    assert!(start.elapsed() < Duration::from_secs(2));
}
#[test]
fn transport_cancellation_interrupts_pending_response() {
    let f = Fixture::new(
        vec![(200, envelope(final_reply("too late")))],
        Duration::from_secs(3),
    );
    let cancel = Arc::new(AtomicBool::new(false));
    let trigger = Arc::clone(&cancel);
    let requests = Arc::clone(&f.requests);
    let thread = std::thread::spawn(move || {
        let start = Instant::now();
        while requests.lock().unwrap().is_empty() {
            assert!(start.elapsed() < Duration::from_secs(3));
            std::thread::sleep(Duration::from_millis(5));
        }
        trigger.store(true, Ordering::Relaxed);
    });
    let start = Instant::now();
    let (r, p) = scripted_transport(&f, &settings(), &cancel);
    thread.join().unwrap();
    assert_eq!(r, Err(ModelError::Cancelled));
    assert!(!p.contains(&"tool_started".into()));
    assert!(start.elapsed() < Duration::from_secs(2));
}
#[test]
fn transport_redacts_decoded_credentials_in_model_text_and_arguments() {
    let body =
        envelope(final_reply("fixture-secret")).replace("fixture-secret", "fixture\\u002dsecret");
    let fixture = Fixture::new(vec![(200, body)], Duration::ZERO);
    let (result, _) = scripted_transport(&fixture, &settings(), &AtomicBool::new(false));
    assert_eq!(result, Ok("[REDACTED]".into()));
    let fixture = Fixture::new(
        vec![
            (
                200,
                envelope(calls(&[(
                    "text_word_count",
                    r#"{"text":"fixture\u002dsecret"}"#,
                )])),
            ),
            (200, envelope(final_reply("fixture done"))),
        ],
        Duration::ZERO,
    );
    let (result, _) = scripted_transport(&fixture, &settings(), &AtomicBool::new(false));
    assert!(result.is_ok());
    let requests = fixture.requests.lock().unwrap();
    let args: Value = serde_json::from_str(
        requests[1]["messages"][2]["tool_calls"][0]["function"]["arguments"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(args["text"], "[REDACTED]");
}
#[test]
fn standalone_task_find_uses_the_same_local_tool() {
    let mut doc = examples().remove(0);
    doc.workflow.nodes[1].config = Config::Task {
        operation: af_demo::Operation::Find,
        argument: "Agent".into(),
    };
    let end = af_demo::execute(
        &doc.validate().unwrap(),
        &AtomicBool::new(false),
        Duration::ZERO,
        |_| {},
    );
    let End::Completed(text) = end else {
        panic!("find failed")
    };
    let value: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(
        value["positions"][0],
        json!({"byte_index":6,"character_index":6})
    );
}
