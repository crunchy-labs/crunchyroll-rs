//! Payment related.

use crate::{Crunchyroll, Request, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Detailed information about a payment.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct PaymentInfo {
    /// Payment method, e.g. `Paypal Billing Agreement test@example.com`, which is PayPal.
    pub payment_method_name: String,
}

/// Invoice of a subscription billing.
#[derive(Debug, Default, Deserialize, Serialize, Request)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Invoice {
    pub invoice_id: String,
    pub invoice_date: DateTime<Utc>,

    pub payment_info: PaymentInfo,

    /// Payed amount.
    pub amount: f32,
    /// Currency, e.g. `EUR` for Euros.
    pub currency_code: String,

    /// Status of the payment.
    pub status: String,

    /// The paid plan, e.g. `cr_premium.1_month`, which is the normal "Fan" premium plan, paid
    /// monthly
    pub description: String,
}

/// The history of all subscription billings.
#[derive(Debug, Default, Deserialize, Serialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct BillingHistory {
    #[serde(rename = "billingHistory")]
    pub invoices: Vec<Invoice>,
}

impl Crunchyroll {
    /// Get billing history.
    pub async fn billing_history(&self) -> Result<BillingHistory> {
        let endpoint = "https://www.crunchyroll.com/v1/billingHistory";
        self.executor.get(endpoint).request().await
    }
}
