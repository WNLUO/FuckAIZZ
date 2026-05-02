use std::{fs, path::PathBuf};

use chrono::Utc;
use tauri::{AppHandle, Manager};

use crate::core::{
    cost::{actual_cost, diff_ratio},
    models::{ExportResult, TestRunReport, TestRunSummary},
};

pub fn save_report(app: &AppHandle, report: &TestRunReport) -> Result<(), String> {
    let path = report_path(app, &report.id)?;
    let content =
        serde_json::to_string_pretty(report).map_err(|err| format!("序列化报告失败：{err}"))?;
    fs::write(path, content).map_err(|err| format!("写入报告失败：{err}"))
}

pub fn load_report(app: &AppHandle, report_id: &str) -> Result<TestRunReport, String> {
    validate_report_id(report_id)?;
    let path = report_path(app, report_id)?;
    let content = fs::read_to_string(path).map_err(|err| format!("读取报告失败：{err}"))?;
    serde_json::from_str(&content).map_err(|err| format!("报告文件损坏：{err}"))
}

pub fn list_reports(app: &AppHandle) -> Result<Vec<TestRunSummary>, String> {
    let dir = reports_dir(app)?;
    let mut summaries = Vec::new();

    for entry in fs::read_dir(dir).map_err(|err| format!("读取报告目录失败：{err}"))? {
        let entry = entry.map_err(|err| format!("读取报告条目失败：{err}"))?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let content = fs::read_to_string(&path).map_err(|err| format!("读取报告失败：{err}"))?;
        if let Ok(report) = serde_json::from_str::<TestRunReport>(&content) {
            summaries.push(TestRunSummary::from(&report));
        }
    }

    summaries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(summaries)
}

pub fn finalize_report(
    app: &AppHandle,
    report_id: &str,
    balance_after: f64,
) -> Result<TestRunReport, String> {
    if !balance_after.is_finite() || balance_after < 0.0 {
        return Err("测试后余额必须是非负数字".to_string());
    }

    let mut report = load_report(app, report_id)?;
    let actual = actual_cost(report.balance_before, balance_after);
    report.balance_after = Some(balance_after);
    report.actual_cost = Some(actual);
    report.diff_cost = Some(actual - report.estimated_cost);
    report.diff_ratio = diff_ratio(actual, report.estimated_cost);
    save_report(app, &report)?;
    Ok(report)
}

pub fn export_report_file(
    app: &AppHandle,
    report_id: &str,
    format: &str,
) -> Result<ExportResult, String> {
    let report = load_report(app, report_id)?;
    let exports_dir = exports_dir(app)?;
    let (extension, content) = match format {
        "json" => (
            "json",
            serde_json::to_string_pretty(&report)
                .map_err(|err| format!("序列化 JSON 报告失败：{err}"))?,
        ),
        "markdown" => ("md", markdown_report(&report)),
        "csv" => ("csv", csv_report(&report)?),
        _ => return Err("不支持的导出格式".to_string()),
    };
    let file_name = format!(
        "{}-{}.{}",
        sanitize_file_component(&report.provider_name),
        report.created_at.replace(':', "-"),
        extension
    );
    let path = exports_dir.join(file_name);
    fs::write(&path, &content).map_err(|err| format!("写入导出文件失败：{err}"))?;
    Ok(ExportResult {
        path: path.to_string_lossy().to_string(),
        content,
    })
}

fn reports_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|err| format!("获取应用数据目录失败：{err}"))?
        .join("reports");
    fs::create_dir_all(&dir).map_err(|err| format!("创建报告目录失败：{err}"))?;
    Ok(dir)
}

fn exports_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|err| format!("获取应用数据目录失败：{err}"))?
        .join("exports");
    fs::create_dir_all(&dir).map_err(|err| format!("创建导出目录失败：{err}"))?;
    Ok(dir)
}

fn report_path(app: &AppHandle, report_id: &str) -> Result<PathBuf, String> {
    validate_report_id(report_id)?;
    Ok(reports_dir(app)?.join(format!("{report_id}.json")))
}

fn validate_report_id(report_id: &str) -> Result<(), String> {
    let valid = report_id
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-');
    if report_id.is_empty() || !valid {
        return Err("报告 ID 无效".to_string());
    }
    Ok(())
}

fn markdown_report(report: &TestRunReport) -> String {
    let verdict = verdict_text(report.diff_ratio);
    let mut output = String::new();
    output.push_str("# AI 中转额度消耗检测报告\n\n");
    output.push_str(&format!("- 测试时间：{}\n", report.created_at));
    output.push_str(&format!("- 应用版本：{}\n", report.app_version));
    output.push_str(&format!("- 平台 Base URL：{}\n", report.base_url));
    output.push_str(&format!("- 模型名：{}\n", report.model_requested));
    if let Some(model_reported) = &report.model_reported {
        output.push_str(&format!("- 响应模型：{}\n", model_reported));
    }
    output.push_str(&format!(
        "- 单价配置：输入 {}/1M，缓存读 {}/1M，输出 {}/1M，倍率 {}x\n",
        report.input_price_per_1m,
        report.cached_input_price_per_1m,
        report.output_price_per_1m,
        report.billing_multiplier
    ));
    output.push_str(&format!("- 当前用量：{:.8}\n", report.current_usd));
    output.push_str(&format!("- 本次目标消耗：{:.8}\n", report.target_usd));
    output.push_str(&format!("- 测试前余额：{}\n", report.balance_before));
    output.push_str(&format!(
        "- 测试后余额：{}\n",
        format_optional(report.balance_after)
    ));
    output.push_str(&format!("- 理论消耗：{:.8}\n", report.estimated_cost));
    output.push_str(&format!(
        "- 实际扣除：{}\n",
        format_optional(report.actual_cost)
    ));
    output.push_str(&format!(
        "- 偏差金额：{}\n",
        format_optional(report.diff_cost)
    ));
    output.push_str(&format!(
        "- 偏差比例：{}\n",
        report
            .diff_ratio
            .map(|value| format!("{:.2}%", value * 100.0))
            .unwrap_or_else(|| "-".to_string())
    ));
    output.push_str(&format!("- 结论提示：{}\n", verdict));
    output.push_str(&format!("- usage 来源：{:?}\n\n", report.usage_source));
    output.push_str("## 请求明细\n\n");
    output.push_str("| # | 状态 | 耗时 ms | 首 token ms | Prompt tokens | Cached tokens | Completion tokens | Total tokens | 原始消耗 | 理论消耗 | 摘要 / 错误 |\n");
    output.push_str("|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---|\n");
    for log in &report.request_logs {
        output.push_str(&format!(
            "| {} | {:?} | {} | {} | {} | {} | {} | {} | {:.8} | {:.8} | {} |\n",
            log.request_index,
            log.status,
            log.latency_ms,
            format_optional_u128(log.first_token_latency_ms),
            log.prompt_tokens,
            log.cached_prompt_tokens,
            log.completion_tokens,
            log.total_tokens,
            log.raw_estimated_cost,
            log.estimated_cost,
            escape_markdown_table(
                log.error_message
                    .as_deref()
                    .unwrap_or(&log.response_summary)
            )
        ));
    }
    output
}

fn csv_report(report: &TestRunReport) -> Result<String, String> {
    let mut writer = csv::Writer::from_writer(Vec::new());
    writer
        .write_record([
            "request_index",
            "status",
            "latency_ms",
            "first_token_latency_ms",
            "prompt_tokens",
            "cached_prompt_tokens",
            "completion_tokens",
            "total_tokens",
            "raw_estimated_cost",
            "estimated_cost",
            "usage_source",
            "response_summary",
            "error_message",
            "created_at",
        ])
        .map_err(|err| format!("写入 CSV 失败：{err}"))?;

    for log in &report.request_logs {
        writer
            .write_record([
                log.request_index.to_string(),
                format!("{:?}", log.status),
                log.latency_ms.to_string(),
                log.first_token_latency_ms
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                log.prompt_tokens.to_string(),
                log.cached_prompt_tokens.to_string(),
                log.completion_tokens.to_string(),
                log.total_tokens.to_string(),
                format!("{:.8}", log.raw_estimated_cost),
                format!("{:.8}", log.estimated_cost),
                format!("{:?}", log.usage_source),
                log.response_summary.clone(),
                log.error_message.clone().unwrap_or_default(),
                log.created_at.clone(),
            ])
            .map_err(|err| format!("写入 CSV 失败：{err}"))?;
    }

    let bytes = writer
        .into_inner()
        .map_err(|err| format!("生成 CSV 失败：{err}"))?;
    String::from_utf8(bytes).map_err(|err| format!("CSV 编码失败：{err}"))
}

fn verdict_text(diff_ratio: Option<f64>) -> &'static str {
    match diff_ratio.map(f64::abs) {
        Some(value) if value >= 0.1 => "存在显著账单差异",
        Some(_) => "未发现显著账单差异",
        None => "余额信息不足，暂无法判断",
    }
}

fn format_optional(value: Option<f64>) -> String {
    value
        .map(|item| format!("{item:.8}"))
        .unwrap_or_else(|| "-".to_string())
}

fn format_optional_u128(value: Option<u128>) -> String {
    value
        .map(|item| item.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn escape_markdown_table(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

fn sanitize_file_component(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
        .collect();
    if sanitized.is_empty() {
        format!("report-{}", Utc::now().timestamp())
    } else {
        sanitized
    }
}
