use std::fmt;

pub enum RisklyError {
    DisallowedAsset(String),
    TradeTooLarge(String),
    ExceedsMaxPosition(String),
    ExceedsDailyVolume(String),
    ExceedsMaxAllocation(String),
}

impl fmt::Display for RisklyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RisklyError::DisallowedAsset(asset) => {
                write!(f, "Asset '{asset}' is not allowed for trading.")
            }
            RisklyError::TradeTooLarge(msg) => write!(f, "Trade too large: {msg}"),
            RisklyError::ExceedsMaxPosition(msg) => write!(f, "Exceeds max position: {msg}"),
            RisklyError::ExceedsDailyVolume(msg) => write!(f, "Exceeds daily volume: {msg}"),
            RisklyError::ExceedsMaxAllocation(msg) => write!(f, "Exceeds max allocation: {msg}"),
        }
    }
}
