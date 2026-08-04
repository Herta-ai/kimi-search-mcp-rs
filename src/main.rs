use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, HeaderName, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, env, sync::Arc};
use tokio::signal;
use tower_http::cors::{Any, CorsLayer};

mod provider;
use provider::{build_registry, ProviderRegistry};

const LATEST_PROTOCOL_VERSION: &str = "2026-07-28";
const SUPPORTED_VERSIONS: &[&str] = &["2025-11-25", "2026-07-28"];

/// 根据客户端请求头/请求体协商协议版本
/// - 优先检查 HTTP Header (mcp-protocol-version)
/// - 其次检查 JSON-RPC params.protocolVersion 或 _meta.protocolVersion
/// - server/discover 默认为 2026-07-28
/// - initialize 无参数时默认回退 2025-11-25
fn negotiate_version(headers: &HeaderMap, req: &RpcRequest) -> Result<&'static str, String> {
    let header_version = headers.get("mcp-protocol-version").and_then(|v| v.to_str().ok());
    let params_version = req
        .params
        .as_ref()
        .and_then(|p| p.get("protocolVersion"))
        .and_then(|v| v.as_str());
    let meta_version = req
        .req_meta()
        .and_then(|m| {
            m.get("protocolVersion")
                .or_else(|| m.get("io.modelcontextprotocol/protocolVersion"))
        })
        .and_then(|v| v.as_str());

    let requested_version = header_version.or(params_version).or(meta_version);

    match requested_version {
        Some(v) => {
            if let Some(&matched) = SUPPORTED_VERSIONS.iter().find(|&&sv| sv == v) {
                Ok(matched)
            } else {
                Err(format!(
                    "UnsupportedProtocolVersion: '{}' is not supported. Supported versions: {}",
                    v,
                    SUPPORTED_VERSIONS.join(", ")
                ))
            }
        }
        None => {
            let method = req
                .method
                .as_deref()
                .or_else(|| headers.get("mcp-method").and_then(|v| v.to_str().ok()));
            if method == Some("server/discover") {
                Ok(LATEST_PROTOCOL_VERSION)
            } else {
                Ok("2025-11-25")
            }
        }
    }
}

// --- 数据结构 ---

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
struct RpcRequest {
    id: Option<serde_json::Value>,
    method: Option<String>,
    params: Option<serde_json::Value>,
    _meta: Option<serde_json::Value>,
}

impl RpcRequest {
    fn req_meta(&self) -> Option<&serde_json::Value> {
        self._meta.as_ref()
    }
}

#[derive(Debug, Serialize)]
struct RpcResponse {
    jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<RpcError>,
}

#[derive(Debug, Serialize)]
struct RpcError {
    code: i32,
    message: String,
}

impl RpcResponse {
    fn success(id: Option<serde_json::Value>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: Some(result),
            error: None,
        }
    }

    fn error(id: Option<serde_json::Value>, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(RpcError { code, message }),
        }
    }
}

struct AppState {
    registry: ProviderRegistry,
    client: Client,
}

// --- MCP 协议处理 ---

async fn process_message(
    state: &AppState,
    mut req: RpcRequest,
    url_params: &HashMap<String, String>,
    headers: &HeaderMap,
    negotiated_version: &str,
) -> Option<RpcResponse> {
    // 优先读取 JSON-RPC Body 中的 method，若无则从 HTTP Header 获取 (mcp-method)
    let header_method = headers
        .get("mcp-method")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let method = req.method.take().or(header_method).unwrap_or_default();

    let header_name = headers
        .get("mcp-name")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    if let Some(meta) = &req._meta {
        println!("[MCP Metadata] Received _meta: {:?}", meta);
    }

    match method.as_str() {
        // 1. 服务发现 RPC (server/discover) 与初始化握手 (initialize 兼容旧客户端)
        "server/discover" | "initialize" => Some(RpcResponse::success(
            req.id,
            json!({
                "protocolVersion": negotiated_version,
                "supportedVersions": SUPPORTED_VERSIONS,
                "capabilities": {
                    "tools": {
                        "listChanged": true
                    }
                },
                "serverInfo": {
                    "name": "web-search-mcp",
                    "version": "1.0.0"
                }
            }),
        )),

        // 2. 客户端通知 (返回 None，在 HTTP 层按照 202 Accepted 处理)
        "notifications/initialized" => None,

        // 3. 统一变更订阅响应 (subscriptions/listen - 2026-07-28 新增)
        "subscriptions/listen" => Some(RpcResponse::success(
            req.id,
            json!({
                "status": "listening",
                "subscriptions": ["toolsListChanged"]
            }),
        )),

        // 4. 获取工具列表 — 动态根据 URL 参数返回
        "tools/list" => {
            let tools = state.registry.get_available_tools(url_params);
            Some(RpcResponse::success(
                req.id,
                json!({
                    "tools": tools
                }),
            ))
        }

        // 5. 调用工具 — 动态分发到对应 Provider
        "tools/call" => {
            let params = req.params.unwrap_or(json!({}));
            let name = params
                .get("name")
                .and_then(|n| n.as_str())
                .map(|s| s.to_string())
                .or(header_name);

            let name = match name {
                Some(n) if !n.is_empty() => n,
                _ => {
                    return Some(RpcResponse::error(
                        req.id,
                        -32603,
                        "Missing tool name in params or Mcp-Name header".to_string(),
                    ));
                }
            };

            let query = params
                .get("arguments")
                .and_then(|args| args.get("query"))
                .and_then(|q| q.as_str());

            let query_text = match query {
                Some(q) if !q.trim().is_empty() => q,
                _ => {
                    return Some(RpcResponse::success(
                        req.id,
                        json!({
                            "content": [{ "type": "text", "text": "请输入搜索关键词" }]
                        }),
                    ));
                }
            };

            match state
                .registry
                .call_tool(&name, query_text, url_params, &state.client)
                .await
            {
                Ok(result) => Some(RpcResponse::success(req.id, result.to_json_value())),
                Err(e) => Some(RpcResponse::error(req.id, -32603, e)),
            }
        }

        // 未知方法
        _ => Some(RpcResponse::error(
            req.id,
            -32601,
            format!("Method '{}' not found", method),
        )),
    }
}

// --- HTTP 路由处理 ---

async fn mcp_post_handler(
    State(state): State<Arc<AppState>>,
    Query(query_params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body_text: String,
) -> Response {
    let req: RpcRequest = if body_text.trim().is_empty() {
        RpcRequest::default()
    } else {
        match serde_json::from_str(&body_text) {
            Ok(r) => r,
            Err(e) => {
                let err_res = RpcResponse::error(None, -32700, format!("Parse error: {}", e));
                return (
                    StatusCode::BAD_REQUEST,
                    [("Mcp-Protocol-Version", LATEST_PROTOCOL_VERSION)],
                    Json(err_res),
                )
                    .into_response();
            }
        }
    };

    // 协商协议版本：支持 2025-11-25 和 2026-07-28，无 header 则回退旧版本
    let negotiated_version = match negotiate_version(&headers, &req) {
        Ok(v) => v,
        Err(msg) => {
            let err_res = RpcResponse::error(req.id, -32020, msg);
            return (
                StatusCode::BAD_REQUEST,
                [("Mcp-Protocol-Version", LATEST_PROTOCOL_VERSION)],
                Json(err_res),
            )
                .into_response();
        }
    };

    // 获取请求方法
    let request_method = req
        .method
        .as_deref()
        .or_else(|| headers.get("mcp-method").and_then(|v| v.to_str().ok()));

    // 处理 subscriptions/listen SSE 流
    let is_listen = request_method == Some("subscriptions/listen");

    let accepts_sse = headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.contains("text/event-stream"));

    if is_listen && accepts_sse {
        return (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "text/event-stream"),
                (header::CACHE_CONTROL, "no-cache"),
                (header::CONNECTION, "keep-alive"),
                (
                    HeaderName::from_static("mcp-protocol-version"),
                    negotiated_version,
                ),
            ],
            "event: ready\ndata: {\"status\":\"listening\"}\n\n",
        )
            .into_response();
    }

    if let Some(response) = process_message(&state, req, &query_params, &headers, negotiated_version).await {
        (
            StatusCode::OK,
            [("Mcp-Protocol-Version", negotiated_version)],
            Json(response),
        )
            .into_response()
    } else {
        // 返回 202 Accepted (Notification 规范)
        (
            StatusCode::ACCEPTED,
            [("Mcp-Protocol-Version", negotiated_version)],
        )
            .into_response()
    }
}

async fn mcp_get_handler() -> Response {
    (
        StatusCode::METHOD_NOT_ALLOWED,
        [("Mcp-Protocol-Version", LATEST_PROTOCOL_VERSION)],
        "Subscription stream not supported directly via GET. Use POST for Streamable HTTP.",
    )
        .into_response()
    }

fn parse_port() -> u16 {
    let args: Vec<String> = env::args().collect();
    let mut port = 3000u16;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-p" | "--port" => {
                if i + 1 < args.len() {
                    if let Ok(p) = args[i + 1].parse::<u16>() {
                        port = p;
                    } else {
                        eprintln!("Invalid port number: {}", args[i + 1]);
                        std::process::exit(1);
                    }
                    i += 2;
                } else {
                    eprintln!("Option -p requires a port number");
                    std::process::exit(1);
                }
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                eprintln!("Usage: {} [-p <port>] [--port <port>]", args[0]);
                std::process::exit(1);
            }
        }
    }
    port
}

#[tokio::main]
async fn main() {
    let port = parse_port();

    let state = Arc::new(AppState {
        registry: build_registry(),
        client: Client::new(),
    });

    // 配置 CORS 支持 MCP 2026-07-28 Headers
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            HeaderName::from_static("mcp-session-id"),
            HeaderName::from_static("mcp-protocol-version"),
            HeaderName::from_static("mcp-method"),
            HeaderName::from_static("mcp-name"),
        ])
        .expose_headers([
            HeaderName::from_static("mcp-protocol-version"),
            HeaderName::from_static("mcp-method"),
            HeaderName::from_static("mcp-name"),
        ]);

    // 路由
    let app = Router::new()
        .route("/mcp", post(mcp_post_handler))
        .route("/mcp", get(mcp_get_handler))
        .with_state(state)
        .layer(cors);

    let addr = format!("0.0.0.0:{}", port);
    println!("🚀 MCP Server running at http://{}", addr);
    println!("🔑 Example: POST http://localhost:{}/mcp?kimi-apiKey=YOUR_KEY", port);
    println!("📚 Supported providers: kimi, zai, volces, tencentmaas, aliyuncs");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

/// 监听退出信号的函数
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            println!("Received Ctrl+C (SIGINT), starting graceful shutdown...");
        },
        _ = terminate => {
            println!("Received Docker Stop (SIGTERM), starting graceful shutdown...");
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_state() -> AppState {
        AppState {
            registry: build_registry(),
            client: Client::new(),
        }
    }

    #[tokio::test]
    async fn test_discover_and_initialize() {
        let state = create_test_state();
        let headers = HeaderMap::new();
        let url_params = HashMap::new();

        // 1. initialize (无 header，回退 2025-11-25)
        let req = RpcRequest {
            id: Some(json!(1)),
            method: Some("initialize".to_string()),
            params: None,
            _meta: None,
        };
        let res = process_message(&state, req, &url_params, &headers, "2025-11-25").await.unwrap();
        assert_eq!(res.jsonrpc, "2.0");
        assert_eq!(res.id, Some(json!(1)));
        let result = res.result.unwrap();
        assert_eq!(result["protocolVersion"], "2025-11-25");
        assert_eq!(result["serverInfo"]["name"], "web-search-mcp");

        // 2. server/discover (带 2026-07-28 header)
        let req = RpcRequest {
            id: Some(json!(2)),
            method: Some("server/discover".to_string()),
            params: None,
            _meta: None,
        };
        let res = process_message(&state, req, &url_params, &headers, "2026-07-28").await.unwrap();
        assert_eq!(res.result.unwrap()["protocolVersion"], "2026-07-28");
    }

    #[tokio::test]
    async fn test_notifications_initialized() {
        let state = create_test_state();
        let headers = HeaderMap::new();
        let url_params = HashMap::new();

        let req = RpcRequest {
            id: None,
            method: Some("notifications/initialized".to_string()),
            params: None,
            _meta: None,
        };
        let res = process_message(&state, req, &url_params, &headers, "2025-11-25").await;
        assert!(res.is_none());
    }

    #[tokio::test]
    async fn test_subscriptions_listen() {
        let state = create_test_state();
        let headers = HeaderMap::new();
        let url_params = HashMap::new();

        let req = RpcRequest {
            id: Some(json!(3)),
            method: Some("subscriptions/listen".to_string()),
            params: None,
            _meta: None,
        };
        let res = process_message(&state, req, &url_params, &headers, "2026-07-28").await.unwrap();
        assert_eq!(res.result.unwrap()["status"], "listening");
    }

    #[tokio::test]
    async fn test_tools_list_with_query_params() {
        let state = create_test_state();
        let headers = HeaderMap::new();
        let mut url_params = HashMap::new();
        url_params.insert("kimi-apiKey".to_string(), "sk-test".to_string());
        url_params.insert("default-search".to_string(), "kimi".to_string());

        let req = RpcRequest {
            id: Some(json!(4)),
            method: Some("tools/list".to_string()),
            params: None,
            _meta: None,
        };
        let res = process_message(&state, req, &url_params, &headers, "2025-11-25").await.unwrap();
        let tools = res.result.unwrap()["tools"].as_array().unwrap().clone();
        assert_eq!(tools.len(), 2);
    }

    #[tokio::test]
    async fn test_tools_call_empty_query() {
        let state = create_test_state();
        let headers = HeaderMap::new();
        let mut url_params = HashMap::new();
        url_params.insert("kimi-apiKey".to_string(), "sk-test".to_string());

        let req = RpcRequest {
            id: Some(json!(5)),
            method: Some("tools/call".to_string()),
            params: Some(json!({
                "name": "kimi-search",
                "arguments": {
                    "query": ""
                }
            })),
            _meta: None,
        };
        let res = process_message(&state, req, &url_params, &headers, "2025-11-25").await.unwrap();
        assert_eq!(
            res.result.unwrap()["content"][0]["text"],
            "请输入搜索关键词"
        );
    }

    #[tokio::test]
    async fn test_header_fallback_method_and_name() {
        let state = create_test_state();
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("mcp-method"),
            "tools/call".parse().unwrap(),
        );
        headers.insert(
            HeaderName::from_static("mcp-name"),
            "kimi-search".parse().unwrap(),
        );
        let url_params = HashMap::new();

        let req = RpcRequest {
            id: Some(json!(6)),
            method: None,
            params: Some(json!({
                "arguments": { "query": "" }
            })),
            _meta: None,
        };
        let res = process_message(&state, req, &url_params, &headers, "2025-11-25").await.unwrap();
        assert_eq!(
            res.result.unwrap()["content"][0]["text"],
            "请输入搜索关键词"
        );
    }

    #[tokio::test]
    async fn test_unknown_method() {
        let state = create_test_state();
        let headers = HeaderMap::new();
        let url_params = HashMap::new();

        let req = RpcRequest {
            id: Some(json!(7)),
            method: Some("invalid/method".to_string()),
            params: None,
            _meta: None,
        };
        let res = process_message(&state, req, &url_params, &headers, "2025-11-25").await.unwrap();
        assert_eq!(res.error.unwrap().code, -32601);
    }

    #[tokio::test]
    async fn test_version_negotiation() {
        let empty_req = RpcRequest::default();

        // 无 header → 回退 2025-11-25
        let headers = HeaderMap::new();
        assert_eq!(negotiate_version(&headers, &empty_req).unwrap(), "2025-11-25");

        // server/discover 无 header → 默认 2026-07-28
        let discover_req = RpcRequest {
            method: Some("server/discover".to_string()),
            ..Default::default()
        };
        assert_eq!(negotiate_version(&headers, &discover_req).unwrap(), "2026-07-28");

        // 2025-11-25 → 接受
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("mcp-protocol-version"),
            "2025-11-25".parse().unwrap(),
        );
        assert_eq!(negotiate_version(&headers, &empty_req).unwrap(), "2025-11-25");

        // 2026-07-28 → 接受
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("mcp-protocol-version"),
            "2026-07-28".parse().unwrap(),
        );
        assert_eq!(negotiate_version(&headers, &empty_req).unwrap(), "2026-07-28");

        // 不支持的版本 → 报错
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("mcp-protocol-version"),
            "9999-01-01".parse().unwrap(),
        );
        assert!(negotiate_version(&headers, &empty_req).is_err());
    }
}
