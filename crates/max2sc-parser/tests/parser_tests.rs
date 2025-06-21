//! Parser tests

use max2sc_parser::{parse_patch_file, parse_patch_string};
use std::path::Path;

#[test]
fn test_parse_simple_patch() {
    let simple_patch = r#"{
        "patcher": {
            "fileversion": 1,
            "appversion": {
                "major": 8,
                "minor": 6,
                "revision": 5,
                "architecture": "x64",
                "modernui": 1
            },
            "rect": [0.0, 0.0, 640.0, 480.0],
            "boxes": [
                {
                    "box": {
                        "id": "obj-1",
                        "maxclass": "newobj",
                        "text": "spat5.panoramix~",
                        "numinlets": 1,
                        "numoutlets": 8,
                        "patching_rect": [100.0, 100.0, 120.0, 22.0]
                    }
                }
            ],
            "lines": []
        }
    }"#;

    let result = parse_patch_string(simple_patch);
    assert!(
        result.is_ok(),
        "Failed to parse simple patch: {:?}",
        result.err()
    );

    let patch = result.unwrap();
    assert_eq!(patch.patcher.boxes.len(), 1);
    assert_eq!(patch.patcher.boxes[0].content.maxclass, "newobj");
    assert_eq!(
        patch.patcher.boxes[0].content.text,
        Some("spat5.panoramix~".to_string())
    );
}

#[test]
fn test_parse_actual_max_patch() {
    let patch_path = Path::new("../../max8_source/AUO_2024_Max8_copy.maxpat");

    if patch_path.exists() {
        let result = parse_patch_file(patch_path);
        assert!(
            result.is_ok(),
            "Failed to parse actual Max patch: {:?}",
            result.err()
        );

        let patch = result.unwrap();
        println!(
            "Successfully parsed patch with {} boxes and {} lines",
            patch.patcher.boxes.len(),
            patch.patcher.lines.len()
        );

        // Find SPAT5 objects
        let spat5_objects: Vec<_> = patch
            .patcher
            .boxes
            .iter()
            .filter(|box_container| {
                box_container
                    .content
                    .text
                    .as_ref()
                    .map(|text| text.contains("spat5"))
                    .unwrap_or(false)
            })
            .collect();

        println!("Found {} SPAT5 objects", spat5_objects.len());

        for spat5_obj in spat5_objects.iter().take(3) {
            println!(
                "SPAT5 object: {} - {}",
                spat5_obj.content.id,
                spat5_obj
                    .content
                    .text
                    .as_ref()
                    .unwrap_or(&"no text".to_string())
            );
        }
    } else {
        println!("Skipping actual Max patch test - file not found");
    }
}

#[test]
fn test_parse_patch_with_connections() {
    let patch_with_lines = r#"{
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

    let result = parse_patch_string(patch_with_lines);
    assert!(
        result.is_ok(),
        "Failed to parse patch with connections: {:?}",
        result.err()
    );

    let patch = result.unwrap();
    assert_eq!(patch.patcher.boxes.len(), 2);
    assert_eq!(patch.patcher.lines.len(), 1);
}
