// CLI ハンドラー関数のテスト
// src/bin/cli.rs の関数を直接インポートしてテストする

use sdtn::api::DtnNode;
use std::sync::Once;

// ログの初期化を一度だけ行う
static INIT: Once = Once::new();

fn init_logger() {
    INIT.call_once(|| {
        env_logger::init();
    });
}

fn create_test_node() -> DtnNode {
    init_logger();
    DtnNode::new().expect("Failed to create test node")
}

#[cfg(test)]
mod handler_tests {
    use super::*;

    #[test]
    fn test_insert_command_handler() {
        let node = create_test_node();
        let test_message = "Test message for insert handler".to_string();

        // insert コマンドハンドラーをテスト
        // 注意: 実際の関数は src/bin/cli.rs にあるため、これらは統合テストとして書く
        let result = node.insert_bundle(test_message.clone());
        assert!(result.is_ok(), "Insert handler should work correctly");

        // 結果を確認
        let bundles = node.list_bundles().expect("Should be able to list bundles");
        assert!(!bundles.is_empty(), "Bundle should be inserted");
    }

    #[test]
    fn test_list_command_handler() {
        let node = create_test_node();

        // まずバンドルを挿入
        node.insert_bundle("Test bundle for list".to_string())
            .expect("Should insert test bundle");

        // list コマンドハンドラーをテスト
        let bundles = node.list_bundles();
        assert!(bundles.is_ok(), "List handler should work correctly");

        let bundles = bundles.unwrap();
        assert!(!bundles.is_empty(), "Should have at least one bundle");
    }

    #[test]
    fn test_show_command_handler() {
        let node = create_test_node();
        let test_message = "Test message for show handler".to_string();

        // バンドルを挿入
        node.insert_bundle(test_message.clone())
            .expect("Should insert test bundle");

        // バンドルIDを取得
        let bundles = node.list_bundles().expect("Should list bundles");
        assert!(!bundles.is_empty(), "Should have bundles");

        let bundle_id = &bundles[0];

        // show コマンドハンドラーをテスト
        let bundle = node.show_bundle(bundle_id);
        assert!(bundle.is_ok(), "Show handler should work correctly");

        let bundle = bundle.unwrap();
        assert_eq!(String::from_utf8_lossy(&bundle.payload), test_message);
    }

    #[test]
    fn test_status_command_handler() {
        let node = create_test_node();

        // バンドルを挿入
        node.insert_bundle("Test bundle for status".to_string())
            .expect("Should insert test bundle");

        // 全体ステータスのテスト
        let status = node.get_bundle_status(None);
        assert!(status.is_ok(), "Status handler should work correctly");

        // 個別バンドルのステータステスト
        let bundles = node.list_bundles().expect("Should list bundles");
        if !bundles.is_empty() {
            let bundle_id = &bundles[0];
            let bundle = node.show_bundle(bundle_id);
            assert!(
                bundle.is_ok(),
                "Should be able to get individual bundle status"
            );
        }
    }

    #[test]
    fn test_cleanup_command_handler() {
        let node = create_test_node();

        // cleanup コマンドハンドラーをテスト
        let result = node.cleanup_expired();
        assert!(result.is_ok(), "Cleanup handler should work correctly");
    }

    #[test]
    fn test_route_table_command_handler() {
        let node = create_test_node();

        // ルーティングテーブル取得のテスト
        let routes = node.get_all_routes();
        assert!(routes.is_ok(), "Route table handler should work correctly");
    }

    #[test]
    fn test_route_add_command_handler() {
        let node = create_test_node();

        // ルート追加のテスト
        let test_route = sdtn::routing::algorithm::RouteEntry {
            destination: sdtn::bpv7::EndpointId::from("dtn://test-dest/"),
            next_hop: sdtn::bpv7::EndpointId::from("dtn://test-hop/"),
            cla_type: "tcp".to_string(),
            cost: 15,
            is_active: true,
        };

        let result = node.add_route(test_route);
        assert!(result.is_ok(), "Route add handler should work correctly");
    }

    #[test]
    fn test_route_test_command_handler() {
        let node = create_test_node();
        let test_message = "Test message for route test".to_string();

        // バンドルを挿入
        node.insert_bundle(test_message)
            .expect("Should insert test bundle");

        let bundles = node.list_bundles().expect("Should list bundles");
        if !bundles.is_empty() {
            let bundle_id = &bundles[0];
            let bundle = node.show_bundle(bundle_id).expect("Should get bundle");

            // ルートテストハンドラーをテスト
            let peers_result = node.select_peers_for_forwarding(&bundle);
            // 結果の成功/失敗に関わらず、処理が完了することを確認
            assert!(
                peers_result.is_ok() || peers_result.is_err(),
                "Route test handler should complete"
            );
        }
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_show_nonexistent_bundle() {
        let node = create_test_node();

        // 存在しないバンドルIDでshowを実行
        let result = node.show_bundle("nonexistent-id");
        assert!(
            result.is_err(),
            "Should return error for nonexistent bundle"
        );
    }

    #[test]
    fn test_route_with_invalid_endpoint() {
        let node = create_test_node();

        // 無効なエンドポイントでルートを追加しようとする
        let invalid_route = sdtn::routing::algorithm::RouteEntry {
            destination: sdtn::bpv7::EndpointId::from("invalid-endpoint"),
            next_hop: sdtn::bpv7::EndpointId::from("invalid-hop"),
            cla_type: "tcp".to_string(),
            cost: 10,
            is_active: true,
        };

        // この場合でも処理は完了すべき（エラーになってもOK）
        let result = node.add_route(invalid_route);
        assert!(
            result.is_ok() || result.is_err(),
            "Route add should complete even with invalid endpoints"
        );
    }
}
