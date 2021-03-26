use host_lib::{
    mock::Mock,
    assistant::{Assistant, GpioPeriodMeasurement},
    conn::Conn,
};
use protocol::{UsartMode, AssistantToHost, HostToAssistant, InputPin, pin::{Level, ReadLevelResult}};
use postcard;
use std::time::Duration;

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

#[test]
fn pin_is_high() {
    // SET UP TEST
    let mock = Mock::new();
    let test_hdl = mock.clone();

    let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
    let mut assistant = Assistant::new(conn);

    let tst_result = AssistantToHost::ReadPinResult(Some(ReadLevelResult{pin: InputPin::Green, level: Level::High, period_ms: None}));
    let serialized_response = postcard::to_stdvec_cobs(&tst_result).unwrap();

    // add fake response
    test_hdl.push_fake_ta_data(&serialized_response);

    // RUN TEST #1
    assert!(assistant.pin_is_high().unwrap());

    // observe host lib behavior
    let msg = test_hdl.pop_host_lib_data().unwrap();
    let mut msg_clone = msg.clone();
    let deser_msg: HostToAssistant = postcard::from_bytes_cobs(&mut msg_clone).unwrap();

    assert_debug_snapshot!(
        "pin_is_high - bytes",
        &msg
    );

    assert_debug_snapshot!(
        "pin_is_high - parsed",
        &deser_msg
    );

    // ASSERT POSTCONDITION
    assert!(test_hdl.is_totally_empty());
}

#[test]
fn pin_is_low() {
    // SET UP TEST
    let mock = Mock::new();
    let test_hdl = mock.clone();

    let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
    let mut assistant = Assistant::new(conn);

    let tst_result = AssistantToHost::ReadPinResult(Some(ReadLevelResult{pin: InputPin::Green, level: Level::Low, period_ms: None}));
    let serialized_response = postcard::to_stdvec_cobs(&tst_result).unwrap();

    // add fake response
    test_hdl.push_fake_ta_data(&serialized_response);

    // RUN TEST #1
    assert!(assistant.pin_is_low().unwrap());

    // observe host lib behavior
    let msg = test_hdl.pop_host_lib_data().unwrap();
    let mut msg_clone = msg.clone();
    let deser_msg: HostToAssistant = postcard::from_bytes_cobs(&mut msg_clone).unwrap();

    assert_debug_snapshot!(
        "pin_is_low - bytes",
        &msg
    );

    assert_debug_snapshot!(
        "pin_is_low - parsed",
        &deser_msg
    );

    // ASSERT POSTCONDITION
    assert!(test_hdl.is_totally_empty());
}

#[test]
fn wait_for_rts() {
    // SET UP TEST
    let mock = Mock::new();
    let test_hdl = mock.clone();

    let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
    let mut assistant = Assistant::new(conn);

    let tst_result = AssistantToHost::ReadPinResult(Some(ReadLevelResult{pin: InputPin::Rts, level: Level::Low, period_ms: None}));
    let serialized_response = postcard::to_stdvec_cobs(&tst_result).unwrap();

    // add fake response
    test_hdl.push_fake_ta_data(&serialized_response);

    // RUN TEST #1
    assert!(assistant.wait_for_rts().unwrap());

    // observe host lib behavior
    let msg = test_hdl.pop_host_lib_data().unwrap();
    let mut msg_clone = msg.clone();
    let deser_msg: HostToAssistant = postcard::from_bytes_cobs(&mut msg_clone).unwrap();

    assert_debug_snapshot!(
        "wait_for_rts - bytes",
        &msg
    );

    assert_debug_snapshot!(
        "wait_for_rts - parsed",
        &deser_msg
    );

    // ASSERT POSTCONDITION
    assert!(test_hdl.is_totally_empty());
}

#[test]
fn send_to_target_usart() {
    let mock = Mock::new();
    let test_hdl = mock.clone();

    let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
    let mut assistant = Assistant::new(conn);
    let data = &[0xFE, 0xED, 0xDA, 0x7A];
    assistant.send_to_target_usart(data).unwrap();

    let msg = test_hdl.pop_host_lib_data().unwrap();
    let mut msg_clone = msg.clone();
    let deser_msg: HostToAssistant = postcard::from_bytes_cobs(&mut msg_clone).unwrap();

    // TODO: Use the protocol to deserialize?
    assert_debug_snapshot!(
        "send_to_target_usart - bytes",
        &msg
    );

    assert_debug_snapshot!(
        "send_to_target_usart - parsed",
        &deser_msg
    );

    assert!(test_hdl.is_totally_empty());
}

#[test]
fn send_to_target_usart_dma() {
    let mock = Mock::new();
    let test_hdl = mock.clone();

    let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
    let mut assistant = Assistant::new(conn);
    let data = &[0xFE, 0xED, 0xDA, 0x7A];
    assistant.send_to_target_usart_dma(data).unwrap();

    let msg = test_hdl.pop_host_lib_data().unwrap();
    let mut msg_clone = msg.clone();
    let deser_msg: HostToAssistant = postcard::from_bytes_cobs(&mut msg_clone).unwrap();

    // TODO: Use the protocol to deserialize?
    assert_debug_snapshot!(
        "send_to_target_usart_dma - bytes",
        &msg
    );

    assert_debug_snapshot!(
        "send_to_target_usart_dma - parsed",
        &deser_msg
    );

    assert!(test_hdl.is_totally_empty());
}

#[test]
fn send_to_target_usart_sync() {
    let mock = Mock::new();
    let test_hdl = mock.clone();

    let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
    let mut assistant = Assistant::new(conn);
    let data = &[0xFE, 0xED, 0xDA, 0x7A];
    assistant.send_to_target_usart_sync(data).unwrap();

    let msg = test_hdl.pop_host_lib_data().unwrap();
    let mut msg_clone = msg.clone();
    let deser_msg: HostToAssistant = postcard::from_bytes_cobs(&mut msg_clone).unwrap();

    // TODO: Use the protocol to deserialize?
    assert_debug_snapshot!(
        "send_to_target_usart_sync - bytes",
        &msg
    );

    assert_debug_snapshot!(
        "send_to_target_usart_sync - parsed",
        &deser_msg
    );

    assert!(test_hdl.is_totally_empty());
}

#[test]
fn receive_from_target_usart() {
    // SET UP TEST
    let mock = Mock::new();
    let test_hdl = mock.clone();

    let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
    let mut assistant = Assistant::new(conn);

    let data = &[0xFE, 0xED, 0xDA, 0x7A];

    let tst_result = AssistantToHost::UsartReceive {
        mode: UsartMode::Regular,
        data,
    };
    let serialized_response = postcard::to_stdvec_cobs(&tst_result).unwrap();

    // add fake response
    test_hdl.push_fake_ta_data(&serialized_response);

    // RUN TEST #1
    assert_eq!(
        assistant
            .receive_from_target_usart(
                data, Duration::from_millis(100)
            ).unwrap().as_slice(),
        data
    );

    // observe host lib behavior
    // NOTE: Host never sends data for this.

    // ASSERT POSTCONDITION
    assert!(test_hdl.is_totally_empty());
}

#[test]
fn receive_from_target_usart_sync() {
    // SET UP TEST
    let mock = Mock::new();
    let test_hdl = mock.clone();

    let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
    let mut assistant = Assistant::new(conn);

    let data = &[0xFE, 0xED, 0xDA, 0x7A];

    let tst_result = AssistantToHost::UsartReceive {
        mode: UsartMode::Sync,
        data,
    };
    let serialized_response = postcard::to_stdvec_cobs(&tst_result).unwrap();

    // add fake response
    test_hdl.push_fake_ta_data(&serialized_response);

    // RUN TEST #1
    assert_eq!(
        assistant
            .receive_from_target_usart_sync(
                data, Duration::from_millis(100)
            ).unwrap().as_slice(),
        data
    );

    // observe host lib behavior
    // NOTE: Host never sends data for this.

    // ASSERT POSTCONDITION
    assert!(test_hdl.is_totally_empty());
}


#[test]
fn receive_from_target_usart_inner() {
    // SET UP TEST
    let mock = Mock::new();
    let test_hdl = mock.clone();

    let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
    let mut assistant = Assistant::new(conn);

    let data = &[0xFE, 0xED, 0xDA, 0x7A];

    let tst_result = AssistantToHost::UsartReceive {
        mode: UsartMode::Dma,
        data,
    };
    let serialized_response = postcard::to_stdvec_cobs(&tst_result).unwrap();

    // add fake response
    test_hdl.push_fake_ta_data(&serialized_response);

    // RUN TEST #1
    assert_eq!(
        assistant
            .receive_from_target_usart_inner(
                data, Duration::from_millis(100), UsartMode::Dma
            ).unwrap().as_slice(),
        data
    );

    // observe host lib behavior
    // NOTE: Host never sends data for this.

    // ASSERT POSTCONDITION
    assert!(test_hdl.is_totally_empty());
}


#[test]
fn measure_timer_interrupt() {
    // SET UP TEST
    let mock = Mock::new();
    let test_hdl = mock.clone();

    let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
    let mut assistant = Assistant::new(conn);

    let tst_result_1 = AssistantToHost::ReadPinResult(Some(ReadLevelResult{pin: InputPin::Blue, level: Level::High, period_ms: Some(10)}));
    let serialized_response_1 = postcard::to_stdvec_cobs(&tst_result_1).unwrap();

    let tst_result_2 = AssistantToHost::ReadPinResult(Some(ReadLevelResult{pin: InputPin::Blue, level: Level::Low, period_ms: Some(10)}));
    let serialized_response_2 = postcard::to_stdvec_cobs(&tst_result_2).unwrap();

    // add fake response
    test_hdl.push_fake_ta_data(&serialized_response_1);
    test_hdl.push_fake_ta_data(&serialized_response_2);

    // RUN TEST #1
    let data = assistant.measure_timer_interrupt(1, Duration::from_millis(100)).unwrap();
    assert_eq!(
        data,
        GpioPeriodMeasurement {
            min: Duration::from_millis(10),
            max: Duration::from_millis(10),
        }
    );

    // observe host lib behavior
    for _ in 0..2 {
        let msg = test_hdl.pop_host_lib_data().unwrap();
        let mut msg_clone = msg.clone();
        let deser_msg: HostToAssistant = postcard::from_bytes_cobs(&mut msg_clone).unwrap();

        assert_debug_snapshot!(
            "measure_timer_interrupt - bytes",
            &msg
        );

        assert_debug_snapshot!(
            "measure_timer_interrupt - parsed",
            &deser_msg
        );
    }

    // ASSERT POSTCONDITION
    assert!(test_hdl.is_totally_empty());
}

#[test]
fn measure_pwm_signal() {
    // SET UP TEST
    let mock = Mock::new();
    let test_hdl = mock.clone();

    let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
    let mut assistant = Assistant::new(conn);

    let tst_result_1 = AssistantToHost::ReadPinResult(Some(ReadLevelResult{pin: InputPin::Pwm, level: Level::High, period_ms: Some(10)}));
    let serialized_response_1 = postcard::to_stdvec_cobs(&tst_result_1).unwrap();

    let tst_result_2 = AssistantToHost::ReadPinResult(Some(ReadLevelResult{pin: InputPin::Pwm, level: Level::Low, period_ms: Some(10)}));
    let serialized_response_2 = postcard::to_stdvec_cobs(&tst_result_2).unwrap();

    // add fake response
    test_hdl.push_fake_ta_data(&serialized_response_1);
    test_hdl.push_fake_ta_data(&serialized_response_2);

    // RUN TEST #1
    let data = assistant.measure_pwm_signal(1, Duration::from_millis(100)).unwrap();
    assert_eq!(
        data,
        GpioPeriodMeasurement {
            min: Duration::from_millis(10),
            max: Duration::from_millis(10),
        }
    );


    // observe host lib behavior
    for _ in 0..2 {
        let msg = test_hdl.pop_host_lib_data().unwrap();
        let mut msg_clone = msg.clone();
        let deser_msg: HostToAssistant = postcard::from_bytes_cobs(&mut msg_clone).unwrap();

        assert_debug_snapshot!(
            "measure_pwm_signal - bytes",
            &msg
        );

        assert_debug_snapshot!(
            "measure_pwm_signal - parsed",
            &deser_msg
        );
    }

    // ASSERT POSTCONDITION
    assert!(test_hdl.is_totally_empty());
}

#[test]
fn expect_nothing_from_target() {
    // SET UP TEST
    let mock = Mock::new();
    let test_hdl = mock.clone();

    let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
    let mut assistant = Assistant::new(conn);

    // RUN TEST #1
    assistant.expect_nothing_from_target(Duration::from_millis(100)).unwrap();

    // RUN TEST #2
    let tst_result_2 = AssistantToHost::ReadPinResult(Some(ReadLevelResult{pin: InputPin::Pwm, level: Level::Low, period_ms: Some(10)}));
    let serialized_response_2 = postcard::to_stdvec_cobs(&tst_result_2).unwrap();

    // add fake response
    test_hdl.push_fake_ta_data(&serialized_response_2);

    assert!(assistant.expect_nothing_from_target(Duration::from_millis(100)).is_err());


    // observe host lib behavior
    // NOTE: Host doesn't send anything

    // ASSERT POSTCONDITION
    assert!(test_hdl.is_totally_empty());
}
