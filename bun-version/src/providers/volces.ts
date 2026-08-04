import { BaseSearchProvider } from "./base";
import type { WebSearchResult } from "../types";

export class VolcesProvider extends BaseSearchProvider {
  readonly name = "volces";
  readonly toolName = "volces-search";
  readonly description = "火山引擎联网搜索 (豆包)";
  readonly requiredParams = ["volces-apiKey"];
  readonly optionalParams = [
    {
      name: "model",
      description: "模型名称",
      defaultValue: "doubao-seed-2-0-mini-260428",
    },
    {
      name: "max_keyword",
      description: "最大关键词数 (1-50)",
      defaultValue: "2",
    },
    {
      name: "limit",
      description: "最大返回结果数 (1-50)",
      defaultValue: "10",
    },
    {
      name: "sources",
      description: "附加搜索源 (逗号分隔: douyin,moji,toutiao)",
      defaultValue: "",
    },
  ];

  async search(
    query: string,
    urlParams: URLSearchParams
  ): Promise<WebSearchResult> {
    const apiKey = urlParams.get("volces-apiKey")!;
    const model = this.getParam(
      urlParams,
      "model",
      "doubao-seed-2-0-mini-260428"
    )!;
    const maxKeyword = parseInt(
      this.getParam(urlParams, "max_keyword", "2")!
    );
    const limit = parseInt(this.getParam(urlParams, "limit", "10")!);
    const sourcesStr = this.getParam(urlParams, "sources", "");

    console.log(`[volces-search] query: "${query}", model: ${model}`);

    const webSearchTool: any = {
      type: "web_search",
      max_keyword: maxKeyword,
      limit,
    };
    if (sourcesStr) {
      webSearchTool.sources = sourcesStr.split(",");
    }

    const resp = await fetch(
      "https://ark.cn-beijing.volces.com/api/v3/responses",
      {
        method: "POST",
        headers: {
          Authorization: `Bearer ${apiKey}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          model,
          stream: false,
          tools: [webSearchTool],
          input: [
            {
              role: "user",
              content: [{ type: "input_text", text: query }],
            },
          ],
        }),
      }
    );

    if (!resp.ok) {
      throw new Error(
        `volces-search API error: ${resp.status} ${await resp.text()}`
      );
    }

    const data = (await resp.json()) as any;
    // 提取 output 中 message 类型的 output_text
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
