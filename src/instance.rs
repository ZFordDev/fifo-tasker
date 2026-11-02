use std::{
    collections::VecDeque,
    fs::{self, File, OpenOptions, TryLockError},
    io::{self, Read, Write},
    net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use crate::logic::Command;
use eframe::egui;

const STARTUP_TIMEOUT: Duration = Duration::from_secs(10);
const IO_TIMEOUT: Duration = Duration::from_millis(300);
const MAGIC: &[u8; 5] = b"FIFO1";

pub(crate) struct Instance {
    commands: Receiver<Command>,
    wake: Arc<Mutex<Option<egui::Context>>>,
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
                    let accepted = (|| -> io::Result<()> {
                        stream.set_read_timeout(Some(IO_TIMEOUT))?;
                        stream.set_write_timeout(Some(IO_TIMEOUT))?;
                        let mut request = [0u8; 53];
                        stream.read_exact(&mut request)?;
                        if &request[..5] != MAGIC || request[5..37] != token {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "Invalid launch request",
                            ));
                        }
                        let id: [u8; 16] = request[37..].try_into().unwrap();
                        // A lost acknowledgement shouldn't make the same launch knock twice.
                        if !recent_requests.contains(&id) {
                            sender
                                .send(Command::OpenDashboard)
                                .map_err(|_| io::Error::other("Application is shutting down"))?;
                            recent_requests.push_back(id);
                            if recent_requests.len() > 64 {
                                recent_requests.pop_front();
                            }
                            if let Some(ctx) = worker_wake.lock().unwrap().as_ref() {
                                ctx.request_repaint_of(egui::ViewportId::ROOT);
                            }
                        }
                        // Acknowledge queueing, not merely connecting to something on this port.
                        stream.write_all(b"OK")
                    })();
                    // Bad or abandoned connections must not take the listener down with them.
                    let _ = accepted;
                }
            })?;
        Ok(Self {
            commands,
            wake,
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

    pub(crate) fn next_command(&self) -> Option<Command> {
        self.commands.try_recv().ok()
    }
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
