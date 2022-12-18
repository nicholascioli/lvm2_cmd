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

use crate::{
    error::LVMError, run_cmd, ResourceCapacity, ResourceName, ResourceSelector, ResourceUUID,
};

#[derive(Clone, Debug, Deserialize)]
pub struct LogicalVolume {
    #[serde(rename = "lv_name")]
    pub name: ResourceName,

    #[serde(rename = "lv_size")]
    pub capacity_bytes: ResourceCapacity,

    // TODO: Destruct from actual volume group
    #[serde(rename = "vg_name")]
    pub volume_group_name: ResourceName,

    #[serde(rename = "lv_attr", deserialize_with = "deserialize_lv_attrs")]
    pub attributes: LogicalVolumeAttributes,

    #[serde(rename = "lv_path")]
    pub path: PathBuf,

    #[serde(rename = "lv_uuid")]
    pub uuid: ResourceUUID,
}

impl LogicalVolume {
    /// Create a [LogicalVolume]
    pub fn create(
        volume_group: &ResourceName,
        opts: LVCreateOptions,
    ) -> Result<LogicalVolume, LVMError> {
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
        args.push(volume_group.to_string());

        // Create the volume (has no output)
        run_cmd::<String>("lvcreate", &args, None::<&str>)
            .and_then(|_| Self::from_id(&volume_group, &opts.name))
    }

    /// Get a [LogicalVolume] from its ID pair (volume group / name)
    pub fn from_id(
        volume_group: &ResourceName,
        name: &ResourceName,
    ) -> Result<LogicalVolume, LVMError> {
        let id = format!("{}/{}", volume_group, name);
        let args = vec![
            "--nolocking",
            "--options",
            "+lv_all",
            "--units",
            "b",
            "--nosuffix",
            &id,
        ];

        run_cmd("lvs", &args, Some("lv"))
            .and_then(|mut lvs| lvs.pop().ok_or(LVMError::NotFound { resource: id }))
    }

    /// List all [LogicalVolume]s available on this system
    pub fn list() -> Result<Vec<LogicalVolume>, LVMError> {
        LogicalVolume::list_for_vg(&ResourceName::empty())
    }

    /// List all [LogicalVolume]s available on this system for a specific volume group
    pub fn list_for_vg(volume_group: &ResourceName) -> Result<Vec<LogicalVolume>, LVMError> {
        let args = vec![
            "--nolocking",
            "--options",
            "+lv_all",
            "--units",
            "b",
            "--nosuffix",
            // Deterministically return sorted by `vg_name`, then `lv_name`
            "--sort",
            "vg_name,lv_name",
            &volume_group,
        ];

        run_cmd("lvs", &args, Some("lv"))
    }

    /// Delete the [LogicalVolume], deactivating it if needed.
    ///
    /// Warning: This _will_ fail if the [LogicalVolume] is currently mounted and
    /// in use.
    pub fn delete(self) -> Result<(), LVMError> {
        run_cmd::<String>("lvremove", &["--force", &self.id()], None::<&str>)?;

        Ok(())
    }

    pub fn id(&self) -> String {
        format!("{}/{}", self.volume_group_name, self.name)
    }

    pub fn activate(&mut self) -> Result<(), LVMError> {
        self.set_activated(true)
    }
    pub fn deactivate(&mut self) -> Result<(), LVMError> {
        self.set_activated(false)
    }

    pub fn set_activated(&mut self, should_activate: bool) -> Result<(), LVMError> {
        run_cmd::<String>(
            "lvchange",
            &["--activate", if should_activate { "ay" } else { "n" }],
            None::<&str>,
        )?;

        Ok(())
    }
}

impl ResourceSelector for LogicalVolume {
    fn from_uuid(uuid: &ResourceUUID) -> Result<Self, LVMError>
    where
        Self: Sized + std::fmt::Debug + serde::de::DeserializeOwned,
    {
        let selector = format!("uuid={}", uuid);
        let args = vec![
            "--nolocking",
            "--options",
            "+lv_all",
            "--units",
            "b",
            "--nosuffix",
            "--select",
            &selector,
        ];

        run_cmd("lvs", &args, Some("lv")).and_then(|mut lvs| {
            lvs.pop().ok_or(LVMError::NotFound {
                resource: uuid.to_string(),
            })
        })
    }
}

#[derive(Clone, Debug)]
pub struct LVCreateOptions {
    /// Whether the logical volume should become active after creation
    pub activate: bool,

    /// The capacity, in bytes, required
    pub capacity_bytes: ResourceCapacity,

    /// Name for the new volume
    pub name: ResourceName,

    /// Optional extra tags to append
    pub tags: Vec<String>,
}
