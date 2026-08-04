import { BaseSearchProvider } from "./base";
import type { WebSearchResult } from "../types";

export class TencentmaasProvider extends BaseSearchProvider {
  readonly name = "tencentmaas";
  readonly toolName = "tencentmaas-search";
  readonly description = "腾讯云联网搜索 (混元)";
  readonly requiredParams = ["tencentmaas-apiKey"];
  readonly optionalParams = [
    { name: "model", description: "模型名称", defaultValue: "hy3" },
    {
      name: "search_source",
      description: "搜索版本 (lite/standard)",
      defaultValue: "standard",
    },
  ];

  async search(
    query: string,
    urlParams: URLSearchParams
  ): Promise<WebSearchResult> {
    const apiKey = urlParams.get("tencentmaas-apiKey")!;
    const model = this.getParam(urlParams, "model", "hy3")!;
    const searchSource = this.getParam(
      urlParams,
      "search_source",
      "standard"
    )!;

    console.log(`[tencentmaas-search] query: "${query}", model: ${model}`);

    const resp = await fetch(
      "https://tokenhub.tencentmaas.com/v1/chat/completions",
      {
        method: "POST",
        headers: {
          Authorization: `Bearer ${apiKey}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          model,
          messages: [
            { role: "system", content: "You are a helpful assistant." },
            { role: "user", content: query },
          ],
          web_search_options: {
            enable: true,
            search_source: searchSource,
          },
          stream: false,
        }),
      }
    );

    if (!resp.ok) {
      throw new Error(
        `tencentmaas-search API error: ${resp.status} ${await resp.text()}`
      );
    }

    const data = (await resp.json()) as any;
    const text = data.choices?.[0]?.message?.content ?? "";

    return {
      content: [{ type: "text", text: text || "未获取到搜索结果" }],
    };
  }
}
