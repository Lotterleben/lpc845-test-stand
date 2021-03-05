use core::panic;

use crate::{
    handle_pin_interrupt, handle_pin_interrupt_dynamic, handle_pin_interrupt_noint_dynamic,
    CTS_PIN_NUMBER, PININT0_DYN_PIN, RED_LED_PIN_NUMBER, RTS_PIN_NUMBER,
    TARGET_TIMER_PIN_NUMBER, FIXED_DIRECTION_PINS,
};
use firmware_lib::usart::Tx;
use heapless::{consts::U4, consts::U8, FnvIndexMap};
use lpc845_messages::{
    pin, AssistantToHost, DynamicPin, HostToAssistant, InputPin, OutputPin, UsartMode,
    PinNumber,
};

#[cfg(feature = "sleep")]
use lpc8xx_hal::cortex_m::asm;

use lpc8xx_hal::{
    pac::USART0,
    usart::state::AsyncMode,
    cortex_m::interrupt,
    gpio,
    prelude::*,
};

use rtic::Mutex;
use rtt_target::rprintln;

pub fn handle_idle(cx: crate::idle::Context) -> ! {
    let host_rx = cx.resources.host_rx_idle;
    let host_tx = cx.resources.host_tx;
    let target_rx = cx.resources.target_rx_idle;
    let target_tx = cx.resources.target_tx;
    let target_tx_dma = cx.resources.target_tx_dma;
    let target_sync_rx = cx.resources.target_sync_rx_idle;
    let target_sync_tx = cx.resources.target_sync_tx;
    let blue_idle = cx.resources.blue_idle;
    let pinint0_idle = cx.resources.pinint0_idle;
    let target_rts_idle = cx.resources.target_rts_idle;
    let pinint0_pin = cx.resources.pinint0_pin;
    let mut dyn_noint_pins = cx.resources.dyn_noint_pins;
    let dyn_noint_levels_out = cx.resources.dyn_noint_levels_out;

    let pwm = cx.resources.pwm_idle;
    let pin_5 = cx.resources.pin_5;

    // TODO(LSS): We need to add handling of fixed direction pin messages!
    let _cts = cx.resources.cts;
    let _rts = cx.resources.rts;

    let mut fixed_pin_levels = FnvIndexMap::<_, _, U8>::new();
    let mut dynamic_int_pin_levels = FnvIndexMap::<_, _, U4>::new();
    let mut dynamic_noint_pin_levels = FnvIndexMap::<_, gpio::Level, U4>::new();

    let mut buf = [0; 256];

    loop {
        target_rx
            .process_raw(|data| {
                host_tx.send_message(
                    &AssistantToHost::UsartReceive {
                        mode: UsartMode::Regular,
                        data,
                    },
                    &mut buf,
                )
            })
            .expect("Error processing USART data");
        target_sync_rx
            .process_raw(|data| {
                host_tx.send_message(
                    &AssistantToHost::UsartReceive {
                        mode: UsartMode::Sync,
                        data,
                    },
                    &mut buf,
                )
            })
            .expect("Error processing USART data");

        host_rx
            .process_message(|message| {
                match message {
                    HostToAssistant::SendUsart {
                        mode: UsartMode::Regular,
                        data,
                    } => target_tx.send_raw(data),
                    HostToAssistant::SendUsart {
                        mode: UsartMode::Dma,
                        data,
                    } => {
                        rprintln!("Sending USART message using DMA.");
                        target_tx_dma.bwrite_all(data)
                    }
                    HostToAssistant::SendUsart {
                        mode: UsartMode::FlowControl,
                        data: _,
                    } => Ok(()),
                    HostToAssistant::SendUsart {
                        mode: UsartMode::Sync,
                        data,
                    } => target_sync_tx.send_raw(data),
                    HostToAssistant::SetPin(pin::SetLevel {
                        pin: OutputPin::Pin5,
                        level,
                    }) => {
                        match level {
                            pin::Level::High => {
                                pin_5.set_high();
                            }
                            pin::Level::Low => {
                                pin_5.set_low();
                            }
                        }
                        Ok(())
                    }
                    HostToAssistant::SetPin(pin::SetLevel { pin: _, level: _ }) => {
                        // currently we don't have defined any other non-dynamic Output Pins that could be set
                        // TODO refactor back: this should be usable
                        unreachable!()
                    }
                    HostToAssistant::SetDynamicPin(pin::SetLevel { pin, level }) => {
                        let pin_number = pin.get_pin_number().unwrap();

                        // TODO less repetitive once we've unified pin types
                        // TODO maybe just add a set_level() function to the hal?
                        match pin_number {
                            RED_LED_PIN_NUMBER => {
                                if pinint0_pin.direction_is_output() {
                                    match level {
                                        pin::Level::High => pinint0_pin.set_high(),
                                        pin::Level::Low => pinint0_pin.set_low(),
                                    }
                                }
                            }
                            CTS_PIN_NUMBER => {
                                todo!("AJM: Refactor")
                                // if cts.direction_is_output() {
                                //     match level {
                                //         pin::Level::High => cts.set_high(),
                                //         pin::Level::Low => cts.set_low(),
                                //     }
                                // }
                            }
                            RTS_PIN_NUMBER => {
                                unreachable!()
                            }
                            pin_number => {
                                dyn_noint_pins.lock(|pin_map| {
                                    // TODO turn this into a macro?
                                    if pin_map.contains_key(&pin_number) {
                                        let pin = pin_map.get_mut(&pin_number).unwrap();
                                        if pin.direction_is_output() {
                                            match level {
                                                pin::Level::High => pin.set_high(),
                                                pin::Level::Low => pin.set_low(),
                                            }
                                        }
                                    // TODO add pin level to buffer right away
                                    } else {
                                        rprintln!("can't query unsupported pin: {:?}", pin_number);
                                    }
                                })
                            }
                        };

                        Ok(())
                    }
                    HostToAssistant::ReadPin(pin::ReadLevel { pin }) => {
                        let result =
                            fixed_pin_levels
                                .get(&(pin as usize))
                                .map(|&(level, period_ms)| pin::ReadLevelResult {
                                    pin,
                                    level,
                                    period_ms,
                                });

                        host_tx
                            .send_message(&AssistantToHost::ReadPinResult(result), &mut buf)
                            .unwrap();

                        Ok(())
                    }
                    HostToAssistant::SetDirection(pin::SetDirection {
                        pin,
                        direction: pin::Direction::Input,
                        level: None,
                    }) => {
                        //rprintln!("{:?} is Input", pin);

                        // TODO(LSS) -> AJM! I'd like to learn how to untagle and merge this with
                        // the match arm below this. problem is the fixed pins

                        match pin.get_pin_number().unwrap() {
                            RED_LED_PIN_NUMBER => {
                                pinint0_pin.switch_to_input();
                                // inintialize interruptable pins so that a status read is possible before the first level
                                // change (TODO is this a separate PR candidate?)
                                let pinint0_level = match pinint0_pin.is_high() {
                                    true => pin::Level::High,
                                    false => pin::Level::Low,
                                };
                                dynamic_int_pin_levels
                                    .insert(RED_LED_PIN_NUMBER as usize, (pinint0_level, None))
                                    .unwrap();
                            }
                            CTS_PIN_NUMBER => {
                                // TODO proper error handling
                                rprintln!("CTS pin is never Input");
                                unreachable!()
                            }
                            RTS_PIN_NUMBER => {
                                todo!("AJM: Excise evil")
                                // rts.switch_to_input()
                            }
                            TARGET_TIMER_PIN_NUMBER => {
                                // Ignore for now, we've hardcoded this pin as input
                                // TODO fix this
                            }
                            pin_number => {
                                dyn_noint_pins.lock(|pin_map| {
                                    if pin_map.contains_key(&pin_number) {
                                        // this is a dynamic non-interrupt pin, set its direction
                                        let pin = pin_map.get_mut(&pin_number).unwrap();
                                        pin.switch_to_input();
                                    } else {
                                        rprintln!("unsupported pin: {:?}", pin_number)
                                    }
                                });
                            }
                        };
                        Ok(())
                    }
                    // TODO merge this with SetDirection for Input; control flow is duplicate
                    HostToAssistant::SetDirection(pin::SetDirection {
                        pin,
                        direction,
                        level: Some(level),
                    }) => {
                        let cd = match direction {
                            pin::Direction::Input => ContextualDirection::Input,
                            pin::Direction::Output => ContextualDirection::Output(level.into())
                        };

                        handle_set_direction_dynamic(
                            pin,
                            cd,
                            pinint0_pin,
                            // cts, // TODO(AJM): Removing evil
                            &mut dynamic_int_pin_levels,
                            &mut dyn_noint_pins
                        ).unwrap();

                        // TODO(LSS) handle err
                        Ok(())
                    }
                    HostToAssistant::SetDirection(pin::SetDirection {
                        pin: _,
                        direction: _,
                        level: _,
                    }) => {
                        // illegal level/direction combination
                        // TODO handle error more neatly
                        unreachable!()
                    }
                    HostToAssistant::ReadDynamicPin(pin::ReadLevel { pin }) => {
                        // AJM!
                        match handle_read_dynamic_pin(
                            pin,
                            &mut dyn_noint_pins,
                            &mut dynamic_noint_pin_levels,
                            host_tx,
                            &mut buf,
                            &mut dynamic_int_pin_levels,
                        ) {
                            Ok(_) => Ok(()),
                            Err(err) => {
                                // This will currently cause a panic anyway due to the expect()
                                // at the end of this match- print a hint to indicate what the cause was
                                rprintln!("Reading dynamic pin level failed: {:?}", err);
                                // TODO(LSS) why do I get
                                // expected value, found enum `Void`
                                // or
                                // expected enum `Void`, found enum `PinReadError`
                                // here? (I think I understand where the missing
                                // Void vaue is coming from but why is rustc expecting void in the
                                // first place?)
                                //Err(err)
                                panic!() // TODO(LSS) fix this, s.a.)
                            }
                        }
                    }
                }
            })
            .expect("Error processing host request");

        host_rx.clear_buf();

        // TODO: is pwm pin ever handled in reading messages?
        handle_pin_interrupt(pwm, InputPin::Pwm, &mut fixed_pin_levels);
        handle_pin_interrupt(blue_idle, InputPin::Blue, &mut fixed_pin_levels);

        handle_pin_interrupt_dynamic(pinint0_idle, PININT0_DYN_PIN, &mut dynamic_int_pin_levels);
        handle_pin_interrupt_dynamic(
            target_rts_idle,
            DynamicPin::GPIO(RTS_PIN_NUMBER),
            &mut dynamic_int_pin_levels,
        );
        handle_pin_interrupt_noint_dynamic(dyn_noint_levels_out, &mut dynamic_noint_pin_levels);

        // We need this critical section to protect against a race
        // conditions with the interrupt handlers. Otherwise, the following
        // sequence of events could occur:
        // 1. We check the queues here, they're empty.
        // 2. New data is received, an interrupt handler adds it to a queue.
        // 3. The interrupt handler is done, we're back here and going to
        //    sleep.
        //
        // This might not be observable, if something else happens to wake
        // us up before the test suite times out. But it could also lead to
        // spurious test failures.
        interrupt::free(|_| {
            let should_sleep =
                !host_rx.can_process() && !target_rx.can_process() && pinint0_idle.is_ready();

            if should_sleep {
                // On LPC84x MCUs, debug mode is not supported when
                // sleeping. This interferes with RTT communication. Only
                // sleep, if the user enables this through a compile-time
                // flag.
                #[cfg(feature = "sleep")]
                asm::wfi();
            }
        });
    }
}


// TODO(LSS) clean this up
use lpc8xx_hal::gpio::direction::Dynamic;
use lpc8xx_hal::gpio::GpioPin;
use crate::PININT0_PIN;

// TODO NEEDS BETTER NAME
#[derive(Debug)]
pub enum ContextualDirection {
    Input,
    Output(gpio::Level),
}

/// Change the direction of `pin` to `direction`.
/// - Level only matters for change to output
///    * group in enum { ToOutPut(level), ToInput } -> bubble up into actual messages across the wire
/// - you should only be calling this with dynamic pins
/// - pin has to be a valid pin number
///    * do the checks above this function?
/// - dynamic_int_pin_levels should not be full
/// - we don't try to change our accidentally dynamic pins (RTS/CTS)
fn handle_set_direction_dynamic(
    pin: DynamicPin,
    direction: ContextualDirection,
    pinint0_pin: &mut GpioPin<PININT0_PIN, Dynamic>,
    dynamic_int_pin_levels: &mut FnvIndexMap<usize, (pin::Level, Option<u32>), U4>,
    dyn_noint_pins: &mut crate::resources::dyn_noint_pins,
) -> Result<(), DynamicPinError> {
    // Check if this pin has a mapped pin number (e.g. is it
    // allocated as a GPIO)
    let pin_number = if let Some(pn) = pin.get_pin_number() {
        pn
    } else {
        return Err(DynamicPinError::NotGpioPin);
    };

    match pin_number {
        RED_LED_PIN_NUMBER => {
            // Why is this special cased?
            //
            // Because it is an "Interrupt" pin, and all other pins
            // are "Polled" pins

            let level = match direction {
                ContextualDirection::Output(gpio_level) => {
                    // For output, trust the commanded output
                    pinint0_pin.switch_to_output(gpio_level);
                    gpio_level
                }
                ContextualDirection::Input => {
                    // For input, read the input after switching to the
                    // input mode
                    pinint0_pin.switch_to_input();
                    pinint0_pin.get_level()
                }
            };

            // inintialize interruptable pins so that a status read is
            // possible before the first level change
            //
            // TODO(LSS): Doing this here is either a bug or is smelly
            //
            // TODO(AJM): We should think about "rolling back" unsuccessful changes,
            // e.g. here, we did change the mode, but didn't properly start tracking
            // the interrupt pin. Is this behavior problematic or could it cause
            // runtime problems? How could a user revert/or recover from this? Should it
            // just be a test-ending "abort" condition?
            dynamic_int_pin_levels
                .insert(RED_LED_PIN_NUMBER as usize, (level.into(), None))
                .map_err(|_e| DynamicPinError::InterruptPinStorageFull)?;

        }
        pn if !FIXED_DIRECTION_PINS.contains(&pn) => {
            // GOOD, not in the fixed pin set

            dyn_noint_pins.lock(|pin_map| {
                // Ensure we have a NoIntPin
                let pin = pin_map
                    .get_mut(&pn)
                    .ok_or_else(|| DynamicPinError::NotDynamicNoIntPin(pn))?;

                match direction {
                    ContextualDirection::Output(gpio_level) => {
                        pin.switch_to_output(gpio_level);
                    }
                    ContextualDirection::Input => {
                        pin.switch_to_input();
                    }
                }
                Ok(())
            })?;
        }
        pn => {
            return Err(DynamicPinError::NotDynamicPin(pn));
        }
    };

    Ok(())
}

fn handle_read_dynamic_pin(
    pin: DynamicPin,
    dyn_noint_pins: &mut crate::resources::dyn_noint_pins,
    dynamic_noint_pin_levels: &mut FnvIndexMap<usize, gpio::Level, U4>,
    host_tx: &mut Tx<USART0, AsyncMode>,
    buf: &mut [u8],
    dynamic_int_pin_levels: &mut FnvIndexMap<usize, (pin::Level, Option<u32>), U4>,
) -> Result<(), DynamicPinError> {
    let pin_number = pin.get_pin_number().unwrap();

    if FIXED_DIRECTION_PINS.contains(&pin_number) {
        return Err(DynamicPinError::NotDynamicPin(pin_number));
    }

    // TODO(LSS) if thsi keeps reappearing make an enum instead of bool
    let is_dyn_noint_pin = dyn_noint_pins.lock(|pin_map| pin_map.contains_key(&pin_number));

    let result = if is_dyn_noint_pin {
        dynamic_noint_pin_levels
            .get(&(pin_number as usize))
            .map(|gpio_level| (pin::Level::from(*gpio_level), None))
    } else {
        dynamic_int_pin_levels
            .get(&(pin_number as usize))
            .map(|maybe_tuple| *maybe_tuple)
    };

    let read_level_result = result.map(|(level, period_ms)| pin::ReadLevelResult {
        pin,
        level,
        period_ms,
    });

    // TODO(LSS) add enum, bubble error up
    host_tx
        .send_message(
            &AssistantToHost::ReadPinResultDynamic(read_level_result),
            buf,
        )
        .unwrap();

    Ok(())
}

#[derive(Debug)]
enum DynamicPinError {
    /// Tried to change a GPIO that is NOT dynamic
    ///
    /// Contains the pin number that was attempted to be changed.
    NotDynamicPin(PinNumber),

    /// Tried to use NoInt methods on a pin that isn't on the NoInt list
    ///
    /// This is likely indicative of a coding error at the Assistant firmware
    /// level, and can likely NOT be recovered from at Runtime
    NotDynamicNoIntPin(PinNumber),

    /// Tried to configure a pin that is NOT marked as a GPIO
    ///
    /// (e.g. a fixed UART/SPI/etc.). No pin number is returned
    /// because non-GPIO pins don't have numbers.
    NotGpioPin,

    /// Tried to track an additional interrupt pin, but did not fit into storage
    ///
    /// This is likely indicative of a coding/resource allocation error at the
    /// Assistant firmware level, and can likely NOT be recovered from at Runtime
    InterruptPinStorageFull,
}

trait ExtGetLevel {
    fn get_level(&self) -> lpc8xx_hal::gpio::Level;
}

impl<GenericPin> ExtGetLevel for GpioPin<GenericPin, Dynamic>
where
    GenericPin: lpc8xx_hal::pins::Trait,
{
    fn get_level(&self) -> lpc8xx_hal::gpio::Level {
        if self.is_high() {
            lpc8xx_hal::gpio::Level::High
        } else {
            lpc8xx_hal::gpio::Level::Low
        }
    }
}
