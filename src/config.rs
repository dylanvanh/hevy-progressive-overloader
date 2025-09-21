use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub hevy_api_key: String,
    pub hevy_api_url: String,
    pub webhook_token: String,
    pub port: String,
    pub gemini_api_key: String,
    pub gemini_model: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let hevy_api_key = env::var("HEVY_API_KEY")?;
        let webhook_token = env::var("WEBHOOK_TOKEN")?;
        let port = env::var("PORT")?;
        let gemini_api_key = env::var("GEMINI_API_KEY")?;
        let gemini_model =
            env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-2.5-pro".to_string());
        let base_url =
            env::var("BASE_URL").unwrap_or_else(|_| "https://api.hevyapp.com".to_string());

        Ok(Self {
            hevy_api_key,
            webhook_token,
            port,
            hevy_api_url: base_url,
            gemini_api_key,
            gemini_model,
        })
    }
}
