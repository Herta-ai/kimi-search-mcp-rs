use super::{SearchProvider, WebSearchResult};
use serde_json::json;
use std::collections::HashMap;

pub struct TencentmaasProvider;

#[async_trait::async_trait]
impl SearchProvider for TencentmaasProvider {
    fn name(&self) -> &str {
        "tencentmaas"
    }

    fn tool_name(&self) -> &str {
        "tencentmaas-search"
    }

    fn description(&self) -> &str {
        "腾讯云联网搜索 (混元)"
    }

    fn required_params(&self) -> &[&str] {
        &["tencentmaas-apiKey"]
    }

    async fn search(
        &self,
        query: &str,
        params: &HashMap<String, String>,
        client: &reqwest::Client,
    ) -> Result<WebSearchResult, String> {
        let api_key = params
            .get("tencentmaas-apiKey")
            .ok_or_else(|| "Missing tencentmaas-apiKey".to_string())?;

        let model = params
            .get("tencentmaas-model")
            .map(|s| s.as_str())
            .unwrap_or("hy3");

        let search_source = params
            .get("tencentmaas-search_source")
            .map(|s| s.as_str())
            .unwrap_or("standard");

        println!(
            "[tencentmaas-search] query: \"{}\", model: {}",
            query, model
        );

        let body = json!({
            "model": model,
            "messages": [
                { "role": "system", "content": "You are a helpful assistant." },
                { "role": "user", "content": query }
            ],
            "web_search_options": {
                "enable": true,
                "search_source": search_source
            },
            "stream": false
        });

        let res = client
            .post("https://tokenhub.tencentmaas.com/v1/chat/completions")
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !res.status().is_success() {
            let err_text = res.text().await.unwrap_or_default();
            return Err(format!("tencentmaas-search API error: {}", err_text));
        }

        let data: serde_json::Value = res
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let text = data["choices"]
            .get(0)
            .and_then(|c| c["message"]["content"].as_str())
            .unwrap_or("未获取到搜索结果");

        Ok(WebSearchResult::text(text))
    }
}
