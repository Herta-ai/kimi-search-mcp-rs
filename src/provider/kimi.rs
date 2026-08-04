use super::{SearchProvider, WebSearchResult};
use serde_json::json;
use std::collections::HashMap;

pub struct KimiProvider;

#[async_trait::async_trait]
impl SearchProvider for KimiProvider {
    fn name(&self) -> &str {
        "kimi"
    }

    fn tool_name(&self) -> &str {
        "kimi-search"
    }

    fn description(&self) -> &str {
        "Kimi AI 联网搜索 (Moonshot)"
    }

    fn required_params(&self) -> &[&str] {
        &["kimi-apiKey"]
    }

    async fn search(
        &self,
        query: &str,
        params: &HashMap<String, String>,
        client: &reqwest::Client,
    ) -> Result<WebSearchResult, String> {
        let api_key = params
            .get("kimi-apiKey")
            .ok_or_else(|| "Missing kimi-apiKey".to_string())?;

        let model = params
            .get("kimi-model")
            .map(|s| s.as_str())
            .unwrap_or("moonshot-v1-32k");

        println!("[kimi-search] query: \"{}\", model: {}", query, model);

        // 1. 初始化对话历史 (messages)
        let mut messages = vec![
            json!({
                "role": "system",
                "content": "你是 Kimi，由 Moonshot AI 提供的人工智能助手，你更擅长中文和英文的对话。你会为用户提供安全，有帮助，准确的回答。同时，你会拒绝一切涉及恐怖主义，种族歧视，黄色暴力等问题的回答。Moonshot AI 为专有名词，不可翻译成其他语言。"
            }),
            json!({
                "role": "user",
                "content": query
            }),
        ];

        // 2. 声明内置联网搜索工具
        let tools = json!([{
            "type": "builtin_function",
            "function": {
                "name": "$web_search"
            }
        }]);

        let mut finish_reason: Option<String> = None;
        let mut result_text = String::new();

        // 3. 循环发起对话，直到模型输出纯文本响应
        while finish_reason.is_none() || finish_reason.as_deref() == Some("tool_calls") {
            let request_body = json!({
                "model": model,
                "messages": messages,
                "tools": tools,
                "stream": false
            });

            let res = client
                .post("https://api.moonshot.cn/v1/chat/completions")
                .bearer_auth(api_key)
                .json(&request_body)
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            if !res.status().is_success() {
                let err_text = res.text().await.unwrap_or_default();
                return Err(format!("kimi-search API error: {}", err_text));
            }

            let res_json: serde_json::Value = res
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;

            let choice = res_json["choices"]
                .get(0)
                .ok_or_else(|| "kimi-search: No choice returned from API".to_string())?;

            let message = &choice["message"];
            let current_finish_reason = choice["finish_reason"].as_str().unwrap_or("").to_string();
            finish_reason = Some(current_finish_reason.clone());

            // 4. 判断是否触发了工具调用
            if current_finish_reason == "tool_calls" || message.get("tool_calls").is_some() {
                // (1) 必须先把 assistant 的回复原样加进 messages 中
                messages.push(message.clone());

                // (2) 遍历所有的 tool_calls 构建 tool 角色消息返回给大模型
                if let Some(tool_calls) = message["tool_calls"].as_array() {
                    for tool_call in tool_calls {
                        let tool_call_id = tool_call["id"].as_str().unwrap_or("");
                        let function_name = tool_call["function"]["name"].as_str().unwrap_or("");
                        let arguments_str = tool_call["function"]["arguments"]
                            .as_str()
                            .unwrap_or("{}");

                        let tool_args: serde_json::Value = serde_json::from_str(arguments_str)
                            .unwrap_or_else(|_| json!({}));

                        println!(
                            "[kimi-search] tool_call: {}, args: {}",
                            function_name, arguments_str
                        );

                        let tool_result = if function_name == "$web_search" {
                            tool_args
                        } else {
                            json!({ "error": "unknown tool" })
                        };

                        messages.push(json!({
                            "role": "tool",
                            "tool_call_id": tool_call_id,
                            "name": function_name,
                            "content": tool_result.to_string()
                        }));
                    }
                }
            } else {
                // 最终回复
                result_text = message["content"]
                    .as_str()
                    .unwrap_or("未获取到搜索结果")
                    .to_string();
            }
        }

        if result_text.trim().is_empty() {
            result_text = "未获取到搜索结果".to_string();
        }

        Ok(WebSearchResult::text(result_text))
    }
}
