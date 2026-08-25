use crate::types::{Distance, WorldPoint};

// Coefficient to convert distance from two scene points into meters
// TODO : fix it with sprites, maps, etc
pub const DISTANCE_TO_METERS_COEFFICIENT: f32 = 0.3;

pub fn distance_between_points(from: &WorldPoint, to: &WorldPoint) -> Distance {
    Distance::from_millimeters(
        ((from.to_vec2().distance(to.to_vec2()) * DISTANCE_TO_METERS_COEFFICIENT) * 1000.) as i64,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::WorldPoint;

    #[test]
    fn distance_between_same_points_is_zero() {
        let point = WorldPoint::new(42.0, 42.0);
        assert_eq!(distance_between_points(&point, &point).millimeters(), 0);
    }

    #[test]
    fn distance_uses_scene_to_meters_coefficient() {
        // 10 scene units * DISTANCE_TO_METERS_COEFFICIENT (0.3) = 3 meters
        let from = WorldPoint::new(0.0, 0.0);
        let to = WorldPoint::new(10.0, 0.0);
        assert_eq!(distance_between_points(&from, &to).millimeters(), 3000);
    }

    #[test]
    fn distance_is_symmetric() {
        let a = WorldPoint::new(-3.0, 8.0);
        let b = WorldPoint::new(5.0, -2.0);
        assert_eq!(
            distance_between_points(&a, &b),
            distance_between_points(&b, &a)
        );
    }
}
