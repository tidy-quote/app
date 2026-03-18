use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc};

const STARTER_QUOTA: u32 = 5;
const SOLO_QUOTA: u32 = 75;

/// A quota limit: either a finite number of quotes per period, or unlimited.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QuotaLimit {
    Limited(u32),
    Unlimited,
}

/// Maps a Stripe price_id to its quota limit.
///
/// `allowed_price_ids` must be ordered: [starter, solo, pro].
pub fn quota_for_price(price_id: &str, allowed_price_ids: &[String]) -> QuotaLimit {
    if allowed_price_ids.len() >= 3 && price_id == allowed_price_ids[2] {
        return QuotaLimit::Unlimited;
    }
    if allowed_price_ids.len() >= 2 && price_id == allowed_price_ids[1] {
        return QuotaLimit::Limited(SOLO_QUOTA);
    }
    QuotaLimit::Limited(STARTER_QUOTA)
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

    fn price_ids() -> Vec<String> {
        vec![
            "price_starter".to_string(),
            "price_solo".to_string(),
            "price_pro".to_string(),
        ]
    }

    #[test]
    fn starter_plan_has_5_quota() {
        let ids = price_ids();
        assert_eq!(
            quota_for_price("price_starter", &ids),
            QuotaLimit::Limited(STARTER_QUOTA)
        );
    }

    #[test]
    fn solo_plan_has_75_quota() {
        let ids = price_ids();
        assert_eq!(
            quota_for_price("price_solo", &ids),
            QuotaLimit::Limited(SOLO_QUOTA)
        );
    }

    #[test]
    fn pro_plan_is_unlimited() {
        let ids = price_ids();
        assert_eq!(quota_for_price("price_pro", &ids), QuotaLimit::Unlimited);
    }

    #[test]
    fn unknown_price_id_defaults_to_starter() {
        let ids = price_ids();
        assert_eq!(
            quota_for_price("price_unknown", &ids),
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
