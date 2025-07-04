# spacearth-dtn

![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)

**spacearth-dtn** は、Rustで書かれた遅延耐性ネットワーク（DTN）の実装です。
宇宙から地上、また地上内の通信断環境でも機能する、**レジリエントな通信技術**を目指しています。

> "宇宙から地上へ。そして、通信断の世界へ。"

## 連絡先

質問、提案、貢献については以下までご連絡ください：
- メール: [hsatlefp@gmail.com](mailto:hsatlefp@gmail.com)

---

## 機能

- 🌍 BPv7準拠のBundle Protocol
- 🛰️ ストアアンドフォワード機構
- 🔌 モジュラーCLA（Convergence Layer Adapter）設計
- 📦 Bundleの永続化とメタデータ管理
- 🛠️ LoRa、BLE、災害シナリオなどへの拡張性

---

## クイックスタート

### CLIツール

Bundle Protocolのバンドルを作成・管理するためのコマンドラインツールが利用可能です：

```bash
# プロジェクトのインストール
cargo install --path .

# バンドルの作成
sdtn insert --message "Hello, DTN!"

# すべてのバンドルを表示
sdtn list

# バンドルの詳細表示（部分IDを使用）
sdtn show --id <partial_id>

# デーモンリスナー（受信側）を開始
sdtn daemon listener --addr 127.0.0.1:3000

# デーモンダイアラー（送信側）を開始
sdtn daemon dialer --addr 127.0.0.1:3000
```

設定は`config/default.toml`で管理され、環境変数で上書き可能です：

```bash
# 設定ファイルの指定
export DTN_CONFIG="config/development.toml"

# 個別設定の上書き
export DTN_BUNDLE_VERSION=8
export DTN_ENDPOINTS_DESTINATION="dtn://new-dest"
```

---

## 動作確認

以下の手順で基本的なDTN通信の動作を確認できます：

### 1. リスナー（受信側）を起動
```bash
# ターミナル1で実行
sdtn daemon listener --addr 127.0.0.1:3000
```

### 2. バンドルを作成し、ダイアラー（送信側）で送信
```bash
# ターミナル2で実行
sdtn insert --message "Hello, DTN!"
sdtn daemon dialer --addr 127.0.0.1:3000
```

この手順により、作成したバンドルがTCP経由で送信され、リスナー側で受信されることを確認できます。

---

## 開発ロードマップ

現在の開発フェーズと今後の計画：

1. ✅ **Bundle構造・CBOR対応**
   - Bundle構造体の定義
   - CBORシリアライズ/デシリアライズ
   - 基本的なCLI操作

2. ✅ **Bundleの保存/ロード**
   - ファイルベースの永続化
   - BundleStore実装
   - 部分ID検索機能
   - テストの自動クリーンアップ
   - バンドルの送信機能

3. ✅ **TCP CLAによる通信機能**
   - TCP CLA（Listener / Dialer）を通じたバンドル送受信
   - ACK返却（テキスト形式）

4. 🚧 **CLA抽象化と送受信統合**
   - `trait Cla` による共通インターフェース定義
   - CLA抽象化経由での送信・受信の動作確認
   - BundleStoreとの統合

5. 🔜 **ルーティング機能の実装（最優先）**
   - 宛先に応じたCLAの選択
   - 静的ルーティングテーブル（destination → next hop）
   - 将来的な動的ルーティングに対応可能な構造を検討

6. ⏸ **CLAの多様化（優先度：低）**
   - BLE CLA：Central / Peripheral ともにRustで記述済（※動作確認はRaspberry Pi購入後に予定）
   - LoRa CLAの段階的実装（USB AT → SPI → embedded / RTOS）
   - `trait Cla` に準拠した動的CLA選択の統合

7. 🚧 **ソフトウェアバスとタスク管理**
   - プロセス間通信
   - メッセージキュー
   - 非同期処理とタスクスケジューリング

8. ⬛ **管理CLI / WebUI**（オプション）
   - 詳細な管理機能
   - 可視化ツール

9. ⬛ **RFC準拠検証**（オプション）
   - RFC 9171準拠テスト
   - 相互運用性テスト

---

## ライセンス

MIT OR Apache-2.0

---

## AI生成コンテンツ

このプロジェクトの一部（README、コードコメント、サンプルロジック）は、AIツールを使用して共同作成または生成されています。
すべてのコードは使用前に手動でレビューとテストが行われています。