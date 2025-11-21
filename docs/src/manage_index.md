# Manage Index Fund

To manage an index fund using Bonança, you will need to make a `json` file
that describes the index fund configuration. An example configuration file is as follows:

```json
{
  "name": "Basic Index Fund",
  "chain": "Solana",
  "child": 0,
  "keyvault": "path/to/keyvault.json",
  "rpc_url": "https://api.mainnet-beta.solana.com",
  "max_offset": 0.01,
  "gas_address": "So11111111111111111111111111111111111111112",

  "aggregator": {
    "name": "Jupiter",
    "api_url": "https://lite-api.jup.ag",
    "api_key": "N/A"
  },

  "oracle": {
    "name": "Jupiter",
    "api_url": "https://lite-api.jup.ag",
    "api_key": "N/A"
  },

  "sectors": [
    {
      "name": "Layer 1",
      "weight": 0.5,
      "assets": [
        {
          "name": "Bitcoin",
          "symbol": "BTC",
          "address": "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh"
        },
        {
          "name": "Ethereum",
          "symbol": "ETH",
          "address": "7vfCXTUXx5WJV5JADk17DUJ4ksgau7utNKj4b963voxs"
        }
      ]
    },
    {
      "name": "DEX",
      "weight": 0.5,
      "assets": [
        {
          "name": "Raydium",
          "symbol": "RAY",
          "address": "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R"
        },
        {
          "name": "Jupiter",
          "symbol": "JUP",
          "address": "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN"
        }
      ]
    }
  ]
}
```

Lets break down the fields in this configuration file:

- `name`: The name of the index fund.
- `chain`: The blockchain where the index fund operates (e.g., "Solana").
- `child`: The child index for hierarchical deterministic wallets.
- `keyvault`: The path to the key vault file that contains the wallet keys.
- `rpc_url`: The RPC URL for connecting to the blockchain network.
- `max_offset`: The maximum allowable offset from the target asset allocation.
- `gas_address`: The address of the native token used for transaction fees.
- `aggregator`: Configuration for the exchange aggregator used for trading assets.
  - `name`: The name of the aggregator (e.g., "Jupiter").
  - `api_url`: The API URL for the aggregator.
  - `api_key`: The API key for accessing the aggregator (if required).
- `oracle`: Configuration for the price oracle used to fetch asset prices.
  - `name`: The name of the oracle (e.g., "Jupiter").
  - `api_url`: The API URL for the oracle.
  - `api_key`: The API key for accessing the oracle (if required).
- `sectors`: An array of sectors that group assets together. Each sector has the following fields:
  - `name`: The name of the sector (e.g., "Layer 1").
  - `weight`: The target weight of the sector in the index fund (e.g., 0.5 for 50%).
  - `assets`: An array of assets within the sector. Each asset has the following fields:
    - `name`: The name of the asset (e.g., "Bitcoin").
    - `symbol`: The symbol of the asset (e.g., "BTC").
    - `address`: The blockchain address of the asset.

Once you have created the configuration file, you can use Bonança to manage your index fund by executing commands such as balancing the fund, depositing assets, and withdrawing funds. Make sure to provide the path to your configuration file when running these commands.

```bash
bonanca balance -i path/to/index_fund_config.json
```

This command will print the balance of the index fund and the allocations of each asset. The allocations are shown as `actual / target`, where `actual` is the current allocation and `target` is the desired allocation based on the weights defined in the configuration file.

```bash
bonanca rebalance -i path/to/index_fund_config.json
```

This command will rebalance the index fund by buying and selling assets to match the target allocations defined in the configuration file. To see what trades would be executed without actually performing them, you can use the `--preview` flag (also `-p` for short):

```bash
bonanca rebalance -i path/to/index_fund_config.json --preview
```

This command will show the proposed trades needed to rebalance the index fund without executing them.

```bash
bonanca close -i path/to/index_fund_config.json -s RECEIVER_ADDRESS
```

This command will close the index fund by transfering all assets and to the specified receiver address. Make sure to replace `RECEIVER_ADDRESS` with the actual address where you want to send the assets.
