//! A one-request chat-completions endpoint on a free local port, for provider tests.
//!
//! The endpoint answers with a fixed completion and hands the test the headers
//! and the body of the request it received, so a test pins what a provider
//! puts on the wire.
#![allow(clippy::expect_used)]

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{mpsc, OnceLock};
use std::time::Duration;

const STREAM_TIMEOUT: Duration = Duration::from_secs(5);
const BODY_TIMEOUT: Duration = Duration::from_secs(5);

pub const STUB_ANSWER: &str = "stub answer";

struct ReceivedRequest {
    /// Header names in lower case.
    headers: HashMap<String, String>,
    body: String,
}

pub struct ChatCompletionsMock {
    pub base_url: String,
    receiver: mpsc::Receiver<ReceivedRequest>,
    received: OnceLock<ReceivedRequest>,
}

impl ChatCompletionsMock {
    pub fn start() -> Self {
        let (listener, base_url) = bind_local_port();
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || serve_one_request(&listener, &sender));
        Self {
            base_url,
            receiver,
            received: OnceLock::new(),
        }
    }

    /// The JSON body of the request the endpoint received. Fails the test when
    /// no request arrives in time.
    pub fn received_body(&self) -> serde_json::Value {
        parse_json_body(&self.received().body)
    }

    /// The headers of the request the endpoint received, keyed by the
    /// lower-case header name. Fails the test when no request arrives in time.
    pub fn received_headers(&self) -> HashMap<String, String> {
        self.received().headers.clone()
    }

    fn received(&self) -> &ReceivedRequest {
        self.received.get_or_init(|| await_request(&self.receiver))
    }
}

#[allow(clippy::expect_used)]
fn await_request(receiver: &mpsc::Receiver<ReceivedRequest>) -> ReceivedRequest {
    receiver
        .recv_timeout(BODY_TIMEOUT)
        .expect("the endpoint received a request")
}

#[allow(clippy::expect_used)]
fn parse_json_body(body: &str) -> serde_json::Value {
    serde_json::from_str(body).expect("the request body is JSON")
}

#[allow(clippy::expect_used)]
fn bind_local_port() -> (TcpListener, String) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind a free local port");
    let base_url = format!("http://{}", listener.local_addr().expect("local address"));
    (listener, base_url)
}

#[allow(clippy::expect_used)]
fn serve_one_request(listener: &TcpListener, sender: &mpsc::Sender<ReceivedRequest>) {
    let (mut stream, _) = listener.accept().expect("accept the request");
    stream
        .set_read_timeout(Some(STREAM_TIMEOUT))
        .expect("set the read timeout");
    let request = read_request(&mut stream);
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
    sender.send(request).expect("hand the request to the test");
}

#[allow(clippy::expect_used)]
fn read_request(stream: &mut impl Read) -> ReceivedRequest {
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
    let head = String::from_utf8_lossy(&received[..header_end]).into_owned();
    let headers = head
        .lines()
        .skip(1)
        .filter_map(|line| {
            let (name, value) = line.split_once(':')?;
            Some((name.trim().to_ascii_lowercase(), value.trim().to_string()))
        })
        .collect();
    ReceivedRequest {
        headers,
        body: String::from_utf8_lossy(&received[header_end..]).into_owned(),
    }
}
