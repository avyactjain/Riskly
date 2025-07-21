use std::{collections::HashMap, sync::Arc, time::Instant};

use tokio::sync::{mpsc, Mutex};

use crate::{
    config::RisklyConfig,
    riskly::{RisklyState, Trade},
    riskly_error::RisklyError,
};

#[derive(Debug)]
pub struct RisklyService {
    config: Arc<RisklyConfig>,
    pub state: Arc<Mutex<RisklyState>>,
    pub state_rx: Arc<Mutex<Option<mpsc::Receiver<RisklyState>>>>,
    pub state_tx: mpsc::Sender<RisklyState>,
}

impl RisklyService {
    pub fn new(config: RisklyConfig) -> Self {
        let (state_tx, state_rx) = mpsc::channel::<RisklyState>(1);

        let state = RisklyState {
            current_positions: HashMap::new(),
            open_orders: vec![],
            daily_volume: HashMap::new(),
            portfolio_value_usd: 0.0,
            pnl_realized: 0.0,
            pnl_unrealized: 0.0,
        };

        Self {
            config: Arc::new(config),
            state: Arc::new(Mutex::new(state)),
            state_rx: Arc::new(Mutex::new(Some(state_rx))),
            state_tx,
        }
    }

    pub async fn evaluate_trade(&self, trade: Trade) -> Result<(), RisklyError> {
        let start_time = Instant::now();
        let asset = trade.asset.clone();
        let quantity = trade.quantity;

        let state_lock_start = Instant::now();
        let state = self.state.lock().await;
        let state_lock_duration = state_lock_start.elapsed();

        let checks_start = Instant::now();

        // 1. Is the asset allowed?
        let asset_check_start = Instant::now();
        if !self.config.allowed_assets.contains(&asset) {
            return Err(RisklyError::DisallowedAsset(asset.clone()));
        }
        let asset_check_duration = asset_check_start.elapsed();

        // 2. Is the trade size too large?
        let size_check_start = Instant::now();
        if let Some(max_size) = self.config.max_trade_size.get(&asset) {
            if &quantity > max_size {
                return Err(RisklyError::TradeTooLarge(format!(
                    "{quantity} > max {max_size}"
                )));
            }
        }
        let size_check_duration = size_check_start.elapsed();

        // 3. Check projected position after this trade
        let position_check_start = Instant::now();
        let current_position = state.current_positions.get(&asset).cloned().unwrap_or(0.0);

        let new_position = match trade.side {
            0 => current_position + quantity,
            1 => current_position - quantity,
            //Todo : Remove _
            _ => {
                return Err(RisklyError::DisallowedAsset("s".to_owned()));
            }
        };

        if let Some(max_position) = self.config.max_position_per_asset.get(&asset) {
            if new_position.abs() > *max_position {
                return Err(RisklyError::ExceedsMaxPosition(format!(
                    "Projected position {new_position} exceeds max {max_position} for {asset}"
                )));
            }
        }
        let position_check_duration = position_check_start.elapsed();

        // 4. Check daily volume
        let volume_check_start = Instant::now();
        let current_volume = state.daily_volume.get(&asset).cloned().unwrap_or(0.0);

        let projected_volume = current_volume + quantity;

        if let Some(max_volume) = self.config.max_daily_volume.get(&asset) {
            if &projected_volume > max_volume {
                return Err(RisklyError::ExceedsDailyVolume(format!(
                    "Daily volume {projected_volume} > max {max_volume} for {asset}"
                )));
            }
        }
        let volume_check_duration = volume_check_start.elapsed();

        // 5. (Optional) Check portfolio allocation
        let allocation_check_start = Instant::now();
        if let Some(max_alloc_pct) = self.config.max_allocation_per_asset_pct.get(&asset) {
            let portfolio_value = state.portfolio_value_usd;
            let asset_value_after_trade = new_position * trade.price;

            if portfolio_value > 0.0 {
                let allocation_pct = asset_value_after_trade / portfolio_value;
                if allocation_pct > *max_alloc_pct {
                    return Err(RisklyError::ExceedsMaxAllocation(format!(
                        "Allocation {allocation_pct:.2} > max {max_alloc_pct:.2} for {asset}"
                    )));
                }
            }
        }
        let allocation_check_duration = allocation_check_start.elapsed();

        let checks_duration = checks_start.elapsed();
        let total_duration = start_time.elapsed();

        println!(
            "evaluate_trade business logic for {}: total={:?}, state_lock={:?}, checks={:?} (asset={:?}, size={:?}, position={:?}, volume={:?}, allocation={:?})",
            asset, total_duration, state_lock_duration, checks_duration,
            asset_check_duration, size_check_duration, position_check_duration,
            volume_check_duration, allocation_check_duration
        );

        // If all checks pass
        Ok(())
    }
}
