use bip39::{Language, Mnemonic};
use ed25519_dalek_bip32::{DerivationPath, ExtendedSigningKey};
use solana_sdk::signer::{Signer, keypair::Keypair};

pub fn get_solana_hd_wallet() {
    let mut rng = bip39::rand::thread_rng();
    let m = Mnemonic::generate_in_with(&mut rng, Language::English, 12).unwrap();

    println!("Mnemonic: {}\n", m);

    let passphrase = "test";
    let seed = m.to_seed_normalized(passphrase);

    let master_key = ExtendedSigningKey::from_seed(&seed).expect("Failed to create master key");

    // println!("Master Key: {:?}", master_key);

    for i in 1..5 {
        // Solana BIP44 path: m/44'/501'/i'/0'
        // m/44'/coin'/account'/change/address
        // let derivation_path = DerivationPath::bip44(501, i, 0, 0).unwrap();
        let derivation_path: DerivationPath = format!("m/44'/501'/{}'/0'/0'", i).parse().unwrap();

        println!("{:?}", derivation_path);

        // Derive child key
        let derived_key = master_key
            .derive(&derivation_path)
            .expect("Failed to derive child key");

        // Convert to Solana keypair
        let secret_key = derived_key.signing_key;

        // println!("{:?}", secret_key.to_bytes());

        let keypair = Keypair::new_from_array(secret_key.to_bytes());

        let pubkey = keypair.pubkey();

        println!("Account {}: {}", i, pubkey);
        println!("  Base58: {}\n", pubkey.to_string());
    }
}
