use crate::TryFromChar;

#[derive(Clone, Debug)]
pub enum LVVolumeType {
    Simple,
    Cache,
    Mirrored { initial_sync: bool },
    Origin { merging_snapshot: bool },
    Raid { initial_sync: bool },
    Snapshot { merging: bool },
    PVMove,
    Virtual,
    MirrorOrRaid { out_of_sync: bool },
    MirrorLog,
    UnderConversion,
    ThinVolume,
    ThinPool { data: bool },
    VDOPool { data: bool },

    // TODO: Way better naming
    RaidOrPoolMetadataOrPoolMetadataSpare,
}

impl TryFromChar for LVVolumeType {
    fn try_from_char(c: char) -> Result<Self, Box<dyn std::error::Error>> {
        let volume_type = match c {
            '-' => LVVolumeType::Simple,
            'C' => LVVolumeType::Cache,
            'm' => LVVolumeType::Mirrored {
                initial_sync: false,
            },
            'M' => LVVolumeType::Mirrored { initial_sync: true },
            'o' => LVVolumeType::Origin {
                merging_snapshot: false,
            },
            'O' => LVVolumeType::Origin {
                merging_snapshot: true,
            },
            'r' => LVVolumeType::Raid { initial_sync: true },
            'R' => LVVolumeType::Raid {
                initial_sync: false,
            },
            's' => LVVolumeType::Snapshot { merging: true },
            'S' => LVVolumeType::Snapshot { merging: false },
            'p' => LVVolumeType::PVMove,
            'v' => LVVolumeType::Virtual,
            'i' => LVVolumeType::MirrorOrRaid { out_of_sync: false },
            'I' => LVVolumeType::MirrorOrRaid { out_of_sync: true },
            'l' => LVVolumeType::MirrorLog,
            'c' => LVVolumeType::UnderConversion,
            'V' => LVVolumeType::ThinVolume,
            't' => LVVolumeType::ThinPool { data: false },
            'T' => LVVolumeType::ThinPool { data: true },
            'd' => LVVolumeType::VDOPool { data: false },
            'D' => LVVolumeType::VDOPool { data: true },
            'e' => LVVolumeType::RaidOrPoolMetadataOrPoolMetadataSpare,

            _ => return Err(format!("invalid flag for VolumeType: {}", c).into()),
        };

        Ok(volume_type)
    }
}
