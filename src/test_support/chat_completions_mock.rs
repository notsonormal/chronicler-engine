//! A one-request chat-completions endpoint on a free local port, for provider tests.
//!
//! The endpoint answers with a fixed completion and hands the test the body of
//! the request it received, so a test pins what a provider puts on the wire.
#![allow(clippy::expect_used)]

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::time::Duration;

const STREAM_TIMEOUT: Duration = Duration::from_secs(5);
const BODY_TIMEOUT: Duration = Duration::from_secs(5);

/// The text of the fixed completion the endpoint answers with.
pub const STUB_ANSWER: &str = "stub answer";

pub struct ChatCompletionsMock {
    /// The `base_url` a connection points at to reach the endpoint.
    pub base_url: String,
    received_body: mpsc::Receiver<String>,
}

impl ChatCompletionsMock {
    /// Binds the endpoint and serves one request on a background thread.
    pub fn start() -> Self {
        let (listener, base_url) = bind_local_port();
        let (sender, received_body) = mpsc::channel();
        std::thread::spawn(move || serve_one_request(&listener, &sender));
        Self {
            base_url,
            received_body,
        }
    }

    /// The JSON body of the request the endpoint received. Fails the test when
    /// no request arrives in time.
    pub fn received_body(&self) -> serde_json::Value {
        await_json_body(&self.received_body)
    }
}

#[allow(clippy::expect_used)]
fn bind_local_port() -> (TcpListener, String) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind a free local port");
    let base_url = format!("http://{}", listener.local_addr().expect("local address"));
    (listener, base_url)
}

#[allow(clippy::expect_used)]
fn serve_one_request(listener: &TcpListener, sender: &mpsc::Sender<String>) {
    let (mut stream, _) = listener.accept().expect("accept the request");
    stream
        .set_read_timeout(Some(STREAM_TIMEOUT))
        .expect("set the read timeout");
    let body = read_request_body(&mut stream);
    let answer = format!(
        r#"{{"choices":[{{"message":{{"content":"{STUB_ANSWER}"}},"finish_reason":"stop"}}]}}"#
    );
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{answer}",
        answer.len()
    );
    stream
        .write_all(response.as_bytes())
        .expect("write the answer");
    sender.send(body).expect("hand the body to the test");
}

#[allow(clippy::expect_used)]
fn await_json_body(received_body: &mpsc::Receiver<String>) -> serde_json::Value {
    let body = received_body
        .recv_timeout(BODY_TIMEOUT)
        .expect("the endpoint received a request");
    serde_json::from_str(&body).expect("the request body is JSON")
}

#[allow(clippy::expect_used)]
fn read_request_body(stream: &mut impl Read) -> String {
    let mut received = Vec::new();
    let mut chunk = [0u8; 4096];
    let (header_end, content_length) = loop {
        let read = stream.read(&mut chunk).expect("read the headers");
        assert!(read > 0, "the client closed before it sent the headers");
        received.extend_from_slice(&chunk[..read]);
        let text = String::from_utf8_lossy(&received).into_owned();
        if let Some(header_end) = text.find("\r\n\r\n") {
            let content_length = text[..header_end]
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().expect("content length"))
                })
                .unwrap_or(0);
            break (header_end + 4, content_length);
        }
    };
    while received.len() < header_end + content_length {
        let read = stream.read(&mut chunk).expect("read the body");
        assert!(read > 0, "the client closed before it sent the body");
        received.extend_from_slice(&chunk[..read]);
    }
    String::from_utf8_lossy(&received[header_end..]).into_owned()
}
