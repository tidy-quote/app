use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc};
use serde::Serialize;

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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanInfo {
    pub name: String,
    pub price_id: String,
    pub price: String,
    pub description: String,
    pub features: Vec<String>,
    pub quota: Option<u32>,
    pub featured: bool,
}

impl PlanConfig {
    pub fn contains(&self, price_id: &str) -> bool {
        price_id == self.starter_price_id
            || price_id == self.solo_price_id
            || price_id == self.pro_price_id
    }

    pub fn plans(&self) -> Vec<PlanInfo> {
        vec![
            PlanInfo {
                name: "Starter".to_string(),
                price_id: self.starter_price_id.clone(),
                price: "$1.99".to_string(),
                description: "Try it out with a few quotes each month.".to_string(),
                features: vec![
                    "5 AI quote generations per month".to_string(),
                    "1 pricing template".to_string(),
                    "Job summary extraction".to_string(),
                    "Follow-up message drafts".to_string(),
                ],
                quota: Some(STARTER_QUOTA),
                featured: false,
            },
            PlanInfo {
                name: "Solo".to_string(),
                price_id: self.solo_price_id.clone(),
                price: "$8.99".to_string(),
                description: "For cleaners quoting multiple jobs a week.".to_string(),
                features: vec![
                    "75 AI quote generations per month".to_string(),
                    "Multiple pricing templates".to_string(),
                    "All tone options".to_string(),
                    "Photo & screenshot uploads".to_string(),
                ],
                quota: Some(SOLO_QUOTA),
                featured: true,
            },
            PlanInfo {
                name: "Pro".to_string(),
                price_id: self.pro_price_id.clone(),
                price: "$19.99".to_string(),
                description: "For busy cleaners who quote every day.".to_string(),
                features: vec![
                    "Unlimited quote generations".to_string(),
                    "Multi-location pricing templates".to_string(),
                    "Priority AI processing".to_string(),
                    "Everything in Solo".to_string(),
                ],
                quota: None,
                featured: false,
            },
        ]
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

    #[test]
    fn billing_period_for_leap_year_february() {
        let now = Utc.with_ymd_and_hms(2028, 2, 29, 23, 59, 59).unwrap();
        let (start, end) = current_billing_period(now);
        assert_eq!(start, Utc.with_ymd_and_hms(2028, 2, 1, 0, 0, 0).unwrap());
        assert_eq!(end, Utc.with_ymd_and_hms(2028, 3, 1, 0, 0, 0).unwrap());
    }

    #[test]
    fn billing_period_for_non_leap_year_february() {
        let now = Utc.with_ymd_and_hms(2027, 2, 15, 12, 0, 0).unwrap();
        let (start, end) = current_billing_period(now);
        assert_eq!(start, Utc.with_ymd_and_hms(2027, 2, 1, 0, 0, 0).unwrap());
        assert_eq!(end, Utc.with_ymd_and_hms(2027, 3, 1, 0, 0, 0).unwrap());
    }

    #[test]
    fn billing_period_for_first_second_of_month() {
        let now = Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap();
        let (start, end) = current_billing_period(now);
        assert_eq!(start, Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap());
        assert_eq!(end, Utc.with_ymd_and_hms(2026, 7, 1, 0, 0, 0).unwrap());
    }

    #[test]
    fn billing_period_for_last_second_of_month() {
        let now = Utc.with_ymd_and_hms(2026, 1, 31, 23, 59, 59).unwrap();
        let (start, end) = current_billing_period(now);
        assert_eq!(start, Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap());
        assert_eq!(end, Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).unwrap());
    }

    #[test]
    fn plan_config_contains_known_ids() {
        let plans = plan_config();
        assert!(plans.contains("price_starter"));
        assert!(plans.contains("price_solo"));
        assert!(plans.contains("price_pro"));
    }

    #[test]
    fn plan_config_rejects_unknown_id() {
        let plans = plan_config();
        assert!(!plans.contains("price_unknown"));
    }

    #[test]
    fn plans_returns_three_entries() {
        let plans = plan_config();
        let info = plans.plans();
        assert_eq!(info.len(), 3);
        assert_eq!(info[0].name, "Starter");
        assert_eq!(info[1].name, "Solo");
        assert_eq!(info[2].name, "Pro");
    }

    #[test]
    fn solo_plan_is_featured() {
        let plans = plan_config();
        let info = plans.plans();
        let featured: Vec<_> = info.iter().filter(|p| p.featured).collect();
        assert_eq!(featured.len(), 1);
        assert_eq!(featured[0].name, "Solo");
    }

    #[test]
    fn pro_plan_has_no_quota() {
        let plans = plan_config();
        let info = plans.plans();
        let pro = info.iter().find(|p| p.name == "Pro").unwrap();
        assert!(pro.quota.is_none());
    }
}
