use snafu::prelude::*;
use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
    time::Duration,
};

mod error;
use error::{IoSnafu, PortError, SerialPortSnafu};

const SUCCESS_CODE: u8 = 0b11111111;
const FAIL_CODE: u8 = 0b11111110;
const BAUD_RATE: u32 = 9600;

#[derive(Debug)]
/// Commands that can be sent to the fingerprint reader
pub enum Command {
    /// Initial handshake to ensure that the fingerprint reader is ready to receive commands
    Ready = 0,
    /// Enroll a fingerprint
    Enroll = 1,
    /// Match a fingerprint to an existing, stored template
    Match = 2,
    /// Clear all fingerprint models currently held by the fingerprint reader
    Clear = 3,
}

impl From<u8> for Command {
    fn from(byte: u8) -> Self {
        match byte {
            0 => Command::Ready,
            1 => Command::Enroll,
            2 => Command::Match,
            3 => Command::Clear,
            _ => panic!("Invalid command byte"), // The programmer themselves should ensure that this never happens
        }
    }
}

/// A port to the fingerprint reader
pub type FingerprintPort = Arc<Mutex<Box<dyn serialport::SerialPort>>>;

/*
Trait required to impl methods onto a type alias.

Also, I just learned that there's a better way to do this with the added benefit of being able to do dependecy injection,
but that's too much effort to change now and not needed for this project, so just learn from my mistakes.
Example with dependency injection: https://github.com/tokio-rs/axum/blob/main/examples/error-handling-and-dependency-injection/src/main.rs
*/
/// Trait that ports to a fingerprint reader must implement. Usually implemented on an Arc<Mutex<dyn serialport::SerialPort>>.
pub trait AttendancePort: Send + Sync + Sized {
    type Error;

    /// Get a port to the fingerprint reader
    fn get() -> Result<Self, Self::Error>;

    /// Enroll a fingerprint
    fn enroll_fingerprint(&self, fingerprint_id: u8) -> Result<(), Self::Error>;

    /// Match a fingerprint and return the fingerprint ID
    fn match_fingerprint(&self) -> Result<i32, Self::Error>;

    /// Clear all fingerprint models currently held by the fingerprint reader
    fn clear_fingerprints(&self) -> Result<(), Self::Error>;

    /// Send a command to the fingerprint reader, returning the response
    fn send_command(&self, command: &[u8; 2]) -> Result<[u8; 2], Self::Error>;
}

impl AttendancePort for FingerprintPort {
    type Error = PortError;

    fn get() -> Result<FingerprintPort, Self::Error> {
        let ports = serialport::available_ports().unwrap();
        println!("Number of available ports: {}", ports.len());
        for (i, port) in ports.iter().enumerate() {
            println!("Port index {}: {}", i, port.port_name);
        }

        // Automatically picks the first port (index 0) in the list
        // You can change the index here if the device is found on a different port
        let port_index = 0;
        let port_name = ports[port_index].port_name.clone();

        let command = [Command::Ready as u8, 0];
        let port = Arc::new(Mutex::new(
            serialport::new(port_name, BAUD_RATE)
                .timeout(Duration::from_millis(10))
                .open()
                .context(SerialPortSnafu {
                    command: Command::from(command[0]),
                })?,
        ));

        // CRUCIAL: The board takes 1-3 seconds to boot up, we must delay otherwise this first handshake will fail
        println!("Starting up the port...");
        std::thread::sleep(std::time::Duration::from_millis(3000));

        let command_response = port.send_command(&command)?;

        PortError::result_from_command_response(&command, &command_response)?;

        println!("Ready!");
        Ok(port)
    }

    fn enroll_fingerprint(&self, fingerprint_id: u8) -> Result<(), Self::Error> {
        let command = [Command::Enroll as u8, fingerprint_id];

        let command_response = self.send_command(&command)?;

        PortError::result_from_command_response(&command, &command_response)?;

        Ok(())
    }

    fn match_fingerprint(&self) -> Result<i32, PortError> {
        let command = [Command::Match as u8, 0];

        let command_response = self.send_command(&command)?;

        PortError::result_from_command_response(&command, &command_response)
    }

    fn clear_fingerprints(&self) -> Result<(), Self::Error> {
        let command = [Command::Clear as u8, 0];

        let command_response = self.send_command(&command)?;

        PortError::result_from_command_response(&command, &command_response)?;

        Ok(())
    }

    fn send_command(&self, command: &[u8; 2]) -> Result<[u8; 2], Self::Error> {
        let mut read_buf: [u8; 2] = [0; 2];

        match self.lock() {
            Ok(mut held_port) => {
                let n_written_bytes = held_port.write(command).context(IoSnafu {
                    command: Command::from(command[0]),
                })?;

                // Ensure that the entire command is sent before reading the response
                held_port.flush().context(IoSnafu {
                    command: Command::from(command[0]),
                })?;
                println!("Wrote {} bytes of command {:?}", n_written_bytes, command);

                // Keep reading until the response is received
                loop {
                    match held_port.read_exact(&mut read_buf) {
                        Ok(_) => {
                            println!("Read bytes: {:?}", read_buf);
                            if read_buf[0] == SUCCESS_CODE || read_buf[0] == FAIL_CODE {
                                println!("Command {:?} completed!", Command::from(command[0]));
                                return Ok(read_buf);
                            }
                            read_buf.fill(0);
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                        Err(e) => {
                            return Err(PortError::Io {
                                source: e,
                                command: Command::from(command[0]),
                            })
                        }
                    }
                }
            }
            Err(_) => Err(PortError::MutexPoison),
        }
    }
}
