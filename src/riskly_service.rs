use std::{collections::HashMap, sync::Arc, time::Instant};

use tokio::sync::{watch, Mutex};

use crate::{
    config::RisklyConfig,
    riskly::{RisklyState, Trade},
    riskly_error::RisklyError,
};

#[derive(Debug)]
pub struct RisklyService {
    config: Arc<RisklyConfig>,
    pub state: Arc<Mutex<RisklyState>>,
    pub state_rx: watch::Receiver<Result<RisklyState, tonic::Status>>,
    pub state_tx: watch::Sender<Result<RisklyState, tonic::Status>>,
}

impl RisklyService {
    pub fn new(config: RisklyConfig) -> Self {
        let state = RisklyState {
            current_positions: HashMap::new(),
            open_orders: vec![],
            daily_volume: HashMap::new(),
        };

        let (state_tx, state_rx) = watch::channel(Ok(state.clone()));

        Self {
            config: Arc::new(config),
            state: Arc::new(Mutex::new(state)),
            state_rx,
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
            _ => {
                return Err(RisklyError::InvalidTradeSide(format!(
                    "Unknown trade side: {}",
                    trade.side
                )));
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

        let checks_duration = checks_start.elapsed();
        let total_duration = start_time.elapsed();

        println!(
            "evaluate_trade business logic for {asset}: total={total_duration:?}, state_lock={state_lock_duration:?}, checks={checks_duration:?} (asset={asset_check_duration:?}, size={size_check_duration:?}, position={position_check_duration:?},  volume={volume_check_duration:?})",
        );

        // If all checks pass
        Ok(())
    }

    pub async fn add_trade(&self, trade: Trade) -> Result<(), RisklyError> {
        // need to first evaluate the trade.
        self.evaluate_trade(trade.clone()).await?;

        // Need to update state in this function.

        //1. Acquire lock to the state.
        let mut current_state = self.state.lock().await;

        let current_position = current_state.current_positions.get_mut(&trade.asset);

        if let Some(quantity) = current_position {
            // position of that asset exists in the state
            match trade.side {
                0 => *quantity += trade.quantity,
                1 => *quantity -= trade.quantity,
                _ => {
                    return Err(RisklyError::InvalidTradeSide(format!(
                        "Unknown trade side: {}",
                        trade.side
                    )));
                }
            }
        } else {
            // position of that asset doesn't exist in the state

            match trade.side {
                0 => current_state
                    .current_positions
                    .insert(trade.asset.clone(), trade.quantity),
                1 => current_state
                    .current_positions
                    .insert(trade.asset.clone(), -trade.quantity),
                _ => {
                    return Err(RisklyError::InvalidTradeSide(format!(
                        "Unknown trade side: {}",
                        trade.side
                    )));
                }
            };
        }

        // Update daily volume
        let current_volume = current_state
            .daily_volume
            .get(&trade.asset)
            .cloned()
            .unwrap_or(0.0);
        current_state
            .daily_volume
            .insert(trade.asset.clone(), current_volume + trade.quantity);

        if let Err(error) = self.state_tx.send(Ok(current_state.clone())) {
            println!("Channel send error {error:?}");
        };

        Ok(())
    }
}
