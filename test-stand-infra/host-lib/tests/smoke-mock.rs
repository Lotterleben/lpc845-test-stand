use host_lib::{
    assistant::{Assistant, AssistantInterface, GpioPeriodMeasurement},
    conn::Conn,
    test_stand::NotConfiguredError,
    test_stand::TestStand,
};
use postcard;
use protocol::{
    pin::{Level, ReadLevelResult},
    AssistantToHost, HostToAssistant, InputPin, UsartMode,
};
use std::time::Duration;

use insta::assert_debug_snapshot;

#[test]
fn make_instance() {
    let mock = MockConn::new();
    let test_hdl = mock.clone();

    let assistant_conn = Conn::from_serial_port(Box::new(mock)).unwrap();

    let test_stand =
        TestStand::new_with_connection(Ok(assistant_conn), Err(NotConfiguredError("")));

    assert!(test_hdl.is_totally_empty());
}

// #[test]
// fn set_pin_5_high() {
//     let mock = MockConn::new();
//     let test_hdl = mock.clone();

//     let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
//     let assistant_conn = Conn::from_serial_port(Box::new(mock)).unwrap();

//     let test_stand =
//         TestStand::new_with_connection(Ok(assistant_conn), Err(NotConfiguredError(""))).unwrap();

//     test_stand.assistant.set_pin_5_high().unwrap();

//     let msg = test_hdl.pop_host_lib_data().unwrap();
//     let mut msg_clone = msg.clone();
//     let deser_msg: HostToAssistant = postcard::from_bytes_cobs(&mut msg_clone).unwrap();

//     assert_debug_snapshot!(
//         "set_pin_5_high - bytes",
//         &msg
//     );

//     assert_debug_snapshot!(
//         "set_pin_5_high - parsed",
//         &deser_msg
//     );

//     assert!(test_hdl.is_totally_empty());
// }

// #[test]
// fn set_pin_5_low() {
//     let mock = MockConn::new();
//     let test_hdl = mock.clone();

//     let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
//     let mut assistant = Assistant::new(conn);
//     assistant.set_pin_5_low().unwrap();

//     let msg = test_hdl.pop_host_lib_data().unwrap();
//     let mut msg_clone = msg.clone();
//     let deser_msg: HostToAssistant = postcard::from_bytes_cobs(&mut msg_clone).unwrap();

//     // TODO: Use the protocol to deserialize?
//     assert_debug_snapshot!(
//         "set_pin_5_low - bytes",
//         &msg
//     );

//     assert_debug_snapshot!(
//         "set_pin_5_low - parsed",
//         &deser_msg
//     );

//     assert!(test_hdl.is_totally_empty());
// }

// #[test]
// fn set_pin_high() {
//     let mock = MockConn::new();
//     let test_hdl = mock.clone();

//     let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
//     let mut assistant = Assistant::new(conn);
//     assistant.set_pin_high().unwrap();

//     let msg = test_hdl.pop_host_lib_data().unwrap();
//     let mut msg_clone = msg.clone();
//     let deser_msg: HostToAssistant = postcard::from_bytes_cobs(&mut msg_clone).unwrap();

//     // TODO: Use the protocol to deserialize?
//     assert_debug_snapshot!(
//         "set_pin_high - bytes",
//         &msg
//     );

//     assert_debug_snapshot!(
//         "set_pin_high - parsed",
//         &deser_msg
//     );

//     assert!(test_hdl.is_totally_empty());
// }

// #[test]
// fn set_pin_low() {
//     let mock = MockConn::new();
//     let test_hdl = mock.clone();

//     let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
//     let mut assistant = Assistant::new(conn);
//     assistant.set_pin_low().unwrap();

//     let msg = test_hdl.pop_host_lib_data().unwrap();
//     let mut msg_clone = msg.clone();
//     let deser_msg: HostToAssistant = postcard::from_bytes_cobs(&mut msg_clone).unwrap();

//     // TODO: Use the protocol to deserialize?
//     assert_debug_snapshot!(
//         "set_pin_low - bytes",
//         &msg
//     );

//     assert_debug_snapshot!(
//         "set_pin_low - parsed",
//         &deser_msg
//     );

//     assert!(test_hdl.is_totally_empty());
// }

#[test]
fn disable_cts() {
    let mock = MockConn::new();
    let test_hdl = mock.clone();

    let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
    let assistant = Assistant::new(conn, 40);

    let mut test_stand =
         TestStand::new_with_connection(Err(NotConfiguredError("")), Ok(assistant)).unwrap();

    // TODO smelly interface. re-think
    let assistant = test_stand.assistant.as_mut().unwrap();

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

// #[test]
// fn enable_cts() {
//     let mock = MockConn::new();
//     let test_hdl = mock.clone();

//     let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
//     let mut assistant = Assistant::new(conn);
//     assistant.enable_cts().unwrap();

//     let msg = test_hdl.pop_host_lib_data().unwrap();
//     let mut msg_clone = msg.clone();
//     let deser_msg: HostToAssistant = postcard::from_bytes_cobs(&mut msg_clone).unwrap();

//     // TODO: Use the protocol to deserialize?
//     assert_debug_snapshot!(
//         "enable_cts - bytes",
//         &msg
//     );

//     assert_debug_snapshot!(
//         "enable_cts - parsed",
//         &deser_msg
//     );

//     assert!(test_hdl.is_totally_empty());
// }

// #[test]
// fn pin_is_high() {
//     // SET UP TEST
//     let mock = MockConn::new();
//     let test_hdl = mock.clone();

//     let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
//     let mut assistant = Assistant::new(conn);

//     let tst_result = AssistantToHost::ReadPinResult(Some(ReadLevelResult{pin: InputPin::Green, level: Level::High, period_ms: None}));
//     let serialized_response = postcard::to_stdvec_cobs(&tst_result).unwrap();

//     // add fake response
//     test_hdl.push_fake_ta_data(&serialized_response);

//     // RUN TEST #1
//     assert!(assistant.pin_is_high().unwrap());

//     // observe host lib behavior
//     let msg = test_hdl.pop_host_lib_data().unwrap();
//     let mut msg_clone = msg.clone();
//     let deser_msg: HostToAssistant = postcard::from_bytes_cobs(&mut msg_clone).unwrap();

//     assert_debug_snapshot!(
//         "pin_is_high - bytes",
//         &msg
//     );

//     assert_debug_snapshot!(
//         "pin_is_high - parsed",
//         &deser_msg
//     );

//     // ASSERT POSTCONDITION
//     assert!(test_hdl.is_totally_empty());
// }

// #[test]
// fn pin_is_low() {
//     // SET UP TEST
//     let mock = MockConn::new();
//     let test_hdl = mock.clone();

//     let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
//     let mut assistant = Assistant::new(conn);

//     let tst_result = AssistantToHost::ReadPinResult(Some(ReadLevelResult{pin: InputPin::Green, level: Level::Low, period_ms: None}));
//     let serialized_response = postcard::to_stdvec_cobs(&tst_result).unwrap();

//     // add fake response
//     test_hdl.push_fake_ta_data(&serialized_response);

//     // RUN TEST #1
//     assert!(assistant.pin_is_low().unwrap());

//     // observe host lib behavior
//     let msg = test_hdl.pop_host_lib_data().unwrap();
//     let mut msg_clone = msg.clone();
//     let deser_msg: HostToAssistant = postcard::from_bytes_cobs(&mut msg_clone).unwrap();

//     assert_debug_snapshot!(
//         "pin_is_low - bytes",
//         &msg
//     );

//     assert_debug_snapshot!(
//         "pin_is_low - parsed",
//         &deser_msg
//     );

//     // ASSERT POSTCONDITION
//     assert!(test_hdl.is_totally_empty());
// }

// #[test]
// fn wait_for_rts() {
//     // SET UP TEST
//     let mock = MockConn::new();
//     let test_hdl = mock.clone();

//     let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
//     let mut assistant = Assistant::new(conn);

//     let tst_result = AssistantToHost::ReadPinResult(Some(ReadLevelResult{pin: InputPin::Rts, level: Level::Low, period_ms: None}));
//     let serialized_response = postcard::to_stdvec_cobs(&tst_result).unwrap();

//     // add fake response
//     test_hdl.push_fake_ta_data(&serialized_response);

//     // RUN TEST #1
//     assert!(assistant.wait_for_rts().unwrap());

//     // observe host lib behavior
//     let msg = test_hdl.pop_host_lib_data().unwrap();
//     let mut msg_clone = msg.clone();
//     let deser_msg: HostToAssistant = postcard::from_bytes_cobs(&mut msg_clone).unwrap();

//     assert_debug_snapshot!(
//         "wait_for_rts - bytes",
//         &msg
//     );

//     assert_debug_snapshot!(
//         "wait_for_rts - parsed",
//         &deser_msg
//     );

//     // ASSERT POSTCONDITION
//     assert!(test_hdl.is_totally_empty());
// }

// #[test]
// fn send_to_target_usart() {
//     let mock = MockConn::new();
//     let test_hdl = mock.clone();

//     let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
//     let mut assistant = Assistant::new(conn);
//     let data = &[0xFE, 0xED, 0xDA, 0x7A];
//     assistant.send_to_target_usart(data).unwrap();

//     let msg = test_hdl.pop_host_lib_data().unwrap();
//     let mut msg_clone = msg.clone();
//     let deser_msg: HostToAssistant = postcard::from_bytes_cobs(&mut msg_clone).unwrap();

//     // TODO: Use the protocol to deserialize?
//     assert_debug_snapshot!(
//         "send_to_target_usart - bytes",
//         &msg
//     );

//     assert_debug_snapshot!(
//         "send_to_target_usart - parsed",
//         &deser_msg
//     );

//     assert!(test_hdl.is_totally_empty());
// }

// #[test]
// fn send_to_target_usart_dma() {
//     let mock = MockConn::new();
//     let test_hdl = mock.clone();

//     let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
//     let mut assistant = Assistant::new(conn);
//     let data = &[0xFE, 0xED, 0xDA, 0x7A];
//     assistant.send_to_target_usart_dma(data).unwrap();

//     let msg = test_hdl.pop_host_lib_data().unwrap();
//     let mut msg_clone = msg.clone();
//     let deser_msg: HostToAssistant = postcard::from_bytes_cobs(&mut msg_clone).unwrap();

//     // TODO: Use the protocol to deserialize?
//     assert_debug_snapshot!(
//         "send_to_target_usart_dma - bytes",
//         &msg
//     );

//     assert_debug_snapshot!(
//         "send_to_target_usart_dma - parsed",
//         &deser_msg
//     );

//     assert!(test_hdl.is_totally_empty());
// }

// #[test]
// fn send_to_target_usart_sync() {
//     let mock = MockConn::new();
//     let test_hdl = mock.clone();

//     let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
//     let mut assistant = Assistant::new(conn);
//     let data = &[0xFE, 0xED, 0xDA, 0x7A];
//     assistant.send_to_target_usart_sync(data).unwrap();

//     let msg = test_hdl.pop_host_lib_data().unwrap();
//     let mut msg_clone = msg.clone();
//     let deser_msg: HostToAssistant = postcard::from_bytes_cobs(&mut msg_clone).unwrap();

//     // TODO: Use the protocol to deserialize?
//     assert_debug_snapshot!(
//         "send_to_target_usart_sync - bytes",
//         &msg
//     );

//     assert_debug_snapshot!(
//         "send_to_target_usart_sync - parsed",
//         &deser_msg
//     );

//     assert!(test_hdl.is_totally_empty());
// }

// #[test]
// fn receive_from_target_usart() {
//     // SET UP TEST
//     let mock = MockConn::new();
//     let test_hdl = mock.clone();

//     let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
//     let mut assistant = Assistant::new(conn);

//     let data = &[0xFE, 0xED, 0xDA, 0x7A];

//     let tst_result = AssistantToHost::UsartReceive {
//         mode: UsartMode::Regular,
//         data,
//     };
//     let serialized_response = postcard::to_stdvec_cobs(&tst_result).unwrap();

//     // add fake response
//     test_hdl.push_fake_ta_data(&serialized_response);

//     // RUN TEST #1
//     assert_eq!(
//         assistant
//             .receive_from_target_usart(
//                 data, Duration::from_millis(100)
//             ).unwrap().as_slice(),
//         data
//     );

//     // observe host lib behavior
//     // NOTE: Host never sends data for this.

//     // ASSERT POSTCONDITION
//     assert!(test_hdl.is_totally_empty());
// }

// #[test]
// fn receive_from_target_usart_sync() {
//     // SET UP TEST
//     let mock = MockConn::new();
//     let test_hdl = mock.clone();

//     let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
//     let mut assistant = Assistant::new(conn);

//     let data = &[0xFE, 0xED, 0xDA, 0x7A];

//     let tst_result = AssistantToHost::UsartReceive {
//         mode: UsartMode::Sync,
//         data,
//     };
//     let serialized_response = postcard::to_stdvec_cobs(&tst_result).unwrap();

//     // add fake response
//     test_hdl.push_fake_ta_data(&serialized_response);

//     // RUN TEST #1
//     assert_eq!(
//         assistant
//             .receive_from_target_usart_sync(
//                 data, Duration::from_millis(100)
//             ).unwrap().as_slice(),
//         data
//     );

//     // observe host lib behavior
//     // NOTE: Host never sends data for this.

//     // ASSERT POSTCONDITION
//     assert!(test_hdl.is_totally_empty());
// }

// #[test]
// fn receive_from_target_usart_inner() {
//     // SET UP TEST
//     let mock = MockConn::new();
//     let test_hdl = mock.clone();

//     let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
//     let mut assistant = Assistant::new(conn);

//     let data = &[0xFE, 0xED, 0xDA, 0x7A];

//     let tst_result = AssistantToHost::UsartReceive {
//         mode: UsartMode::Dma,
//         data,
//     };
//     let serialized_response = postcard::to_stdvec_cobs(&tst_result).unwrap();

//     // add fake response
//     test_hdl.push_fake_ta_data(&serialized_response);

//     // RUN TEST #1
//     assert_eq!(
//         assistant
//             .receive_from_target_usart_inner(
//                 data, Duration::from_millis(100), UsartMode::Dma
//             ).unwrap().as_slice(),
//         data
//     );

//     // observe host lib behavior
//     // NOTE: Host never sends data for this.

//     // ASSERT POSTCONDITION
//     assert!(test_hdl.is_totally_empty());
// }

// #[test]
// fn measure_timer_interrupt() {
//     // SET UP TEST
//     let mock = MockConn::new();
//     let test_hdl = mock.clone();

//     let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
//     let mut assistant = Assistant::new(conn);

//     let tst_result_1 = AssistantToHost::ReadPinResult(Some(ReadLevelResult{pin: InputPin::Blue, level: Level::High, period_ms: Some(10)}));
//     let serialized_response_1 = postcard::to_stdvec_cobs(&tst_result_1).unwrap();

//     let tst_result_2 = AssistantToHost::ReadPinResult(Some(ReadLevelResult{pin: InputPin::Blue, level: Level::Low, period_ms: Some(10)}));
//     let serialized_response_2 = postcard::to_stdvec_cobs(&tst_result_2).unwrap();

//     // add fake response
//     test_hdl.push_fake_ta_data(&serialized_response_1);
//     test_hdl.push_fake_ta_data(&serialized_response_2);

//     // RUN TEST #1
//     let data = assistant.measure_timer_interrupt(1, Duration::from_millis(100)).unwrap();
//     assert_eq!(
//         data,
//         GpioPeriodMeasurement {
//             min: Duration::from_millis(10),
//             max: Duration::from_millis(10),
//         }
//     );

//     // observe host lib behavior
//     for _ in 0..2 {
//         let msg = test_hdl.pop_host_lib_data().unwrap();
//         let mut msg_clone = msg.clone();
//         let deser_msg: HostToAssistant = postcard::from_bytes_cobs(&mut msg_clone).unwrap();

//         assert_debug_snapshot!(
//             "measure_timer_interrupt - bytes",
//             &msg
//         );

//         assert_debug_snapshot!(
//             "measure_timer_interrupt - parsed",
//             &deser_msg
//         );
//     }

//     // ASSERT POSTCONDITION
//     assert!(test_hdl.is_totally_empty());
// }

// #[test]
// fn measure_pwm_signal() {
//     // SET UP TEST
//     let mock = MockConn::new();
//     let test_hdl = mock.clone();

//     let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
//     let mut assistant = Assistant::new(conn);

//     let tst_result_1 = AssistantToHost::ReadPinResult(Some(ReadLevelResult{pin: InputPin::Pwm, level: Level::High, period_ms: Some(10)}));
//     let serialized_response_1 = postcard::to_stdvec_cobs(&tst_result_1).unwrap();

//     let tst_result_2 = AssistantToHost::ReadPinResult(Some(ReadLevelResult{pin: InputPin::Pwm, level: Level::Low, period_ms: Some(10)}));
//     let serialized_response_2 = postcard::to_stdvec_cobs(&tst_result_2).unwrap();

//     // add fake response
//     test_hdl.push_fake_ta_data(&serialized_response_1);
//     test_hdl.push_fake_ta_data(&serialized_response_2);

//     // RUN TEST #1
//     let data = assistant.measure_pwm_signal(1, Duration::from_millis(100)).unwrap();
//     assert_eq!(
//         data,
//         GpioPeriodMeasurement {
//             min: Duration::from_millis(10),
//             max: Duration::from_millis(10),
//         }
//     );

//     // observe host lib behavior
//     for _ in 0..2 {
//         let msg = test_hdl.pop_host_lib_data().unwrap();
//         let mut msg_clone = msg.clone();
//         let deser_msg: HostToAssistant = postcard::from_bytes_cobs(&mut msg_clone).unwrap();

//         assert_debug_snapshot!(
//             "measure_pwm_signal - bytes",
//             &msg
//         );

//         assert_debug_snapshot!(
//             "measure_pwm_signal - parsed",
//             &deser_msg
//         );
//     }

//     // ASSERT POSTCONDITION
//     assert!(test_hdl.is_totally_empty());
// }

// #[test]
// fn expect_nothing_from_target() {
//     // SET UP TEST
//     let mock = MockConn::new();
//     let test_hdl = mock.clone();

//     let conn = Conn::from_serial_port(Box::new(mock)).unwrap();
//     let mut assistant = Assistant::new(conn);

//     // RUN TEST #1
//     assistant.expect_nothing_from_target(Duration::from_millis(100)).unwrap();

//     // RUN TEST #2
//     let tst_result_2 = AssistantToHost::ReadPinResult(Some(ReadLevelResult{pin: InputPin::Pwm, level: Level::Low, period_ms: Some(10)}));
//     let serialized_response_2 = postcard::to_stdvec_cobs(&tst_result_2).unwrap();

//     // add fake response
//     test_hdl.push_fake_ta_data(&serialized_response_2);

//     assert!(assistant.expect_nothing_from_target(Duration::from_millis(100)).is_err());

//     // observe host lib behavior
//     // NOTE: Host doesn't send anything

//     // ASSERT POSTCONDITION
//     assert!(test_hdl.is_totally_empty());
// }

// MOCK OF THE `Conn` object from the main crate
// TODO(AJM) - move to a file in the `tests/` folder

use serialport::{
    ClearBuffer, DataBits, Error as SpError, FlowControl, Parity, SerialPort, StopBits,
};
use std::io::{Error as IoError, ErrorKind};
use std::io::{Read, Write};

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub struct MockConn {
    // Data SENT by the PC TO the TA
    data_out: Arc<Mutex<VecDeque<Vec<u8>>>>,

    // Data RECEIVED by the PC FROM the TA
    data_in: Arc<Mutex<VecDeque<Vec<u8>>>>,
}

impl Clone for MockConn {
    fn clone(&self) -> Self {
        Self {
            data_in: self.data_in.clone(),
            data_out: self.data_out.clone(),
        }
    }
}

impl MockConn {
    pub fn new() -> Self {
        Self {
            data_out: Arc::new(Mutex::new(VecDeque::new())),
            data_in: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn is_totally_empty(&self) -> bool {
        let in_empty = if let Ok(vd) = self.data_in.lock() {
            vd.is_empty()
        } else {
            todo!()
        };

        let out_empty = if let Ok(vd) = self.data_out.lock() {
            vd.is_empty()
        } else {
            todo!()
        };

        in_empty && out_empty
    }

    pub fn push_fake_ta_data(&self, data: &[u8]) {
        if let Ok(mut vd) = self.data_in.lock() {
            vd.push_front(data.to_vec());
        } else {
            todo!()
        }
    }

    pub fn pop_host_lib_data(&self) -> Option<Vec<u8>> {
        if let Ok(mut vd) = self.data_out.lock() {
            vd.pop_back()
        } else {
            todo!()
        }
    }
}

impl Read for MockConn {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, IoError> {
        if let Ok(mut vd) = self.data_in.lock() {
            if let Some(data) = vd.pop_front() {
                // support partial reads
                let actual_size = buf.len().min(data.len());
                let (start, end) = data.split_at(actual_size);

                buf[..actual_size].copy_from_slice(start);

                if !end.is_empty() {
                    let end = end.to_vec();
                    vd.push_front(end);
                }
                Ok(actual_size)
            } else {
                return Err(IoError::new(ErrorKind::TimedOut, "Hello"));
            }
        } else {
            todo!()
        }
    }
}

impl Write for MockConn {
    fn write(&mut self, buf: &[u8]) -> Result<usize, IoError> {
        if let Ok(mut vd) = self.data_out.lock() {
            vd.push_back(buf.to_vec());
            Ok(buf.len())
        } else {
            todo!()
        }
    }
    fn flush(&mut self) -> Result<(), IoError> {
        Ok(())
    }
}

impl SerialPort for MockConn {
    fn name(&self) -> std::option::Option<std::string::String> {
        todo!()
    }
    fn baud_rate(&self) -> Result<u32, SpError> {
        todo!()
    }
    fn data_bits(&self) -> Result<DataBits, SpError> {
        todo!()
    }
    fn flow_control(&self) -> Result<FlowControl, SpError> {
        todo!()
    }
    fn parity(&self) -> Result<Parity, SpError> {
        todo!()
    }
    fn stop_bits(&self) -> Result<StopBits, SpError> {
        todo!()
    }
    fn timeout(&self) -> Duration {
        todo!()
    }
    fn set_baud_rate(&mut self, _: u32) -> Result<(), SpError> {
        todo!()
    }
    fn set_data_bits(&mut self, _: DataBits) -> Result<(), SpError> {
        todo!()
    }
    fn set_flow_control(&mut self, _: FlowControl) -> Result<(), SpError> {
        todo!()
    }
    fn set_parity(&mut self, _: Parity) -> Result<(), SpError> {
        todo!()
    }
    fn set_stop_bits(&mut self, _: StopBits) -> Result<(), SpError> {
        todo!()
    }
    fn set_timeout(&mut self, _: Duration) -> Result<(), SpError> {
        // ignore timeouts
        Ok(())
    }
    fn write_request_to_send(&mut self, _: bool) -> Result<(), SpError> {
        todo!()
    }
    fn write_data_terminal_ready(&mut self, _: bool) -> Result<(), SpError> {
        todo!()
    }
    fn read_clear_to_send(&mut self) -> Result<bool, SpError> {
        todo!()
    }
    fn read_data_set_ready(&mut self) -> Result<bool, SpError> {
        todo!()
    }
    fn read_ring_indicator(&mut self) -> Result<bool, SpError> {
        todo!()
    }
    fn read_carrier_detect(&mut self) -> Result<bool, SpError> {
        todo!()
    }
    fn bytes_to_read(&self) -> Result<u32, SpError> {
        todo!()
    }
    fn bytes_to_write(&self) -> Result<u32, SpError> {
        todo!()
    }
    fn clear(&self, _: ClearBuffer) -> Result<(), SpError> {
        todo!()
    }
    fn try_clone(&self) -> Result<Box<(dyn SerialPort + 'static)>, SpError> {
        Ok(Box::new(self.clone()))
    }
    fn set_break(&self) -> Result<(), SpError> {
        todo!()
    }
    fn clear_break(&self) -> Result<(), SpError> {
        todo!()
    }
}
