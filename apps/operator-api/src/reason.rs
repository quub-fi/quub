//! Policy reason code names for JSON.

use quub_primitives::PolicyReason;

pub fn reason_name(code: u16) -> &'static str {
    match code {
        0 => "allow",
        1 => "frozen_from",
        2 => "frozen_to",
        3 => "not_allowlisted",
        4 => "amount_over_limit",
        5 => "missing_travel_rule",
        6 => "dual_control_required",
        7 => "paused",
        8 => "malformed",
        _ => "unknown",
    }
}

pub fn from_policy_reason(r: PolicyReason) -> u16 {
    r.as_u16()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_from_is_one() {
        assert_eq!(from_policy_reason(PolicyReason::FrozenFrom), 1);
        assert_eq!(reason_name(1), "frozen_from");
    }
}
