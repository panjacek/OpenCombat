use serde::{Deserialize, Serialize};
use std::cmp::min;

use crate::types::Distance;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Feeling {
    UnderFire(u32),
}

pub const UNDER_FIRE_TICK: u32 = 10;
pub const UNDER_FIRE_MAX: u32 = 200;
pub const UNDER_FIRE_DANGER: u32 = 150;
pub const UNDER_FIRE_WARNING: u32 = 100;

impl Feeling {
    pub fn blast_increase_value(distance: Distance) -> u32 {
        if distance.meters() < 5 {
            150
        } else if distance.meters() < 10 {
            100
        } else {
            50
        }
    }

    pub fn proximity_bullet_increase_value(distance: Distance) -> u32 {
        if distance.meters() < 3 {
            100
        } else if distance.meters() < 10 {
            35
        } else {
            1
        }
    }

    pub fn decrease(&mut self) {
        match self {
            Feeling::UnderFire(value) => {
                if *value < UNDER_FIRE_TICK {
                    *value = 0;
                } else {
                    *value -= UNDER_FIRE_TICK
                }
            }
        }
    }

    pub fn increase(&mut self, add: u32) {
        match self {
            Feeling::UnderFire(value) => *value = min(*value + add, UNDER_FIRE_MAX),
        }
    }

    pub fn is_warning(&self) -> bool {
        match self {
            Feeling::UnderFire(value) => *value >= UNDER_FIRE_WARNING && *value < UNDER_FIRE_DANGER,
        }
    }

    pub fn is_danger(&self) -> bool {
        match self {
            Feeling::UnderFire(value) => *value >= UNDER_FIRE_DANGER && *value < UNDER_FIRE_MAX,
        }
    }

    pub fn is_max(&self) -> bool {
        match self {
            Feeling::UnderFire(value) => *value >= UNDER_FIRE_MAX,
        }
    }

    pub fn value_mut(&mut self) -> &mut u32 {
        match self {
            Feeling::UnderFire(value) => value,
        }
    }

    pub fn value(&self) -> &u32 {
        match self {
            Feeling::UnderFire(value) => value,
        }
    }

    pub fn exist(&self) -> bool {
        match self {
            Feeling::UnderFire(value) => *value > 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blast_increase_value_has_three_distance_tiers() {
        assert_eq!(Feeling::blast_increase_value(Distance::from_meters(0)), 150);
        assert_eq!(
            Feeling::blast_increase_value(Distance::from_millimeters(4999)),
            150
        );
        assert_eq!(Feeling::blast_increase_value(Distance::from_meters(5)), 100);
        assert_eq!(
            Feeling::blast_increase_value(Distance::from_millimeters(9999)),
            100
        );
        assert_eq!(Feeling::blast_increase_value(Distance::from_meters(10)), 50);
        assert_eq!(
            Feeling::blast_increase_value(Distance::from_meters(500)),
            50
        );
    }

    #[test]
    fn proximity_bullet_increase_value_has_three_distance_tiers() {
        assert_eq!(
            Feeling::proximity_bullet_increase_value(Distance::from_meters(0)),
            100
        );
        assert_eq!(
            Feeling::proximity_bullet_increase_value(Distance::from_millimeters(2999)),
            100
        );
        assert_eq!(
            Feeling::proximity_bullet_increase_value(Distance::from_meters(3)),
            35
        );
        assert_eq!(
            Feeling::proximity_bullet_increase_value(Distance::from_millimeters(9999)),
            35
        );
        assert_eq!(
            Feeling::proximity_bullet_increase_value(Distance::from_meters(10)),
            1
        );
    }

    #[test]
    fn increase_caps_at_under_fire_max() {
        let mut feeling = Feeling::UnderFire(UNDER_FIRE_MAX - 1);
        feeling.increase(10);
        assert_eq!(feeling.value(), &UNDER_FIRE_MAX);
    }

    #[test]
    fn decrease_steps_down_by_tick_and_floors_at_zero() {
        let mut feeling = Feeling::UnderFire(25);
        feeling.decrease();
        assert_eq!(feeling.value(), &(25 - UNDER_FIRE_TICK));

        let mut small = Feeling::UnderFire(UNDER_FIRE_TICK - 1);
        small.decrease();
        assert_eq!(small.value(), &0, "must floor at zero, not underflow");
    }

    #[test]
    fn severity_thresholds_are_staircased() {
        let mut warning = Feeling::UnderFire(UNDER_FIRE_WARNING);
        assert!(warning.is_warning());
        assert!(!warning.is_danger());
        assert!(!warning.is_max());

        let mut danger = Feeling::UnderFire(UNDER_FIRE_DANGER);
        assert!(!danger.is_warning());
        assert!(danger.is_danger());
        assert!(!danger.is_max());

        let mut maxed = Feeling::UnderFire(UNDER_FIRE_MAX);
        maxed.increase(1000);
        assert!(maxed.is_max());
        assert!(!maxed.is_danger());

        let mut calm = Feeling::UnderFire(UNDER_FIRE_WARNING - 1);
        assert!(!calm.is_warning());
    }
}
