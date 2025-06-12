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

---

## 日本語説明（Japanese Section）

**spacearth-dtn** は、Rustで書かれた遅延耐性ネットワーク（DTN）の実装です。  
宇宙から地上、また地上内の通信断環境でも機能する、**レジリエントな通信技術**を目指しています。

### 連絡先

質問、提案、貢献については以下までご連絡ください：
- メール: [hsatlefp@gmail.com](mailto:hsatlefp@gmail.com)

### 開発ロードマップ

1. ✅ **Bundle構造・CBOR対応** (完了)
   - Bundle構造体の定義
   - CBORシリアライズ/デシリアライズ
   - 基本的なCLI操作

2. ✅ **Bundleの保存/ロード** (完了)
   - ファイルベースの永続化
   - BundleStore実装
   - 部分ID検索機能
   - テストの自動クリーンアップ
   - バンドルの送信機能

3. 🔜 **転送制御 (forwarding)** (次期)
   - 中継ノードでのルーティング
   - ルーティングルール実装

4. 🚧 **CLA (Convergence Layer Adapter)**
   - TCP/UDP通信
   - LoRa/BLE対応
   - HTTP/HTTPS対応

5. 🚧 **ソフトウェアバス**
   - プロセス間通信
   - メッセージキュー

6. 🚧 **イベントループ / タスク管理**
   - 非同期処理
   - タスクスケジューリング

7. ⬛ **管理CLI / WebUI** (オプション)
   - 詳細な管理機能
   - 可視化ツール

8. ⬛ **RFC準拠検証** (オプション)
   - RFC 9171準拠テスト
   - 相互運用性テスト

今後、LoRa・BLEなどのCLA（通信層）との統合や、CLI・Web可視化ツールなども展開予定です。

開発初期フェーズにつき、Pull Request・Issue歓迎します！
