use host_lib::{
    mock::Mock,
    assistant::Assistant,
    conn::Conn,
};
use protocol::HostToAssistant;
use postcard;

use insta::assert_debug_snapshot;

#[test]
fn make_instance() {
    let mock = Mock::new();
    let test_hdl = mock.clone();

    let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
    let _assistant = Assistant::new(conn);

    assert!(test_hdl.is_totally_empty());
}

#[test]
fn set_pin_5_high() {
    let mock = Mock::new();
    let test_hdl = mock.clone();

    let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
    let mut assistant = Assistant::new(conn);
    assistant.set_pin_5_high().unwrap();

    let msg = test_hdl.pop_host_lib_data().unwrap();
    let mut msg_clone = msg.clone();
    let deser_msg: HostToAssistant = postcard::from_bytes_cobs(&mut msg_clone).unwrap();

    // TODO: Use the protocol to deserialize?
    // Add insta for snapshot tests
    assert_debug_snapshot!(
        "set_pin_5_high - bytes",
        &msg
    );

    assert_debug_snapshot!(
        "set_pin_5_high - parsed",
        &deser_msg
    );

    assert!(test_hdl.is_totally_empty());
}

#[test]
fn set_pin_5_low() {
    let mock = Mock::new();
    let test_hdl = mock.clone();

    let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
    let mut assistant = Assistant::new(conn);
    assistant.set_pin_5_low().unwrap();

    let msg = test_hdl.pop_host_lib_data().unwrap();
    let mut msg_clone = msg.clone();
    let deser_msg: HostToAssistant = postcard::from_bytes_cobs(&mut msg_clone).unwrap();

    // TODO: Use the protocol to deserialize?
    // Add insta for snapshot tests
    assert_debug_snapshot!(
        "set_pin_5_low - bytes",
        &msg
    );

    assert_debug_snapshot!(
        "set_pin_5_low - parsed",
        &deser_msg
    );

    assert!(test_hdl.is_totally_empty());
}

#[test]
fn set_pin_high() {
    let mock = Mock::new();
    let test_hdl = mock.clone();

    let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
    let mut assistant = Assistant::new(conn);
    assistant.set_pin_high().unwrap();

    let msg = test_hdl.pop_host_lib_data().unwrap();
    let mut msg_clone = msg.clone();
    let deser_msg: HostToAssistant = postcard::from_bytes_cobs(&mut msg_clone).unwrap();

    // TODO: Use the protocol to deserialize?
    // Add insta for snapshot tests
    assert_debug_snapshot!(
        "set_pin_high - bytes",
        &msg
    );

    assert_debug_snapshot!(
        "set_pin_high - parsed",
        &deser_msg
    );

    assert!(test_hdl.is_totally_empty());
}

#[test]
fn set_pin_low() {
    let mock = Mock::new();
    let test_hdl = mock.clone();

    let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
    let mut assistant = Assistant::new(conn);
    assistant.set_pin_low().unwrap();

    let msg = test_hdl.pop_host_lib_data().unwrap();
    let mut msg_clone = msg.clone();
    let deser_msg: HostToAssistant = postcard::from_bytes_cobs(&mut msg_clone).unwrap();

    // TODO: Use the protocol to deserialize?
    // Add insta for snapshot tests
    assert_debug_snapshot!(
        "set_pin_low - bytes",
        &msg
    );

    assert_debug_snapshot!(
        "set_pin_low - parsed",
        &deser_msg
    );

    assert!(test_hdl.is_totally_empty());
}

#[test]
fn disable_cts() {
    let mock = Mock::new();
    let test_hdl = mock.clone();

    let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
    let mut assistant = Assistant::new(conn);
    assistant.disable_cts().unwrap();

    let msg = test_hdl.pop_host_lib_data().unwrap();
    let mut msg_clone = msg.clone();
    let deser_msg: HostToAssistant = postcard::from_bytes_cobs(&mut msg_clone).unwrap();

    // TODO: Use the protocol to deserialize?
    // Add insta for snapshot tests
    assert_debug_snapshot!(
        "disable_cts - bytes",
        &msg
    );

    assert_debug_snapshot!(
        "disable_cts - parsed",
        &deser_msg
    );

    assert!(test_hdl.is_totally_empty());
}

#[test]
fn enable_cts() {
    let mock = Mock::new();
    let test_hdl = mock.clone();

    let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
    let mut assistant = Assistant::new(conn);
    assistant.enable_cts().unwrap();

    let msg = test_hdl.pop_host_lib_data().unwrap();
    let mut msg_clone = msg.clone();
    let deser_msg: HostToAssistant = postcard::from_bytes_cobs(&mut msg_clone).unwrap();

    // TODO: Use the protocol to deserialize?
    // Add insta for snapshot tests
    assert_debug_snapshot!(
        "enable_cts - bytes",
        &msg
    );

    assert_debug_snapshot!(
        "enable_cts - parsed",
        &deser_msg
    );

    assert!(test_hdl.is_totally_empty());
}
