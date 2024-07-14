use crate::orderbook::*;
use crate::redis_manager;
use crate::redis_manager::TradeAdded;
use crate::typs::from_api::*;

use crate::typs::to_ws::TradeData;
use crate::Kind;
use crate::Market;
use crate::OrderBook;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use self::redis_manager::OrderUpdate;
use self::redis_manager::RedisManager;

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Debug, Clone, Copy)]
pub struct Balance {
    available: usize,
    locked: usize,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Snapshot {
    orderbooks: Vec<OrderBook>,
    balances: HashMap<String, HashMap<String, Balance>>,
}

#[derive(Clone)]
pub struct Engine {
    orderbooks: Vec<OrderBook>,

    // balance should look like this ::

    //    {user_id}, {
    //         "BTC": {
    //             available: 10000000,
    //             locked: 0
    //         },
    //         "TATA": {
    //             available: 10000000,
    //             locked: 0
    //         }
    balances: HashMap<String, HashMap<String, Balance>>,
    redis_manager: Arc<Mutex<RedisManager>>,
}

impl Engine {
    pub fn new() -> Self {
        let path = Path::new("./snapshot.json");
        let mut orderbooks = vec![];
        let mut balances = HashMap::new();

        if path.exists() {
            let mut file = match File::open(&path) {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Failed to open the file: {}", e);
                    return Self {
                        orderbooks,
                        balances,
                        redis_manager: Arc::new(Mutex::new(RedisManager::new().unwrap())),
                    };
                }
            };

            let mut contents = String::new();
            if let Err(e) = file.read_to_string(&mut contents) {
                eprintln!("Failed to read the file: {}", e);
                return Self {
                    orderbooks,
                    balances,
                    redis_manager: Arc::new(Mutex::new(RedisManager::new().unwrap())),
                };
            }

            let snapshot: Snapshot = match serde_json::from_str(&contents) {
                Ok(snapshot) => snapshot,
                Err(e) => {
                    eprintln!("Failed to parse JSON: {}", e);
                    return Self {
                        orderbooks,
                        balances,
                        redis_manager: Arc::new(Mutex::new(RedisManager::new().unwrap())),
                    };
                }
            };

            orderbooks = snapshot.orderbooks;
            balances = snapshot.balances;
        } else {
            orderbooks.push(OrderBook {
                bids: Vec::new(),
                asks: Vec::new(),
                base_asset: String::from("BTC"),
                quote_asset: String::from("USD"),
                last_trade_id: 0,
                current_price: 0,
                bid_depth: HashMap::new(),
                ask_depth: HashMap::new(),
            });

            balances.insert("default_user".to_string(), {
                let mut asset_balances = HashMap::new();
                asset_balances.insert(
                    "BTC".to_string(),
                    Balance {
                        available: 10000000,
                        locked: 0,
                    },
                );
                asset_balances.insert(
                    "TATA".to_string(),
                    Balance {
                        available: 10000000,
                        locked: 0,
                    },
                );
                asset_balances
            });
        }

        let engine = Self {
            orderbooks,
            balances,
            redis_manager: Arc::new(Mutex::new(RedisManager::new().unwrap())), // Initialize RedisManager here
        };
        engine.start_snapshot_save();
        engine
    }

    pub fn start_snapshot_save(&self) {
        let orderbooks = self.orderbooks.clone();
        let balances = self.balances.clone();

        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(3));
            let snapshot = Snapshot {
                orderbooks: orderbooks.clone(),
                balances: balances.clone(),
            };
            match serde_json::to_string(&snapshot) {
                Ok(json) => {
                    let mut file = match OpenOptions::new()
                        .write(true)
                        .create(true)
                        .open("./snapshot.json")
                    {
                        Ok(file) => file,
                        Err(e) => {
                            eprintln!("Failed to open the file for writing: {}", e);
                            continue;
                        }
                    };

                    if let Err(e) = file.write_all(json.as_bytes()) {
                        eprintln!("Failed to write to the file: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to serialize snapshot: {}", e);
                }
            }
        });
    }

    pub fn process(self, message: MessageFromApi, client_id: String) {
        match message {
            MessageFromApi::CreateOrder => {
                todo!()
            }
        }
    }

    pub fn create_order(
        &self,
        market: Market,
        price: usize,
        qty: usize,
        side: Kind,
        userid: String,
    ) -> CreatedOrder {
        let orderbook = self.orderbooks.iter().find(|o| o.ticker() == market);

        if let Some(orderbook) = orderbook {
            let (base_asset, quote_asset) = market.assets();
        }
    }

    pub fn update_db_orders(
        &self,
        order: Order,
        executed_qty: usize,
        fills: Vec<Fills>,
        market: Market,
    ) {
        let redis_manager = self.redis_manager.lock().unwrap();

        let data = OrderUpdate {
            order_id: order.order_id,
            executed_qty,
            market: Some(market),
            price: Some(order.price),
            quantity: Some(order.quantity),
            side: Some(order.side),
        };

        let msg = redis_manager::DbMessage::OrderUpdate { data };
        if let Err(e) = redis_manager.push_message(&msg) {
            eprintln!("Failed to push message to Redis: {}", e);
        }

        fills.iter().map(|fill| {
            let data = OrderUpdate {
                order_id: fill.marker_userid.to_owned(),
                executed_qty: fill.quantity,
                market: None,
                price: None,
                quantity: None,
                side: None,
            };
            let msg = redis_manager::DbMessage::OrderUpdate { data };
            if let Err(e) = redis_manager.push_message(&msg) {
                eprintln!("Failed to push message to Redis: {}", e);
            };
        });
    }

    pub fn create_db_trades(&self, fills: Vec<Fills>, market: Market, user_id: String) {
        let redis_manager = self.redis_manager.lock().unwrap();
        fills.iter().map(|fill| {
            let trade_added = TradeAdded {
                market: market.to_owned(),
                id: fill.tradeid.to_string(),
                is_buyer_maker: fill.other_userid == user_id,
                price: fill.price,
                quantity: fill.quantity,
                quotequantity: &fill.price * &fill.quantity,
                timestamp: 000,
            };

            let msg = redis_manager::DbMessage::TradeAdded { data: trade_added };
            if let Err(e) = redis_manager.push_message(&msg) {
                eprintln!("Failed to push message to Redis: {}", e);
            };
        });
    }

    pub fn publish_ws_trade(fills: Vec<Fills>, market: Market, userid: String) {
        let redis_manager = self.redis_manager.lock().unwrap();
        fills.iter().map(|fill| {
            if let Err(e) = redis_manager.push_message(&msg) {
                eprintln!("Failed to push message to Redis: {}", e);
            };
        });
    }
}

pub struct CreatedOrder {
    executed_qty: usize,
    fills: Vec<Fills>,
    order_id: String,
}
