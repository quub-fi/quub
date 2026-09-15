use quub_primitives::PolicyReason;

pub fn reason_name(code: u16) -> &'static str {
    match code {
        x if x == PolicyReason::Allow.as_u16() => "allow",
        x if x == PolicyReason::FrozenFrom.as_u16() => "frozen_from",
        x if x == PolicyReason::FrozenTo.as_u16() => "frozen_to",
        x if x == PolicyReason::NotAllowlisted.as_u16() => "not_allowlisted",
        x if x == PolicyReason::AmountOverLimit.as_u16() => "amount_over_limit",
        x if x == PolicyReason::MissingTravelRule.as_u16() => "missing_travel_rule",
        x if x == PolicyReason::DualControlRequired.as_u16() => "dual_control_required",
        x if x == PolicyReason::Paused.as_u16() => "paused",
        x if x == PolicyReason::Malformed.as_u16() => "malformed",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_from_is_one() {
        assert_eq!(PolicyReason::FrozenFrom.as_u16(), 1);
        assert_eq!(reason_name(1), "frozen_from");
    }
}
