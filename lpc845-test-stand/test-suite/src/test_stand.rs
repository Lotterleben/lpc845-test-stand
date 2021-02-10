use std::sync::{LockResult, MutexGuard};

use host_lib::{
    assistant::{Assistant, AssistantInterface},
    test_stand::NotConfiguredError,
};

use super::target::Target;


/// An instance of the test stand
///
/// Used to access all resources that a test case requires.
pub struct TestStand {
    _guard: LockResult<MutexGuard<'static, ()>>,

    pub target: Option<Target>,
    pub assistant: AssistantInterface<Assistant>,
}

impl TestStand {
    /// Initializes the test stand
    ///
    /// Reads the `test-stand.toml` configuration file and initializes test
    /// stand resources, as configured in there.
    pub fn new() -> Result<Self, TestStandInitError> {

        let test_stand =
            host_lib::TestStand::new().map_err(|err| TestStandInitError::Inner(err))?;

        let target = match test_stand.target {
            Ok(conn) => { Some(Target::new(conn)) }
            Err(_) => {
                // Could not configure target. Assuming that no target is set in `test-stand.toml`
                // and an extranal target is being used
                None
            }
        };

        Ok(Self {
            _guard: test_stand.guard,
            target: target,
            assistant: AssistantInterface::new(test_stand.assistant?),
        })
    }
}

#[derive(Debug)]
pub enum TestStandInitError {
    Inner(host_lib::test_stand::TestStandInitError),
    NotConfigured(NotConfiguredError),
}

impl From<NotConfiguredError> for TestStandInitError {
    fn from(err: NotConfiguredError) -> Self {
        Self::NotConfigured(err)
    }
}
