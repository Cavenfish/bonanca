# Wallet creation

Bonança requires a wallet to manage your index fund. You can create a new wallet using the following command:

```bash
bonanca create -f path/to/keyvault.json
```

This command will generate a new wallet and store the necessary keys in the specified key vault file. During the creation process, you will be prompted to set a password for the key vault to ensure its security. This
password will be required whenever you access the wallet in the future.

Bonança creates a master key for an HD (Hierarchical Deterministic) wallet, which can generate multiple child keys for different blockchains. This means you can manage multiple wallets for multiple blockchains (e.g., Solana, EVM-compatible chains) using a single key vault file.

Bonança encrypts the key vault file using the password you provide, so even if someone gains access to the file, they will not be able to use it without the password. Although the encryption methods are strong and secure, brute force attacks are still possible if a weak password is chosen. Therefore, it is crucial to select a strong and unique password for your key vault.

An example key vault file structure is as follows:

```json
{
  "vault": {
    "cipher": "aes256-gcm",
    "cipher_params": { "nonce": "287189f34a1433d2de201d08" },
    "cipher_text": "7a34170003c0a7b3ccb75bac28757801a7d9b5e1ff062afa4af5f3c03e7d8982eb1f36ccce87436e42b44ffea6bcf39eba8c15d4e79ee0bf012811fca81ae1e112c0f8ae5d8e43ac8cad1ae961b11207",
    "kdf": "pbkdf2",
    "kdf_params": {
      "key_length": 32,
      "n": 600000,
      "salt": "M6lWvNAGuZBSp9fBGAUEqw"
    },
    "mac": "$argon2id$v=19$m=19456,t=2,p=1$M6lWvNAGuZBSp9fBGAUEqw$/U5VYPmg3+BQj0ttOyPnOUjH7bP23V9/tgvBpovna/8",
    "salt": "M6lWvNAGuZBSp9fBGAUEqw"
  },
  "chain_keys": [
    {
      "chain": "Solana",
      "public_keys": ["AbwHhAquPXvDfxvWEh1b4mG969DQF9wJQSK5k8XKSKtG"]
    },
    {
      "chain": "EVM",
      "public_keys": ["0x50940F0C5779BE15F7ACB12E9b75128e1415BFec"]
    }
  ]
}
```

This shows that the master key is encrypted using AES-256-GCM and the key derivation function used is PBKDF2 with 600,000 iterations. Your password is not stored in the file, only a hash of it (the `mac` field) is stored for verification purposes. The hashing algorithm used is Argon2id, which is a secure password hashing algorithm. The `salt` fields are random values used to enhance security during encryption and hashing, similarly the `nonce` field is a random value used during encryption. Changes to any of these fields will result in failure to decrypt the wallet.

After creating the wallet, you can use it to manage your index fund, including depositing assets, withdrawing funds, and executing trades.
