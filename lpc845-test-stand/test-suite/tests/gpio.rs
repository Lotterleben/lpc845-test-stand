//! Test Suite for the GPIO API in LPC8xx HAL
//!
//! This test suite communicates with hardware. See top-level README.md for
//! wiring instructions.

use std::thread::sleep;
use std::time;

use lpc845_messages::{pin::Level, PinNumber};
use host_lib::assistant::{Assistant, InputPin, LEGAL_DYNAMIC_PINS};
use lpc845_test_suite::{Result, TestStand};

const RED_LED_PIN: PinNumber = 29;
const GRN_LED_PIN: PinNumber = 31;

const PIN_WAIT_TIME: u64 = 300;

#[test]
fn assistant_should_change_and_read_noint_dyn_pin() -> Result {
    // SETUP
    let mut test_stand = TestStand::new()?;
    // NOTE: The assistant's green led pin is a noint dyn pin, i.e. a dynamic pin that's read
    // by polling rather than by interrupt
    let pin_number = GRN_LED_PIN;
    // check if direction setting works in both directions
    let out_pin = test_stand
        .assistant
        .create_gpio_output_pin(pin_number, Level::Low)?;
    let mut in_pin = out_pin.into_input_pin()?;

    // TEST & ASSERT POSTCONDITION
    test_stand.target.set_pin_low()?;
    // ensure we don't read before the next timer tick
    sleep(time::Duration::from_millis(PIN_WAIT_TIME));
    assert!(in_pin.is_low()?);

    // TEST & ASSERT POSTCONDITION
    test_stand.target.set_pin_high()?;
    // ensure we don't read before the next timer tick
    sleep(time::Duration::from_millis(PIN_WAIT_TIME));
    assert!(in_pin.is_high()?);

    Ok(())
}

#[test]
fn target_should_set_pin_level() -> Result {
    // SETUP
    let mut test_stand = TestStand::new()?;
    let mut in_pin = test_stand.assistant.create_gpio_input_pin(GRN_LED_PIN)?;

    // TEST & ASSERT POSTCONDITION
    test_stand.target.set_pin_low()?;
    // green is a dynamic noint pin; ensure we don't read before the next timer tick
    sleep(time::Duration::from_millis(PIN_WAIT_TIME));
    assert!(in_pin.is_low()?);

    // TEST & ASSERT POSTCONDITION
    test_stand.target.set_pin_high()?;
    // green is a dynamic noint pin; ensure we don't read before the next timer tick
    sleep(time::Duration::from_millis(PIN_WAIT_TIME));
    assert!(in_pin.is_high()?);

    Ok(())
}

#[test]
fn target_should_read_input_level() -> Result {
    // SETUP
    let mut test_stand = TestStand::new()?;
    let mut out_pin = test_stand
        .assistant
        .create_gpio_output_pin(RED_LED_PIN, Level::Low)?;

    // RUN TEST
    out_pin.set_low()?;
    assert!(test_stand.target.pin_is_low()?);

    out_pin.set_high()?;
    assert!(test_stand.target.pin_is_high()?);

    Ok(())
}
/// This is a regression test:
/// Previously, interruptable pins could only be read successfully after the first
/// interrupt was triggered by a level change. This test ensures that this bug stays fixed.
#[test]
fn assistant_should_read_level_without_level_change_interruptable_out_pin() -> Result {
    let interruptable_pin = RED_LED_PIN;

    // check init low
    let test_stand = TestStand::new()?;
    let mut out_pin = test_stand
        .assistant
        .create_gpio_output_pin(interruptable_pin, Level::High)?;

    assert!(out_pin.is_set_high()?);

    Ok(())
}

#[test]
fn assistant_should_read_level_repeatedly_interruptable_pin() -> Result {
    let interruptable_pin = RED_LED_PIN;

    // SETUP
    let test_stand = TestStand::new()?;
    let mut out_pin = test_stand
        .assistant
        .create_gpio_output_pin(interruptable_pin, Level::High)?;

    // RUN TEST
    assert!(out_pin.is_set_high()?);
    assert!(out_pin.is_set_high()?);
    assert!(out_pin.is_set_high()?);

    out_pin.set_low()?;
    assert!(out_pin.is_set_low()?);
    assert!(out_pin.is_set_low()?);
    assert!(out_pin.is_set_low()?);

    Ok(())
}

#[test]
fn assistant_should_read_level_repeatedly_polled_pin() -> Result {

    // SETUP
    let mut test_stand = TestStand::new()?;
    let mut in_pin = test_stand
        .assistant
        .create_gpio_input_pin(GRN_LED_PIN)?;

    // RUN TEST & ASSERT POSTCONDITION
    test_stand.target.set_pin_low()?;
    // green is a dynamic noint pin; ensure we don't read before the next timer tick
    sleep(time::Duration::from_millis(PIN_WAIT_TIME));
    assert!(in_pin.is_low()?);
    assert!(in_pin.is_low()?);

    // TEST & ASSERT POSTCONDITION
    test_stand.target.set_pin_high()?;
    // green is a dynamic noint pin; ensure we don't read before the next timer tick
    sleep(time::Duration::from_millis(PIN_WAIT_TIME));
    assert!(in_pin.is_high()?);
    assert!(in_pin.is_high()?);

    Ok(())
}


/// Ensure that pins which trigger an interrupt on change return the correct level
/// immediately, no wait period required.
#[test]
fn assistant_should_return_interruptable_pin_status_immediately() -> Result {
    /*
     Note: The only interrupt-triggered dynamic puin at the moment is pin 29 (red led).
     The current test-target API only allows using Pin 29 (red led) as target out,
     assistant in. We can still check if reading works immediately, though, because
     output pin levels can be read too.
    */
    let interruptable_pin = RED_LED_PIN;

    // SETUP
    let test_stand = TestStand::new()?;
    let mut out_pin = test_stand
        .assistant
        .create_gpio_output_pin(interruptable_pin, Level::High)?;

    // RUN TEST
    out_pin.set_low()?;
    assert!(out_pin.is_set_low()?);

    out_pin.set_high()?;
    assert!(out_pin.is_set_high()?);

    Ok(())
}


#[test]
fn assistant_all_dyn_gpio_pins_should_work() -> Result {

    let test_stand = TestStand::new()?;

    for pin in &LEGAL_DYNAMIC_PINS {
        // SETUP
        let mut out_pin = test_stand
            .assistant
            .create_gpio_output_pin(*pin, Level::Low)?;

        // RUN TEST & ASSERT POSTCONDITION
        out_pin.set_high()?;
        sleep(time::Duration::from_millis(PIN_WAIT_TIME));
        assert!(out_pin.is_set_high()?);

        out_pin.set_low()?;
        sleep(time::Duration::from_millis(PIN_WAIT_TIME));
        assert!(out_pin.is_set_low()?);
    }

    Ok(())
}

#[test]
#[allow(unused_assignments)] // to silence the last conversion
fn assistant_red_led_should_be_toggleable_by_pin_direction() -> Result {
    // SETUP
    let test_stand = TestStand::new()?;
    // ensure pin is low (-> red led is on) when we start
    let mut out_pin = test_stand
        .assistant
        .create_gpio_output_pin(RED_LED_PIN, Level::Low)?;
    let mut in_pin: InputPin<Assistant>; // we'll need this during the loop

    // RUN TEST
    for _ in 0..5 {
        in_pin = out_pin.into_input_pin()?;
        sleep(time::Duration::from_secs(2));
        out_pin = in_pin.into_output_pin(Level::Low)?;
        sleep(time::Duration::from_secs(2));
    }

    // ASSERT POSTCONDITION
    // 👀  manually assert that led is toggling on/off every 2 secs

    Ok(())
}

#[test]
#[allow(unused_assignments)] // to silence the last conversion
fn wonky_in_out_conversion_should_work() -> Result {
    // SETUP
    let mut test_stand = TestStand::new()?;
    // ensure pin is low (-> red led is on) when we start
    let mut out_pin = test_stand
        .assistant
        .create_gpio_output_pin(RED_LED_PIN, Level::Low)?;
    assert!(test_stand.target.pin_is_low()?);

    let in_pin = out_pin.into_input_pin()?;
    sleep(time::Duration::from_secs(2));

    out_pin = in_pin.into_output_pin(Level::High)?;
    assert!(test_stand.target.pin_is_high()?);

    Ok(())
}

#[test]
fn assistant_red_led_should_be_toggleable_by_level() -> Result {
    // SETUP
    let test_stand = TestStand::new()?;
    let mut out_pin = test_stand
        .assistant
        .create_gpio_output_pin(RED_LED_PIN, Level::Low)?;
    out_pin.set_high()?;

    // RUN TEST
    sleep(time::Duration::from_secs(2));
    out_pin.set_low()?;
    sleep(time::Duration::from_secs(2));
    out_pin.set_high()?;

    sleep(time::Duration::from_secs(2));
    out_pin.set_low()?;
    sleep(time::Duration::from_secs(2));
    out_pin.set_high()?;

    sleep(time::Duration::from_secs(2));
    out_pin.set_low()?;
    sleep(time::Duration::from_secs(2));
    out_pin.set_high()?;

    // ASSERT POSTCONDITION
    // 👀  manually assert that led is toggling on/off every 2 secs

    Ok(())
}
