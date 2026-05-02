import { useEffect, useMemo, useState } from "react";
import {
  AlertTriangle,
  BookOpen,
  ChevronLeft,
  ChevronRight,
  ListChecks,
  Loader2,
  Play,
  RefreshCw,
  Square,
  WandSparkles,
  X
} from "lucide-react";
import {
  Area,
  AreaChart,
  CartesianGrid,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis
} from "recharts";
import {
  generateTestPrompt,
  listProviderModels,
  listenToProgress,
  refreshPricingCatalog,
  startTestRun,
  stopTestRun
} from "./services/tauriApi";
import { formatCurrency } from "./services/pricing";
import {
  MODEL_PRICES,
  findModelPriceInCatalog,
  fromPricingCatalogModel,
  type ModelPrice
} from "./services/modelPricing";
import { PROMPT_LIBRARY, randomPrompt } from "./services/promptLibrary";
import type {
  ProviderModel,
  RequestLog,
  StartTestRunInput,
  TestProgress,
  TestRunReport
} from "./types";

const DEFAULT_MAX_OUTPUT_TOKENS = 512;
const LOG_PAGE_SIZE = 12;

const defaultForm: StartTestRunInput = {
  name: "测试平台",
  base_url: "https://example.com",
  api_key: "",
  model: "gpt-4o-mini",
  prompt: "请用三句话解释为什么本地优先的软件更适合处理敏感配置。",
  input_price_per_1m: 0.15,
  cached_input_price_per_1m: 0.075,
  output_price_per_1m: 0.6,
  billing_multiplier: 1,
  max_tokens: DEFAULT_MAX_OUTPUT_TOKENS,
  timeout_secs: 60,
  current_usd: 0,
  target_usd: 0.01,
  max_requests: 0,
  concurrency: 3,
  balance_before: 0
};

function App() {
  const [form, setForm] = useState<StartTestRunInput>(defaultForm);
  const [currentReport, setCurrentReport] = useState<TestRunReport | null>(null);
  const [logs, setLogs] = useState<RequestLog[]>([]);
  const [progress, setProgress] = useState<TestProgress | null>(null);
  const [providerModels, setProviderModels] = useState<ProviderModel[]>([]);
  const [pricingCatalog, setPricingCatalog] = useState<ModelPrice[]>(MODEL_PRICES);
  const [modelMode, setModelMode] = useState<"official" | "provider" | "manual">("manual");
  const [modelSearch, setModelSearch] = useState("");
  const [promptPresetId, setPromptPresetId] = useState(PROMPT_LIBRARY[0].id);
  const [, setStatusText] = useState("等待配置");
  const [busy, setBusy] = useState<"run" | "models" | "pricing" | "prompt" | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void listenToProgress((payload) => {
      setProgress(payload);
      setStatusText(statusLabel(payload.status));
      if (payload.latest_log) {
        setLogs((existing) => {
          const withoutDuplicate = existing.filter(
            (item) => item.request_index !== payload.latest_log?.request_index
          );
          return [...withoutDuplicate, payload.latest_log as RequestLog].sort(
            (a, b) => a.request_index - b.request_index
          );
        });
      }
    }).then((dispose) => {
      unlisten = dispose;
    });
    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, []);

  const chartData = useMemo(
    () =>
      logs.map((log) => ({
        name: `#${log.request_index}`,
        cost: Number(log.estimated_cost.toFixed(8)),
        tokens: log.total_tokens
      })),
    [logs]
  );

  function updateField<K extends keyof StartTestRunInput>(field: K, value: StartTestRunInput[K]) {
    setForm((current) => {
      const next = { ...current, [field]: value };
      if (field === "base_url") {
        next.name = deriveProviderName(String(value));
      }
      if (field === "model") {
        const price = findModelPriceInCatalog(String(value), pricingCatalog);
        if (price) {
          next.input_price_per_1m = price.inputUsdPer1M;
          next.cached_input_price_per_1m = price.cachedInputUsdPer1M ?? price.inputUsdPer1M;
          next.output_price_per_1m = price.outputUsdPer1M;
        }
      }
      return next;
    });
  }

  function selectModel(modelId: string, mode: "official" | "provider" | "manual") {
    setModelMode(mode);
    const price = findModelPriceInCatalog(modelId, pricingCatalog);
    setForm((current) => ({
      ...current,
      model: modelId,
      max_tokens: DEFAULT_MAX_OUTPUT_TOKENS,
      input_price_per_1m: price?.inputUsdPer1M ?? current.input_price_per_1m,
      cached_input_price_per_1m:
        price?.cachedInputUsdPer1M ?? price?.inputUsdPer1M ?? current.cached_input_price_per_1m,
      output_price_per_1m: price?.outputUsdPer1M ?? current.output_price_per_1m
    }));
  }

  async function handleRefreshPricingCatalog() {
    setBusy("pricing");
    setError(null);
    setStatusText("正在更新价格库");
    try {
      const result = await refreshPricingCatalog();
      const remotePrices = result.models.map(fromPricingCatalogModel);
      const merged = mergePriceCatalog(remotePrices, MODEL_PRICES);
      setPricingCatalog(merged);
      const matched = findModelPriceInCatalog(form.model, merged);
      if (matched) {
        setForm((current) => ({
          ...current,
          input_price_per_1m: matched.inputUsdPer1M,
          cached_input_price_per_1m: matched.cachedInputUsdPer1M ?? matched.inputUsdPer1M,
          output_price_per_1m: matched.outputUsdPer1M
        }));
      }
      setStatusText(`已更新 ${merged.length} 个价格条目`);
    } catch (err) {
      setStatusText("更新价格库失败");
      setError(normalizeError(err));
    } finally {
      setBusy(null);
    }
  }

  async function handleFetchModels() {
    setBusy("models");
    setError(null);
    setStatusText("正在获取模型");
    try {
      const models = await listProviderModels({ ...form, prompt: form.prompt || "list models" });
      setProviderModels(models);
      setStatusText(`已获取 ${models.length} 个模型`);
      if (models.length && !models.some((item) => item.id === form.model)) {
        const firstKnown = models.find((item) => findModelPriceInCatalog(item.id, pricingCatalog)) ?? models[0];
        selectModel(firstKnown.id, "provider");
      }
    } catch (err) {
      setStatusText("获取模型失败");
      setError(normalizeError(err));
    } finally {
      setBusy(null);
    }
  }

  async function handleGeneratePrompt() {
    setBusy("prompt");
    setError(null);
    setStatusText("正在生成测试内容");
    try {
      const result = await generateTestPrompt({ ...form, max_tokens: DEFAULT_MAX_OUTPUT_TOKENS });
      updateField("prompt", result.prompt);
      setPromptPresetId("generated");
      setStatusText("已生成新测试内容");
    } catch (err) {
      setStatusText("生成测试内容失败");
      setError(normalizeError(err));
    } finally {
      setBusy(null);
    }
  }

  function handleRandomPrompt() {
    const prompt = randomPrompt();
    setPromptPresetId(prompt.id);
    updateField("prompt", prompt.prompt);
  }

  async function handleStart() {
    setBusy("run");
    setError(null);
    setStatusText("测试运行中");
    setLogs([]);
    setProgress(null);
    setCurrentReport(null);
    try {
      const report = await startTestRun({
        ...form,
        name: deriveProviderName(form.base_url),
        max_tokens: DEFAULT_MAX_OUTPUT_TOKENS,
        max_requests: 0,
        balance_before: 0
      });
      setCurrentReport(report);
      setLogs(report.request_logs);
      setStatusText(statusLabel(report.status));
    } catch (err) {
      setStatusText("测试失败");
      setError(normalizeError(err));
    } finally {
      setBusy(null);
    }
  }

  async function handleStop() {
    try {
      await stopTestRun();
      setStatusText("正在停止");
    } catch (err) {
      setError(normalizeError(err));
    }
  }

  return (
    <main className="app-shell">
      <section className="workspace">
        {error ? (
          <div className="error-banner" role="alert">
            <AlertTriangle aria-hidden="true" />
            {error}
          </div>
        ) : null}

        <Workbench
          form={form}
          updateField={updateField}
          busy={busy}
          progress={progress}
          logs={logs}
          chartData={chartData}
          currentReport={currentReport}
          providerModels={providerModels}
          pricingCatalog={pricingCatalog}
          modelMode={modelMode}
          modelSearch={modelSearch}
          promptPresetId={promptPresetId}
          currency="$"
          onSelectModel={selectModel}
          onModelSearchChange={setModelSearch}
          onPromptPresetChange={setPromptPresetId}
          onStart={handleStart}
          onStop={handleStop}
          onFetchModels={handleFetchModels}
          onRefreshPricingCatalog={handleRefreshPricingCatalog}
          onRandomPrompt={handleRandomPrompt}
          onGeneratePrompt={handleGeneratePrompt}
        />
      </section>
    </main>
  );
}

interface WorkbenchProps {
  form: StartTestRunInput;
  updateField: <K extends keyof StartTestRunInput>(field: K, value: StartTestRunInput[K]) => void;
  busy: "run" | "models" | "pricing" | "prompt" | null;
  progress: TestProgress | null;
  logs: RequestLog[];
  chartData: Array<{ name: string; cost: number; tokens: number }>;
  currentReport: TestRunReport | null;
  providerModels: ProviderModel[];
  pricingCatalog: ModelPrice[];
  modelMode: "official" | "provider" | "manual";
  modelSearch: string;
  promptPresetId: string;
  currency: string;
  onSelectModel: (modelId: string, mode: "official" | "provider" | "manual") => void;
  onModelSearchChange: (value: string) => void;
  onPromptPresetChange: (value: string) => void;
  onStart: () => void;
  onStop: () => void;
  onFetchModels: () => void;
  onRefreshPricingCatalog: () => void;
  onRandomPrompt: () => void;
  onGeneratePrompt: () => void;
}

function Workbench(props: WorkbenchProps) {
  const {
    form,
    updateField,
    busy,
    progress,
    logs,
    chartData,
    currentReport,
    providerModels,
    pricingCatalog,
    modelMode,
    modelSearch,
    promptPresetId,
    currency,
    onSelectModel,
    onModelSearchChange,
    onPromptPresetChange,
    onStart,
    onStop,
    onFetchModels,
    onRefreshPricingCatalog,
    onRandomPrompt,
    onGeneratePrompt
  } = props;
  const [officialModelOpen, setOfficialModelOpen] = useState(false);
  const [logOpen, setLogOpen] = useState(false);
  const [logPage, setLogPage] = useState(1);
  const isRunning = busy === "run";
  const estimatedCost = progress?.estimated_cost ?? currentReport?.estimated_cost ?? 0;
  const finalEstimatedUsage = form.current_usd + form.target_usd;
  const totalTokens = progress?.total_tokens ?? currentReport?.request_logs.reduce((sum, log) => sum + log.total_tokens, 0) ?? 0;
  const requestCount = progress?.request_count ?? logs.length;
  const latestFirstTokenLatencyMs = [...logs]
    .reverse()
    .find((log) => typeof log.first_token_latency_ms === "number")?.first_token_latency_ms;
  const latencyLogs = logs.filter((log) => log.status === "success" && log.latency_ms > 0);
  const averageLatencyMs = latencyLogs.length
    ? latencyLogs.reduce((sum, log) => sum + log.latency_ms, 0) / latencyLogs.length
    : null;
  const failureRate = requestCount > 0 ? ((progress?.failed_count ?? logs.filter((log) => log.status === "error").length) / requestCount) * 100 : 0;
  const totalLogPages = Math.max(1, Math.ceil(logs.length / LOG_PAGE_SIZE));
  const currentLogPage = Math.min(logPage, totalLogPages);
  const pageStart = (currentLogPage - 1) * LOG_PAGE_SIZE;
  const pagedLogs = logs.slice(pageStart, pageStart + LOG_PAGE_SIZE);
  const visibleCatalog = useMemo(() => {
    const normalized = modelSearch.trim().toLowerCase();
    const preferredProviders = new Set([
      "OpenAI",
      "Anthropic",
      "Google",
      "DeepSeek",
      "Z.ai",
      "Xiaomi MiMo",
      "Alibaba Cloud",
      "Google Vertex AI",
      "Azure OpenAI",
      "AWS Bedrock"
    ]);
    const filtered = pricingCatalog.filter((price) => {
      if (normalized) {
        return `${price.provider} ${price.id} ${price.displayName}`.toLowerCase().includes(normalized);
      }
      return preferredProviders.has(price.provider);
    });
    return filtered.slice(0, 120);
  }, [modelSearch, pricingCatalog]);
  const providerKnownModels = providerModels.filter((item) => findModelPriceInCatalog(item.id, pricingCatalog));
  const providerUnknownModels = providerModels.filter((item) => !findModelPriceInCatalog(item.id, pricingCatalog));
  const selectOfficialModel = (modelId: string) => {
    onSelectModel(modelId, "official");
    setOfficialModelOpen(false);
  };

  useEffect(() => {
    if (logPage > totalLogPages) {
      setLogPage(totalLogPages);
    }
  }, [logPage, totalLogPages]);

  return (
    <div className="workbench-grid">
      <section className="panel config-panel">
        <div className="panel-title">
          <h3>平台配置</h3>
          <span>OpenAI 兼容接口</span>
        </div>
        <div className="field-grid">
          <label>
            Base URL / 自定义路径
            <input
              value={form.base_url}
              onChange={(event) => updateField("base_url", event.target.value)}
              placeholder="https://api.example.com/proxy/openai/v1"
            />
          </label>
          <label>
            API Key
            <input
              type="password"
              value={form.api_key}
              onChange={(event) => updateField("api_key", event.target.value)}
              placeholder="请使用测试Key来进行测试"
              autoComplete="off"
            />
          </label>
          <label className="input-with-unit">
            <span className="field-caption">
              当前用量
              <small>输入当前key的使用量</small>
            </span>
            <input
              type="number"
              min="0"
              step="0.000001"
              value={form.current_usd}
              onChange={(event) => updateField("current_usd", Number(event.target.value))}
            />
            <span className="unit">USD</span>
          </label>
          <label className="input-with-unit">
            <span className="field-caption">
              目标消耗
              <small>本次任务目标消耗多少</small>
            </span>
            <input
              type="number"
              min="0.000001"
              step="0.000001"
              value={form.target_usd}
              onChange={(event) => updateField("target_usd", Number(event.target.value))}
            />
            <span className="unit">USD</span>
          </label>
          <label className="input-with-unit">
            <span className="field-caption">
              并发数
              <small>同时发起的测试请求</small>
            </span>
            <input
              type="number"
              min="1"
              max="20"
              step="1"
              value={form.concurrency}
              onChange={(event) => updateField("concurrency", Number(event.target.value))}
            />
            <span className="unit">路</span>
          </label>
        </div>

        <div className="model-section">
          <div className="section-head">
            <div>
              <strong>模型</strong>
            </div>
            <div className="mini-actions">
              <button className="secondary compact" disabled={busy !== null} onClick={onRefreshPricingCatalog}>
                {busy === "pricing" ? <Loader2 className="spin" aria-hidden="true" /> : <RefreshCw aria-hidden="true" />}
                更新价格库
              </button>
              <button className="secondary compact" disabled={busy !== null} onClick={() => setOfficialModelOpen(true)}>
                <BookOpen aria-hidden="true" />
                官方模型
              </button>
              <button className="secondary compact" disabled={busy !== null} onClick={onFetchModels}>
                {busy === "models" ? <Loader2 className="spin" aria-hidden="true" /> : <ListChecks aria-hidden="true" />}
                获取平台模型
              </button>
            </div>
          </div>

          <div className="provider-models">
            <strong>接口返回模型</strong>
            <div className="provider-model-table-wrap">
              <table className="model-table">
                <thead>
                  <tr>
                    <th></th>
                    <th>模型</th>
                    <th>匹配</th>
                    <th>输入 / 1M</th>
                    <th>缓存 / 1M</th>
                    <th>输出 / 1M</th>
                  </tr>
                </thead>
                <tbody>
                  {[...providerKnownModels, ...providerUnknownModels].slice(0, 120).map((model) => {
                    const price = findModelPriceInCatalog(model.id, pricingCatalog);
                    return (
                      <tr
                        key={model.id}
                        className={modelMode === "provider" && form.model === model.id ? "selected" : ""}
                        onClick={() => onSelectModel(model.id, "provider")}
                      >
                        <td>
                          <input
                            type="radio"
                            name="model"
                            checked={modelMode === "provider" && form.model === model.id}
                            onChange={() => onSelectModel(model.id, "provider")}
                          />
                        </td>
                        <td>
                          <strong>{model.id}</strong>
                          {model.owned_by ? <small>{model.owned_by}</small> : null}
                        </td>
                        <td>{price?.provider ?? "-"}</td>
                        <td>{price ? `$${price.inputUsdPer1M}` : "-"}</td>
                        <td>{price ? `$${price.cachedInputUsdPer1M ?? price.inputUsdPer1M}` : "-"}</td>
                        <td>{price ? `$${price.outputUsdPer1M}` : "-"}</td>
                      </tr>
                    );
                  })}
                  {!providerModels.length ? (
                    <tr>
                      <td colSpan={6} className="table-empty">
                        点击“获取平台模型”后显示平台返回的模型
                      </td>
                    </tr>
                  ) : null}
                </tbody>
              </table>
            </div>
          </div>

          <label className="manual-model">
            <span className="manual-model-head">
              <input
                type="radio"
                name="model"
                checked={modelMode === "manual"}
                onChange={() => onSelectModel(form.model, "manual")}
              />
              手动输入模型
            </span>
            <input
              value={form.model}
              onFocus={() => onSelectModel(form.model, "manual")}
              onChange={(event) => updateField("model", event.target.value)}
            />
          </label>
        </div>

        <div className="field-grid price-grid">
          <label>
            输入单价 / 1M tokens
            <input
              type="number"
              min="0"
              step="0.000001"
              value={form.input_price_per_1m}
              onChange={(event) => updateField("input_price_per_1m", Number(event.target.value))}
            />
          </label>
          <label>
            缓存读单价 / 1M tokens
            <input
              type="number"
              min="0"
              step="0.000001"
              value={form.cached_input_price_per_1m}
              onChange={(event) => updateField("cached_input_price_per_1m", Number(event.target.value))}
            />
          </label>
          <label>
            输出单价 / 1M tokens
            <input
              type="number"
              min="0"
              step="0.000001"
              value={form.output_price_per_1m}
              onChange={(event) => updateField("output_price_per_1m", Number(event.target.value))}
            />
          </label>
          <label>
            计费倍率
            <input
              type="number"
              min="0"
              step="0.01"
              value={form.billing_multiplier}
              onChange={(event) => updateField("billing_multiplier", Number(event.target.value))}
            />
          </label>
        </div>

      </section>

      <div className="side-stack">
        <section className="panel meter-panel">
          <div className="panel-title">
            <h3>实时消耗</h3>
            <button className="secondary compact" onClick={() => setLogOpen(true)}>
              请求日志
            </button>
          </div>
          <div className="metric-grid">
            <Metric label="本次理论消耗" value={formatCurrency(estimatedCost, currency)} />
            <Metric label="请求次数" value={requestCount.toLocaleString()} />
            <Metric label="首 token 耗时" value={formatLatencyMs(latestFirstTokenLatencyMs)} />
            <Metric label="总平均耗时" value={formatLatencySeconds(averageLatencyMs)} />
            <Metric label="累计 tokens" value={totalTokens.toLocaleString()} />
            <Metric label="失败率" value={`${failureRate.toFixed(1)}%`} />
            <Metric label="目标进度" value={`${Math.min(100, (estimatedCost / form.target_usd) * 100 || 0).toFixed(1)}%`} />
            <Metric label="预计最终用量" value={formatCurrency(finalEstimatedUsage, currency)} />
          </div>
          <div className="chart-box">
            {chartData.length ? (
              <ResponsiveContainer width="100%" height={190}>
                <AreaChart data={chartData} margin={{ top: 8, right: 14, bottom: 0, left: 18 }}>
                  <defs>
                    <linearGradient id="costGradient" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stopColor="#2563eb" stopOpacity={0.45} />
                      <stop offset="95%" stopColor="#2563eb" stopOpacity={0.03} />
                    </linearGradient>
                  </defs>
                  <CartesianGrid strokeDasharray="3 3" stroke="#d8dee8" />
                  <XAxis dataKey="name" tick={{ fontSize: 12 }} />
                  <YAxis tick={{ fontSize: 12 }} width={68} tickFormatter={(value) => `$${Number(value).toFixed(4)}`} />
                  <Tooltip />
                  <Area type="monotone" dataKey="cost" stroke="#2563eb" fill="url(#costGradient)" />
                </AreaChart>
              </ResponsiveContainer>
            ) : (
              <div className="empty-state">等待测试数据</div>
            )}
          </div>
        </section>

        <section className="panel prompt-panel">
          <div className="prompt-section">
            <div className="section-head">
              <div>
                <strong>测试内容</strong>
              </div>
              <div className="mini-actions">
                <button className="secondary compact" disabled={busy !== null} onClick={onRandomPrompt}>
                  随机内置
                </button>
                <button className="secondary compact" disabled={busy !== null} onClick={onGeneratePrompt}>
                  {busy === "prompt" ? <Loader2 className="spin" aria-hidden="true" /> : <WandSparkles aria-hidden="true" />}
                  API 生成
                </button>
              </div>
            </div>
            <div className="prompt-presets prompt-list">
              {PROMPT_LIBRARY.map((item) => (
                <label className="prompt-option" key={item.id}>
                  <input
                    type="radio"
                    name="prompt-preset"
                    checked={promptPresetId === item.id}
                    onChange={() => {
                      onPromptPresetChange(item.id);
                      updateField("prompt", item.prompt);
                    }}
                  />
                  <span>{item.label}</span>
                </label>
              ))}
              {promptPresetId === "generated" ? (
                <label className="prompt-option">
                  <input type="radio" name="prompt-preset" checked readOnly />
                  <span>API 生成</span>
                </label>
              ) : null}
            </div>
            <label className="wide-field">
              <textarea value={form.prompt} onChange={(event) => updateField("prompt", event.target.value)} />
            </label>
          </div>
          <div className="action-row">
            <button className="primary" disabled={busy !== null} onClick={onStart}>
              <Play aria-hidden="true" />
              开始测试
            </button>
            <button className="danger" disabled={!isRunning} onClick={onStop}>
              <Square aria-hidden="true" />
              停止
            </button>
          </div>
        </section>
      </div>

      {officialModelOpen ? (
        <div className="modal-backdrop" role="presentation" onMouseDown={() => setOfficialModelOpen(false)}>
          <section className="modal-panel model-modal" role="dialog" aria-modal="true" onMouseDown={(event) => event.stopPropagation()}>
            <div className="modal-head">
              <h3>官方模型</h3>
              <button className="icon-button" onClick={() => setOfficialModelOpen(false)} aria-label="关闭">
                <X aria-hidden="true" />
              </button>
            </div>
            <label className="model-search" aria-label="搜索官方模型">
              <input
                value={modelSearch}
                onChange={(event) => onModelSearchChange(event.target.value)}
                placeholder="输入模型或厂商，例如 gpt-5、claude、gemini"
              />
            </label>
            <div className="model-choice-list modal-table-wrap">
              <table className="model-table">
                <thead>
                  <tr>
                    <th></th>
                    <th>模型</th>
                    <th>厂商</th>
                    <th>输入 / 1M</th>
                    <th>缓存 / 1M</th>
                    <th>输出 / 1M</th>
                  </tr>
                </thead>
                <tbody>
                  {visibleCatalog.map((price) => (
                    <tr
                      key={`${price.provider}:${price.id}`}
                      className={modelMode === "official" && form.model === price.id ? "selected" : ""}
                      onClick={() => selectOfficialModel(price.id)}
                    >
                      <td>
                        <input
                          type="radio"
                          name="model"
                          checked={modelMode === "official" && form.model === price.id}
                          onChange={() => selectOfficialModel(price.id)}
                        />
                      </td>
                      <td>
                        <strong>{price.displayName}</strong>
                      </td>
                      <td>{price.provider}</td>
                      <td>${price.inputUsdPer1M}</td>
                      <td>${price.cachedInputUsdPer1M ?? price.inputUsdPer1M}</td>
                      <td>${price.outputUsdPer1M}</td>
                    </tr>
                  ))}
                  {!visibleCatalog.length ? (
                    <tr>
                      <td colSpan={6} className="table-empty">
                        没有匹配的价格条目
                      </td>
                    </tr>
                  ) : null}
                </tbody>
              </table>
            </div>
          </section>
        </div>
      ) : null}

      {logOpen ? (
        <div className="modal-backdrop" role="presentation" onMouseDown={() => setLogOpen(false)}>
          <section className="modal-panel log-modal" role="dialog" aria-modal="true" onMouseDown={(event) => event.stopPropagation()}>
            <div className="modal-head">
              <h3>请求日志</h3>
              <button className="icon-button" onClick={() => setLogOpen(false)} aria-label="关闭">
                <X aria-hidden="true" />
              </button>
            </div>
            <div className="table-wrap log-table-wrap">
              <table className="log-table">
                <thead>
                  <tr>
                    <th>#</th>
                    <th>状态</th>
                    <th>耗时</th>
                    <th>首 token</th>
                    <th>Tokens</th>
                    <th>缓存</th>
                    <th>理论消耗</th>
                    <th>摘要 / 错误</th>
                  </tr>
                </thead>
                <tbody>
                  {pagedLogs.map((log) => (
                    <tr key={log.request_index}>
                      <td>{log.request_index}</td>
                      <td>
                        <span className={`status-pill ${log.status}`}>{log.status === "success" ? "成功" : "错误"}</span>
                      </td>
                      <td>{log.latency_ms}ms</td>
                      <td>{formatLatencyMs(log.first_token_latency_ms)}</td>
                      <td>{log.total_tokens}</td>
                      <td>{log.cached_prompt_tokens}</td>
                      <td className="cost-cell">{formatCurrency(log.estimated_cost, currency)}</td>
                      <td>{log.error_message || log.response_summary}</td>
                    </tr>
                  ))}
                  {!logs.length ? (
                    <tr>
                      <td colSpan={8} className="table-empty">
                        暂无请求
                      </td>
                    </tr>
                  ) : null}
                </tbody>
              </table>
            </div>
            <div className="pagination">
              <button className="secondary compact" disabled={currentLogPage <= 1} onClick={() => setLogPage((page) => Math.max(1, page - 1))}>
                <ChevronLeft aria-hidden="true" />
                上一页
              </button>
              <span>
                {currentLogPage} / {totalLogPages}
              </span>
              <button className="secondary compact" disabled={currentLogPage >= totalLogPages} onClick={() => setLogPage((page) => Math.min(totalLogPages, page + 1))}>
                下一页
                <ChevronRight aria-hidden="true" />
              </button>
            </div>
          </section>
        </div>
      ) : null}
    </div>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="metric">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function normalizeError(err: unknown) {
  if (typeof err === "string") {
    return err;
  }
  if (err instanceof Error) {
    return err.message;
  }
  return "操作失败";
}

function formatLatencyMs(value?: number | null) {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    return "-";
  }
  return `${Math.round(value)}ms`;
}

function formatLatencySeconds(value?: number | null) {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    return "-";
  }
  return `${(value / 1000).toFixed(2)}s`;
}

function statusLabel(status: string) {
  const labels: Record<string, string> = {
    running: "运行中",
    completed: "已完成",
    stopped: "已停止",
    paused_on_failures: "连续失败暂停",
    stopped_on_budget_guard: "预算保护停止",
    failed: "失败"
  };
  return labels[status] ?? status;
}

function deriveProviderName(baseUrl: string) {
  try {
    return new URL(baseUrl).host || "测试平台";
  } catch {
    return "测试平台";
  }
}

function mergePriceCatalog(primary: ModelPrice[], fallback: ModelPrice[]) {
  const seen = new Set<string>();
  const merged: ModelPrice[] = [];
  for (const price of [...primary, ...fallback]) {
    const key = `${price.provider}:${price.id}`.toLowerCase();
    if (seen.has(key)) {
      continue;
    }
    seen.add(key);
    merged.push(price);
  }
  return merged;
}

export default App;
