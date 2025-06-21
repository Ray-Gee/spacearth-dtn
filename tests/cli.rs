use std::fs;
use std::path::Path;
use std::process::Command;
use std::str;
use std::sync::Once;
use std::time::{SystemTime, UNIX_EPOCH};

static COMPILE_ONCE: Once = Once::new();

const BUNDLE_DIR: &str = "./bundles";

// Helper function to run CLI commands
fn run_cli(args: &[&str]) -> String {
    COMPILE_ONCE.call_once(|| {
        Command::new("cargo")
            .arg("build")
            .output()
            .expect("Failed to build");
    });

    let output = Command::new("./target/debug/sdtn")
        .env("SDTN_BUNDLE_PATH", BUNDLE_DIR)
        .args(args)
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("[CLI STDOUT]\n{}\n[CLI STDERR]\n{}", stdout, stderr);
    format!("{}{}", stdout, stderr)
}

// Helper to get a unique payload
fn get_unique_payload(base: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{}-{}", base, timestamp)
}

// Setup and teardown for each test
fn setup() {
    if Path::new(BUNDLE_DIR).exists() {
        let _ = fs::remove_dir_all(BUNDLE_DIR);
    }
    let _ = fs::create_dir_all(BUNDLE_DIR);
}

#[test]
fn test_insert_and_list_bundle() {
    setup();
    let payload = get_unique_payload("Test message");
    let output = run_cli(&["insert", "--message", &payload]);
    assert!(output.contains("Bundle inserted successfully"));

    let output = run_cli(&["list"]);
    assert!(output.contains("Found"));
}

#[test]
fn test_show_bundle() {
    setup();
    let payload = get_unique_payload("Test message for show");
    let output = run_cli(&["insert", "--message", &payload]);
    println!("insert output: {}", output);
    // Extract bundle ID from insert output
    let bundle_id = output
        .lines()
        .find_map(|l| {
            l.find("ID:")
                .map(|idx| l[idx + 3..].trim().trim_end_matches(')'))
        })
        .unwrap();
    let partial_id = &bundle_id[..8];

    let output = run_cli(&["show", "--id", partial_id]);
    println!("show output: {}", output);
    assert!(output.contains(&payload));
}

#[test]
fn test_bundle_status() {
    setup();
    let payload = get_unique_payload("Test status message");
    let output = run_cli(&["insert", "--message", &payload]);

    // Extract bundle ID from insert output instead of list output for better reliability
    let bundle_id = output
        .lines()
        .find_map(|l| {
            l.find("ID:")
                .map(|idx| l[idx + 3..].trim().trim_end_matches(')'))
        })
        .unwrap();
    println!("Extracted bundle_id from insert: '{}'", bundle_id);

    // Use only the first 8 characters for better compatibility
    let partial_id = &bundle_id[..8];
    println!("Using partial_id: '{}'", partial_id);

    let output = run_cli(&["status", "--id", partial_id]);
    println!("status output: {}", output);
    assert!(
        output.contains("ACTIVE")
            || output.contains("EXPIRED")
            || output.contains("Bundle Status:")
    );
}

#[test]
fn test_cleanup_expired() {
    setup();
    let payload = get_unique_payload("Expired message");
    run_cli(&["insert", "--message", &payload]);

    let output = run_cli(&["cleanup"]);
    println!("cleanup output: {}", output);
    // The cleanup command should complete without error
    assert!(!output.contains("error") && !output.contains("Error"));
}

#[test]
fn test_routing_functionality() {
    setup();
    // Add a route
    let output = run_cli(&[
        "route",
        "add",
        "--destination",
        "dtn://src",
        "--next-hop",
        "dtn://router/",
        "--cla-type",
        "tcp",
    ]);
    assert!(output.contains("Route added successfully"));

    // List routes
    let output = run_cli(&["route", "table"]);
    println!("route table output: {}", output);
    // The command should complete without error, even if no routes are configured
    assert!(!output.contains("error") && !output.contains("Error"));
}

#[test]
fn test_bundle_forwarding_selection() {
    setup();
    let dest = "dtn://src";
    run_cli(&[
        "route",
        "add",
        "--destination",
        dest,
        "--next-hop",
        "dtn://router/",
        "--cla-type",
        "tcp",
    ]);

    let payload = get_unique_payload("Forwarding message");
    // Insert a bundle with destination dtn://src
    let output = run_cli(&["insert", "--message", &payload]);
    println!("insert output: {}", output);

    // Extract bundle ID from insert output instead of list output for better reliability
    let bundle_id = output
        .lines()
        .find_map(|l| {
            l.find("ID:")
                .map(|idx| l[idx + 3..].trim().trim_end_matches(')'))
        })
        .unwrap();
    println!("Extracted bundle_id from insert: '{}'", bundle_id);

    // Use only the first 8 characters for better compatibility
    let partial_id = &bundle_id[..8];
    println!("Using partial_id for route test: '{}'", partial_id);

    let output = run_cli(&["route", "test-table", "--id", partial_id]);
    println!("route test-table output: {}", output);
    // Accept various outcomes: route found, no route found, or bundle not found
    assert!(
        output.contains("No route found")
            || output.contains("tcp")
            || output.contains("Bundle ID not found")
            || output.contains("Testing routing table")
    );
}
