use crate::clients::models::requests::UpdateRoutineRequest;
use crate::clients::models::responses::{RoutineResponse, WorkoutResponse};
use crate::config::Config;
use anyhow::Result;
use reqwest::{Client, Url};

const WORKOUTS_ENDPOINT: &str = "/v1/workouts/";
const ROUTINES_ENDPOINT: &str = "/v1/routines/";

#[derive(Clone)]
pub struct HevyClient {
    http: Client,
    base: Url,
    api_key: String,
}

impl HevyClient {
    pub fn new(config: &Config) -> Result<Self> {
        Ok(Self {
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()?,
            base: Url::parse(&config.hevy_api_url)?,
            api_key: config.hevy_api_key.clone(),
        })
    }

    pub async fn get_workout(&self, workout_id: &str) -> Result<WorkoutResponse> {
        let api_key = &self.api_key;
        let url = self
            .base
            .join(&format!("{}{}", WORKOUTS_ENDPOINT, workout_id))?;

        let response = self.http.get(url).header("api-key", api_key).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(anyhow::anyhow!(
                "API request failed with status {}: {}",
                status,
                body
            ));
        }

        let body = response.text().await?;

        let api_response: WorkoutResponse = serde_json::from_str(&body)?;
        Ok(api_response)
    }

    pub async fn get_routine(&self, routine_id: &str) -> Result<RoutineResponse> {
        let api_key = &self.api_key;
        let url = self
            .base
            .join(&format!("{}{}", ROUTINES_ENDPOINT, routine_id))?;

        let response = self.http.get(url).header("api-key", api_key).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(anyhow::anyhow!(
                "API request failed with status {}: {}",
                status,
                body
            ));
        }

        let body = response.text().await?;

        let api_response: RoutineResponse = serde_json::from_str(&body)?;
        Ok(api_response)
    }

    pub async fn update_routine(
        &self,
        routine_id: &str,
        request: UpdateRoutineRequest,
    ) -> Result<RoutineResponse> {
        let api_key = &self.api_key;
        let url = self
            .base
            .join(&format!("{}{}", ROUTINES_ENDPOINT, routine_id))?;

        let json_body = serde_json::to_string(&request)?;

        let response = self
            .http
            .put(url)
            .header("api-key", api_key)
            .header("Content-Type", "application/json")
            .body(json_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(anyhow::anyhow!(
                "API request failed with status {}: {}",
                status,
                body
            ));
        }

        let body = response.text().await?;

        let api_response: RoutineResponse = serde_json::from_str(&body)?;
        Ok(api_response)
    }
}
