# EVM Wallet

To interact with an EVM-compatible wallet using `bonanca` you can use
the `EvmWallet` struct.

## Parse and Format Decimals

You can use the `parse_decimals` and `format_decimals` methods to convert between
the raw integer representation of token amounts and the human-readable decimal format.

#### Rust

```rust,ignore
// Format a float into a big int
let big_amount = wallet.format_native(2.5); // Native (ETH, POL, ...) only
let big_amount_token = wallet.format_token(14.2, "TOKEN_ADDRESS"); // Any token

// Parse a big int number into a float
let x0 = wallet.parse_native(big_amount); // Native (ETH, POL, ...) only
let x1 = wallet.parse_token(big_amount_token, "TOKEN_ADDRESS"); // Any token
```

#### Python

```python
# Format a float into a big int
big_amount = wallet.format_native(2.5)# Native (ETH, POL, ...) only
big_amount_token = wallet.format_token(14.2, "TOKEN_ADDRESS")# Any token

# Parse a big int number into a float
x0 = wallet.parse_native(big_amount)# Native (ETH, POL, ...) only
x1 = wallet.parse_token(big_amount_token, "TOKEN_ADDRESS")# Any token
```

## Balances

Bonança provides methods to check native and token balances in your wallet. You can use the `balance` method to retrieve the native balance and the `token_balance` method for tokens.

#### Rust

```rust,ignore
// Get native balance (as float)
let nat_bal = wallet.balance().await?;

// Get token balance (as float)
let tkn_bal = wallet.token_balance("TOKEN_ADDRESS").await?;
```

#### Python

```python
sol_bal = wallet.balance()
spl_bal = wallet.token_balance("TOKEN_ADDRESS")
```

## Token Approvals

To approve an address for spending your tokens you can use the
`approve_token_spending` method.

#### Rust

```rust,ignore
// Approve 2.05 tokens to be spent by spender address
wallet.approve_token_spending("TOKEN_ADDRESS","SPENDER_ADDRESS", 2.05).await?;
```

#### Python

```python
wallet.approve_token_spending("TOKEN_ADDRESS","SPENDER_ADDRESS", 2.05)
```

## Transfers

For native transfers you can use the `transfer` method, and `token_transfer` for
tokens. For native transfers, specify the recipient's public key and the amount
as a decimal value. For token transfers, specify the recipient's token
account, the token address, and the amount in decimal value.

#### Rust

```rust,ignore
// Transfer 2.5 native
let receipt = wallet.transfer(2.5, "TO_ADDRESS").await?;

// Transfer 2.5 token
let receipt2 = wallet.token_transfer("TOKEN_ADDRESS", 2.5, "TO_ADDRESS").await?;
```

#### Python

```python
# Transfer 2.5 native
receipt = wallet.transfer(2.5, "TO_ADDRESS")

# Transfer 2.5 token
receipt2 = wallet.token_transfer("TOKEN_ADDRESS", 2.5, "TO_ADDRESS")
```
