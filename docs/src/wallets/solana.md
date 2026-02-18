# Solana Wallet

To interact with a Solana wallet using Bonança you can use the `SolWallet`
struct. For all examples shown below its assumed a wallet has been initialized
as `wallet`. In the examples where signing is not required a `view` wallet is
sufficient.

## Parse and Format Decimals

You can use the `parse_decimals` and `format_decimals` methods to convert between
the raw integer representation of token amounts and the human-readable decimal format.

#### Rust

```rust,ignore
// Format a float into a big int
let big_amount = wallet.format_native(2.5); // Sol only
let big_amount_token = wallet.format_token(14.2, "TOKEN_MINT"); // Any SPL token

// Parse a big int number into a float
let x0 = wallet.parse_native(big_amount); // Sol only
let x1 = wallet.parse_token(big_amount_token, "TOKEN_MINT"); // Any SPL token
```

```python
# Format a float into a big int
big_amount = wallet.format_native(2.5) # Sol only
big_amount_token = wallet.format_token(14.2, "TOKEN_MINT") # Any SPL token

# Parse a big int number into a float
x0 = wallet.parse_native(big_amount)# Sol only
x1 = wallet.parse_token(big_amount_token, "TOKEN_MINT")# Any SPL token
```

## Balances

Bonança provides methods to check the balance of SOL and SPL tokens in your wallet. You can use the `balance` method to retrieve the balance of SOL and the `token_balance` method for SPL tokens.

#### Rust

```rust,ignore
// Get Sol balance (as float)
let sol_bal = wallet.balance().await?;

// Get SPL token balance (as float)
let spl_bal = wallet.token_balance("TOKEN_MINT").await?;
```

#### Python

```python
sol_bal = wallet.balance()
spl_bal = wallet.token_balance("TOKEN_MINT")
```

## Create Token Account

Before you can hold or transfer SPL tokens, you need to create a token account for the specific token mint. You can use the `create_token_account` method to create a new token account.

#### Rust

```rust,ignore
let ata_pubkey = wallet.create_token_account("TOKEN_MINT").await?;
```

#### Python

```python
ata_pubkey = wallet.create_token_account("TOKEN_MINT")
```

## Burn Token

To burn SPL tokens, you can use the `burn_token` method. This will permanently remove the specified amount of tokens from circulation. Note that unlike EVM
chains in Solana you can't burn tokens by sending them to burn address.

#### Rust

```rust,ignore
let receipt = wallet.burn_token("TOKEN_MINT", 2.5).await?;
```

#### Python

```python
receipt = wallet.burn_token("TOKEN_MINT", 2.5)
```

## Close Token Account

To close a token account, you can use the `close_token_account` method. This
can only be done when the token balance is zero (you must send or burn all
tokens before calling this method). After closing the account the SOL deposit
for the account will be returned to your wallet.

#### Rust

```rust,ignore
wallet.close_token_account("TOKEN_MINT").await?;
```

#### Python

```python
wallet.close_token_account("TOKEN_MINT")
```

## Transfers

To transfer SOL you can use the `transfer` method, and `token_transfer` for SPL
tokens. For SOL transfers, specify the recipient's public key and the amount
as a decimal value. For SPL token transfers, specify the recipient's token
account, the token mint, and the amount in decimal value.

#### Rust

```rust,ignore
// Transfer 2.5 Sol
let receipt = wallet.transfer(2.5, "TO_ADDRESS").await?;

// Transfer 2.5 SPL token
let receipt2 = wallet.token_transfer("TOKEN_MINT", 2.5, "TO_ADDRESS").await?;
```

#### Python

```python
# Transfer 2.5 Sol
receipt = wallet.transfer(2.5, "TO_ADDRESS")

# Transfer 2.5 SPL token
receipt2 = wallet.token_transfer("TOKEN_MINT", 2.5, "TO_ADDRESS")
```
