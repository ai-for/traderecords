use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::TradeRecordError;

const MIN_BUY_REASONS: usize = 3;
const MIN_SELL_REASONS: usize = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TradeRecord {
    pub id: Uuid,
    pub symbol: String,
    pub quantity: Decimal,
    pub buy_price: Decimal,
    pub sell_price: Option<Decimal>,
    pub buy_reasons: Vec<String>,
    pub sell_reasons: Vec<String>,
    pub take_profit_price: Decimal,
    pub stop_loss_price: Decimal,
    pub opened_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTradeRecord {
    pub symbol: String,
    pub quantity: Decimal,
    pub buy_price: Decimal,
    pub sell_price: Option<Decimal>,
    pub buy_reasons: Vec<String>,
    pub sell_reasons: Vec<String>,
    pub take_profit_price: Decimal,
    pub stop_loss_price: Decimal,
    pub opened_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

impl NewTradeRecord {
    pub fn build(self) -> Result<TradeRecord, TradeRecordError> {
        let buy_reasons = sanitize_reasons(self.buy_reasons, MIN_BUY_REASONS, ReasonKind::Buy)?;
        let sell_reasons = sanitize_reasons(self.sell_reasons, MIN_SELL_REASONS, ReasonKind::Sell)?;

        if self.take_profit_price <= Decimal::ZERO {
            return Err(TradeRecordError::InvalidTakeProfit);
        }

        if self.stop_loss_price <= Decimal::ZERO {
            return Err(TradeRecordError::InvalidStopLoss);
        }

        Ok(TradeRecord {
            id: Uuid::new_v4(),
            symbol: self.symbol,
            quantity: self.quantity,
            buy_price: self.buy_price,
            sell_price: self.sell_price,
            buy_reasons,
            sell_reasons,
            take_profit_price: self.take_profit_price,
            stop_loss_price: self.stop_loss_price,
            opened_at: self.opened_at,
            closed_at: self.closed_at,
            notes: self.notes,
        })
    }
}

impl TradeRecord {
    pub fn update_buy_reasons(&mut self, reasons: Vec<String>) -> Result<(), TradeRecordError> {
        self.buy_reasons = sanitize_reasons(reasons, MIN_BUY_REASONS, ReasonKind::Buy)?;
        Ok(())
    }

    pub fn update_sell_reasons(&mut self, reasons: Vec<String>) -> Result<(), TradeRecordError> {
        self.sell_reasons = sanitize_reasons(reasons, MIN_SELL_REASONS, ReasonKind::Sell)?;
        Ok(())
    }

    pub fn update_take_profit_price(&mut self, price: Decimal) -> Result<(), TradeRecordError> {
        if price <= Decimal::ZERO {
            return Err(TradeRecordError::InvalidTakeProfit);
        }
        self.take_profit_price = price;
        Ok(())
    }

    pub fn update_stop_loss_price(&mut self, price: Decimal) -> Result<(), TradeRecordError> {
        if price <= Decimal::ZERO {
            return Err(TradeRecordError::InvalidStopLoss);
        }
        self.stop_loss_price = price;
        Ok(())
    }
}

#[derive(Copy, Clone)]
enum ReasonKind {
    Buy,
    Sell,
}

fn sanitize_reasons(
    reasons: Vec<String>,
    minimum: usize,
    kind: ReasonKind,
) -> Result<Vec<String>, TradeRecordError> {
    let mut sanitized = Vec::with_capacity(reasons.len());

    for (index, reason) in reasons.into_iter().enumerate() {
        let trimmed = reason.trim().to_string();
        if trimmed.is_empty() {
            return Err(TradeRecordError::EmptyReason { index });
        }
        sanitized.push(trimmed);
    }

    if sanitized.len() < minimum {
        return match kind {
            ReasonKind::Buy => Err(TradeRecordError::NotEnoughBuyReasons {
                required: minimum,
                provided: sanitized.len(),
            }),
            ReasonKind::Sell => Err(TradeRecordError::NotEnoughSellReasons {
                required: minimum,
                provided: sanitized.len(),
            }),
        };
    }

    Ok(sanitized)
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::TimeZone;
    use rust_decimal::Decimal;

    fn sample_record() -> NewTradeRecord {
        NewTradeRecord {
            symbol: "AAPL".into(),
            quantity: Decimal::new(10, 0),
            buy_price: Decimal::new(15000, 2),
            sell_price: None,
            buy_reasons: vec![
                "估值偏低".into(),
                "财报超预期".into(),
                "技术面突破".into(),
            ],
            sell_reasons: vec!["目标价达到".into()],
            take_profit_price: Decimal::new(18000, 2),
            stop_loss_price: Decimal::new(13000, 2),
            opened_at: Utc.with_ymd_and_hms(2023, 1, 1, 9, 30, 0).unwrap(),
            closed_at: None,
            notes: None,
        }
    }

    #[test]
    fn build_trade_record_success() {
        let record = sample_record().build().expect("record should be valid");
        assert_eq!(record.buy_reasons.len(), 3);
        assert_eq!(record.sell_reasons.len(), 1);
    }

    #[test]
    fn reject_when_buy_reasons_less_than_three() {
        let mut new_record = sample_record();
        new_record.buy_reasons = vec!["只有一个理由".into()];

        let error = new_record.build().expect_err("should fail");

        assert_eq!(
            error,
            TradeRecordError::NotEnoughBuyReasons {
                required: MIN_BUY_REASONS,
                provided: 1,
            }
        );
    }

    #[test]
    fn reject_empty_reason_entries() {
        let mut new_record = sample_record();
        new_record.buy_reasons = vec![
            "  ".into(),
            "财报超预期".into(),
            "技术面突破".into(),
        ];

        let error = new_record.build().expect_err("should fail");

        assert_eq!(error, TradeRecordError::EmptyReason { index: 0 });
    }

    #[test]
    fn reject_invalid_take_profit_price() {
        let mut new_record = sample_record();
        new_record.take_profit_price = Decimal::new(-1, 0);

        let error = new_record.build().expect_err("should fail");

        assert_eq!(error, TradeRecordError::InvalidTakeProfit);
    }

    #[test]
    fn reject_invalid_stop_loss_price() {
        let mut new_record = sample_record();
        new_record.stop_loss_price = Decimal::new(0, 0);

        let error = new_record.build().expect_err("should fail");

        assert_eq!(error, TradeRecordError::InvalidStopLoss);
    }

    #[test]
    fn update_reasons_requires_minimum_entries() {
        let mut record = sample_record().build().unwrap();

        let err = record
            .update_sell_reasons(vec![])
            .expect_err("should reject");

        assert_eq!(
            err,
            TradeRecordError::NotEnoughSellReasons {
                required: MIN_SELL_REASONS,
                provided: 0,
            }
        );
    }
}
