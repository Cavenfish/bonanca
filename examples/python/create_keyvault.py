from bonanca import KeyVault

# Create a new keyvault with an English mneomonic
keyvault = KeyVault.new("English")

# Other language options are:
# Simplified Chinese, Traditional Chinese,
# French, Italian, Japanese, Korean, and Spanish

# Get pubkey dictionary
pubkeys = keyvault.chain_keys()

# Make new child key on Solana
keyvault.make_new_child("Solana")

# Write keyvault.json file
keyvault.write("./keyvault.json")
