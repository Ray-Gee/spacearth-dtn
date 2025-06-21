#[test]
fn test_module_path() {
    // Note: path will be "sdtn::cla::tests::ble_tests" for moved tests
    let path = module_path!();
    assert!(path.contains("cla::tests::ble_tests"));
}

#[test]
fn test_current_module() {
    let current_module = module_path!();
    assert!(current_module.contains("cla::tests::ble_tests"));
}
