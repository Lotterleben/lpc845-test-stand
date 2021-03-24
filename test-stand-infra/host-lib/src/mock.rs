use serialport::{
    SerialPort,
    Error as SpError,
    ClearBuffer,
    StopBits,
    Parity,
    DataBits,
    FlowControl,
};
use std::io::{Read, Write};
use std::io::Error as IoError;
use std::time::Duration;

use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

pub struct Mock {
    data_out: Arc<Mutex<VecDeque<Vec<u8>>>>,
    data_in: Arc<Mutex<VecDeque<Vec<u8>>>>,
}

impl Mock {
    pub fn new() -> Self {
        Self {
            data_out: Arc::new(Mutex::new(VecDeque::new())),
            data_in: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
}

impl Read for Mock {
    fn read(&mut self, _: &mut [u8]) -> Result<usize, IoError> {
        todo!()
    }
}

impl Write for Mock {
    fn write(&mut self, _: &[u8]) -> Result<usize, IoError> {
        todo!()
    }
    fn flush(&mut self) -> Result<(), IoError> {
        todo!()
    }
}

impl SerialPort for Mock {
    fn name(&self) -> std::option::Option<std::string::String> {
        todo!()
    }
    fn baud_rate(&self) -> Result<u32, SpError> {
        todo!()
    }
    fn data_bits(&self) -> Result<DataBits, SpError> {
        todo!()
    }
    fn flow_control(&self) -> Result<FlowControl, SpError> {
        todo!()
    }
    fn parity(&self) -> Result<Parity, SpError> {
        todo!()
    }
    fn stop_bits(&self) -> Result<StopBits, SpError> {
        todo!()
    }
    fn timeout(&self) -> Duration {
        todo!()
    }
    fn set_baud_rate(&mut self, _: u32) -> Result<(), SpError> {
        todo!()
    }
    fn set_data_bits(&mut self, _: DataBits) -> Result<(), SpError> {
        todo!()
    }
    fn set_flow_control(
        &mut self,
        _: FlowControl,
    ) -> Result<(), SpError> {
        todo!()
    }
    fn set_parity(&mut self, _: Parity) -> Result<(), SpError> {
        todo!()
    }
    fn set_stop_bits(&mut self, _: StopBits) -> Result<(), SpError> {
        todo!()
    }
    fn set_timeout(&mut self, _: Duration) -> Result<(), SpError> {
        todo!()
    }
    fn write_request_to_send(&mut self, _: bool) -> Result<(), SpError> {
        todo!()
    }
    fn write_data_terminal_ready(&mut self, _: bool) -> Result<(), SpError> {
        todo!()
    }
    fn read_clear_to_send(&mut self) -> Result<bool, SpError> {
        todo!()
    }
    fn read_data_set_ready(&mut self) -> Result<bool, SpError> {
        todo!()
    }
    fn read_ring_indicator(&mut self) -> Result<bool, SpError> {
        todo!()
    }
    fn read_carrier_detect(&mut self) -> Result<bool, SpError> {
        todo!()
    }
    fn bytes_to_read(&self) -> Result<u32, SpError> {
        todo!()
    }
    fn bytes_to_write(&self) -> Result<u32, SpError> {
        todo!()
    }
    fn clear(&self, _: ClearBuffer) -> Result<(), SpError> {
        todo!()
    }
    fn try_clone(&self) -> Result<Box<(dyn SerialPort + 'static)>, SpError> {
        Ok(Box::new(Self {
            data_in: self.data_in.clone(),
            data_out: self.data_out.clone(),
        }))
    }
    fn set_break(&self) -> Result<(), SpError> {
        todo!()
    }
    fn clear_break(&self) -> Result<(), SpError> {
        todo!()
    }
}
