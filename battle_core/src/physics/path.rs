use crate::{config::ServerConfig, map::Map, types::*, utils::angleg};
use pathfinding::prelude::astar;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

pub enum PathMode {
    Walk,
    Drive(VehicleSize),
}
impl PathMode {
    pub fn include_vehicles(&self) -> bool {
        match self {
            PathMode::Walk => false,
            PathMode::Drive(_) => true,
        }
    }
}

pub const COST_AHEAD: i32 = 0;
pub const COST_DIAGONAL: i32 = 10;
pub const COST_CORNER: i32 = 20;
pub const COST_BACK_CORNER: i32 = 30;
pub const COST_BACK: i32 = 50;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, EnumIter)]
pub enum Direction {
    North,
    NorthEst,
    Est,
    SouthEst,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl Direction {
    pub fn from_angle(angle: &Angle) -> Self {
        // normalize into [0, 360) first: angleg() can return negative degrees
        // (atan2 range shifted by FRAC_PI_2), and e.g. -45deg is NorthWest
        let degrees = angle.0.to_degrees().rem_euclid(360.0);
        if degrees >= 337.5 || degrees <= 22.5 {
            Self::North
        } else if degrees > 22.5 && degrees <= 67.5 {
            Self::NorthEst
        } else if degrees > 67.5 && degrees <= 112.5 {
            Self::Est
        } else if degrees > 112.5 && degrees <= 157.5 {
            Self::SouthEst
        } else if degrees > 157.5 && degrees <= 202.5 {
            Self::South
        } else if degrees > 202.5 && degrees <= 247.5 {
            Self::SouthWest
        } else if degrees > 247.5 && degrees <= 292.5 {
            Self::West
        } else {
            Self::NorthWest
        }
    }

    pub fn modifier(&self) -> (i32, i32) {
        match self {
            Direction::NorthWest => (-1, -1),
            Direction::North => (0, -1),
            Direction::NorthEst => (1, -1),
            Direction::Est => (1, 0),
            Direction::SouthEst => (1, 1),
            Direction::South => (0, 1),
            Direction::SouthWest => (-1, 1),
            Direction::West => (-1, 0),
        }
    }

    pub fn angle_cost(&self, direction: &Direction) -> i32 {
        match self {
            Direction::North => match direction {
                Direction::North => COST_AHEAD,
                Direction::NorthEst => COST_DIAGONAL,
                Direction::Est => COST_CORNER,
                Direction::SouthEst => COST_BACK_CORNER,
                Direction::South => COST_BACK,
                Direction::SouthWest => COST_BACK_CORNER,
                Direction::West => COST_CORNER,
                Direction::NorthWest => COST_DIAGONAL,
            },
            Direction::NorthEst => match direction {
                Direction::North => COST_DIAGONAL,
                Direction::NorthEst => COST_AHEAD,
                Direction::Est => COST_DIAGONAL,
                Direction::SouthEst => COST_CORNER,
                Direction::South => COST_BACK_CORNER,
                Direction::SouthWest => COST_BACK,
                Direction::West => COST_BACK_CORNER,
                Direction::NorthWest => COST_CORNER,
            },
            Direction::Est => match direction {
                Direction::North => COST_CORNER,
                Direction::NorthEst => COST_DIAGONAL,
                Direction::Est => COST_AHEAD,
                Direction::SouthEst => COST_DIAGONAL,
                Direction::South => COST_CORNER,
                Direction::SouthWest => COST_BACK_CORNER,
                Direction::West => COST_BACK,
                Direction::NorthWest => COST_BACK_CORNER,
            },
            Direction::SouthEst => match direction {
                Direction::North => COST_BACK_CORNER,
                Direction::NorthEst => COST_CORNER,
                Direction::Est => COST_DIAGONAL,
                Direction::SouthEst => COST_AHEAD,
                Direction::South => COST_DIAGONAL,
                Direction::SouthWest => COST_CORNER,
                Direction::West => COST_BACK_CORNER,
                Direction::NorthWest => COST_BACK,
            },
            Direction::South => match direction {
                Direction::North => COST_BACK,
                Direction::NorthEst => COST_BACK_CORNER,
                Direction::Est => COST_CORNER,
                Direction::SouthEst => COST_DIAGONAL,
                Direction::South => COST_AHEAD,
                Direction::SouthWest => COST_DIAGONAL,
                Direction::West => COST_CORNER,
                Direction::NorthWest => COST_BACK_CORNER,
            },
            Direction::SouthWest => match direction {
                Direction::North => COST_BACK_CORNER,
                Direction::NorthEst => COST_BACK,
                Direction::Est => COST_BACK_CORNER,
                Direction::SouthEst => COST_CORNER,
                Direction::South => COST_DIAGONAL,
                Direction::SouthWest => COST_AHEAD,
                Direction::West => COST_DIAGONAL,
                Direction::NorthWest => COST_CORNER,
            },
            Direction::West => match direction {
                Direction::North => COST_CORNER,
                Direction::NorthEst => COST_BACK_CORNER,
                Direction::Est => COST_BACK,
                Direction::SouthEst => COST_BACK_CORNER,
                Direction::South => COST_CORNER,
                Direction::SouthWest => COST_DIAGONAL,
                Direction::West => COST_AHEAD,
                Direction::NorthWest => COST_DIAGONAL,
            },
            Direction::NorthWest => match direction {
                Direction::North => COST_DIAGONAL,
                Direction::NorthEst => COST_CORNER,
                Direction::Est => COST_BACK_CORNER,
                Direction::SouthEst => COST_BACK,
                Direction::South => COST_BACK_CORNER,
                Direction::SouthWest => COST_CORNER,
                Direction::West => COST_DIAGONAL,
                Direction::NorthWest => COST_AHEAD,
            },
        }
    }
}

// TODO : When "to" is unreachable (ex. for vehicle) do not search a path (it consume all path before stop)
pub fn find_path(
    config: &ServerConfig,
    map: &Map,
    from: &GridPoint,
    to: &GridPoint,
    exclude_first: bool,
    path_mode: &PathMode,
    start_direction: &Option<Direction>,
) -> Option<Vec<GridPoint>> {
    if !map.contains(from) || !map.contains(to) {
        return None;
    }
    let start_direction = start_direction.unwrap_or(Direction::from_angle(&angleg(to, from)));

    match astar(
        &(*from, start_direction),
        |p| map.successors(p, path_mode),
        |p| {
            (p.0.to_vec2().distance(to.to_vec2()) * config.path_finding_heuristic_coefficient)
                as i32
        },
        |p| p.0 == *to,
    ) {
        None => None,
        Some(path) => {
            if exclude_first {
                let new_path = path.0[1..].to_vec();
                if !new_path.is_empty() {
                    Some(new_path.iter().map(|x| x.0).collect())
                } else {
                    None
                }
            } else {
                Some(path.0.iter().map(|x| x.0).collect())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    fn degrees(value: f32) -> Angle {
        Angle(value.to_radians())
    }

    #[test]
    fn from_angle_maps_the_eight_compass_sectors() {
        let cases = [
            (0.0, Direction::North),
            (30.0, Direction::NorthEst),
            (90.0, Direction::Est),
            (135.0, Direction::SouthEst),
            (180.0, Direction::South),
            (225.0, Direction::SouthWest),
            (270.0, Direction::West),
            (315.0, Direction::NorthWest),
            // sector boundaries resolve to the counterclockwise neighbor
            // (e.g. exactly 22.5 degrees counts as North, not NorthEst)
            (22.5, Direction::North),
            (22.6, Direction::NorthEst),
            // negative angles normalize: regression for the NW-sector bug
            (-30.0, Direction::NorthWest),
            (-45.0, Direction::NorthWest),
            (-90.0, Direction::West),
            (350.0, Direction::North),
        ];
        for (angle, expected) in cases {
            assert_eq!(
                Direction::from_angle(&degrees(angle)),
                expected,
                "{angle} degrees"
            );
        }
    }

    #[test]
    fn modifiers_point_to_adjacent_grid_tiles() {
        let cases = [
            (Direction::North, (0, -1)),
            (Direction::NorthEst, (1, -1)),
            (Direction::Est, (1, 0)),
            (Direction::SouthEst, (1, 1)),
            (Direction::South, (0, 1)),
            (Direction::SouthWest, (-1, 1)),
            (Direction::West, (-1, 0)),
            (Direction::NorthWest, (-1, -1)),
        ];
        for (direction, modifier) in cases {
            assert_eq!(direction.modifier(), modifier);
        }
    }

    #[test]
    fn angle_cost_is_symmetric_across_all_direction_pairs() {
        for facing in Direction::iter() {
            for step in Direction::iter() {
                assert_eq!(
                    facing.angle_cost(&step),
                    step.angle_cost(&facing),
                    "cost({facing:?}, {step:?}) must equal cost({step:?}, {facing:?})"
                );
            }
        }
    }

    #[test]
    fn angle_cost_ladder_matches_constants() {
        // semantic ladder spot-checks on one facing
        assert_eq!(
            Direction::North.angle_cost(&Direction::NorthEst),
            COST_DIAGONAL
        );
        assert_eq!(Direction::North.angle_cost(&Direction::Est), COST_CORNER);
        assert_eq!(
            Direction::North.angle_cost(&Direction::SouthEst),
            COST_BACK_CORNER
        );
        assert_eq!(Direction::North.angle_cost(&Direction::South), COST_BACK);
        // and the same ladder holds rotated, via the symmetry property
        for facing in Direction::iter() {
            let clockwise_next = match facing {
                Direction::North => Direction::NorthEst,
                Direction::NorthEst => Direction::Est,
                Direction::Est => Direction::SouthEst,
                Direction::SouthEst => Direction::South,
                Direction::South => Direction::SouthWest,
                Direction::SouthWest => Direction::West,
                Direction::West => Direction::NorthWest,
                Direction::NorthWest => Direction::North,
            };
            assert_eq!(facing.angle_cost(&clockwise_next), COST_DIAGONAL);
            assert_eq!(facing.angle_cost(&facing), COST_AHEAD);
        }
    }

    #[test]
    fn path_mode_vehicle_inclusion() {
        assert!(!PathMode::Walk.include_vehicles());
        assert!(PathMode::Drive(VehicleSize(2)).include_vehicles());
    }
}
