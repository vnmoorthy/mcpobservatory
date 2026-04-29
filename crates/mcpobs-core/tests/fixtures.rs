use mcpobs_core::protocol::jsonrpc::{JsonRpcId, ParsedMessage};

#[test]
fn fixture_initialize_parses_as_request() {
    let bytes = include_bytes!("fixtures/initialize.json");
    let m = ParsedMessage::parse(bytes.to_vec());
    assert!(m.is_request());
    assert_eq!(m.method(), Some("initialize"));
    assert_eq!(m.id(), Some(JsonRpcId::Num(1)));
}

#[test]
fn fixture_tools_list_parses_as_response() {
    let bytes = include_bytes!("fixtures/tools_list.json");
    let m = ParsedMessage::parse(bytes.to_vec());
    assert!(m.is_response());
}

#[test]
fn fixture_error_parses_as_error() {
    let bytes = include_bytes!("fixtures/error.json");
    let m = ParsedMessage::parse(bytes.to_vec());
    assert!(m.is_error());
}
