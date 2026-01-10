pub mod hd_keys;
pub mod keyvault;
mod utils;

#[cfg(test)]
mod tests {
    use super::hd_keys::HDkeys;
    use super::keyvault::KeyVault;
    use super::utils::{decrypt_seed, verify_password};
    use argon2::password_hash::SaltString;

    #[test]
    fn test_encrypt_and_decrypt() {
        let password = "password";
        let mnemonic = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong";
        let hd_keys = HDkeys::from_mnemonic(mnemonic);

        let seed_hex = "b6a6d8921942dd9806607ebc2750416b289adea669198769f2e15ed926c3aa92bf88ece232317b4ea463e84b0fcd3b53577812ee449ccc448eb45e6f544e25b6";
        let seed: [u8; 64] = hex::decode(seed_hex)
            .unwrap()
            .as_slice()
            .try_into()
            .unwrap();

        assert_eq!(hd_keys.seed, seed);
        assert_eq!(hex::encode(hd_keys.seed), seed_hex);

        let salt = SaltString::from_b64("M6lWvNAGuZBSp9fBGAUEqw").unwrap();
        let nonce = "287189f34a1433d2de201d08";

        let keyvault = hd_keys.get_keyvault(&salt, nonce, password).unwrap();

        assert!(verify_password(&keyvault.vault.mac, password));

        let decrypted_seed = decrypt_seed(
            &keyvault.vault.cipher_text,
            password,
            &keyvault.vault.cipher_params.nonce,
            &keyvault.vault.kdf_params,
        )
        .unwrap();

        assert_eq!(hd_keys.seed, decrypted_seed);
    }

    #[test]
    fn test_decrypt_keyvault() {
        let keyvault: KeyVault = serde_json::from_str(
            r#"
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
            "#,
        ).unwrap();

        let password = "password";
        let seed_hex = "b6a6d8921942dd9806607ebc2750416b289adea669198769f2e15ed926c3aa92bf88ece232317b4ea463e84b0fcd3b53577812ee449ccc448eb45e6f544e25b6";
        let seed: [u8; 64] = hex::decode(seed_hex)
            .unwrap()
            .as_slice()
            .try_into()
            .unwrap();

        assert!(verify_password(&keyvault.vault.mac, password));

        let decrypted_seed = decrypt_seed(
            &keyvault.vault.cipher_text,
            password,
            &keyvault.vault.cipher_params.nonce,
            &keyvault.vault.kdf_params,
        )
        .unwrap();

        assert_eq!(seed, decrypted_seed);
    }
}
