use serde::{Deserialize, Serialize};

// Default value functions
fn default_ticker_event() -> String {
    "ticker".to_string()
}

fn default_depth_event() -> String {
    "depth".to_string()
}

fn default_trade_event() -> String {
    "trade".to_string()
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct TickerUpdateMessage {
    pub stream: String,
    pub data: TickerData,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct TickerData {
    pub c: Option<String>,
    pub h: Option<String>,
    pub l: Option<String>,
    pub v: Option<String>,
    pub v_2: Option<String>,
    pub s: Option<String>,
    pub id: usize,
    #[serde(default = "default_ticker_event")]
    pub e: String, // "ticker"
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct DepthUpdateMessage {
    pub stream: String,
    pub data: DepthData,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct DepthData {
    pub b: Option<Vec<(String, String)>>,
    pub a: Option<Vec<(String, String)>>,
    #[serde(default = "default_depth_event")]
    pub e: String, // "depth"
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct TradeAddedMessage {
    pub stream: String,
    pub data: TradeData,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct TradeData {
    #[serde(default = "default_trade_event")]
    pub e: String, // "trade"
    pub t: usize,
    pub m: bool,
    pub p: String,
    pub q: String,
    pub s: String, // symbol
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum WsMessage {
    TickerUpdateMessage { data: TickerData },
    DepthUpdateMessage { data: DepthData },
    TradeAddedMessage { data: TradeData },
}
