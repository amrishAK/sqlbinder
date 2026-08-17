use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QueryResultEnvelope {
    pub count: usize,
    pub items: Vec<QueryResultItem>,
}


#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(transparent)]
pub struct QueryResultItem {
    pub values: BTreeMap<String, QueryResultValue>,
}



#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum QueryResultValue {
    Null,
    String(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
    Bytes(Vec<u8>),
}