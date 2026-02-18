# Grid Bot Example

The Grid Bot is a Rust-based automated trading bot that implements a grid trading strategy on EVM-compatible blockchains. Grid trading works by placing buy and sell orders at regular price intervals (grid levels) to capture profits from price fluctuations.

You can see the example code [here](https://github.com/Cavenfish/bonanca/tree/main/examples/crates/grid-bot).

## Features

- Automated grid trading on EVM chains
- Configurable grid parameters (size, max orders, price delta)
- Trading history and profit tracking
- Real-time order management
- Balance monitoring

### Building the Project

```bash
cd examples/crates/grid-bot
cargo build --release
```

## Basic Usage

This example project build a CLI which includes help docs that cover basic usage.

```bash
grid-bot --help
```

## Configuration

Create a `grid_bot.json` configuration file with the following structure:

```json
{
  "chain": "base",
  "rpc_url": "https://base.drpc.org",
  "keyvault": "./keyvault.json",
  "log_file": "./grid_bot_log.json",
  "child": 0,

  "trading_pair": {
    "token_a": {
      "name": "Circle USD",
      "symbol": "USDC",
      "address": "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",
      "decimals": 6
    },
    "token_b": {
      "name": "Ethereum",
      "symbol": "WETH",
      "address": "0x4200000000000000000000000000000000000006",
      "decimals": 18
    },
    "num_grids": 5,
    "upper_limit": 2300.0,
    "lower_limit": 1700.0,
    "buy_amount": 0.1,
    "sell_amount": 0.1
  }
}
```

## Parameters

- **num_grids**: Number of grid levels to place
- **upper_limit**: The highest price at which to place sell orders
- **lower_limit**: The lowest price at which to place buy orders
- **buy_amount**: The amount of token_a to use for each buy level
- **sell_amount**: The amount of token_b to sell at each sell level
