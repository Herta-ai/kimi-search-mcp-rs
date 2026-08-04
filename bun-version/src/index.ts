import { server } from "./server";

console.log(`🚀 MCP Server running at http://localhost:${server.port}`);
console.log(`🔑 Example: POST http://localhost:3000/mcp?kimi-apiKey=YOUR_KEY`);
console.log(`📚 Supported providers: kimi, zai, volces, tencentmaas, aliyuncs`);
