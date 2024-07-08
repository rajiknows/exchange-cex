use crate::{Depth, Order};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Fill {
    pub price: String,
    pub qty: usize,
    pub trade_id: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderPlaced {
    pub order_id: String,
    pub executed_qty: usize,
    pub fills: Vec<Fill>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderCancelled {
    order_id: String,
    executed_qty: usize,
    remaining_qty: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpenOrders {
    orders: Vec<Order>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageToApi {
    Depth { payload: Depth },
    OrderPlaced { payload: OrderPlaced },
    OrderCancelled { payload: OrderCancelled },
    OpenOrders { payload: OpenOrders },
}



