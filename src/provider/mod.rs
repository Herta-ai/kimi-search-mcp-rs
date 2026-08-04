use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

/// MCP Tool 定义
#[derive(Debug, Clone, Serialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// 搜索结果内容项
#[derive(Debug, Clone, Serialize)]
pub struct ContentItem {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

/// 搜索结果
#[derive(Debug, Clone, Serialize)]
pub struct WebSearchResult {
    pub content: Vec<ContentItem>,
}

impl WebSearchResult {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            content: vec![ContentItem {
                content_type: "text".to_string(),
                text: text.into(),
            }],
        }
    }

    pub fn to_json_value(&self) -> Value {
        serde_json::to_value(self).unwrap_or_else(|_| {
            serde_json::json!({
                "content": []
            })
        })
    }
}

/// 搜索 Provider 抽象接口
#[async_trait::async_trait]
pub trait SearchProvider: Send + Sync {
    /// Provider 标识符，如 "kimi"、"zai"
    fn name(&self) -> &str;

    /// MCP 工具名，如 "kimi-search"
    fn tool_name(&self) -> &str;

    /// 工具描述
    fn description(&self) -> &str;

    /// 必需的 URL 参数名（含前缀），如 ["kimi-apiKey"]
    fn required_params(&self) -> &[&str];

    /// 检查此 Provider 在给定 URL 参数下是否可用
    fn is_available(&self, params: &HashMap<String, String>) -> bool {
        self.required_params()
            .iter()
            .all(|p| params.get(*p).is_some_and(|v| !v.trim().is_empty()))
    }

    /// 返回 MCP Tool 定义（用于 tools/list）
    fn get_tool_definition(&self) -> McpTool {
        McpTool {
            name: self.tool_name().to_string(),
            description: self.description().to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "搜索内容" }
                },
                "required": ["query"]
            }),
        }
    }

    /// 执行搜索
    async fn search(
        &self,
        query: &str,
        params: &HashMap<String, String>,
        client: &reqwest::Client,
    ) -> Result<WebSearchResult, String>;
}

/// Provider 注册表
pub struct ProviderRegistry {
    providers: Vec<Box<dyn SearchProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    pub fn register(&mut self, provider: Box<dyn SearchProvider>) {
        self.providers.push(provider);
    }

    /// 根据 URL 参数返回可用的 MCP 工具列表
    pub fn get_available_tools(&self, params: &HashMap<String, String>) -> Vec<McpTool> {
        let mut tools: Vec<McpTool> = self
            .providers
            .iter()
            .filter(|p| p.is_available(params))
            .map(|p| p.get_tool_definition())
            .collect();

        // 处理 default-search 别名
        if let Some(default_search) = params.get("default-search") {
            if let Some(target) = self
                .providers
                .iter()
                .find(|p| p.name() == default_search && p.is_available(params))
            {
                let mut alias_tool = target.get_tool_definition();
                alias_tool.name = "web-search".to_string();
                alias_tool.description = format!("网络搜索 (默认使用 {})", target.name());
                tools.push(alias_tool);
            }
        }

        tools
    }

    /// 调用指定工具
    pub async fn call_tool(
        &self,
        tool_name: &str,
        query: &str,
        params: &HashMap<String, String>,
        client: &reqwest::Client,
    ) -> Result<WebSearchResult, String> {
        // 处理 web-search 别名
        let resolved_name = if tool_name == "web-search" {
            let default_search = params
                .get("default-search")
                .ok_or_else(|| "No default-search configured for web-search alias".to_string())?;
            format!("{}-search", default_search)
        } else {
            tool_name.to_string()
        };

        let provider = self
            .providers
            .iter()
            .find(|p| p.tool_name() == resolved_name && p.is_available(params))
            .ok_or_else(|| format!("Tool '{}' not found or not configured", tool_name))?;

        provider.search(query, params, client).await
    }
}

// 导出所有 Provider 子模块
pub mod aliyuncs;
pub mod kimi;
pub mod tencentmaas;
pub mod volces;
pub mod zai;

pub use aliyuncs::AliyuncsProvider;
pub use kimi::KimiProvider;
pub use tencentmaas::TencentmaasProvider;
pub use volces::VolcesProvider;
pub use zai::ZaiProvider;

/// 创建全局注册表并注册所有 Provider
pub fn build_registry() -> ProviderRegistry {
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(KimiProvider));
    registry.register(Box::new(ZaiProvider));
    registry.register(Box::new(VolcesProvider));
    registry.register(Box::new(TencentmaasProvider));
    registry.register(Box::new(AliyuncsProvider));
    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_tools_discovery_empty() {
        let registry = build_registry();
        let params = HashMap::new();
        let tools = registry.get_available_tools(&params);
        assert_eq!(tools.len(), 0);
    }

    #[test]
    fn test_dynamic_tools_discovery_single() {
        let registry = build_registry();
        let mut params = HashMap::new();
        params.insert("kimi-apiKey".to_string(), "sk-test".to_string());

        let tools = registry.get_available_tools(&params);
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "kimi-search");
    }

    #[test]
    fn test_dynamic_tools_discovery_multiple_and_alias() {
        let registry = build_registry();
        let mut params = HashMap::new();
        params.insert("kimi-apiKey".to_string(), "sk-test1".to_string());
        params.insert("zai-apiKey".to_string(), "sk-test2".to_string());
        params.insert("default-search".to_string(), "kimi".to_string());

        let tools = registry.get_available_tools(&params);
        assert_eq!(tools.len(), 3);
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"kimi-search"));
        assert!(names.contains(&"zai-search"));
        assert!(names.contains(&"web-search"));
    }

    #[test]
    fn test_aliyuncs_requires_both_params() {
        let registry = build_registry();
        let mut params = HashMap::new();
        params.insert("aliyuncs-apiKey".to_string(), "sk-test".to_string());

        // 仅有 apiKey，没有 baseUrl 时不可用
        let tools = registry.get_available_tools(&params);
        assert_eq!(tools.len(), 0);

        // 两者都具备时可用
        params.insert("aliyuncs-baseUrl".to_string(), "https://test.maas.aliyuncs.com".to_string());
        let tools = registry.get_available_tools(&params);
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "aliyuncs-search");
    }
}
