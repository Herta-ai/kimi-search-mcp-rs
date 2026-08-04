import { BaseSearchProvider } from "./base";
import type { WebSearchResult } from "../types";

export class ZaiProvider extends BaseSearchProvider {
  readonly name = "zai";
  readonly toolName = "zai-search";
  readonly description = "智谱 AI 联网搜索";
  readonly requiredParams = ["zai-apiKey"];
  readonly optionalParams = [
    {
      name: "search_engine",
      description: "搜索引擎 (search_std/search_pro/search_pro_sogou/search_pro_quark)",
      defaultValue: "search_std",
    },
    { name: "count", description: "返回结果数 (1-50)", defaultValue: "10" },
    {
      name: "search_recency_filter",
      description: "时间过滤 (oneDay/oneWeek/oneMonth/oneYear/noLimit)",
      defaultValue: "noLimit",
    },
    {
      name: "content_size",
      description: "内容长度 (medium/high)",
      defaultValue: "medium",
    },
  ];

  async search(
    query: string,
    urlParams: URLSearchParams
  ): Promise<WebSearchResult> {
    const apiKey = urlParams.get("zai-apiKey")!;
    const searchEngine = this.getParam(
      urlParams,
      "search_engine",
      "search_std"
    )!;
    const count = parseInt(this.getParam(urlParams, "count", "10")!);
    const recencyFilter = this.getParam(
      urlParams,
      "search_recency_filter",
      "noLimit"
    )!;
    const contentSize = this.getParam(urlParams, "content_size", "medium");

    console.log(`[zai-search] query: "${query}", engine: ${searchEngine}`);

    const resp = await fetch(
      "https://open.bigmodel.cn/api/paas/v4/web_search",
      {
        method: "POST",
        headers: {
          Authorization: `Bearer ${apiKey}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          search_query: query,
          search_engine: searchEngine,
          search_intent: false,
          count,
          search_recency_filter: recencyFilter,
          ...(contentSize ? { content_size: contentSize } : {}),
        }),
      }
    );

    if (!resp.ok) {
      throw new Error(
        `zai-search API error: ${resp.status} ${await resp.text()}`
      );
    }

    const data = (await resp.json()) as any;
    const results = data.search_result || [];
    const text = results
      .map(
        (r: any, i: number) =>
          `[${i + 1}] ${r.title}\n${r.content}\n来源: ${r.link}`
      )
      .join("\n\n");

    return {
      content: [{ type: "text", text: text || "未找到搜索结果" }],
    };
  }
}
