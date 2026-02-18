# View vs Load

Bonança offers you the ability to interact with wallets in two modes; `view` and
`load`. In `view` mode the wallet is only the public key, whereas, `load` has
both public and private keys. This is useful for when you want to check token
balances without having to expose your private key or type your password in.

### Rust

In Rust `view` and `load` are both methods of the wallet struct.

```rust,ignore
use bonanca::wallets::{EvmWallet, SolWallet}
use std::path::Path;

fn main() {
    let filename = Path::new("./keyvault.json");
    let child = 0;

    let evm_wallet_view = EvmWallet::view(filename, "rpc_url", child)
    let sol_wallet_view = SolWallet::view(filename, "rpc_url", child)

    let evm_wallet_load = EvmWallet::load(filename, "rpc_url", child)
    let sol_wallet_load = SolWallet::load(filename, "rpc_url", child)
}
```

### Python

In Python the dynamic style `view` and `load` isn't possible, so instead there
are two different classes for each method.

```python
import bonanca

wallet_view = bonanca.wallets.EvmWalletView("keyvault.json", "rpc_url", 0)

wallet_load = bonanca.wallets.EvmWallet("keyvault.json", "rpc_url", 0)
```
