//! Library to support the test suite running on the host computer

// AJM TODO
#![allow(unreachable_code)]

pub mod assistant;
pub mod config;
pub mod conn;
pub mod error;
pub mod pin;
pub mod test_stand;


compile_error!("""
    Notes: 2021-03-31

    We need to begin removing the todo statements, primarily by
    using a match statement to convert the internal DynamicPin Token
    into ephemeral instances of InputPin or OutputPin in order to
    create the correct type for serialization

    Lotte: Replace our local DynamicPin with a HAL DynamicPin?
    James: What about other HALs?
    Lotte: Can we make this a trait/generic instead? Use ones we have?
    James: This requires dynamic dispatch, but that might be okay

    Consider doing this instead of manual matching everywhere
""");

pub use self::{
    assistant::Assistant,
    config::Config,
    conn::Conn,
    error::{
        Error,
        Result,
    },
    test_stand::TestStand,
};
