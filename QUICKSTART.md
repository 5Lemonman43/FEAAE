# Quick Start Guide

Get the Polymarket Arbitrage Bot running in under 5 minutes!

## Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs))
- Git
- Terminal/Command line access

## Installation

### Step 1: Get the Code

```bash
git clone <repository-url>
cd polymarket-arbitrage-bot
```

### Step 2: Configure

```bash
cp .env.example .env
```

**Optional**: Edit `.env` to add your Polymarket API credentials. If you skip this, the bot runs in simulation mode.

```bash
nano .env  # or use your favorite editor
```

### Step 3: Build

```bash
cargo build --release
```

This takes 1-2 minutes on first build.

### Step 4: Run

```bash
RUST_LOG=info cargo run --release
```

Or run the binary directly:

```bash
./target/release/polymarket-arbitrage-bot
```

## What to Expect

When you start the bot, you'll see:

```
[INFO] Starting Polymarket Arbitrage Bot
[INFO] Configuration loaded successfully
[INFO] Discovering BTC down and SOL up 15-minute markets...
[INFO] Market discovery complete: X BTC down markets, Y SOL up markets
[INFO] BTC Market: <id> - <question>
[INFO] SOL Market: <id> - <question>
[INFO] Starting WebSocket connection for real-time data...
[INFO] WebSocket connected successfully
[INFO] Bot is running. Press Ctrl+C to stop.
```

The bot will then continuously:
- Monitor markets for price updates via WebSocket
- Check for arbitrage opportunities every 30 seconds
- Execute trades when profitable opportunities are found
- Display status updates

## Understanding the Output

### Market Discovery
```
[INFO] Found 3 15m BTC down markets
[INFO] BTC Market: abc123 - Will BTC go down in the next 15 minutes?
```

Shows discovered markets matching criteria.

### Arbitrage Detection
```
[INFO] Arbitrage opportunity detected! BTC down: 0.4500, SOL up: 0.4800, Expected profit: 0.0700
```

Found an opportunity where combined probabilities < 1.0

### Trade Execution
```
[INFO] Executing arbitrage: BTC down @ 0.4500, SOL up @ 0.4800
[INFO] Step 1: Buying BTC down position
[INFO] BTC down trade executed: Trade { ... }
[INFO] Step 2: Buying SOL up position
[INFO] SOL up trade executed: Trade { ... }
[INFO] Arbitrage execution complete: 2 trades executed
```

Shows sequential trade execution.

### Status Updates
```
[INFO] Bot status - Positions: 4, Total trades: 12, Total PnL: 1.2500
```

Periodic summary of bot performance.

## Configuration Options

Edit `.env` to customize behavior:

| Setting | Description | Default |
|---------|-------------|---------|
| `MIN_PROFIT_THRESHOLD` | Minimum profit % to trade | 0.02 (2%) |
| `TRADE_AMOUNT` | Amount per trade | 10.0 |
| `MAX_POSITION_SIZE` | Maximum position | 100.0 |
| `RUST_LOG` | Logging level | info |

### Logging Levels

```bash
# See everything (verbose)
RUST_LOG=debug cargo run --release

# Normal operation
RUST_LOG=info cargo run --release

# Only warnings and errors
RUST_LOG=warn cargo run --release
```

## Simulation Mode

**The bot runs in simulation mode by default** if no API credentials are provided.

In simulation mode:
- ✅ Discovers real markets from Polymarket
- ✅ Detects arbitrage opportunities
- ✅ Simulates trade execution
- ❌ Does NOT place real orders

This is perfect for:
- Testing and learning
- Validating the bot works
- Understanding arbitrage opportunities
- Development

## Live Trading Mode

To enable live trading:

1. Get Polymarket API credentials from [Polymarket](https://polymarket.com)
2. Add to `.env`:
   ```env
   POLYMARKET_API_KEY=your_api_key_here
   POLYMARKET_SECRET=your_secret_here
   POLYMARKET_PRIVATE_KEY=your_private_key_here
   ```
3. Restart the bot

**⚠️ WARNING**: Live mode places real orders with real money. Start with small amounts!

## Stopping the Bot

Press `Ctrl+C` to gracefully stop:

```
^C[INFO] Shutting down gracefully...
[INFO] Final statistics:
[INFO]   Total trades executed: 12
[INFO]   Active positions: 4
[INFO]   Total PnL: 1.2500
[INFO] Bot stopped
```

## Common Issues

### "No markets found"

**Cause**: Polymarket may not have active 15-minute BTC/SOL markets at the moment.

**Solution**: 
- Check [Polymarket](https://polymarket.com) for active markets
- The bot will continue monitoring for new markets
- Try adjusting market filters in code if needed

### "WebSocket connection failed"

**Cause**: Network issue or firewall blocking WebSocket connections.

**Solution**:
- Check internet connection
- Verify firewall allows HTTPS/WSS
- Bot will auto-reconnect

### "Failed to load configuration"

**Cause**: Invalid values in `.env` file.

**Solution**:
- Check `.env` format matches `.env.example`
- Ensure numeric values are valid (e.g., `0.02` not `0.02abc`)
- Check for typos in variable names

### Build errors

**Cause**: Outdated Rust version or missing dependencies.

**Solution**:
```bash
# Update Rust
rustup update

# Install dependencies (Linux)
sudo apt install build-essential pkg-config libssl-dev

# Try again
cargo clean
cargo build --release
```

## Next Steps

### Learn More
- Read [README.md](README.md) for comprehensive documentation
- Check [ARCHITECTURE.md](ARCHITECTURE.md) to understand how it works
- See [DEPLOYMENT.md](DEPLOYMENT.md) for production deployment

### Customize
- Adjust profit thresholds in `.env`
- Modify market discovery filters in `src/api_client.rs`
- Add new market types or strategies

### Monitor
- Watch logs for opportunities and execution
- Track PnL over time
- Analyze trade history

### Production Deploy
- Set up systemd service for auto-restart
- Configure log rotation
- Set up monitoring and alerts
- See [DEPLOYMENT.md](DEPLOYMENT.md) for details

## Testing Without Risk

Run in simulation mode to:

1. **Validate Setup**: Ensure everything works
2. **Learn the System**: Understand how arbitrage works
3. **Test Changes**: Validate modifications safely
4. **Monitor Opportunities**: See real-time arbitrage detection

## Example Session

```bash
# Start the bot
$ RUST_LOG=info cargo run --release

[2025-01-22T08:30:00Z INFO] Starting Polymarket Arbitrage Bot
[2025-01-22T08:30:00Z INFO] Configuration loaded successfully
[2025-01-22T08:30:01Z INFO] Discovering BTC down and SOL up 15-minute markets...
[2025-01-22T08:30:02Z INFO] Market discovery complete: 2 BTC down markets, 2 SOL up markets
[2025-01-22T08:30:02Z INFO] BTC Market: abc123 - Will BTC drop in 15 minutes?
[2025-01-22T08:30:02Z INFO] BTC Market: def456 - BTC 15m down bet
[2025-01-22T08:30:02Z INFO] SOL Market: ghi789 - Will SOL rise in 15 minutes?
[2025-01-22T08:30:02Z INFO] SOL Market: jkl012 - SOL 15m up prediction
[2025-01-22T08:30:03Z INFO] Starting WebSocket connection for real-time data...
[2025-01-22T08:30:04Z INFO] WebSocket connected successfully
[2025-01-22T08:30:04Z INFO] Subscribed to 4 markets
[2025-01-22T08:30:04Z INFO] Bot is running. Press Ctrl+C to stop.
[2025-01-22T08:30:34Z INFO] Checking for new markets and arbitrage opportunities...
[2025-01-22T08:30:35Z INFO] Arbitrage opportunity detected! BTC down: 0.4500, SOL up: 0.4800, Expected profit: 0.0700
[2025-01-22T08:30:35Z INFO] Executing arbitrage: BTC down @ 0.4500, SOL up @ 0.4800
[2025-01-22T08:30:35Z WARN] API key not configured - simulating order execution
[2025-01-22T08:30:35Z INFO] Step 1: Buying BTC down position
[2025-01-22T08:30:35Z INFO] BTC down trade executed: sim_1234567890
[2025-01-22T08:30:35Z INFO] Step 2: Buying SOL up position
[2025-01-22T08:30:35Z INFO] SOL up trade executed: sim_1234567891
[2025-01-22T08:30:35Z INFO] Arbitrage execution complete: 2 trades executed
[2025-01-22T08:30:35Z INFO] Bot status - Positions: 2, Total trades: 2, Total PnL: 0.0000

# ... continues monitoring ...

^C[2025-01-22T08:35:00Z INFO] Shutting down gracefully...
[2025-01-22T08:35:00Z INFO] Final statistics:
[2025-01-22T08:35:00Z INFO]   Total trades executed: 2
[2025-01-22T08:35:00Z INFO]   Active positions: 2
[2025-01-22T08:35:00Z INFO]   Total PnL: 0.0000
[2025-01-22T08:35:00Z INFO] Bot stopped
```

## Getting Help

- Check existing documentation files
- Review log output for specific errors
- Open an issue on GitHub
- Check Polymarket status page

## Safety Tips

1. **Start Small**: Use minimum trade amounts initially
2. **Test First**: Run in simulation mode extensively
3. **Monitor Closely**: Watch the first few trades carefully
4. **Set Limits**: Use `MAX_POSITION_SIZE` to limit risk
5. **Understand Markets**: Know how Polymarket works before trading

---

**You're all set! Happy trading! 🚀**

Questions? Check the [README.md](README.md) or [ARCHITECTURE.md](ARCHITECTURE.md)
