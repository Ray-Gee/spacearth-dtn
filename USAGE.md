# SpaceArth DTN Library Usage

SpaceArth DTN is a Rust implementation of Delay Tolerant Networking (DTN). This library can be used both as a CLI application and as a library.

## Using as a Library

### Basic Usage Example

```rust
use sdtn::{DtnCli, BundleStatus};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create DTN CLI instance with default settings
    let cli = DtnCli::new()?;

    // Insert a bundle
    cli.insert_bundle("Hello from SpaceArth DTN!".to_string())?;

    // List all bundles
    let bundles = cli.list_bundles()?;
    println!("Found {} bundles", bundles.len());

    // Show details of a specific bundle
    if let Some(id) = bundles.first() {
        let bundle = cli.show_bundle(id)?;
        println!("Message: {}", String::from_utf8_lossy(&bundle.payload));
    }

    // Get status summary
    let status = cli.get_bundle_status(None)?;
    match status {
        BundleStatus::Summary { active, expired, total } => {
            println!("Active: {}, Expired: {}, Total: {}", active, expired, total);
        }
        _ => unreachable!(),
    }

    // Clean up expired bundles
    cli.cleanup_expired()?;

    Ok(())
}
```

### Using Custom Storage Path

```rust
use sdtn::DtnCli;

// Specify custom storage path
let cli = DtnCli::with_store_path("./my_custom_bundles")?;

// Or use configuration options
let cli = DtnCli::with_config(Some("./my_bundles"))?;
let cli_default = DtnCli::with_config(None)?; // Use default path
```

### Using Convenience Functions

```rust
use sdtn::convenience;

// Quick operations using default path (./bundles)
convenience::insert_bundle_quick("Quick message")?;
let bundles = convenience::list_bundles_quick()?;
let bundle = convenience::show_bundle_quick("partial_id")?;
```

### Advanced Usage Example

```rust
use sdtn::{DtnCli, BundleStatus};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create instance with default settings
    let cli = DtnCli::new()?;

    // Or use Default trait
    let cli = DtnCli::default();

    // Insert multiple bundles
    let messages = [
        "First message",
        "Second message with emoji: 🚀🌍",
        "Third message with numbers: 12345",
    ];

    for msg in messages.iter() {
        cli.insert_bundle(msg.to_string())?;
    }

    // Get detailed status for a specific bundle
    let bundles = cli.list_bundles()?;
    if let Some(id) = bundles.first() {
        let status = cli.get_bundle_status(Some(id))?;
        match status {
            BundleStatus::Single { id, bundle } => {
                println!("Bundle ID: {}", id);
                println!("Source: {}", bundle.primary.source);
                println!("Destination: {}", bundle.primary.destination);
                println!("Expired: {}", bundle.is_expired());
                println!("Message: {}", String::from_utf8_lossy(&bundle.payload));
            }
            _ => unreachable!(),
        }
    }

    // Start TCP listener daemon
    // cli.start_tcp_listener("127.0.0.1:8080".to_string()).await?;

    Ok(())
}
```

## Available APIs

### DtnCli

Provides the main DTN client API.

#### Constructors

- `new() -> anyhow::Result<Self>`: Create DTN CLI instance with default settings (./bundles)
- `with_store_path(store_path: &str) -> anyhow::Result<Self>`: Create instance with custom storage path
- `with_config(store_path: Option<&str>) -> anyhow::Result<Self>`: Create instance with configuration options
- `default()`: Default trait implementation (same as `new()`)

#### Methods

- `insert_bundle(message: String) -> anyhow::Result<()>`: Insert a new bundle
- `list_bundles() -> anyhow::Result<Vec<String>>`: List all bundle IDs
- `show_bundle(partial_id: &str) -> anyhow::Result<Bundle>`: Show bundle details
- `get_bundle_status(partial_id: Option<&str>) -> anyhow::Result<BundleStatus>`: Get bundle status
- `cleanup_expired() -> anyhow::Result<()>`: Clean up expired bundles
- `start_tcp_listener(bind_addr: String) -> anyhow::Result<()>`: Start TCP listener daemon
- `start_tcp_dialer(target_addr: String) -> anyhow::Result<()>`: Start TCP dialer daemon

### BundleStatus

Enum representing bundle status information.

```rust
pub enum BundleStatus {
    Single {
        id: String,
        bundle: Bundle,
    },
    Summary {
        active: usize,
        expired: usize,
        total: usize,
    },
}
```

### convenience

Module providing quick operations with default settings.

- `insert_bundle_quick(message: &str) -> anyhow::Result<()>`
- `list_bundles_quick() -> anyhow::Result<Vec<String>>`
- `show_bundle_quick(partial_id: &str) -> anyhow::Result<Bundle>`

## Design Philosophy

### Intuitive API Design

- **Default Settings**: Start using immediately with `DtnCli::new()`
- **Customizable**: Customize with `with_store_path()` when needed
- **Convenience Functions**: Use `convenience` module for one-off operations

### Usage Guidelines

```rust
// 👍 Recommended: General usage
let cli = DtnCli::new()?;

// 👍 Recommended: When custom path is needed
let cli = DtnCli::with_store_path("./my_bundles")?;

// 👍 Recommended: One-off operations
convenience::insert_bundle_quick("message")?;

// 👍 Recommended: Using Default trait
let cli = DtnCli::default();
```

## Error Handling

The library uses `anyhow::Result` for error handling:

```rust
match cli.show_bundle("nonexistent") {
    Ok(bundle) => println!("Found bundle: {:?}", bundle),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Configuration

Bundle configuration is managed in the `config.toml` file. See [Configuration Documentation](CONFIG.md) for details.

## Running Examples

You can run the following commands in the project root directory to see examples:

```bash
# Basic usage example
cargo run --example basic

# Advanced usage example
cargo run --example advanced
```

## License

This project is published under the MIT OR Apache-2.0 license.