# Architecture Overview

## System Design

The Polymarket Arbitrage Bot is built with a modular architecture that separates concerns and enables maintainability and extensibility.

```
┌─────────────────────────────────────────────────────────┐
│                       Main Bot                          │
│                  (orchestration layer)                  │
└───────────┬─────────────────────────────────┬───────────┘
            │                                 │
            ▼                                 ▼
┌───────────────────────┐         ┌──────────────────────┐
│  Market Discovery     │         │   WebSocket Client   │
│  - REST API calls     │         │  - Real-time data    │
│  - Filter markets     │         │  - Auto-reconnect    │
│  - BTC/SOL detection  │         │  - Message parsing   │
└───────────┬───────────┘         └──────────┬───────────┘
            │                                 │
            │                                 │
            ▼                                 ▼
┌──────────────────────────────────────────────────────────┐
│                      Bot State                           │
│  - Current positions                                     │
│  - Active markets                                        │
│  - Order books                                           │
│  - Trade history                                         │
└──────────────────┬───────────────────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────────────────┐
│               Arbitrage Engine                           │
│  - Opportunity detection                                 │
│  - Profit calculation                                    │
│  - Sequential trade execution                            │
└──────────────────┬───────────────────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────────────────┐
│             Trade Execution API                          │
│  - Order placement                                       │
│  - Status tracking                                       │
│  - Error handling                                        │
└──────────────────────────────────────────────────────────┘
```

## Module Breakdown

### 1. Main (`main.rs`)
**Responsibility**: Bot orchestration and lifecycle management

- Initializes all components
- Manages async tasks (WebSocket, market discovery, arbitrage)
- Handles graceful shutdown
- Coordinates between modules

**Key Operations**:
- Load configuration from environment
- Discover initial markets
- Start WebSocket connection
- Launch monitoring loops
- Handle shutdown signals

### 2. API Client (`api_client.rs`)
**Responsibility**: HTTP communication with Polymarket APIs

**Key Functions**:
- `discover_markets()`: Fetch all available markets
- `discover_15m_btc_down_markets()`: Filter for BTC down 15m markets
- `discover_15m_sol_up_markets()`: Filter for SOL up 15m markets
- `get_order_book()`: Fetch current order book for a market
- `place_order()`: Execute buy/sell orders
- `get_market_price()`: Get current market price

**Features**:
- Automatic API fallback (simulation mode without credentials)
- Intelligent market filtering
- Error handling with retry logic

### 3. WebSocket Client (`websocket.rs`)
**Responsibility**: Real-time data streaming

**Key Functions**:
- `connect_and_subscribe()`: Establish WebSocket connection
- `establish_connection()`: Internal connection handler
- Automatic reconnection with exponential backoff

**Message Types Handled**:
- Price updates
- Order book changes
- Trade executions
- Market status updates

**Features**:
- Automatic reconnection (1s → 2s → 4s → ... → 60s backoff)
- Heartbeat mechanism
- Message parsing and routing
- Connection health monitoring

### 4. Arbitrage Engine (`arbitrage.rs`)
**Responsibility**: Detect and execute arbitrage opportunities

**Key Functions**:
- `detect_opportunities()`: Scan for profitable arbitrage
- `check_market_pair()`: Evaluate specific BTC/SOL pair
- `execute_arbitrage()`: Sequential trade execution
- `update_market_prices()`: Refresh price data
- `check_and_execute_arbitrage()`: Main orchestration

**Algorithm**:
```
1. Fetch current prices for BTC down and SOL up
2. Calculate: expected_profit = 1.0 - (P_btc_down + P_sol_up)
3. If expected_profit > threshold:
   a. Buy BTC down position
   b. Wait for confirmation
   c. Buy SOL up position
   d. Update positions and PnL
```

**Risk Management**:
- Sequential execution (avoid partial positions)
- Position size limits
- Profit threshold requirements
- Trade confirmation before next step

### 5. Types (`types.rs`)
**Responsibility**: Data structures and state management

**Key Types**:
- `Market`: Market metadata and pricing
- `OrderBook`: Bid/ask data
- `Trade`: Individual trade records
- `Position`: Current holdings
- `ArbitrageOpportunity`: Detected opportunities
- `BotState`: Global bot state
- `WsMessage`: WebSocket message types

**State Management**:
- Position tracking with average cost basis
- PnL calculations
- Trade history
- Order book caching

### 6. Configuration (`config.rs`)
**Responsibility**: Environment-based configuration

**Settings**:
- API credentials
- WebSocket URLs
- Trading parameters (amounts, thresholds)
- Logging levels

**Features**:
- Environment variable parsing
- Sensible defaults
- Validation

### 7. Error Handling (`error.rs`)
**Responsibility**: Centralized error management

**Error Types**:
- WebSocket errors
- API errors
- Trading errors
- Network errors
- Configuration errors

**Features**:
- Structured error types with `thiserror`
- Error context and chaining
- Conversion from standard errors

## Data Flow

### Market Discovery Flow
```
1. HTTP GET to Polymarket API
2. Filter by keywords (BTC/SOL, 15m, up/down)
3. Filter by status (active, not closed)
4. Store in BotState
5. Refresh every 30 seconds
```

### Real-time Data Flow
```
1. WebSocket connection established
2. Subscribe to discovered markets
3. Receive price/book updates
4. Update BotState
5. Trigger arbitrage checks
```

### Arbitrage Execution Flow
```
1. Periodic check (every 30s)
2. Detect opportunities
3. Validate profit > threshold
4. Execute BTC down trade
5. Wait for confirmation
6. Execute SOL up trade
7. Update positions and PnL
8. Log results
```

## Concurrency Model

The bot uses Tokio's async runtime for concurrent operations:

- **Main Task**: Orchestration and shutdown handling
- **WebSocket Task**: Real-time data streaming
- **Discovery Task**: Periodic market discovery
- **Arbitrage Task**: Periodic opportunity checking

### Shared State

`Arc<Mutex<BotState>>` provides thread-safe shared state:
- Positions
- Markets
- Order books
- Trade history

## Error Recovery

### WebSocket Disconnection
- Automatic reconnection with exponential backoff
- State preservation during reconnection
- No data loss for positions/trades

### API Failures
- Retry logic with backoff
- Graceful degradation (continue with cached data)
- Error logging for monitoring

### Trade Failures
- Partial execution handling
- Position reconciliation
- Skip second leg if first leg fails

## Performance Considerations

### Optimization Strategies
1. **Async I/O**: Non-blocking network operations
2. **Connection Pooling**: Reuse HTTP connections
3. **Message Batching**: Efficient WebSocket handling
4. **Selective Updates**: Only update changed data

### Latency Sources
1. Network latency to Polymarket servers
2. Market data propagation delay
3. Order execution time
4. State synchronization overhead

### Scalability
- Can monitor hundreds of markets
- Handle high-frequency price updates
- Process multiple concurrent arbitrage opportunities
- Configurable rate limiting

## Security Considerations

1. **Credential Management**
   - API keys stored in environment variables
   - Never logged or exposed
   - Optional (can run without)

2. **Network Security**
   - HTTPS for REST APIs
   - WSS for WebSocket connections
   - TLS certificate validation

3. **Trade Safety**
   - Position limits
   - Profit thresholds
   - Sequential execution to avoid partial risk

## Extensibility Points

### Adding New Market Types
1. Create new discovery function in `api_client.rs`
2. Add market type to `BotState`
3. Update arbitrage logic in `arbitrage.rs`

### Adding New Data Sources
1. Implement new WebSocket client
2. Add message types to `types.rs`
3. Update state management

### Adding New Strategies
1. Create new engine in separate module
2. Implement opportunity detection
3. Add execution logic
4. Integrate with main bot

### Adding Monitoring/Alerts
1. Add notification module
2. Hook into trade execution
3. Monitor state changes
4. Send alerts on conditions

## Testing Strategy

### Unit Tests
- Individual function validation
- Edge case handling
- Error scenarios

### Integration Tests
- API client with mock server
- WebSocket with test server
- State management workflows

### End-to-End Tests
- Full bot lifecycle
- Simulated arbitrage scenarios
- Error recovery flows

## Future Enhancements

1. **Multi-Market Arbitrage**: Beyond BTC/SOL pairs
2. **Advanced Order Types**: Limit orders, stop-loss
3. **Machine Learning**: Opportunity prediction
4. **Historical Analysis**: Backtesting framework
5. **Web Dashboard**: Real-time monitoring UI
6. **Alert System**: Notifications for opportunities
7. **Risk Management**: Portfolio-level controls
8. **Performance Optimization**: Lower latency execution
