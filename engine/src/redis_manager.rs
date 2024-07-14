use crate::{Kind, Market};
use serde::{Deserialize, Serialize};
const URL: &str = "rediss://127.0.0.1/";
use crate::typs::to_api::MessageToApi;
use crate::typs::to_ws::WsMessage;
use redis::Commands;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DbMessage {
    TradeAdded { data: TradeAdded },
    OrderUpdate { data: OrderUpdate },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TradeAdded {
    pub id: String,
    pub is_buyer_maker: bool,
    pub price: usize,
    pub quantity: usize,
    pub quotequantity: usize,
    pub timestamp: usize,
    pub market: Market,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderUpdate {
    pub order_id: String,
    pub executed_qty: usize,
    pub market: Option<Market>,
    pub price: Option<usize>,
    pub quantity: Option<usize>,
    pub side: Option<Kind>,
}

pub struct RedisManager {
    client: redis::Client,
}

impl RedisManager {
    pub fn new() -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(URL)?;
        Ok(RedisManager { client })
    }

    pub fn connect(&self) -> Result<redis::Connection, redis::RedisError> {
        self.client.get_connection()
    }

    pub fn push_message(&self, msg: &DbMessage) -> Result<(), redis::RedisError> {
        let mut conn = self.connect().unwrap();
        let serialized_message = serde_json::to_string(&msg).unwrap();
        let _ = conn.lpush("db_processor", serialized_message);
        Ok(())
    }

    pub fn publish_message(
        &self,
        channel: String,
        msg: &WsMessage,
    ) -> Result<(), redis::RedisError> {
        let mut conn = self.connect().unwrap();
        let serialized_message = serde_json::to_string(&msg).unwrap();
        conn.publish(channel, serialized_message)?;
        Ok(())
    }

    pub fn send_to_api(
        &self,
        client_id: String,
        msg: &MessageToApi,
    ) -> Result<(), redis::RedisError> {
        let mut conn = self.connect().unwrap();
        let serialized_message = serde_json::to_string(&msg).unwrap();
        conn.publish(client_id, serialized_message)?;
        Ok(())
    }
}
