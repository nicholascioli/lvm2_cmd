use std::{error::Error, fmt::Display};

/// Represents an error returned by an LVM method
#[derive(Debug)]
pub enum LVMError {
    /// Represents an error after running the LVM2 command
    Command {
        command: String,
        args: Vec<String>,
        message: String,
    },

    /// Represents an error attempting to run the LVM2 command
    Internal { io: std::io::Error },

    /// Represents an error in parsing the output of the LVM2 command
    MalformedOutput { cause: String, result: String },

    /// Represents an error in finding a specified resource
    NotFound { resource: String },
}

impl Error for LVMError {}

impl Display for LVMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Command {
                command,
                args,
                message,
            } => write!(
                f,
                "could not run `{}` with args `{:?}`: {}",
                command, args, message
            ),
            Self::Internal { io } => write!(f, "could not run lvm command: {}", io.to_string()),
            Self::MalformedOutput { cause, result } => write!(
                f,
                "output of lvm command is malformed: {} -> {}",
                cause, result
            ),
            Self::NotFound { resource } => write!(f, "requested resource not found: {}", resource),
        }
    }
}
