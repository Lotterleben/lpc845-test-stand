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
