//! API for remotely controlling and monitoring pins on a test node


use core::panic;
use std::{
    convert::TryInto,
    fmt::Debug,
    mem::transmute,
    thread::sleep,
    time::Duration,
};

use serde::{
    Deserialize,
    Serialize,
};

use protocol::{
    pin,
    HostToAssistant,
};

use crate::{
    assistant::{
        InputPin,
        AssistantError,
    },
    conn::{
        Conn,
        ConnReceiveError,
        ConnSendError,
    }
};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum PinDirection {
    Input,
    Output
}

// TODO investigate if copy is dangerous after all here
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
pub(crate) enum PinToken {
    GPIO(u8), // TODO make pinnumber
}

impl PinToken {
    pub(crate) fn try_as_input_pin(&self) -> Result<protocol::InputPin, AssistantError> {
        match self {
            PinToken::GPIO(30) => Ok(protocol::InputPin::Blue),
            PinToken::GPIO(31) => Ok(protocol::InputPin::Green),
            PinToken::GPIO(18) => Ok(protocol::InputPin::Rts),
            // TODO what's the pwm pin again?
            _ => Err(AssistantError::NotDynamic),
        }
    }

    pub(crate) fn try_as_output_pin(&self) -> Result<protocol::OutputPin, AssistantError> {
        match self {
            PinToken::GPIO(5) => Ok(protocol::OutputPin::Pin5),
            PinToken::GPIO(29) => Ok(protocol::OutputPin::Red),
            PinToken::GPIO(19) => Ok(protocol::OutputPin::Cts),
            _ => Err(AssistantError::NotDynamic),
        }
    }
}

// /// Represents one of the pins that the assistant is monitoring
// #[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
// pub enum InputPin {
//     Blue  = 0,
//     Green = 1,
//     Rts   = 2,
//     Pwm   = 3,
// }

// /// Represents one of the pins that the assistant can set
// #[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
// pub enum OutputPin {
//     Pin5,
//     Cts,
//     Red,
// }


// impl From<pin::SetLevel<DynamicPin>> for HostToAssistant<'_> {
//     fn from(set_level: pin::SetLevel<DynamicPin>) -> Self {
//         // todo: If we are input, send input, if we are output, send output
//         // Self::SetDynamicPin(set_level)
//         todo!("See above");
//     }
// }

// impl From<pin::ReadLevel<DynamicPin>> for HostToAssistant<'_> {
//     fn from(read_level: pin::ReadLevel<DynamicPin>) -> Self {
//         // todo: If we are input, send input, if we are output, send output
//         // Self::ReadDynamicPin(read_level)
//         todo!("See above");
//     }
// }

/// API for remotely controlling and monitoring a pin on a test node
///
/// This struct is intended as a building block for higher-level interfaces
/// that control the test nodes of a specific test stand.
pub struct Pin<Id> {
    pub(crate) pin: Id,
}

impl<Id> Pin<Id>
where
    Id: Copy,
{
    /// Create a new instance of `Pins`
    pub fn new(pin: Id) -> Self {
        Self {
            pin,
        }
    }

    /// Commands the node to change pin level
    ///
    /// Constructs the command, calls the `wrap` closure to wrap that command
    /// into a message that the node will understand, then sends that message to
    /// the node through `conn`.
    pub fn set_level<M>(&mut self,
        level: pin::Level,
        conn: &mut Conn,
    )
        -> Result<(), ConnSendError>
        where
            M: From<pin::SetLevel<Id>> + Serialize,
    {
        let command = pin::SetLevel { pin: self.pin, level };
        let message: M = command.into();
        conn.send(&message)?;

        Ok(())
    }

    /// Read level for the given pin
    ///
    /// Receives from `conn`, expecting to receive a "level changed" message.
    /// Uses `unwrap` to get a `pin::LevelChange` from the message.
    pub fn read_level<'de, Request, Reply>(&mut self,
        timeout: Duration,
        conn: &mut Conn,
    )
        -> Result<(pin::Level, Option<u32>), ReadLevelError>
        where
            Id: Debug + Eq,
            Request: From<pin::ReadLevel<Id>> + Serialize,
            Reply: TryInto<pin::ReadLevelResult<Id>, Error=Reply>
                + Debug
                + Deserialize<'de>,
    {
        // Wait for a bit, to give whatever event is expected to change the
        // level some time to happen.
        sleep(timeout);

        let request = pin::ReadLevel {  pin: self.pin };
        let request: Request = request.into();
        conn.send(&request)
            .map_err(|err| ReadLevelError::Send(err))?;

        // The compiler believes that `buf` doesn't live long enough, because
        // the lifetime of the buffer needs to be `'de`, due to the
        // `Deserialize` bound on `Reply`. This is wrong though: Nothing we
        // return from this method still references the buffer, so the following
        // `transmute`, which transmutes a mutable reference to `buf` to a
        // mutable reference with unbounded lifetime, is sound.
        let mut buf: Vec<u8> = Vec::new();
        let buf = unsafe { transmute(&mut buf) };

        let reply = conn.receive::<Reply>(timeout, buf)
            .map_err(|err| ReadLevelError::Receive(err))?;

        match reply.try_into() {
            Ok(
                pin::ReadLevelResult {
                    pin,
                    level,
                    period_ms,
                }
            )
                if pin == self.pin
            => {
                Ok((level, period_ms))
            }
            Err(message) => {
                Err(
                    ReadLevelError::UnexpectedMessage(
                        format!("{:?}", message)
                    )
                )
            }
            message => {
                Err(
                    ReadLevelError::UnexpectedMessage(
                        format!("{:?}", message)
                    )
                )
            }
        }
    }
}


#[derive(Debug)]
pub enum ReadLevelError {
    Send(ConnSendError),
    Receive(ConnReceiveError),
    UnexpectedMessage(String),
    Timeout,
}
