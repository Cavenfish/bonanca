use std::str::FromStr;

use alloy::{
    sol,
    sol_types::{SolStruct, SolValue, eip712_domain},
};
use alloy_primitives::{Address, B256, U256, address, keccak256};
use anyhow::Result;
use bonanca_api_lib::defi::cow::{CowApi, CowQuote, CowSwapData, CowSwapOrder};
use bonanca_wallets::wallets::evm::EvmWallet;

sol! {
    #[allow(missing_docs)]
    struct Order {
        address sellToken;
        address buyToken;
        address receiver;
        uint256 sellAmount;
        uint256 buyAmount;
        uint256 validTo;
        bytes32 appData;
        uint256 feeAmount;
        bytes32 kind;
        bool partiallyFillable;
        bytes32 sellTokenBalance;
        bytes32 buyTokenBalance;
    }
}

impl Order {
    pub fn new(quote: &CowQuote) -> Result<Self> {
        Ok(Self {
            sellToken: Address::from_str(&quote.sell_token)?,
            buyToken: Address::from_str(&quote.sell_token)?,
            receiver: Address::from_str(&quote.sell_token)?,
            sellAmount: U256::from(quote.sell_amount),
            buyAmount: U256::from(quote.buy_amount),
            validTo: U256::from(quote.valid_to),
            appData: B256::abi_decode(&quote.app_data.as_bytes())?,
            feeAmount: U256::from(quote.fee_amount),
            kind: keccak256(quote.kind.as_bytes()),
            partiallyFillable: quote.partially_fillable,
            sellTokenBalance: keccak256(quote.sell_token_balance.as_bytes()),
            buyTokenBalance: keccak256(quote.buy_token_balance.as_bytes()),
        })
    }
}

pub struct CoW {
    api: CowApi,
    chain_id: u64,
}

impl CoW {
    pub fn new(chain: &str) -> Result<Self> {
        let api = CowApi::new(chain);

        let chain_id = match chain {
            "mainnet" => 1,
            "bnb" => 56,
            "xdai" => 100,
            "polygon" => 137,
            "lens" => 232,
            "base" => 8453,
            "arbitrum_one" => 42161,
            "avalanche" => 43114,
            "linea" => 59144,
            _ => Err(anyhow::anyhow!("Unsupported chain ID"))?,
        };

        Ok(Self { api, chain_id })
    }

    pub async fn get_swap_quote(
        &self,
        wallet: &EvmWallet,
        sell: &str,
        buy: &str,
        amount: f64,
    ) -> Result<CowSwapOrder> {
        let big_amount = wallet.parse_token_amount(amount, sell).await?;

        let data = CowSwapData {
            sell_token: sell.to_string(),
            buy_token: buy.to_string(),
            sell_amount_before_fee: big_amount,
            kind: "sell".to_string(),
            from: wallet.pubkey.to_string(),
        };

        let quote = self.api.get_swap_quote(&data).await?;

        Ok(quote)
    }

    pub async fn swap(&self, wallet: &EvmWallet, quote: &CowSwapOrder) -> Result<()> {
        let domain = eip712_domain! {
            name: "Gnosis Protocol",
            version: "v2",
            chain_id: self.chain_id,
            verifying_contract: address!("0x9008D19f58AAbD9eD0D60971565AA8510560ab41"),
        };

        let order = Order::new(&quote.quote)?;
        let hash = order.eip712_signing_hash(&domain);
        let sig = wallet.sign_hash(&hash).await?;

        let signed_order = quote.quote.sign(sig.to_string());

        let uid = self.api.post_swap_order(&signed_order).await?;

        println!("{}", uid);

        Ok(())
    }
}
