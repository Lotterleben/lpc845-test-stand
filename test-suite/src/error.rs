/// Test-suite specific error module
use host_lib::test_stand::NotConfiguredError;

use super::{
    assistant::{
        AssistantExpectNothingError, AssistantPinOperationError, AssistantPinReadError,
        AssistantSetPinDirectionInputError, AssistantSetPinDirectionOutputError,
        AssistantSetPinHighError, AssistantSetPinLowError, AssistantUsartSendError,
        AssistantUsartWaitError,
    },
    target::{
        TargetI2cError, TargetPinReadError, TargetSetPinHighError, TargetSetPinLowError,
        TargetSpiError, TargetStartTimerInterruptError, TargetUsartSendError, TargetUsartWaitError,
        TargetWaitForAddressError,
    },
    test_stand::TestStandInitError,
};

/// Result type specific to this test suite
pub type Result<T = ()> = std::result::Result<T, Error>;

/// Error type specific to this test suite
#[derive(Debug)]
pub enum Error {
    AssistantExpectNothing(AssistantExpectNothingError),
    AssistantPinRead(AssistantPinReadError),
    AssistantPinOperation(AssistantPinOperationError),
    AssistantSetPinHigh(AssistantSetPinHighError),
    AssistantSetPinLow(AssistantSetPinLowError),
    AssistantSetPinDirectionInput(AssistantSetPinDirectionInputError),
    AssistantSetPinDirectionOutput(AssistantSetPinDirectionOutputError),
    AssistantUsartSend(AssistantUsartSendError),
    AssistantUsartWait(AssistantUsartWaitError),
    NotConfigured(NotConfiguredError),
    TargetI2c(TargetI2cError),
    TargetPinRead(TargetPinReadError),
    TargetSetPinHigh(TargetSetPinHighError),
    TargetSetPinLow(TargetSetPinLowError),
    TargetSpi(TargetSpiError),
    TargetStartTimerInterrupt(TargetStartTimerInterruptError),
    TargetUsartSend(TargetUsartSendError),
    TargetUsartWait(TargetUsartWaitError),
    TargetWaitForAddress(TargetWaitForAddressError),
    TestStandInit(TestStandInitError),
}

impl From<AssistantExpectNothingError> for Error {
    fn from(err: AssistantExpectNothingError) -> Self {
        Self::AssistantExpectNothing(err)
    }
}

impl From<AssistantPinReadError> for Error {
    fn from(err: AssistantPinReadError) -> Self {
        Self::AssistantPinRead(err)
    }
}

impl From<AssistantPinOperationError> for Error {
    fn from(err: AssistantPinOperationError) -> Self {
        Self::AssistantPinOperation(err)
    }
}

impl From<AssistantSetPinHighError> for Error {
    fn from(err: AssistantSetPinHighError) -> Self {
        Self::AssistantSetPinHigh(err)
    }
}

impl From<AssistantSetPinLowError> for Error {
    fn from(err: AssistantSetPinLowError) -> Self {
        Self::AssistantSetPinLow(err)
    }
}

impl From<AssistantSetPinDirectionInputError> for Error {
    fn from(err: AssistantSetPinDirectionInputError) -> Self {
        Self::AssistantSetPinDirectionInput(err)
    }
}

impl From<AssistantSetPinDirectionOutputError> for Error {
    fn from(err: AssistantSetPinDirectionOutputError) -> Self {
        Self::AssistantSetPinDirectionOutput(err)
    }
}

impl From<AssistantUsartSendError> for Error {
    fn from(err: AssistantUsartSendError) -> Self {
        Self::AssistantUsartSend(err)
    }
}

impl From<AssistantUsartWaitError> for Error {
    fn from(err: AssistantUsartWaitError) -> Self {
        Self::AssistantUsartWait(err)
    }
}

impl From<NotConfiguredError> for Error {
    fn from(err: NotConfiguredError) -> Self {
        Self::NotConfigured(err)
    }
}

impl From<TargetI2cError> for Error {
    fn from(err: TargetI2cError) -> Self {
        Self::TargetI2c(err)
    }
}

impl From<TargetPinReadError> for Error {
    fn from(err: TargetPinReadError) -> Self {
        Self::TargetPinRead(err)
    }
}

impl From<TargetSpiError> for Error {
    fn from(err: TargetSpiError) -> Self {
        Self::TargetSpi(err)
    }
}

impl From<TargetStartTimerInterruptError> for Error {
    fn from(err: TargetStartTimerInterruptError) -> Self {
        Self::TargetStartTimerInterrupt(err)
    }
}

impl From<TargetUsartSendError> for Error {
    fn from(err: TargetUsartSendError) -> Self {
        Self::TargetUsartSend(err)
    }
}

impl From<TargetSetPinHighError> for Error {
    fn from(err: TargetSetPinHighError) -> Self {
        Self::TargetSetPinHigh(err)
    }
}

impl From<TargetSetPinLowError> for Error {
    fn from(err: TargetSetPinLowError) -> Self {
        Self::TargetSetPinLow(err)
    }
}

impl From<TargetUsartWaitError> for Error {
    fn from(err: TargetUsartWaitError) -> Self {
        Self::TargetUsartWait(err)
    }
}

impl From<TargetWaitForAddressError> for Error {
    fn from(err: TargetWaitForAddressError) -> Self {
        Self::TargetWaitForAddress(err)
    }
}

impl From<TestStandInitError> for Error {
    fn from(err: TestStandInitError) -> Self {
        Self::TestStandInit(err)
    }
}
