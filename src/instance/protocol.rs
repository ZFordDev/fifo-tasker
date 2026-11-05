use std::{
    io::{self, Read, Write},
    net::TcpStream,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};

use crate::{
    logic::tasks::{TaskData, TaskDocument},
    task_watch::ReloadHandle,
};

pub(super) const MAGIC: &[u8; 5] = b"FIFO2";
pub(super) const MAX_REQUEST: usize = 4096;
pub(super) const MAX_RESPONSE: usize = 4 * 1024 * 1024;
pub(super) const RESPONSE_TIMEOUT: Duration = Duration::from_secs(4);

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(
    tag = "command",
    rename_all = "snake_case",
    try_from = "RequestMessage"
)]
pub(crate) enum Request {
    OpenHud,
    OpenDashboard,
    List,
    Status,
    Reload,
}

// A strict envelope also checks extra fields on commands without arguments.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RequestMessage {
    command: String,
}

impl TryFrom<RequestMessage> for Request {
    type Error = String;

    fn try_from(message: RequestMessage) -> Result<Self, Self::Error> {
        match message.command.as_str() {
            "open_hud" => Ok(Self::OpenHud),
            "open_dashboard" => Ok(Self::OpenDashboard),
            "list" => Ok(Self::List),
            "status" => Ok(Self::Status),
            "reload" => Ok(Self::Reload),
            _ => Err(format!("Unknown request command: {}", message.command)),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "result", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Response {
    Queued,
    Tasks {
        revision: u64,
        document: TaskDocument,
        load_error: Option<String>,
        watch_error: Option<String>,
    },
    Status {
        pid: u32,
        revision: u64,
        task_count: usize,
        watching: bool,
        load_error: Option<String>,
        watch_error: Option<String>,
    },
    Reloaded {
        revision: u64,
    },
    Error {
        code: String,
        message: String,
    },
}

impl Response {
    pub(super) fn error(code: &str, message: impl Into<String>) -> Self {
        Self::Error {
            code: code.into(),
            message: message.into(),
        }
    }
}

#[derive(Clone)]
pub(super) struct TaskService {
    pub data: Arc<RwLock<TaskData>>,
    pub reload: Option<ReloadHandle>,
}

impl TaskService {
    pub fn respond(&self, request: &Request) -> Response {
        if matches!(request, Request::Reload) {
            return match &self.reload {
                Some(loader) => match loader.reload(Duration::from_secs(2)) {
                    Ok(revision) => Response::Reloaded { revision },
                    Err(error) => Response::error("reload_failed", error),
                },
                None => Response::error(
                    "reload_unavailable",
                    "The background task loader is unavailable",
                ),
            };
        }
        // Copy one coherent snapshot, never mix a list with another revision's status.
        let data = self.data.read().unwrap();
        match request {
            Request::List => Response::Tasks {
                revision: data.revision,
                document: data.document.clone(),
                load_error: data.load_error.clone(),
                watch_error: data.watch_error.clone(),
            },
            Request::Status => Response::Status {
                pid: std::process::id(),
                revision: data.revision,
                task_count: data.document.tasks().len(),
                watching: self.reload.as_ref().is_some_and(ReloadHandle::is_active)
                    && data.watch_error.is_none(),
                load_error: data.load_error.clone(),
                watch_error: data.watch_error.clone(),
            },
            _ => Response::error("invalid_request", "This request is not a task-data query"),
        }
    }
}

pub(super) fn encode(response: &Response) -> io::Result<Vec<u8>> {
    let bytes = serde_json::to_vec(response).map_err(io::Error::other)?;
    if bytes.len() <= MAX_RESPONSE {
        return Ok(bytes);
    }
    serde_json::to_vec(&Response::error(
        "response_too_large",
        "Task response exceeds the 4 MiB protocol limit",
    ))
    .map_err(io::Error::other)
}

pub(super) fn read_frame(
    stream: &mut TcpStream,
    limit: usize,
    deadline: Instant,
) -> io::Result<Vec<u8>> {
    let mut length = [0; 4];
    read_until(stream, &mut length, deadline)?;
    let length = u32::from_be_bytes(length) as usize;
    if length == 0 || length > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Message length exceeds the protocol limit",
        ));
    }
    let mut bytes = vec![0; length];
    read_until(stream, &mut bytes, deadline)?;
    Ok(bytes)
}

pub(super) fn write_frame(
    stream: &mut TcpStream,
    bytes: &[u8],
    deadline: Instant,
) -> io::Result<()> {
    write_until(stream, &(bytes.len() as u32).to_be_bytes(), deadline)?;
    write_until(stream, bytes, deadline)
}

// A trickle of bytes doesn't get to extend a connection's deadline indefinitely.
pub(super) fn read_until(
    stream: &mut TcpStream,
    mut bytes: &mut [u8],
    deadline: Instant,
) -> io::Result<()> {
    while !bytes.is_empty() {
        stream.set_read_timeout(Some(remaining(deadline)?))?;
        match stream.read(bytes) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Incomplete protocol message",
                ));
            }
            Ok(count) => bytes = &mut bytes[count..],
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

pub(super) fn write_until(
    stream: &mut TcpStream,
    mut bytes: &[u8],
    deadline: Instant,
) -> io::Result<()> {
    while !bytes.is_empty() {
        stream.set_write_timeout(Some(remaining(deadline)?))?;
        match stream.write(bytes) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "Could not send protocol message",
                ));
            }
            Ok(count) => bytes = &bytes[count..],
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

fn remaining(deadline: Instant) -> io::Result<Duration> {
    deadline
        .checked_duration_since(Instant::now())
        .filter(|duration| !duration.is_zero())
        .ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "Protocol deadline elapsed"))
}
