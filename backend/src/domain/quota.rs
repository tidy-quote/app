use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc};

const STARTER_QUOTA: u32 = 5;
const SOLO_QUOTA: u32 = 75;

/// A quota limit: either a finite number of quotes per period, or unlimited.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QuotaLimit {
    Limited(u32),
    Unlimited,
}

/// Maps plan price IDs to their tier names for quota lookup.
#[derive(Debug, Clone)]
pub struct PlanConfig {
    pub starter_price_id: String,
    pub solo_price_id: String,
    pub pro_price_id: String,
}

impl PlanConfig {
    pub fn contains(&self, price_id: &str) -> bool {
        price_id == self.starter_price_id
            || price_id == self.solo_price_id
            || price_id == self.pro_price_id
    }
}

pub fn quota_for_price(price_id: &str, plans: &PlanConfig) -> QuotaLimit {
    if price_id == plans.pro_price_id {
        QuotaLimit::Unlimited
    } else if price_id == plans.solo_price_id {
        QuotaLimit::Limited(SOLO_QUOTA)
    } else {
        QuotaLimit::Limited(STARTER_QUOTA)
    }
}

/// Returns (period_start, period_end) for the calendar month containing `now`.
pub fn current_billing_period(now: DateTime<Utc>) -> (DateTime<Utc>, DateTime<Utc>) {
    let start = Utc.from_utc_datetime(
        &NaiveDate::from_ymd_opt(now.year(), now.month(), 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap(),
    );

    let next_month = if now.month() == 12 {
        NaiveDate::from_ymd_opt(now.year() + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(now.year(), now.month() + 1, 1).unwrap()
    };
    let end = Utc.from_utc_datetime(&next_month.and_hms_opt(0, 0, 0).unwrap());

    (start, end)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan_config() -> PlanConfig {
        PlanConfig {
            starter_price_id: "price_starter".to_string(),
            solo_price_id: "price_solo".to_string(),
            pro_price_id: "price_pro".to_string(),
        }
    }

    #[test]
    fn starter_plan_has_5_quota() {
        let plans = plan_config();
        assert_eq!(
            quota_for_price("price_starter", &plans),
            QuotaLimit::Limited(STARTER_QUOTA)
        );
    }

    #[test]
    fn solo_plan_has_75_quota() {
        let plans = plan_config();
        assert_eq!(
            quota_for_price("price_solo", &plans),
            QuotaLimit::Limited(SOLO_QUOTA)
        );
    }

    #[test]
    fn pro_plan_is_unlimited() {
        let plans = plan_config();
        assert_eq!(quota_for_price("price_pro", &plans), QuotaLimit::Unlimited);
    }

    #[test]
    fn unknown_price_id_defaults_to_starter() {
        let plans = plan_config();
        assert_eq!(
            quota_for_price("price_unknown", &plans),
            QuotaLimit::Limited(STARTER_QUOTA)
        );
    }

    #[test]
    fn billing_period_for_mid_month() {
        let now = Utc.with_ymd_and_hms(2026, 3, 15, 10, 30, 0).unwrap();
        let (start, end) = current_billing_period(now);
        assert_eq!(start, Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0).unwrap());
        assert_eq!(end, Utc.with_ymd_and_hms(2026, 4, 1, 0, 0, 0).unwrap());
    }

    #[test]
    fn billing_period_for_december() {
        let now = Utc.with_ymd_and_hms(2026, 12, 25, 0, 0, 0).unwrap();
        let (start, end) = current_billing_period(now);
        assert_eq!(start, Utc.with_ymd_and_hms(2026, 12, 1, 0, 0, 0).unwrap());
        assert_eq!(end, Utc.with_ymd_and_hms(2027, 1, 1, 0, 0, 0).unwrap());
    }
}
