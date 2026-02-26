from sys import exception
import bonanca
import tomllib
import json
from pathlib import Path
from typing import Dict


class Token:
    """Token information"""

    name: str
    symbol: str
    address: str
    decimals: int

    def __init__(self, token: Dict):
        self.name = token["name"]
        self.symbol = token["symbol"]
        self.address = token["address"]
        self.decimals = token["decimals"]


class TradeSettings:
    """Trade settings for the scalper bot"""

    size: float
    max_orders: int
    delta: float
    profit: float
    expiry: tuple[int, int]

    def __init__(self, settings: Dict):
        self.size = settings["size"]
        self.max_orders = settings["max_orders"]
        self.delta = settings["delta"]
        self.profit = settings["profit"]

        tmp = settings["expiry"].split("-")
        self.expiry = (int(tmp[0]) * 24 + int(tmp[1]), int(tmp[2]))


class Log:
    """Logging for the scalper bot"""

    active_orders: list[str]
    buy_history: Dict
    sell_history: Dict

    def __init__(self, log: Dict):
        self.active_orders = log["active_orders"]
        self.buy_history = log["buy_history"]
        self.sell_history = log["sell_history"]

    def add_buy(self, amount: float, price: float):
        avg_price = (
            self.buy_history["avg_price"] * self.buy_history["bought"] + price * amount
        ) / (self.buy_history["bought"] + amount)

        self.buy_history["bought"] += amount
        self.buy_history["avg_price"] = avg_price

    def add_sell(self, amount: float, price: float):
        avg_price = (
            self.sell_history["avg_price"] * self.sell_history["sold"] + price * amount
        ) / (self.sell_history["sold"] + amount)

        self.sell_history["sold"] += amount
        self.sell_history["avg_price"] = avg_price

    def to_dict(self):
        return {
            "active_orders": self.active_orders,
            "buy_history": self.buy_history,
            "sell_history": self.sell_history,
        }

    def save(self, log_file: Path):
        with open(log_file, "w") as f:
            json.dump(self.to_dict(), f, indent=2)


class Config:
    """Configuration for the scalper bot"""

    chain: str
    rpc_url: str
    keyvault: Path
    child: int
    log_file: Path
    base: Token
    target: Token
    trade_settings: TradeSettings

    def __init__(self, config: Dict):
        self.chain = config["chain"]
        self.rpc_url = config["rpc_url"]
        self.keyvault = Path(config["keyvault"])
        self.child = config["child"]
        self.log_file = Path(config["log_file"])
        self.base = Token(config["base"])
        self.target = Token(config["target"])
        self.trade_settings = TradeSettings(config["trade_settings"])


class Scalper:
    """Scalper bot"""

    def __init__(self, config: Path):
        with open(config, "rb") as io:
            cfg = tomllib.load(io)

        self.config = Config(cfg)
        self.load_dex(self.config.chain)
        self.load_oracle()

        if not self.config.log_file.exists():
            init = {
                "active_orders": [],
                "buy_history": {
                    "bought": 0.0,
                    "avg_price": 0.0,
                },
                "sell_history": {
                    "sold": 0.0,
                    "avg_price": 0.0,
                },
            }
            with open(self.config.log_file, "w") as f:
                json.dump(init, f, indent=2)

            self.log = Log(init)
        else:
            with open(self.config.log_file, "r") as f:
                self.log = Log(json.load(f))

    def load_wallet(self, dry=True):
        if dry:
            self.wallet = bonanca.wallets.EvmWallet.view(
                self.config.keyvault, self.config.rpc_url, self.config.child
            )
        else:
            self.wallet = bonanca.wallets.EvmWallet.load(
                self.config.keyvault, self.config.rpc_url, self.config.child
            )

    def load_dex(self, chain: str):
        try:
            self.dex = bonanca.defi.CoW(chain)
        except exception as e:
            print(f"Error loading DEX: {e}")
            raise

    def load_oracle(self):
        try:
            self.oracle = bonanca.oracle.DefiLlama()
        except exception as e:
            print(f"Error loading Oracle: {e}")
            raise

    def print_balances(self):
        self.load_wallet(self.config.chain)

        base_bal = self.wallet.token_balance(self.config.base.address)
        target_bal = self.wallet.token_balance(self.config.target.address)

        base_usd = self.oracle.get_token_price(
            self.config.base.address, base_bal, self.config.chain
        )
        target_usd = self.oracle.get_token_price(
            self.config.target.address, target_bal, self.config.chain
        )

        print(f"{self.config.base.symbol} balance: {base_bal}")
        print(f"{self.config.target.symbol} balance: {target_bal}")
        print(f"{self.config.base.symbol} value: ${base_usd}")
        print(f"{self.config.target.symbol} value: ${target_usd}")

    def check_spendable_base(self):
        base_bal = self.wallet.token_balance(self.config.base.address)

        expired: list[str] = []
        for uid in self.log.active_orders:
            order = self.dex.get_order_info(uid)

            if (
                order["status"] == "open"
                and order["sell_token"] == self.config.base.address.lower()
            ):
                amount = float(order["sell_amount"]) / (10**self.config.base.decimals)
                executed = float(order["executed_sell_amount"]) / (
                    10**self.config.base.decimals
                )
                base_bal -= amount - executed

            if order["status"] == "expired":
                expired.append(uid)

        self.prune_expired_orders(expired)
        return base_bal

    def set_buy_levels(self, dry=True):
        bal = self.check_spendable_base()
        N = int(bal // self.config.trade_settings.size)

        if N > self.config.trade_settings.max_orders:
            N = self.config.trade_settings.max_orders

        price = self.oracle.get_token_price(
            self.config.target.address, 1.0, self.config.chain
        )

        print(f"Current price: ${price:.4f}")

        dp = price * self.config.trade_settings.delta

        orders: list[str] = []
        for _ in range(N):
            price -= dp
            print(f"Buy level: ${price:.4f}")
            if not dry:
                uid = self.dex.limit_order_by_price(
                    self.wallet,
                    self.config.base.address,
                    self.config.target.address,
                    self.config.trade_settings.size,
                    price,
                    self.config.trade_settings.expiry,
                )
                orders.append(uid)

        self.log_orders(orders)

    def set_sell_levels(self, dry=True):
        if not self.log.active_orders:
            print("No active orders to set sell levels for")
            return

        price = self.oracle.get_token_price(
            self.config.target.address, 1.0, self.config.chain
        )

        trades: list[Dict] = []
        tp_uids: list[str] = []
        for uid in self.log.active_orders:
            order = self.dex.get_order_info(uid)

            if (
                order["status"] == "open"
                and order["sell_token"] == self.config.target.address.lower()
            ):
                got = float(order["executed_sell_amount"])
                want = float(order["sell_amount"])

                if got / want > 0.98:
                    self.log_trade(order)

            if (
                order["status"] == "fulfilled"
                and order["sell_token"] == self.config.target.address.lower()
            ):
                self.log_trade(order)

            if (
                order["status"] == "fulfilled"
                and order["sell_token"] == self.config.base.address.lower()
            ):
                self.set_sell(price, order, trades, tp_uids, dry=dry)

            if (
                order["status"] == "open"
                and order["sell_token"] == self.config.base.address.lower()
            ):
                got = float(order["executed_sell_amount"])
                want = float(order["sell_amount"])

                if got / want > 0.98:
                    self.set_sell(price, order, trades, tp_uids, dry=dry)

        self.log_trades(trades)
        self.log_orders(tp_uids)

    def set_sell(
        self,
        price: float,
        order: Dict,
        trades: list[Dict],
        tp_uids: list[str],
        dry=True,
    ):
        sell_amount = float(order["executed_buy_amount"]) / (
            10**self.config.target.decimals
        )
        buy_amount = (
            float(order["executed_sell_amount"])
            / (10**self.config.base.decimals)
            * (1 + self.config.trade_settings.profit)
        )

        sell_price = buy_amount / sell_amount

        if sell_price < price:
            sell_price = price

        print(f"Sell level: ${sell_price:.4f}")
        if not dry:
            trades.append(order)
            tp_uid = self.dex.limit_order_by_price(
                self.wallet,
                self.config.target.address,
                self.config.base.address,
                sell_amount,
                1 / sell_price,
                self.config.trade_settings.expiry,
            )
            tp_uids.append(tp_uid)

    def log_orders(self, orders: list[str]):
        if not orders:
            return

        self.log.active_orders.extend(orders)
        self.log.save(self.config.log_file)

    def log_trades(self, trades: list[Dict]):
        if not trades:
            return

        for trade in trades:
            self.log_trade(trade)

    def log_trade(self, trade: Dict):
        if trade["sell_token"] == self.config.base.address.lower():
            bought = float(trade["executed_buy_amount"]) / (
                10**self.config.target.decimals
            )
            price = (
                float(trade["executed_sell_amount"])
                / (10**self.config.base.decimals)
                / bought
            )
            self.log.add_buy(bought, price)

        if trade["sell_token"] == self.config.target.address.lower():
            sold = float(trade["executed_sell_amount"]) / (
                10**self.config.target.decimals
            )
            buy_amount = float(trade["executed_buy_amount"]) / (
                10**self.config.base.decimals
            )
            price = buy_amount / sold

            self.log.add_sell(sold, price)

        self.log.active_orders.remove(trade["uid"])
        self.log.save(self.config.log_file)

    def prune_expired_orders(self, expired_uids: list[str]):
        for uid in expired_uids:
            self.log.active_orders.remove(uid)

        self.log.save(self.config.log_file)

    def run(self, dry=True):
        self.load_wallet(dry=dry)
        self.set_buy_levels(dry=dry)
        self.set_sell_levels(dry=dry)


if __name__ == "__main__":
    scalper = Scalper(Path("./scalper.toml"))
    scalper.run(dry=True)
