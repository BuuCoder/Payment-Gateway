// Stripe API client
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Serialize)]
pub struct CreatePaymentIntentRequest {
    pub amount: i64, // Amount in cents
    pub currency: String,
}

#[derive(Deserialize, Debug)]
pub struct PaymentIntent {
    pub id: String,
    pub client_secret: String,
    pub amount: i64,
    pub currency: String,
    pub status: String,
}

pub struct StripeClient {
    api_key: String,
    client: reqwest::Client,
}

impl Clone for StripeClient {
    fn clone(&self) -> Self {
        Self {
            api_key: self.api_key.clone(),
            client: reqwest::Client::new(),
        }
    }
}

impl StripeClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    pub async fn create_payment_intent(&self, amount: i64, currency: &str) -> Result<PaymentIntent> {
        let response = self.client
            .post("https://api.stripe.com/v1/payment_intents")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[
                ("amount", amount.to_string()),
                ("currency", currency.to_string()),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Stripe API error: {}", error_text));
        }

        let payment_intent = response.json::<PaymentIntent>().await?;
        Ok(payment_intent)
    }

    pub async fn retrieve_payment_intent(&self, intent_id: &str) -> Result<PaymentIntent> {
        let url = format!("https://api.stripe.com/v1/payment_intents/{}", intent_id);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Stripe API error: {}", error_text));
        }

        let payment_intent = response.json::<PaymentIntent>().await?;
        Ok(payment_intent)
    }
}

