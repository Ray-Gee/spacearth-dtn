use clap::Parser;
use spacearth_dtn::bundle::*;
use spacearth_dtn::config::{Config, generate_creation_timestamp};
use spacearth_dtn::store::BundleStore;

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

        let id_full = store.list()?.first().unwrap().clone();
        let id_partial = &id_full[..8];

        let loaded = store.load_by_partial_id(id_partial)?;
        assert_eq!(loaded.payload, b"test");
        Ok(())
    }
}
