use std::time::Instant;
use tonic::transport::Server;
use tonic::{async_trait, Response};

use crate::config::RisklyConfig;
use crate::riskly::riskly_server::{Riskly, RisklyServer};
use crate::riskly::TradeEvaluationResponse;
use crate::riskly_service::RisklyService;

pub mod riskly {
    tonic::include_proto!("riskly");
}

pub mod config;
pub mod riskly_error;
pub mod riskly_service;

#[async_trait]
impl Riskly for RisklyService {
    async fn evaluate_trade(
        &self,
        trade: tonic::Request<riskly::Trade>,
    ) -> Result<tonic::Response<riskly::TradeEvaluationResponse>, tonic::Status> {
        let start_time = Instant::now();
        let trade_inner = trade.into_inner();
        
        let result = match self.evaluate_trade(trade_inner).await {
            Ok(()) => {
                Ok(Response::new(TradeEvaluationResponse {
                    allowed: true,
                    reason: "allowed".to_string(),
                }))
            }
            Err(err) => {
                Ok(Response::new(TradeEvaluationResponse {
                    allowed: false,
                    reason: err.to_string(),
                }))
            }
        };
        
        let duration = start_time.elapsed();
        println!("evaluate_trade endpoint took: {:?}", duration);
        
        result
    }

    async fn add_trade(
        &self,
        _request: tonic::Request<riskly::Trade>,
    ) -> Result<tonic::Response<riskly::Ack>, tonic::Status> {
        unimplemented!()
    }

    async fn add_order(
        &self,
        _request: tonic::Request<riskly::OpenOrder>,
    ) -> Result<tonic::Response<riskly::Ack>, tonic::Status> {
        unimplemented!()
    }

    async fn remove_order(
        &self,
        _request: tonic::Request<riskly::RemoveOrderRequest>,
    ) -> Result<tonic::Response<riskly::Ack>, tonic::Status> {
        unimplemented!()
    }

    async fn get_state(
        &self,
        _request: tonic::Request<riskly::Empty>,
    ) -> Result<tonic::Response<riskly::RisklyState>, tonic::Status> {
        unimplemented!()
    }

    async fn get_current_position(
        &self,
        _request: tonic::Request<riskly::AssetRequest>,
    ) -> Result<tonic::Response<riskly::PositionResponse>, tonic::Status> {
        unimplemented!()
    }

    async fn get_open_orders(
        &self,
        _request: tonic::Request<riskly::Empty>,
    ) -> Result<tonic::Response<riskly::OpenOrdersResponse>, tonic::Status> {
        unimplemented!()
    }

    async fn get_daily_volume(
        &self,
        _request: tonic::Request<riskly::AssetRequest>,
    ) -> Result<tonic::Response<riskly::DailyVolumeResponse>, tonic::Status> {
        unimplemented!()
    }

    type StreamStateStream =
        tokio_stream::wrappers::ReceiverStream<Result<riskly::RisklyState, tonic::Status>>;

    async fn stream_state(
        &self,
        _request: tonic::Request<riskly::Empty>,
    ) -> Result<tonic::Response<Self::StreamStateStream>, tonic::Status> {
        unimplemented!()
    }

    async fn reset_daily_limits(
        &self,
        _request: tonic::Request<riskly::Empty>,
    ) -> Result<tonic::Response<riskly::Ack>, tonic::Status> {
        unimplemented!()
    }

    async fn update_market_value(
        &self,
        _request: tonic::Request<riskly::PriceUpdateRequest>,
    ) -> Result<tonic::Response<riskly::Ack>, tonic::Status> {
        unimplemented!()
    }

    async fn disable_trading(
        &self,
        _request: tonic::Request<riskly::Empty>,
    ) -> Result<tonic::Response<riskly::Ack>, tonic::Status> {
        unimplemented!()
    }

    async fn enable_trading(
        &self,
        _request: tonic::Request<riskly::Empty>,
    ) -> Result<tonic::Response<riskly::Ack>, tonic::Status> {
        unimplemented!()
    }

    async fn is_trading_enabled(
        &self,
        _request: tonic::Request<riskly::Empty>,
    ) -> Result<tonic::Response<riskly::TradingStatusResponse>, tonic::Status> {
        unimplemented!()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RisklyConfig::from_json_file("src/local.json")?;
    let addr = config.listen_address.parse()?;

    let riskly_service = RisklyService::new(config);

    println!("Riskly Server Starting on : {addr}");
    Server::builder()
        .add_service(RisklyServer::new(riskly_service))
        .serve(addr)
        .await?;

    Ok(())
}
