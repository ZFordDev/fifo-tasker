use std::{
    collections::VecDeque,
    fs::{self, File, OpenOptions, TryLockError},
    io::{self, Read, Write},
    net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    path::Path,
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use crate::logic::Command;
use crate::{logic::tasks::TaskData, task_watch::ReloadHandle};
use eframe::egui;

mod protocol;
pub(crate) use protocol::{Request, Response};

const STARTUP_TIMEOUT: Duration = Duration::from_secs(10);
const IO_TIMEOUT: Duration = Duration::from_millis(300);
const MAGIC: &[u8; 5] = b"FIFO1";

pub(crate) struct Instance {
    commands: Receiver<Command>,
    wake: Arc<Mutex<Option<egui::Context>>>,
    service: Arc<Mutex<Option<protocol::TaskService>>>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
    address: SocketAddr,
    // Keep this handle until the listener stops. An empty file isn't a running app.
    _lock: File,
}

impl Instance {
    pub(crate) fn acquire() -> io::Result<Option<Self>> {
        let dirs = directories::ProjectDirs::from("dev", "ZFordDev", "FIFO Tasker")
            .ok_or_else(|| io::Error::other("Cannot locate the user's app-data directory"))?;
        Self::acquire_in(&dirs.data_local_dir().join("instance"), STARTUP_TIMEOUT)
    }

    fn acquire_in(directory: &Path, timeout: Duration) -> io::Result<Option<Self>> {
        let mut builder = fs::DirBuilder::new();
        builder.recursive(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(directory)?;
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let lock = options.open(directory.join("owner.lock"))?;
        let mut request_id = [0u8; 16];
        getrandom::fill(&mut request_id).map_err(|error| io::Error::other(error.to_string()))?;
        let deadline = Instant::now() + timeout;
        loop {
            match lock.try_lock() {
                Ok(()) => return Self::primary(lock, directory).map(Some),
                Err(TryLockError::Error(error)) => return Err(error),
                Err(TryLockError::WouldBlock) => {}
            }
            // The winner may still be publishing its address. Give it a moment to get dressed.
            if Self::notify(directory, &request_id).is_ok() {
                return Ok(None);
            }
            if Instant::now() >= deadline {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "FIFO Tasker is running but did not acknowledge the dashboard request",
                ));
            }
            thread::sleep(Duration::from_millis(50));
        }
    }

    fn primary(lock: File, directory: &Path) -> io::Result<Self> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
        let address = listener.local_addr()?;
        let mut token = [0u8; 32];
        getrandom::fill(&mut token).map_err(|error| io::Error::other(error.to_string()))?;
        let mut endpoint = Vec::from(address.port().to_be_bytes());
        endpoint.extend_from_slice(&std::process::id().to_be_bytes());
        endpoint.extend_from_slice(&token);
        let mut options = OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        // Separate from the locked file: Windows file locks also block ordinary reads.
        options
            .open(directory.join("endpoint"))?
            .write_all(&endpoint)?;

        let (sender, commands) = mpsc::channel();
        let wake = Arc::new(Mutex::new(None::<egui::Context>));
        let stop = Arc::new(AtomicBool::new(false));
        let service = Arc::new(Mutex::new(None));
        let worker_service = Arc::clone(&service);
        let worker_wake = Arc::clone(&wake);
        let worker_stop = Arc::clone(&stop);
        let worker = thread::Builder::new()
            .name("instance-listener".into())
            .spawn(move || {
                let mut recent_requests = VecDeque::new();
                for connection in listener.incoming() {
                    if worker_stop.load(Ordering::Acquire) {
                        break;
                    }
                    let mut stream = match connection {
                        Ok(stream) => stream,
                        Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                        Err(error) => {
                            eprintln!("Instance listener failed: {error}");
                            break;
                        }
                    };
                    let accepted = serve(
                        &mut stream,
                        &token,
                        &sender,
                        &worker_wake,
                        &worker_service,
                        &mut recent_requests,
                    );
                    // Bad or abandoned connections must not take the listener down with them.
                    let _ = accepted;
                }
            })?;
        Ok(Self {
            commands,
            wake,
            service,
            stop,
            worker: Some(worker),
            address,
            _lock: lock,
        })
    }

    fn notify(directory: &Path, request_id: &[u8; 16]) -> io::Result<()> {
        let endpoint = fs::read(directory.join("endpoint"))?;
        if endpoint.len() != 38 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Incomplete instance endpoint",
            ));
        }
        let port = u16::from_be_bytes([endpoint[0], endpoint[1]]);
        let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
        let mut stream = TcpStream::connect_timeout(&address, IO_TIMEOUT)?;
        stream.set_read_timeout(Some(IO_TIMEOUT))?;
        stream.set_write_timeout(Some(IO_TIMEOUT))?;
        #[cfg(windows)]
        {
            let pid = u32::from_be_bytes(endpoint[2..6].try_into().unwrap());
            // Pass foreground permission to the owner; Windows still has the final say.
            unsafe {
                windows_sys::Win32::UI::WindowsAndMessaging::AllowSetForegroundWindow(pid);
            }
        }
        stream.write_all(MAGIC)?;
        stream.write_all(&endpoint[6..])?;
        stream.write_all(request_id)?;
        let mut reply = [0u8; 2];
        stream.read_exact(&mut reply)?;
        if &reply != b"OK" {
            return Err(io::Error::other("Invalid launch acknowledgement"));
        }
        Ok(())
    }

    pub(crate) fn attach(&self, ctx: egui::Context) {
        // Requests arriving before the first frame stay queued until the GUI is ready.
        ctx.request_repaint_of(egui::ViewportId::ROOT);
        *self.wake.lock().unwrap() = Some(ctx);
    }

    pub(crate) fn serve_tasks(&self, data: Arc<RwLock<TaskData>>, reload: Option<ReloadHandle>) {
        *self.service.lock().unwrap() = Some(protocol::TaskService { data, reload });
    }

    #[expect(
        dead_code,
        reason = "Client entry point will be connected to CLI arguments in Stage 6"
    )]
    pub(crate) fn request(request: Request) -> io::Result<Response> {
        let dirs = directories::ProjectDirs::from("dev", "ZFordDev", "FIFO Tasker")
            .ok_or_else(|| io::Error::other("Cannot locate the user's app-data directory"))?;
        Self::request_in(
            &dirs.data_local_dir().join("instance"),
            &request,
            STARTUP_TIMEOUT,
        )
    }

    fn request_in(directory: &Path, request: &Request, timeout: Duration) -> io::Result<Response> {
        let mut id = [0u8; 16];
        getrandom::fill(&mut id).map_err(|error| io::Error::other(error.to_string()))?;
        let deadline = Instant::now() + timeout;
        loop {
            let error = match exchange(directory, request, &id, deadline) {
                Ok(Response::Error { code, message }) if code == "not_ready" => {
                    io::Error::other(message)
                }
                Ok(response) => return Ok(response),
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::InvalidData | io::ErrorKind::PermissionDenied
                    ) =>
                {
                    return Err(error);
                }
                Err(error) => error,
            };
            if Instant::now() >= deadline {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    format!("No completed response from FIFO Tasker: {error}"),
                ));
            }
            thread::sleep(Duration::from_millis(50));
        }
    }

    pub(crate) fn next_command(&self) -> Option<Command> {
        self.commands.try_recv().ok()
    }
}

struct CachedReply {
    id: [u8; 16],
    request: Option<Request>, // None is the original launch-only protocol.
    bytes: Vec<u8>,
}

fn serve(
    stream: &mut TcpStream,
    token: &[u8; 32],
    sender: &mpsc::Sender<Command>,
    wake: &Mutex<Option<egui::Context>>,
    service: &Mutex<Option<protocol::TaskService>>,
    recent: &mut VecDeque<CachedReply>,
) -> io::Result<()> {
    let deadline = Instant::now() + IO_TIMEOUT;
    let mut header = [0u8; 53];
    protocol::read_until(stream, &mut header, deadline)?;
    let legacy = &header[..5] == MAGIC;
    if (!legacy && &header[..5] != protocol::MAGIC) || header[5..37] != *token {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid instance request",
        ));
    }
    let id = header[37..].try_into().unwrap();
    let request = if legacy {
        None
    } else {
        let parsed =
            protocol::read_frame(stream, protocol::MAX_REQUEST, deadline).and_then(|bytes| {
                serde_json::from_slice::<Request>(&bytes)
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
            });
        match parsed {
            Ok(request) => Some(request),
            Err(error) => {
                return protocol::write_frame(
                    stream,
                    &protocol::encode(&Response::error("invalid_request", error.to_string()))?,
                    Instant::now() + IO_TIMEOUT,
                );
            }
        }
    };
    if let Some(cached) = recent.iter().find(|cached| cached.id == id) {
        if cached.request != request {
            if legacy {
                return Err(io::Error::other(
                    "Request ID was reused for a different command",
                ));
            }
            return protocol::write_frame(
                stream,
                &protocol::encode(&Response::error(
                    "request_id_reused",
                    "Request ID belongs to another command",
                ))?,
                Instant::now() + IO_TIMEOUT,
            );
        }
        return send_reply(stream, &cached.bytes, legacy);
    }
    let command = match request {
        None | Some(Request::OpenDashboard) => Some(Command::OpenDashboard),
        Some(Request::OpenHud) => Some(Command::OpenHud),
        _ => None,
    };
    let response = if let Some(command) = command {
        sender
            .send(command)
            .map_err(|_| io::Error::other("Application is shutting down"))?;
        if let Some(ctx) = wake.lock().unwrap().as_ref() {
            ctx.request_repaint_of(egui::ViewportId::ROOT);
        }
        Response::Queued
    } else {
        // Release the service lock before waiting for background reload completion.
        let handler = service.lock().unwrap().clone();
        match handler {
            Some(handler) => handler.respond(request.as_ref().unwrap()),
            None => {
                return protocol::write_frame(
                    stream,
                    &protocol::encode(&Response::error(
                        "not_ready",
                        "Task service is still starting",
                    ))?,
                    Instant::now() + IO_TIMEOUT,
                );
            }
        }
    };
    let bytes = if legacy {
        b"OK".to_vec()
    } else {
        protocol::encode(&response)?
    };
    let sent = send_reply(stream, &bytes, legacy);
    // Remember completed replies even if the client missed the response.
    recent.push_back(CachedReply { id, request, bytes });
    while recent.len() > 64
        || recent.iter().map(|reply| reply.bytes.len()).sum::<usize>() > 8 * 1024 * 1024
    {
        recent.pop_front();
    }
    sent
}

fn send_reply(stream: &mut TcpStream, bytes: &[u8], legacy: bool) -> io::Result<()> {
    let deadline = Instant::now() + IO_TIMEOUT;
    if legacy {
        protocol::write_until(stream, bytes, deadline)
    } else {
        protocol::write_frame(stream, bytes, deadline)
    }
}

fn exchange(
    directory: &Path,
    request: &Request,
    id: &[u8; 16],
    deadline: Instant,
) -> io::Result<Response> {
    let endpoint = fs::read(directory.join("endpoint"))?;
    if endpoint.len() != 38 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Incomplete instance endpoint",
        ));
    }
    let address = SocketAddr::from((
        Ipv4Addr::LOCALHOST,
        u16::from_be_bytes([endpoint[0], endpoint[1]]),
    ));
    let remaining = deadline.saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "Request deadline elapsed",
        ));
    }
    let mut stream = TcpStream::connect_timeout(&address, remaining.min(IO_TIMEOUT))?;
    #[cfg(windows)]
    if matches!(request, Request::OpenHud | Request::OpenDashboard) {
        let pid = u32::from_be_bytes(endpoint[2..6].try_into().unwrap());
        unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::AllowSetForegroundWindow(pid);
        }
    }
    let send_deadline = deadline.min(Instant::now() + IO_TIMEOUT);
    protocol::write_until(&mut stream, protocol::MAGIC, send_deadline)?;
    protocol::write_until(&mut stream, &endpoint[6..], send_deadline)?;
    protocol::write_until(&mut stream, id, send_deadline)?;
    let bytes = serde_json::to_vec(request).map_err(io::Error::other)?;
    protocol::write_frame(&mut stream, &bytes, send_deadline)?;
    let bytes = protocol::read_frame(
        &mut stream,
        protocol::MAX_RESPONSE,
        deadline.min(Instant::now() + protocol::RESPONSE_TIMEOUT),
    )?;
    serde_json::from_slice(&bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

#[cfg(test)]
mod tests;

impl Drop for Instance {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        // Wake the blocking accept so shutdown doesn't leave a listener thread behind.
        let _ = TcpStream::connect_timeout(&self.address, IO_TIMEOUT);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        // Leave the lock file in place: deleting it could let two launches lock different files.
    }
}
