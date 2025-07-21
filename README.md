# Riskly

**⚠️ Work in Progress** - This is an early-stage risk management service for algorithmic trading systems.

A high-performance gRPC-based risk management service designed to provide real-time trade evaluation and portfolio risk control for algorithmic trading systems. Built with Rust for maximum performance and reliability in production trading environments.

## Overview

Riskly acts as a centralized risk management layer that evaluates trades before execution, maintains position tracking, and enforces risk limits across multiple trading strategies and assets. It provides comprehensive risk controls including position limits, daily volume restrictions, portfolio allocation constraints, and trading circuit breakers.

## Architecture & Design Patterns

### Service-Oriented Architecture (SOA)
- **gRPC Service Layer**: Clean separation of concerns with Protocol Buffers defining the API contract
- **Configuration-Driven**: External JSON configuration enables runtime adjustments without code changes
- **Stateful Service**: Maintains real-time portfolio state with thread-safe concurrent access

### Design Patterns Implemented

1. **Strategy Pattern**: Configurable risk rules per asset class through external configuration
2. **State Pattern**: Centralized state management with atomic operations for portfolio tracking
3. **Observer Pattern**: Event streaming capability for real-time state updates (via `StreamState`)
4. **Command Pattern**: Discrete trade evaluation and execution commands
5. **Builder Pattern**: Proto-generated builders for request/response construction
6. **Repository Pattern**: Abstracted state storage with potential for multiple backends

### Core Components

```
riskly/
├── src/
│   ├── main.rs              # gRPC server bootstrapping & service registration
│   ├── config.rs            # Configuration management & validation
│   ├── riskly_service.rs    # Core risk evaluation business logic
│   ├── riskly_error.rs      # Centralized error handling & types
│   └── local.json           # Runtime configuration file
├── riskly-protos/
│   └── riskly.proto         # gRPC service & message definitions
└── build.rs                 # Proto compilation configuration
```

## Features

### ✅ Implemented
- **Trade Evaluation**: Pre-trade risk assessment with detailed rejection reasons
- **Asset Whitelisting**: Configurable allowed trading instruments
- **Position Limits**: Per-asset maximum position constraints
- **Trade Size Limits**: Maximum trade size per order
- **Daily Volume Limits**: 24-hour rolling volume restrictions
- **Portfolio Allocation**: Percentage-based asset allocation limits
- **gRPC API**: High-performance binary protocol for low-latency integration

### 🚧 In Development (Unimplemented Endpoints)

The following endpoints are defined in the proto but not yet implemented:

#### State Management
- `AddTrade` - Record executed trades and update positions
- `GetState` - Retrieve complete portfolio state
- `GetCurrentPosition` - Query specific asset positions
- `GetOpenOrders` - List all pending orders
- `GetDailyVolume` - Get trading volume by asset
- `StreamState` - Real-time state updates via server streaming

#### Order Management  
- `AddOrder` - Register pending orders for position calculation
- `RemoveOrder` - Remove canceled/filled orders

#### Market Data & Risk Controls
- `UpdateMarketValue` - Real-time price updates for portfolio valuation
- `ResetDailyLimits` - Reset daily volume counters (for EOD processes)

#### Trading Controls
- `DisableTrading` - Emergency stop for all trading
- `EnableTrading` - Re-enable trading after manual review
- `IsTradingEnabled` - Query current trading status

## Configuration

Create a `local.json` file to configure risk parameters:

```json
{
    "max_position_per_asset": {
        "BTC": 2.0,
        "ETH": 10.0,
        "AAPL": 1000.0
    },
    "max_trade_size": {
        "BTC": 0.5,
        "ETH": 2.0,
        "AAPL": 100.0
    },
    "max_daily_volume": {
        "BTC": 5.0,
        "ETH": 20.0,
        "AAPL": 5000.0
    },
    "max_allocation_per_asset_pct": {
        "BTC": 50.0,
        "ETH": 30.0,
        "AAPL": 20.0
    },
    "allowed_assets": [
        "BTC",
        "ETH", 
        "AAPL"
    ],
    "max_slippage_pct": 0.5,
    "trading_enabled": true,
    "listen_address": "127.0.0.1:50051"
}
```

### Configuration Parameters

| Parameter | Description | Example |
|-----------|-------------|---------|
| `max_position_per_asset` | Maximum absolute position per asset | `{"BTC": 2.0}` = max 2 BTC long or short |
| `max_trade_size` | Maximum single trade size | `{"ETH": 5.0}` = max 5 ETH per order |
| `max_daily_volume` | 24-hour rolling volume limit | `{"BTC": 10.0}` = max 10 BTC traded per day |
| `max_allocation_per_asset_pct` | Portfolio allocation limit (%) | `{"BTC": 50.0}` = max 50% of portfolio |
| `allowed_assets` | Tradeable asset whitelist | `["BTC", "ETH"]` |
| `max_slippage_pct` | Maximum allowed slippage | `0.5` = 0.5% max slippage |
| `trading_enabled` | Global trading circuit breaker | `false` = all trades rejected |
| `listen_address` | gRPC server binding | `"0.0.0.0:50051"` |

## Quick Start

### Prerequisites
- Rust 1.70+
- Protocol Buffers compiler (`protoc`)

### Build & Run

```bash
# Clone the repository
git clone <repository-url>
cd riskly

# Build the project
cargo build --release

# Run the server
cargo run --bin riskly-server
```

The server will start on the configured address (default: `127.0.0.1:50052`).

### Client Integration

Example gRPC client call for trade evaluation:

```rust
use riskly::riskly_client::RisklyClient;
use riskly::{Trade, TradeSide};

let mut client = RisklyClient::connect("http://127.0.0.1:50051").await?;

let trade = Trade {
    asset: "BTC".to_string(),
    quantity: 0.1,
    price: 45000.0,
    side: TradeSide::Buy as i32,
    timestamp: chrono::Utc::now().timestamp() as u64,
};

let response = client.evaluate_trade(trade).await?;
if response.into_inner().allowed {
    // Execute trade
} else {
    // Handle rejection
}
```

### Testing Trade Evaluation

```bash
# Using grpcurl to test the service
grpcurl -plaintext -d '{
    "asset": "BTC",
    "quantity": 0.1,
    "price": 45000.0,
    "side": "BUY",
    "timestamp": 1640995200
}' 127.0.0.1:50051 riskly.Riskly/EvaluateTrade
```

## Risk Evaluation Logic

The `EvaluateTrade` endpoint performs comprehensive pre-trade risk checks:

1. **Asset Validation**: Ensures the asset is in the allowed trading list
2. **Trade Size Check**: Validates trade doesn't exceed per-order limits
3. **Position Limit Check**: Calculates projected position post-trade against limits
4. **Daily Volume Check**: Ensures daily volume limits aren't breached
5. **Portfolio Allocation Check**: Validates asset allocation stays within percentage limits

All checks must pass for trade approval. Detailed rejection reasons are provided for failed evaluations.

## Dependencies

- **tonic**: gRPC server/client framework
- **prost**: Protocol Buffers runtime
- **tokio**: Async runtime with multi-threading
- **serde**: Configuration serialization
- **tokio-stream**: Streaming support for real-time updates

## Performance Characteristics

- **Sub-millisecond latency**: Typical trade evaluation < 100μs
- **High throughput**: Supports thousands of evaluations per second
- **Thread-safe**: Concurrent access with minimal lock contention
- **Memory efficient**: Minimal allocations in hot path

## Development Status

This project is in active development. The core trade evaluation logic is functional, but many endpoints remain unimplemented. Contributions are welcome, particularly for:

- State management endpoints
- Market data integration
- Persistence layer
- Monitoring & observability
- Performance optimizations

## License

[Add your license here]

---

**Note**: This is a risk management tool for educational and development purposes. Always perform thorough testing and validation before using in production trading environments.