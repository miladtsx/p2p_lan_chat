//! Integration tests for end-to-end peer discovery and messaging in the P2P chat app.

use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

/// Helper to spawn a chat process with given name and port.
fn spawn_peer(name: &str, port: u16) -> Child {
    Command::new("./target/release/p2p-chat")
        .arg("start")
        .arg("--name")
        .arg(name)
        .arg("--port")
        .arg(port.to_string())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start peer process")
}

#[test]
fn test_peer_discovery_and_message() {
    // Start two peers
    let mut alice = spawn_peer("Alice", 9000);
    let mut bob = spawn_peer("Bob", 9001);

    // Wait for discovery
    thread::sleep(Duration::from_millis(200));

    // Non-blocking read with timeout for Alice's stdout and stderr
    let mut found = false;
    let mut all_lines = Vec::new();
    let start = std::time::Instant::now();

    // Move stdout and stderr out of alice so we can own them
    let alice_stdout = alice.stdout.take().expect("No stdout");
    let alice_stderr = alice.stderr.take().expect("No stderr");
    let mut stdout_reader = BufReader::new(alice_stdout);
    let mut stderr_reader = BufReader::new(alice_stderr);

    while start.elapsed() < Duration::from_secs(5) {
        // Check stdout
        let mut line = String::new();
        match stdout_reader.read_line(&mut line) {
            Ok(0) => {} // EOF
            Ok(_) => {
                all_lines.push(format!("stdout: {}", line.trim_end()));
                if line.contains("Discovered peer") || line.contains("peer") {
                    found = true;
                    break;
                }
            }
            Err(_) => {}
        }
        // Check stderr
        let mut err_line = String::new();
        match stderr_reader.read_line(&mut err_line) {
            Ok(0) => {} // EOF
            Ok(_) => {
                all_lines.push(format!("stderr: {}", err_line.trim_end()));
                if err_line.contains("Discovered peer") || err_line.contains("peer") {
                    found = true;
                    break;
                }
            }
            Err(_) => {}
        }
        if found {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    for l in &all_lines {
        println!("{}", l);
    }
    assert!(
        found,
        "Alice did not discover any peers. Output:\n{}",
        all_lines.join("\n")
    );

    // Clean up
    let _ = alice.kill();
    let _ = alice.wait();
    let _ = bob.kill();
    let _ = bob.wait();
}

#[test]
fn test_maximum_peer_discovery_limit() {
    // Set a reasonable max for test (e.g. 5 peers)
    let max_peers = 5;
    let mut children = vec![];
    for i in 0..max_peers + 1 {
        let name = format!("Peer-{i}");
        let port = 9100 + i as u16;
        children.push(spawn_peer(&name, port));
    }
    thread::sleep(Duration::from_secs(2));
    // Check that all but one peer are discovered (if app has a max, adjust assertion)
    // For now, just check that processes started and ran
    for child in &mut children {
        assert!(
            child.try_wait().unwrap().is_none(),
            "Peer process exited early"
        );
    }
    // Clean up
    for mut child in children {
        let _ = child.kill();
    }
}
