use super::*;
use std::{
    path::PathBuf,
    process::{Child, Command as ProcessCommand, Stdio},
};

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        let mut id = [0u8; 8];
        getrandom::fill(&mut id).unwrap();
        let path =
            std::env::temp_dir().join(format!("fifo-instance-test-{:x}", u64::from_ne_bytes(id)));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

// A failed assertion should not leave a helper app camping in the background.
struct TestChild(Child);
impl Drop for TestChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn child(directory: &Path) -> TestChild {
    TestChild(
        ProcessCommand::new(std::env::current_exe().unwrap())
            .args(["--exact", "instance::tests::process_helper", "--nocapture"])
            .env("FIFO_INSTANCE_TEST_DIR", directory)
            .stdout(Stdio::null())
            .spawn()
            .unwrap(),
    )
}

fn wait_for(mut ready: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(8);
    while !ready() {
        assert!(
            Instant::now() < deadline,
            "Timed out waiting for test process"
        );
        thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn forwards_before_gui_attach_and_restarts_with_stale_metadata() {
    let directory = TestDirectory::new();
    let owner = Instance::acquire_in(&directory.0, Duration::from_secs(1))
        .unwrap()
        .unwrap();
    assert!(
        Instance::acquire_in(&directory.0, Duration::from_secs(1))
            .unwrap()
            .is_none()
    );
    assert!(matches!(
        owner.commands.recv_timeout(Duration::from_secs(1)).unwrap(),
        Command::OpenDashboard
    ));
    drop(owner);
    // Old endpoint files must not prevent a new owner after normal shutdown.
    assert!(
        Instance::acquire_in(&directory.0, Duration::from_secs(1))
            .unwrap()
            .is_some()
    );
}

#[test]
fn later_launch_waits_for_owner_to_publish_endpoint() {
    let directory = TestDirectory::new();
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(directory.0.join("owner.lock"))
        .unwrap();
    lock.lock().unwrap();
    let path = directory.0.clone();
    let later = thread::spawn(move || {
        Instance::acquire_in(&path, Duration::from_secs(2))
            .unwrap()
            .is_none()
    });
    thread::sleep(Duration::from_millis(100));
    let owner = Instance::primary(lock, &directory.0).unwrap();
    assert!(later.join().unwrap());
    assert!(matches!(
        owner.commands.recv_timeout(Duration::from_secs(1)).unwrap(),
        Command::OpenDashboard
    ));
}

#[test]
fn stalled_and_invalid_clients_do_not_break_delivery() {
    let directory = TestDirectory::new();
    let owner = Instance::acquire_in(&directory.0, Duration::from_secs(1))
        .unwrap()
        .unwrap();
    let _stalled = TcpStream::connect(owner.address).unwrap();
    let mut bad = TcpStream::connect(owner.address).unwrap();
    bad.write_all(&[0u8; 53]).unwrap();
    assert!(
        Instance::acquire_in(&directory.0, Duration::from_secs(2))
            .unwrap()
            .is_none()
    );
    assert!(matches!(
        owner.commands.recv_timeout(Duration::from_secs(1)).unwrap(),
        Command::OpenDashboard
    ));
    assert!(owner.next_command().is_none());
}

#[test]
fn lost_acknowledgement_retry_does_not_deliver_twice() {
    let directory = TestDirectory::new();
    let owner = Instance::acquire_in(&directory.0, Duration::from_secs(1))
        .unwrap()
        .unwrap();
    let endpoint = fs::read(directory.0.join("endpoint")).unwrap();
    let id = [42u8; 16];
    let mut stream = TcpStream::connect(owner.address).unwrap();
    stream.write_all(MAGIC).unwrap();
    stream.write_all(&endpoint[6..]).unwrap();
    stream.write_all(&id).unwrap();
    drop(stream); // Pretend the client never received the acknowledgement.
    Instance::notify(&directory.0, &id).unwrap();
    assert!(matches!(
        owner.commands.recv_timeout(Duration::from_secs(1)).unwrap(),
        Command::OpenDashboard
    ));
    assert!(owner.next_command().is_none());
}

#[test]
fn unresponsive_owner_times_out_without_creating_another_instance() {
    let directory = TestDirectory::new();
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(directory.0.join("owner.lock"))
        .unwrap();
    lock.lock().unwrap();
    let started = Instant::now();
    let error = Instance::acquire_in(&directory.0, Duration::from_millis(150))
        .err()
        .unwrap();
    assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    assert!(started.elapsed() < Duration::from_secs(2));
    assert!(!directory.0.join("endpoint").exists());
}

#[test]
fn simultaneous_processes_elect_one_owner_and_deliver_every_launch() {
    let directory = TestDirectory::new();
    let mut children: Vec<_> = (0..8).map(|_| child(&directory.0)).collect();
    wait_for(|| fs::read(directory.0.join("received")).is_ok_and(|bytes| bytes.len() == 7));
    let owners = fs::read_dir(&directory.0)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().starts_with("owner-"))
        .count();
    assert_eq!(owners, 1);
    fs::write(directory.0.join("release"), b"done").unwrap();
    for child in &mut children {
        wait_for(|| child.0.try_wait().unwrap().is_some());
        assert!(child.0.wait().unwrap().success());
    }
}

#[test]
fn killed_owner_releases_lock_without_manual_cleanup() {
    let directory = TestDirectory::new();
    let mut owner = child(&directory.0);
    wait_for(|| directory.0.join("ready").exists());
    owner.0.kill().unwrap();
    owner.0.wait().unwrap();
    assert!(
        Instance::acquire_in(&directory.0, Duration::from_secs(1))
            .unwrap()
            .is_some()
    );
}

#[test]
fn process_helper() {
    let Some(directory) = std::env::var_os("FIFO_INSTANCE_TEST_DIR") else {
        return;
    };
    let directory = PathBuf::from(directory);
    let Some(owner) = Instance::acquire_in(&directory, Duration::from_secs(4)).unwrap() else {
        return;
    };
    fs::write(
        directory.join(format!("owner-{}", std::process::id())),
        b"owner",
    )
    .unwrap();
    fs::write(directory.join("ready"), b"ready").unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut received = OpenOptions::new()
        .create(true)
        .append(true)
        .open(directory.join("received"))
        .unwrap();
    while !directory.join("release").exists() {
        assert!(
            Instant::now() < deadline,
            "Parent did not release test helper"
        );
        while let Some(command) = owner.next_command() {
            assert!(matches!(command, Command::OpenDashboard));
            received.write_all(b"x").unwrap();
        }
        thread::sleep(Duration::from_millis(10));
    }
}
