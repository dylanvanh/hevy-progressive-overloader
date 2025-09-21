use anyhow::Result;
use gemini_rust::{Gemini, Tool};
use std::fmt;

#[derive(Clone)]
pub struct GeminiClient {
    client: Gemini,
    model: String,
}

impl fmt::Debug for GeminiClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GeminiClient")
            .field("model", &self.model)
            .field("client", &"<Gemini client>")
            .finish()
    }
}

impl GeminiClient {
    pub fn new(api_key: String, model: String) -> Self {
        let client = Gemini::new(api_key).expect("unable to create Gemini API client");
        Self { client, model }
    }

    pub async fn generate_text_with_search(&self, prompt: &str) -> Result<String> {
        let google_search_tool = Tool::google_search();
        let request_builder = self
            .client
            .generate_content()
            .with_user_message(prompt)
            .with_tool(google_search_tool);

        let response = request_builder.execute().await?;

        let text = response.text();
        Ok(text)
    }
}
