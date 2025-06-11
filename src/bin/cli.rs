use clap::Parser;
use spacearth_dtn::bundle::*;
use spacearth_dtn::config::{Config, generate_creation_timestamp};

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
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let opts = Opts::parse();

    match opts.cmd {
        Command::Insert { message } => {
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

            let encoded = serde_cbor::to_vec(&bundle)?;
            std::fs::write("bundle.cbor", encoded)?;
            println!("Bundle saved to bundle.cbor");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_opts_parse_insert() {
        let args = vec!["test-bin", "insert", "--message", "hello"];
        let opts = Opts::parse_from(args);
        match opts.cmd {
            Command::Insert { message } => assert_eq!(message, "hello"),
        }
    }
}
