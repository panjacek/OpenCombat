use oc_core::game::{soldier::SoldierType, squad::SquadType};
use oc_core::health::Health;
use oc_core::morale::Morale;
use oc_core::spawn::SpawnZoneName;
use std::str::FromStr;

#[test]
fn spawn_zone_names_parse_from_short_codes() {
    let cases = [
        ("N", SpawnZoneName::North),
        ("NE", SpawnZoneName::NorthEst),
        ("E", SpawnZoneName::Est),
        ("SE", SpawnZoneName::SouthEst),
        ("S", SpawnZoneName::South),
        ("SW", SpawnZoneName::SouthWest),
        ("W", SpawnZoneName::West),
        ("NW", SpawnZoneName::NorthWest),
        ("ALL", SpawnZoneName::All),
    ];
    for (code, expected) in cases {
        assert_eq!(
            SpawnZoneName::from_str(code).expect(code),
            expected,
            "parsing {code}"
        );
    }
}

#[test]
fn spawn_zone_name_rejects_unknown_codes_with_message() {
    let error = SpawnZoneName::from_str("XX").expect_err("must not parse");
    assert!(error.to_string().contains("XX"), "error should echo input");
}

#[test]
fn all_spawn_zone_code_is_not_a_placement_zone_object() {
    assert!(!SpawnZoneName::All.allowed_for_zone_object());
    let placed = [
        SpawnZoneName::North,
        SpawnZoneName::NorthEst,
        SpawnZoneName::Est,
        SpawnZoneName::SouthEst,
        SpawnZoneName::South,
        SpawnZoneName::SouthWest,
        SpawnZoneName::West,
        SpawnZoneName::NorthWest,
    ];
    assert!(placed.iter().all(|z| z.allowed_for_zone_object()));
}

#[test]
fn morale_maps_health_states_to_expected_values() {
    assert!((Morale::from_health(&Health::Good).0 - 1.0).abs() < f32::EPSILON);
    assert!((Morale::from_health(&Health::Unconscious).0 - 0.5).abs() < f32::EPSILON);
    assert!(Morale::from_health(&Health::Dead).0.abs() < f32::EPSILON);
}

#[test]
fn soldier_and_squad_type_names_match_display_names() {
    assert_eq!(SoldierType::Type1.name(), "Type 1");
    assert_eq!(SoldierType::Bren.name(), "Bren");
    assert_eq!(SoldierType::Mg34.name(), "Mg34");
    assert_eq!(SquadType::Type1.name(), "Type 1");
    assert_eq!(SquadType::Bren.name(), "Bren");
    assert_eq!(SquadType::Mg34.name(), "Mg34");
}
