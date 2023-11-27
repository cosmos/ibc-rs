use core::time::Duration;

pub fn calculate_block_delay(
    delay_period_time: &Duration,
    max_expected_time_per_block: &Duration,
) -> u64 {
    let delay_period_time = delay_period_time.as_secs();
    let max_expected_time_per_block = max_expected_time_per_block.as_secs();
    if max_expected_time_per_block == 0 {
        return 0;
    }
    if delay_period_time % max_expected_time_per_block == 0 {
        return delay_period_time / max_expected_time_per_block;
    }

    // TODO: Use `u64::div_ceil` here instead
    (delay_period_time / max_expected_time_per_block) + 1
}

#[cfg(test)]
mod tests {

    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::remainder_zero(10, 2, 5)]
    #[case::remainder_not_zero(10, 3, 4)]
    #[case::max_expected_zero(10, 0, 0)]
    #[case::delay_period_zero(0, 2, 0)]
    #[case::both_zero(0, 0, 0)]
    #[case::delay_less_than_max(10, 11, 1)]
    fn test_calculate_block_delay_zero(
        #[case] delay_period_time: u64,
        #[case] max_expected_time_per_block: u64,
        #[case] expected: u64,
    ) {
        assert_eq!(
            calculate_block_delay(
                &Duration::from_secs(delay_period_time),
                &Duration::from_secs(max_expected_time_per_block)
            ),
            expected
        );
    }
}
