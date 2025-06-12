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
# Build the project
cargo build --release

# Create a bundle
spacearth-dtn insert --message "Hello, DTN!"

# List all bundles
spacearth-dtn list

# Show bundle details (using partial ID)
spacearth-dtn show --id <partial_id>

# Dispatch all bundles to the destination
spacearth-dtn dispatch
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

3. 🔜 **Forwarding Control** (Next)
   - Relay node routing
   - Routing rules implementation

4. 🚧 **CLA (Convergence Layer Adapter)**
   - TCP/UDP communication
   - LoRa/BLE support
   - HTTP/HTTPS support

5. 🚧 **Software Bus**
   - Inter-process communication
   - Message queue

6. 🚧 **Event Loop / Task Management**
   - Async processing
   - Task scheduling

7. ⬛ **Management CLI / WebUI** (Optional)
   - Advanced management features
   - Visualization tools

8. ⬛ **RFC Compliance** (Optional)
   - RFC 9171 compliance tests
   - Interoperability tests

---

## License

MIT OR Apache-2.0

---

## AI-Generated Content

Some parts of this project (README, code comments, and sample logic) are co-authored or generated using AI tools.  
All code is manually reviewed and tested before use.
