mod vg_access_mode;
mod vg_attributes;

pub use vg_access_mode::*;
pub use vg_attributes::*;

use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;

use crate::{
    error::LVMError,
    lv::{LVCreateOptions, LogicalVolume},
    run_cmd, ResourceCapacity, ResourceName, ResourceSelector, ResourceUUID,
};

#[derive(Clone, Debug, Deserialize)]
pub struct VolumeGroup {
    #[serde(rename = "vg_name")]
    pub name: ResourceName,

    #[serde(rename = "vg_uuid")]
    pub uuid: ResourceUUID,

    #[serde(rename = "vg_size")]
    pub capacity_bytes: ResourceCapacity,

    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub lv_count: usize,

    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub pv_count: usize,

    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub snap_count: usize,

    #[serde(
        rename = "vg_free",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub space_free_bytes: usize,

    #[serde(rename = "vg_attr", deserialize_with = "deserialize_vg_attrs")]
    pub attributes: VolumeGroupAttributes,
}

impl VolumeGroup {
    /// Create a new [VolumeGroup] from a set of physical devices
    pub fn create(
        physical_devices: Vec<String>,
        opts: VGCreateOptions,
    ) -> Result<VolumeGroup, LVMError> {
        let max_logical_volumes = opts.max_logical_volumes.unwrap_or_default().to_string();
        let max_physical_volumes = opts.max_physical_volumes.unwrap_or_default().to_string();
        let mut args = vec![
            "--clustered",
            opts.is_clustered
                .map(|c| if c { "y" } else { "n" })
                .unwrap_or("n"),
            "--maxlogicalvolumes",
            &max_logical_volumes,
            "--maxphysicalvolumes",
            &max_physical_volumes,
            &opts.name,
        ];

        // Add the physical devices last
        args.extend(physical_devices.iter().map(|pv| pv.as_str()));

        // Create the volume group (has no output)
        run_cmd::<String>("vgcreate", &args, None::<&str>)?;

        // Return the newly created volume
        Self::from_id(&opts.name)
    }

    /// Get a specific [VolumeGroup] by its name
    pub fn from_id(volume_group: &ResourceName) -> Result<VolumeGroup, LVMError> {
        let args = vec![
            "--nolocking",
            "--options",
            "+vg_all",
            "--units",
            "b",
            "--nosuffix",
            // Deterministically return sorted by `vg_name`
            "--sort",
            "vg_name",
            &volume_group,
        ];

        run_cmd("vgs", &args, Some("vg")).and_then(|mut lvs| {
            lvs.pop().ok_or(LVMError::NotFound {
                resource: volume_group.to_string(),
            })
        })
    }

    /// Get all available [VolumeGroup]s on this system
    pub fn list() -> Result<Vec<VolumeGroup>, LVMError> {
        // Attempt to not modify the system as it is read
        let args = vec![
            "--nolocking",
            "--options",
            "+vg_all",
            "--units",
            "b",
            "--nosuffix",
            // Deterministically return sorted by `vg_name`
            "--sort",
            "vg_name",
        ];

        run_cmd("vgs", &args, Some("vg"))
    }

    /// List all [LogicalVolume]s for this volume group
    pub fn list_lvs(&self) -> Result<Vec<LogicalVolume>, LVMError> {
        LogicalVolume::list_for_vg(&self.name)
    }

    /// Add a [LogicalVolume] to the volume group
    pub fn add_lv(&self, opts: LVCreateOptions) -> Result<LogicalVolume, LVMError> {
        LogicalVolume::create(&self.name, opts)
    }

    /// Remove a [LogicalVolume] from the volume group
    pub fn remove_lv(&self, name: &ResourceName) -> Result<(), LVMError> {
        let lv = LogicalVolume::from_id(&self.name, &name)?;

        lv.delete()
    }
}

impl ResourceSelector for VolumeGroup {
    fn from_uuid(uuid: &ResourceUUID) -> Result<Self, LVMError>
    where
        Self: Sized + std::fmt::Debug + serde::de::DeserializeOwned,
    {
        let selector = format!("uuid={}", uuid);
        let args = vec![
            "--nolocking",
            "--options",
            "+vg_all",
            "--units",
            "b",
            "--nosuffix",
            "--select",
            &selector,
        ];

        run_cmd("vgs", &args, Some("vg")).and_then(|mut lvs| {
            lvs.pop().ok_or(LVMError::NotFound {
                resource: uuid.to_string(),
            })
        })
    }
}

pub struct VGCreateOptions {
    /// The name of the [VolumeGroup]
    name: ResourceName,

    /// Whether to enable clustered mode
    is_clustered: Option<bool>,

    /// Whether to enforce a maximum amount of allowed [LogicalVolume]s
    max_logical_volumes: Option<usize>,

    /// Whether to enforce a maximum amount of allowed physical volumes
    max_physical_volumes: Option<usize>,
}
