//! `terminus mcp`: the archive, reachable from inside Claude Code.
//!
//! A subcommand of the same binary, dispatched from `main.rs` like every
//! other command, so install has one artifact to place and no version skew to
//! keep in step.
//!
//! **Bounded by its stdin.** It reads newline-delimited JSON-RPC
//! messages until EOF and returns 0. No socket, no listener, no port, no
//! background thread, and no ingest lock - reads run beside an ingest pass
//! under WAL, exactly as `cmd::status` already does. Claude Code's stdio
//! transport closes stdin first and only escalates to SIGTERM two seconds
//! later, measured in the 2.1.229 bundle, so exiting at EOF is the whole
//! lifecycle and no signal handling is needed on any platform. A server that
//! outlived its client would be the orphaned-process failure class this project
//! exists partly to retire.
//!
//! **stdout is the transport.** Nothing but JSON-RPC messages is ever written
//! there; every diagnostic goes to stderr. That includes the one warning the
//! shared read path prints for a store older than this build.
//!
//! **The store is opened per call, read-only.** Not once at startup: a session
//! that is ingested while this process is alive should be searchable by the next
//! call, and an open that happened before the client's first question would
//! answer with an archive that is already out of date. `cmd::read::open` is the
//! same entry point `search`, `show` and `sessions` use, so the read-only rule
//! and the missing-store answer are one implementation rather than two.
//!
//! **Nothing here may panic on what a client sent.** A panic writes a Rust
//! backtrace onto the transport stream and takes the session's tool with it, and
//! what the user sees is a tool that silently stopped working. The four ways to
//! do it are denied below rather than reviewed for, across this module and both
//! of its children; test code is exempt, where an assertion is a panic on
//! purpose.

#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing
    )
)]

pub mod rpc;
pub mod tools;

use std::io::{BufRead, Write};

use serde_json::{json, Value};

use super::Failure;

/// The server name a client shows the user.
const SERVER_NAME: &str = "terminus";

/// The protocol revisions this server agrees to, newest first.
///
/// Taken from the ones Claude Code 2.1.229 accepts, read out of that bundle
/// rather than guessed. The client sends its own in `initialize` and the server
/// answers with one it supports; a client that cannot use the answer
/// disconnects, and one that can proceeds.
///
/// `2026-07-28` is not in this list because that revision has no `initialize`.
/// It is offered through `server/discover` instead, and a client on it names
/// it in each request's `_meta`.
///
/// `2025-03-26` is left out because agreeing to a revision the server does not
/// speak is worse than answering an older one the client then falls back to. It
/// is the one revision that requires JSON-RPC batching, which `2024-11-05` did
/// not have and `2025-06-18` removed, and this server answers a batch with a
/// single `-32600`.
const PROTOCOL_VERSIONS: [&str; 4] = ["2025-11-25", "2025-06-18", "2024-11-05", "2024-10-07"];

/// What the server answers when the client asked for a revision it does not
/// know.
///
/// The oldest revision every current client still accepts would be the safe
/// choice for reach and the wrong one for meaning, since the tool-annotation
/// fields this server relies on were not in it. `2025-06-18` is the revision
/// this server's descriptors are written against.
const DEFAULT_PROTOCOL_VERSION: &str = "2025-06-18";

/// The one revision of the stateless era, the one a request names in `_meta`
/// rather than agreeing to through `initialize`.
const MODERN_PROTOCOL_VERSION: &str = "2026-07-28";

/// The `_meta` key a modern request names its revision under. Its presence is
/// what makes a request modern; a request without it is served as the older
/// era, with no check that an `initialize` came first.
const PROTOCOL_VERSION_KEY: &str = "io.modelcontextprotocol/protocolVersion";

/// The parsed command line for `mcp`, which takes nothing.
///
/// A unit rather than nothing at all so the dispatch table reads the same as
/// every other command's, and so a future flag has somewhere to land.
#[derive(Debug)]
pub struct Args;

/// `mcp` takes no arguments: it is spawned by a client, not typed.
///
/// Rejecting rather than ignoring, the rule every subcommand here keeps: a
/// command that silently ignored an argument is how `verify --json` came to look
/// supported before it was.
pub fn parse(parser: &mut lexopt::Parser) -> Result<Args, Failure> {
    if let Some(arg) = parser.next().map_err(|e| Failure::Misuse(e.to_string()))? {
        return Err(Failure::Misuse(crate::unexpected(arg)));
    }
    Ok(Args)
}

pub fn run(_args: Args) -> Result<(), Failure> {
    serve(std::io::stdin().lock(), std::io::stdout().lock())
}

/// The loop, over any reader and writer so a test can drive it without a spawn.
///
/// Lines are read as **bytes** and converted lossily rather than read as UTF-8:
/// `read_line` on a non-UTF-8 byte is an `InvalidData` error, and a transport
/// glitch must be one malformed message answered with a parse error, not the end
/// of the session.
pub fn serve(mut input: impl BufRead, mut output: impl Write) -> Result<(), Failure> {
    let mut line: Vec<u8> = Vec::new();
    loop {
        line.clear();
        let read = input
            .read_until(b'\n', &mut line)
            .map_err(|e| Failure::Operational(format!("reading stdin: {e}")))?;
        // EOF. The client closed the pipe, which is how this process is meant to
        // end: exit 0, having held nothing.
        if read == 0 {
            return Ok(());
        }

        let text = String::from_utf8_lossy(&line);
        let text = text.trim();
        // A blank line is framing, not a message. Answering one with a parse
        // error would make a transport that writes `\r\n` look broken.
        if text.is_empty() {
            continue;
        }

        let Some(response) = answer(text) else {
            continue;
        };
        // Flushed per message: the client is blocked on this line, and a
        // response sitting in a buffer is a hung tool call.
        if writeln!(output, "{response}").is_err() || output.flush().is_err() {
            // The client went away mid-write. That is the same shutdown EOF is,
            // arriving through the other pipe, and it is not a failure of the
            // archive.
            return Ok(());
        }
    }
}

/// One line in, one response line out - or `None` for a notification, which is
/// answered with nothing, ever.
fn answer(line: &str) -> Option<Value> {
    match rpc::parse(line) {
        rpc::Incoming::Notification => None,
        rpc::Incoming::Invalid { id, code, message } => {
            Some(rpc::failure(id.as_ref(), code, &message))
        }
        rpc::Incoming::Request(request) => {
            // Refused before the method runs, `server/discover` and `ping`
            // included. `initialize` is the older era's opening and is never
            // version checked, whatever its `_meta` holds.
            if request.method != "initialize" {
                if let Some(requested) = unsupported(&request.params) {
                    return Some(rpc::failure_with_data(
                        &request.id,
                        rpc::code::UNSUPPORTED_PROTOCOL_VERSION,
                        &format!("this server supports protocol {MODERN_PROTOCOL_VERSION} only"),
                        json!({"supported": [MODERN_PROTOCOL_VERSION], "requested": requested}),
                    ));
                }
            }
            Some(route(&request))
        }
    }
}

/// Run one request that has passed the version check.
fn route(request: &rpc::Request) -> Value {
    match request.method.as_str() {
        "initialize" => rpc::result(&request.id, initialize(&request.params)),
        "server/discover" => rpc::result(&request.id, discover()),
        // `ttlMs` 0 so a host fetches the list again rather than keep one
        // from an older binary; `private` because it comes from one user's
        // own server.
        "tools/list" => rpc::result(
            &request.id,
            json!({"tools": tools::descriptors(), "ttlMs": 0, "cacheScope": "private"}),
        ),
        // The only method that reads the archive. Its `Err` is a protocol
        // failure - params that do not name a call this server has - and
        // never a tool that could not answer, which is a result carrying a
        // reason.
        "tools/call" => match tools::call(&request.params) {
            Ok(result) => rpc::result(&request.id, result),
            Err((code, message)) => rpc::failure(Some(&request.id), code, &message),
        },
        // Answered with a result carrying only what every result carries,
        // not `-32601`: the bundled client implements `ping` on both sides,
        // and a liveness probe answered "no such method" reads as an
        // unhealthy server.
        "ping" => rpc::result(&request.id, json!({})),
        other => rpc::failure(
            Some(&request.id),
            rpc::code::METHOD_NOT_FOUND,
            &format!("this server has no method {other:?}"),
        ),
    }
}

/// The revision a modern request named, when it is not the one this server
/// speaks. `None` for a request that names `2026-07-28` and for one with no
/// version in `_meta` at all, which is served as the older era.
///
/// Read with `get` throughout, so params, `_meta` or the version that are not
/// what a client should have sent read as absent rather than panic.
fn unsupported(params: &Value) -> Option<&Value> {
    let asked = params.get("_meta")?.get(PROTOCOL_VERSION_KEY)?;
    (asked.as_str() != Some(MODERN_PROTOCOL_VERSION)).then_some(asked)
}

/// The `server/discover` answer: the one stateless revision, the one
/// capability, and the cache fields `tools/list` carries.
///
/// `supportedVersions` lists `2026-07-28` alone. A client picks one of these
/// for its stateless requests, and no older revision has a stateless form. No
/// `instructions`: anything there sits in every session's context, and the
/// tool descriptions already say what each tool returns.
fn discover() -> Value {
    json!({
        "supportedVersions": [MODERN_PROTOCOL_VERSION],
        "capabilities": {"tools": {}},
        "ttlMs": 0,
        "cacheScope": "private",
    })
}

/// The handshake answer: the negotiated revision, the one capability this
/// server has, and who it is.
fn initialize(params: &Value) -> Value {
    let requested = params.get("protocolVersion").and_then(Value::as_str);
    let version = match requested {
        Some(asked) if PROTOCOL_VERSIONS.contains(&asked) => asked,
        _ => DEFAULT_PROTOCOL_VERSION,
    };
    json!({
        "protocolVersion": version,
        // Tools and nothing else. No resources, no prompts, no sampling, and
        // no `listChanged`: the tool list is fixed at compile time, so a server
        // advertising that it might change would be promising a notification it
        // can never have cause to send.
        "capabilities": {"tools": {}},
        "serverInfo": {"name": SERVER_NAME, "version": env!("CARGO_PKG_VERSION")},
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Drive the loop over a byte buffer and read back one response per line.
    fn exchange(input: &str) -> Vec<Value> {
        let mut output: Vec<u8> = Vec::new();
        serve(input.as_bytes(), &mut output).expect("the loop ends at EOF");
        String::from_utf8_lossy(&output)
            .lines()
            .map(|line| {
                serde_json::from_str(line)
                    .unwrap_or_else(|e| panic!("stdout is not JSON-RPC ({e}): {line}"))
            })
            .collect()
    }

    /// The client's own revision is echoed when the server knows it, and an
    /// unknown one is answered with the server's pinned default rather than
    /// with an error.
    #[test]
    fn initialize_negotiates_a_version_both_sides_know() {
        for asked in PROTOCOL_VERSIONS {
            let line = format!(
                r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"protocolVersion":"{asked}"}}}}"#
            );
            let responses = exchange(&line);
            assert_eq!(responses.len(), 1);
            assert_eq!(
                responses[0]["result"]["protocolVersion"],
                Value::from(asked)
            );
        }

        // 2026-07-28 has no `initialize`. It is offered through
        // `server/discover`, so asking for it here is answered like any
        // revision the list does not hold. Pinned by name because the loop
        // above only walks what the list holds.
        assert!(!PROTOCOL_VERSIONS.contains(&"2026-07-28"));
        let responses = exchange(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2026-07-28"}}"#,
        );
        assert_eq!(
            responses[0]["result"]["protocolVersion"],
            Value::from(DEFAULT_PROTOCOL_VERSION),
            "initialize agreed to 2026-07-28, which has no initialize and is offered through server/discover"
        );

        let responses = exchange(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"1999-01-01"}}"#,
        );
        assert_eq!(
            responses[0]["result"]["protocolVersion"],
            Value::from(DEFAULT_PROTOCOL_VERSION),
            "an unknown revision must be answered, not refused"
        );
        assert_eq!(
            responses[0]["result"]["serverInfo"]["name"],
            Value::from(SERVER_NAME)
        );
        assert!(responses[0]["result"]["capabilities"]["tools"].is_object());
    }

    /// `2025-03-26` requires batches, which this server refuses, so it is not
    /// offered. The loop above walks the list and stops covering it once it is
    /// gone, so this pins it by name.
    #[test]
    fn the_revision_that_requires_batches_is_not_offered() {
        assert!(!PROTOCOL_VERSIONS.contains(&"2025-03-26"));

        let responses = exchange(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26"}}"#,
        );
        assert_eq!(responses.len(), 1);
        assert_eq!(
            responses[0]["result"]["protocolVersion"],
            Value::from(DEFAULT_PROTOCOL_VERSION),
            "the server agreed to a revision whose batches it does not answer"
        );
    }

    /// One response per request, in order, and none at all for a notification.
    #[test]
    fn a_notification_is_never_answered_and_every_request_is() {
        let responses = exchange(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{}}\n\
             {\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n\
             \n\
             {\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"ping\"}\n\
             {\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/list\"}\n",
        );

        let ids: Vec<&Value> = responses.iter().map(|r| &r["id"]).collect();
        assert_eq!(
            ids,
            vec![&Value::from(1), &Value::from(2), &Value::from(3)],
            "one response per request, in order, and none for the notification"
        );
        assert_eq!(
            responses[1]["result"],
            json!({
                "resultType": "complete",
                "_meta": {rpc::SERVER_INFO_KEY: {"name": SERVER_NAME, "version": env!("CARGO_PKG_VERSION")}},
            }),
            "ping carries what every result carries and nothing else"
        );
        assert_eq!(
            responses[2]["result"]["tools"].as_array().map(Vec::len),
            Some(3)
        );
    }

    /// An unknown method is `-32601`, and the loop answers the next request
    /// afterwards rather than ending.
    #[test]
    fn an_unknown_method_is_an_error_and_the_loop_carries_on() {
        let responses = exchange(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"resources/list\"}\n\
             garbage that is not json\n\
             {\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"ping\"}\n",
        );
        assert_eq!(responses.len(), 3);
        assert_eq!(
            responses[0]["error"]["code"],
            Value::from(rpc::code::METHOD_NOT_FOUND)
        );
        assert_eq!(
            responses[1]["error"]["code"],
            Value::from(rpc::code::PARSE_ERROR)
        );
        assert_eq!(responses[1]["id"], Value::Null);
        assert!(responses[2]["result"].is_object(), "the loop carried on");
    }

    /// The `_meta` a stateless request carries, naming `version`.
    fn meta(version: impl Into<Value>) -> Value {
        json!({"io.modelcontextprotocol/protocolVersion": version.into()})
    }

    /// One line per request, in order.
    fn lines(requests: &[Value]) -> String {
        requests
            .iter()
            .map(|request| format!("{request}\n"))
            .collect()
    }

    /// What every result carries under `_meta`.
    fn server_info() -> Value {
        json!({"io.modelcontextprotocol/serverInfo":
               {"name": "terminus", "version": env!("CARGO_PKG_VERSION")}})
    }

    /// The tool names a `tools/list` result holds, in the order it holds them.
    fn tool_names(result: &Value) -> Vec<&str> {
        result["tools"]
            .as_array()
            .map(|tools| tools.iter().filter_map(|t| t["name"].as_str()).collect())
            .unwrap_or_default()
    }

    /// `server/discover` gets the same whole object with `_meta` naming
    /// 2026-07-28 and with no params at all.
    ///
    /// Compared whole and written out literally, not built from the module's
    /// constants: a check of single members would still pass with an
    /// `instructions` key added or an older revision in `supportedVersions`.
    #[test]
    fn server_discover_answers_the_one_stateless_revision_with_or_without_meta() {
        let expected = json!({
            "resultType": "complete",
            "supportedVersions": ["2026-07-28"],
            "capabilities": {"tools": {}},
            "_meta": server_info(),
            "ttlMs": 0,
            "cacheScope": "private",
        });
        let responses = exchange(&lines(&[
            json!({"jsonrpc": "2.0", "id": 1, "method": "server/discover",
                   "params": {"_meta": meta("2026-07-28")}}),
            json!({"jsonrpc": "2.0", "id": 2, "method": "server/discover"}),
        ]));
        assert_eq!(responses.len(), 2);
        for response in &responses {
            assert!(response.get("error").is_none(), "{response}");
            assert_eq!(response["result"], expected, "{response}");
        }
    }

    /// All five methods carry `resultType` and the server's identity, with and
    /// without `_meta`, and `tools/list` carries its cache fields and the three
    /// tools in order either way.
    ///
    /// Walked over every method because the members are added in one place: a
    /// method that built its result around that place rather than through it
    /// would pass a test of any other method.
    #[test]
    fn every_result_carries_the_result_type_and_the_servers_identity_on_both_eras() {
        // No `query`, so the call is refused before the store is opened and this
        // test reads no archive. The refusal is still a result.
        let methods = [
            ("initialize", json!({"protocolVersion": "2025-06-18"})),
            ("ping", json!({})),
            ("tools/list", json!({})),
            (
                "tools/call",
                json!({"name": "recall_search", "arguments": {}}),
            ),
            ("server/discover", json!({})),
        ];
        for era in [None, Some(meta("2026-07-28"))] {
            for (method, params) in &methods {
                let mut params = params.clone();
                if let Some(meta) = &era {
                    params["_meta"] = meta.clone();
                }
                let request =
                    json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params});
                let responses = exchange(&lines(std::slice::from_ref(&request)));
                assert_eq!(responses.len(), 1, "{request}");
                let response = &responses[0];
                assert!(response.get("error").is_none(), "{request} -> {response}");
                assert_eq!(
                    response["result"]["resultType"], "complete",
                    "{request} -> {response}"
                );
                assert_eq!(
                    response["result"]["_meta"],
                    server_info(),
                    "{request} -> {response}"
                );

                if *method == "tools/list" {
                    assert_eq!(response["result"]["ttlMs"], 0, "{response}");
                    assert_eq!(response["result"]["cacheScope"], "private", "{response}");
                    assert_eq!(
                        tool_names(&response["result"]),
                        ["recall_search", "recall_context", "recall_get"],
                        "{response}"
                    );
                }
            }
        }
    }

    /// A request whose `_meta` names any other revision is refused with
    /// `-32022` before its method runs, with `data` listing what is supported
    /// and echoing what was asked as it was sent, a number included.
    ///
    /// The `tools/call` here names no query, so a check that ran after the
    /// method would answer with a refusal result rather than this error.
    #[test]
    fn a_request_naming_another_revision_is_refused_with_what_is_supported() {
        let methods = [
            ("server/discover", json!({})),
            ("tools/list", json!({})),
            (
                "tools/call",
                json!({"name": "recall_search", "arguments": {}}),
            ),
        ];
        for asked in [json!("2025-11-25"), json!("2026-07-29"), json!(20260728)] {
            for (method, params) in &methods {
                let mut params = params.clone();
                params["_meta"] = meta(asked.clone());
                let request =
                    json!({"jsonrpc": "2.0", "id": 4, "method": method, "params": params});
                let responses = exchange(&lines(std::slice::from_ref(&request)));
                assert_eq!(responses.len(), 1, "{request}");
                let response = &responses[0];
                assert!(response.get("result").is_none(), "{request} -> {response}");
                assert_eq!(response["id"], 4, "{response}");
                assert_eq!(response["error"]["code"], -32022, "{request} -> {response}");
                assert_eq!(
                    response["error"]["data"],
                    json!({"supported": ["2026-07-28"], "requested": asked}),
                    "{request} -> {response}"
                );
            }
        }
    }

    /// The version check runs before the method: an unknown tool named on
    /// another revision is `-32022`, and on 2026-07-28 it is still `-32602`.
    /// `initialize` is never checked, whatever its `_meta` names.
    #[test]
    fn the_version_check_runs_first_and_never_on_initialize() {
        let responses = exchange(&lines(&[
            json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call",
                   "params": {"name": "recall_everything", "_meta": meta("2025-11-25")}}),
            json!({"jsonrpc": "2.0", "id": 2, "method": "tools/call",
                   "params": {"name": "recall_everything", "_meta": meta("2026-07-28")}}),
            json!({"jsonrpc": "2.0", "id": 3, "method": "initialize",
                   "params": {"protocolVersion": "2025-06-18", "_meta": meta("2025-11-25")}}),
        ]));
        assert_eq!(responses.len(), 3);
        assert_eq!(responses[0]["error"]["code"], -32022, "{}", responses[0]);
        assert_eq!(
            responses[1]["error"]["code"],
            Value::from(rpc::code::INVALID_PARAMS),
            "{}",
            responses[1]
        );
        assert_eq!(
            responses[2]["result"]["protocolVersion"], "2025-06-18",
            "initialize was version checked: {}",
            responses[2]
        );
    }

    /// A `tools/list` with no `_meta` and no `initialize` before it gets the
    /// three tools. It is served as the older era with no ordering check, so a
    /// client that lists before it opens is not refused.
    #[test]
    fn a_tools_list_with_no_meta_and_no_initialize_is_served() {
        let responses = exchange(r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#);
        assert_eq!(responses.len(), 1);
        assert!(responses[0].get("error").is_none(), "{}", responses[0]);
        assert_eq!(
            tool_names(&responses[0]["result"]),
            ["recall_search", "recall_context", "recall_get"]
        );
    }
}
