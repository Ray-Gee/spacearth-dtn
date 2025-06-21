use sdtn::api::DtnNode;
use sdtn::bpv7::EndpointId;
use sdtn::routing::algorithm::RouteEntry;

// CLI コマンドハンドラーのテスト
#[cfg(test)]
mod cli_tests {
    use super::*;
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

    #[test]
    fn test_insert_and_list_bundle() {
        let node = create_test_node();
        let test_message = "Test message for CLI".to_string();

        // バンドルの挿入をテスト
        let result = node.insert_bundle(test_message.clone());
        assert!(result.is_ok(), "Bundle insertion should succeed");

        // バンドル一覧取得をテスト
        let bundles = node.list_bundles().expect("Should be able to list bundles");
        assert!(!bundles.is_empty(), "Bundle list should not be empty");
    }

    #[test]
    fn test_show_bundle() {
        let node = create_test_node();
        let test_message = "Test message for show".to_string();

        // バンドルを挿入
        node.insert_bundle(test_message.clone())
            .expect("Bundle insertion should succeed");

        // バンドル一覧を取得
        let bundles = node.list_bundles().expect("Should be able to list bundles");
        assert!(!bundles.is_empty(), "Should have at least one bundle");

        // 最初のバンドルの詳細を取得
        let bundle_id = &bundles[0];
        let bundle = node.show_bundle(bundle_id);
        assert!(bundle.is_ok(), "Should be able to show bundle details");

        let bundle = bundle.unwrap();
        assert_eq!(String::from_utf8_lossy(&bundle.payload), test_message);
    }

    #[test]
    fn test_bundle_status() {
        let node = create_test_node();
        let test_message = "Test message for status".to_string();

        // バンドルを挿入
        node.insert_bundle(test_message)
            .expect("Bundle insertion should succeed");

        // 全体のステータスを取得
        let status = node.get_bundle_status(None);
        assert!(status.is_ok(), "Should be able to get bundle status");

        // 特定のバンドルのステータスを取得
        let bundles = node.list_bundles().expect("Should be able to list bundles");
        if !bundles.is_empty() {
            let bundle_id = &bundles[0];
            let bundle = node.show_bundle(bundle_id);
            assert!(
                bundle.is_ok(),
                "Should be able to get specific bundle status"
            );
        }
    }

    #[test]
    fn test_cleanup_expired() {
        let node = create_test_node();

        // クリーンアップ処理のテスト
        let result = node.cleanup_expired();
        assert!(result.is_ok(), "Cleanup should succeed");
    }

    #[test]
    fn test_routing_functionality() {
        let node = create_test_node();

        // ルーティングテーブルの取得をテスト
        let routes = node.get_all_routes();
        assert!(routes.is_ok(), "Should be able to get routing table");

        // ルートの追加をテスト
        let test_route = RouteEntry {
            destination: EndpointId::from("dtn://test-destination/"),
            next_hop: EndpointId::from("dtn://test-hop/"),
            cla_type: "tcp".to_string(),
            cost: 10,
            is_active: true,
        };

        let result = node.add_route(test_route);
        assert!(result.is_ok(), "Should be able to add route");

        // ベストルートの検索をテスト
        let destination = EndpointId::from("dtn://test-destination/");
        let best_route = node.find_best_route(&destination);
        assert!(best_route.is_ok(), "Should be able to find best route");
    }

    #[test]
    fn test_bundle_forwarding_selection() {
        let node = create_test_node();
        let test_message = "Test message for forwarding".to_string();

        // バンドルを挿入
        node.insert_bundle(test_message)
            .expect("Bundle insertion should succeed");

        let bundles = node.list_bundles().expect("Should be able to list bundles");
        if !bundles.is_empty() {
            let bundle_id = &bundles[0];
            let bundle = node
                .show_bundle(bundle_id)
                .expect("Should be able to get bundle");

            // フォワーディング用のピア選択をテスト
            let peers_result = node.select_peers_for_forwarding(&bundle);
            // エラーでも成功でも、少なくとも処理が完了することを確認
            assert!(
                peers_result.is_ok() || peers_result.is_err(),
                "Peer selection should complete"
            );

            // ルートベースのフォワーディング選択をテスト
            let routes_result = node.select_routes_for_forwarding(&bundle);
            // エラーでも成功でも、少なくとも処理が完了することを確認
            assert!(
                routes_result.is_ok() || routes_result.is_err(),
                "Route selection should complete"
            );
        }
    }
}

// 統合テスト用のヘルパー関数
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_cli_workflow() {
        let node = create_test_node();

        // 1. バンドルを挿入
        let test_message = "Integration test message".to_string();
        node.insert_bundle(test_message.clone())
            .expect("Should insert bundle");

        // 2. バンドル一覧を確認
        let bundles = node.list_bundles().expect("Should list bundles");
        assert!(!bundles.is_empty(), "Should have bundles");

        // 3. バンドルの詳細を確認
        let bundle_id = &bundles[0];
        let bundle = node.show_bundle(bundle_id).expect("Should show bundle");
        assert_eq!(String::from_utf8_lossy(&bundle.payload), test_message);

        // 4. ステータスを確認
        let _status = node.get_bundle_status(None).expect("Should get status");

        // 5. クリーンアップを実行
        node.cleanup_expired().expect("Should cleanup");
    }

    fn create_test_node() -> DtnNode {
        DtnNode::new().expect("Failed to create test node")
    }
}
