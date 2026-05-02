use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use tokio::net::lookup_host;
use url::Url;

use super::models::{ProbeInput, StartTestRunInput};

pub async fn validate_base_url(raw: &str) -> Result<String, String> {
    let parsed = Url::parse(raw.trim()).map_err(|err| format!("Base URL 无效：{err}"))?;
    let scheme = parsed.scheme();
    let host = parsed
        .host_str()
        .ok_or_else(|| "Base URL 缺少 host".to_string())?;
    let is_localhost = matches!(host, "localhost" | "127.0.0.1" | "::1");
    let allow_local_dev = cfg!(debug_assertions);

    if scheme != "https" {
        if !(allow_local_dev && scheme == "http" && is_localhost) {
            return Err("Base URL 必须使用 https；开发模式仅允许 http://localhost".to_string());
        }
    }

    if is_localhost {
        if allow_local_dev {
            return Ok(normalize_url(parsed));
        }
        return Err("生产模式禁止请求 localhost".to_string());
    }

    if let Ok(ip) = host.parse::<IpAddr>() {
        reject_private_ip(ip)?;
        return Ok(normalize_url(parsed));
    }

    let port = parsed.port_or_known_default().unwrap_or(443);
    let resolved = lookup_host((host, port))
        .await
        .map_err(|err| format!("无法解析 Base URL host：{err}"))?;

    for addr in resolved {
        reject_private_ip(addr.ip())?;
    }

    Ok(normalize_url(parsed))
}

pub fn validate_probe_input(input: &ProbeInput) -> Result<(), String> {
    if input.api_key.trim().is_empty() {
        return Err("API Key 不能为空".to_string());
    }
    if input.model.trim().is_empty() {
        return Err("模型不能为空".to_string());
    }
    if !input.input_price_per_1m.is_finite() || input.input_price_per_1m < 0.0 {
        return Err("输入单价必须是非负数字".to_string());
    }
    if !input.cached_input_price_per_1m.is_finite() || input.cached_input_price_per_1m < 0.0 {
        return Err("缓存读单价必须是非负数字".to_string());
    }
    if !input.output_price_per_1m.is_finite() || input.output_price_per_1m < 0.0 {
        return Err("输出单价必须是非负数字".to_string());
    }
    if !input.billing_multiplier.is_finite() || input.billing_multiplier < 0.0 {
        return Err("计费倍率必须是非负数字".to_string());
    }
    if input.max_tokens == 0 {
        return Err("最大输出 tokens 必须大于 0".to_string());
    }
    if input.timeout_secs < 5 {
        return Err("请求超时不能低于 5 秒".to_string());
    }
    Ok(())
}

pub fn validate_start_input(input: &StartTestRunInput) -> Result<(), String> {
    validate_probe_input(&input.clone().into())?;
    if input.prompt.trim().is_empty() {
        return Err("测试内容不能为空".to_string());
    }
    if !input.target_usd.is_finite() || input.target_usd <= 0.0 {
        return Err("目标消耗必须大于 0".to_string());
    }
    if !input.current_usd.is_finite() || input.current_usd < 0.0 {
        return Err("当前用量必须是非负数字".to_string());
    }
    if input.concurrency == 0 || input.concurrency > 20 {
        return Err("并发数必须在 1 到 20 之间".to_string());
    }
    if !input.balance_before.is_finite() || input.balance_before < 0.0 {
        return Err("测试前余额必须是非负数字".to_string());
    }
    Ok(())
}

fn normalize_url(mut parsed: Url) -> String {
    parsed.set_fragment(None);
    parsed.set_query(None);
    parsed.as_str().trim_end_matches('/').to_string()
}

fn reject_private_ip(ip: IpAddr) -> Result<(), String> {
    let blocked = match ip {
        IpAddr::V4(ip) => is_blocked_ipv4(ip),
        IpAddr::V6(ip) => is_blocked_ipv6(ip),
    };

    if blocked {
        Err(format!("出于安全原因，禁止请求内网或敏感地址：{ip}"))
    } else {
        Ok(())
    }
}

fn is_blocked_ipv4(ip: Ipv4Addr) -> bool {
    ip.is_private()
        || ip.is_loopback()
        || ip.is_link_local()
        || ip.is_broadcast()
        || ip.is_documentation()
        || ip.is_unspecified()
        || ip.octets() == [169, 254, 169, 254]
}

fn is_blocked_ipv6(ip: Ipv6Addr) -> bool {
    ip.is_loopback() || ip.is_unspecified() || is_unique_local(ip) || is_unicast_link_local(ip)
}

fn is_unique_local(ip: Ipv6Addr) -> bool {
    (ip.segments()[0] & 0xfe00) == 0xfc00
}

fn is_unicast_link_local(ip: Ipv6Addr) -> bool {
    (ip.segments()[0] & 0xffc0) == 0xfe80
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_private_ipv4() {
        assert!(reject_private_ip(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))).is_err());
        assert!(reject_private_ip(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))).is_err());
        assert!(reject_private_ip(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))).is_ok());
    }

    #[test]
    fn rejects_metadata_address() {
        assert!(reject_private_ip(IpAddr::V4(Ipv4Addr::new(169, 254, 169, 254))).is_err());
    }
}
