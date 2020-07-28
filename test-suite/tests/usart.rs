//! Test Suite for the USART API in LPC8xx HAL
//!
//! This test suite communicates with hardware. See top-level README.md for
//! wiring instructions.


use std::time::Duration;

use lpc845_test_suite::{
    Result,
    TestStand,
};


#[test]
fn it_should_send_messages() -> Result {
    let mut test_stand = TestStand::new()?;

    let message = b"Hello, world!";
    test_stand.target.send_usart(message)?;

    let timeout  = Duration::from_millis(50);
    let received = test_stand.assistant
        .receive_from_target_usart(message, timeout)?;

    assert_eq!(received, message);
    Ok(())
}

#[test]
fn it_should_receive_messages() -> Result {
    let mut test_stand = TestStand::new()?;

    let message = b"Hello, world!";
    test_stand.assistant.send_to_target_usart(message)?;

    let timeout  = Duration::from_millis(50);
    let received = test_stand.target.wait_for_usart_rx(message, timeout)?;

    assert_eq!(received, message);
    Ok(())
}

#[test]
fn it_should_send_messages_using_dma() -> Result {
    let mut test_stand = TestStand::new()?;

    let message = b"Hello, world!";
    test_stand.target.send_usart_dma(message)?;

    let timeout  = Duration::from_millis(50);
    let received = test_stand.assistant
        .receive_from_target_usart(message, timeout)?;

    assert_eq!(received, message);
    Ok(())
}

#[test]
fn it_should_receive_messages_via_dma() -> Result {
    let mut test_stand = TestStand::new()?;

    let message = b"Hello, world!";
    test_stand.assistant.send_to_target_usart_dma(message)?;

    let timeout  = Duration::from_millis(50);
    let received = test_stand.target.wait_for_usart_rx_dma(message, timeout)?;

    assert_eq!(received, message);
    Ok(())
}
