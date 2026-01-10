# KeyVault

Bonança stores the seed to your HD wallet in what is called a `KeyVault`,
which can be serialized into a `json` file. This takes inspiration from
`keystore` files, if you are unfamiliar with them consider reading
[this post](https://julien-maffre.medium.com/what-is-an-ethereum-keystore-file-86c8c5917b97).

When serialized, your HD wallet seed is encrypted using a password you
define. An example key vault `json` file structure is as follows:

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
  "chain_keys": {
    "Solana": ["AbwHhAquPXvDfxvWEh1b4mG969DQF9wJQSK5k8XKSKtG"],
    "EVM": ["0x50940F0C5779BE15F7ACB12E9b75128e1415BFec"]
  }
}
```

Breaking down the components:

- `cipher` is the encryption algorithm used to encrypt your HD wallet
  seed
- Within `cipher_params` you find `nonce` which is an initialization
  vector for the AES-256-GCM algorithm
- `cipher_text` is your encrypted seed
- `kdf` is the key derivation function used (PBKDF2)
- `kdf_params` are the parameters used in the key derivation function
  - `key_length` is the length in bytes of the key
  - `n` is the number of cycles in the algorithm, where 600,000
    is used to make encrypting/decrypting time consuming to
    prevent brute force attacks
  - `salt` is the password salt
- `mac` is your hashed and salted password using Argon2 (its parameters
  are included in the mac string)
- `salt` again the password salt
- `chain_keys` these are your public keys for various blockchains

Changes to any of these fields can result in failure to decrypt the seed,
and possibly permanent loss of all funds.

The only point of weakness within the `KeyVault` encryption is your choice
of password. If a malicious actor were to get a hold of your `json` file,
they could try to decrypt it by either guessing the password or the seed.
Guessing the seed is extremely difficult and unlikely, whereas guessing
a password is more doable. This means picking a good password is crucial.

`KeyVault` includes your public keys within the `chain_keys` field, which
allows you to view your accounts without having to decrypt your `KeyVault`.
There are positives and negatives to this approach, with the main benefit
being that you can check wallet balances without supplying your password.
The main drawback is that if a malicious actor were to get this file, they
would know how valuable breaking the encryption is.
