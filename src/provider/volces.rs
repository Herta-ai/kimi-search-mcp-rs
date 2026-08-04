use super::{SearchProvider, WebSearchResult};
use serde_json::json;
use std::collections::HashMap;

pub struct VolcesProvider;

#[async_trait::async_trait]
impl SearchProvider for VolcesProvider {
    fn name(&self) -> &str {
        "volces"
    }

    fn tool_name(&self) -> &str {
        "volces-search"
    }

    fn description(&self) -> &str {
        "火山引擎联网搜索 (豆包)"
    }

    fn required_params(&self) -> &[&str] {
        &["volces-apiKey"]
    }

    async fn search(
        &self,
        query: &str,
        params: &HashMap<String, String>,
        client: &reqwest::Client,
    ) -> Result<WebSearchResult, String> {
        let api_key = params
            .get("volces-apiKey")
            .ok_or_else(|| "Missing volces-apiKey".to_string())?;

        let model = params
            .get("volces-model")
            .map(|s| s.as_str())
            .unwrap_or("doubao-seed-2-0-mini-260428");

        let max_keyword = params
            .get("volces-max_keyword")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(2);

        let limit = params
            .get("volces-limit")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(10);

        let sources_str = params
            .get("volces-sources")
            .map(|s| s.as_str())
            .unwrap_or("");

        println!("[volces-search] query: \"{}\", model: {}", query, model);

        let mut web_search_tool = json!({
            "type": "web_search",
            "max_keyword": max_keyword,
            "limit": limit
        });

        if !sources_str.is_empty() {
            let sources: Vec<&str> = sources_str.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
            if let Some(obj) = web_search_tool.as_object_mut() {
                obj.insert("sources".to_string(), json!(sources));
            }
        }

        let body = json!({
            "model": model,
            "stream": false,
            "tools": [web_search_tool],
            "input": [
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "input_text",
                            "text": query
                        }
                    ]
                }
            ]
        });

        let res = client
            .post("https://ark.cn-beijing.volces.com/api/v3/responses")
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !res.status().is_success() {
            let err_text = res.text().await.unwrap_or_default();
            return Err(format!("volces-search API error: {}", err_text));
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
