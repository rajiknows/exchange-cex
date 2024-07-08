use crate::redis_manager;
use crate::typs::from_api::*;
use crate::typs::*;
use crate::OrderBook;
use crate::{Fills,Order};
use std::collections::HashMap;
use std::fs::{File,OpenOptions};
use std::io::{Read,Write};
use std::path::Path;
use serde::{Serialize,Deserialize};
use std::thread;
use std::time::Duration;


#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Debug, Clone, Copy)]
pub struct Balance {
    available: usize,
    locked: usize,
}

#[derive(Serialize, Deserialize, PartialEq,  Debug, Clone)]
pub struct Snapshot{
    orderbooks: Vec<OrderBook>,
    balances: HashMap<String, HashMap<String, Balance>>,
}


#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
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
                    };
                }
            };

            let mut contents = String::new();
            if let Err(e) = file.read_to_string(&mut contents) {
                eprintln!("Failed to read the file: {}", e);
                return Self {
                    orderbooks,
                    balances,
                };
            }

            let snapshot: Snapshot = match serde_json::from_str(&contents) {
                Ok(snapshot) => snapshot,
                Err(e) => {
                    eprintln!("Failed to parse JSON: {}", e);
                    return Self {
                        orderbooks,
                        balances,
                    };
                }
            };

            orderbooks = snapshot.orderbooks;
            balances = snapshot.balances;
        } else {
            orderbooks.push(OrderBook {
                bids:Vec::new(),
                asks:Vec::new(),
                base_asset: String::from("BTC"),
                quote_asset: String::from("USD"),
                last_trade_id: 00,
                current_price:0,
                bid_depth: HashMap::new(),
                ask_depth:HashMap::new(),
            });

            // Initialize your balances with some default values if needed
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
        };
        engine.start_snapshot_save();
        engine
    }



    pub fn start_snapshot_save(&self) {
        let orderbooks = self.orderbooks.clone();
        let balances = self.balances.clone();

        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(3));
                let snapshot = Snapshot {
                    orderbooks: orderbooks.clone(),
                    balances: balances.clone(),
                };
                match serde_json::to_string(&snapshot) {
                    Ok(json) => {
                        let mut file = match OpenOptions::new().write(true).create(true).open("./snapshot.json") {
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
            }
        });
    }
}
