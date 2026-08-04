# 🌐 Web Search MCP

<p align="center">
  <strong>🔍 多引擎 AI 联网搜索 MCP 服务 — 一站式接入，随心切换</strong>
</p>

<p align="center">
  <a href="#-功能特性">功能特性</a> •
  <a href="#-快速开始">快速开始</a> •
  <a href="#-搜索引擎">搜索引擎</a> •
  <a href="#-配置参数">配置参数</a> •
  <a href="#-api-接口">API 接口</a> •
  <a href="#-部署">部署</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/runtime-Bun-000000?style=for-the-badge&logo=bun&logoColor=white" alt="Bun">
  <img src="https://img.shields.io/badge/TypeScript-3178C6?style=for-the-badge&logo=typescript&logoColor=white" alt="TypeScript">
  <img src="https://img.shields.io/badge/MCP-2026--07--28-FF6B6B?style=for-the-badge" alt="MCP">
  <img src="https://img.shields.io/badge/license-MIT-green?style=for-the-badge" alt="License">
  <img src="https://img.shields.io/badge/dependencies-0-brightgreen?style=for-the-badge" alt="Zero Dependencies">
</p>

---

## ✨ 功能特性

| 特性 | 说明 |
|:---:|------|
| 🔌 **多引擎支持** | Kimi · 智谱 · 火山引擎 · 腾讯云 · 阿里云，一个服务全搞定 |
| 🎯 **动态工具发现** | 按需传参，只暴露你配置了的搜索引擎 |
| 🔀 **默认搜索别名** | `default-search=kimi` 即可生成通用 `web-search` 工具 |
| 📡 **MCP 协议** | 完整实现 Model Context Protocol (2026-07-28) |
| ⚡ **零运行时依赖** | 全部使用原生 `fetch`，无第三方依赖 |
| 🚀 **轻量高效** | 基于 Bun 运行时，启动快、内存占用低 |
| 🐳 **Docker 支持** | 开箱即用的 Dockerfile，便于容器化部署 |
| 🧩 **易于扩展** | 实现接口 + 注册一行代码 = 新增搜索引擎 |

---

## 🔍 搜索引擎

<table>
  <thead>
    <tr>
      <th>Provider</th>
      <th>Tool 名称</th>
      <th>必需参数</th>
      <th>说明</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>🌙 <strong>Kimi</strong></td>
      <td><code>kimi-search</code></td>
      <td><code>kimi-apiKey</code></td>
      <td>月之暗面 Moonshot AI 联网搜索</td>
    </tr>
    <tr>
      <td>🧠 <strong>智谱 (Zai)</strong></td>
      <td><code>zai-search</code></td>
      <td><code>zai-apiKey</code></td>
      <td>智谱 BigModel 搜索 API，支持多搜索引擎</td>
    </tr>
    <tr>
      <td>🌋 <strong>火山引擎 (Volces)</strong></td>
      <td><code>volces-search</code></td>
      <td><code>volces-apiKey</code></td>
      <td>豆包大模型联网搜索，支持抖音/头条等源</td>
    </tr>
    <tr>
      <td>☁️ <strong>腾讯云 (Tencentmaas)</strong></td>
      <td><code>tencentmaas-search</code></td>
      <td><code>tencentmaas-apiKey</code></td>
      <td>混元大模型联网搜索</td>
    </tr>
    <tr>
      <td>🟠 <strong>阿里云 (Aliyuncs)</strong></td>
      <td><code>aliyuncs-search</code></td>
      <td><code>aliyuncs-apiKey</code> + <code>aliyuncs-baseUrl</code></td>
      <td>通义千问联网搜索（需提供业务空间 URL）</td>
    </tr>
  </tbody>
</table>

> [!TIP]
> 阿里云需要同时提供 `aliyuncs-apiKey` **和** `aliyuncs-baseUrl`（如 `https://{WorkspaceId}.cn-beijing.maas.aliyuncs.com`）才会在 `tools/list` 中显示。

---

## 🚀 快速开始

### 📋 环境要求

- [Bun](https://bun.sh) >= 1.0.0
- 至少一个搜索引擎的 API Key

### 📦 安装依赖

```bash
bun install
```

### 🏃 开发运行

```bash
bun run dev
```

服务启动后即可使用：

```
🚀 MCP Server running at http://localhost:3000
🔑 Example: POST http://localhost:3000/mcp?kimi-apiKey=YOUR_KEY
📚 Supported providers: kimi, zai, volces, tencentmaas, aliyuncs
```

### 🔨 构建可执行文件

```bash
bun run build
```

编译后的可执行文件位于 `dist/kimi-search-mcp`

---

## ⚙️ 配置参数

所有参数均通过 **URL Query String** 传递，格式为 `{provider}-{param}`。

### 🌙 Kimi

| 参数 | 必填 | 默认值 | 说明 |
|------|:----:|--------|------|
| `kimi-apiKey` | ✅ | — | Moonshot API Key |
| `kimi-model` | — | `moonshot-v1-32k` | 模型名称 |

### 🧠 智谱 (Zai)

| 参数 | 必填 | 默认值 | 说明 |
|------|:----:|--------|------|
| `zai-apiKey` | ✅ | — | 智谱 API Key |
| `zai-search_engine` | — | `search_std` | 搜索引擎 (`search_std` / `search_pro` / `search_pro_sogou` / `search_pro_quark`) |
| `zai-count` | — | `10` | 返回结果数 (1-50) |
| `zai-search_recency_filter` | — | `noLimit` | 时间过滤 (`oneDay` / `oneWeek` / `oneMonth` / `oneYear` / `noLimit`) |
| `zai-content_size` | — | `medium` | 内容长度 (`medium` / `high`) |

### 🌋 火山引擎 (Volces)

| 参数 | 必填 | 默认值 | 说明 |
|------|:----:|--------|------|
| `volces-apiKey` | ✅ | — | 火山引擎 API Key |
| `volces-model` | — | `doubao-seed-2-0-mini-260428` | 模型名称 |
| `volces-max_keyword` | — | `2` | 最大关键词数 (1-50) |
| `volces-limit` | — | `10` | 最大返回结果数 (1-50) |
| `volces-sources` | — | — | 附加搜索源，逗号分隔 (`douyin,moji,toutiao`) |

### ☁️ 腾讯云 (Tencentmaas)

| 参数 | 必填 | 默认值 | 说明 |
|------|:----:|--------|------|
| `tencentmaas-apiKey` | ✅ | — | 腾讯云 API Key |
| `tencentmaas-model` | — | `hy3` | 模型名称 |
| `tencentmaas-search_source` | — | `standard` | 搜索版本 (`lite` / `standard`) |

### 🟠 阿里云 (Aliyuncs)

| 参数 | 必填 | 默认值 | 说明 |
|------|:----:|--------|------|
| `aliyuncs-apiKey` | ✅ | — | 阿里云 API Key |
| `aliyuncs-baseUrl` | ✅ | — | 业务空间 URL（如 `https://{WorkspaceId}.cn-beijing.maas.aliyuncs.com`） |
| `aliyuncs-model` | — | `qwen3.7-flash` | 模型名称 |

### 🔀 默认搜索

| 参数 | 说明 |
|------|------|
| `default-search` | 设为 Provider 名称（如 `kimi`），会额外暴露一个 `web-search` 通用工具，调用时委托给指定 Provider |

---

## 📡 API 接口

### 端点

```
POST http://localhost:3000/mcp?{参数}
```

### 💡 使用示例

<details>
<summary><b>🔹 只用 Kimi 搜索</b></summary>

```
POST /mcp?kimi-apiKey=sk-xxxxxxxx
```

`tools/list` 返回：`kimi-search`

</details>

<details>
<summary><b>🔹 多引擎 + 默认搜索</b></summary>

```
POST /mcp?kimi-apiKey=sk-xxx&zai-apiKey=zzz&default-search=kimi
```

`tools/list` 返回：`kimi-search`、`zai-search`、`web-search`

> 调用 `web-search` 等同于调用 `kimi-search`

</details>

<details>
<summary><b>🔹 火山引擎自定义参数</b></summary>

```
POST /mcp?volces-apiKey=sk-xxx&volces-max_keyword=5&volces-sources=douyin,toutiao
```

`tools/list` 返回：`volces-search`

</details>

<details>
<summary><b>🔹 阿里云（需要 baseUrl）</b></summary>

```
POST /mcp?aliyuncs-apiKey=sk-xxx&aliyuncs-baseUrl=https://workspace.cn-beijing.maas.aliyuncs.com
```

`tools/list` 返回：`aliyuncs-search`

</details>

<details>
<summary><b>🔹 全部引擎拉满</b></summary>

```
POST /mcp?kimi-apiKey=sk-a&zai-apiKey=sk-b&volces-apiKey=sk-c&tencentmaas-apiKey=sk-d&aliyuncs-apiKey=sk-e&aliyuncs-baseUrl=https://ws.maas.aliyuncs.com&default-search=zai
```

`tools/list` 返回：`kimi-search`、`zai-search`、`volces-search`、`tencentmaas-search`、`aliyuncs-search`、`web-search`

</details>

---

### 📨 请求 & 响应示例

**获取工具列表：**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/list"
}
```

**调用搜索工具：**

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "kimi-search",
    "arguments": {
      "query": "今天北京天气怎么样"
    }
  }
}
```

**响应：**

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "搜索结果..."
      }
    ]
  }
}
```

---

## 🖥️ 在客户端中使用

### Claude Desktop

在 Claude Desktop 配置文件中添加：

- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "web-search": {
      "url": "http://localhost:3000/mcp?kimi-apiKey=YOUR_API_KEY&default-search=kimi"
    }
  }
}
```

### Cursor

在 `.cursor/mcp.json` 中添加：

```json
{
  "mcpServers": {
    "web-search": {
      "url": "http://localhost:3000/mcp?kimi-apiKey=YOUR_API_KEY&default-search=kimi"
    }
  }
}
```

---

## 🐳 部署

### Docker 构建

```bash
docker build -t web-search-mcp .
```

### Docker 运行

```bash
docker run -d -p 3000:3000 web-search-mcp
```

### Docker Compose

```yaml
version: '3'
services:
  web-search-mcp:
    build: .
    ports:
      - "3000:3000"
```

---

## 📁 项目结构

```
web-search-mcp/
├── src/
│   ├── index.ts              # 🚪 入口文件
│   ├── server.ts             # 🌐 MCP HTTP 服务器（动态路由）
│   ├── types.ts              # 📝 公共类型定义
│   ├── registry.ts           # 📦 Provider 注册表 & 别名处理
│   └── providers/            # 🔌 搜索引擎 Provider 目录
│       ├── base.ts           #   ├─ 接口定义 & 抽象基类
│       ├── kimi.ts           #   ├─ 🌙 Kimi (Moonshot)
│       ├── zai.ts            #   ├─ 🧠 智谱 (BigModel)
│       ├── volces.ts         #   ├─ 🌋 火山引擎 (豆包)
│       ├── tencentmaas.ts    #   ├─ ☁️ 腾讯云 (混元)
│       └── aliyuncs.ts       #   └─ 🟠 阿里云 (通义千问)
├── dist/                     # 📦 编译输出
├── Dockerfile                # 🐳 Docker 构建文件
├── package.json              # ⚙️ 项目配置
└── tsconfig.json             # 🔧 TypeScript 配置
```

---

## 🧩 扩展新引擎

只需 **3 步** 即可新增一个搜索引擎：

**① 创建 Provider 文件** `src/providers/my-engine.ts`

```typescript
import { BaseSearchProvider } from "./base";
import type { WebSearchResult } from "../types";

export class MyEngineProvider extends BaseSearchProvider {
  readonly name = "myengine";
  readonly toolName = "myengine-search";
  readonly description = "我的自定义搜索引擎";
  readonly requiredParams = ["myengine-apiKey"];
  readonly optionalParams = [];

  async search(query: string, urlParams: URLSearchParams): Promise<WebSearchResult> {
    const apiKey = urlParams.get("myengine-apiKey")!;
    // ... 调用搜索 API ...
    return { content: [{ type: "text", text: "搜索结果" }] };
  }
}
```

**② 在注册表中注册** `src/registry.ts`

```typescript
import { MyEngineProvider } from "./providers/my-engine";

registry.register(new MyEngineProvider());
```

**③ 完成！** 传入 `myengine-apiKey=xxx` 即可使用 ✅

---

## 🛠️ 技术栈

| 技术 | 用途 |
|:---:|------|
| [Bun](https://bun.sh) | ⚡ 高性能 JavaScript 运行时 |
| [TypeScript](https://www.typescriptlang.org/) | 🔒 类型安全 |
| [MCP Protocol](https://modelcontextprotocol.io/) | 📡 Model Context Protocol (2026-07-28) |

> **零运行时依赖** — 所有 HTTP 请求均使用原生 `fetch` API

---

## 📄 许可证

[MIT License](LICENSE)

---

<p align="center">
  Made with ❤️ using <a href="https://bun.sh">Bun</a> — 🌐 搜索无界，引擎随心
</p>
