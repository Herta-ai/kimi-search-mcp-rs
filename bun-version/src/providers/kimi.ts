import { BaseSearchProvider } from "./base";
import type { WebSearchResult } from "../types";

export class KimiProvider extends BaseSearchProvider {
  readonly name = "kimi";
  readonly toolName = "kimi-search";
  readonly description = "Kimi AI 联网搜索 (Moonshot)";
  readonly requiredParams = ["kimi-apiKey"];
  readonly optionalParams = [
    {
      name: "model",
      description: "模型名称",
      defaultValue: "moonshot-v1-32k",
    },
  ];

  async search(
    query: string,
    urlParams: URLSearchParams
  ): Promise<WebSearchResult> {
    const apiKey = urlParams.get("kimi-apiKey")!;
    const model = this.getParam(urlParams, "model", "moonshot-v1-32k")!;

    console.log(`[kimi-search] query: "${query}", model: ${model}`);

    const messages: any[] = [
      {
        role: "system",
        content:
          "你是 Kimi，由 Moonshot AI 提供的人工智能助手，你更擅长中文和英文的对话。你会为用户提供安全，有帮助，准确的回答。同时，你会拒绝一切涉及恐怖主义，种族歧视，黄色暴力等问题的回答。Moonshot AI 为专有名词，不可翻译成其他语言。",
      },
      { role: "user", content: query },
    ];

    const tools = [
      {
        type: "builtin_function",
        function: { name: "$web_search" },
      },
    ];

    let finishReason: string | null = null;
    let resultText = "";

    // 多步循环：Kimi 可能多次返回 tool_calls，需要逐步执行直到获得最终文本
    while (finishReason === null || finishReason === "tool_calls") {
      const resp = await fetch("https://api.moonshot.cn/v1/chat/completions", {
        method: "POST",
        headers: {
          Authorization: `Bearer ${apiKey}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          model,
          messages,
          tools,
          stream: false,
        }),
      });

      if (!resp.ok) {
        throw new Error(
          `kimi-search API error: ${resp.status} ${await resp.text()}`
        );
      }

      const data = (await resp.json()) as any;
      const choice = data.choices?.[0];

      if (!choice) {
        throw new Error("kimi-search: No choice returned from API");
      }

      finishReason = choice.finish_reason;

      if (finishReason === "tool_calls") {
        // 将 assistant 的 tool_calls 消息加入上下文
        messages.push(choice.message);

        // 逐个处理 tool_calls
        for (const toolCall of choice.message.tool_calls) {
          const toolName = toolCall.function.name;
          const toolArgs = JSON.parse(toolCall.function.arguments);

          console.log(
            `[kimi-search] tool_call: ${toolName}, args:`,
            toolArgs
          );

          // builtin_function 由 Kimi 服务端执行，客户端只需返回确认
          const toolResult =
            toolName === "$web_search"
              ? toolArgs
              : { error: "unknown tool" };

          messages.push({
            role: "tool",
            tool_call_id: toolCall.id,
            name: toolName,
            content: JSON.stringify(toolResult),
          });
        }
      } else {
        // 最终回复
        resultText = choice.message?.content ?? "";
      }
    }

    return {
      content: [{ type: "text", text: resultText || "未获取到搜索结果" }],
    };
  }
}
