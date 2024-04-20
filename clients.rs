use openlimits::binance::Binance;
use openlimits::binance::BinanceParameters;
use openlimits::coinbase::Hyperliquid;
use openlimits::coinbase::HyperliquidParameters;
use openlimits::exchange::Exchange;

#[derive(Clone)]
pub struct Client {
    pub binance: Binance,
    pub hyperliquid: Hyperliquid
}

impl Client {
    pub async fn new() -> Self {
        Self {
            binance: Binance::new(BinanceParameters::prod())
                        .await
                        .expect("Couldn't create binance client"),

            hyperliquid: Hyperliquid::new(HyperliquidParameters::prod())
                        .await
                        .expect("Couldn't create hyperliquod client"),
        }
    }
}
