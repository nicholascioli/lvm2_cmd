use crate::TryFromChar;

#[derive(Clone, Debug)]
pub enum VolumeGroupAccessMode {
    SingleNode,
    Shared,

    /// This [VolumeGroup] is shared with other nodes in the cluster.
    ///
    /// If the cluster infrastructure is unavailable on a particular node at a particular time,
    /// you may still be able to use Volume Groups that are not marked as clustered.
    Clustered,
}

impl TryFromChar for VolumeGroupAccessMode {
    fn try_from_char(c: char) -> Result<Self, Box<dyn std::error::Error>> {
        let access = match c {
            '-' => VolumeGroupAccessMode::SingleNode,
            's' => VolumeGroupAccessMode::Shared,
            'c' => VolumeGroupAccessMode::Clustered,

            _ => return Err(format!("invalid access mode flag: {}", c).into()),
        };

        Ok(access)
    }
}
