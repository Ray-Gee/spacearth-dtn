use crate::cla::*;

#[test]
fn test_module_exports_exist() {
    // Test that the re-exports work by referencing the types
    // This ensures the modules are properly exposed

    // Check that we can reference the manager types
    let _manager_type = std::any::TypeId::of::<ClaManager>();
    let _convergence_layer_type = std::any::TypeId::of::<dyn ConvergenceLayer>();

    // Check that we can reference the TCP types
    let _dialer_type = std::any::TypeId::of::<TcpClaDialer>();
    let _listener_type = std::any::TypeId::of::<TcpClaListener>();
}

#[test]
fn test_modules_are_accessible() {
    // This test verifies that all modules are accessible

    // Check that we can access the module paths
    let _manager_module = module_path!();
    assert!(module_path!().contains("cla"));

    // These imports should work if modules are public
    use crate::cla::manager::ClaManager;
    use crate::cla::tcp::client::TcpClaDialer;
    use crate::cla::tcp::server::TcpClaListener;

    let _ = std::any::TypeId::of::<ClaManager>();
    let _ = std::any::TypeId::of::<TcpClaDialer>();
    let _ = std::any::TypeId::of::<TcpClaListener>();
}

#[test]
fn test_reexports_work() {
    // Test that the re-exports match the original types
    assert_eq!(
        std::any::TypeId::of::<ClaManager>(),
        std::any::TypeId::of::<crate::cla::manager::ClaManager>()
    );

    assert_eq!(
        std::any::TypeId::of::<TcpClaDialer>(),
        std::any::TypeId::of::<crate::cla::tcp::client::TcpClaDialer>()
    );

    assert_eq!(
        std::any::TypeId::of::<TcpClaListener>(),
        std::any::TypeId::of::<crate::cla::tcp::server::TcpClaListener>()
    );
}
