//! Bounded text-only demo interpreter, not Canonical IR or a production runtime.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

pub const MAX_TEXT: usize = 16_384;
pub mod model;
pub mod tools;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Workflow {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub config: Config,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Config {
    Input {
        text: String,
    },
    Task {
        operation: Operation,
        argument: String,
    },
    Condition {
        contains: String,
    },
    AgentLoop {
        max_iterations: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        model: Option<model::AgentSettings>,
    },
    Output,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    Uppercase,
    Lowercase,
    Trim,
    Prefix,
    WordCount,
    Find,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub port: Port,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Port {
    Next,
    True,
    False,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub version: u32,
    pub workflow: Workflow,
    pub layout: BTreeMap<String, Position>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

/// Successfully checked demo workflow, never a formal proof or runtime grant.
#[derive(Clone, Debug)]
pub struct ValidatedWorkflow(Workflow);

fn limit(text: &str) -> Result<(), String> {
    if text.len() > MAX_TEXT {
        Err("text exceeds 16384 UTF-8 bytes".into())
    } else {
        Ok(())
    }
}

impl Document {
    pub fn validate(&self) -> Result<ValidatedWorkflow, String> {
        if self.version != 1 {
            return Err("unsupported demo document version".into());
        }
        if self.layout.len() > 32
            || self.layout.iter().any(|(id, p)| {
                !self.workflow.nodes.iter().any(|n| n.id == *id)
                    || !p.x.is_finite()
                    || !p.y.is_finite()
                    || !(0.0..=4000.0).contains(&p.x)
                    || !(0.0..=3000.0).contains(&p.y)
            })
        {
            return Err("layout must reference existing nodes within the canvas".into());
        }
        self.workflow.validate()
    }
}

impl Workflow {
    pub fn validate(&self) -> Result<ValidatedWorkflow, String> {
        if self.nodes.is_empty() || self.nodes.len() > 32 || self.edges.len() > 64 {
            return Err("use 1–32 nodes and at most 64 edges".into());
        }
        let mut ids = BTreeSet::new();
        let mut total_iterations = 0;
        for n in &self.nodes {
            if n.id.is_empty()
                || n.id.len() > 48
                || !n.id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
                || !ids.insert(n.id.as_str())
            {
                return Err(format!("{}: invalid or duplicate node ID", n.id));
            }
            if n.label.len() > 120 {
                return Err(format!("{}: label too long", n.id));
            }
            match &n.config {
                Config::Input { text } => limit(text),
                Config::Task { argument, .. } => limit(argument),
                Config::Condition { contains } => limit(contains),
                Config::AgentLoop {
                    max_iterations,
                    model,
                } => {
                    if let Some(settings) = model {
                        settings.validate().map_err(|e| format!("{}: {e}", n.id))?;
                    }
                    af_core::primitive::parse_uint32_text(max_iterations)
                        .map_err(|e| e.to_string())
                        .and_then(|v| {
                            if (1..=32).contains(&v) {
                                total_iterations +=
                                    model.as_ref().map_or(v, |m| v.max(m.tool_budget));
                                Ok(())
                            } else {
                                Err("max iterations must be 1–32".into())
                            }
                        })
                }
                Config::Output => Ok(()),
            }
            .map_err(|e| format!("{}: {e}", n.id))?;
        }
        if total_iterations > 128 {
            return Err("combined Agent Loop budget exceeds 128 tool calls".into());
        }
        let inputs: Vec<_> = self
            .nodes
            .iter()
            .filter(|n| matches!(n.config, Config::Input { .. }))
            .collect();
        if inputs.len() != 1
            || self
                .nodes
                .iter()
                .filter(|n| matches!(n.config, Config::Output))
                .count()
                != 1
        {
            return Err("exactly one Input and one Output are required".into());
        }
        let mut pairs = BTreeSet::new();
        for e in &self.edges {
            if !ids.contains(e.from.as_str()) || !ids.contains(e.to.as_str()) || e.from == e.to {
                return Err("edge references missing node or itself".into());
            }
            if !pairs.insert((&e.from, e.port)) {
                return Err(format!("{}: output port already connected", e.from));
            }
        }
        for n in &self.nodes {
            let incoming = self.edges.iter().filter(|e| e.to == n.id).count();
            let ports: BTreeSet<_> = self
                .edges
                .iter()
                .filter(|e| e.from == n.id)
                .map(|e| e.port)
                .collect();
            let expected = match n.config {
                Config::Condition { .. } => BTreeSet::from([Port::True, Port::False]),
                Config::Output => BTreeSet::new(),
                _ => BTreeSet::from([Port::Next]),
            };
            let input_ok = match n.config {
                Config::Input { .. } => incoming == 0,
                Config::Output => incoming > 0,
                _ => incoming == 1,
            };
            if ports != expected || !input_ok {
                return Err(format!(
                    "{}: invalid text port wiring (only Output may merge branches)",
                    n.id
                ));
            }
        }
        let mut degree: BTreeMap<_, _> = self
            .nodes
            .iter()
            .map(|n| {
                (
                    n.id.as_str(),
                    self.edges.iter().filter(|e| e.to == n.id).count(),
                )
            })
            .collect();
        let mut ready = vec![inputs[0].id.as_str()];
        let mut visited = BTreeSet::new();
        while let Some(id) = ready.pop() {
            visited.insert(id);
            for e in self.edges.iter().filter(|e| e.from == id) {
                let d = degree.get_mut(e.to.as_str()).ok_or("missing target")?;
                *d -= 1;
                if *d == 0 {
                    ready.push(&e.to);
                }
            }
        }
        if visited.len() != self.nodes.len() {
            return Err("cycle or unreachable node; use bounded Agent Loop instead".into());
        }
        Ok(ValidatedWorkflow(self.clone()))
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Event {
    pub sequence: usize,
    pub node: String,
    pub phase: String,
    pub message: String,
    pub value: String,
    pub value_truncated: bool,
    pub elapsed_ms: u128,
    pub duration_ms: Option<u128>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum End {
    Completed(String),
    Failed(String),
    Cancelled,
}

fn emit(
    trace: &mut impl FnMut(Event),
    seq: &mut usize,
    node: &Node,
    phase: &str,
    message: &str,
    value: &str,
) {
    *seq += 1;
    trace(Event {
        sequence: *seq,
        node: node.id.clone(),
        phase: phase.into(),
        message: message.into(),
        value_truncated: false,
        value: value.to_owned(),
        elapsed_ms: 0,
        duration_ms: None,
    });
}

fn pause(cancel: &AtomicBool, delay: Duration) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed() < delay {
        if cancel.load(Ordering::Relaxed) {
            return false;
        }
        std::thread::sleep(Duration::from_millis(5).min(delay.saturating_sub(start.elapsed())));
    }
    !cancel.load(Ordering::Relaxed)
}

/// Executes one text path; AgentLoop owns a bounded deterministic tool loop.
pub fn execute(
    validated: &ValidatedWorkflow,
    cancel: &AtomicBool,
    delay: Duration,
    trace: impl FnMut(Event),
) -> End {
    execute_with_adapter(validated, cancel, delay, &model::Unconfigured, trace)
}

pub fn execute_with_adapter(
    validated: &ValidatedWorkflow,
    cancel: &AtomicBool,
    delay: Duration,
    adapter: &dyn model::ModelAdapter,
    mut trace: impl FnMut(Event),
) -> End {
    let began = std::time::Instant::now();
    let end = execute_inner(validated, cancel, delay, adapter, began, |mut event| {
        event.elapsed_ms = began.elapsed().as_millis();
        event.message = adapter.redact(&event.message);
        let clean = adapter.redact(&event.value);
        let preview: String = clean.chars().take(512).collect();
        event.value_truncated = preview.len() < clean.len();
        event.value = preview;
        trace(event);
    });
    match end {
        End::Completed(s) => End::Completed(adapter.redact(&s)),
        End::Failed(s) => End::Failed(adapter.redact(&s)),
        End::Cancelled => End::Cancelled,
    }
}

fn execute_inner(
    validated: &ValidatedWorkflow,
    cancel: &AtomicBool,
    delay: Duration,
    adapter: &dyn model::ModelAdapter,
    began: std::time::Instant,
    mut trace: impl FnMut(Event),
) -> End {
    let w = &validated.0;
    let mut current = w
        .nodes
        .iter()
        .find(|n| matches!(n.config, Config::Input { .. }))
        .expect("validated input");
    let mut value = String::new();
    let mut seq = 0;
    loop {
        if !pause(cancel, delay) {
            return End::Cancelled;
        }
        if began.elapsed() > Duration::from_secs(300) {
            return fail(
                &mut trace,
                &mut seq,
                current,
                "workflow exceeded 300 second timeout",
                "",
            );
        }
        emit(
            &mut trace,
            &mut seq,
            current,
            "start",
            "Text input received",
            &value,
        );
        let mut port = Port::Next;
        match &current.config {
            Config::Input { text } => value.clone_from(text),
            Config::Task {
                operation,
                argument,
            } => match run_task(*operation, argument, &value) {
                Ok(result) => value = result,
                Err(e) => return fail(&mut trace, &mut seq, current, &e, ""),
            },
            Config::Condition { contains } => {
                port = if value.contains(contains) {
                    Port::True
                } else {
                    Port::False
                };
                emit(
                    &mut trace,
                    &mut seq,
                    current,
                    "branch",
                    &format!("Selected {port:?} branch (case-sensitive contains)"),
                    &value,
                );
            }
            Config::AgentLoop {
                max_iterations,
                model: Some(settings),
            } => {
                let max = max_iterations.parse().expect("validated iteration limit");
                let result = model::run_agent(
                    adapter,
                    settings,
                    max,
                    &value,
                    cancel,
                    began + Duration::from_secs(300),
                    |phase, message, value, duration| {
                        emit(
                            &mut |mut event: Event| {
                                event.duration_ms = duration;
                                trace(event);
                            },
                            &mut seq,
                            current,
                            phase,
                            message,
                            value,
                        );
                    },
                );
                match result {
                    Ok(answer) => value = answer,
                    Err(model::ModelError::Cancelled) => {
                        emit(
                            &mut trace,
                            &mut seq,
                            current,
                            "cancelled",
                            "Model request cancelled; transport was reaped",
                            "",
                        );
                        return End::Cancelled;
                    }
                    Err(model::ModelError::Timeout) => {
                        return fail(&mut trace, &mut seq, current, "model/Agent timeout", "");
                    }
                    Err(model::ModelError::Failed(e)) => {
                        return fail(&mut trace, &mut seq, current, &e, "");
                    }
                }
            }
            Config::AgentLoop {
                max_iterations,
                model: None,
            } => {
                let max: u32 = max_iterations.parse().expect("validated bound");
                let mut done = false;
                for iteration in 0..=max {
                    if cancel.load(Ordering::Relaxed) {
                        return End::Cancelled;
                    }
                    let next = value
                        .char_indices()
                        .find(|(_, c)| c.to_uppercase().to_string() != c.to_string());
                    emit(
                        &mut trace,
                        &mut seq,
                        current,
                        "decision",
                        &format!(
                            "Offline deterministic policy · iteration {iteration}/{max}: {}",
                            if next.is_none() {
                                "goal reached; stop"
                            } else {
                                "uppercase next character"
                            }
                        ),
                        &value,
                    );
                    let Some((offset, character)) = next else {
                        done = true;
                        break;
                    };
                    if iteration == max {
                        break;
                    }
                    if !pause(cancel, delay) {
                        return End::Cancelled;
                    }
                    emit(
                        &mut trace,
                        &mut seq,
                        current,
                        "tool_call",
                        "Local tool: uppercase one Unicode character",
                        &character.to_string(),
                    );
                    let result = tools::execute(
                        "text_transform",
                        &serde_json::json!({"text":character.to_string(),"operation":"uppercase"}),
                    )
                    .expect("bounded character tool");
                    value.replace_range(
                        offset..offset + character.len_utf8(),
                        result["text"].as_str().expect("transform returns text"),
                    );
                    if let Err(e) = limit(&value) {
                        return fail(&mut trace, &mut seq, current, &e, "");
                    }
                    emit(
                        &mut trace,
                        &mut seq,
                        current,
                        "tool_result",
                        "Tool result passed back to offline policy",
                        &value,
                    );
                }
                if !done {
                    return fail(
                        &mut trace,
                        &mut seq,
                        current,
                        "maximum iterations reached before goal",
                        &value,
                    );
                }
            }
            Config::Output => {
                emit(
                    &mut trace,
                    &mut seq,
                    current,
                    "complete",
                    "Final text output",
                    &value,
                );
                return End::Completed(value);
            }
        }
        if let Err(e) = limit(&value) {
            return fail(&mut trace, &mut seq, current, &e, "");
        }
        emit(
            &mut trace,
            &mut seq,
            current,
            "complete",
            "Node completed",
            &value,
        );
        let edge = w
            .edges
            .iter()
            .find(|e| e.from == current.id && e.port == port)
            .expect("validated port");
        current = w
            .nodes
            .iter()
            .find(|n| n.id == edge.to)
            .expect("validated target");
    }
}

fn fail(trace: &mut impl FnMut(Event), seq: &mut usize, n: &Node, error: &str, value: &str) -> End {
    emit(trace, seq, n, "error", error, value);
    End::Failed(format!("{}: {error}", n.id))
}

fn run_task(operation: Operation, argument: &str, value: &str) -> Result<String, String> {
    use serde_json::json;
    match operation {
        Operation::Prefix => Ok(format!("{argument}{value}")),
        Operation::WordCount => Ok(
            tools::execute("text_word_count", &json!({"text":value}))?["word_count"].to_string(),
        ),
        Operation::Find => {
            Ok(tools::execute("text_find", &json!({"text":value,"query":argument}))?.to_string())
        }
        Operation::Uppercase | Operation::Lowercase | Operation::Trim => {
            let op = match operation {
                Operation::Uppercase => "uppercase",
                Operation::Lowercase => "lowercase",
                _ => "trim",
            };
            Ok(
                tools::execute("text_transform", &json!({"text":value,"operation":op}))?["text"]
                    .as_str()
                    .ok_or("transform did not return text")?
                    .to_owned(),
            )
        }
    }
}

pub fn examples() -> Vec<Document> {
    let input = || Node {
        id: "input".into(),
        label: "Input".into(),
        config: Config::Input {
            text: "Hello AgentFlow".into(),
        },
    };
    let task = |id: &str, label: &str, operation, argument: &str| Node {
        id: id.into(),
        label: label.into(),
        config: Config::Task {
            operation,
            argument: argument.into(),
        },
    };
    let output = || Node {
        id: "output".into(),
        label: "Output".into(),
        config: Config::Output,
    };
    let edge = |from: &str, to: &str, port| Edge {
        from: from.into(),
        to: to.into(),
        port,
    };
    let doc = |nodes: Vec<Node>, edges, positions: Vec<(f64, f64)>| Document {
        version: 1,
        layout: nodes
            .iter()
            .zip(positions)
            .map(|(n, (x, y))| (n.id.clone(), Position { x, y }))
            .collect(),
        workflow: Workflow { nodes, edges },
    };
    let mut documents = vec![
        doc(
            vec![
                input(),
                task("text", "Uppercase text", Operation::Uppercase, ""),
                output(),
            ],
            vec![
                edge("input", "text", Port::Next),
                edge("text", "output", Port::Next),
            ],
            vec![(65.0, 130.0), (355.0, 130.0), (645.0, 130.0)],
        ),
        doc(
            vec![
                input(),
                Node {
                    id: "condition".into(),
                    label: "Contains Agent?".into(),
                    config: Config::Condition {
                        contains: "Agent".into(),
                    },
                },
                task("yes", "Task A", Operation::Prefix, "Matched: "),
                task("no", "Task B", Operation::Prefix, "Not matched: "),
                output(),
            ],
            vec![
                edge("input", "condition", Port::Next),
                edge("condition", "yes", Port::True),
                edge("condition", "no", Port::False),
                edge("yes", "output", Port::Next),
                edge("no", "output", Port::Next),
            ],
            vec![
                (35.0, 175.0),
                (285.0, 175.0),
                (545.0, 55.0),
                (545.0, 295.0),
                (815.0, 175.0),
            ],
        ),
        doc(
            vec![
                Node {
                    id: "input".into(),
                    label: "Goal text".into(),
                    config: Config::Input {
                        text: "agent".into(),
                    },
                },
                Node {
                    id: "agent".into(),
                    label: "Offline Agent Loop".into(),
                    config: Config::AgentLoop {
                        max_iterations: "8".into(),
                        model: None,
                    },
                },
                output(),
            ],
            vec![
                edge("input", "agent", Port::Next),
                edge("agent", "output", Port::Next),
            ],
            vec![(65.0, 130.0), (355.0, 130.0), (645.0, 130.0)],
        ),
    ];
    for (text, label, allowed) in [
        (
            "请用简短中文解释什么是 Agent 工作流。",
            "Real LLM · 问答",
            Vec::new(),
        ),
        (
            "请实际使用工具，统计文本「AgentFlow makes tools useful」的单词数，并将同一文本转为大写。请根据真实工具结果用中文回答。",
            "Real LLM · 自主工具",
            tools::NAMES.iter().map(|n| (*n).to_owned()).collect(),
        ),
    ] {
        let budget = if allowed.is_empty() { 0 } else { 8 };
        documents.push(doc(vec![Node{id:"input".into(),label:"User Input".into(),config:Config::Input{text:text.into()}},Node{id:"agent".into(),label:label.into(),config:Config::AgentLoop{max_iterations:"6".into(),model:Some(model::AgentSettings{model:String::new(),system_prompt:"You are a helpful assistant. Use the provided local tools for requested exact text operations. Never pretend to execute tools. Use their actual results in your final answer. Answer in Chinese.".into(),allowed_tools:allowed,tool_budget:budget,timeout_seconds:120})}},output()],vec![edge("input","agent",Port::Next),edge("agent","output",Port::Next)],vec![(65.0,130.0),(355.0,130.0),(645.0,130.0)]));
    }
    documents
}
