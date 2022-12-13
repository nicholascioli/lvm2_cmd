pub mod lv;
pub mod vg;

use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Command;

use regex::Regex;
use serde::de::DeserializeOwned;

lazy_static::lazy_static! {
    // TODO: Allow an env var to specify this
    static ref LVM_COMMAND: PathBuf = which::which("lvm").expect("could not locate lvm binary!");

    static ref NAME_REGEX: Regex = Regex::new("^[a-zA-Z0-9+_.\\-]+$").expect("could not compile name enforcement regex!");
}

/// Finds the next nearest valid size multiple for an LVM construct
pub fn nearest_size_multiple(x: usize) -> usize {
    ((x - 1) | 511) + 1
}

/// Validates a name, returnning an error if it is an invalid name for LVM
pub fn validate_name(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !NAME_REGEX.is_match(&name) {
        return Err(format!("name must be valid for LVM2 ([a-zA-Z0-9_.+-]): {}", name).into());
    }

    Ok(())
}

/// Trait that represents a struct that can be deserialized from a single character
pub trait TryFromChar {
    fn try_from_char(c: char) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Permissions {
    Writeable,
    ReadOnly,

    /// Like read-only, but symbolizes a read-only activation of a writeable volume.
    ReadOnlyActivation,
}

impl TryFromChar for Permissions {
    fn try_from_char(c: char) -> Result<Self, Box<dyn std::error::Error>> {
        let perm = match c {
            'w' => Permissions::Writeable,
            'r' => Permissions::ReadOnly,
            'R' => Permissions::ReadOnlyActivation,
            _ => return Err("invalid flage for Permissions".into()),
        };

        Ok(perm)
    }
}

#[derive(Clone, Debug)]
pub enum AllocationPolicy {
    Anyware { locked: bool },
    Contiguous { locked: bool },
    Inherited { locked: bool },
    Cling { locked: bool },
    Normal { locked: bool },
}

impl TryFromChar for AllocationPolicy {
    fn try_from_char(c: char) -> Result<Self, Box<dyn std::error::Error>> {
        let res = match c {
            'a' => AllocationPolicy::Anyware { locked: false },
            'A' => AllocationPolicy::Anyware { locked: true },
            'c' => AllocationPolicy::Contiguous { locked: false },
            'C' => AllocationPolicy::Contiguous { locked: true },
            'i' => AllocationPolicy::Inherited { locked: false },
            'I' => AllocationPolicy::Inherited { locked: true },
            'l' => AllocationPolicy::Cling { locked: false },
            'L' => AllocationPolicy::Cling { locked: true },
            'n' => AllocationPolicy::Normal { locked: false },
            'N' => AllocationPolicy::Normal { locked: true },
            _ => return Err(format!("invalid flag for AllocationPolicy: {}", c).into()),
        };

        Ok(res)
    }
}

/// Runs a command and then unwraps the results, converting it into the templated output
pub(crate) fn run_cmd<T>(
    cmd: impl AsRef<str>,
    args: &[impl AsRef<OsStr>],
    outer_key: Option<impl AsRef<str>>,
) -> Result<Vec<T>, Box<dyn std::error::Error>>
where
    T: DeserializeOwned + std::fmt::Debug,
{
    const DEFAULT_ARGS: [&str; 2] = ["--reportformat", "json"];
    let mut base = Command::new(&*LVM_COMMAND);

    // Run the command
    let out = base.arg(cmd.as_ref()).args(&DEFAULT_ARGS).args(args);

    #[cfg(feature = "logging")]
    log::info!(
        "Running command {:?} with args {:?}",
        out.get_program(),
        out.get_args()
    );

    let out = out.output()?;
    if !out.status.success() {
        return Err(format!(
            "could not run lvm command: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )
        .into());
    }

    // Unwrap the report
    let wrapped = std::str::from_utf8(&out.stdout)?;

    #[cfg(feature = "logging")]
    log::debug!("Command executed with: {}", wrapped);

    let unwrapped = if let Some(wrapping) = &outer_key {
        let unwrapped = serde_json::from_str::<serde_json::Value>(wrapped)
            .map_err(|e| e.to_string())
            .and_then(|mut v| {
                v.pointer_mut(&format!("/report/0/{}", wrapping.as_ref()))
                    .map(|v| v.take())
                    .ok_or_else(|| {
                        String::from("malformed lvm2 output: wrapping is the wrong format")
                    })
            })
            .and_then(|unwrapped| {
                unwrapped.as_array().map(|v| v.clone()).ok_or_else(|| {
                    String::from("malformed lvm2 output: wrapped value is not an array")
                })
            })?;

        #[cfg(feature = "logging")]
        log::debug!("Got unwrapped output: {:?}", unwrapped);

        unwrapped
    } else {
        vec![wrapped.into()]
    };

    let as_type: Result<Vec<_>, _> = unwrapped
        .into_iter()
        .map(|value| serde_json::from_value(value))
        .collect();

    #[cfg(feature = "logging")]
    log::debug!("Got mapped answer: {:?}", as_type);

    Ok(as_type?)
}
