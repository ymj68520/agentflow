//! Loopback-only HTTP transport for the non-production demo.
use af_demo::{Document, End, Event};
use serde::Serialize;
use serde_json::{Value, json};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Clone, Serialize)]
struct Run {
    id: u64,
    status: String,
    events: Vec<Event>,
    output: Option<String>,
    error: Option<String>,
}
struct State {
    next: u64,
    run: Option<Run>,
    cancel: Arc<AtomicBool>,
    model: Arc<dyn af_demo::model::ModelAdapter>,
}

fn respond(stream: &mut TcpStream, code: u16, mime: &str, body: &[u8]) -> std::io::Result<()> {
    write!(
        stream,
        "HTTP/1.1 {code} Response\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nContent-Security-Policy: default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'none'\r\n\r\n",
        body.len()
    )?;
    stream.write_all(body)
}

fn json_response(stream: &mut TcpStream, code: u16, value: &Value) -> std::io::Result<()> {
    respond(
        stream,
        code,
        "application/json; charset=utf-8",
        &serde_json::to_vec(value)?,
    )
}

struct Request {
    method: String,
    path: String,
    body: Vec<u8>,
}

fn request(stream: &mut TcpStream, port: u16) -> Result<Request, String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    let header_end = loop {
        if let Some(i) = bytes.windows(4).position(|s| s == b"\r\n\r\n") {
            break i + 4;
        }
        if bytes.len() >= 16_384 {
            return Err("headers too large".into());
        }
        let mut chunk = [0_u8; 1024];
        let count = match stream.read(&mut chunk) {
            Ok(0) if bytes.is_empty() => return Err("EMPTY_IDLE_CONNECTION".into()),
            Err(error)
                if bytes.is_empty()
                    && matches!(
                        error.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
            {
                return Err("EMPTY_IDLE_CONNECTION".into());
            }
            Ok(count) => count,
            Err(error) => return Err(format!("HTTP header read failed: {error}")),
        };
        if count == 0 {
            return Err("incomplete headers".into());
        }
        bytes.extend_from_slice(&chunk[..count]);
    };
    let header = std::str::from_utf8(&bytes[..header_end]).map_err(|e| e.to_string())?;
    let mut lines = header.split("\r\n");
    let parts: Vec<_> = lines
        .next()
        .unwrap_or_default()
        .split_whitespace()
        .collect();
    if parts.len() != 3 || parts[2] != "HTTP/1.1" {
        return Err("invalid HTTP request".into());
    }
    let method = parts[0].to_owned();
    let path = parts[1].to_owned();
    let mut headers = std::collections::BTreeMap::new();
    for line in lines.filter(|s| !s.is_empty()) {
        let (key, value) = line.split_once(':').ok_or("malformed header")?;
        if headers
            .insert(key.to_ascii_lowercase(), value.trim())
            .is_some()
        {
            return Err("duplicate header".into());
        }
    }
    let host = headers.get("host").copied().unwrap_or_default();
    if host != format!("127.0.0.1:{port}") && host != format!("localhost:{port}") {
        return Err("loopback Host required".into());
    }
    if let Some(origin) = headers.get("origin") {
        if *origin != format!("http://{host}") {
            return Err("cross-origin request rejected".into());
        }
    }
    if headers.contains_key("transfer-encoding") {
        return Err("chunked requests unsupported".into());
    }
    let length: usize = headers
        .get("content-length")
        .unwrap_or(&"0")
        .parse()
        .map_err(|_| "invalid length")?;
    if length > 131_072 {
        return Err("request body exceeds 128 KiB".into());
    }
    if method == "POST"
        && (headers.get("x-agentflow-demo") != Some(&"1")
            || !headers
                .get("content-type")
                .is_some_and(|s| s.starts_with("application/json")))
    {
        return Err("JSON and X-AgentFlow-Demo: 1 required".into());
    }
    while bytes.len() < header_end + length {
        let mut chunk = [0_u8; 4096];
        let count = stream.read(&mut chunk).map_err(|e| e.to_string())?;
        if count == 0 {
            return Err("incomplete body".into());
        }
        bytes.extend_from_slice(&chunk[..count]);
    }
    Ok(Request {
        method,
        path,
        body: bytes[header_end..header_end + length].to_vec(),
    })
}

fn start(state: &Arc<Mutex<State>>, doc: &Document) -> Result<u64, String> {
    let validated = doc.validate()?;
    let mut locked = state.lock().map_err(|e| e.to_string())?;
    if locked.run.as_ref().is_some_and(|r| r.status == "running") {
        return Err("another demo run is active; cancel or wait".into());
    }
    let id = locked.next;
    locked.next += 1;
    locked.cancel = Arc::new(AtomicBool::new(false));
    let cancel = Arc::clone(&locked.cancel);
    locked.run = Some(Run {
        id,
        status: "running".into(),
        events: Vec::new(),
        output: None,
        error: None,
    });
    let shared = Arc::clone(state);
    let model = Arc::clone(&locked.model);
    // One active worker only; bounded nodes/text/iterations bound retained trace.
    std::thread::Builder::new()
        .name("demo-workflow".into())
        .spawn(move || {
            let result = af_demo::execute_with_adapter(
                &validated,
                &cancel,
                Duration::from_millis(180),
                model.as_ref(),
                |event| {
                    if let Ok(mut s) = shared.lock() {
                        if let Some(r) = &mut s.run {
                            r.events.push(event);
                        }
                    }
                },
            );
            if let Ok(mut s) = shared.lock() {
                if let Some(r) = &mut s.run {
                    match result {
                        End::Completed(value) => {
                            r.status = "completed".into();
                            r.output = Some(value);
                        }
                        End::Failed(e) => {
                            r.status = "failed".into();
                            r.error = Some(e);
                        }
                        End::Cancelled => {
                            r.status = "cancelled".into();
                            r.error = Some("Cancelled by user".into());
                        }
                    }
                }
            }
        })
        .map_err(|e| {
            locked.run = None;
            e.to_string()
        })?;
    Ok(id)
}

fn handle(stream: &mut TcpStream, port: u16, state: &Arc<Mutex<State>>) -> std::io::Result<()> {
    let req = match request(stream, port) {
        Ok(r) => r,
        Err(error) if error == "EMPTY_IDLE_CONNECTION" => return Ok(()),
        Err(e) => return json_response(stream, 400, &json!({"error":e})),
    };
    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/") => respond(
            stream,
            200,
            "text/html; charset=utf-8",
            include_bytes!("../web/index.html"),
        ),
        ("GET", "/app.css") => respond(
            stream,
            200,
            "text/css; charset=utf-8",
            include_bytes!("../web/app.css"),
        ),
        ("GET", "/app.js") => respond(
            stream,
            200,
            "text/javascript; charset=utf-8",
            include_bytes!("../web/app.js"),
        ),
        ("GET", "/api/examples") => json_response(stream, 200, &json!(af_demo::examples())),
        ("GET", "/api/model") => {
            let s = state
                .lock()
                .map_err(|_| std::io::Error::other("state poisoned"))?;
            json_response(
                stream,
                200,
                &json!({"model":s.model.info(),"tools":af_demo::tools::NAMES}),
            )
        }
        ("POST", "/api/validate" | "/api/run") => {
            let doc = match serde_json::from_slice::<Document>(&req.body) {
                Ok(d) => d,
                Err(e) => return json_response(stream, 422, &json!({"error":e.to_string()})),
            };
            if req.path == "/api/validate" {
                match doc.validate() {
                    Ok(_) => json_response(stream, 200, &json!({"valid":true})),
                    Err(e) => json_response(stream, 422, &json!({"error":e})),
                }
            } else {
                match start(state, &doc) {
                    Ok(id) => json_response(stream, 202, &json!({"id":id})),
                    Err(e) => json_response(stream, 422, &json!({"error":e})),
                }
            }
        }
        ("POST", "/api/cancel") => {
            let value: Value = match serde_json::from_slice(&req.body) {
                Ok(value) => value,
                Err(e) => return json_response(stream, 422, &json!({"error":e.to_string()})),
            };
            let s = state
                .lock()
                .map_err(|_| std::io::Error::other("state poisoned"))?;
            if s.run.as_ref().is_some_and(|r| {
                Some(r.id) == value.get("id").and_then(Value::as_u64) && r.status == "running"
            }) {
                s.cancel.store(true, Ordering::Relaxed);
                json_response(stream, 200, &json!({"cancelling":true}))
            } else {
                json_response(stream, 409, &json!({"error":"no matching active run"}))
            }
        }
        ("GET", path) if path.starts_with("/api/run?id=") => {
            let id = path.trim_start_matches("/api/run?id=").parse::<u64>().ok();
            let s = state
                .lock()
                .map_err(|_| std::io::Error::other("state poisoned"))?;
            match &s.run {
                Some(r) if Some(r.id) == id => json_response(stream, 200, &json!(r)),
                _ => json_response(
                    stream,
                    404,
                    &json!({"error":"run not found; server retains latest run only"}),
                ),
            }
        }
        _ => json_response(stream, 404, &json!({"error":"not found"})),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let port = match args.as_slice() {
        [_] => 8787,
        [_, flag, value] if flag == "--port" => value.parse::<u16>()?,
        _ => return Err("usage: af-demo [--port 8787]".into()),
    };
    if port == 0 {
        return Err("port must be nonzero".into());
    }
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port))?;
    let model: Arc<dyn af_demo::model::ModelAdapter> =
        match std::env::var_os("AF_DEMO_MODEL_CONFIG") {
            Some(path) => Arc::new(af_demo::model::OpenAiCompatible::from_file(
                std::path::Path::new(&path),
            )?),
            None => Arc::new(af_demo::model::Unconfigured),
        };
    let state = Arc::new(Mutex::new(State {
        next: 1,
        run: None,
        cancel: Arc::new(AtomicBool::new(false)),
        model,
    }));
    println!("AgentFlow Studio · LOCAL NON-PRODUCTION DEMO · http://127.0.0.1:{port}");
    println!(
        "Offline deterministic and optional real LLM modes. Only configured model endpoint and closed local text tools. No formal Runtime, RootExport or durable execution."
    );
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                if let Err(e) = handle(&mut stream, port, &state) {
                    eprintln!("request: {e}");
                }
            }
            Err(e) => eprintln!("accept: {e}"),
        }
    }
    Ok(())
}
