# Scalper Bot Example

The Scalper Bot is a Python-based trading bot. Scalping involves making many small trades to capture tiny price differences (spreads). This uses the CoW
protocol for limit orders.

You can check out the code [here](https://github.com/Cavenfish/bonanca/blob/main/examples/python/scalper.py).

## Features

- Automated scalping on EVM-compatible blockchains
- Configurable trade parameters and limits
- Detailed trade logging and performance tracking
- Order history tracking (buy/sell averages)
- Profit monitoring and reporting
- Support for any EVM token pair

## Configuration

Create a `scalper.toml` configuration file:

```toml
chain = "Polygon"
rpc_url = "https://polygon-rpc.com"
keyvault = "./keyvault.json"
child = 0
log_file = "./grid_bot_log.json"

[base]
name = "Wrapped Ether"
symbol = "WETH"
address = "0x7ceB23fD6bC0adD59E27f9EA9d0231e0f01cc726"
decimals = 18

[target]
name = "USDC Coin"
symbol = "USDC"
address = "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359"
decimals = 6

[trade_settings]
size = 0.1
max_orders = 5
delta = 0.2
profit = 1.5
expiry = "0-30-300"
```

## Configuration Parameters

### Trade Settings

- **size**: The size of each scalp trade (in base token units)
- **max_orders**: Maximum number of concurrent orders
- **delta**: Price difference between placed buy orders (percentage)
- **profit**: Minimum profit target for sell orders (percentage)
- **expiry**: Order expiration time format "DAYS-HOURS-MINUTES"

## Output Files

- **log_file**: JSON file containing:
  - Active orders
  - Current profit
  - Buy history (total bought, average buy price)
  - Sell history (total sold, average sell price)
