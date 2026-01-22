# Polymarket Arbitrage Bot

A high-performance Rust-based arbitrage bot for Polymarket that discovers and trades on 15-minute crypto markets, specifically targeting BTC down and SOL up positions.

## Features

- **Automatic Market Discovery**: Continuously discovers active 15-minute BTC down and SOL up markets
- **Real-time WebSocket Data**: Live price updates and order book data via WebSocket streams
- **Arbitrage Detection**: Identifies profitable arbitrage opportunities between correlated markets
- **Sequential Trade Execution**: Executes trades in optimal order (BTC down first, then SOL up)
- **State Management**: Tracks positions, PnL, and trade history
- **Robust Error Handling**: Automatic WebSocket reconnection with exponential backoff
- **Graceful Shutdown**: Proper cleanup and final statistics on exit

## Architecture

The bot is structured into modular components:

- **api_client.rs**: Polymarket REST API integration for market data and order placement
- **websocket.rs**: WebSocket client with automatic reconnection logic
- **arbitrage.rs**: Core arbitrage detection and execution engine
- **types.rs**: Data structures for markets, trades, positions, and state
- **config.rs**: Configuration management via environment variables
- **error.rs**: Custom error types for comprehensive error handling
- **main.rs**: Bot orchestration and lifecycle management

## Prerequisites

- Rust 1.70 or higher
- Polymarket API credentials (optional - bot can run in simulation mode)

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd polymarket-arbitrage-bot
```

2. Copy the example environment file:
```bash
cp .env.example .env
```

3. Configure your environment variables in `.env`:
```env
# Optional: API credentials for live trading
POLYMARKET_API_KEY=your_api_key_here
POLYMARKET_SECRET=your_secret_here
POLYMARKET_PRIVATE_KEY=your_private_key_here

# WebSocket endpoint
POLYMARKET_WS_URL=wss://ws-subscriptions-clob.polymarket.com/ws/market

# Trading parameters
MIN_PROFIT_THRESHOLD=0.02  # Minimum 2% profit to execute arbitrage
MAX_POSITION_SIZE=100.0    # Maximum position size
TRADE_AMOUNT=10.0          # Amount per trade

# Logging level
RUST_LOG=info
```

## Building

Build the project in release mode for optimal performance:

```bash
cargo build --release
```

## Running

### Development Mode

Run with debug logging:
```bash
RUST_LOG=debug cargo run
```

### Production Mode

Run the optimized binary:
```bash
./target/release/polymarket-arbitrage-bot
```

### Simulation Mode

The bot can run without API credentials for testing and development. It will:
- Discover real markets from Polymarket
- Detect arbitrage opportunities
- Simulate trade execution without actual orders

Simply omit the API credentials from `.env` to run in simulation mode.

## How It Works

### 1. Market Discovery

The bot continuously queries the Polymarket API to discover active 15-minute markets:
- **BTC Down Markets**: Markets betting on Bitcoin price decreasing within 15 minutes
- **SOL Up Markets**: Markets betting on Solana price increasing within 15 minutes

Filters applied:
- Active and not closed
- Contains relevant keywords (BTC/Bitcoin, SOL/Solana, 15m/fifteen minute)
- Direction indicators (down/lower/decrease for BTC, up/higher/increase for SOL)

### 2. Real-time Data Streaming

Connects to Polymarket's WebSocket API for:
- Live price updates for discovered markets
- Order book changes
- Trade executions
- Market status updates

Features automatic reconnection with exponential backoff (1s → 2s → 4s → ... → 60s).

### 3. Arbitrage Detection

Monitors market pairs for arbitrage opportunities:

```
Combined Probability = P(BTC down) + P(SOL up)
```

If `Combined Probability < 1.0`, there's a potential arbitrage opportunity.

**Expected Profit** = `1.0 - Combined Probability`

Example:
- BTC down market: 0.45 (45% probability)
- SOL up market: 0.48 (48% probability)
- Combined: 0.93
- **Expected profit: 0.07 (7%)**

### 4. Trade Execution

When an opportunity is detected above the `MIN_PROFIT_THRESHOLD`:

1. **Buy BTC down position** at current market price
2. **Wait for execution confirmation**
3. **Buy SOL up position** at current market price
4. **Update positions and PnL**

Sequential execution ensures proper risk management.

### 5. Position Management

The bot tracks:
- Current positions and average entry prices
- Unrealized PnL based on current market prices
- Trade history with timestamps
- Total portfolio PnL

## Configuration Options

| Variable | Description | Default |
|----------|-------------|---------|
| `POLYMARKET_API_KEY` | API key for authentication | None (simulation mode) |
| `POLYMARKET_WS_URL` | WebSocket endpoint URL | Polymarket production WS |
| `MIN_PROFIT_THRESHOLD` | Minimum profit % to execute | 0.02 (2%) |
| `MAX_POSITION_SIZE` | Maximum position size | 100.0 |
| `TRADE_AMOUNT` | Amount per trade | 10.0 |
| `RUST_LOG` | Logging level | info |

## Monitoring

The bot logs key events:

```
[INFO] Starting Polymarket Arbitrage Bot
[INFO] Market discovery complete: 3 BTC down markets, 2 SOL up markets
[INFO] WebSocket connected successfully
[INFO] Arbitrage opportunity detected! BTC down: 0.4500, SOL up: 0.4800, Expected profit: 0.0700
[INFO] Executing arbitrage: BTC down @ 0.4500, SOL up @ 0.4800
[INFO] Step 1: Buying BTC down position
[INFO] BTC down trade executed
[INFO] Step 2: Buying SOL up position
[INFO] SOL up trade executed
[INFO] Bot status - Positions: 4, Total trades: 12, Total PnL: 1.2500
```

## Error Handling

The bot handles various error scenarios:

- **WebSocket disconnection**: Automatic reconnection with exponential backoff
- **API rate limits**: Graceful handling and retry logic
- **Order failures**: Logs errors and continues monitoring
- **Network issues**: Retry logic with configurable timeouts
- **Invalid data**: Robust parsing with fallback values

## Safety Features

- **Simulation mode**: Test without real money
- **Position limits**: Configurable maximum position sizes
- **Profit thresholds**: Only execute when profit exceeds minimum
- **Sequential execution**: Ensures proper risk management
- **Graceful shutdown**: Clean exit with final statistics

## Graceful Shutdown

Press `Ctrl+C` to stop the bot:

```
[INFO] Shutting down gracefully...
[INFO] Final statistics:
[INFO]   Total trades executed: 12
[INFO]   Active positions: 4
[INFO]   Total PnL: 1.2500
[INFO] Bot stopped
```

## Development

### Running Tests

```bash
cargo test
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## Troubleshooting

### "No markets found"

- Check that Polymarket has active 15-minute BTC/SOL markets
- Verify API endpoint is accessible
- Check network connectivity

### "WebSocket connection failed"

- Verify `POLYMARKET_WS_URL` is correct
- Check firewall settings
- Ensure internet connectivity
- Bot will retry automatically

### "Failed to place order"

- Verify API credentials are correct
- Check account balance
- Verify market is still active
- Check API rate limits

## Performance

Optimized for low latency:
- Async/await for concurrent operations
- Efficient WebSocket handling
- Minimal memory footprint
- Fast arbitrage detection

## Security

- API credentials stored in `.env` (not committed to git)
- Secure WebSocket connections (WSS)
- No sensitive data in logs
- Environment-based configuration

## Limitations

- Requires active 15-minute markets on Polymarket
- Subject to API rate limits
- Network latency affects arbitrage execution speed
- Market liquidity may impact fill prices

## Future Enhancements

- [ ] Multi-market arbitrage beyond BTC/SOL
- [ ] Advanced order types (limit orders, stop-loss)
- [ ] Machine learning for opportunity prediction
- [ ] Historical data analysis and backtesting
- [ ] Web dashboard for monitoring
- [ ] Alert notifications (Telegram, Discord, email)
- [ ] Multi-exchange arbitrage
- [ ] Gas optimization for on-chain execution

## License

MIT License - see LICENSE file for details

## Disclaimer

This bot is for educational purposes. Cryptocurrency trading carries significant risk. Use at your own risk. The authors are not responsible for any financial losses incurred.

## Contributing

Contributions welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

## Support

For issues and questions:
- Open an issue on GitHub
- Check existing issues for solutions
- Review the troubleshooting section

---

**Happy Trading! 🚀**
