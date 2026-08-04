import type { SearchProvider, McpTool } from "./providers/base";
import type { WebSearchResult } from "./types";

// --- 导入所有 Provider ---
import { KimiProvider } from "./providers/kimi";
import { ZaiProvider } from "./providers/zai";
import { VolcesProvider } from "./providers/volces";
import { TencentmaasProvider } from "./providers/tencentmaas";
import { AliyuncsProvider } from "./providers/aliyuncs";

export class ProviderRegistry {
  private providers: SearchProvider[] = [];

  register(provider: SearchProvider): void {
    this.providers.push(provider);
  }

  /** 根据 URL 参数返回可用的 MCP 工具列表 */
  getAvailableTools(urlParams: URLSearchParams): McpTool[] {
    const tools: McpTool[] = [];

    for (const provider of this.providers) {
      if (provider.isAvailable(urlParams)) {
        tools.push(provider.getToolDefinition());
      }
    }

    // 处理 default-search 别名
    const defaultSearch = urlParams.get("default-search");
    if (defaultSearch) {
      const target = this.providers.find(
        (p) => p.name === defaultSearch && p.isAvailable(urlParams)
      );
      if (target) {
        const aliasTool = target.getToolDefinition();
        tools.push({
          ...aliasTool,
          name: "web-search",
          description: `网络搜索 (默认使用 ${target.name})`,
        });
      }
    }

    return tools;
  }

  /** 调用指定工具 */
  async callTool(
    toolName: string,
    query: string,
    urlParams: URLSearchParams
  ): Promise<WebSearchResult> {
    // 处理 web-search 别名
    let resolvedName = toolName;
    if (toolName === "web-search") {
      const defaultSearch = urlParams.get("default-search");
      if (!defaultSearch) {
        throw new Error("No default-search configured for web-search alias");
      }
      resolvedName = `${defaultSearch}-search`;
    }

    const provider = this.providers.find(
      (p) => p.toolName === resolvedName && p.isAvailable(urlParams)
    );
    if (!provider) {
      throw new Error(`Tool '${toolName}' not found or not configured`);
    }

    return provider.search(query, urlParams);
  }
}

// --- 创建全局注册表实例并注册所有 Provider ---
export const registry = new ProviderRegistry();
registry.register(new KimiProvider());
registry.register(new ZaiProvider());
registry.register(new VolcesProvider());
registry.register(new TencentmaasProvider());
registry.register(new AliyuncsProvider());
