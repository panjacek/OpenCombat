use battle_core::config::ServerConfig;
use battle_core::deployment::DeploymentReader;
use battle_core::map::reader::MapReader;
use battle_core::physics::path::{find_path, PathMode};
use battle_core::types::GridPoint;
use std::path::{Path, PathBuf};

fn demo1_map() -> battle_core::map::Map {
    let resources = Path::new(env!("CARGO_MANIFEST_DIR")).join("../resources");
    MapReader::new("Demo1", &resources)
        .expect("Demo1 map must load")
        .build()
        .expect("Demo1 map must build")
}

/// Soldiers spawn inside the map; take a deployment squad position as a sane open tile.
fn first_deployment_grid_point(map: &battle_core::map::Map) -> GridPoint {
    let deployment_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../assets/demo1_deployment.json");
    let deployment = DeploymentReader::from_file(&deployment_path).expect("demo1 parses");
    let world_point = deployment.soldiers()[0].world_point();
    map.grid_point_from_world_point(&world_point)
}

#[test]
fn path_exists_between_two_open_tiles_on_demo1() {
    let map = demo1_map();
    let from = first_deployment_grid_point(&map);
    // walk 20 tiles east and 10 south; Demo1 interior is wide open around spawns
    let to = GridPoint::new(from.x + 20, from.y + 10);
    assert!(map.contains(&from), "origin must be on map");
    assert!(map.contains(&to), "target must be on map");

    let config = ServerConfig::default();
    let path = find_path(&config, &map, &from, &to, false, &PathMode::Walk, &None)
        .expect("a walkable path should exist between these tiles");

    assert_eq!(path.first(), Some(&from), "path starts at origin");
    assert_eq!(path.last(), Some(&to), "path ends at target");
    assert!(path.len() > 5, "path should have intermediate steps");
}

#[test]
fn exclude_first_drops_origin_from_path() {
    let map = demo1_map();
    let from = first_deployment_grid_point(&map);
    let to = GridPoint::new(from.x + 8, from.y);
    let config = ServerConfig::default();

    let with_first = find_path(&config, &map, &from, &to, false, &PathMode::Walk, &None)
        .expect("inclusive path exists");
    let without_first = find_path(&config, &map, &from, &to, true, &PathMode::Walk, &None)
        .expect("exclusive path exists");

    assert_eq!(with_first.first(), Some(&from));
    assert_ne!(
        with_first.first(),
        without_first.first(),
        "exclude_first must drop the origin tile"
    );
    assert_eq!(with_first.last(), without_first.last());
    assert_eq!(with_first.len(), without_first.len() + 1);
}

#[test]
fn same_start_and_target_gives_single_tile_or_empty_path() {
    let map = demo1_map();
    let point = first_deployment_grid_point(&map);
    let config = ServerConfig::default();

    let full = find_path(&config, &map, &point, &point, false, &PathMode::Walk, &None)
        .expect("degenerate inclusive path exists");
    assert_eq!(full, vec![point]);

    assert!(
        find_path(&config, &map, &point, &point, true, &PathMode::Walk, &None).is_none(),
        "excluding the only tile leaves nothing"
    );
}

#[test]
fn find_path_is_deterministic_for_same_inputs() {
    let map = demo1_map();
    let from = first_deployment_grid_point(&map);
    let to = GridPoint::new(from.x + 15, from.y - 6);
    let config = ServerConfig::default();

    let a = find_path(&config, &map, &from, &to, true, &PathMode::Walk, &None);
    let b = find_path(&config, &map, &from, &to, true, &PathMode::Walk, &None);
    assert_eq!(a, b, "same inputs must yield identical paths");
}

#[test]
fn out_of_map_endpoints_return_no_path() {
    let map = demo1_map();
    let inside = first_deployment_grid_point(&map);
    let outside = GridPoint::new(-50, -50);
    assert!(!map.contains(&outside));

    let config = ServerConfig::default();
    assert!(find_path(
        &config,
        &map,
        &inside,
        &outside,
        false,
        &PathMode::Walk,
        &None
    )
    .is_none());
}
