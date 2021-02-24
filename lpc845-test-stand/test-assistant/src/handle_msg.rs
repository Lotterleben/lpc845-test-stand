#![allow(unused_imports)]

use core::marker::PhantomData;

use heapless::{consts::U4, consts::U8, spsc::Consumer, spsc::Producer, FnvIndexMap};
use lpc8xx_hal::{
    prelude::*,
    cortex_m::interrupt,
    gpio::{self, direction::Dynamic, direction::Output, GpioPin},
    i2c,
    init_state::Enabled,
    mrt::{
        MRT0,
        MRT1,
        MRT2,
        MRT3,
    },
    nb::{
        self,
        block,
    },
    pac::{
        I2C0,
        SPI0,
        USART0,
        USART1,
        USART2,
        USART3,
    },
    pinint::{
        PININT0,
        PININT1,
        PININT2,
        PININT3,
    },
    pins::{
        DynamicPinDirection,
        GenericPin,
        PIO0_8,
        PIO0_9,
        PIO0_20,
        PIO0_23,
        PIO1_1,
    },
    spi::{
        self,
        SPI,
    },
    syscon::{
        IOSC,
        frg,
    },
    usart::{
        self,
        state::{AsyncMode, SyncMode},
    },
    Peripherals,
};
use rtt_target::rprintln;
#[cfg(feature = "sleep")]
use lpc8xx_hal::cortex_m::asm;
use lpc845_messages::{AssistantToHost, HostToAssistant, InputPin, OutputPin, DynamicPin, UsartMode, pin};
use firmware_lib::{
    pin_interrupt::{self, PinInterrupt},
    timer_interrupt::{PinMeasurementEvent, TimerInterrupt},
    usart::{RxIdle, RxInt, Tx, Usart},
};
use rtic::Mutex;
use crate::{
    handle_pin_interrupt_noint_dynamic,
    RTS_PIN_NUMBER,
    handle_pin_interrupt_dynamic,
    PININT0_DYN_PIN,
    handle_pin_interrupt,
};

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
    let pinint0_pin  = cx.resources.pinint0_pin;
    let mut dyn_noint_pins = cx.resources.dyn_noint_pins;
    let dyn_noint_levels_out = cx.resources.dyn_noint_levels_out;
    let cts          = cx.resources.cts;
    let rts          = cx.resources.rts;
    let pwm          = cx.resources.pwm_idle;
    let pin_5        = cx.resources.pin_5;


    let mut pins = FnvIndexMap::<_, _, U8>::new();
    let mut dynamic_int_pins = FnvIndexMap::<_, _, U4>::new();
    let mut dynamic_noint_pins = FnvIndexMap::<_, gpio::Level, U4>::new();

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
                    } => {
                        target_tx.send_raw(data)
                    }
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
                    } => {
                        Ok(())
                    }
                    HostToAssistant::SendUsart {
                        mode: UsartMode::Sync,
                        data,
                    } => {
                        target_sync_tx.send_raw(data)
                    }
                    HostToAssistant::SetPin(
                        pin::SetLevel {
                            pin: OutputPin::Pin5,
                            level,
                        }
                    ) => {
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
                    HostToAssistant::SetPin(
                        pin::SetLevel {
                            pin: _,
                            level: _,
                        }
                    ) => {
                        // currently we don't have defined any other non-dynamic Output Pins that could be set
                        // TODO refactor back: this should be usable
                        unreachable!()
                    }
                    HostToAssistant::SetDynamicPin(
                        pin::SetLevel {
                            pin,
                            level,
                        }
                    ) => {
                        let pin_number = pin.get_pin_number().unwrap();

                        // TODO less repetitive once we've unified pin types
                        // TODO maybe just add a set_level() function to the hal?
                        match pin_number {
                            RED_LED_PIN_NUMBER => {
                                if pinint0_pin.direction_is_output() {
                                    match level {
                                        pin::Level::High => { pinint0_pin.set_high() }
                                        pin::Level::Low => { pinint0_pin.set_low() }
                                    }
                                }
                            }
                            CTS_PIN_NUMBER => {
                                if cts.direction_is_output() {
                                    match level {
                                        pin::Level::High => { cts.set_high() }
                                        pin::Level::Low => { cts.set_low() }
                                    }
                            }}
                            RTS_PIN_NUMBER => { unreachable!() }
                            pin_number => {
                                dyn_noint_pins.lock(|pin_map|{ // TODO turn this into a macro?
                                    if pin_map.contains_key(&pin_number) {
                                        let pin = pin_map.get_mut(&pin_number).unwrap();
                                        if pin.direction_is_output() {
                                            match level {
                                                pin::Level::High => { pin.set_high() }
                                                pin::Level::Low => { pin.set_low() }
                                            }
                                        }
                                        // TODO add pin level to buffer right away
                                    }
                                    else {
                                        rprintln!("can't query unsupported pin: {:?}", pin_number);
                                    }
                                })
                            }
                        };

                        Ok(())
                    }
                    HostToAssistant::ReadPin(
                        pin::ReadLevel { pin }
                    ) => {
                        let result = pins.get(&(pin as usize))
                            .map(|&(level, period_ms)| {
                                pin::ReadLevelResult {
                                    pin,
                                    level,
                                    period_ms,
                                }
                            });

                        host_tx
                            .send_message(
                                &AssistantToHost::ReadPinResult(result),
                                &mut buf,
                            )
                            .unwrap();

                        Ok(())
                    }
                    HostToAssistant::SetDirection(
                        pin::SetDirection {
                            pin,
                            direction: pin::Direction::Input,
                            level: None,
                        }
                    ) => {
                        //rprintln!("{:?} is Input", pin);

                        match pin.get_pin_number().unwrap() {
                            RED_LED_PIN_NUMBER => {
                                pinint0_pin.switch_to_input();
                                // inintialize interruptable pins so that a status read is possible before the first level
                                // change (TODO is this a separate PR candidate?)
                                let pinint0_level = match pinint0_pin.is_high() {
                                    true => pin::Level::High,
                                    false => pin::Level::Low,
                                };
                                dynamic_int_pins.insert(RED_LED_PIN_NUMBER as usize, (pinint0_level, None))
                                                .unwrap();
                            },
                            CTS_PIN_NUMBER => {
                                // TODO proper error handling
                                rprintln!("CTS pin is never Input");
                                unreachable!()
                            }
                            RTS_PIN_NUMBER => {
                                rts.switch_to_input()
                            },
                            30 => {
                                // Ignore for now, we've hardcoded this pin as input
                                // TODO fix this
                            }
                            pin_number => {
                                dyn_noint_pins.lock(|pin_map|{
                                    if pin_map.contains_key(&pin_number) {
                                        // this is a dynamic non-interrupt pin, set its direction
                                        let pin = pin_map.get_mut(&pin_number).unwrap();
                                        pin.switch_to_input();
                                    }
                                    else {
                                        rprintln!("unsupported pin: {:?}", pin_number)
                                    }
                                });
                            },
                        };
                        Ok(())
                    },
                    // TODO merge this with SetDirection for Input; control flow is duplicate
                    HostToAssistant::SetDirection(
                        pin::SetDirection {
                            pin,
                            direction: pin::Direction::Output,
                            level: Some(level),
                        }
                    ) => {
                        //rprintln!("{:?} is Output | Level {:?}", pin, level);
                        // convert from lpc8xx_hal::gpio::Level to protocol::pin::Level
                        // TODO use into() here
                        let gpio_level = match level {
                            pin::Level::High => {gpio::Level::High}
                            pin::Level::Low => {gpio::Level::Low}
                        };

                        // todo nicer and more generic once we start enabling ALL the pins
                        match pin.get_pin_number().unwrap() {
                            RED_LED_PIN_NUMBER => {
                                pinint0_pin.switch_to_output(gpio_level);
                                // inintialize interruptable pins so that a status read is
                                // possible before the first level change
                                // (TODO is this a separate PR candidate?)
                                let pinint0_level = match pinint0_pin.is_high() {
                                    true => pin::Level::High,
                                    false => pin::Level::Low,
                                };
                                dynamic_int_pins.insert(RED_LED_PIN_NUMBER as usize, (pinint0_level, None))
                                                .unwrap();
                            },
                            CTS_PIN_NUMBER => cts.switch_to_output(gpio_level),
                            RTS_PIN_NUMBER => {
                                // TODO proper error handling
                                rprintln!("RTS pin is never Output");
                                unreachable!()
                            }
                            pin_number => {
                                dyn_noint_pins.lock(|pin_map|{
                                    if pin_map.contains_key(&pin_number) {
                                        // this is a dynamic non-interrupt pin, set its direction
                                        let pin = pin_map.get_mut(&pin_number).unwrap();
                                        pin.switch_to_output(gpio_level);
                                    } else {
                                        rprintln!("unsupported pin")
                                    }
                                });
                            },
                        };
                        Ok(())
                    },
                    HostToAssistant::SetDirection(
                        pin::SetDirection {
                            pin: _,
                            direction: _,
                            level: _,
                        }
                    ) => {
                        // illegal level/direction combination
                        // TODO handle error more neatly
                        unreachable!()
                    },
                    HostToAssistant::ReadDynamicPin(
                        pin::ReadLevel { pin }
                    ) => {
                        //rprintln!("READ DYNAMIC PIN {:?}", pin);

                        let pin_number = pin.get_pin_number().unwrap();

                        // TODO return bool from closure and when I've figured that out also
                        // fix pin_is_input
                        let mut is_dyn_noint_pin = false;
                        dyn_noint_pins.lock(|pin_map|{
                            is_dyn_noint_pin = pin_map.contains_key(&pin_number);
                        });

                        let result = match (pin_number, is_dyn_noint_pin) {
                            (30, false) => {
                                // is target timer; not dynamic yet
                                // TODO don't hardcode this!
                                pins
                                    .get(&(InputPin::Blue as usize))
                                    .map(|&(level, period_ms)| {
                                        pin::ReadLevelResult {
                                            pin,
                                            level,
                                            period_ms,
                                        }
                                    })
                            }
                            (pin_number, true) => {
                                dynamic_noint_pins
                                .get(&(pin_number as usize))
                                .map(|gpio_level| {
                                    pin::ReadLevelResult {
                                        pin,
                                        level: pin::Level::from(*gpio_level),
                                        period_ms: None,
                                    }
                                })
                            }
                            (pin_number, false) => {
                                dynamic_int_pins
                                .get(&(pin_number as usize))
                                .map(|&(level, period_ms)| {
                                    pin::ReadLevelResult {
                                        pin,
                                        level,
                                        period_ms,
                                    }
                                })
                            }
                        };

                        host_tx
                            .send_message(
                                &AssistantToHost::ReadPinResultDynamic(result),
                                &mut buf,
                            )
                            .unwrap();

                        Ok(())

                    }
                }
            })
            .expect("Error processing host request");
        host_rx.clear_buf();

        // TODO: is pwm pin ever handled in reading messages?
        handle_pin_interrupt(pwm,   InputPin::Pwm,   &mut pins);
        handle_pin_interrupt(blue_idle, InputPin::Blue, &mut pins);

        handle_pin_interrupt_dynamic(pinint0_idle, PININT0_DYN_PIN, &mut dynamic_int_pins);
        handle_pin_interrupt_dynamic(
            target_rts_idle,
            DynamicPin::GPIO(RTS_PIN_NUMBER),
            &mut dynamic_int_pins,
        );
        handle_pin_interrupt_noint_dynamic(dyn_noint_levels_out, &mut dynamic_noint_pins);

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
