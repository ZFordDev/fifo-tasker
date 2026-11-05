use std::{
    io,
    path::{Path, PathBuf},
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, SyncSender},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use eframe::egui;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use crate::{logic::tasks::TaskData, storage};

pub(crate) struct TaskWatcher {
    _watcher: RecommendedWatcher,
    trigger: SyncSender<()>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
    reload_handle: ReloadHandle,
}

type ReloadResult = Result<u64, String>;

#[derive(Clone)]
pub(crate) struct ReloadHandle {
    requests: SyncSender<SyncSender<ReloadResult>>,
    trigger: SyncSender<()>,
    stop: Arc<AtomicBool>,
}

impl ReloadHandle {
    pub(crate) fn is_active(&self) -> bool {
        !self.stop.load(Ordering::Acquire)
    }

    pub(crate) fn reload(&self, timeout: Duration) -> ReloadResult {
        if !self.is_active() {
            return Err("Task loader is shutting down".into());
        }
        let (reply, result) = mpsc::sync_channel(1);
        self.requests
            .try_send(reply)
            .map_err(|error| format!("Cannot queue reload: {error}"))?;
        let _ = self.trigger.try_send(());
        result.recv_timeout(timeout).map_err(|error| {
            format!("Reload did not complete: {error}; its outcome may be unknown")
        })?
    }
}

impl TaskWatcher {
    pub(crate) fn new(
        path: PathBuf,
        data: Arc<RwLock<TaskData>>,
        ctx: egui::Context,
    ) -> io::Result<Self> {
        let parent = path
            .parent()
            .ok_or_else(|| io::Error::other("Task file has no parent directory"))?;
        let (trigger, events) = mpsc::sync_channel(1);
        let callback_trigger = trigger.clone();
        let callback_data = Arc::clone(&data);
        let callback_ctx = ctx.clone();
        let mut watcher =
            notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
                match event {
                    Ok(event) if !event.kind.is_access() => {
                        // The directory also catches editors that replace the file on save.
                        let recovered = callback_data.write().unwrap().watch_error.take().is_some();
                        if recovered {
                            repaint(&callback_ctx);
                        }
                        let _ = callback_trigger.try_send(()); // Coalesce bursts, not a queue of every keystroke.
                    }
                    Ok(_) => {}
                    Err(error) => {
                        callback_data.write().unwrap().watch_error = Some(error.to_string());
                        repaint(&callback_ctx);
                    }
                }
            })
            .map_err(io::Error::other)?;
        watcher
            .watch(parent, RecursiveMode::NonRecursive)
            .map_err(io::Error::other)?;

        let stop = Arc::new(AtomicBool::new(false));
        let (requests, reload_requests) = mpsc::sync_channel::<SyncSender<ReloadResult>>(8);
        let reload_handle = ReloadHandle {
            requests,
            trigger: trigger.clone(),
            stop: Arc::clone(&stop),
        };
        let worker_stop = Arc::clone(&stop);
        let worker = thread::Builder::new()
            .name("task-file-reload".into())
            .spawn(move || {
                while events.recv().is_ok() {
                    if worker_stop.load(Ordering::Acquire) {
                        break;
                    }
                    // Let a save finish, but don't postpone updates forever during frequent writes.
                    let deadline = Instant::now() + Duration::from_secs(1);
                    loop {
                        let remaining = deadline.saturating_duration_since(Instant::now());
                        if remaining.is_zero()
                            || events
                                .recv_timeout(remaining.min(Duration::from_millis(180)))
                                .is_err()
                        {
                            break;
                        }
                        if worker_stop.load(Ordering::Acquire) {
                            return;
                        }
                    }
                    if worker_stop.load(Ordering::Acquire) {
                        break;
                    }
                    // Only acknowledge requests that arrived before this read began.
                    let replies: Vec<_> = reload_requests.try_iter().collect();
                    let (changed, result) = reload(&path, &data);
                    if changed {
                        repaint(&ctx);
                    }
                    for reply in replies {
                        let _ = reply.send(result.clone());
                    }
                }
            })?;
        let _ = trigger.try_send(()); // Close the gap between startup loading and subscribing.
        Ok(Self {
            _watcher: watcher,
            trigger,
            stop,
            worker: Some(worker),
            reload_handle,
        })
    }

    pub(crate) fn reload_handle(&self) -> ReloadHandle {
        self.reload_handle.clone()
    }
}

fn reload(path: &Path, shared: &RwLock<TaskData>) -> (bool, ReloadResult) {
    let loaded = storage::load(path); // File I/O and validation never hold the shared-state lock.
    let mut data = shared.write().unwrap();
    let error = match loaded {
        Ok(Some(document)) => {
            let changed = data.document != document || data.revision == 0;
            if changed {
                data.document = document;
                data.revision = data.revision.saturating_add(1);
            }
            let recovered = data.load_error.take().is_some();
            return (changed || recovered, Ok(data.revision));
        }
        Ok(None) if data.revision == 0 && data.load_error.is_none() => {
            return (
                false,
                Err(format!("Task data file is missing: {}", path.display())),
            );
        }
        Ok(None) => format!("Task data file is missing: {}", path.display()),
        Err(error) => error.to_string(),
    };
    if data.load_error.as_ref() == Some(&error) {
        return (false, Err(error));
    }
    data.load_error = Some(error.clone());
    (true, Err(error))
}

fn repaint(ctx: &egui::Context) {
    let viewports = ctx.input(|input| input.raw.viewports.keys().copied().collect::<Vec<_>>());
    for id in viewports {
        ctx.request_repaint_of(id);
    }
    ctx.request_repaint_of(egui::ViewportId::ROOT); // The owner also stays current while hidden.
}

impl Drop for TaskWatcher {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        let _ = self.trigger.try_send(());
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
