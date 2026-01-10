# HD Wallet Creation

Bonança can create a master key for a Hierarchical Deterministic (HD)
wallet, which can generate multiple child keys for different blockchains.
This means you can manage multiple wallets for multiple blockchains (e.g.,
Solana, EVM-compatible chains) using a single `KeyVault` file. If you are
unfamiliar with HD wallets consider reading [this post](https://learnmeabitcoin.com/technical/keys/hd-wallets/).

HD wallets are stored as `KeyVault` files in Bonança, which is explained
in more depth [here](keyvault.md). You can create a `KeyVault` using Rust
or Python.

```rust,ignore
use bonanca::keyvault::KeyVault;

fn main() {
  // Create new KeyVault with english mneomonic
  let key_vault = KeyVault::new("English");
  let filename = Path::new("./keyvault.json");

  // Write json file
  key_vault.write(filename)
}
```

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
