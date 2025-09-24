use crate::clients::models::requests::{RoutineUpdate, UpdateRoutineRequest};
use crate::clients::models::responses::{
    RoutineApiResponse, RoutineResponse, RoutineUpdateApiResponse, WorkoutResponse,
    WorkoutsListResponse,
};
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

    pub async fn get_workouts(&self, page: i32, page_size: i32) -> Result<WorkoutsListResponse> {
        let api_key = &self.api_key;
        let mut url = self.base.join("/v1/workouts")?;
        url.query_pairs_mut()
            .append_pair("page", &page.to_string())
            .append_pair("pageSize", &page_size.to_string());

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
        let api_response: WorkoutsListResponse = serde_json::from_str(&body)
            .map_err(|e| anyhow::anyhow!("Failed to parse workouts list response: {}", e))?;

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

        let api_response: RoutineApiResponse = serde_json::from_str(&body)
            .map_err(|e| anyhow::anyhow!("Failed to parse routine response: {}", e))?;

        let routine = api_response.routine;
        Ok(routine)
    }

    pub async fn update_routine(
        &self,
        routine_id: &str,
        request: RoutineUpdate,
    ) -> Result<RoutineResponse> {
        let api_key = &self.api_key;
        let url = self
            .base
            .join(&format!("{}{}", ROUTINES_ENDPOINT, routine_id))?;

        let api_request = UpdateRoutineRequest { routine: request };
        let json_body = serde_json::to_string(&api_request)?;

        tracing::debug!(
            routine_id = %routine_id,
            request_body = %json_body,
            "hevy.update_routine.request"
        );

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

        tracing::debug!(
            routine_id = %routine_id,
            response_body = %body,
            "hevy.update_routine.response"
        );

        let api_response: RoutineUpdateApiResponse = serde_json::from_str(&body)?;
        let routine = api_response
            .routine
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("API returned empty routine array"))?;
        Ok(routine)
    }
}
