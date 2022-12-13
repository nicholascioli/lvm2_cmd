use crate::TryFromChar;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LVState {
    Unknown,
    Inactive,
    Active,
    Historical,
    Suspended,
    InvalidSnapshot { suspended: bool },
    SnapshotMergeFailed { suspended: bool },
    DevicePresentWithoutTables,
    DevicePresentWithInactiveTables,
    ThinPoolCheckNeeded { suspended: bool },
}

impl TryFromChar for LVState {
    fn try_from_char(c: char) -> Result<Self, Box<dyn std::error::Error>> {
        let state = match c {
            '-' => LVState::Inactive,
            'a' => LVState::Active,
            'h' => LVState::Historical,
            's' => LVState::Suspended,
            'I' => LVState::InvalidSnapshot { suspended: false },
            'S' => LVState::InvalidSnapshot { suspended: true },
            'm' => LVState::SnapshotMergeFailed { suspended: false },
            'M' => LVState::SnapshotMergeFailed { suspended: true },
            'd' => LVState::DevicePresentWithoutTables,
            'i' => LVState::DevicePresentWithInactiveTables,
            'c' => LVState::ThinPoolCheckNeeded { suspended: false },
            'C' => LVState::ThinPoolCheckNeeded { suspended: true },
            'X' => LVState::Unknown,

            _ => return Err(format!("invalid flag for LVState: {}", c).into()),
        };

        Ok(state)
    }
}
