//! Generic protocol related to pins
//!
//! The types in this module are not specific to any test stand setup, and can
//! be re-used for different test stands.


use serde::{
    Deserialize,
    Serialize,
};

use lpc8xx_hal::gpio;

/// Sent by the host to command a test node to set a pin to a specific level
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct SetLevel<Id> {
    /// The pin whose level should be set
    pub pin: Id,

    /// The new level of the pin
    pub level: Level,
}

/// Sent by the host to command a test node to set a pin to a specific direction
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct SetDirection<Id> {
    /// The pin whose direction should be set
    pub pin: Id,

    /// The new direction of the pin
    pub direction: Direction,

    /// The initial level the pin should be set to. Ignored if `direction` is Input.
    pub level: Option<Level>,
}

/// Sent by the host to request the current level of a pin
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct ReadLevel<Id> {
    /// The pin whose level to read
    pub pin: Id,
}

/// Sent by a test node in response to a `ReadLevel` message
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct ReadLevelResult<Id> {
    /// The pin whose level has changed
    pub pin: Id,

    /// The new level of the pin
    pub level: Level,

    /// The period since the last change of this pin's level, in milliseconds
    ///
    /// This value might not be available, because this is the first change of
    /// this pin's level, or because the test node doesn't measure the period.
    ///
    /// If the time since the last change has been too long, this value will
    /// not be reliable.
    pub period_ms: Option<u32>,
}

/// The voltage level of a pin
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum Level {
    /// High voltage
    High,
    /// Low voltage
    Low,
}

impl From<gpio::Level> for Level {
    fn from(level: gpio::Level) -> Self {
        match level {
            gpio::Level::High => Level::High,
            gpio::Level::Low => Level::Low,
        }
    }
}

impl From<Level> for gpio::Level {
    fn from(level: Level) -> Self {
        match level {
            Level::High => gpio::Level::High,
            Level::Low => gpio::Level::Low,
        }
    }
}

/// Represents the direction of a pin
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum Direction {
    Input,
    Output,
}
