export const PROMPT_LIBRARY = [
  {
    id: "product-review",
    label: "产品评审",
    prompt:
      "你是一名产品顾问。请评审一个本地优先的桌面工具，重点分析用户信任、隐私边界、功能优先级和首次使用体验，并给出五条可执行改进建议。"
  },
  {
    id: "data-summary",
    label: "数据总结",
    prompt:
      "请把下面这个业务场景整理成结构化分析：一个团队需要比较三家 API 服务商在价格、延迟、稳定性和错误诊断上的差异，请输出指标表、风险点和下一步验证计划。"
  },
  {
    id: "technical-plan",
    label: "技术方案",
    prompt:
      "请设计一个轻量级的本地日志分析流程，用于统计请求成功率、平均延迟、token 消耗和异常响应。要求包含数据结构、处理步骤和三个边界情况。"
  },
  {
    id: "creative-writing",
    label: "创意写作",
    prompt:
      "请写一段 500 字以内的中文短文，主题是一个工程师在深夜重构计费系统时如何发现隐藏假设。语气克制、具体，不要使用夸张情节。"
  },
  {
    id: "comparison",
    label: "方案对比",
    prompt:
      "请比较纯前端应用、桌面应用和云端代理三种架构在处理 API Key、跨域请求、本地报告和长期维护上的差异，并用简洁表格输出。"
  }
];

export function randomPrompt() {
  return PROMPT_LIBRARY[Math.floor(Math.random() * PROMPT_LIBRARY.length)];
}
