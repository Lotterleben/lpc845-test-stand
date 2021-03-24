use host_lib::{
    mock::Mock,
    assistant::Assistant,
    conn::Conn,
};

#[test]
fn make_instance() {
    let mock = Mock { };
    let conn = Conn::from_serial_port(Box::new(mock)).unwrap();

    Assistant::new(conn);
}
