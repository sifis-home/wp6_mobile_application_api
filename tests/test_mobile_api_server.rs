use assert_cmd::cargo::cargo_bin;
use assert_cmd::prelude::*;
use mobile_api::SifisHome;
use nix::sys::signal;
use nix::sys::signal::Signal;
use nix::unistd::Pid;
use predicates::prelude::*;
use rocket::form::validate::Contains;
use std::error::Error;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::thread::{sleep, JoinHandle};
use std::time::Duration;
use tempfile::TempDir;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::TryRecvError;

const SERVER_NAME: &str = "mobile_api_server";

// Test ignored for miri, because file operations are not available when isolation is enabled.
#[cfg_attr(miri, ignore)]
#[tokio::test]
async fn test_server_binary() -> Result<(), Box<dyn Error>> {
    // Making temporary directory for testing
    let tmp_dir = TempDir::new()?;
    let mut tmp_sifis_home_path = PathBuf::from(tmp_dir.path());
    tmp_sifis_home_path.push("sifis-home");
    std::fs::create_dir_all(&tmp_sifis_home_path).unwrap();

    // Using our custom environment settings for testing the server binary
    std::env::set_var("SIFIS_HOME_PATH", tmp_sifis_home_path.into_os_string());
    std::env::set_var("ROCKET_ADDRESS", "127.0.0.1");
    std::env::set_var("ROCKET_PORT", "28000");

    // First launch should fail because device.json is missing
    let mut command = Command::cargo_bin(SERVER_NAME)?;
    command
        .assert()
        .failure()
        // Should tell that device.json not found
        .stderr(
            predicate::str::is_match("^Device information file.*device\\.json.*not found\\.")
                .unwrap(),
        )
        // Should tell about create_device_info app
        .stderr(predicate::str::contains(
            "You can use create_device_info application to create it.",
        ));

    // Server should shut down gracefully with SIGTERM signal
    tokio::select! {
        result = test_graceful_shutdown() => {
            assert!(result.is_ok());
        }
        _ = tokio::time::sleep(Duration::from_secs(10)) => {
            panic!("The server graceful shutdown check timed out");
        }
    }
    Ok(())
}

async fn test_graceful_shutdown() -> Result<(), Box<dyn Error>> {
    // Running with valgrind?
    if let Ok(value) = std::env::var("LD_PRELOAD") {
        if value.contains("/valgrind/") || value.contains("/vgpreload") {
            println!("The graceful shutdown test was skipped because we cannot send SIGTERM for");
            println!("the server when it is run within the Valgrind checking tool.");
            return Ok(());
        }
    }

    // Saving a test device.json to start the server
    let sifis_home = SifisHome::new();
    let device_info = sifis_home.new_info("Test".to_string()).unwrap();
    sifis_home.save_info(&device_info).unwrap();

    // This test does the following:
    //
    // 1. Look for "Rocket has launched from" message from the server
    // 2. Send SIGTERM signal for the server
    // 3. Wait for the server to shutdown
    // 4. Check for graceful shutdown

    // Starting server process now
    let server_bin_path = cargo_bin(SERVER_NAME);
    let mut server = tokio::process::Command::new(&server_bin_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let stdout = server.stdout.take().unwrap();
    let stderr = server.stderr.take().unwrap();
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();
    let server_pid = Pid::from_raw(server.id().unwrap() as i32);
    let mut sigterm_thread_handle: Option<JoinHandle<()>> = None;

    // A message channel for telling the SIGTERM sender to stop
    let (sigterm_tx, _) = broadcast::channel::<()>(1);

    // Asynchronous processing until server gracefully shutdowns or something goes wrong
    loop {
        tokio::select! {
            result = stdout_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        println!("stdout: {}", line);
                        if line.contains("Rocket has launched from") {
                            // Server started successfully, start sending SIGTERM for it
                            let rx = sigterm_tx.subscribe();
                            sigterm_thread_handle = Some(thread::spawn(move || {
                                sigterm_sender(server_pid, rx);
                            }));
                        } else if line.contains("Graceful shutdown completed successfully") {
                            // Test went okay, we can break out of the loop
                            break;
                        }
                    },
                    Err(_) => panic!("Stdout reader error"),
                    _ => (),
                }
            }
            result = stderr_reader.next_line() => {
                match result {
                    Ok(Some(line)) => panic!("Unexpected stderr output: {}", line),
                    Err(_) => panic!("Stderr reader error"),
                    _ => (),
                }
            }
        }
    }

    // Tell SIGTERM sender to stop and wait it to complete
    sigterm_tx.send(()).unwrap();
    if let Some(handle) = sigterm_thread_handle {
        handle.join().unwrap();
    }

    // This point is only reached if server output contained message about graceful shutdown.

    // Checking that the exit code is 0
    let exit_status = server.wait().await?;
    assert_eq!(exit_status.code().unwrap(), 0);

    Ok(())
}

fn sigterm_sender(pid: Pid, mut rx: broadcast::Receiver<()>) {
    sleep(Duration::from_millis(100));
    loop {
        println!("Sending SIGTERM for process {}", pid);
        signal::kill(pid, Some(Signal::SIGTERM)).expect("Could not send SIGTERM");
        sleep(Duration::from_millis(100));
        match rx.try_recv() {
            Err(TryRecvError::Empty) => (), // Try sending SIGTERM again
            _ => break,
        };
    }
}
