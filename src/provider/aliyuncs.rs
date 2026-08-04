use super::{SearchProvider, WebSearchResult};
use serde_json::json;
use std::collections::HashMap;

pub struct AliyuncsProvider;

#[async_trait::async_trait]
impl SearchProvider for AliyuncsProvider {
    fn name(&self) -> &str {
        "aliyuncs"
    }

    fn tool_name(&self) -> &str {
        "aliyuncs-search"
    }

    fn description(&self) -> &str {
        "阿里云联网搜索 (通义千问)"
    }

    fn required_params(&self) -> &[&str] {
        &["aliyuncs-apiKey", "aliyuncs-baseUrl"]
    }

    async fn search(
        &self,
        query: &str,
        params: &HashMap<String, String>,
        client: &reqwest::Client,
    ) -> Result<WebSearchResult, String> {
        let api_key = params
            .get("aliyuncs-apiKey")
            .ok_or_else(|| "Missing aliyuncs-apiKey".to_string())?;

        let base_url = params
            .get("aliyuncs-baseUrl")
            .ok_or_else(|| "Missing aliyuncs-baseUrl".to_string())?;

        let model = params
            .get("aliyuncs-model")
            .map(|s| s.as_str())
            .unwrap_or("qwen3.7-flash");

        println!(
            "[aliyuncs-search] query: \"{}\", baseUrl: {}",
            query, base_url
        );

        let clean_base_url = base_url.trim_end_matches('/');
        let url = format!("{}/compatible-mode/v1/responses", clean_base_url);

        let body = json!({
            "model": model,
            "input": query,
            "tools": [{ "type": "web_search" }]
        });

        let res = client
            .post(&url)
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !res.status().is_success() {
            let err_text = res.text().await.unwrap_or_default();
            return Err(format!("aliyuncs-search API error: {}", err_text));
        }

        let data: serde_json::Value = res
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let outputs = data["output"].as_array();
        let mut text_parts = Vec::new();

        if let Some(outputs_arr) = outputs {
            for output in outputs_arr {
                if output["type"].as_str() == Some("message") {
                    if let Some(content_arr) = output["content"].as_array() {
                        for c in content_arr {
                            if c["type"].as_str() == Some("output_text") {
                                if let Some(text) = c["text"].as_str() {
                                    if !text.is_empty() {
                                        text_parts.push(text);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let result_text = if text_parts.is_empty() {
            "未获取到搜索结果".to_string()
        } else {
            text_parts.join("\n")
        };

        Ok(WebSearchResult::text(result_text))
    }
}
