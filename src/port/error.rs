use super::{Command, FAIL_CODE, SUCCESS_CODE};
use snafu::prelude::*;
use std::io;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
/// Errors that can occur when using the fingerprint port
pub enum PortError {
    #[snafu(display("An IO error occured while using the port: {source}"))]
    Io { source: io::Error, command: Command },
    #[snafu(display("The Mutex holding the port was poisoned"))]
    MutexPoison,
    #[snafu(display("A serial port error occured: {source}"))]
    SerialPort {
        source: serialport::Error,
        command: Command,
    },
    #[snafu(display("An error occured while executing a fingerprint command: {source}"))]
    Command {
        source: CommandError,
        command: Command,
    },
}

impl PortError {
    /// Converts a response from the port into a result that can be easily propagated away
    pub fn result_from_command_response(command: &[u8], response: &[u8]) -> Result<i32, Self> {
        match response[0] {
            SUCCESS_CODE => Ok(response[1] as i32), // The second byte contains the target data
            FAIL_CODE => Err(PortError::Command {
                source: CommandError::from(response[1]),
                command: Command::from(command[0]),
            }),
            _ => Err(PortError::Command {
                source: CommandError::InvalidResponse,
                command: Command::from(command[0]),
            }),
        }
    }
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
/// Errors that can occur when executing a fingerprint command on the board. These errors are converted from responses from the board.
pub enum CommandError {
    #[snafu(display("An invalid response was received when executing a fingerprint command."))]
    InvalidResponse,
    #[snafu(display("An unknown error occured when executing a fingerprint command."))]
    UnknownError,
    #[snafu(display("A packet error occured when executing a fingerprint command."))]
    PacketError,
    #[snafu(display("Failed to capture a fingerprint image."))]
    ImageFail,
    #[snafu(display("The fingerprint image is too messy."))]
    ImageMess,
    #[snafu(display("Failed to generate a character file due to the lack of character points of the fingerprint image."))]
    FeatureFail,
    #[snafu(display("An invalid image was detected when executing a fingerprint command."))]
    InvalidImage,
    #[snafu(display("The first and second fingers did not match during enrollment."))]
    EnrollMismatch,
    #[snafu(display("Invalid location to store the fingerprint template model"))]
    BadLocation,
    #[snafu(display("A flash error occured when executing a fingerprint command."))]
    FlashError,
    #[snafu(display("No matching fingerprint was found."))]
    FingerprintNotFound,
    #[snafu(display("An unknown command was received by the device."))]
    UnknownCommand,
}

impl From<u8> for CommandError {
    fn from(byte: u8) -> Self {
        match byte {
            1 => CommandError::UnknownError,
            2 => CommandError::PacketError,
            3 => CommandError::ImageFail,
            4 => CommandError::ImageMess,
            5 => CommandError::FeatureFail,
            6 => CommandError::InvalidImage,
            7 => CommandError::EnrollMismatch,
            8 => CommandError::BadLocation,
            9 => CommandError::FlashError,
            10 => CommandError::FingerprintNotFound,
            11 => CommandError::UnknownCommand,
            _ => CommandError::UnknownError,
        }
    }
}
