use host_lib::{
    mock::Mock,
    assistant::Assistant,
    conn::Conn,
};

#[test]
fn make_instance() {
    let mock = Mock::new();
    let test_hdl = mock.clone();

    let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
    let assistant = Assistant::new(conn);

    assert!(test_hdl.is_totally_empty());
}

#[test]
fn set_pin_5_high() {
    let mock = Mock::new();
    let test_hdl = mock.clone();

    let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
    let mut assistant = Assistant::new(conn);
    assistant.set_pin_5_high().unwrap();

    // TODO: Use the protocol to deserialize?
    // Add insta for snapshot tests
    assert_eq!(b"", test_hdl.pop_host_lib_data().unwrap().as_slice());

    assert!(test_hdl.is_totally_empty());
}
