//! Test assistant firmware
//!
//! Used to assist the test suite in interfacing with the test target. Needs to
//! be downloaded to an LPC845-BRK board before the test cases can be run.

#![no_main]
#![no_std]

extern crate panic_rtt_target;

use core::marker::PhantomData;

use heapless::{consts::U4, consts::U8, spsc::Consumer, spsc::Producer, FnvIndexMap};
use lpc8xx_hal::{
    prelude::*,
    cortex_m::interrupt,
    gpio::{self, direction::Dynamic, direction::Output, DynamicGpioPin, GpioPin},
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

// By default (and we haven't changed that setting)
// the SysTick timer runs at half the system
// frequency. The system frequency runs at 12 MHz by
// default (again, we haven't changed it), meaning
// the SysTick timer runs at 6 MHz.
//
// At 6 MHz, 1 ms are 6000 timer ticks.
// TODO: value picked for human reada/debuggability; adjust
const TIMER_INT_PERIOD_MS: u32 = 200 * 6000; // fires every 200 milliseconds

// TODO find a place to share them with t-s and t-t?
/// some commonly used pin numbers
const RTS_PIN_NUMBER: u8 = 18;
const CTS_PIN_NUMBER: u8 = 19;
const RED_LED_PIN_NUMBER: u8 = 29;
const GREEN_LED_PIN_NUMBER : u8 = 31;

/// The maxiumum number of GPIO pins that are direction-changeable at runtime and read
/// periodically (i.e. do not trigger any interrupts)
#[allow(non_camel_case_types)]
type NUM_DYN_NOINT_PINS = U4;

/// NOTE TO USERS: adjust the pins in this list to change which pin *could* be used as input pin.
/// Currently only two Input-able pins are supported.
#[allow(non_camel_case_types)]
type PININT0_PIN = lpc8xx_hal::pins::PIO1_2; // make sure that these
const PININT0_DYN_PIN: DynamicPin = DynamicPin::GPIO(RED_LED_PIN_NUMBER); // two match!

#[rtic::app(device = lpc8xx_hal::pac)]
const APP: () = {
    struct Resources {
        host_rx_int: RxInt<'static, USART0, AsyncMode>,
        host_rx_idle: RxIdle<'static>,
        host_tx: Tx<USART0, AsyncMode>,

        target_rx_int: RxInt<'static, USART1, AsyncMode>,
        target_rx_idle: RxIdle<'static>,
        target_tx: Tx<USART1, AsyncMode>,
        target_tx_dma:
            usart::Tx<USART2, usart::state::Enabled<u8, AsyncMode>, usart::state::NoThrottle>,
        target_rts_int: pin_interrupt::Int<'static, PININT2, PIO0_9, MRT2>,
        target_rts_idle: pin_interrupt::Idle<'static>,

        target_sync_rx_int: RxInt<'static, USART3, SyncMode>,
        target_sync_rx_idle: RxIdle<'static>,
        target_sync_tx: Tx<USART3, SyncMode>,

        target_timer_int: pin_interrupt::Int<'static, PININT1, PIO1_1, MRT1>,
        blue_idle: pin_interrupt::Idle<'static>,

        pinint0_int: pin_interrupt::Int<'static, PININT0, PININT0_PIN, MRT0>,
        pinint0_idle: pin_interrupt::Idle<'static>,

        dyn_noint_pins: FnvIndexMap<u8, DynamicGpioPin<Dynamic>, NUM_DYN_NOINT_PINS>,
        /// Level measurements for pins in dyn_noint_pins, indexed by pin id
        dyn_noint_levels_in:
            // TODO increase size
            Producer<'static, PinMeasurementEvent, U4>,
        dyn_noint_levels_out:
            // TODO refactor this and the bove into pin_interrupt(-like) Int/Idle wrappers?
            // TODO increase size
            Consumer<'static, PinMeasurementEvent, U4>,

        pwm_int:  pin_interrupt::Int<'static, PININT3, PIO0_23, MRT3>,
        pwm_idle: pin_interrupt::Idle<'static>,

        pin_5: GpioPin<PIO0_20, Output>,
        rts: GpioPin<PIO0_9, Dynamic>, // TODO make unidirectional again
        cts: GpioPin<PIO0_8, Dynamic>, // TODO make unidirectional again
        pinint0_pin: GpioPin<PININT0_PIN, Dynamic>, // pin that triggers PININT0 interrupt

        i2c: i2c::Slave<I2C0, Enabled<PhantomData<IOSC>>, Enabled>,
        spi: SPI<SPI0, Enabled<spi::Slave>>,
    }

    #[init]
    fn init(context: init::Context) -> init::LateResources {
        // Normally, access to a `static mut` would be unsafe, but we know that
        // this method is only called once, which means we have exclusive access
        // here. RTFM knows this too, and by putting these statics right here,
        // at the beginning of the method, we're opting into some RTFM magic
        // that gives us safe access to them.
        static mut HOST: Usart = Usart::new();
        static mut TARGET: Usart = Usart::new();
        static mut TARGET_SYNC: Usart = Usart::new();

        static mut INT0: PinInterrupt = PinInterrupt::new(); // formerly known as GREEN
        static mut TARGET_TIMER: PinInterrupt = PinInterrupt::new(); // formerly known as BLUE
        static mut RTS:  PinInterrupt = PinInterrupt::new();
        static mut PWM:  PinInterrupt = PinInterrupt::new();

        static mut PIN_TIMERINT: TimerInterrupt<PinMeasurementEvent> = TimerInterrupt::new();

        rtt_target::rtt_init_print!();
        rprintln!("Starting assistant.");

        // Get access to the device's peripherals. This can't panic, since this
        // is the only place in this program where we call this method.
        let p = Peripherals::take().unwrap_or_else(|| unreachable!());

        let mut systick = context.core.SYST;

        let mut syscon = p.SYSCON.split();
        let swm = p.SWM.split();
        let gpio = p.GPIO.enable(&mut syscon.handle);
        let pinint = p.PININT.enable(&mut syscon.handle);
        let timers = p.MRT0.split(&mut syscon.handle);

        let mut swm_handle = swm.handle.enable(&mut syscon.handle);

        // Initialize and enable timer interrupts
        systick.set_reload(TIMER_INT_PERIOD_MS);
        systick.clear_current();
        systick.enable_interrupt();
        systick.enable_counter();

        // Configure interrupts for pins that could be connected to target's GPIO pins
        let pinint0_pin = p.pins.pio1_2.into_dynamic_pin(
            gpio.tokens.pio1_2,
            gpio::Level::High, // off by default
            DynamicPinDirection::Input,
        );
        let mut pinint0_int = pinint
            .interrupts
            .pinint0
            .select::<PININT0_PIN>(&mut syscon.handle);
        pinint0_int.enable_rising_edge();
        pinint0_int.enable_falling_edge();

        // initialize data structures for all dynamic pins that are *not* interrupt-controlled
        let mut dyn_noint_pins = FnvIndexMap::<u8, DynamicGpioPin<Dynamic>, NUM_DYN_NOINT_PINS>::new();

        // TODO add ALL the pins \o,
        let test_pin_number31: u8 = GREEN_LED_PIN_NUMBER;
        let test_pin_number33: u8 = 33;
        let test_pin_number6: u8 = 6;
        let test_dyn_pin31 = p
            .pins
            .pio1_0
            .into_dynamic_pin_2(gpio.tokens.pio1_0, gpio::Level::Low, DynamicPinDirection::Input);
        let test_dyn_pin33 = p
            .pins
            .pio0_6
            .into_dynamic_pin_2(gpio.tokens.pio0_6, gpio::Level::Low, DynamicPinDirection::Input);
        let test_dyn_pin6 = p
            .pins
            .pio0_21
            .into_dynamic_pin_2(gpio.tokens.pio0_21, gpio::Level::Low, DynamicPinDirection::Input);
        let _ = dyn_noint_pins.insert(test_pin_number31, test_dyn_pin31);
        let _ = dyn_noint_pins.insert(test_pin_number33, test_dyn_pin33);
        let _ = dyn_noint_pins.insert(test_pin_number6, test_dyn_pin6);

        // init queue that stores level reading for all dyn noint pins
        let (dyn_noint_levels_in, dyn_noint_levels_out) = PIN_TIMERINT.init();

        // Configure interrupt for pin connected to target's timer interrupt pin
        let _target_timer = p.pins.pio1_1.into_input_pin(gpio.tokens.pio1_1);
        let mut target_timer_int = pinint
            .interrupts
            .pinint1
            .select::<PIO1_1>(&mut syscon.handle);
        target_timer_int.enable_rising_edge();
        target_timer_int.enable_falling_edge();

        // Configure interrupt for pin connected to target's PWM pin
        let _pwm = p.pins.pio0_23.into_input_pin(gpio.tokens.pio0_23);
        let mut pwm_int = pinint
            .interrupts
            .pinint3
            .select::<PIO0_23>(&mut syscon.handle);
        pwm_int.enable_rising_edge();
        pwm_int.enable_falling_edge();

        // Configure GPIO pin 5
        let pin_5 = p.pins.pio0_20.into_output_pin(
            gpio.tokens.pio0_20,
            gpio::Level::Low,
        );

        let cts = p
            .pins
            .pio0_8
            .into_dynamic_pin(gpio.tokens.pio0_8, gpio::Level::Low, DynamicPinDirection::Output);

        // Configure the clock for USART0, using the Fractional Rate Generator
        // (FRG) and the USART's own baud rate divider value (BRG). See user
        // manual, section 17.7.1.
        //
        // This assumes a system clock of 12 MHz (which is the default and, as
        // of this writing, has not been changed in this program). The resulting
        // rate is roughly 115200 baud.
        let clock_config = {
            syscon.frg0.select_clock(frg::Clock::FRO);
            syscon.frg0.set_mult(22);
            syscon.frg0.set_div(0xFF);
            usart::Clock::new(&syscon.frg0, 5, 16)
        };

        // Assign pins to USART0 for RX/TX functions. On the LPC845-BRK, those
        // are the pins connected to the programmer, and bridged to the host via
        // USB.
        //
        // Careful, the LCP845-BRK documentation uses the opposite designations
        // (i.e. from the perspective of the on-board programmer, not the
        // microcontroller).
        let (u0_rxd, _) = swm
            .movable_functions
            .u0_rxd
            .assign(p.pins.pio0_24.into_swm_pin(), &mut swm_handle);
        let (u0_txd, _) = swm
            .movable_functions
            .u0_txd
            .assign(p.pins.pio0_25.into_swm_pin(), &mut swm_handle);

        // Use USART0 to communicate with the test suite
        let mut host = p.USART0.enable_async(
            &clock_config,
            &mut syscon.handle,
            u0_rxd,
            u0_txd,
            usart::Settings::default(),
        );
        host.enable_interrupts(usart::Interrupts {
            RXRDY: true,
            ..usart::Interrupts::default()
        });

        // Assign pins to USART1.
        let (u1_rxd, _) = swm
            .movable_functions
            .u1_rxd
            .assign(p.pins.pio0_26.into_swm_pin(), &mut swm_handle);
        let (u1_txd, _) = swm
            .movable_functions
            .u1_txd
            .assign(p.pins.pio0_27.into_swm_pin(), &mut swm_handle);

        // Use USART1 to communicate with the test target
        let mut target = p.USART1.enable_async(
            &clock_config,
            &mut syscon.handle,
            u1_rxd,
            u1_txd,
            usart::Settings::default(),
        );
        target.enable_interrupts(usart::Interrupts {
            RXRDY: true,
            ..usart::Interrupts::default()
        });

        // Configure interrupt for RTS pin
        let rts = p.pins.pio0_9.into_dynamic_pin(
            gpio.tokens.pio0_9,
            gpio::Level::High, // off by default (shouldn't matter because rts is input)
            DynamicPinDirection::Input,
        );
        let mut rts_int = pinint
            .interrupts
            .pinint2
            .select::<PIO0_9>(&mut syscon.handle);
        rts_int.enable_rising_edge();
        rts_int.enable_falling_edge();
        let (rts_int, rts_idle) = RTS.init(rts_int, timers.mrt2);

        // Assign pins to USART2.
        let (u2_rxd, _) = swm
            .movable_functions
            .u2_rxd
            .assign(p.pins.pio0_28.into_swm_pin(), &mut swm_handle);
        let (u2_txd, _) = swm
            .movable_functions
            .u2_txd
            .assign(p.pins.pio0_29.into_swm_pin(), &mut swm_handle);

        // Use USART2 as secondary means to communicate with test target.
        let target2 = p.USART2.enable_async(
            &clock_config,
            &mut syscon.handle,
            u2_rxd,
            u2_txd,
            usart::Settings::default(),
        );

        // Assign pins to USART3.
        let (u3_rxd, _) = swm
            .movable_functions
            .u3_rxd
            .assign(p.pins.pio0_13.into_swm_pin(), &mut swm_handle);
        let (u3_txd, _) = swm
            .movable_functions
            .u3_txd
            .assign(p.pins.pio0_14.into_swm_pin(), &mut swm_handle);
        let (u3_sclk, _) = swm
            .movable_functions
            .u3_sclk
            .assign(p.pins.pio0_15.into_swm_pin(), &mut swm_handle);

        // Use USART3 as tertiary means to communicate with the test target.
        let mut target_sync = p.USART3.enable_sync_as_slave(
            &syscon.iosc,
            &mut syscon.handle,
            u3_rxd,
            u3_txd,
            u3_sclk,
            usart::Settings::default(),
        );
        target_sync.enable_interrupts(usart::Interrupts {
            RXRDY: true,
            ..usart::Interrupts::default()
        });

        let (host_rx_int, host_rx_idle, host_tx) = HOST.init(host);
        let (target_rx_int, target_rx_idle, target_tx) = TARGET.init(target);
        let (target_sync_rx_int, target_sync_rx_idle, target_sync_tx) =
            TARGET_SYNC.init(target_sync);

        let (pinint0_int, pinint0_idle) = INT0.init(pinint0_int, timers.mrt0);
        let (target_timer_int, blue_idle) =
            TARGET_TIMER.init(target_timer_int, timers.mrt1);
        let (pwm_int,     pwm_idle)   = PWM.init(pwm_int, timers.mrt3);

        // Assign I2C0 pin functions
        let (i2c0_sda, _) = swm
            .fixed_functions
            .i2c0_sda
            .assign(p.pins.pio0_11.into_swm_pin(), &mut swm_handle);
        let (i2c0_scl, _) = swm
            .fixed_functions
            .i2c0_scl
            .assign(p.pins.pio0_10.into_swm_pin(), &mut swm_handle);

        // Initialize I2C0
        let mut i2c = p
            .I2C0
            .enable(&syscon.iosc, i2c0_scl, i2c0_sda, &mut syscon.handle)
            .enable_slave_mode(0x48)
            .expect("Not using a valid address");
        i2c.enable_interrupts(i2c::Interrupts {
            slave_pending: true,
            ..i2c::Interrupts::default()
        });

        let (spi0_sck, _) = swm
            .movable_functions
            .spi0_sck
            .assign(p.pins.pio0_16.into_swm_pin(), &mut swm_handle);
        let (spi0_mosi, _) = swm
            .movable_functions
            .spi0_mosi
            .assign(p.pins.pio0_17.into_swm_pin(), &mut swm_handle);
        let (spi0_miso, _) = swm
            .movable_functions
            .spi0_miso
            .assign(p.pins.pio0_18.into_swm_pin(), &mut swm_handle);
        let (spi0_ssel0, _) = swm
            .movable_functions
            .spi0_ssel0
            .assign(p.pins.pio0_19.into_swm_pin(), &mut swm_handle);

        let mut spi = p.SPI0.enable_as_slave(
            &syscon.iosc,
            &mut syscon.handle,
            spi::MODE_0,
            spi0_sck,
            spi0_mosi,
            spi0_miso,
            spi0_ssel0,
        );
        spi.enable_interrupts(spi::Interrupts {
            rx_ready: true,
            ..Default::default()
        });
        spi.enable_interrupts(spi::Interrupts {
            rx_ready: true,
            slave_select_asserted: true,
            slave_select_deasserted: true,
            ..Default::default()
        });

        init::LateResources {
            host_rx_int,
            host_rx_idle,
            host_tx,

            target_rx_int,
            target_rx_idle,
            target_tx,
            target_tx_dma: target2.tx,
            target_rts_int: rts_int,
            target_rts_idle: rts_idle,

            target_sync_rx_int,
            target_sync_rx_idle,
            target_sync_tx,

            target_timer_int,
            blue_idle,

            pinint0_int,
            pinint0_idle,

            pinint0_pin,

            dyn_noint_pins,
            dyn_noint_levels_in,
            dyn_noint_levels_out,

            pwm_int,
            pwm_idle,

            pin_5,
            cts,
            rts,

            i2c: i2c.slave,
            spi,
        }
    }

    #[idle(
        resources = [
            host_rx_idle,
            host_tx,
            target_rx_idle,
            target_tx,
            target_tx_dma,
            target_sync_rx_idle,
            target_sync_tx,
            pinint0_idle,
            pwm_idle,
            blue_idle,
            target_rts_idle,
            pinint0_pin,
            pin_5,
            dyn_noint_pins,
            dyn_noint_levels_out,
            cts,
            rts,
        ]
    )]
    fn idle(cx: idle::Context) -> ! {
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

    #[task(binds = USART0, resources = [host_rx_int])]
    fn usart0(cx: usart0::Context) {
        // ignore serial errors from the host
        let _ = cx.resources
            .host_rx_int
            .receive();
    }

    #[task(binds = USART1, resources = [target_rx_int])]
    fn usart1(cx: usart1::Context) {
        cx.resources
            .target_rx_int
            .receive()
            .expect("Error receiving from USART1");
    }

    #[task(binds = PIN_INT6_USART3, resources = [target_sync_rx_int])]
    fn usart3(cx: usart3::Context) {
        cx.resources
            .target_sync_rx_int
            .receive()
            .expect("Error receiving from USART3");
    }

    #[task(binds = PIN_INT3, resources = [pwm_int])]
    fn pinint3(context: pinint3::Context) {
        context.resources.pwm_int.handle_interrupt();
    }

    #[task(binds = PIN_INT0, resources = [pinint0_int])]
    fn pinint0(context: pinint0::Context) {
        context.resources.pinint0_int.handle_interrupt();
    }

    #[task(binds = PIN_INT1, resources = [target_timer_int])]
    fn pinint1(context: pinint1::Context) {
        context.resources.target_timer_int.handle_interrupt();
    }

    #[task(binds = PIN_INT2, resources = [target_rts_int])]
    fn pinint2(context: pinint2::Context) {
        context.resources.target_rts_int.handle_interrupt();
    }

    #[task(binds = SysTick, resources = [dyn_noint_levels_in, dyn_noint_pins])]
    fn syst(context: syst::Context) {
        for (pin_number, pin) in context.resources.dyn_noint_pins.iter() {
            // TODO more elegantly? or add get_level to hal?
            let level = match pin.is_high() {
                true => {
                    gpio::Level::High
                }
                false => {
                    gpio::Level::Low
                }
            };

            let _ = context.resources.dyn_noint_levels_in.enqueue(
                PinMeasurementEvent{ pin_number: *pin_number, level}
                );
        }
    }

    #[task(binds = I2C0, resources = [i2c])]
    fn i2c0(context: i2c0::Context) {
        static mut DATA: Option<u8> = None;

        rprintln!("I2C: Handling I2C0 interrupt...");

        match context.resources.i2c.wait() {
            Ok(i2c::slave::State::AddressMatched(i2c)) => {
                rprintln!("I2C: Address matched.");

                i2c.ack().unwrap();

                rprintln!("I2C: Ack'ed address.");
            }
            Ok(i2c::slave::State::RxReady(i2c)) => {
                rprintln!("I2C: Ready to receive.");

                *DATA = Some(i2c.read().unwrap());
                i2c.ack().unwrap();

                rprintln!("I2C: Received and ack'ed.");
            }
            Ok(i2c::slave::State::TxReady(i2c)) => {
                rprintln!("I2C: Ready to transmit.");

                if let Some(data) = *DATA {
                    i2c.transmit(data << 1).unwrap();
                    rprintln!("I2C: Transmitted.");
                }
            }
            Err(nb::Error::WouldBlock) => {
                // I2C not ready; nothing to do
            }
            Err(err) => {
                panic!("I2C error: {:?}", err);
            }
        }
    }

    #[task(binds = SPI0, resources = [spi])]
    fn spi0(context: spi0::Context) {
        static mut ACTIVE: bool = false;

        let spi = context.resources.spi;

        if spi.is_slave_select_asserted() {
            *ACTIVE = true;
        }
        if *ACTIVE {
            if spi.is_ready_to_receive() {
                let data = spi.receive().unwrap();
                block!(spi.transmit(data << 1)).unwrap();
            }
        }
        if spi.is_slave_select_deasserted() {
            *ACTIVE = false;
        }
    }
};

/// Collect data from all Interrupts that were fired for `pin`
fn handle_pin_interrupt_dynamic(
    int: &mut pin_interrupt::Idle,
    pin: DynamicPin,
    // TODO: why are we even using usize for index if values never exceed u0?
    pins: &mut FnvIndexMap<usize, (pin::Level, Option<u32>), U4>,
) {
    while let Some(event) = int.next() {
        match event {
            pin_interrupt::Event { level, period } => {
                let pin_number = pin.get_pin_number().unwrap();
                // convert from hal level to our level
                let level = pin::Level::from(level);

                let period_ms = period.map(|value| value / 12_000);
                pins.insert(pin_number as usize, (level, period_ms))
                    .unwrap();
            }
        }
    }
}

// TODO merge w handle_pin_interrupt_dynamic / make more generic
fn handle_pin_interrupt_noint_dynamic(
    consumer: &mut Consumer<'static, PinMeasurementEvent, U4>,
    pins: &mut FnvIndexMap<usize, gpio::Level, U4>,
) {
    while let Some(event) = consumer.dequeue() {
        match event {
            PinMeasurementEvent { pin_number, level } => {
                // TODO note that this stores a diff level type than handle_pin_interrupt_dynamic()
                // -> angleichen
                pins.insert(pin_number as usize, level).unwrap();
            }
        }
    }
}

// TODO merge w handle_pin_interrupt_dynamic / make more generic
fn handle_pin_interrupt(
    int:  &mut pin_interrupt::Idle,
    pin:  InputPin,
    pins: &mut FnvIndexMap<usize, (pin::Level, Option<u32>), U8>,
) {
    while let Some(event) = int.next() {
        match event {
            pin_interrupt::Event { level, period } => {
                // convert from hal level to our level
                let level = pin::Level::from(level);

                let period_ms = period.map(|value| value / 12_000);
                pins.insert(pin as usize, (level, period_ms)).unwrap();
            }
        }
    }
}
