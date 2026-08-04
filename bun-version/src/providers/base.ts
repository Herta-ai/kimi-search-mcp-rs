import type { WebSearchResult } from "../types";

/** Provider 参数定义 */
export interface ParamDef {
  /** 参数名（不含 provider 前缀），如 "model"、"search_engine" */
  name: string;
  /** 描述 */
  description: string;
  /** 默认值 */
  defaultValue?: string;
}

/** MCP Tool 定义 */
export interface McpTool {
  name: string;
  description: string;
  inputSchema: {
    type: "object";
    properties: Record<string, any>;
    required: string[];
  };
}

/** 搜索 Provider 接口 — 新增 Provider 只需实现此接口 */
export interface SearchProvider {
  /** Provider 标识符，如 "kimi"、"zai" */
  readonly name: string;
  /** MCP 工具名，如 "kimi-search" */
  readonly toolName: string;
  /** 工具描述 */
  readonly description: string;
  /** 必需的 URL 参数名（含前缀），如 ["kimi-apiKey"] */
  readonly requiredParams: string[];
  /** 可选参数定义 */
  readonly optionalParams: ParamDef[];

  /**
   * 检查此 Provider 在给定 URL 参数下是否可用。
   * 默认行为：所有 requiredParams 都有值则返回 true。
   */
  isAvailable(urlParams: URLSearchParams): boolean;

  /** 返回 MCP Tool 定义（用于 tools/list） */
  getToolDefinition(): McpTool;

  /** 执行搜索 */
  search(query: string, urlParams: URLSearchParams): Promise<WebSearchResult>;
}

/**
 * 抽象基类，提供默认 isAvailable 和 getToolDefinition 实现
 */
export abstract class BaseSearchProvider implements SearchProvider {
  abstract readonly name: string;
  abstract readonly toolName: string;
  abstract readonly description: string;
  abstract readonly requiredParams: string[];
  abstract readonly optionalParams: ParamDef[];

  isAvailable(urlParams: URLSearchParams): boolean {
    return this.requiredParams.every((p) => urlParams.has(p));
  }

  getToolDefinition(): McpTool {
    return {
      name: this.toolName,
      description: this.description,
      inputSchema: {
        type: "object",
        properties: {
          query: { type: "string", description: "搜索内容" },
        },
        required: ["query"],
      },
    };
  }

  /** 获取 Provider 专属参数值的便捷方法 */
  protected getParam(
    urlParams: URLSearchParams,
    paramName: string,
    defaultValue?: string
  ): string | undefined {
    return urlParams.get(`${this.name}-${paramName}`) ?? defaultValue;
  }

  abstract search(
    query: string,
    urlParams: URLSearchParams
  ): Promise<WebSearchResult>;
}
