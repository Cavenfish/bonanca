import bonanca

wbtc = "0x1BFD67037B42Cf73acF2047067bd4F2C47D9BfD6"
usdc = "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359"

# Add your own wallet info
wallet = bonanca.wallets.EvmWallet("", "", 0)

# Change chain_id to match chain you are using
aave = bonanca.defi.AaveV3(1)

# Add your api key and update chain_id
zerox = bonanca.defi.ZeroX("", 1)

# The following 3 transactions assumes all token
# allowances are satisfied, if not you can use
# wallet.approve_token_spending(token, spender, amount)

# borrow wbtc
receipt1 = aave.borrow(wallet, wbtc, 0.002)

# sell wbtc for usdc
receipt2 = zerox.quick_swap(wallet, wbtc, usdc, 0.002)

# deposit usdc in aave
bal = wallet.token_balance(usdc)
receipt3 = aave.supply(wallet, usdc, bal)
