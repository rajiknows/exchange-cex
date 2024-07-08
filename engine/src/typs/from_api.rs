use serde::{Serialize,Deserialize};

use crate::{Kind, Market};
#[derive(Serialize, Deserialize, Debug, Clone)]

pub enum MessageFromApi {
    CreateOrder { data: CreateOrder },
    CancelOrder { data: CancelOrder },
    OnRamp { data: OnRamp },
    GetDepth { data: GetDepth },
    GetOpenOrders { data: GetOpenOrders },
}

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct CreateOrder {
    market: Market,
    price: usize,
    quantity: usize,
    side: Kind,
    user_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CancelOrder {
    order_id: String,
    market: Market,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OnRamp {
    amount: usize,
    user_id: String,
    txn_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetDepth {
    market: Market,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetOpenOrders {
    user_id: String,
    market: Market,
}
