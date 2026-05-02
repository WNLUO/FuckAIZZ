#!/usr/bin/env node

import { execFileSync } from "node:child_process";

const requiredSecrets = [
  "APPLE_CERTIFICATE",
  "APPLE_CERTIFICATE_PASSWORD",
  "APPLE_SIGNING_IDENTITY",
  "APPLE_ID",
  "APPLE_PASSWORD",
  "APPLE_TEAM_ID"
];

const token = process.env.GITHUB_TOKEN || process.env.GH_TOKEN;
if (!token) {
  exitWithHelp("缺少 GITHUB_TOKEN 或 GH_TOKEN 环境变量。");
}

const repo = process.argv[2] || readRepoFromGitRemote();
if (!repo) {
  exitWithHelp("没有找到 GitHub remote。也可以手动传入：npm run check:release-secrets -- owner/repo");
}

const response = await fetch(`https://api.github.com/repos/${repo}/actions/secrets?per_page=100`, {
  headers: {
    Authorization: `Bearer ${token}`,
    Accept: "application/vnd.github+json",
    "X-GitHub-Api-Version": "2022-11-28"
  }
});

if (!response.ok) {
  const text = await response.text();
  console.error(`GitHub API 返回 ${response.status}: ${text.slice(0, 300)}`);
  process.exit(1);
}

const payload = await response.json();
const existing = new Set((payload.secrets || []).map((secret) => secret.name));
const missing = requiredSecrets.filter((name) => !existing.has(name));

console.log(`Repo: ${repo}`);
console.log(`已配置发布 secrets: ${requiredSecrets.filter((name) => existing.has(name)).join(", ") || "无"}`);

if (missing.length) {
  console.log(`缺少: ${missing.join(", ")}`);
  process.exit(2);
}

console.log("macOS 签名/公证所需 secrets 已齐。");

function readRepoFromGitRemote() {
  try {
    const output = execFileSync("git", ["remote", "get-url", "origin"], { encoding: "utf8" }).trim();
    const sshMatch = output.match(/github\.com[:/]([^/]+\/[^/.]+)(?:\.git)?$/);
    const httpsMatch = output.match(/github\.com\/([^/]+\/[^/.]+)(?:\.git)?$/);
    return (sshMatch?.[1] || httpsMatch?.[1] || "").trim();
  } catch {
    return "";
  }
}

function exitWithHelp(message) {
  console.error(message);
  console.error("示例：GITHUB_TOKEN=ghp_xxx npm run check:release-secrets -- owner/repo");
  process.exit(1);
}
