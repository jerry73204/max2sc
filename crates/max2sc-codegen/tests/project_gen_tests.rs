//! Project generation tests

use max2sc_codegen::generate_project;
use max2sc_parser::parse_patch_string;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_generate_basic_project() {
    let simple_patch = r#"{
        "patcher": {
            "rect": [0.0, 0.0, 640.0, 480.0],
            "boxes": [
                {
                    "box": {
                        "id": "obj-1",
                        "maxclass": "newobj",
                        "text": "cycle~ 440",
                        "numinlets": 2,
                        "numoutlets": 1
                    }
                },
                {
                    "box": {
                        "id": "obj-2", 
                        "maxclass": "newobj",
                        "text": "dac~",
                        "numinlets": 2,
                        "numoutlets": 0
                    }
                }
            ],
            "lines": [
                {
                    "patchline": {
                        "source": ["obj-1", 0],
                        "destination": ["obj-2", 0]
                    }
                }
            ]
        }
    }"#;

    let patch = parse_patch_string(simple_patch).expect("Failed to parse patch");
    let output_dir = tempdir().expect("Failed to create temp dir");

    let result = generate_project(&patch, output_dir.path());
    assert!(
        result.is_ok(),
        "Failed to generate project: {:?}",
        result.err()
    );

    let project = result.unwrap();

    // Check project structure
    assert_eq!(project.main_file, "main.scd");
    assert!(
        !project.synth_defs.is_empty(),
        "Should have generated SynthDefs"
    );
    assert_eq!(
        project.bus_config.audio_buses.len(),
        3,
        "Should have 3 audio buses"
    );
    assert_eq!(
        project.bus_config.control_buses.len(),
        2,
        "Should have 2 control buses"
    );

    // Check files were created
    assert!(
        output_dir.path().join("main.scd").exists(),
        "main.scd should exist"
    );
    assert!(
        output_dir.path().join("cleanup.scd").exists(),
        "cleanup.scd should exist"
    );
    assert!(
        output_dir.path().join("lib").join("SynthDefs.scd").exists(),
        "SynthDefs.scd should exist"
    );
    assert!(
        output_dir.path().join("config").join("buses.yaml").exists(),
        "buses.yaml should exist"
    );
    assert!(
        output_dir.path().join("README.md").exists(),
        "README.md should exist"
    );

    println!(
        "Successfully generated project with {} SynthDefs",
        project.synth_defs.len()
    );
}

#[test]
fn test_generate_spatial_project() {
    let spatial_patch = r#"{
        "patcher": {
            "rect": [0.0, 0.0, 640.0, 480.0],
            "boxes": [
                {
                    "box": {
                        "id": "obj-1",
                        "maxclass": "newobj",
                        "text": "spat5.panoramix~",
                        "numinlets": 1,
                        "numoutlets": 8
                    }
                },
                {
                    "box": {
                        "id": "obj-2",
                        "maxclass": "newobj", 
                        "text": "spat5.hoa.encoder~",
                        "numinlets": 1,
                        "numoutlets": 4
                    }
                }
            ],
            "lines": []
        }
    }"#;

    let patch = parse_patch_string(spatial_patch).expect("Failed to parse spatial patch");
    let output_dir = tempdir().expect("Failed to create temp dir");

    let result = generate_project(&patch, output_dir.path());
    assert!(
        result.is_ok(),
        "Failed to generate spatial project: {:?}",
        result.err()
    );

    let project = result.unwrap();

    // Should have generated spatial synth
    assert!(
        !project.synth_defs.is_empty(),
        "Should have generated spatial SynthDefs"
    );
    let has_spatial = project.synth_defs.iter().any(|s| s.name == "spatialSynth");
    assert!(has_spatial, "Should have generated spatialSynth");

    // Check main file content mentions spatial
    let main_content =
        fs::read_to_string(output_dir.path().join("main.scd")).expect("Failed to read main.scd");
    assert!(
        main_content.contains("2 objects"),
        "Should mention object count"
    );

    println!("Successfully generated spatial project");
}

#[test]
fn test_project_file_contents() {
    let simple_patch = r#"{
        "patcher": {
            "rect": [0.0, 0.0, 640.0, 480.0],
            "boxes": [
                {
                    "box": {
                        "id": "obj-1",
                        "maxclass": "newobj",
                        "text": "cycle~ 440"
                    }
                }
            ],
            "lines": []
        }
    }"#;

    let patch = parse_patch_string(simple_patch).expect("Failed to parse patch");
    let output_dir = tempdir().expect("Failed to create temp dir");

    generate_project(&patch, output_dir.path()).expect("Failed to generate project");

    // Check main.scd content
    let main_content =
        fs::read_to_string(output_dir.path().join("main.scd")).expect("Failed to read main.scd");
    assert!(main_content.contains("Auto-generated SuperCollider project"));
    assert!(main_content.contains("server.waitForBoot"));

    // Check SynthDefs.scd content
    let synthdefs_content = fs::read_to_string(output_dir.path().join("lib").join("SynthDefs.scd"))
        .expect("Failed to read SynthDefs.scd");
    assert!(synthdefs_content.contains("SynthDef"));
    assert!(synthdefs_content.contains("testSynth"));

    // Check buses.yaml content
    let buses_content = fs::read_to_string(output_dir.path().join("config").join("buses.yaml"))
        .expect("Failed to read buses.yaml");
    assert!(buses_content.contains("audio_buses:"));
    assert!(buses_content.contains("control_buses:"));
    assert!(buses_content.contains("Main Output"));

    // Check README.md content
    let readme_content =
        fs::read_to_string(output_dir.path().join("README.md")).expect("Failed to read README.md");
    assert!(readme_content.contains("Converted SuperCollider Project"));
    assert!(readme_content.contains("## Project Structure"));

    println!("All generated files have correct content");
}
