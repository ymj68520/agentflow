//! OpenAI-compatible non-streaming Chat Completions, using the installed curl
//! TLS transport. Credentials travel over child stdin, never argv or trace.
use crate::{MAX_TEXT, tools};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentSettings {
    pub model: String,
    pub system_prompt: String,
    pub allowed_tools: Vec<String>,
    pub tool_budget: u32,
    pub timeout_seconds: u32,
}
impl AgentSettings {
    pub fn validate(&self) -> Result<(), String> {
        if self.model.len() > 128
            || self.model.chars().any(char::is_control)
            || self.system_prompt.len() > MAX_TEXT
        {
            return Err("invalid model name or system prompt length".into());
        }
        let unique: std::collections::BTreeSet<_> = self.allowed_tools.iter().collect();
        if unique.len() != self.allowed_tools.len()
            || self
                .allowed_tools
                .iter()
                .any(|n| !tools::NAMES.contains(&n.as_str()))
        {
            return Err("unknown or duplicate allowed tool".into());
        }
        if self.tool_budget > 32 || !(1..=180).contains(&self.timeout_seconds) {
            return Err("tool budget must be 0–32 and timeout 1–180 seconds".into());
        }
        Ok(())
    }
}

#[derive(Clone, Serialize)]
pub struct ModelInfo {
    pub configured: bool,
    pub default_model: String,
    pub endpoint: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub arguments: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub function: Function,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Reply {
    pub role: String,
    pub content: Option<String>,
    #[serde(
        default,
        deserialize_with = "null_calls",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub tool_calls: Vec<ToolCall>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
}
#[derive(Debug, PartialEq, Eq)]
pub enum ModelError {
    Cancelled,
    Timeout,
    Failed(String),
}

fn null_calls<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<ToolCall>, D::Error> {
    Ok(Option::<Vec<ToolCall>>::deserialize(deserializer)?.unwrap_or_default())
}

pub trait ModelAdapter: Send + Sync {
    fn info(&self) -> ModelInfo;
    fn complete(
        &self,
        request: &Value,
        cancel: &AtomicBool,
        deadline: Instant,
    ) -> Result<Reply, ModelError>;
    fn redact(&self, text: &str) -> String {
        text.to_owned()
    }
}

pub struct Unconfigured;
impl ModelAdapter for Unconfigured {
    fn info(&self) -> ModelInfo {
        ModelInfo {
            configured: false,
            default_model: String::new(),
            endpoint: String::new(),
        }
    }
    fn complete(&self, _: &Value, _: &AtomicBool, _: Instant) -> Result<Reply, ModelError> {
        Err(ModelError::Failed(
            "real model is not configured; configure the server or choose Offline Deterministic"
                .into(),
        ))
    }
}

// Deliberately neither Debug nor Serialize: the key cannot enter diagnostics.
pub struct OpenAiCompatible {
    endpoint: String,
    api_key: String,
    default_model: String,
}

pub fn endpoint(base: &str) -> Result<String, String> {
    if base.len() > 2048
        || base.chars().any(|c| c.is_whitespace() || c.is_control())
        || base.contains(['?', '#', '\\', '@'])
    {
        return Err("invalid model base URL".into());
    }
    let (scheme, rest) = base.split_once("://").ok_or("base URL requires https://")?;
    let authority = rest.split('/').next().unwrap_or_default();
    if authority.is_empty()
        || !authority
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b".-:[]".contains(&c))
    {
        return Err("invalid model host".into());
    }
    if scheme != "https"
        && !(scheme == "http"
            && (authority == "localhost"
                || authority.starts_with("localhost:")
                || authority == "127.0.0.1"
                || authority.starts_with("127.0.0.1:")
                || authority == "[::1]"
                || authority.starts_with("[::1]:")))
    {
        return Err("HTTPS required (HTTP allowed only for explicit loopback fixtures)".into());
    }
    let trimmed = base.trim_end_matches('/');
    Ok(if trimmed.ends_with("/chat/completions") {
        trimmed.to_owned()
    } else if rest.trim_end_matches('/') == authority {
        format!("{trimmed}/v1/chat/completions")
    } else {
        format!("{trimmed}/chat/completions")
    })
}

impl OpenAiCompatible {
    pub fn new(base: &str, key: String, default_model: String) -> Result<Self, String> {
        if key.is_empty()
            || key.len() > 4096
            || key.chars().any(|c| c.is_control() || c.is_whitespace())
            || default_model.is_empty()
            || default_model.len() > 128
            || default_model.chars().any(char::is_control)
        {
            return Err("invalid server-side model configuration".into());
        }
        let endpoint = endpoint(base)?;
        if endpoint.contains(&key) || default_model.contains(&key) {
            return Err("invalid server-side model configuration".into());
        }
        Ok(Self {
            endpoint,
            api_key: key,
            default_model,
        })
    }
    pub fn from_file(path: &std::path::Path) -> Result<Self, String> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Config {
            base_url: String,
            api_key: String,
            default_model: String,
        }
        let meta = path
            .symlink_metadata()
            .map_err(|_| "model configuration cannot be read")?;
        if !meta.is_file() || meta.len() > 8192 {
            return Err("model configuration must be a small regular file".into());
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if meta.permissions().mode() & 0o077 != 0 {
                return Err("model configuration must have mode 0600".into());
            }
        }
        let bytes = std::fs::read(path).map_err(|_| "model configuration cannot be read")?;
        let c: Config = serde_json::from_slice(&bytes)
            .map_err(|_| "model configuration has invalid fields or JSON")?;
        Self::new(&c.base_url, c.api_key, c.default_model)
    }
}

fn quoted(value: &str) -> String {
    format!(
        "\"{}\"",
        value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    )
}
struct ChildGuard(Child);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

impl ModelAdapter for OpenAiCompatible {
    fn info(&self) -> ModelInfo {
        ModelInfo {
            configured: true,
            default_model: self.default_model.clone(),
            endpoint: self.endpoint.clone(),
        }
    }
    fn redact(&self, text: &str) -> String {
        text.replace(&self.api_key, "[REDACTED]")
    }
    fn complete(
        &self,
        request: &Value,
        cancel: &AtomicBool,
        deadline: Instant,
    ) -> Result<Reply, ModelError> {
        if cancel.load(Ordering::Relaxed) {
            return Err(ModelError::Cancelled);
        }
        let remaining = deadline
            .saturating_duration_since(Instant::now())
            .min(Duration::from_secs(60));
        if remaining.is_zero() {
            return Err(ModelError::Timeout);
        }
        let data = serde_json::to_string(request)
            .map_err(|_| ModelError::Failed("request encoding failed".into()))?;
        if data.len() > 131_072 {
            return Err(ModelError::Failed(
                "model conversation exceeds 128 KiB".into(),
            ));
        }
        let config = format!(
            "url = {}\nheader = {}\nheader = \"Content-Type: application/json\"\ndata-binary = {}\n",
            quoted(&self.endpoint),
            quoted(&format!("Authorization: Bearer {}", self.api_key)),
            quoted(&data)
        );
        let mut child = ChildGuard(
            Command::new("/usr/bin/curl")
                .args([
                    "-q",
                    "--config",
                    "-",
                    "--silent",
                    "--show-error",
                    "--noproxy",
                    "*",
                    "--proto",
                    "=http,https",
                    "--connect-timeout",
                    "10",
                    "--max-time",
                    &format!("{:.3}", remaining.as_secs_f64()),
                    "--max-filesize",
                    "131072",
                    "--write-out",
                    "\nAF_HTTP:%{http_code}",
                ])
                .env_clear()
                .env("PATH", "/usr/bin:/bin")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|_| ModelError::Failed("cannot start installed curl transport".into()))?,
        );
        let stdout = child
            .0
            .stdout
            .take()
            .ok_or_else(|| ModelError::Failed("transport stdout unavailable".into()))?;
        let over = std::sync::Arc::new(AtomicBool::new(false));
        let flag = std::sync::Arc::clone(&over);
        let reader = std::thread::Builder::new()
            .name("bounded-model-response".into())
            .spawn(move || {
                let mut data = Vec::new();
                let result = stdout.take(131_073).read_to_end(&mut data);
                if data.len() > 131_072 {
                    flag.store(true, Ordering::Relaxed);
                }
                result.map(|_| data)
            })
            .map_err(|_| ModelError::Failed("response reader unavailable".into()))?;
        let write_result = child
            .0
            .stdin
            .take()
            .ok_or_else(|| ModelError::Failed("transport stdin unavailable".into()))
            .and_then(|mut input| {
                input
                    .write_all(config.as_bytes())
                    .map_err(|_| ModelError::Failed("transport input failed".into()))
            });
        let local_deadline = Instant::now() + remaining;
        let status = loop {
            if write_result.is_err() {
                break Err(ModelError::Failed("transport input failed".into()));
            }
            if cancel.load(Ordering::Relaxed) {
                break Err(ModelError::Cancelled);
            }
            if Instant::now() >= deadline || Instant::now() >= local_deadline {
                break Err(ModelError::Timeout);
            }
            if over.load(Ordering::Relaxed) {
                break Err(ModelError::Failed("model response exceeds 128 KiB".into()));
            }
            match child.0.try_wait() {
                Ok(Some(status)) => break Ok(status),
                Ok(None) => std::thread::sleep(Duration::from_millis(10)),
                Err(_) => break Err(ModelError::Failed("transport wait failed".into())),
            }
        };
        if status.is_err() {
            let _ = child.0.kill();
            let _ = child.0.wait();
        }
        let bytes = reader
            .join()
            .map_err(|_| ModelError::Failed("response reader failed".into()))?
            .map_err(|_| ModelError::Failed("response read failed".into()))?;
        let status = status?;
        if over.load(Ordering::Relaxed) {
            return Err(ModelError::Failed("model response exceeds 128 KiB".into()));
        }
        if status.code() == Some(28) {
            return Err(ModelError::Timeout);
        }
        if !status.success() {
            return Err(ModelError::Failed(format!(
                "model transport failed (curl exit {})",
                status.code().unwrap_or(-1)
            )));
        }
        let raw = std::str::from_utf8(&bytes)
            .map_err(|_| ModelError::Failed("model response is not UTF-8".into()))?;
        let (body, code) = raw
            .rsplit_once("\nAF_HTTP:")
            .ok_or_else(|| ModelError::Failed("HTTP status unavailable".into()))?;
        if code != "200" {
            return Err(ModelError::Failed(format!(
                "model HTTP {code}; response body suppressed"
            )));
        }
        let mut reply = parse_reply(body)?;
        if let Some(text) = &mut reply.content {
            *text = self.redact(text);
        }
        if let Some(text) = &mut reply.reasoning_content {
            *text = self.redact(text);
        }
        for call in &mut reply.tool_calls {
            if call.id.contains(&self.api_key) || call.function.name.contains(&self.api_key) {
                return Err(ModelError::Failed(
                    "model returned an invalid sensitive tool identity".into(),
                ));
            }
            if let Ok(mut args) = serde_json::from_str::<Value>(&call.function.arguments) {
                scrub_json(&mut args, &|s| self.redact(s));
                call.function.arguments = args.to_string();
            } else {
                call.function.arguments = "[invalid JSON arguments]".into();
            }
        }
        Ok(reply)
    }
}

fn scrub_json(value: &mut Value, redact: &impl Fn(&str) -> String) {
    match value {
        Value::String(s) => *s = redact(s),
        Value::Array(values) => {
            for v in values {
                scrub_json(v, redact);
            }
        }
        Value::Object(values) => {
            let previous = std::mem::take(values);
            for (k, mut v) in previous {
                scrub_json(&mut v, redact);
                values.insert(redact(&k), v);
            }
        }
        _ => {}
    }
}

pub fn parse_reply(body: &str) -> Result<Reply, ModelError> {
    let protocol = || ModelError::Failed("invalid Chat Completions response".into());
    let value: Value = serde_json::from_str(body).map_err(|_| protocol())?;
    let choices = value
        .get("choices")
        .and_then(Value::as_array)
        .filter(|a| a.len() == 1)
        .ok_or_else(protocol)?;
    let finish = choices[0]
        .get("finish_reason")
        .and_then(Value::as_str)
        .ok_or_else(protocol)?;
    if !["stop", "tool_calls"].contains(&finish) {
        return Err(ModelError::Failed(format!(
            "model stopped without a complete answer ({})",
            if finish == "length" {
                "token limit"
            } else {
                "refusal or unsupported finish reason"
            }
        )));
    }
    let reply: Reply =
        serde_json::from_value(choices[0]["message"].clone()).map_err(|_| protocol())?;
    if reply.role != "assistant"
        || reply.tool_calls.len() > 32
        || reply.content.as_ref().is_some_and(|s| s.len() > MAX_TEXT)
        || reply
            .reasoning_content
            .as_ref()
            .is_some_and(|s| s.len() > MAX_TEXT)
        || (reply.tool_calls.is_empty()
            && reply.content.as_ref().is_none_or(|s| s.trim().is_empty()))
    {
        return Err(protocol());
    }
    Ok(reply)
}

/// Each model request consumes one iteration; function calls consume a separate
/// budget. Validate an entire batch before executing any function in that batch.
pub fn run_agent(
    adapter: &dyn ModelAdapter,
    settings: &AgentSettings,
    max: u32,
    input: &str,
    cancel: &AtomicBool,
    workflow_deadline: Instant,
    mut emit: impl FnMut(&str, &str, &str, Option<u128>),
) -> Result<String, ModelError> {
    settings.validate().map_err(ModelError::Failed)?;
    let began = Instant::now();
    let deadline =
        (began + Duration::from_secs(u64::from(settings.timeout_seconds))).min(workflow_deadline);
    let model = if settings.model.is_empty() {
        adapter.info().default_model
    } else {
        settings.model.clone()
    };
    if !adapter.info().configured {
        return Err(ModelError::Failed(
            "real model is not configured; no offline fallback was used".into(),
        ));
    }
    let mut messages = vec![
        json!({"role":"system","content":settings.system_prompt}),
        json!({"role":"user","content":input}),
    ];
    let mut used = 0;
    let mut ids = std::collections::BTreeSet::new();
    for iteration in 1..=max {
        if cancel.load(Ordering::Relaxed) {
            return Err(ModelError::Cancelled);
        }
        if Instant::now() >= deadline {
            return Err(ModelError::Timeout);
        }
        let mut request =
            json!({"model":model,"messages":messages,"stream":false,"max_tokens":2048});
        if !settings.allowed_tools.is_empty() && settings.tool_budget > 0 {
            request["tools"] = json!(tools::definitions(&settings.allowed_tools));
            request["tool_choice"] = json!("auto");
        }
        emit(
            "model_request",
            &format!("Model Request · {model} · iteration {iteration}/{max}"),
            "Server credential attached privately; headers are never traced",
            None,
        );
        let started = Instant::now();
        let reply = adapter.complete(&request, cancel, deadline)?;
        if cancel.load(Ordering::Relaxed) {
            return Err(ModelError::Cancelled);
        }
        if Instant::now() >= deadline {
            return Err(ModelError::Timeout);
        }
        emit(
            "model_response",
            &format!("Model Response · {} tool call(s)", reply.tool_calls.len()),
            reply.content.as_deref().unwrap_or("[tool-call response]"),
            Some(started.elapsed().as_millis()),
        );
        if reply.tool_calls.is_empty() {
            let answer = reply.content.unwrap_or_default();
            emit(
                "model_final_answer",
                "Model Final Answer",
                &answer,
                Some(began.elapsed().as_millis()),
            );
            return Ok(answer);
        }
        for call in &reply.tool_calls {
            emit(
                "tool_requested",
                &format!(
                    "Tool Requested · {}",
                    tools::logical_name(&call.function.name).unwrap_or("UNKNOWN")
                ),
                &call.function.arguments,
                None,
            );
        }
        let count = u32::try_from(reply.tool_calls.len())
            .map_err(|_| ModelError::Failed("too many tool calls".into()))?;
        if count > settings.tool_budget.saturating_sub(used) {
            return Err(ModelError::Failed(
                "tool call budget exceeded; batch was not executed".into(),
            ));
        }
        let mut validated = Vec::new();
        for call in &reply.tool_calls {
            if call.kind != "function"
                || call.id.is_empty()
                || call.id.len() > 256
                || !ids.insert(call.id.clone())
                || call.function.arguments.len() > MAX_TEXT
            {
                return Err(ModelError::Failed(
                    "invalid/duplicate tool call identity or oversized arguments".into(),
                ));
            }
            let logical = tools::logical_name(&call.function.name)
                .ok_or_else(|| ModelError::Failed("unknown tool; no function executed".into()))?;
            if !settings.allowed_tools.iter().any(|n| n == logical) {
                return Err(ModelError::Failed(
                    "tool is not allowed for this Agent; no function executed".into(),
                ));
            }
            let args: Value = serde_json::from_str(&call.function.arguments)
                .map_err(|_| ModelError::Failed(format!("{logical}: invalid argument JSON")))?;
            tools::validate(&call.function.name, &args).map_err(ModelError::Failed)?;
            validated.push(args);
        }
        messages.push(
            serde_json::to_value(&reply)
                .map_err(|_| ModelError::Failed("assistant history encoding failed".into()))?,
        );
        for (call, args) in reply.tool_calls.iter().zip(validated) {
            if cancel.load(Ordering::Relaxed) {
                return Err(ModelError::Cancelled);
            }
            if Instant::now() >= deadline {
                return Err(ModelError::Timeout);
            }
            let name = tools::logical_name(&call.function.name).unwrap_or("UNKNOWN");
            emit(
                "tool_started",
                &format!("Tool Started · {name}"),
                &call.function.arguments,
                None,
            );
            let start = Instant::now();
            let result = tools::execute(&call.function.name, &args).map_err(ModelError::Failed)?;
            let text = result.to_string();
            emit(
                "tool_completed",
                &format!("Tool Completed · {name}"),
                &text,
                Some(start.elapsed().as_millis()),
            );
            used += 1;
            messages.push(json!({"role":"tool","tool_call_id":call.id,"content":text}));
        }
    }
    Err(ModelError::Failed(
        "maximum model iterations reached before a final answer".into(),
    ))
}
