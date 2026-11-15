//! The pace of a Race or a Duel: the shot allowance, and the proposition cooldown.

/// Shots a player may fire as soon as the game starts.
pub const FREE_SHOTS: i64 = 15;
/// The first earned shot arrives that long after the start…
pub const FIRST_SHOT_MS: i64 = 5_000;
/// …and each next one that much later than the interval before it.
pub const SHOT_STEP_MS: i64 = 500;

const COOLDOWN_MS: i64 = 5_000;
const COOLDOWN_PER_WRONG_MS: i64 = 1_000;
const COOLDOWN_CAP_MS: i64 = 10_000;

/// When the k-th earned shot arrives: `5 s × k + 0.25 s × k × (k − 1)`.
const fn earned_at_ms(k: i64) -> i64 {
    FIRST_SHOT_MS * k + SHOT_STEP_MS * k * (k - 1) / 2
}

/// Shots earned since the start, kept until fired.
const fn earned(elapsed_ms: i64) -> i64 {
    let mut k = 0;
    while earned_at_ms(k + 1) <= elapsed_ms {
        k += 1;
    }
    k
}

/// The shots a player may fire now, `fired` new shots after the start.
pub const fn shots_left(elapsed_ms: i64, fired: i64) -> i64 {
    FREE_SHOTS + earned(elapsed_ms) - fired
}

/// How long before the next earned shot arrives.
pub fn next_shot_in_ms(elapsed_ms: i64) -> u64 {
    u64::try_from(earned_at_ms(earned(elapsed_ms) + 1) - elapsed_ms).unwrap_or(0)
}

/// How long before the next proposition: the first is immediate, the second waits 5 s.
pub fn proposition_in_ms(wrong_propositions: i64, since_last_ms: Option<i64>) -> u64 {
    since_last_ms.map_or(0, |since| {
        let cooldown = (COOLDOWN_MS + COOLDOWN_PER_WRONG_MS * (wrong_propositions - 1).max(0))
            .min(COOLDOWN_CAP_MS);
        u64::try_from(cooldown - since).unwrap_or(0)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn earned_shots_arrive_at_growing_intervals_and_accumulate() {
        assert_eq!(shots_left(0, 0), 15);
        assert_eq!(shots_left(4_999, 15), 0);
        assert_eq!(next_shot_in_ms(4_999), 1);
        assert_eq!(shots_left(5_000, 15), 1);
        assert_eq!(next_shot_in_ms(5_000), 5_500);
        assert_eq!(shots_left(10_500, 15), 2);
        assert_eq!(shots_left(16_500, 15), 3);
        assert_eq!(shots_left(16_499, 10), 7);
    }

    #[test]
    fn the_cooldown_grows_by_a_second_per_wrong_proposition_up_to_10_s() {
        assert_eq!(proposition_in_ms(0, None), 0);
        assert_eq!(proposition_in_ms(1, Some(0)), 5_000);
        assert_eq!(proposition_in_ms(3, Some(2_000)), 5_000);
        assert_eq!(proposition_in_ms(6, Some(0)), 10_000);
        assert_eq!(proposition_in_ms(9, Some(0)), 10_000);
        assert_eq!(proposition_in_ms(1, Some(5_000)), 0);
    }
}
