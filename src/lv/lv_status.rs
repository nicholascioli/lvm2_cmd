use crate::TryFromChar;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LVStatus {
    Closed,
    Open,
    Unknown,
}

impl TryFromChar for LVStatus {
    fn try_from_char(c: char) -> Result<Self, Box<dyn std::error::Error>> {
        let state = match c {
            '-' => LVStatus::Closed,
            'o' => LVStatus::Open,
            'X' => LVStatus::Unknown,

            _ => return Err(format!("invalid flag for LVOpen: {}", c).into()),
        };

        Ok(state)
    }
}
