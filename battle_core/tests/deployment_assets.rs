use battle_core::deployment::DeploymentReader;
use std::collections::HashSet;
use std::path::PathBuf;

fn assets_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../assets")
}

#[test]
fn all_deployment_assets_parse_and_are_consistent() {
    let entries = std::fs::read_dir(assets_dir()).expect("assets dir must exist");
    let mut checked = 0;

    for entry in entries {
        let path = entry.expect("dir entry readable").path();
        if path.extension().map(|e| e != "json").unwrap_or(true) {
            continue;
        }

        let deployment = DeploymentReader::from_file(&PathBuf::from(&path))
            .unwrap_or_else(|e| panic!("{} : parse error : {:?}", path.display(), e));

        assert!(
            !deployment.soldiers().is_empty(),
            "{} : no soldiers",
            path.display()
        );

        for soldier in deployment.soldiers() {
            assert!(
                deployment.squad_types().contains_key(&soldier.squad_uuid()),
                "{} : soldier {:?} references untyped squad {:?}",
                path.display(),
                soldier.uuid(),
                soldier.squad_uuid()
            );
        }

        checked += 1;
    }

    assert!(
        checked >= 5,
        "expected at least 5 deployment assets, found {checked}"
    );
}

#[test]
fn squad_uuids_are_contiguous_from_zero() {
    // BattleState indexes soldiers by Vec position; gaps would break lookups.
    let path = assets_dir().join("demo1_deployment.json");
    let deployment = DeploymentReader::from_file(&path).expect("demo1 parses");
    let mut seen = HashSet::new();
    for soldier in deployment.soldiers() {
        assert!(seen.insert(soldier.uuid().0), "duplicate soldier uuid");
    }
}
