use std::collections::HashMap;

use uuid::Uuid;

use crate::{error::TradeRecordError, models::trade_record::{NewTradeRecord, TradeRecord}};

#[derive(Default)]
pub struct TradeBook {
    records: HashMap<Uuid, TradeRecord>,
}

impl TradeBook {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, new_record: NewTradeRecord) -> Result<Uuid, TradeRecordError> {
        let record = new_record.build()?;
        let id = record.id;
        self.records.insert(id, record);
        Ok(id)
    }

    pub fn get(&self, id: &Uuid) -> Option<&TradeRecord> {
        self.records.get(id)
    }

    pub fn all(&self) -> Vec<&TradeRecord> {
        self.records.values().collect()
    }

    pub fn upsert(&mut self, record: TradeRecord) {
        self.records.insert(record.id, record);
    }

    pub fn remove(&mut self, id: &Uuid) -> Option<TradeRecord> {
        self.records.remove(id)
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}
