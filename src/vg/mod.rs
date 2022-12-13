mod vg_access_mode;
mod vg_attributes;

pub use vg_access_mode::*;
pub use vg_attributes::*;

use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;

use crate::{
    lv::{LVCreateOptions, LogicalVolume},
    run_cmd, validate_name,
};

#[derive(Clone, Debug, Deserialize)]
pub struct VolumeGroup {
    #[serde(rename = "vg_name")]
    pub name: String,

    #[serde(
        rename = "vg_size",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub capacity_bytes: u64,

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
    ) -> Result<VolumeGroup, Box<dyn std::error::Error>> {
        // Make sure that the volume group is valid
        validate_name(&opts.name)?;

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
        run_cmd::<String>("vgcreate", &args, None::<&str>)
            .map_err(|err| format!("could not create vg: {}", err.to_string()))?;

        // Return the newly created volume
        Self::get(&opts.name)
            .map_err(|err| format!("could not get recently created vg: {}", err.to_string()).into())
    }

    /// Get a specific [VolumeGroup] by its name
    pub fn get(volume_group: impl AsRef<str>) -> Result<VolumeGroup, Box<dyn std::error::Error>> {
        // Make sure that the volume group is valid
        validate_name(volume_group.as_ref())?;

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
            volume_group.as_ref(),
        ];

        run_cmd("vgs", &args, Some("vg")).and_then(|mut lvs| {
            lvs.pop()
                .ok_or("could not get specified volume group".into())
        })
    }

    /// Get all available [VolumeGroup]s on this system
    pub fn list() -> Result<Vec<VolumeGroup>, Box<dyn std::error::Error>> {
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
    pub fn list_lvs(&self) -> Result<Vec<LogicalVolume>, Box<dyn std::error::Error>> {
        LogicalVolume::list_for_vg(&self.name)
    }

    /// Add a [LogicalVolume] to the volume group
    pub fn add_lv(
        &self,
        opts: LVCreateOptions,
    ) -> Result<LogicalVolume, Box<dyn std::error::Error>> {
        LogicalVolume::create(&self.name, opts)
    }

    /// Remove a [LogicalVolume] from the volume group
    pub fn remove_lv(&self, name: impl AsRef<str>) -> Result<(), Box<dyn std::error::Error>> {
        let lv = LogicalVolume::get(&self.name, &name)?;

        lv.delete()
    }
}

pub struct VGCreateOptions {
    /// The name of the [VolumeGroup]
    name: String,

    /// Whether to enable clustered mode
    is_clustered: Option<bool>,

    /// Whether to enforce a maximum amount of allowed [LogicalVolume]s
    max_logical_volumes: Option<u64>,

    /// Whether to enforce a maximum amount of allowed physical volumes
    max_physical_volumes: Option<u64>,
}
