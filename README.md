# spacearth-dtn

![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)

**spacearth-dtn** is a Rust-based implementation of Delay Tolerant Networking (DTN),  
designed for resilient communication between space and earth — and beyond.

This project aims to offer modular, efficient, and extensible components for BPv7-based DTN systems,  
suitable for both space and terrestrial disruption-tolerant networks.

> "From space to earth. For the disconnected."

## Contact

For questions, suggestions, or contributions, please contact:
- Email: [hsatlefp@gmail.com](mailto:hsatlefp@gmail.com)

---

## Features

- 🌍 BPv7-compliant Bundle Protocol
- 🛰️ Store-and-forward mechanism
- 🔌 Modular CLA (Convergence Layer Adapter) design
- 📦 Bundle persistence and metadata management
- 🛠️ Extensible for LoRa, BLE, disaster scenarios, and more

---

## Quick Start

### CLI Tool

A command-line tool for creating and managing Bundle Protocol bundles is available:

```bash
# Install the project
cargo install --path .

# Create a bundle
sdtn insert --message "Hello, DTN!"

# List all bundles
sdtn list

# Show bundle details (using partial ID)
sdtn show --id <partial_id>

# Start daemon listener (receiver)
sdtn daemon listener --addr 127.0.0.1:3000

# Start daemon dialer (sender)
sdtn daemon dialer --addr 127.0.0.1:3000
```

Configuration is managed in `config/default.toml` and can be overridden with environment variables:

```bash
# Specify config file
export DTN_CONFIG="config/development.toml"

# Override individual settings
export DTN_BUNDLE_VERSION=8
export DTN_ENDPOINTS_DESTINATION="dtn://new-dest"
```

---

## Testing the Setup

You can verify basic DTN communication with the following steps:

### 1. Start the listener (receiver)
```bash
# Run in terminal 1
sdtn daemon listener --addr 127.0.0.1:3000
```

### 2. Create a bundle and send it via dialer (sender)
```bash
# Run in terminal 2
sdtn insert --message "Hello, DTN!"
sdtn daemon dialer --addr 127.0.0.1:3000
```

This procedure allows you to verify that the created bundle is transmitted via TCP and received by the listener.

---

## Development Roadmap

Current development phase and future plans:

1. ✅ **Bundle Structure & CBOR Support** (Completed)
   - Bundle structure definition
   - CBOR serialization/deserialization
   - Basic CLI operations

2. ✅ **Bundle Storage/Load** (Completed)
   - File-based persistence
   - BundleStore implementation
   - Partial ID lookup support
   - Automatic test cleanup
   - Bundle dispatch functionality

3. ✅ **CLA (Convergence Layer Adapter)** (Completed)
   - TCP communication
   - Basic daemon listener/dialer functionality

4. 🔜 **Bundle Transmission & Reception** (Next)
   - Bundle sending via CLA
   - Bundle receiving and processing
   - Integration with BundleStore

5. 🚧 **Forwarding Control**
   - Relay node routing
   - Routing rules implementation

6. 🚧 **Software Bus**
   - Inter-process communication
   - Message queue

7. 🚧 **Event Loop / Task Management**
   - Async processing
   - Task scheduling

8. ⬛ **Management CLI / WebUI** (Optional)
   - Advanced management features
   - Visualization tools

9. ⬛ **RFC Compliance** (Optional)
   - RFC 9171 compliance tests
   - Interoperability tests

---

## License

MIT OR Apache-2.0

---

## AI-Generated Content

Some parts of this project (README, code comments, and sample logic) are co-authored or generated using AI tools.  
All code is manually reviewed and tested before use.
