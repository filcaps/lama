use crate::client::Client;
use crate::exchange::Exchange;
use crate::pair::Pair;
use openlimits::exchange::ExchangeMarketData;
use openlimits::model::OrderBookRequest;
use openlimits::model::OrderBookResponse;
use chrono::Local;
use stopwatch::Stopwatch;
use rust_decimal::Decimal;

//a parser trait for OderBookRequest.
trait Parser {
    fn parse(client: Exchange, pair: &Pair) -> OrderBookRequest;
}

//implements parser trait.
impl Parser for OrderBookRequest {
    fn parse(client: Exchange, pair: &Pair) -> OrderBookRequest {
        match &client {
            Exchange::Binance => match pair {
                Pair::RdntUsdt => OrderBookRequest {market_pair: "RDNTUSDT".to_string()},
            Exchange::Hyperliquid => match pair {
                Pair::RdntUsdc => OrderBookRequest {market_pair: "RDNT".to_string()},
            }
        }
    }
}

#[derive(Clone)]
pub struct Arbitrage {
    client: Client,
    binance_bid: Decimal,
    binance_ask: Decimal,
    hyperliquid_bid: Decimal,
    hyperliquid_ask: Decimal,
    pair: Pair,
}

impl Arbitrage {
    pub async fn new(pair: Pair) -> Self {
        Self {
            client: Client::new().await,
            binance_bid: Decimal::default(),
            binance_ask: Decimal::default(),
            hyperliquid_bid: Decimal::default(),
            hyperliquid_ask: Decimal::default(),
            pair
        }
    }
    
    async fn get_order_book(&self, client: Exchange) -> OrderBookResponse {
        match &client {
            Exchange::Binance => self.client
                                            .binance
                                            .order_book(&OrderBookRequest::parse(client, &self.pair))
                                            .await
                                            .expect("Couldn't get binance order book"),

            Exchange::Hyperliquid => self.client
                                            .hyperliquid.order_book(&OrderBookRequest::parse(client, &self.pair))
                                            .await
                                            .expect("Couldn't get hyperliquid order book")
        }
    }

    async fn update_prices(&mut self) {
        //cloning self to be able to use on the tokio spawn.
        let self_ = self.clone();
        //gets the binance order book.
        let binance_order_book = tokio::spawn(async move { 
            self_.get_order_book(Exchange::Binance)
                .await 
        })
        .await
        .expect("Couldn't get binance order book");
        
        //gets the hyperliquid order book.
        let hyperliquid_order_book = self.get_order_book(Exchange::Hyperliquid).await;

        //gets the last bid and the last ask of binance order book.
        let (binance_last_bid, binance_last_ask) = std::thread::spawn(move || {
            let bid = binance_order_book
                        .bids
                        .into_iter()
                        .last();

            let ask = binance_order_book
                        .asks
                        .into_iter()
                        .last();
            (bid, ask)
        })
        .join()
        .expect("Couldn't get binance bid and ask");

        //gets the last bid and the last ask of hyperliquid order book.
        let (hyperliquid_last_bid, hyperliquid_last_ask) = {
            let bid = hyperliquid_order_book
                        .bids
                        .iter()
                        .last();

            let ask = hyperliquid_order_book
                        .asks
                        .iter()
                        .last();

            (bid, ask)

        };

        //updates the binance bid
        if let Some(bid) = binance_last_bid {
            self.binance_bid = bid.price;
        }
        
        //updates the hyperliquid bid
        if let Some(bid) = hyperliquid_last_bid {
            self.hyperliquid_bid = bid.price;
        }
    
        //updates the binance ask.
        if let Some(ask) = binance_last_ask {
             self.binance_ask = ask.price;
        }

        //updates the hyperliquid ask.                   
        if let Some(ask) = coinbase_last_ask {
            self.hyperliquid_ask = ask.price;
        }
    }

    pub async fn looks_for_opportunities(&mut self) {
        //creates a chronometer to measure the interval time for show "seeking arbitrage opportunity" message in the screen.
        let mut chronometer = Stopwatch::new();
        chronometer.start();

        loop {
            //updates prices
            self.update_prices().await;
            //gets the local time
            let local_time = Local::now().format("%c").to_string();

            //checks whether the bid price on binance is higher than the ask price on hyperliquid.
            if self.binance_bid > self.hyperliquid_ask {         
                let profit = &self.binance_bid - &self.hyperliquid_ask;
                println!("{}\nArbitrage opportunity found \nBuy: {} at {} on hyperliquid \nSell:  {} at {} on binance \nProfit: {}\n", 
                    local_time,
                    self.pair_as_str(),
                    self.hyperliquid_ask,
                    self.pair_as_str(),
                    self.binance_bid,
                    profit
                );

            //checks whether the bid price on hyperliquid is higher than the ask price on binance.
            } else if self.hyperliquid_bid > self.binance_ask {
                let profit = &self.hyperliquid_bid - &self.binance_ask;
                println!("{}\nArbitrage opportunity found \nBuy: {} at {} on binance \nSell:  {} at {} on hyperliquid \nProfit: {}\n", 
                    local_time,
                    self.pair_as_str(),
                    self.binance_ask,
                    self.pair_as_str(),
                    self.hyperliquid_bid,
                    profit
                );
            } 
        }
    }
        }
    }
}
