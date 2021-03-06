# STM32L4 Test Stand

## About

Test stand for some peripheral APIs of [stm32l4xx-hal].

This test stand is modelled on the [LPC845 Test Stand] and re-uses some parts of it. Please check out the README there for more documentation and troubleshooting tips.


## Hardware setup

You need the following development boards:

- Target: [STM32L433 Nucleo](https://www.st.com/en/evaluation-tools/nucleo-l433rc-p.html)
- Assistant: [LPC845-BRK](https://www.nxp.com/products/processors-and-microcontrollers/arm-microcontrollers/general-purpose-mcus/lpc800-cortex-m0-plus-/lpc845-breakout-board-for-lpc84x-family-mcus:LPC845-BRK)

Connect both boards to the host computer via their USB ports. This is required both to download the firmware and to communicate with it during the test.

In addition, you need to connect the following pins of the target and the assistant:

| Target  | Assistant | Note                                 |
| ------- | --------- | ------------------------------------ |
| CN5  35 |        19 | USART: CTS                           |
| CN6  12 |        24 | I2C: SDA                             |
| CN6  14 |        23 | I2C: SCL                             |
| CN6  16 |        15 | USART: Target RX (DMA), Assistant TX |
| CN6  22 |         4 | SPI: SSEL                            |
| CN7   1 |        12 | USART: Target TX, Assistant RX       |
| CN7   3 |         8 | PWM                                  |
| CN7   4 |         2 | SPI: MOSI                            |
| CN7   5 |         3 | SPI: MISO                            |
| CN7   6 |         1 | SPI: SCK                             |
| CN7   9 |        13 | USART: Target RX, Assistant TX       |
| CN9   4 |        18 | USART: RTS                           |
| CN9   8 |        30 | Timer interrupt signal               |
| CN10  4 |        29 | GPIO: Target In, Assistant Out       |
| CN10  5 |        31 | GPIO: Target Out, Assistant In       |
| CN10  6 |         5 | ADC                                  |


[stm32l4xx-hal]: https://github.com/stm32-rs/stm32l4xx-hal
[LPC845 Test Stand]: https://github.com/braun-embedded/embedded-test-stand/tree/master/lpc845-test-stand

### Software setup

Besides a Rust toolchain, you need `cargo-embed` to download the firmware:

```
cargo install cargo-embed --version 0.10.1
```

Since this test stand, in contrast to the LPC845 test stand, uses two different kinds of development boards, you don't need to set up any serial numbers in the documentation.
