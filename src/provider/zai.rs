use super::{SearchProvider, WebSearchResult};
use serde_json::json;
use std::collections::HashMap;

pub struct ZaiProvider;

#[async_trait::async_trait]
impl SearchProvider for ZaiProvider {
    fn name(&self) -> &str {
        "zai"
    }

    fn tool_name(&self) -> &str {
        "zai-search"
    }

    fn description(&self) -> &str {
        "智谱 AI 联网搜索"
    }

    fn required_params(&self) -> &[&str] {
        &["zai-apiKey"]
    }

    async fn search(
        &self,
        query: &str,
        params: &HashMap<String, String>,
        client: &reqwest::Client,
    ) -> Result<WebSearchResult, String> {
        let api_key = params
            .get("zai-apiKey")
            .ok_or_else(|| "Missing zai-apiKey".to_string())?;

        let search_engine = params
            .get("zai-search_engine")
            .map(|s| s.as_str())
            .unwrap_or("search_std");

        let count = params
            .get("zai-count")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(10);

        let recency_filter = params
            .get("zai-search_recency_filter")
            .map(|s| s.as_str())
            .unwrap_or("noLimit");

        let content_size = params
            .get("zai-content_size")
            .map(|s| s.as_str())
            .unwrap_or("medium");

        println!(
            "[zai-search] query: \"{}\", engine: {}",
            query, search_engine
        );

        let mut body = json!({
            "search_query": query,
            "search_engine": search_engine,
            "search_intent": false,
            "count": count,
            "search_recency_filter": recency_filter,
        });

        if !content_size.is_empty() {
            if let Some(obj) = body.as_object_mut() {
                obj.insert(
                    "content_size".to_string(),
                    serde_json::Value::String(content_size.to_string()),
                );
            }
        }

        let res = client
            .post("https://open.bigmodel.cn/api/paas/v4/web_search")
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !res.status().is_success() {
            let err_text = res.text().await.unwrap_or_default();
            return Err(format!("zai-search API error: {}", err_text));
        }

        let data: serde_json::Value = res
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let results = data["search_result"].as_array();
        let formatted_text = match results {
            Some(items) if !items.is_empty() => items
                .iter()
                .enumerate()
                .map(|(i, r)| {
                    let title = r["title"].as_str().unwrap_or("");
                    let content = r["content"].as_str().unwrap_or("");
                    let link = r["link"].as_str().unwrap_or("");
                    format!("[{}] {}\n{}\n来源: {}", i + 1, title, content, link)
                })
                .collect::<Vec<_>>()
                .join("\n\n"),
            _ => "未找到搜索结果".to_string(),
        };

        Ok(WebSearchResult::text(formatted_text))
    }
}
