# Solana DeFi Interfaces

Bonanca provides interfaces to interact with popular DeFi protocols on Solana. This section covers the main DeFi integrations available.

## Jupiter - DEX Aggregator

Jupiter is the leading decentralized exchange aggregator on Solana, sourcing liquidity from multiple DEXs to find the best prices for token swaps. It also offers a lending protocol called Jupiter Earn.

### Overview

- **Token Swaps**: Get best pricing across multiple Solana DEXs
- **Limit Orders**: Place orders that execute at specified prices
- **Price Queries**: Check token prices and value conversions
- **Lending/Earn**: Deposit tokens to earn yield
- **Lendable Markets**: Query available lending markets

### Supported Operations

#### Rust

```rust,ignore
use bonanca::defi::Jupiter;
use bonanca::wallets::solana::SolWallet;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize wallet
    let wallet = SolWallet::new(
        "base58_private_key",
        "https://api.mainnet-beta.solana.com"
    )?;

    // Initialize Jupiter with API key
    let jupiter = Jupiter::new("your_api_key".to_string());

    // Token mint addresses
    let sol = "So11111111111111111111111111111111111111112";
    let usdc = "EPjFWdd5Au17FXxeB6VWyeXKSa3LvwV0LCH7f3uyxVEP";
    let orca = "orcaEKTdK7LKz57chYcUBK6GDZT5bNvmnucEZDfmWQe";

    // Example 1: Get token price in USD
    let sol_price = jupiter.get_token_price(token_mint, 1.0).await?;
    println!("1 SOL = ${}", sol_price);

    // Example 2: Get swap quote
    let quote = jupiter.get_swap_quote(
        &wallet,
        sol,      // Sell token (SOL)
        usdc,     // Buy token (USDC)
        1.0       // Amount in decimal format
    ).await?;

    println!("Input Amount: {}", quote.in_amount);
    println!("Output Amount: {}", quote.out_amount);
    println!("Price Impact: {}%", quote.price_impact_pct);
    println!("Slippage (bps): {}", quote.slippage_bps);

    // Example 3: Execute swap with quote
    let swap_receipt = jupiter.swap(&wallet, quote).await?;
    println!("Swap signature: {}", swap_receipt.hash);
    println!("Slot: {}", swap_receipt.slot);

    // Example 4: Quick swap (get quote and swap in one call)
    let receipt = jupiter.quick_swap(
        &wallet,
        sol,
        usdc,
        1.0
    ).await?;
    println!("Quick swap signature: {}", receipt.hash);

    // Example 5: Place limit order
    // Order is valid for 24 hours (86400 seconds)
    let limit_receipt = jupiter.limit_order(
        &wallet,
        sol,           // Sell token
        usdc,          // Buy token
        1.0,           // Amount to sell
        34.5,          // Minimum to receive
        Duration::from_secs(86400)
    ).await?;
    println!("Limit order signature: {}", limit_receipt.hash);

    // Example 6: Limit order by price
    // Sell 1 SOL if price reaches 35 USDC per SOL
    let price_limit_receipt = jupiter.limit_order_by_price(
        &wallet,
        sol,
        usdc,
        1.0,                  // Amount to sell
        35.0,                 // Price threshold
        Duration::from_secs(86400)
    ).await?;
    println!("Price limit order: {}", price_limit_receipt.hash);

    // Example 7: Get available lending markets
    let markets = jupiter.get_lendable_tokens().await?;
    for market in &markets {
        println!("Market: {} ({})", market.name, market.symbol);
        println!("  Supply Rate: {}", market.supply_rate);
        println!("  Total Assets: {}", market.total_assets);
        println!("  Total Supply: {}", market.total_supply);
    }

    // Example 8: Deposit into lending market
    let deposit_receipt = jupiter.deposit(
        &wallet,
        usdc,   // Token to deposit
        100.0   // Amount in decimal format
    ).await?;
    println!("Deposit signature: {}", deposit_receipt.hash);

    // Example 9: Withdraw from lending market
    let withdraw_receipt = jupiter.withdraw(
        &wallet,
        usdc,
        50.0
    ).await?;
    println!("Withdraw signature: {}", withdraw_receipt.hash);

    Ok(())
}
```

#### Python

```python
import bonanca
import time

# Initialize wallet
wallet = bonanca.wallets.SolWallet(
    "base58_private_key",
    "https://api.mainnet-beta.solana.com"
)

# Initialize Jupiter
jupiter = bonanca.defi.Jupiter("your_api_key")

# Token mint addresses
sol = "So11111111111111111111111111111111111111112"
usdc = "EPjFWdd5Au17FXxeB6VWyeXKSa3LvwV0LCH7f3uyxVEP"
orca = "orcaEKTdK7LKz57chYcUBK6GDZT5bNvmnucEZDfmWQe"

# Get token price
sol_price = jupiter.get_token_price(wallet, sol, 1.0)
print(f"1 SOL = ${sol_price}")

# Get swap quote
quote = jupiter.get_swap_quote(wallet, sol, usdc, 1.0)
print(f"Input: {quote.in_amount}")
print(f"Output: {quote.out_amount}")
print(f"Price Impact: {quote.price_impact_pct}%")

# Execute swap
swap_receipt = jupiter.swap(wallet, quote)
print(f"Swap tx: {swap_receipt.hash}")

# Quick swap (simpler one-liner)
receipt = jupiter.quick_swap(wallet, sol, usdc, 1.0)
print(f"Quick swap tx: {receipt.hash}")

# Limit order (valid for 1 day = 86400 seconds)
limit_receipt = jupiter.limit_order(
    wallet,
    sol,              # Sell token
    usdc,             # Buy token
    1.0,              # Amount to sell
    34.5,             # Minimum to receive
    86400             # Lifetime in seconds
)
print(f"Limit order: {limit_receipt.hash}")

# Price-based limit order
price_receipt = jupiter.limit_order_by_price(
    wallet,
    sol,
    usdc,
    1.0,              # Amount to sell
    35.0,             # Price threshold
    86400
)
print(f"Price limit: {price_receipt.hash}")

# Get available lending markets
markets = jupiter.get_lendable_tokens(wallet)
for market in markets:
    print(f"Market: {market.name} ({market.symbol})")
    print(f"  APY: {market.supply_rate}")
    print(f"  TVL: {market.total_assets}")

# Deposit into lending protocol
deposit_receipt = jupiter.deposit(wallet, usdc, 100.0)
print(f"Deposit tx: {deposit_receipt.hash}")

# Withdraw from lending protocol
withdraw_receipt = jupiter.withdraw(wallet, usdc, 50.0)
print(f"Withdraw tx: {withdraw_receipt.hash}")
```

### Quote Structure

The swap quote includes:

- **in_amount**: Exact input amount needed
- **out_amount**: Expected output amount
- **other_amount_threshold**: Minimum output considering slippage
- **price_impact_pct**: Price impact percentage
- **slippage_bps**: Slippage in basis points
- **context_slot**: Slot when quote was created
- **swap_mode**: Type of swap (ExactIn, ExactOut)

### Lending Markets

Jupiter Earn provides lending markets with:

- **supply_rate**: Current APY for deposits
- **total_assets**: Total assets in the market
- **total_supply**: Total supply shares issued
- **decimals**: Token decimal places
- **symbol**: Token symbol

### API Key

Get a free API key from [Jupiter](https://jup.ag/):

- Free tier available for development
- Rate limits apply
- Required for production swaps

---

## Kamino - Yield Optimization Vaults

Kamino is a vault protocol on Solana that optimizes yield strategies by allocating assets across multiple lending markets. Kamino vaults provide automated yield farming and lending strategies.

### Overview

- **Vault Discovery**: Find vaults by name, ID, or underlying token
- **Vault Metrics**: APY, TVL, performance tracking
- **Position Management**: Deposit and withdraw from vaults
- **User Positions**: Track vault holdings across all vaults
- **Strategy Allocation**: Vaults allocate across KLend reserves

### Important Note

Kamino is currently available in **Rust only**. Python bindings are not yet available. The interface requires direct interaction with the Kamino smart contracts via Anchor.

### Supported Operations (Rust Only)

```rust,ignore
use bonanca::defi::Kamino;
use bonanca::wallets::solana::SolWallet;

#[tokio::main]
async fn main() -> Result<()> {
    let wallet = SolWallet::new(
        "base58_private_key",
        "https://api.mainnet-beta.solana.com"
    )?;

    let kamino = Kamino::new();

    // Example 1: Get all available vaults
    let all_vaults = kamino.get_vaults().await?;
    println!("Total vaults: {}", all_vaults.len());
    for vault in &all_vaults {
        println!("Vault: {} ({})", vault.state.name, vault.address);
        println!("  Token: {}", vault.state.token_mint);
        println!("  APY: {}", vault.state.name);
    }

    // Example 2: Find vault by name
    let usdc_vault = kamino.get_vault_data_by_name("Kamino USDC")
        .await?;
    println!("USDC Vault Address: {}", usdc_vault.address);

    // Example 3: Find vault by ID
    let vault_id = "7i..."; // Vault address
    let vault_data = kamino.get_vault_data_by_id(vault_id).await?;
    println!("Vault Name: {}", vault_data.state.name);

    // Example 4: Get user's vault positions
    let positions = kamino.get_user_data(&wallet.pubkey.to_string()).await?;
    println!("User positions: {}", positions.len());
    for position in &positions {
        println!("Vault: {}", position.vault_address);
        println!("  Staked Shares: {}", position.staked_shares);
        println!("  Unstaked Shares: {}", position.unstaked_shares);
        println!("  Total Shares: {}", position.total_shares);
    }

    // Example 5: Find all USDC vaults
    let usdc_mint = "EPjFWdd5Au17FXxeB6VWyeXKSa3LvwV0LCH7f3uyxVEP";
    let usdc_vaults = kamino.get_token_vaults(usdc_mint).await?;
    println!("USDC vaults available: {}", usdc_vaults.len());

    // Example 6: Deposit into a vault
    let vault = kamino.get_vault_data_by_name("Kamino USDC").await?;
    kamino.supply(&wallet, &vault, 1000.0).await?;
    println!("Deposited 1000 USDC into vault");

    // Example 7: Withdraw from a vault
    kamino.withdraw(&wallet, &vault, 500.0).await?;
    println!("Withdrew 500 USDC from vault");

    // Example 8: Monitor vault performance
    println!("\nVault Details:");
    println!("  Name: {}", vault.state.name);
    println!("  Token Mint: {}", vault.state.token_mint);
    println!("  Shares Mint: {}", vault.state.shares_mint);
    println!("  Total Shares Issued: {}", vault.state.shares_issued);
    println!("  Performance Fee: {} bps", vault.state.performance_fee_bps);
    println!("  Management Fee: {} bps", vault.state.management_fee_bps);

    Ok(())
}
```

### Vault Information

Each vault provides detailed information:

- **name**: Human-readable vault name
- **token_mint**: The token deposited in this vault
- **shares_mint**: The receipt token for vault shares
- **address**: Vault account address
- **token_available**: Amount available for withdrawal
- **shares_issued**: Total shares in circulation
- **performance_fee_bps**: Fee on profits (basis points)
- **management_fee_bps**: Annual management fee
- **vault_allocation_strategy**: How tokens are allocated across reserves

### Vault Allocation

Kamino vaults automatically allocate deposits across multiple lending reserves:

- Optimizes yield across KLend markets
- Rebalances automatically
- Charges performance and management fees
- Reinvests earned interest

### User Positions

Track positions across all vaults:

- **staked_shares**: Shares in staking/earning strategies
- **unstaked_shares**: Shares not yet fully invested
- **total_shares**: Total position size
- One entry per vault per user

### Fee Structure

- **Performance Fee**: Percentage of earned yield (in basis points)
- **Management Fee**: Annual fee charged (in basis points)
- **Example**: 10% performance fee = 1000 bps, 1% management fee = 100 bps

---

## Swap Best Practices

### General Guidelines

1. **Slippage Management**
   - Always check `other_amount_threshold` on quotes
   - Jupiter automatically handles slippage configuration
   - Adjust slippage for volatile markets

2. **Quote Validity**
   - Quotes are typically valid for ~30 seconds
   - Refresh quotes before executing large swaps
   - Use quick_swap for time-sensitive trades

3. **Price Impact**
   - Monitor `price_impact_pct` in quotes
   - Larger swaps have higher price impact
   - Consider breaking into smaller swaps if > 5% impact

### Limit Order Strategy

- **Expiration**: Set lifetime based on trading frequency
  - Short-term: 1-24 hours
  - Day trading: 1 hour
  - Long-term: 7+ days

- **Order Amounts**
  - limit_order takes exact amounts
  - limit_order_by_price calculates amounts from price

- **Order Book**
  - Limit orders wait for market to reach your price
  - No execution guarantee
  - Prices are checked continuously

## Vault Management Best Practices

### Deposit Strategy

1. **Find Appropriate Vault**
   - Check vault APY and TVL
   - Verify vault allocation strategy
   - Review fee structure

2. **Monitor Positions**
   - Track share value over time
   - Monitor APY changes
   - Watch fee deductions

3. **Withdrawal Timing**
   - Withdraw when strategy is optimal
   - Consider gas costs
   - Account for any lock-up periods

### Risk Management

- **Concentration Risk**: Diversify across multiple vaults/tokens
- **Smart Contract Risk**: Kamino/KLend are audited but not risk-free
- **Market Risk**: Underlying tokens can be volatile
- **Strategy Risk**: Allocations may not perform as expected

## Common Patterns

### Simple Swap

```rust,ignore
let receipt = jupiter.quick_swap(&wallet, sell_token, buy_token, 1.0).await?;
```

### Quoted Swap

```rust,ignore
let quote = jupiter.get_swap_quote(&wallet, sell, buy, amount).await?;
// Check quote before proceeding
let receipt = jupiter.swap(&wallet, quote).await?;
```

### Deposit to Earn

```rust,ignore
let markets = jupiter.get_lendable_tokens().await?;
// Find market with best APY
let receipt = jupiter.deposit(&wallet, chosen_market, amount).await?;
```

### Vault Position Tracking

```rust,ignore
let kamino = Kamino::new();
let positions = kamino.get_user_data(&wallet_pubkey).await?;
for position in positions {
    let vault = kamino.get_vault_data_by_id(&position.vault_address).await?;
    println!("Position in {}: {}", vault.state.name, position.total_shares);
}
```

## Token Addresses

Common Solana token addresses:

- **SOL**: So11111111111111111111111111111111111111112
- **USDC**: EPjFWdd5Au17FXxeB6VWyeXKSa3LvwV0LCH7f3uyxVEP
- **USDT**: Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenEst
- **ORCA**: orcaEKTdK7LKz57chYcUBK6GDZT5bNvmnucEZDfmWQe
- **JUP**: JUPyiwrYJFskidvHPcMj5kLSnsxPJ3Bad7Or7SvxJvP
- **COPE**: 8HGyAAB1yoM1ttS7pnK6DGPhcF1JSQhQTiNrgQpTsEq
- **RAY**: 4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX4r

## Troubleshooting

### Swap Failures

- **Insufficient Balance**: Ensure wallet has enough tokens
- **Slippage Exceeded**: Quote may have expired, refresh and try again
- **Network Congestion**: Retry after waiting or increase priority fee
- **Liquidity Issues**: Requested pair may not have sufficient liquidity

### Limit Order Issues

- **Order Not Filling**: Price may not have reached your target
- **Expired Order**: Lifetime has passed, create new order
- **Invalid Amounts**: Ensure amounts are correctly formatted

### Vault Operations

- **Deposit Fails**: Verify token balance and vault capacity
- **Withdraw Fails**: Ensure sufficient shares, check vault liquidity
- **Account Creation**: First interaction with vault may require account setup

## Security Considerations

- Never share your private key
- Use environment variables for sensitive data
- Test on devnet before mainnet
- Monitor transaction fees and balances
- Verify token addresses before trading
- Start with small amounts while testing
