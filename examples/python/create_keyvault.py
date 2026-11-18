import argparse

from bonanca import KeyVault


def main() -> None:
    parser = argparse.ArgumentParser(description="Create a demo keyvault.")
    # NOTE: Passing passwords via command-line arguments is not a secure way to
    # handle secrets (they may be exposed in shell history/process listings).
    # This is included here for demo purposes only.
    parser.add_argument(
        "--password",
        required=True,
        help="Keyvault password (demo only; not a secure way to handle secrets).",
    )
    parser.add_argument("--lang", default="English", help="Mnemonic language.")
    parser.add_argument(
        "--output", default="./keyvault.json", help="Output path for the keyvault file."
    )
    args = parser.parse_args()

    # Create a new keyvault and return its mnemonic.
    keyvault, mnemonic = KeyVault.new(args.lang, args.password)

    # Make a new child key on Solana (example).
    keyvault.make_new_child("Solana")

    # Write keyvault.json file.
    keyvault.write(args.output)

    print(f"Keyvault written to {args.output}")
    print(f"Mnemonic (store securely): {mnemonic}")


if __name__ == "__main__":
    main()
