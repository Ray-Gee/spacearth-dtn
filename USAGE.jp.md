# SpaceArth DTN ライブラリ使用方法

SpaceArth DTNは、Delay Tolerant Networking (DTN) のRust実装です。このライブラリは、CLIアプリケーションとしても、ライブラリとしても使用できます。

## ライブラリとしての使用

### 基本的な使用例

```rust
use sdtn::{DtnCli, BundleStatus};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // DTN CLIインスタンスを作成（デフォルト設定）
    let cli = DtnCli::new()?;

    // バンドルを挿入
    cli.insert_bundle("Hello from SpaceArth DTN!".to_string())?;

    // すべてのバンドルをリスト
    let bundles = cli.list_bundles()?;
    println!("Found {} bundles", bundles.len());

    // 特定のバンドルの詳細を表示
    if let Some(id) = bundles.first() {
        let bundle = cli.show_bundle(id)?;
        println!("Message: {}", String::from_utf8_lossy(&bundle.payload));
    }

    // ステータスサマリーを取得
    let status = cli.get_bundle_status(None)?;
    match status {
        BundleStatus::Summary { active, expired, total } => {
            println!("Active: {}, Expired: {}, Total: {}", active, expired, total);
        }
        _ => unreachable!(),
    }

    // 期限切れバンドルをクリーンアップ
    cli.cleanup_expired()?;

    Ok(())
}
```

### カスタムストレージパスの使用

```rust
use sdtn::DtnCli;

// カスタムストレージパスを指定
let cli = DtnCli::with_store_path("./my_custom_bundles")?;

// または設定オプションを使用
let cli = DtnCli::with_config(Some("./my_bundles"))?;
let cli_default = DtnCli::with_config(None)?; // デフォルトパスを使用
```

### 便利関数の使用

```rust
use sdtn::convenience;

// デフォルトパス（./bundles）を使用したクイック操作
convenience::insert_bundle_quick("Quick message")?;
let bundles = convenience::list_bundles_quick()?;
let bundle = convenience::show_bundle_quick("partial_id")?;
```

### 高度な使用例

```rust
use sdtn::{DtnCli, BundleStatus};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // デフォルト設定でインスタンス作成
    let cli = DtnCli::new()?;

    // またはDefault traitを使用
    let cli = DtnCli::default();

    // 複数のバンドルを挿入
    let messages = [
        "First message",
        "Second message with emoji: 🚀🌍",
        "Third message with numbers: 12345",
    ];

    for msg in messages.iter() {
        cli.insert_bundle(msg.to_string())?;
    }

    // 特定のバンドルの詳細ステータスを取得
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

    // TCPリスナーデーモンを開始
    // cli.start_tcp_listener("127.0.0.1:8080".to_string()).await?;

    Ok(())
}
```

## 利用可能なAPI

### DtnCli

メインのDTNクライアントAPIを提供します。

#### コンストラクタ

- `new() -> anyhow::Result<Self>`: デフォルト設定（./bundles）でDTN CLIインスタンスを作成
- `with_store_path(store_path: &str) -> anyhow::Result<Self>`: カスタムストレージパスでインスタンスを作成
- `with_config(store_path: Option<&str>) -> anyhow::Result<Self>`: 設定オプションでインスタンスを作成
- `default()`: Default traitの実装（`new()`と同等）

#### メソッド

- `insert_bundle(message: String) -> anyhow::Result<()>`: 新しいバンドルを挿入
- `list_bundles() -> anyhow::Result<Vec<String>>`: すべてのバンドルIDをリスト
- `show_bundle(partial_id: &str) -> anyhow::Result<Bundle>`: バンドルの詳細を表示
- `get_bundle_status(partial_id: Option<&str>) -> anyhow::Result<BundleStatus>`: バンドルステータスを取得
- `cleanup_expired() -> anyhow::Result<()>`: 期限切れバンドルをクリーンアップ
- `start_tcp_listener(bind_addr: String) -> anyhow::Result<()>`: TCPリスナーデーモンを開始
- `start_tcp_dialer(target_addr: String) -> anyhow::Result<()>`: TCPダイアラーデーモンを開始

### BundleStatus

バンドルのステータス情報を表す列挙型です。

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

デフォルト設定でクイック操作を提供するモジュールです。

- `insert_bundle_quick(message: &str) -> anyhow::Result<()>`
- `list_bundles_quick() -> anyhow::Result<Vec<String>>`
- `show_bundle_quick(partial_id: &str) -> anyhow::Result<Bundle>`

## 設計思想

### 直感的なAPI設計

- **デフォルト設定**: `DtnCli::new()`でそのまま使用開始
- **カスタマイズ可能**: 必要に応じて`with_store_path()`でカスタマイズ
- **便利関数**: 一回限りの操作には`convenience`モジュール

### 使い分けの指針

```rust
// 👍 推奨: 一般的な使用
let cli = DtnCli::new()?;

// 👍 推奨: カスタムパスが必要な場合
let cli = DtnCli::with_store_path("./my_bundles")?;

// 👍 推奨: 一回限りの操作
convenience::insert_bundle_quick("message")?;

// 👍 推奨: Default traitを使用
let cli = DtnCli::default();
```

## エラーハンドリング

ライブラリは`anyhow::Result`を使用してエラーハンドリングを行います：

```rust
match cli.show_bundle("nonexistent") {
    Ok(bundle) => println!("Found bundle: {:?}", bundle),
    Err(e) => eprintln!("Error: {}", e),
}
```

## 設定

バンドルの設定は`config.toml`ファイルで管理されます。詳細は[設定ドキュメント](CONFIG.md)を参照してください。

## 例の実行

プロジェクトのルートディレクトリで以下のコマンドを実行して例を確認できます：

```bash
# 基本的な使用例
cargo run --example basic

# 高度な使用例
cargo run --example advanced
```

## ライセンス

このプロジェクトはMIT OR Apache-2.0ライセンスの下で公開されています。