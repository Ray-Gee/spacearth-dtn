use crate::bpv7::EndpointId;

#[test]
fn test_endpoint_id_creation() {
    let eid = EndpointId::new("dtn://node1".to_string());
    assert_eq!(eid.as_str(), "dtn://node1");
}

#[test]
fn test_endpoint_id_from_str() {
    let eid = EndpointId::from("dtn://node2");
    assert_eq!(eid.as_str(), "dtn://node2");
}

#[test]
fn test_dtn_scheme_detection() {
    let eid = EndpointId::from("dtn://example");
    assert!(eid.is_dtn_scheme());

    let eid2 = EndpointId::from("http://example");
    assert!(!eid2.is_dtn_scheme());
}

#[test]
fn test_null_endpoint() {
    let eid1 = EndpointId::from("dtn:none");
    assert!(eid1.is_null());

    let eid2 = EndpointId::from("");
    assert!(eid2.is_null());

    let eid3 = EndpointId::from("dtn://node");
    assert!(!eid3.is_null());
}

#[test]
fn test_display() {
    let eid = EndpointId::from("dtn://test");
    assert_eq!(format!("{}", eid), "dtn://test");
}

#[test]
fn test_serialization() {
    let eid = EndpointId::from("dtn://serialize-test");
    let json = serde_json::to_string(&eid).unwrap();
    let deserialized: EndpointId = serde_json::from_str(&json).unwrap();
    assert_eq!(eid, deserialized);
}
