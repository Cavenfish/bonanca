use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use alloy::{
    sol,
    sol_types::{SolStruct, eip712_domain},
};
use alloy_primitives::{Address, FixedBytes, U256, address, keccak256};
use anyhow::Result;
use bonanca_api_lib::defi::cow::{CowApi, CowQuote, CowSwapData, CowSwapOrder, CowSwapPlacedOrder};
use bonanca_wallets::wallets::evm::EvmWallet;

sol! {
    #[allow(missing_docs)]
    struct Order {
        address sellToken;
        address buyToken;
        address receiver;
        uint256 sellAmount;
        uint256 buyAmount;
        uint32 validTo;
        bytes32 appData;
        uint256 feeAmount;
        string kind;
        bool partiallyFillable;
        string sellTokenBalance;
        string buyTokenBalance;
    }
}

impl Order {
    pub fn new(quote: &CowQuote) -> Result<Self> {
        Ok(Self {
            sellToken: Address::from_str(&quote.sell_token)?,
            buyToken: Address::from_str(&quote.buy_token)?,
            receiver: Address::from_str(&quote.receiver.as_ref().unwrap())?,
            sellAmount: U256::from(quote.sell_amount),
            buyAmount: U256::from(quote.buy_amount),
            validTo: quote.valid_to,
            appData: keccak256(quote.app_data.as_bytes()),
            feeAmount: U256::from(quote.fee_amount),
            kind: quote.kind.clone(),
            partiallyFillable: quote.partially_fillable,
            sellTokenBalance: quote.sell_token_balance.clone(),
            buyTokenBalance: quote.buy_token_balance.clone(),
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

    fn get_signing_hash(&self, order: Order) -> Result<FixedBytes<32>> {
        let domain = eip712_domain! {
            name: "Gnosis Protocol",
            version: "v2",
            chain_id: self.chain_id,
            verifying_contract: address!("0x9008D19f58AAbD9eD0D60971565AA8510560ab41"),
        };

        let eip712_hash = order.eip712_signing_hash(&domain);

        Ok(eip712_hash)
    }

    pub async fn get_order_info(&self, uid: &str) -> Result<CowSwapPlacedOrder> {
        self.api.get_order_info(uid).await
    }

    pub async fn get_user_orders(
        &self,
        user: &str,
        limit: Option<u16>,
    ) -> Result<Vec<CowSwapPlacedOrder>> {
        self.api.get_user_orders(user, limit).await
    }

    pub async fn get_market_quote(
        &self,
        wallet: &EvmWallet,
        sell: &str,
        buy: &str,
        amount: f64,
    ) -> Result<CowSwapOrder> {
        let taker = wallet.pubkey.to_string();
        let big_amount = wallet.format_token(amount, sell).await?;

        let data = CowSwapData {
            sell_token: sell.to_string(),
            buy_token: buy.to_string(),
            sell_amount_before_fee: big_amount,
            kind: "sell".to_string(),
            from: taker.clone(),
            receiver: taker,
            app_data: "{}".to_string(), // hash below is keccak256("{}")
            app_data_hash: "0xb48d38f93eaa084033fc5970bf96e559c33c4cdc07d889ab00b4d63f9590739d"
                .to_string(),
        };

        let quote = self.api.get_swap_quote(&data).await?;

        Ok(quote)
    }

    pub async fn post_market_order(
        &self,
        wallet: &EvmWallet,
        quote: CowSwapOrder,
    ) -> Result<String> {
        let order = Order::new(&quote.quote)?;
        let hash = self.get_signing_hash(order)?;
        let sig = wallet.sign_hash(&hash).await?;
        let signed_order = quote.sign(sig.to_string());

        let uid = self.api.post_swap_order(&signed_order).await?;

        Ok(uid)
    }

    pub async fn limit_order(
        &self,
        wallet: &EvmWallet,
        sell: &str,
        buy: &str,
        sell_amount: f64,
        buy_amount: f64,
        lifetime: Duration,
    ) -> Result<String> {
        let taker = wallet.pubkey.to_string();
        let sell_amnt = wallet.format_token(sell_amount, sell).await?;
        let buy_amnt = wallet.format_token(buy_amount, buy).await?;
        let valid_to = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + lifetime.as_secs();

        let quote = CowQuote::build_quote(
            sell,
            buy,
            Some(taker.clone()),
            sell_amnt,
            buy_amnt,
            valid_to as u32,
        );

        let order = Order::new(&quote)?;
        let hash = self.get_signing_hash(order)?;
        let sig = wallet.sign_hash(&hash).await?;
        let signed_order = quote.sign(sig.to_string(), taker);

        let uid = self.api.post_swap_order(&signed_order).await?;

        Ok(uid)
    }

    pub async fn limit_order_by_price(
        &self,
        wallet: &EvmWallet,
        sell: &str,
        buy: &str,
        amount: f64,
        price: f64,
        lifetime: Duration,
    ) -> Result<String> {
        let taker = wallet.pubkey.to_string();
        let sell_amount = wallet.format_token(amount, sell).await?;
        let buy_amount = wallet.format_token(amount / price, buy).await?;
        let valid_to = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + lifetime.as_secs();

        let quote = CowQuote::build_quote(
            sell,
            buy,
            Some(taker.clone()),
            sell_amount,
            buy_amount,
            valid_to as u32,
        );

        let order = Order::new(&quote)?;
        let hash = self.get_signing_hash(order)?;
        let sig = wallet.sign_hash(&hash).await?;
        let signed_order = quote.sign(sig.to_string(), taker);

        let uid = self.api.post_swap_order(&signed_order).await?;

        Ok(uid)
    }
}
