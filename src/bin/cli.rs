use clap::Parser;
use spacearth_dtn::bundle::*;
use spacearth_dtn::cla::manager::ClaManager;
use spacearth_dtn::config::{generate_creation_timestamp, Config};
use spacearth_dtn::store::BundleStore;
use std::sync::Arc;

#[derive(Parser)]
struct Opts {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Parser)]
enum Command {
    Insert {
        #[clap(short, long)]
        message: String,
    },
    List,
    Show {
        #[clap(short, long)]
        id: String,
    },
    Receive,
    Daemon {
        #[clap(subcommand)]
        cmd: DaemonCmd,
    },
    Cleanup,
}

#[derive(Parser)]
enum DaemonCmd {
    Listener {
        #[clap(long)]
        addr: String,
    },
    Dialer {
        #[clap(long)]
        addr: String,
    },
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let opts = Opts::parse();

    let store = BundleStore::new("./bundles")?;

    match opts.cmd {
        Command::Insert { message } => {
            handle_insert(&store, message)?;
        }

        Command::List => {
            let bundles = store.list()?;
            for id in bundles {
                println!("📦 {id}");
            }
        }

        Command::Show { id } => {
            let bundle = store.load_by_partial_id(&id)?;
            println!("📄 ID: {}", id);
            println!("  Source: {}", bundle.primary.source);
            println!("  Destination: {}", bundle.primary.destination);
            println!("  Message: {}", String::from_utf8_lossy(&bundle.payload));
        }

        Command::Receive => {
            todo!();
        }

        Command::Daemon { cmd } => match cmd {
            DaemonCmd::Listener { addr } => {
                let cla = Arc::new(spacearth_dtn::cla::TcpClaListener {
                    bind_addr: addr,
                    receive_callback: Arc::new(|bundle| {
                        if let Err(e) = (|| -> anyhow::Result<()> {
                            let store = BundleStore::new("./bundles")?;
                            store.insert(&bundle)?;
                            Ok(())
                        })() {
                            eprintln!("❌ Failed to insert bundle: {e}");
                        }
                    }),
                });
                let manager = ClaManager::new(|bundle| {
                    println!("📥 Received: {:?}", bundle);
                });
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()?
                    .block_on(async {
                        manager.register(cla).await;
                        futures::future::pending::<()>().await;
                    });
            }

            DaemonCmd::Dialer { addr } => {
                let cla = Arc::new(spacearth_dtn::cla::TcpClaDialer { target_addr: addr });
                let manager = ClaManager::new(|bundle| {
                    println!("📤 Should not receive here (Dialer): {:?}", bundle);
                });
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()?
                    .block_on(async {
                        manager.register(cla).await;
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    });
            }
        },

        Command::Cleanup => {
            store.cleanup_expired()?;
        }
    }

    Ok(())
}

fn handle_insert(store: &BundleStore, message: String) -> anyhow::Result<()> {
    let config = Config::load()?;
    let bundle = Bundle {
        primary: PrimaryBlock {
            version: config.bundle.version,
            destination: config.endpoints.destination,
            source: config.endpoints.source,
            report_to: config.endpoints.report_to,
            creation_timestamp: generate_creation_timestamp(),
            lifetime: config.bundle.lifetime,
        },
        payload: message.into_bytes(),
    };

    store.insert(&bundle)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use tempfile::TempDir;

    #[test]
    fn test_opts_parse_insert() {
        let args = vec!["test-bin", "insert", "--message", "hello"];
        let opts = Opts::parse_from(args);
        match opts.cmd {
            Command::Insert { message } => assert_eq!(message, "hello"),
            _ => panic!("Unexpected command"),
        }
    }

    #[test]
    fn test_opts_parse_list() {
        let args = vec!["test-bin", "list"];
        let opts = Opts::parse_from(args);
        match opts.cmd {
            Command::List => {}
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn test_opts_parse_show() {
        let args = vec!["test-bin", "show", "--id", "abc123"];
        let opts = Opts::parse_from(args);
        match opts.cmd {
            Command::Show { id } => assert_eq!(id, "abc123"),
            _ => panic!("Expected Show command"),
        }
    }

    #[test]
    fn test_opts_parse_cleanup() {
        let args = vec!["test-bin", "cleanup"];
        let opts = Opts::parse_from(args);
        match opts.cmd {
            Command::Cleanup => {}
            _ => panic!("Expected Cleanup command"),
        }
    }

    #[test]
    fn test_opts_parse_daemon_listener() {
        let args = vec!["test-bin", "daemon", "listener", "--addr", "127.0.0.1:8080"];
        let opts = Opts::parse_from(args);
        match opts.cmd {
            Command::Daemon {
                cmd: DaemonCmd::Listener { addr },
            } => {
                assert_eq!(addr, "127.0.0.1:8080");
            }
            _ => panic!("Expected Daemon Listener command"),
        }
    }

    #[test]
    fn test_opts_parse_daemon_dialer() {
        let args = vec!["test-bin", "daemon", "dialer", "--addr", "127.0.0.1:8080"];
        let opts = Opts::parse_from(args);
        match opts.cmd {
            Command::Daemon {
                cmd: DaemonCmd::Dialer { addr },
            } => {
                assert_eq!(addr, "127.0.0.1:8080");
            }
            _ => panic!("Expected Daemon Dialer command"),
        }
    }

    #[test]
    fn test_handle_insert() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let store = BundleStore::new(temp_dir.path())?;
        let result = handle_insert(&store, "test message".to_string());
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_partial_lookup() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let store = BundleStore::new(temp_dir.path())?;
        let bundle = Bundle {
            primary: PrimaryBlock {
                version: 7,
                destination: "dtn://dest".into(),
                source: "dtn://src".into(),
                report_to: "dtn://report".into(),
                creation_timestamp: 12345,
                lifetime: 3600,
            },
            payload: b"test".to_vec(),
        };
        store.insert(&bundle)?;

        let id_full = store
            .list()?
            .first()
            .expect("expected at least one bundle")
            .clone();
        let id_partial = &id_full[..8];

        let loaded = store.load_by_partial_id(id_partial)?;
        assert_eq!(loaded.payload, b"test");
        Ok(())
    }

    #[test]
    fn test_handle_insert_with_various_messages() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let store = BundleStore::new(temp_dir.path())?;

        // Test different message types
        let messages = vec![
            "simple message",
            "メッセージ with unicode",
            "message with numbers 123456",
            "",  // empty message
            "very long message that contains a lot of text to test if the system can handle longer messages properly",
        ];

        for (i, msg) in messages.iter().enumerate() {
            let result = handle_insert(&store, msg.to_string());
            assert!(result.is_ok(), "Failed to insert message: {}", msg);

            // Add a small delay to ensure different timestamps
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        // Verify bundles were inserted
        let bundles = store.list()?;
        assert!(
            bundles.len() >= 1,
            "Expected at least 1 bundle, got {}",
            bundles.len()
        );
        Ok(())
    }

    #[test]
    fn test_command_list_functionality() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let store = BundleStore::new(temp_dir.path())?;

        // Insert a test bundle
        handle_insert(&store, "test for list".to_string())?;

        // Test list command logic
        let bundles = store.list()?;
        assert!(!bundles.is_empty());

        // Simulate printing (we can't test actual stdout easily)
        for id in bundles {
            assert!(!id.is_empty());
        }
        Ok(())
    }

    #[test]
    fn test_command_show_functionality() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let store = BundleStore::new(temp_dir.path())?;

        // Insert a test bundle
        handle_insert(&store, "test message for show".to_string())?;

        // Get the bundle ID
        let bundles = store.list()?;
        let id = bundles.first().unwrap();
        let partial_id = &id[..8];

        // Test show command logic
        let bundle = store.load_by_partial_id(partial_id)?;
        assert_eq!(bundle.payload, b"test message for show");
        assert!(!bundle.primary.source.is_empty());
        assert!(!bundle.primary.destination.is_empty());
        Ok(())
    }

    #[test]
    fn test_command_cleanup_functionality() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let store = BundleStore::new(temp_dir.path())?;

        // Insert test bundles with delays to ensure different timestamps
        handle_insert(&store, "test cleanup 1".to_string())?;
        std::thread::sleep(std::time::Duration::from_millis(1));
        handle_insert(&store, "test cleanup 2".to_string())?;

        let bundles_before = store.list()?;
        assert!(
            bundles_before.len() >= 1,
            "Expected at least 1 bundle, got {}",
            bundles_before.len()
        );

        // Test cleanup command logic (should work even if no expired bundles)
        let result = store.cleanup_expired();
        assert!(result.is_ok());

        // Bundles should still be there (not expired)
        let bundles_after = store.list()?;
        assert!(
            bundles_after.len() >= 1,
            "Expected at least 1 bundle after cleanup, got {}",
            bundles_after.len()
        );
        Ok(())
    }

    #[test]
    fn test_bundle_creation_consistency() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let store = BundleStore::new(temp_dir.path())?;

        let message = "consistency test";
        handle_insert(&store, message.to_string())?;

        let bundles = store.list()?;
        let bundle = store.load_by_partial_id(&bundles[0][..8])?;

        // Verify bundle structure
        assert_eq!(bundle.primary.version, 7);
        assert_eq!(String::from_utf8_lossy(&bundle.payload), message);
        assert!(bundle.primary.creation_timestamp > 0);
        assert!(bundle.primary.lifetime > 0);
        Ok(())
    }

    #[test]
    fn test_show_command_with_nonexistent_id() {
        let temp_dir = TempDir::new().unwrap();
        let store = BundleStore::new(temp_dir.path()).unwrap();

        // Try to show a bundle that doesn't exist
        let result = store.load_by_partial_id("nonexistent");
        assert!(result.is_err());
    }
}
