use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Debug, Clone)]
pub enum Status {
    Accepted,
    Rejected,
}

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Debug, Clone, Copy)]
pub enum Kind {
    BUY,
    SELL,
}

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Debug, Clone)]
pub enum Market {
    TataInr,
    GoogleDollar,
    NvidiaInr,
    TeslaDollar,
}
impl Market {
    pub fn assets(&self) -> (&str, &str) {
        match self {
            Market::TataInr => ("TATA", "INR"),
            Market::GoogleDollar => ("GOOGLE", "DOLLAR"),
            Market::NvidiaInr => ("NVIDIA", "INR"),
            Market::TeslaDollar => ("TESLA", "DOLLAR"),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Debug, Clone)]
pub enum FillStatus {
    Unfilled,
    PartiallyFilled,
    Filled,
}

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Debug, Clone)]
pub struct Order {
    pub order_id: String,
    pub price: usize,
    pub quantity: usize,
    pub filled: usize,
    pub side: Kind,
    user_id: String,
}

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Debug, Clone)]
pub struct Bid {
    pub order: Order,
    pub side: Kind,
}

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Debug, Clone)]
pub struct Ask {
    pub order: Order,
    pub side: Kind,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Depth {
    pub bid_depth: HashMap<usize, usize>,
    pub ask_depth: HashMap<usize, usize>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Fillresult {
    pub _status: FillStatus,
    pub executedqty: usize,
    pub fills: Vec<Fills>,
    depth: Depth,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct OrderBook {
    pub bids: Vec<Bid>,
    pub asks: Vec<Ask>,
    pub base_asset: String,
    pub quote_asset: String,
    pub last_trade_id: usize,
    pub current_price: usize,
    pub bid_depth: HashMap<usize, usize>, // Price to total quantity for bids
    pub ask_depth: HashMap<usize, usize>,
}

impl OrderBook {
    pub fn new(
        base_asset: String,
        bids: Vec<Bid>,
        asks: Vec<Ask>,
        last_trade_id: usize,
        current_price: usize,
    ) -> Self {
        OrderBook {
            bids,
            asks,
            base_asset,
            quote_asset: String::from("INR"),
            last_trade_id,
            current_price,
            bid_depth: HashMap::new(),
            ask_depth: HashMap::new(),
        }
    }

    pub fn ticker(&self) -> String {
        format!("{}_{}", &self.base_asset, &self.quote_asset)
    }

    pub fn getsnapshot(&self) -> Self {
        self.clone()
    }

    pub fn add_order(&mut self, order: &mut Order) -> Fillresult {
        let fill_result = match order.side {
            Kind::BUY => self.match_bid(order.clone()),
            Kind::SELL => self.match_ask(order.clone()),
        };

        order.filled = fill_result.executedqty;

        if fill_result.executedqty < order.quantity {
            match order.side {
                Kind::BUY => self.bids.push(Bid {
                    order: order.clone(),
                    side: Kind::BUY,
                }),
                Kind::SELL => self.asks.push(Ask {
                    order: order.clone(),
                    side: Kind::SELL,
                }),
            }
        }

        fill_result
    }

    pub fn match_bid(&mut self, order: Order) -> Fillresult {
        let mut fills: Vec<Fills> = Vec::new();
        let mut executed_qty: usize = 0;
        let mut to_remove = Vec::new();

        for (i, ask) in self.asks.iter_mut().enumerate() {
            if ask.order.price <= order.price && executed_qty < order.quantity {
                let filled_qty = std::cmp::min(
                    order.quantity - executed_qty,
                    ask.order.quantity - ask.order.filled,
                );
                executed_qty += filled_qty;
                ask.order.filled += filled_qty;

                fills.push(Fills {
                    price: ask.order.price,
                    quantity: filled_qty,
                    tradeid: self.last_trade_id + 1,
                    other_userid: ask.order.user_id.clone(),
                    marker_userid: order.order_id.clone(),
                });

                if ask.order.filled == ask.order.quantity {
                    to_remove.push(i);
                }

                // Update ask depth
                *self.ask_depth.entry(ask.order.price).or_insert(0) -= filled_qty;
                if self.ask_depth[&ask.order.price] == 0 {
                    self.ask_depth.remove(&ask.order.price);
                }
            }
        }

        for &i in to_remove.iter().rev() {
            self.asks.remove(i);
        }

        Fillresult {
            fills,
            executedqty: executed_qty,
            _status: if executed_qty == order.quantity {
                FillStatus::Filled
            } else if executed_qty > 0 {
                FillStatus::PartiallyFilled
            } else {
                FillStatus::Unfilled
            },
            depth: Depth {
                bid_depth: (self.bid_depth.clone()),
                ask_depth: (self.ask_depth.clone()),
            },
        }
    }

    pub fn match_ask(&mut self, order: Order) -> Fillresult {
        let mut fills: Vec<Fills> = Vec::new();
        let mut executed_qty: usize = 0;
        let mut to_remove = Vec::new();

        for (i, bid) in self.bids.iter_mut().enumerate() {
            if bid.order.price >= order.price && executed_qty < order.quantity {
                let filled_qty = std::cmp::min(
                    order.quantity - executed_qty,
                    bid.order.quantity - bid.order.filled,
                );
                executed_qty += filled_qty;
                bid.order.filled += filled_qty;

                fills.push(Fills {
                    price: bid.order.price,
                    quantity: filled_qty,
                    tradeid: self.last_trade_id + 1,
                    other_userid: bid.order.user_id.clone(),
                    marker_userid: order.order_id.clone(),
                });

                if bid.order.filled == bid.order.quantity {
                    to_remove.push(i);
                }

                // Update bid depth
                *self.bid_depth.entry(bid.order.price).or_insert(0) -= filled_qty;
                if self.bid_depth[&bid.order.price] == 0 {
                    self.bid_depth.remove(&bid.order.price);
                }
            }
        }

        for &i in to_remove.iter().rev() {
            self.bids.remove(i);
        }

        Fillresult {
            fills,
            executedqty: executed_qty,
            _status: if executed_qty == order.quantity {
                FillStatus::Filled
            } else if executed_qty > 0 {
                FillStatus::PartiallyFilled
            } else {
                FillStatus::Unfilled
            },
            depth: Depth {
                bid_depth: (self.bid_depth.clone()),
                ask_depth: (self.ask_depth.clone()),
            },
        }
    }

    pub fn get_open_orders(&self, user_id: &str) -> Vec<Order> {
        let asks: Vec<Order> = self
            .asks
            .iter()
            .filter(|x| x.order.user_id == user_id)
            .map(|x| x.order.clone())
            .collect();

        let bids: Vec<Order> = self
            .bids
            .iter()
            .filter(|x| x.order.user_id == user_id)
            .map(|x| x.order.clone())
            .collect();

        [asks, bids].concat()
    }

    pub fn cancel_bid(&mut self, order: &Order) -> Option<usize> {
        if let Some(index) = self
            .bids
            .iter()
            .position(|x| x.order.order_id == order.order_id)
        {
            let price = self.bids[index].order.price;
            self.bids.remove(index);
            Some(price)
        } else {
            None
        }
    }

    pub fn cancel_ask(&mut self, order: &Order) -> Option<usize> {
        if let Some(index) = self
            .asks
            .iter()
            .position(|x| x.order.order_id == order.order_id)
        {
            let price = self.asks[index].order.price;
            self.asks.remove(index);
            Some(price)
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd)]
pub struct OrderInputSchema {
    pub base_asset: String,
    pub quote_asset: String,
    pub price: usize,
    pub quantity: usize,
    pub side: Kind,
    // pub typ: OrderType,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BookWithQuantity {
    pub bids: HashMap<usize, usize>,
    pub asks: HashMap<usize, usize>,
}

impl BookWithQuantity {
    pub fn new() -> BookWithQuantity {
        BookWithQuantity {
            bids: HashMap::new(),
            asks: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Debug, Clone)]
pub struct Fills {
    pub price: usize,
    pub quantity: usize,
    pub tradeid: usize,
    pub other_userid: String,
    pub marker_userid: String,
}

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Debug, Clone)]
pub enum Status {
    Accepted,
    Rejected,
}
