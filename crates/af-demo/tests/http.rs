//! Real HTTP/binary tests. Children inherit the qualified test cgroup limits.
use serde_json::{Value, json};
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

struct Server {
    child: Child,
    port: u16,
}
impl Server {
    fn new() -> Self {
        let socket = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = socket.local_addr().unwrap().port();
        drop(socket);
        let child = Command::new(env!("CARGO_BIN_EXE_af-demo"))
            .args(["--port", &port.to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut server = Self { child, port };
        let start = Instant::now();
        while TcpStream::connect(("127.0.0.1", port)).is_err() {
            assert!(server.child.try_wait().unwrap().is_none());
            assert!(start.elapsed() < Duration::from_secs(5));
            std::thread::sleep(Duration::from_millis(10));
        }
        server
    }
    fn raw(&self, data: &str) -> (u16, String) {
        let mut stream = TcpStream::connect(("127.0.0.1", self.port)).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        stream.write_all(data.as_bytes()).unwrap();
        stream.shutdown(Shutdown::Write).unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        let (headers, body) = response.split_once("\r\n\r\n").expect("HTTP response");
        (
            headers.split_whitespace().nth(1).unwrap().parse().unwrap(),
            body.to_owned(),
        )
    }
    fn api(&self, method: &str, path: &str, body: Option<&Value>) -> (u16, Value) {
        let data = body.map_or_else(String::new, Value::to_string);
        let request = format!(
            "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nX-AgentFlow-Demo: 1\r\nContent-Length: {}\r\n\r\n{data}",
            self.port,
            data.len()
        );
        let (code, text) = self.raw(&request);
        (code, serde_json::from_str(&text).unwrap())
    }
    fn finish(&self, id: u64) -> Value {
        let start = Instant::now();
        loop {
            let (code, run) = self.api("GET", &format!("/api/run?id={id}"), None);
            assert_eq!(code, 200);
            if run["status"] != "running" {
                return run;
            }
            assert!(start.elapsed() < Duration::from_secs(10));
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn real_binary_executes_all_examples_and_both_branches() {
    let server = Server::new();
    let (code, examples) = server.api("GET", "/api/examples", None);
    assert_eq!(code, 200);
    for (index, expected) in [
        (0, "HELLO AGENTFLOW"),
        (1, "Matched: Hello AgentFlow"),
        (2, "AGENT"),
    ] {
        let (code, response) = server.api("POST", "/api/run", Some(&examples[index]));
        assert_eq!(code, 202);
        let run = server.finish(response["id"].as_u64().unwrap());
        assert_eq!(run["status"], "completed");
        assert_eq!(run["output"], expected);
    }
    let mut condition = examples[1].clone();
    condition["workflow"]["nodes"][0]["config"]["text"] = json!("offline");
    let (_, response) = server.api("POST", "/api/run", Some(&condition));
    let run = server.finish(response["id"].as_u64().unwrap());
    assert_eq!(run["output"], "Not matched: offline");
    assert!(
        !run["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["node"] == "yes")
    );
}

#[test]
fn transport_rejects_cross_origin_oversized_and_unknown_requests() {
    let s = Server::new();
    let h = format!("Host: 127.0.0.1:{}\r\n", s.port);
    assert_eq!(
        s.raw(&format!(
            "POST /api/run HTTP/1.1\r\n{h}Origin: https://evil.invalid\r\nContent-Length: 0\r\n\r\n"
        ))
        .0,
        400
    );
    assert_eq!(s.raw("GET / HTTP/1.1\r\nHost: evil.invalid\r\n\r\n").0, 400);
    assert_eq!(
        s.raw(&format!(
            "POST /api/run HTTP/1.1\r\n{h}Content-Length: 131073\r\n\r\n"
        ))
        .0,
        400
    );
    assert_eq!(
        s.raw(&format!(
            "POST /api/run HTTP/1.1\r\n{h}Content-Length: 0\r\n\r\n"
        ))
        .0,
        400
    );
    assert_eq!(
        s.raw(&format!("GET /../../Cargo.lock HTTP/1.1\r\n{h}\r\n"))
            .0,
        404
    );
    let (_, page) = s.raw(&format!("GET / HTTP/1.1\r\n{h}\r\n"));
    assert!(page.contains("非生产原型"));
    assert!(!page.contains("https://"));
}

#[test]
fn active_run_is_exclusive_and_cancel_is_acknowledged() {
    let s = Server::new();
    let example = json!(af_demo::examples()[2]);
    let (_, a) = s.api("POST", "/api/run", Some(&example));
    let id = a["id"].as_u64().unwrap();
    assert_eq!(s.api("POST", "/api/run", Some(&example)).0, 422);
    assert_eq!(
        s.api("POST", "/api/cancel", Some(&json!({"id":id+1}))).0,
        409
    );
    assert_eq!(s.api("POST", "/api/cancel", Some(&json!({"id":id}))).0, 200);
    let run = s.finish(id);
    assert_eq!(run["status"], "cancelled");
    assert!(run["output"].is_null());
}

#[test]
fn invalid_graph_and_json_fail_closed() {
    let s = Server::new();
    let mut example = json!(af_demo::examples()[0]);
    example["workflow"]["edges"][0]["port"] = json!("true");
    assert_eq!(s.api("POST", "/api/validate", Some(&example)).0, 422);
    assert_eq!(s.api("POST", "/api/run", Some(&example)).0, 422);
    assert_eq!(s.raw(&format!("POST /api/cancel HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nX-AgentFlow-Demo: 1\r\nContent-Type: application/json\r\nContent-Length: 1\r\n\r\n{{",s.port)).0,422);
}
#[test]
fn idle_preconnection_is_closed_without_a_spurious_http_error() {
    let server = Server::new();
    let mut idle = TcpStream::connect(("127.0.0.1", server.port)).unwrap();
    idle.set_read_timeout(Some(Duration::from_secs(4))).unwrap();
    std::thread::sleep(Duration::from_millis(2200));
    let mut byte = [0];
    assert_eq!(
        idle.read(&mut byte).unwrap(),
        0,
        "idle connection must not receive an HTTP error body"
    );
    let (code, _) = server.api("GET", "/api/examples", None);
    assert_eq!(code, 200);
}
