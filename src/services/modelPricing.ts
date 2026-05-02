import type { PricingCatalogModel } from "../types";

export interface ModelPrice {
  id: string;
  provider: string;
  displayName: string;
  inputUsdPer1M: number;
  cachedInputUsdPer1M?: number;
  outputUsdPer1M: number;
  note?: string;
  aliases?: string[];
}

export const MODEL_PRICES: ModelPrice[] = [
  {
    id: "gpt-5.5",
    provider: "OpenAI",
    displayName: "GPT-5.5",
    inputUsdPer1M: 5,
    cachedInputUsdPer1M: 0.5,
    outputUsdPer1M: 30
  },
  {
    id: "gpt-5.4",
    provider: "OpenAI",
    displayName: "GPT-5.4",
    inputUsdPer1M: 2.5,
    cachedInputUsdPer1M: 0.25,
    outputUsdPer1M: 15
  },
  {
    id: "gpt-5.4-mini",
    provider: "OpenAI",
    displayName: "GPT-5.4 mini",
    inputUsdPer1M: 0.75,
    cachedInputUsdPer1M: 0.075,
    outputUsdPer1M: 4.5
  },
  {
    id: "gpt-5.1",
    provider: "OpenAI",
    displayName: "GPT-5.1",
    inputUsdPer1M: 1.25,
    cachedInputUsdPer1M: 0.125,
    outputUsdPer1M: 10
  },
  {
    id: "gpt-5",
    provider: "OpenAI",
    displayName: "GPT-5",
    inputUsdPer1M: 1.25,
    cachedInputUsdPer1M: 0.125,
    outputUsdPer1M: 10
  },
  {
    id: "gpt-5-mini",
    provider: "OpenAI",
    displayName: "GPT-5 mini",
    inputUsdPer1M: 0.25,
    cachedInputUsdPer1M: 0.025,
    outputUsdPer1M: 2
  },
  {
    id: "gpt-5-nano",
    provider: "OpenAI",
    displayName: "GPT-5 nano",
    inputUsdPer1M: 0.05,
    cachedInputUsdPer1M: 0.005,
    outputUsdPer1M: 0.4
  },
  {
    id: "gpt-4.1",
    provider: "OpenAI",
    displayName: "GPT-4.1",
    inputUsdPer1M: 2,
    cachedInputUsdPer1M: 0.5,
    outputUsdPer1M: 8
  },
  {
    id: "gpt-4.1-mini",
    provider: "OpenAI",
    displayName: "GPT-4.1 mini",
    inputUsdPer1M: 0.4,
    cachedInputUsdPer1M: 0.1,
    outputUsdPer1M: 1.6
  },
  {
    id: "gpt-4.1-nano",
    provider: "OpenAI",
    displayName: "GPT-4.1 nano",
    inputUsdPer1M: 0.1,
    cachedInputUsdPer1M: 0.025,
    outputUsdPer1M: 0.4
  },
  {
    id: "gpt-4o",
    provider: "OpenAI",
    displayName: "GPT-4o",
    inputUsdPer1M: 2.5,
    cachedInputUsdPer1M: 1.25,
    outputUsdPer1M: 10
  },
  {
    id: "gpt-4o-mini",
    provider: "OpenAI",
    displayName: "GPT-4o mini",
    inputUsdPer1M: 0.15,
    cachedInputUsdPer1M: 0.075,
    outputUsdPer1M: 0.6
  },
  {
    id: "gpt-5.3-codex",
    provider: "OpenAI",
    displayName: "GPT-5.3 Codex",
    inputUsdPer1M: 1.75,
    outputUsdPer1M: 14
  },
  {
    id: "gpt-5.3-codex-spark",
    provider: "OpenAI",
    displayName: "GPT-5.3 Codex Spark",
    inputUsdPer1M: 1.75,
    outputUsdPer1M: 14,
    note: "官方价格页未单列 Spark，按 gpt-5.3-codex 标准价匹配。"
  },
  {
    id: "gpt-image-2",
    provider: "OpenAI",
    displayName: "GPT-image-2",
    inputUsdPer1M: 5,
    outputUsdPer1M: 30,
    note: "采用文本输入价；图像输入为 $8/1M tokens。"
  },
  {
    id: "gpt-image-1.5",
    provider: "OpenAI",
    displayName: "GPT-image-1.5",
    inputUsdPer1M: 5,
    outputUsdPer1M: 32,
    note: "采用文本输入价和图像输出价；图像输入为 $8/1M tokens。"
  },
  {
    id: "gpt-image-1",
    provider: "OpenAI",
    displayName: "GPT Image 1",
    inputUsdPer1M: 5,
    outputUsdPer1M: 40,
    note: "采用文本输入价和图像输出价；图像输入为 $10/1M tokens。"
  },
  {
    id: "gpt-image-1-mini",
    provider: "OpenAI",
    displayName: "GPT Image 1 mini",
    inputUsdPer1M: 2,
    outputUsdPer1M: 8,
    note: "采用文本输入价和图像输出价；图像输入为 $2.5/1M tokens。"
  },
  {
    id: "claude-opus-4-1",
    provider: "Anthropic",
    displayName: "Claude Opus 4.1",
    inputUsdPer1M: 15,
    outputUsdPer1M: 75,
    aliases: ["claude-opus-4.1", "claude-opus-4-1-20250805"]
  },
  {
    id: "claude-sonnet-4-5",
    provider: "Anthropic",
    displayName: "Claude Sonnet 4.5",
    inputUsdPer1M: 3,
    outputUsdPer1M: 15,
    aliases: ["claude-sonnet-4.5", "claude-sonnet-4-5-20250929"]
  },
  {
    id: "claude-haiku-4-5",
    provider: "Anthropic",
    displayName: "Claude Haiku 4.5",
    inputUsdPer1M: 1,
    outputUsdPer1M: 5,
    aliases: ["claude-haiku-4.5", "claude-haiku-4-5-20251001"]
  },
  {
    id: "claude-3-5-haiku",
    provider: "Anthropic",
    displayName: "Claude 3.5 Haiku",
    inputUsdPer1M: 0.8,
    outputUsdPer1M: 4,
    aliases: ["claude-3-5-haiku-20241022"]
  },
  {
    id: "gemini-2.5-pro",
    provider: "Google",
    displayName: "Gemini 2.5 Pro",
    inputUsdPer1M: 1.25,
    outputUsdPer1M: 10,
    note: "官方价格按上下文长度分档；此处采用 <=200k tokens 档。"
  },
  {
    id: "gemini-2.5-flash",
    provider: "Google",
    displayName: "Gemini 2.5 Flash",
    inputUsdPer1M: 0.3,
    outputUsdPer1M: 2.5
  },
  {
    id: "gemini-2.5-flash-lite",
    provider: "Google",
    displayName: "Gemini 2.5 Flash-Lite",
    inputUsdPer1M: 0.1,
    outputUsdPer1M: 0.4
  },
  {
    id: "gemini-2.0-flash",
    provider: "Google",
    displayName: "Gemini 2.0 Flash",
    inputUsdPer1M: 0.1,
    outputUsdPer1M: 0.4
  },
  {
    id: "deepseek-chat",
    provider: "DeepSeek",
    displayName: "DeepSeek Chat",
    inputUsdPer1M: 0.14,
    cachedInputUsdPer1M: 0.0028,
    outputUsdPer1M: 0.28,
    note: "兼容名，对应 deepseek-v4-flash，采用 cache miss 输入价格。"
  },
  {
    id: "deepseek-reasoner",
    provider: "DeepSeek",
    displayName: "DeepSeek Reasoner",
    inputUsdPer1M: 0.14,
    cachedInputUsdPer1M: 0.0028,
    outputUsdPer1M: 0.28,
    note: "兼容名，对应 deepseek-v4-flash thinking mode，采用 cache miss 输入价格。"
  },
  {
    id: "deepseek-v4-flash",
    provider: "DeepSeek",
    displayName: "DeepSeek V4 Flash",
    inputUsdPer1M: 0.14,
    cachedInputUsdPer1M: 0.0028,
    outputUsdPer1M: 0.28,
    note: "采用 cache miss 输入价格。"
  },
  {
    id: "deepseek-v4-pro",
    provider: "DeepSeek",
    displayName: "DeepSeek V4 Pro",
    inputUsdPer1M: 0.435,
    cachedInputUsdPer1M: 0.003625,
    outputUsdPer1M: 0.87,
    note: "采用 cache miss 输入价格；官方折扣价有效至 2026-05-31 15:59 UTC。"
  },
  {
    id: "glm-5.1",
    provider: "Z.ai",
    displayName: "GLM-5.1",
    inputUsdPer1M: 1.4,
    cachedInputUsdPer1M: 0.3,
    outputUsdPer1M: 4.4,
    aliases: ["zhipu/glm-5.1", "bigmodel/glm-5.1"]
  },
  {
    id: "glm-5",
    provider: "Z.ai",
    displayName: "GLM-5",
    inputUsdPer1M: 1,
    cachedInputUsdPer1M: 0.25,
    outputUsdPer1M: 3.2,
    aliases: ["zhipu/glm-5", "bigmodel/glm-5"]
  },
  {
    id: "glm-5-turbo",
    provider: "Z.ai",
    displayName: "GLM-5 Turbo",
    inputUsdPer1M: 1.2,
    cachedInputUsdPer1M: 0.25,
    outputUsdPer1M: 4,
    aliases: ["zhipu/glm-5-turbo", "bigmodel/glm-5-turbo"]
  },
  {
    id: "glm-4.7",
    provider: "Z.ai",
    displayName: "GLM-4.7",
    inputUsdPer1M: 0.6,
    cachedInputUsdPer1M: 0.12,
    outputUsdPer1M: 2.2,
    aliases: ["zhipu/glm-4.7", "bigmodel/glm-4.7"]
  },
  {
    id: "glm-4.7-flashx",
    provider: "Z.ai",
    displayName: "GLM-4.7 FlashX",
    inputUsdPer1M: 0.07,
    cachedInputUsdPer1M: 0.014,
    outputUsdPer1M: 0.4,
    aliases: ["glm-4.7-flash-x", "zhipu/glm-4.7-flashx"]
  },
  {
    id: "glm-4.7-flash",
    provider: "Z.ai",
    displayName: "GLM-4.7 Flash",
    inputUsdPer1M: 0,
    outputUsdPer1M: 0,
    note: "官方标注 Free。"
  },
  {
    id: "glm-4.6",
    provider: "Z.ai",
    displayName: "GLM-4.6",
    inputUsdPer1M: 0.6,
    cachedInputUsdPer1M: 0.12,
    outputUsdPer1M: 2.2,
    aliases: ["zhipu/glm-4.6", "bigmodel/glm-4.6"]
  },
  {
    id: "glm-4.5",
    provider: "Z.ai",
    displayName: "GLM-4.5",
    inputUsdPer1M: 0.6,
    cachedInputUsdPer1M: 0.12,
    outputUsdPer1M: 2.2,
    aliases: ["zhipu/glm-4.5", "bigmodel/glm-4.5"]
  },
  {
    id: "glm-4.5-x",
    provider: "Z.ai",
    displayName: "GLM-4.5 X",
    inputUsdPer1M: 2.2,
    cachedInputUsdPer1M: 0.45,
    outputUsdPer1M: 8.9,
    aliases: ["glm-4.5x", "zhipu/glm-4.5-x"]
  },
  {
    id: "glm-4.5-air",
    provider: "Z.ai",
    displayName: "GLM-4.5 Air",
    inputUsdPer1M: 0.2,
    cachedInputUsdPer1M: 0.04,
    outputUsdPer1M: 1.1,
    aliases: ["zhipu/glm-4.5-air", "bigmodel/glm-4.5-air"]
  },
  {
    id: "glm-4.5-airx",
    provider: "Z.ai",
    displayName: "GLM-4.5 AirX",
    inputUsdPer1M: 1.1,
    cachedInputUsdPer1M: 0.22,
    outputUsdPer1M: 4.5,
    aliases: ["glm-4.5-air-x", "zhipu/glm-4.5-airx"]
  },
  {
    id: "glm-4.5-flash",
    provider: "Z.ai",
    displayName: "GLM-4.5 Flash",
    inputUsdPer1M: 0,
    outputUsdPer1M: 0,
    note: "官方标注 Free。"
  },
  {
    id: "glm-4-32b-0414-128k",
    provider: "Z.ai",
    displayName: "GLM-4-32B-0414-128K",
    inputUsdPer1M: 0.1,
    cachedInputUsdPer1M: 0.1,
    outputUsdPer1M: 0.1,
    aliases: ["glm-4-32b", "glm-4-32b-128k"]
  },
  {
    id: "glm-5v-turbo",
    provider: "Z.ai",
    displayName: "GLM-5V Turbo",
    inputUsdPer1M: 1.2,
    cachedInputUsdPer1M: 0.25,
    outputUsdPer1M: 4,
    aliases: ["glm-5v", "zhipu/glm-5v-turbo"]
  },
  {
    id: "glm-4.6v",
    provider: "Z.ai",
    displayName: "GLM-4.6V",
    inputUsdPer1M: 0.3,
    cachedInputUsdPer1M: 0.06,
    outputUsdPer1M: 0.9,
    aliases: ["glm-4-6v", "zhipu/glm-4.6v"]
  },
  {
    id: "glm-4.6v-flashx",
    provider: "Z.ai",
    displayName: "GLM-4.6V FlashX",
    inputUsdPer1M: 0.04,
    cachedInputUsdPer1M: 0.008,
    outputUsdPer1M: 0.4,
    aliases: ["glm-4.6v-flash-x", "zhipu/glm-4.6v-flashx"]
  },
  {
    id: "glm-4.6v-flash",
    provider: "Z.ai",
    displayName: "GLM-4.6V Flash",
    inputUsdPer1M: 0,
    outputUsdPer1M: 0,
    note: "官方标注 Free。"
  },
  {
    id: "glm-4.5v",
    provider: "Z.ai",
    displayName: "GLM-4.5V",
    inputUsdPer1M: 0.6,
    cachedInputUsdPer1M: 0.12,
    outputUsdPer1M: 1.8,
    aliases: ["zhipu/glm-4.5v"]
  },
  {
    id: "glm-ocr",
    provider: "Z.ai",
    displayName: "GLM-OCR",
    inputUsdPer1M: 0.03,
    outputUsdPer1M: 0.03
  },
  {
    id: "mimo-v2.5-pro",
    provider: "Xiaomi MiMo",
    displayName: "MiMo-V2.5-Pro",
    inputUsdPer1M: 1,
    cachedInputUsdPer1M: 0.2,
    outputUsdPer1M: 3,
    note: "采用 <=256K 输入档；256K-1M 档为 $2/$6。",
    aliases: ["mimo-v2-pro", "xiaomi/mimo-v2.5-pro", "xiaomi/mimo-v2-pro"]
  },
  {
    id: "mimo-v2.5",
    provider: "Xiaomi MiMo",
    displayName: "MiMo-V2.5",
    inputUsdPer1M: 0.4,
    cachedInputUsdPer1M: 0.08,
    outputUsdPer1M: 2,
    note: "采用 <=256K 输入档；256K-1M 档为 $0.8/$4。",
    aliases: ["xiaomi/mimo-v2.5"]
  },
  {
    id: "mimo-v2-omni",
    provider: "Xiaomi MiMo",
    displayName: "MiMo-V2-Omni",
    inputUsdPer1M: 0.4,
    cachedInputUsdPer1M: 0.08,
    outputUsdPer1M: 2,
    aliases: ["xiaomi/mimo-v2-omni"]
  },
  {
    id: "mimo-v2-flash",
    provider: "Xiaomi MiMo",
    displayName: "MiMo-V2-Flash",
    inputUsdPer1M: 0.1,
    cachedInputUsdPer1M: 0.01,
    outputUsdPer1M: 0.3,
    aliases: ["xiaomi/mimo-v2-flash"]
  },
  {
    id: "qwen-max",
    provider: "Alibaba Cloud",
    displayName: "Qwen Max",
    inputUsdPer1M: 1.6,
    outputUsdPer1M: 6.4
  },
  {
    id: "qwen-plus",
    provider: "Alibaba Cloud",
    displayName: "Qwen Plus",
    inputUsdPer1M: 0.4,
    outputUsdPer1M: 1.2
  },
  {
    id: "qwen-turbo",
    provider: "Alibaba Cloud",
    displayName: "Qwen Turbo",
    inputUsdPer1M: 0.05,
    outputUsdPer1M: 0.2
  }
];

export function findModelPrice(modelId: string) {
  return findModelPriceInCatalog(modelId, MODEL_PRICES);
}

export function findModelPriceInCatalog(modelId: string, catalog: ModelPrice[]) {
  const normalized = normalizeModelId(modelId);
  return catalog.find((price) => {
    const candidates = [price.id, price.displayName, ...(price.aliases ?? [])];
    return candidates.some((candidate) => normalizeModelId(candidate) === normalized);
  });
}

export function normalizeModelId(value: string) {
  return value.trim().toLowerCase().replace(/_/g, "-");
}

export function groupPricesByProvider(catalog = MODEL_PRICES) {
  return catalog.reduce<Record<string, ModelPrice[]>>((groups, price) => {
    groups[price.provider] = groups[price.provider] ?? [];
    groups[price.provider].push(price);
    return groups;
  }, {});
}

export function fromPricingCatalogModel(model: PricingCatalogModel): ModelPrice {
  return {
    id: model.id,
    provider: providerLabel(model.provider),
    displayName: model.display_name || model.id,
    inputUsdPer1M: roundPrice(model.input_usd_per_1m),
    cachedInputUsdPer1M:
      typeof model.cached_input_usd_per_1m === "number"
        ? roundPrice(model.cached_input_usd_per_1m)
        : undefined,
    outputUsdPer1M: roundPrice(model.output_usd_per_1m),
    note: model.note ?? model.source ?? undefined
  };
}

function providerLabel(value: string) {
  const labels: Record<string, string> = {
    openai: "OpenAI",
    anthropic: "Anthropic",
    vertex_ai: "Google Vertex AI",
    "vertex_ai-language-models": "Google Vertex AI",
    gemini: "Google",
    deepseek: "DeepSeek",
    zhipu: "Z.ai",
    bigmodel: "Z.ai",
    "z.ai": "Z.ai",
    z_ai: "Z.ai",
    xiaomi: "Xiaomi MiMo",
    mimo: "Xiaomi MiMo",
    dashscope: "Alibaba Cloud",
    openrouter: "OpenRouter",
    azure: "Azure OpenAI",
    bedrock: "AWS Bedrock"
  };
  return labels[value] ?? value;
}

function roundPrice(value: number) {
  return Number(value.toFixed(8));
}
