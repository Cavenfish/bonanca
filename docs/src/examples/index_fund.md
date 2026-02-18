# Index Fund Example

The Index Fund is a Rust-based portfolio management tool that maintains a diversified cryptocurrency index. It allows you to manage a basket of tokens with automatic rebalancing to maintain target weights.

You can see the example code [here](https://github.com/Cavenfish/bonanca/tree/main/examples/crates/index-fund).

## Features

- Create and manage a diversified portfolio of cryptocurrencies
- Automatic rebalancing to target weights
- Multiple rebalancing strategies

## Building the Project

```bash
cd examples/crates/index-fund
cargo build --release
```

## Basic Usage

This example project build a CLI which includes help docs that cover basic usage.

```bash
index --help
```

## Configuration

Create an index fund configuration file:

```json
{
  "name": "Basic Index Fund",
  "chain": "EVM:Polygon",
  "chain_id": 137,
  "rpc_url": "https://1rpc.io/matic",
  "keyvault": "./keyvault.json",
  "child": 0,
  "max_offset": 0.01,

  "aggregator": {
    "name": "0x",
    "api_key": "API_KEY"
  },

  "oracle": {
    "name": "DefiLlama",
    "api_key": "API_KEY"
  },

  "auxiliary_assets": [
    {
      "name": "Dai Stablecoin",
      "symbol": "DAI",
      "address": "0x8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063"
    }
  ],

  "sectors": [
    {
      "name": "Layer 1",
      "weight": 0.4,
      "assets": [
        {
          "name": "Bitcoin",
          "symbol": "WBTC",
          "address": "0x1BFD67037B42Cf73acF2047067bd4F2C47D9BfD6"
        },
        {
          "name": "Ethereum",
          "symbol": "WETH",
          "address": "0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619"
        }
      ]
    },
    {
      "name": "DeFi",
      "weight": 0.2,
      "assets": [
        {
          "name": "Uniswap",
          "symbol": "UNI",
          "address": "0xb33EaAd8d922B1083446DC23f610c2567fB5180f"
        },
        {
          "name": "Aave",
          "symbol": "AAVE",
          "address": "0xD6DF932A45C0f255f85145f286eA0b292B21C90B"
        }
      ]
    },
    {
      "name": "Infrastructure",
      "weight": 0.2,
      "assets": [
        {
          "name": "ChainLink",
          "symbol": "LINK",
          "address": "0x53E0bca35eC356BD5ddDFebbD1Fc0fD03FaBad39"
        },
        {
          "name": "Graph Token",
          "symbol": "GRT",
          "address": "0x5fe2B58c013d7601147DcdD68C143A77499f5531"
        }
      ]
    },
    {
      "name": "Stablecoins",
      "weight": 0.2,
      "assets": [
        {
          "name": "Circle USD",
          "symbol": "USDC",
          "address": "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359"
        },
        {
          "name": "Stasis EUR",
          "symbol": "EURS",
          "address": "0xE111178A87A3BFf0c8d18DECBa5798827539Ae99"
        }
      ]
    }
  ]
}
```
