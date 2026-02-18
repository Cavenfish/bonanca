# HD Wallets

Bonança can create a master key for a Hierarchical Deterministic (HD)
wallet, which can generate multiple child keys for different blockchains.
This means you can manage multiple wallets for multiple blockchains (e.g.,
Solana, EVM-compatible chains) using a single `KeyVault` file. If you are
unfamiliar with HD wallets consider reading [this post](https://learnmeabitcoin.com/technical/keys/hd-wallets/).

## Creating a New HD Wallet

HD wallets are stored as `KeyVault` files in Bonança, which is explained
in more depth [here](keyvault.md). You can create a `KeyVault` using Rust
or Python.

#### Rust

```rust,ignore
use bonanca::keyvault::KeyVault;
use std::path::Path;

fn main() {
  // Create new KeyVault with english mneomonic
  let key_vault = KeyVault::new("English");
  let filename = Path::new("./keyvault.json");

  // Write json file
  key_vault.write(filename)
}
```

#### Python

```python
from bonanca import KeyVault

# Create new KeyVault with english mneomonic
key_vault = KeyVault.new("English")

# Write json file
key_vault.write("./keyvault.json")
```

The available languages for the mneomonic are:

- English
- Simplified Chinese
- Traditional Chinese,
- French
- Italian
- Japanese
- Korean
- Spanish

## Create a KeyVault from a Mneomonic

You can also create a `KeyVault` from an existing mneomonic phrase. This is useful if you already have a wallet and want to manage it using Bonança.

#### Rust

```rust,ignore
use bonanca::keyvault::KeyVault;

fn main() {
  // Your mneomonic (in any language listed above)
  let mneomonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

  let key_vault = KeyVault::from_mneomonic(mneomonic);
  let filename = Path::new("./keyvault.json");

  // Write json file
  key_vault.write(filename)
}
```

#### Python

```python
from bonanca import KeyVault

# Your mneomonic (in any language listed above)
mneomonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about

key_vault = KeyVault.from_mneomonic(mneomonic)

# Write json file
key_vault.write("./keyvault.json")
```
