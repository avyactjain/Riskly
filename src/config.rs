use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RisklyConfig {
    /// Max allowed position per asset (absolute units, e.g., 1.5 BTC)
    pub max_position_per_asset: HashMap<String, f64>,

    /// Max trade size per asset (per order)
    pub max_trade_size: HashMap<String, f64>,

    /// Max total traded volume per asset per day
    pub max_daily_volume: HashMap<String, f64>,

    /// Max portfolio allocation per asset (percentage of portfolio value)
    pub max_allocation_per_asset_pct: HashMap<String, f64>,

    /// Whitelisted tradable assets (e.g., ["BTC", "ETH"])
    pub allowed_assets: Vec<String>,

    /// Maximum slippage allowed (in %, e.g., 0.5 means 0.5%)
    pub max_slippage_pct: f64,

    /// Hard stop toggle (e.g., false = trading disabled)
    pub trading_enabled: bool,

    /// Address to listen on (e.g., "127.0.0.1:50051")
    pub listen_address: String,
}

impl RisklyConfig {
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let data = fs::read_to_string(path)?;
        let config: RisklyConfig = serde_json::from_str(&data)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_config() {
        let json = r#"
        {
            "max_position_per_asset": {"BTC": 2.0, "ETH": 10.0},
            "max_trade_size": {"BTC": 0.5, "ETH": 2.0},
            "max_daily_volume": {"BTC": 5.0, "ETH": 20.0},
            "max_allocation_per_asset_pct": {"BTC": 50.0, "ETH": 30.0},
            "allowed_assets": ["BTC", "ETH"],
            "max_slippage_pct": 0.5,
            "trading_enabled": true,
            "listen_address": "127.0.0.1:50051"
        }
        "#;
        let config: RisklyConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.max_position_per_asset["BTC"], 2.0);
        assert_eq!(config.allowed_assets, vec!["BTC", "ETH"]);
        assert!(config.trading_enabled);
        assert_eq!(config.listen_address, "127.0.0.1:50051");
    }
}
