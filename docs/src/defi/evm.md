# EVM DeFi Interfaces

Bonanca provides interfaces to interact with popular DeFi protocols on EVM-compatible blockchains. This section covers the main DeFi integrations available.

## Aave V3 - Lending Protocol

Aave V3 is a decentralized lending protocol that allows users to supply assets as collateral, borrow tokens, and earn interest. The AaveV3 interface provides methods for managing lending positions.

### Overview

- **Supply**: Deposit tokens to earn interest
- **Borrow**: Borrow tokens using your collateral
- **Repay**: Pay back borrowed tokens
- **Withdraw**: Withdraw supplied tokens
- **Account Data**: Query user's collateral, debt, and health factor

### Supported Operations

#### Rust

```rust,ignore
use bonanca::defi::AaveV3;
use bonanca::wallets::evm::EvmWallet;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize wallet
    let wallet = EvmWallet::new(
        "private_key",
        "rpc_url",
        137  // Chain ID for Polygon
    )?;

    // Initialize Aave V3 for Polygon (chain_id=137)
    let aave = AaveV3::new(137);

    // Example 1: Supply USDC
    let usdc_address = "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359";
    let receipt = aave.supply(&wallet, usdc_address, 100.0).await?;
    println!("Supply tx: {:?}", receipt.transaction_hash);

    // Example 2: Check user account data
    let user_data = aave.get_user_data(
        &wallet.pubkey.to_string(),
        &wallet.client
    ).await?;
    println!("Health Factor: {}", user_data.health_factor);
    println!("Total Collateral: {}", user_data.total_collateral);
    println!("Total Debt: {}", user_data.total_debt);
    println!("Available Borrows: {}", user_data.available_borrows);

    // Example 3: Borrow WETH
    let weth_address = "0x7ceB23fD6bC0adD59E27f9EA9d0231e0f01cc726";
    let borrow_receipt = aave.borrow(&wallet, weth_address, 0.5).await?;
    println!("Borrow tx: {:?}", borrow_receipt.transaction_hash);

    // Example 4: Repay borrowed tokens
    let repay_receipt = aave.repay(&wallet, weth_address, 0.3).await?;
    println!("Repay tx: {:?}", repay_receipt.transaction_hash);

    // Example 5: Withdraw collateral
    let withdraw_receipt = aave.withdraw(&wallet, usdc_address, 50.0).await?;
    println!("Withdraw tx: {:?}", withdraw_receipt.transaction_hash);

    Ok(())
}
```

#### Python

```python
import bonanca

# Initialize wallet
wallet = bonanca.wallets.EvmWallet(
    "private_key",
    "https://polygon-rpc.com",
    137  # Chain ID for Polygon
)

# Initialize Aave V3
aave = bonanca.defi.AaveV3(137)

# Token addresses on Polygon
usdc = "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359"
weth = "0x7ceB23fD6bC0adD59E27f9EA9d0231e0f01cc726"

# Supply tokens
supply_receipt = aave.supply(wallet, usdc, 100.0)
print(f"Supply tx: {supply_receipt['transaction_hash']}")

# Check account data
account_data = aave.get_user_data(wallet.pubkey, wallet)
print(f"Health Factor: {account_data['Health Factor']}")
print(f"Total Collateral: {account_data['Total Collateral']}")
print(f"Total Debt: {account_data['Total Debt']}")

# Borrow tokens
borrow_receipt = aave.borrow(wallet, weth, 0.5)
print(f"Borrow tx: {borrow_receipt['transaction_hash']}")

# Repay debt
repay_receipt = aave.repay(wallet, weth, 0.3)
print(f"Repay tx: {repay_receipt['transaction_hash']}")

# Withdraw collateral
withdraw_receipt = aave.withdraw(wallet, usdc, 50.0)
print(f"Withdraw tx: {withdraw_receipt['transaction_hash']}")
```

### Chain IDs

Common Aave V3 deployments:

- Ethereum: 1
- Polygon: 137
- Arbitrum: 42161
- Optimism: 10
- Base: 8453
- Avalanche: 43114

### Important Notes

- Ensure sufficient collateral before borrowing
- Monitor your health factor to avoid liquidation
- The health factor formula considers collateral and debt ratios
- Interest rates vary based on market utilization
- Requires token approval before first interaction with Aave

---

## CoW (Coincidence of Wants) - DEX Aggregator

CoW is an order-based DEX aggregator that uses batch auctions to execute trades. It supports both market orders and limit orders.

### Overview

- **Market Orders**: Execute immediately at best available price
- **Limit Orders**: Execute only when price reaches your target
- **Order Queries**: Check order status and history
- **EIP-712 Signing**: Secure order signing

### Supported Operations

#### Rust

```rust,ignore
use bonanca::defi::CoW;
use bonanca::wallets::evm::EvmWallet;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let wallet = EvmWallet::new(
        "private_key",
        "https://polygon-rpc.com",
        137
    )?;

    // Initialize CoW for Polygon
    let cow = CoW::new("polygon")?;

    let weth = "0x7ceB23fD6bC0adD59E27f9EA9d0231e0f01cc726";
    let usdc = "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359";

    // Example 1: Execute market order (WETH → USDC)
    let market_quote = cow.get_market_quote(
        &wallet,
        weth,
        usdc,
        0.1  // 0.1 WETH
    ).await?;

    let order_uid = cow.post_market_order(
        &wallet,
        market_quote
    ).await?;
    println!("Market order placed: {}", order_uid);

    // Example 2: Check order status
    let order_info = cow.get_order_info(&order_uid).await?;
    println!("Status: {}", order_info.status);
    println!("Executed Buy Amount: {}", order_info.executed_buy_amount);
    println!("Executed Sell Amount: {}", order_info.executed_sell_amount);

    // Example 3: Get user order history
    let user_orders = cow.get_user_orders(
        &wallet.pubkey.to_string(),
        Some(10)  // Last 10 orders
    ).await?;
    println!("Recent orders: {} found", user_orders.len());

    // Example 4: Place limit order
    // Valid for 1 hour (60 minutes + 0 seconds)
    let limit_duration = Duration::new(3600, 0);
    let limit_uid = cow.limit_order(
        &wallet,
        weth,      // Selling WETH
        usdc,      // Buying USDC
        0.1,       // Sell amount
        200.0,     // Buy amount (minimum)
        limit_duration
    ).await?;
    println!("Limit order placed: {}", limit_uid);

    // Example 5: Place limit order by price
    let price_limit_uid = cow.limit_order_by_price(
        &wallet,
        weth,
        usdc,
        0.1,       // Amount of WETH to sell
        2000.0,    // Minimum price (USDC per WETH)
        limit_duration
    ).await?;
    println!("Price limit order placed: {}", price_limit_uid);

    Ok(())
}
```

#### Python

```python
import bonanca

wallet = bonanca.wallets.EvmWallet(
    "private_key",
    "https://polygon-rpc.com",
    137
)

# Initialize CoW for Polygon
cow = bonanca.defi.CoW("polygon")

weth = "0x7ceB23fD6bC0adD59E27f9EA9d0231e0f01cc726"
usdc = "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359"

# Market order: Sell 0.1 WETH for USDC
order_uid = cow.market_order(wallet, weth, usdc, 0.1)
print(f"Order UID: {order_uid}")

# Get order details
order_info = cow.get_order_info(order_uid)
print(f"Status: {order_info['status']}")
print(f"Buy Amount: {order_info['executed_buy_amount']}")
print(f"Sell Amount: {order_info['executed_sell_amount']}")

# Get all orders for a user
user_orders = cow.get_user_orders(wallet.pubkey, 10)
print(f"Found {len(user_orders)} recent orders")

# Limit order: Sell 0.1 WETH for minimum 200 USDC, valid for 1 hour
limit_uid = cow.limit_order(
    wallet,
    weth,               # Sell token
    usdc,               # Buy token
    0.1,                # Sell amount
    200.0,              # Minimum buy amount
    (1, 0)              # Duration: 1 hour, 0 minutes
)
print(f"Limit order: {limit_uid}")

# Limit order by price
price_uid = cow.limit_order_by_price(
    wallet,
    weth,
    usdc,
    0.1,                # Amount to sell
    2000.0,             # Minimum price
    (1, 0)              # Duration
)
print(f"Price limit order: {price_uid}")
```

### Supported Chains

- Ethereum (mainnet)
- Polygon
- Arbitrum
- Optimism
- Base
- Gnosis Chain (xDai)
- Linea
- Avalanche
- BNB Chain

### Important Notes

- CoW Protocol uses batch auctions for better pricing
- Limit orders expire after specified duration
- Orders must be signed with EIP-712
- Focus on MEV protection compared to traditional DEXs
- No gas required for failed orders

---

## Morpho - Lending Optimization

Morpho optimizes lending by matching users peer-to-peer while falling back to lending pools. The MorphoVaultV1 interface provides access to Morpho vaults.

### Overview

- **Supply**: Deposit into optimized vaults
- **Withdraw**: Withdraw from vaults
- **Vault Data**: Query available vaults for specific tokens
- **User Positions**: Track user's vault positions

### Supported Operations

#### Rust

```rust,ignore
use bonanca::defi::MorphoVaultV1;
use bonanca::wallets::evm::EvmWallet;

#[tokio::main]
async fn main() -> Result<()> {
    let wallet = EvmWallet::new(
        "private_key",
        "https://polygon-rpc.com",
        137
    )?;

    let morpho = MorphoVaultV1::new();

    // Example 1: Get user's vault positions
    let user_positions = morpho.get_user_data(
        &wallet.pubkey.to_string(),
        137  // Chain ID
    ).await?;
    println!("User positions: {:?}", user_positions);

    // Example 2: Find USDC vaults
    let usdc_vaults = morpho.get_token_vaults(
        "USDC",
        137
    ).await?;
    println!("Available USDC vaults: {}", usdc_vaults.len());
    for vault in &usdc_vaults {
        println!("Vault: {:?}", vault);
    }

    // Example 3: Supply to a vault
    // First, get a vault address from get_token_vaults()
    let vault_address = "0x..."; // Example vault address
    let deposit_receipt = morpho.supply(
        &wallet,
        vault_address,
        100.0  // Amount in decimal format
    ).await?;
    println!("Deposit tx: {:?}", deposit_receipt.transaction_hash);

    // Example 4: Withdraw from vault
    let withdraw_receipt = morpho.withdraw(
        &wallet,
        vault_address,
        50.0
    ).await?;
    println!("Withdraw tx: {:?}", withdraw_receipt.transaction_hash);

    Ok(())
}
```

#### Python

```python
import bonanca

wallet = bonanca.wallets.EvmWallet(
    "private_key",
    "https://polygon-rpc.com",
    137
)

morpho = bonanca.defi.MorphoVaultV1()

# Get user's vault positions
user_data = morpho.get_user_data(wallet.pubkey, 137, wallet)
print(f"User positions: {user_data}")

# Find USDC vaults
usdc_vaults = morpho.get_token_vaults("USDC", 137, wallet)
print(f"Available vaults: {usdc_vaults}")

# Supply to a vault (replace with actual vault address)
vault_address = "0x..."
deposit_receipt = morpho.supply(wallet, vault_address, 100.0)
print(f"Deposit tx: {deposit_receipt['transaction_hash']}")

# Withdraw from vault
withdraw_receipt = morpho.withdraw(wallet, vault_address, 50.0)
print(f"Withdraw tx: {withdraw_receipt['transaction_hash']}")
```

### Morpho Vaults

Morpho offers specialized vaults for different strategies:

- **USDC Vaults**: Stable yield on USDC
- **WETH Vaults**: Ethereum staking
- **Strategy Vaults**: Custom yield strategies
- **Metamorpho Vaults**: User-created vault bundles

### Important Notes

- Vault addresses vary by chain
- Deposits are immediately invested
- Withdrawals may take time depending on vault liquidity
- Each vault has different risk/reward profiles
- Query vault details for APY and TVL information

---

## 0x (ZeroX) - DEX Aggregator

0x is a decentralized exchange aggregator that sources liquidity from multiple DEXs to find the best prices for token swaps.

### Overview

- **Swap Quotes**: Get best available swap prices
- **Quick Swaps**: Execute swaps in one transaction
- **Issue Checking**: Verify swap feasibility
- **Multi-source Routing**: Optimal liquidity from multiple DEXs

### Supported Operations

#### Rust

```rust,ignore
use bonanca::defi::ZeroX;
use bonanca::wallets::evm::EvmWallet;

#[tokio::main]
async fn main() -> Result<()> {
    let wallet = EvmWallet::new(
        "private_key",
        "https://polygon-rpc.com",
        137
    )?;

    // Initialize 0x with API key (get from 0x.org)
    let zerox = ZeroX::new("your_api_key".to_string(), 137);

    let weth = "0x7ceB23fD6bC0adD59E27f9EA9d0231e0f01cc726";
    let usdc = "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359";

    // Example 1: Check for swap issues
    let issues = zerox.check_swap(
        &wallet,
        weth,
        usdc,
        0.1  // 0.1 WETH
    ).await?;

    if issues.allowance.is_some() {
        println!("Token approval required");
    }
    if issues.balance.is_some() {
        println!("Insufficient balance");
    }

    // Example 2: Get swap quote
    let quote = zerox.get_swap_quote(
        &wallet,
        weth,
        usdc,
        0.1
    ).await?;

    println!("Buy Amount: {}", quote.buy_amount);
    println!("Min Buy Amount: {}", quote.min_buy_amount);
    println!("Liquidity Available: {}", quote.liquidity_available);
    println!("Allowance Target: {}", quote.allowance_target);

    // Example 3: Execute swap with quote
    let swap_receipt = zerox.swap(
        &wallet,
        quote
    ).await?;
    println!("Swap tx: {:?}", swap_receipt.transaction_hash);

    // Example 4: Quick swap (get quote and swap in one call)
    // This is more convenient for simpler scenarios
    let receipt = zerox.quick_swap(
        &wallet,
        weth,
        usdc,
        0.1
    ).await?;
    println!("Quick swap tx: {:?}", receipt.transaction_hash);

    Ok(())
}
```

#### Python

```python
import bonanca

wallet = bonanca.wallets.EvmWallet(
    "private_key",
    "https://polygon-rpc.com",
    137
)

# Initialize 0x (replace with actual API key from 0x.org)
zerox = bonanca.defi.ZeroX("api_key", 137)

weth = "0x7ceB23fD6bC0adD59E27f9EA9d0231e0f01cc726"
usdc = "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359"

# Check for swap issues (allowance, balance)
issues = zerox.check_swap(wallet, weth, usdc, 0.1)
print(f"Issues: {issues}")

# Get detailed quote
quote = zerox.get_swap_quote(wallet, weth, usdc, 0.1)
print(f"Buy Amount: {quote.buy_amount}")
print(f"Min Buy Amount: {quote.min_buy_amount}")
print(f"Liquidity Available: {quote.liquidity_available}")

# Execute swap with quote
swap_receipt = zerox.swap(wallet, quote)
print(f"Swap tx: {swap_receipt['transaction_hash']}")

# Quick swap (recommended for simple swaps)
quick_receipt = zerox.quick_swap(wallet, weth, usdc, 0.1)
print(f"Quick swap tx: {quick_receipt['transaction_hash']}")
```

### Supported Chains

- Ethereum
- Polygon
- Arbitrum
- Optimism
- Base
- Avalanche
- BNB Chain
- Linea
- Scroll

### Quote Information

The swap quote returns:

- **buy_amount**: Exact amount of output tokens
- **min_buy_amount**: Minimum considering slippage
- **sell_amount**: Amount of input tokens
- **allowance_target**: Address to approve for token spending
- **liquidity_available**: Whether sufficient liquidity exists

### Important Notes

- Requires 0x API key (free tier available)
- Always check issues before executing trades
- Slippage is automatically handled
- Gas estimates included in quotes
- Quote validity is typically 1 minute
- Quick swaps are simpler but less flexible than manual swap flow

---

## General Best Practices

### Security

1. **Private Key Management**
   - Never hardcode private keys
   - Use environment variables or secure vaults
   - Rotate keys regularly

2. **Token Approval**
   - Check allowances before transactions
   - Use minimal necessary amounts
   - Revoke approvals when done

3. **Position Management**
   - Monitor health factors in lending protocols
   - Set alerts for liquidation risk
   - Rebalance positions regularly

### Gas Optimization

- Batch multiple operations when possible
- Use limit orders for time-insensitive trades
- Monitor gas prices before executing
- Consider network congestion

### Testing

- Test on testnet before mainnet
- Start with small amounts
- Verify transactions before committing funds
- Use price impact slippage limits

### Transaction Confirmation

All operations return transaction receipts containing:

- Transaction hash
- Block number
- Gas used
- Transaction status

Always verify successful execution before relying on results.
