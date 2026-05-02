pub fn estimate_cost(
    prompt_tokens: u32,
    cached_prompt_tokens: u32,
    completion_tokens: u32,
    input_price_per_1m: f64,
    cached_input_price_per_1m: f64,
    output_price_per_1m: f64,
) -> f64 {
    let cached_tokens = cached_prompt_tokens.min(prompt_tokens);
    let uncached_tokens = prompt_tokens - cached_tokens;
    ((uncached_tokens as f64 * input_price_per_1m)
        + (cached_tokens as f64 * cached_input_price_per_1m)
        + (completion_tokens as f64 * output_price_per_1m))
        / 1_000_000.0
}

pub fn apply_billing_multiplier(raw_cost: f64, multiplier: f64) -> f64 {
    raw_cost * multiplier
}

pub fn actual_cost(balance_before: f64, balance_after: f64) -> f64 {
    (balance_before - balance_after).max(0.0)
}

pub fn diff_ratio(actual_cost: f64, estimated_cost: f64) -> Option<f64> {
    if estimated_cost <= f64::EPSILON {
        None
    } else {
        Some((actual_cost - estimated_cost) / estimated_cost)
    }
}

pub fn estimate_tokens_from_text(value: &str) -> u32 {
    let char_count = value.chars().count() as u32;
    char_count.div_ceil(4).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimates_standard_input_output_cost() {
        let cost = estimate_cost(1_000_000, 0, 500_000, 1.25, 1.25, 10.0);
        assert!((cost - 6.25).abs() < f64::EPSILON);
    }

    #[test]
    fn estimates_cached_input_cost() {
        let cost = estimate_cost(1_000_000, 250_000, 500_000, 1.25, 0.125, 10.0);
        assert!((cost - 5.96875).abs() < f64::EPSILON);
    }

    #[test]
    fn applies_billing_multiplier() {
        assert!((apply_billing_multiplier(10.0, 0.1) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn calculates_diff_ratio() {
        assert_eq!(diff_ratio(1.2, 1.0), Some(0.19999999999999996));
        assert_eq!(diff_ratio(1.0, 0.0), None);
    }
}
