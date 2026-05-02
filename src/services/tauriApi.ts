import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  ExportFormat,
  ExportResult,
  ProbeInput,
  PromptGenerationResult,
  PricingCatalogResult,
  ProviderModel,
  StartTestRunInput,
  TestProgress,
  TestRunReport,
  TestRunSummary
} from "../types";

export function listProviderModels(input: ProbeInput) {
  return invoke<ProviderModel[]>("list_provider_models", { input });
}

export function generateTestPrompt(input: ProbeInput) {
  return invoke<PromptGenerationResult>("generate_test_prompt", { input });
}

export function refreshPricingCatalog() {
  return invoke<PricingCatalogResult>("refresh_pricing_catalog");
}

export function startTestRun(input: StartTestRunInput) {
  return invoke<TestRunReport>("start_test_run", { input });
}

export function stopTestRun() {
  return invoke<void>("stop_test_run");
}

export function listTestRuns() {
  return invoke<TestRunSummary[]>("list_test_runs");
}

export function getTestRun(reportId: string) {
  return invoke<TestRunReport>("get_test_run", { reportId });
}

export function finalizeTestRun(reportId: string, balanceAfter: number) {
  return invoke<TestRunReport>("finalize_test_run", { reportId, balanceAfter });
}

export function exportReport(reportId: string, format: ExportFormat) {
  return invoke<ExportResult>("export_report", { reportId, format });
}

export function listenToProgress(handler: (payload: TestProgress) => void): Promise<UnlistenFn> {
  return listen<TestProgress>("test-progress", (event) => handler(event.payload));
}
