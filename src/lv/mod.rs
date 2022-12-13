mod lv_attributes;
mod lv_state;
mod lv_status;
mod lv_volume_type;

pub use lv_attributes::*;
pub use lv_state::*;
pub use lv_status::*;
pub use lv_volume_type::*;

use std::path::PathBuf;

use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;

use crate::{nearest_size_multiple, run_cmd, validate_name};

#[derive(Clone, Debug, Deserialize)]
pub struct LogicalVolume {
    #[serde(rename = "lv_name")]
    pub name: String,

    #[serde(
        rename = "lv_size",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub capacity_bytes: u64,

    // TODO: Destruct from actual volume group
    #[serde(rename = "vg_name")]
    pub volume_group_name: String,

    #[serde(rename = "lv_attr", deserialize_with = "deserialize_lv_attrs")]
    pub attributes: LogicalVolumeAttributes,

    #[serde(rename = "lv_path")]
    pub path: PathBuf,

    #[serde(rename = "lv_uuid")]
    pub uuid: String,
}

impl LogicalVolume {
    /// Create a [LogicalVolume]
    pub fn create(
        volume_group: impl AsRef<str>,
        opts: LVCreateOptions,
    ) -> Result<LogicalVolume, Box<dyn std::error::Error>> {
        // Make sure that the size is a multiple of 512, as per LVM2
        let nearest = nearest_size_multiple(opts.capacity_bytes);
        if opts.capacity_bytes != nearest {
            return Err(format!(
                "capacity must be a multiple of 512. got {}, but next closest multiple is {}",
                opts.capacity_bytes, nearest
            )
            .into());
        }

        // Make sure that the supplied name is valid
        validate_name(&opts.name)?;

        let tags: Vec<String> = opts
            .tags
            .into_iter()
            .flat_map(|t: String| ["--addtag".into(), t])
            .collect();

        let byte_str = format!("{}B", opts.capacity_bytes);

        let should_activate = if opts.activate { "ay" } else { "an" };
        let args = vec![
            "--activate",
            should_activate,
            "--name",
            &opts.name,
            "--size",
            &byte_str,
        ];

        let mut args = [args.into_iter().map(|s| s.into()).collect(), tags].concat();
        args.push(volume_group.as_ref().into());

        // Create the volume (has no output)
        run_cmd::<String>("lvcreate", &args, None::<&str>)
            .map_err(|err| format!("could not create lv: {}", err.to_string()))?;

        // Return the newly created volume
        Self::get(&volume_group, &opts.name)
            .map_err(|err| format!("could not get recently created lv: {}", err.to_string()).into())
    }

    /// Get a [LogicalVolume] from its ID pair (volume group / name)
    pub fn get(
        volume_group: impl AsRef<str>,
        name: impl AsRef<str>,
    ) -> Result<LogicalVolume, Box<dyn std::error::Error>> {
        // Make sure that the supplied name / volume group is valid
        validate_name(volume_group.as_ref())?;
        validate_name(name.as_ref())?;

        let id = format!("{}/{}", volume_group.as_ref(), name.as_ref());
        let args = vec![
            "--nolocking",
            "--options",
            "+lv_all",
            "--units",
            "b",
            "--nosuffix",
            &id,
        ];

        run_cmd("lvs", &args, Some("lv")).and_then(|mut lvs| {
            lvs.pop()
                .ok_or(format!("could not get specified logical volume: {}", id).into())
        })
    }

    /// List all [LogicalVolume]s available on this system
    pub fn list() -> Result<Vec<LogicalVolume>, Box<dyn std::error::Error>> {
        LogicalVolume::list_for_vg("")
    }

    /// List all [LogicalVolume]s available on this system for a specific volume group
    // TODO: Should this take an option?
    pub fn list_for_vg(
        volume_group: impl AsRef<str>,
    ) -> Result<Vec<LogicalVolume>, Box<dyn std::error::Error>> {
        let mut args = vec![
            "--nolocking",
            "--options",
            "+lv_all",
            "--units",
            "b",
            "--nosuffix",
            // Deterministically return sorted by `vg_name`, then `lv_name`
            "--sort",
            "vg_name,lv_name",
        ];

        // TODO: Should we do this?
        if !volume_group.as_ref().is_empty() {
            // Make sure that the volume group is valid
            validate_name(volume_group.as_ref())?;

            args.push(volume_group.as_ref());
        };

        run_cmd("lvs", &args, Some("lv"))
    }

    /// Delete the [LogicalVolume], deactivating it if needed.
    ///
    /// Warning: This _will_ fail if the [LogicalVolume] is currently mounted and
    /// in use.
    pub fn delete(self) -> Result<(), Box<dyn std::error::Error>> {
        run_cmd::<String>("lvremove", &["--force", &self.id()], None::<&str>)?;

        Ok(())
    }

    pub fn id(&self) -> String {
        format!("{}/{}", self.volume_group_name, self.name)
    }

    pub fn activate(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.set_activated(true)
    }
    pub fn deactivate(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.set_activated(false)
    }

    pub fn set_activated(
        &mut self,
        should_activate: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        run_cmd::<String>(
            "lvchange",
            &["--activate", if should_activate { "ay" } else { "n" }],
            None::<&str>,
        )?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct LVCreateOptions {
    /// Whether the logical volume should become active after creation
    pub activate: bool,

    /// The capacity, in bytes, required
    pub capacity_bytes: usize,

    /// Name for the new volume.
    ///
    /// Make sure that it is a valid LVM name, namely [a-zA-Z0-9_.]
    pub name: String,

    /// Optional extra tags to append
    pub tags: Vec<String>,
}
