use af_demo::{Config, End, Port, examples, execute};
use std::sync::atomic::AtomicBool;
use std::time::Duration;

fn run(index: usize, text: Option<&str>) -> (End, Vec<af_demo::Event>) {
    let mut doc = examples().remove(index);
    if let Some(text) = text {
        doc.workflow.nodes[0].config = Config::Input { text: text.into() };
    }
    let mut trace = Vec::new();
    let result = execute(
        &doc.validate().unwrap(),
        &AtomicBool::new(false),
        Duration::ZERO,
        |e| trace.push(e),
    );
    (result, trace)
}

#[test]
fn sequential_text_is_real() {
    let (end, trace) = run(0, None);
    assert_eq!(end, End::Completed("HELLO AGENTFLOW".into()));
    assert_eq!(trace.last().unwrap().value, "HELLO AGENTFLOW");
}
#[test]
fn branches_are_exclusive() {
    for (text, expected, selected, skipped) in [
        ("Agent", "Matched: Agent", "yes", "no"),
        ("hello", "Not matched: hello", "no", "yes"),
    ] {
        let (end, trace) = run(1, Some(text));
        assert_eq!(end, End::Completed(expected.into()));
        assert!(trace.iter().any(|e| e.node == selected));
        assert!(!trace.iter().any(|e| e.node == skipped));
    }
}
#[test]
fn loop_uses_tool_feedback_and_stops() {
    let (end, trace) = run(2, None);
    assert_eq!(end, End::Completed("AGENT".into()));
    assert_eq!(trace.iter().filter(|e| e.phase == "tool_call").count(), 5);
    assert_eq!(trace.iter().filter(|e| e.phase == "tool_result").count(), 5);
    assert_eq!(trace.iter().filter(|e| e.phase == "decision").count(), 6);
}
#[test]
fn loop_bound_is_enforced() {
    let mut doc = examples().remove(2);
    doc.workflow.nodes[1].config = Config::AgentLoop {
        max_iterations: "1".into(),
        model: None,
    };
    let mut trace = Vec::new();
    assert!(matches!(
        execute(
            &doc.validate().unwrap(),
            &AtomicBool::new(false),
            Duration::ZERO,
            |e| trace.push(e)
        ),
        End::Failed(_)
    ));
    assert_eq!(trace.iter().filter(|e| e.phase == "tool_call").count(), 1);
    assert_eq!(trace.last().unwrap().phase, "error");
}
#[test]
fn pre_cancel_does_no_work() {
    let doc = examples().remove(2);
    let end = execute(
        &doc.validate().unwrap(),
        &AtomicBool::new(true),
        Duration::ZERO,
        |_| panic!("must not execute"),
    );
    assert_eq!(end, End::Cancelled);
}
#[test]
fn cancellation_interrupts_loop() {
    let doc = examples().remove(2);
    let cancel = AtomicBool::new(false);
    let end = execute(&doc.validate().unwrap(), &cancel, Duration::ZERO, |e| {
        if e.phase == "tool_result" {
            cancel.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    });
    assert_eq!(end, End::Cancelled);
}
#[test]
fn document_round_trip_preserves_semantics_and_layout() {
    for d in examples() {
        let data = serde_json::to_string(&d).unwrap();
        let restored: af_demo::Document = serde_json::from_str(&data).unwrap();
        restored.validate().unwrap();
        assert_eq!(data, serde_json::to_string(&restored).unwrap());
    }
}
#[test]
fn reject_wrong_port() {
    let mut d = examples().remove(0);
    d.workflow.edges[0].port = Port::True;
    assert!(d.validate().is_err());
}
#[test]
fn reject_duplicate_port() {
    let mut d = examples().remove(0);
    d.workflow.edges.push(d.workflow.edges[0].clone());
    assert!(d.validate().is_err());
}
#[test]
fn reject_back_edge_cycle() {
    let mut d = examples().remove(0);
    d.workflow.edges[1].to = "input".into();
    assert!(d.validate().is_err());
}
#[test]
fn reject_dangling_node() {
    let mut d = examples().remove(0);
    let mut n = d.workflow.nodes[1].clone();
    n.id = "orphan".into();
    d.workflow.nodes.push(n);
    assert!(d.validate().is_err());
}
#[test]
fn reject_missing_branch() {
    let mut d = examples().remove(1);
    d.workflow.edges.remove(2);
    assert!(d.validate().is_err());
}
#[test]
fn reject_invalid_bound() {
    for bound in ["0", "33", "01", "-1", "999999999999"] {
        let mut d = examples().remove(2);
        d.workflow.nodes[1].config = Config::AgentLoop {
            max_iterations: bound.into(),
            model: None,
        };
        assert!(d.validate().is_err());
    }
}
#[test]
fn reject_unknown_fields_and_tool() {
    let data = serde_json::to_string(&examples()[0]).unwrap();
    assert!(
        serde_json::from_str::<af_demo::Document>(&data.replace("uppercase", "shell")).is_err()
    );
    assert!(
        serde_json::from_str::<af_demo::Document>(&data.replacen(
            "\"version\":1",
            "\"secret\":true,\"version\":1",
            1
        ))
        .is_err()
    );
}
#[test]
fn bounded_unicode_expansion() {
    let (end, _) = run(0, Some("ß中国"));
    assert_eq!(end, End::Completed("SS中国".into()));
    let (end, _) = run(0, Some(&"ΐ".repeat(5000)));
    assert!(matches!(end, End::Failed(_)));
}
#[test]
fn trace_sequences_are_contiguous() {
    let (_, trace) = run(2, None);
    assert!(trace.iter().enumerate().all(|(i, e)| e.sequence == i + 1));
}
#[test]
fn reject_out_of_canvas_layout() {
    let mut d = examples().remove(0);
    d.layout.get_mut("input").unwrap().x = -1.0;
    assert!(d.validate().is_err());
}
#[test]
fn long_trace_is_explicit_preview_not_truncated_output() {
    let text = "a".repeat(1000);
    let (end, trace) = run(0, Some(&text));
    assert_eq!(end, End::Completed(text.to_uppercase()));
    assert!(trace.iter().any(|e| e.value_truncated));
    assert!(trace.iter().all(|e| e.value.chars().count() <= 512));
}
#[test]
fn combined_loop_budget_is_bounded() {
    let mut d = examples().remove(0);
    for i in 0..5 {
        let mut n = d.workflow.nodes[1].clone();
        n.id = format!("loop{i}");
        n.config = Config::AgentLoop {
            max_iterations: "32".into(),
            model: None,
        };
        d.workflow.nodes.push(n);
    }
    assert!(d.validate().unwrap_err().contains("128"));
}
#[test]
fn shipped_json_examples_match_builtin_workflows() {
    for (raw, expected) in [
        include_str!("../examples/sequential.json"),
        include_str!("../examples/condition.json"),
        include_str!("../examples/agent-loop.json"),
        include_str!("../examples/real-qa.json"),
        include_str!("../examples/real-tools.json"),
    ]
    .into_iter()
    .zip(examples())
    {
        let doc: af_demo::Document = serde_json::from_str(raw).unwrap();
        doc.validate().unwrap();
        assert_eq!(
            serde_json::to_value(doc).unwrap(),
            serde_json::to_value(expected).unwrap()
        );
    }
}
