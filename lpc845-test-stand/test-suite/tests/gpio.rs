//! Test Suite for the GPIO API in LPC8xx HAL
//!
//! This test suite communicates with hardware. See top-level README.md for
//! wiring instructions.

use std::thread::sleep;
use std::time::{self, Duration, Instant};

use host_lib::assistant::{Assistant, AssistantError, InputPin, LEGAL_DYNAMIC_PINS};
use lpc845_messages::{pin::Level, PinNumber};
use lpc845_test_suite::{Result, TestStand};

const RED_LED_PIN: PinNumber = 29;
const GRN_LED_PIN: PinNumber = 31;

const PIN_WAIT_TIME: u64 = 300;

// requires a 75% 1s PWM signal connected to RED_LED_PIN so disabled (ignored) by default
#[test]
#[ignore] 
fn blinky() -> Result {
    const HIGH_PULSE_DURATION: Duration = Duration::from_millis(750);
    const LOW_PULSE_DURATION: Duration = Duration::from_millis(250);
    const POLL_DELAY: Duration = Duration::from_micros(500);
    const PULSE_TEST_ITERATIONS: usize = 3;
    const PULSE_DURATION_TOLERANCE: Duration = Duration::from_millis(50);
    fn synchronize_pulse_measurement(input_pin: &mut InputPin<Assistant>) -> Result {
        let pwm_period = LOW_PULSE_DURATION + HIGH_PULSE_DURATION;
        let pin_synchronization_timeout = 2 * pwm_period;

        wait_for_pin_with_time(Level::High, input_pin, pin_synchronization_timeout)?;

        wait_for_pin_with_time(Level::Low, input_pin, pin_synchronization_timeout)?;

        Ok(())
    }

    fn measure_pulse_durations(
        input_pin: &mut InputPin<host_lib::assistant::Assistant>,
    ) -> core::result::Result<(Duration, Duration), AssistantError> {
        let pwm_period = LOW_PULSE_DURATION + HIGH_PULSE_DURATION;
        // wait for pin to go high
        let t_low = wait_for_pin_with_time(Level::High, input_pin, pwm_period)?;

        // wait for pin to go low
        let t_high = wait_for_pin_with_time(Level::Low, input_pin, pwm_period)?;

        Ok((t_low, t_high))
    }

    fn wait_for_pin_with_time(
        desired_state: Level,
        input_pin: &mut InputPin<Assistant>,
        max_time: Duration,
    ) -> core::result::Result<Duration, AssistantError> {
        let start = Instant::now();

        // Iterate until we hit max time
        while start.elapsed() < max_time {
            // Has the pin reached the desired state?
            let current_state = if input_pin.is_high()? {
                Level::High
            } else {
                Level::Low
            };

            if desired_state == current_state {
                // If so, return the ammount of time since we started measuring
                return Ok(start.elapsed());
            }

            std::thread::sleep(POLL_DELAY);
        }

        panic!("TIMEOUT");
    }

    fn is_duration_within_tolerance(
        actual: Duration,
        expected: Duration,
        tolerance: Duration,
    ) -> bool {
        (actual < (expected + tolerance)) && (actual > (expected - tolerance))
    }

    let test_stand = TestStand::new()?;
    let mut input_pin = test_stand.assistant.create_gpio_input_pin(RED_LED_PIN)?;

    synchronize_pulse_measurement(&mut input_pin)?;

    for i in 0..PULSE_TEST_ITERATIONS {
        let (t_low, t_high) = measure_pulse_durations(&mut input_pin)?;
        println!("{}: {:?}, {:?}", i, t_low, t_high);

        assert!(is_duration_within_tolerance(
            t_low,
            LOW_PULSE_DURATION,
            PULSE_DURATION_TOLERANCE
        ));

        assert!(is_duration_within_tolerance(
            t_high,
            HIGH_PULSE_DURATION,
            PULSE_DURATION_TOLERANCE
        ));
    }

    Ok(())
}

#[test]
fn assistant_should_change_and_read_noint_dyn_pin() -> Result {
    // SETUP
    let mut test_stand = TestStand::new()?;
    let mut target = test_stand
        .target
        .take()
        .expect("target likely not configured in test-stand.toml");
    let assistant = test_stand.assistant;

    // NOTE: The assistant's green led pin is a noint dyn pin, i.e. a dynamic pin that's read
    // by polling rather than by interrupt
    let pin_number = GRN_LED_PIN;
    // check if direction setting works in both directions
    let out_pin = assistant.create_gpio_output_pin(pin_number, Level::Low)?;
    let mut in_pin = out_pin.into_input_pin()?;

    // TEST & ASSERT POSTCONDITION
    target.set_pin_low()?;
    // ensure we don't read before the next timer tick
    sleep(time::Duration::from_millis(PIN_WAIT_TIME));
    assert!(in_pin.is_low()?);

    // TEST & ASSERT POSTCONDITION
    target.set_pin_high()?;
    // ensure we don't read before the next timer tick
    sleep(time::Duration::from_millis(PIN_WAIT_TIME));
    assert!(in_pin.is_high()?);

    Ok(())
}

#[test]
fn target_should_set_pin_level() -> Result {
    // SETUP
    let mut test_stand = TestStand::new()?;
    let mut target = test_stand
        .target
        .take()
        .expect("target likely not configured in test-stand.toml");
    let assistant = test_stand.assistant;
    let mut in_pin = assistant.create_gpio_input_pin(GRN_LED_PIN)?;

    // TEST & ASSERT POSTCONDITION
    target.set_pin_low()?;
    // green is a dynamic noint pin; ensure we don't read before the next timer tick
    sleep(time::Duration::from_millis(PIN_WAIT_TIME));
    assert!(in_pin.is_low()?);

    // TEST & ASSERT POSTCONDITION
    target.set_pin_high()?;
    // green is a dynamic noint pin; ensure we don't read before the next timer tick
    sleep(time::Duration::from_millis(PIN_WAIT_TIME));
    assert!(in_pin.is_high()?);

    Ok(())
}

#[test]
fn target_should_read_input_level() -> Result {
    // SETUP
    let mut test_stand = TestStand::new()?;
    let mut target = test_stand
        .target
        .take()
        .expect("target likely not configured in test-stand.toml");
    let mut out_pin = test_stand
        .assistant
        .create_gpio_output_pin(RED_LED_PIN, Level::Low)?;

    // RUN TEST
    out_pin.set_low()?;
    assert!(target.pin_is_low()?);

    out_pin.set_high()?;
    assert!(target.pin_is_high()?);

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
    let mut target = test_stand
        .target
        .take()
        .expect("target likely not configured in test-stand.toml");
    let mut in_pin = test_stand.assistant.create_gpio_input_pin(GRN_LED_PIN)?;

    // RUN TEST & ASSERT POSTCONDITION
    target.set_pin_low()?;
    // green is a dynamic noint pin; ensure we don't read before the next timer tick
    sleep(time::Duration::from_millis(PIN_WAIT_TIME));
    assert!(in_pin.is_low()?);
    assert!(in_pin.is_low()?);

    // TEST & ASSERT POSTCONDITION
    target.set_pin_high()?;
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
    let mut target = test_stand
        .target
        .take()
        .expect("target likely not configured in test-stand.toml");
    // ensure pin is low (-> red led is on) when we start
    let mut out_pin = test_stand
        .assistant
        .create_gpio_output_pin(RED_LED_PIN, Level::Low)?;
    assert!(target.pin_is_low()?);

    let in_pin = out_pin.into_input_pin()?;
    sleep(time::Duration::from_secs(2));

    out_pin = in_pin.into_output_pin(Level::High)?;
    assert!(target.pin_is_high()?);

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
