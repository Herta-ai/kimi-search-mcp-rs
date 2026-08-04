import { registry } from "./registry";

const PROTOCOL_VERSION = "2026-07-28";

const corsHeaders = {
  "Access-Control-Allow-Origin": "*",
  "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
  "Access-Control-Allow-Headers":
    "Content-Type, Mcp-Session-Id, Mcp-Protocol-Version, Mcp-Method, Mcp-Name, Authorization",
  "Access-Control-Expose-Headers":
    "Mcp-Protocol-Version, Mcp-Method, Mcp-Name",
};

// 提取处理逻辑为纯函数（MCP 2026-07-28 无状态核心）
async function processMessage(
  message: any,
  urlParams: URLSearchParams,
  reqHeaders?: Headers
): Promise<any> {
  // 优先读取 JSON-RPC Body 中的 method，若无则从 HTTP Header 获取 (mcp-method)
  const method = message.method || reqHeaders?.get("mcp-method");
  const params = message.params || {};
  const id = message.id ?? null;
  const _meta = message._meta;

  if (_meta) {
    console.log(`[MCP Metadata] Received _meta:`, _meta);
  }

  // 1. 服务发现 RPC (server/discover) 与初始化握手 (initialize 兼容旧客户端)
  if (method === "server/discover" || method === "initialize") {
    return {
      jsonrpc: "2.0",
      id,
      result: {
        protocolVersion: PROTOCOL_VERSION,
        capabilities: {
          tools: {
            listChanged: true,
          },
        },
        serverInfo: {
          name: "web-search-mcp",
          version: "1.0.0",
        },
      },
    };
  }

  // 3. 客户端通知 (返回 null，在 HTTP 层按照 202 Accepted 处理)
  if (method === "notifications/initialized") {
    return null;
  }

  // 4. 统一变更订阅响应流 (subscriptions/listen - 2026-07-28 新增)
  if (method === "subscriptions/listen") {
    return {
      jsonrpc: "2.0",
      id,
      result: {
        status: "listening",
        subscriptions: ["toolsListChanged"],
      },
    };
  }

  // 5. 获取工具列表 — 动态根据 URL 参数返回
  if (method === "tools/list") {
    return {
      jsonrpc: "2.0",
      id,
      result: {
        tools: registry.getAvailableTools(urlParams),
      },
    };
  }

  // 6. 调用工具 — 动态分发到对应 Provider
  if (method === "tools/call") {
    // 兼容：工具名称可从 params.name 或 HTTP Header "mcp-name" 获取
    const name = params.name || reqHeaders?.get("mcp-name");
    const args = params.arguments;

    try {
      if (!name) {
        throw new Error("Missing tool name in params or Mcp-Name header");
      }

      if (!args?.query) {
        return {
          jsonrpc: "2.0",
          id,
          result: {
            content: [{ type: "text", text: "请输入搜索关键词" }],
          },
        };
      }

      const result = await registry.callTool(name, args.query, urlParams);
      return {
        jsonrpc: "2.0",
        id,
        result,
      };
    } catch (e: any) {
      return {
        jsonrpc: "2.0",
        id,
        error: { code: -32603, message: e.message },
      };
    }
  }

  // 未知方法
  return {
    jsonrpc: "2.0",
    id,
    error: { code: -32601, message: `Method '${method}' not found` },
  };
}

// --- Bun Server 启动 ---

export const server = Bun.serve({
  port: 3000,
  idleTimeout: 0,
  async fetch(req, server) {
    const url = new URL(req.url);
    const path = url.pathname;

    // 跨域 OPTIONS 处理
    if (req.method === "OPTIONS") {
      return new Response(null, {
        headers: {
          ...corsHeaders,
          "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
        },
      });
    }

    // 统一使用 /mcp 端点处理
    if (path === "/mcp") {
      // 1. 处理客户端 POST 请求（无状态 Request/Response）
      if (req.method === "POST") {
        try {
          // 读取 2026-07-28 Header 路由信息（req.headers.get 不区分大小写）
          const headerMethod = req.headers.get("mcp-method");
          const headerName = req.headers.get("mcp-name");

          let body: any = {};
          const textBody = await req.text();
          if (textBody.trim().length > 0) {
            body = JSON.parse(textBody);
          }

          // 如果 Header 中提供了 Mcp-Method，且 Body 中无 method 时使用 Header
          if (headerMethod && !body.method) {
            body.method = headerMethod;
          }

          // 如果 Header 中提供了 Mcp-Name 且 params 中无 name
          if (headerName) {
            body.params = body.params || {};
            if (!body.params.name) {
              body.params.name = headerName;
            }
          }

          // 如果是 subscriptions/listen 请求且客户端请求 SSE 连接流
          if (
            body.method === "subscriptions/listen" &&
            req.headers.get("accept")?.includes("text/event-stream")
          ) {
            return new Response("event: ready\ndata: {\"status\":\"listening\"}\n\n", {
              status: 200,
              headers: {
                ...corsHeaders,
                "Content-Type": "text/event-stream",
                "Cache-Control": "no-cache",
                "Connection": "keep-alive",
                "Mcp-Protocol-Version": PROTOCOL_VERSION,
              },
            });
          }

          const response = await processMessage(body, url.searchParams, req.headers);

          // 客户端发送的是 notification（如 notifications/initialized），按照规范返回 202 无 body
          if (!response) {
            return new Response(null, {
              status: 202,
              headers: {
                ...corsHeaders,
                "Mcp-Protocol-Version": PROTOCOL_VERSION,
              },
            });
          }

          // 标准 JSON-RPC 响应，附带协议版本 Header
          return new Response(JSON.stringify(response), {
            status: 200,
            headers: {
              ...corsHeaders,
              "Content-Type": "application/json",
              "Mcp-Protocol-Version": PROTOCOL_VERSION,
            },
          });
        } catch (err) {
          console.error("Error processing message:", err);
          return new Response("Invalid JSON or Processing Error", {
            status: 400,
            headers: {
              ...corsHeaders,
              "Mcp-Protocol-Version": PROTOCOL_VERSION,
            },
          });
        }
      }

      // 2. GET 请求提示
      if (req.method === "GET") {
        return new Response("Subscription stream not supported directly via GET. Use POST for Streamable HTTP.", {
          status: 405,
          headers: {
            ...corsHeaders,
            "Mcp-Protocol-Version": PROTOCOL_VERSION,
          },
        });
      }
    }

    return new Response("Not Found", { status: 404, headers: corsHeaders });
  },
});