# 🔍 Kimi Search MCP Server

<div align="center">

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Docker](https://img.shields.io/badge/Docker-Ready-2496ED?logo=docker)](Dockerfile)

**基于 Moonshot AI Kimi 的 MCP 联网搜索服务**

[English](#english) | [简体中文](#简体中文)

</div>

---

## 简体中文

### 📖 项目简介

本项目是一个轻量级的 [MCP (Model Context Protocol)](https://modelcontextprotocol.io/) 服务器实现，使用 Rust 编写，通过 [Moonshot AI](https://moonshot.cn/) 的 Kimi API 提供联网搜索功能。

让任何支持 MCP 协议的 AI 应用都能够实时联网搜索，获取最新信息。

### ✨ 特性

- 🚀 **高性能** - Rust 实现，内存安全，极致性能
- 🪶 **轻量级** - 静态编译，二进制文件体积小
- 🐳 **容器化** - 开箱即用的 Docker 支持
- 🔌 **MCP 协议** - 标准实现，兼容性好
- 🌐 **CORS 支持** - 跨域友好，方便集成
- 🔐 **安全传递** - API Key 通过请求参数传递，不存储

### 🛠️ 技术栈

| 技术 | 用途 |
|------|------|
| [Axum](https://github.com/tokio-rs/axum) | Web 框架 |
| [Tokio](https://tokio.rs/) | 异步运行时 |
| [Reqwest](https://docs.rs/reqwest) | HTTP 客户端 |
| [Serde](https://serde.rs/) | 序列化框架 |

### 📦 快速开始

#### 方式一：Docker 运行（推荐）

```bash
# 构建镜像
docker build -t kimi-search-mcp .

# 运行容器
docker run -d -p 3000:3000 kimi-search-mcp
```

#### 方式二：本地编译

```bash
# 克隆项目
git clone https://github.com/your-username/kimi-search-mcp-rs.git
cd kimi-search-mcp-rs

# 编译运行
cargo run --release
```

服务将在 `http://localhost:3000` 启动。

### 📡 API 使用

#### MCP 端点

```
POST http://localhost:3000/mcp?apiKey=YOUR_KIMI_API_KEY&model=kimi-k2-0905-preview
```

#### 请求示例

**1. 初始化连接**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {}
  }
}
```

**2. 获取工具列表**

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/list"
}
```

**3. 调用搜索工具**

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "search",
    "arguments": {
      "query": "今天的新闻"
    }
  }
}
```

#### 响应示例

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "根据搜索结果，今天的主要新闻包括..."
      }
    ]
  }
}
```

### 🔧 配置参数

| 参数 | 必填 | 默认值 | 说明 |
|------|:----:|--------|------|
| `apiKey` | ✅ | - | Moonshot AI API Key |
| `model` | ❌ | `kimi-k2-0905-preview` | 使用的模型名称 |

支持的模型列表：
- `kimi-k2-0905-preview`
- `kimi-k1-8k`
- `kimi-k1-32k`
- `kimi-k1-128k`

### 🤝 客户端集成

#### Claude Desktop

在 Claude Desktop 配置文件中添加：

```json
{
  "mcpServers": {
    "kimi-search": {
      "url": "http://localhost:3000/mcp",
      "queryParams": {
        "apiKey": "YOUR_KIMI_API_KEY"
      }
    }
  }
}
```

#### Cursor / 其他 MCP 客户端

配置 MCP 服务器 URL：

```
http://localhost:3000/mcp?apiKey=YOUR_KIMI_API_KEY
```

### 📝 开发

```bash
# 开发模式运行
cargo run

# 运行测试
cargo test

# 构建发布版本
cargo build --release
```

### 📄 许可证

本项目采用 MIT 许可证。详见 [LICENSE](LICENSE) 文件。

---

## English

### 📖 Overview

A lightweight [MCP (Model Context Protocol)](https://modelcontextprotocol.io/) server implementation in Rust, providing web search capabilities through [Moonshot AI](https://moonshot.cn/)'s Kimi API.

Enable any MCP-compatible AI application to perform real-time web searches and access up-to-date information.

### ✨ Features

- 🚀 **High Performance** - Rust implementation with memory safety
- 🪶 **Lightweight** - Statically compiled, minimal binary size
- 🐳 **Docker Ready** - Out-of-the-box containerization support
- 🔌 **MCP Protocol** - Standard implementation with great compatibility
- 🌐 **CORS Support** - Cross-origin friendly for easy integration
- 🔐 **Secure** - API Key passed via request parameters, never stored

### 📦 Quick Start

#### Docker (Recommended)

```bash
docker build -t kimi-search-mcp .
docker run -d -p 3000:3000 kimi-search-mcp
```

#### Local Build

```bash
git clone https://github.com/your-username/kimi-search-mcp-rs.git
cd kimi-search-mcp-rs
cargo run --release
```

Server starts at `http://localhost:3000`.

### 📡 API Usage

#### MCP Endpoint

```
POST http://localhost:3000/mcp?apiKey=YOUR_KIMI_API_KEY&model=kimi-k2-0905-preview
```

#### Tool: `search`

Search the web using Kimi's built-in web search capability.

**Parameters:**
- `query` (string, required): The search query

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "search",
    "arguments": {
      "query": "latest technology news"
    }
  }
}
```

### 🔧 Configuration

| Parameter | Required | Default | Description |
|-----------|:--------:|---------|-------------|
| `apiKey` | ✅ | - | Moonshot AI API Key |
| `model` | ❌ | `kimi-k2-0905-preview` | Model to use |

### 📄 License

MIT License. See [LICENSE](LICENSE) for details.

---

<div align="center">

**Made with ❤️ using Rust**

[⬆ Back to Top](#-kimi-search-mcp-server)

</div>
