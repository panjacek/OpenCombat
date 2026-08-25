use battle_core::deployment::{DeploymentReader, DeploymentReaderError};
use std::path::PathBuf;

fn temp_file(name: &str, content: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("oc_test_{}_{}", std::process::id(), name));
    std::fs::write(&path, content).expect("temp file writable");
    path
}

#[test]
fn missing_file_yields_read_error() {
    let path = std::env::temp_dir().join(format!("oc_missing_{}.json", std::process::id()));
    let result = DeploymentReader::from_file(&path);
    assert!(result.is_err(), "missing file must fail");
}

#[test]
fn invalid_json_yields_format_error() {
    let path = temp_file("invalid", "{ definitely not json");
    let result = DeploymentReader::from_file(&path);
    let error = format!("{}", result.expect_err("garbage must fail"));
    assert!(
        error.to_lowercase().contains("format"),
        "unexpected error: {error}"
    );
    let _ = std::fs::remove_file(path);
}

#[test]
fn missing_required_field_is_rejected_not_defaulted() {
    // soldier without `type_`: the exact upstream asset drift that shipped broken demos
    let path = temp_file(
        "no_type",
        r#"{
            "soldiers": [ { "uuid": 0, "side": "A", "world_point": {"x": 1.0, "y": 1.0},
                            "squad_uuid": 0, "main_weapon": null, "magazines": [],
                            "order": "Idle", "behavior": {"Idle": "StandUp"} } ],
            "vehicles": [], "boards": {}, "squad_types": {}
        }"#,
    );
    let result = DeploymentReader::from_file(&path);
    match result {
        Err(DeploymentReaderError::Format(_)) => {
            // rejected as a schema violation - the upstream drift class that
            // shipped broken demo assets must never parse silently
        }
        other => panic!("expected Format error for missing type_, got {other:?}"),
    }
    let _ = std::fs::remove_file(path);
}

#[test]
fn soldier_referencing_untyped_squad_still_parses() {
    // parsing succeeds; squad consistency is a later battle-state concern.
    // This documents the contract boundary between reader and builder.
    let path = temp_file(
        "untyped_squad_ref",
        r#"{
            "soldiers": [ { "uuid": 0, "type_": "Type1", "side": "A",
                            "world_point": {"x": 1.0, "y": 1.0},
                            "squad_uuid": 99, "main_weapon": null, "magazines": [],
                            "order": "Idle", "behavior": {"Idle": "StandUp"} } ],
            "vehicles": [], "boards": {}, "squad_types": {}
        }"#,
    );
    let deployment = DeploymentReader::from_file(&path).expect("valid schema parses");
    assert_eq!(deployment.soldiers().len(), 1);
    let _ = std::fs::remove_file(path);
}
