import { BaseSearchProvider } from "./base";
import type { WebSearchResult } from "../types";

export class AliyuncsProvider extends BaseSearchProvider {
  readonly name = "aliyuncs";
  readonly toolName = "aliyuncs-search";
  readonly description = "阿里云联网搜索 (通义千问)";
  readonly requiredParams = ["aliyuncs-apiKey", "aliyuncs-baseUrl"];
  readonly optionalParams = [
    {
      name: "model",
      description: "模型名称",
      defaultValue: "qwen3.7-flash",
    },
  ];

  async search(
    query: string,
    urlParams: URLSearchParams
  ): Promise<WebSearchResult> {
    const apiKey = urlParams.get("aliyuncs-apiKey")!;
    const baseUrl = urlParams.get("aliyuncs-baseUrl")!;
    const model = this.getParam(urlParams, "model", "qwen3.7-flash")!;

    console.log(`[aliyuncs-search] query: "${query}", baseUrl: ${baseUrl}`);

    const url = `${baseUrl.replace(/\/$/, "")}/compatible-mode/v1/responses`;

    const resp = await fetch(url, {
      method: "POST",
      headers: {
        Authorization: `Bearer ${apiKey}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        model,
        input: query,
        tools: [{ type: "web_search" }],
      }),
    });

    if (!resp.ok) {
      throw new Error(
        `aliyuncs-search API error: ${resp.status} ${await resp.text()}`
      );
    }

    const data = (await resp.json()) as any;
    const outputs = data.output || [];
    const textParts: string[] = [];
    for (const output of outputs) {
      if (output.type === "message" && output.content) {
        for (const c of output.content) {
          if (c.type === "output_text" && c.text) {
            textParts.push(c.text);
          }
        }
      }
    }

    return {
      content: [
        {
          type: "text",
          text: textParts.join("\n") || "未获取到搜索结果",
        },
      ],
    };
  }
}
