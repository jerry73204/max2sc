//! OSC configuration parser tests

use max2sc_max_types::parse_osc_text;
use max2sc_parser::parse_osc_config;
use std::path::Path;

#[test]
fn test_parse_simple_osc_config() {
    let simple_osc = r#"# Test OSC configuration
/master/name "Master"
/master/numio 32 32
/bus/1/format "WFS"
/bus/1/name "WFS Bus 1"
/bus/1/speakers/aed -39.3518 0.0 1.29321 -35.3748 0.0 1.22642 -30.9638 0.0 1.16619
/bus/1/speaker/1/delay 0.0186
/bus/1/speaker/2/delay 0.2132
/bus/1/speaker/1/gain -3.5
/bus/1/speaker/2/gain -2.1
"#;

    let result = parse_osc_text(simple_osc);
    assert!(
        result.is_ok(),
        "Failed to parse simple OSC config: {:?}",
        result.err()
    );

    let config = result.unwrap();

    // Check that we have some commands
    assert!(!config.commands.is_empty(), "No OSC commands parsed");

    // Check speaker arrays
    assert_eq!(config.speaker_arrays.len(), 1, "Expected 1 speaker array");

    let array = &config.speaker_arrays[0];
    assert_eq!(array.bus_id, 1);
    assert_eq!(array.format, "WFS");
    assert_eq!(array.name, "WFS Bus 1");
    assert_eq!(array.speakers.len(), 3, "Expected 3 speakers from AED data");

    // Check first speaker
    let speaker1 = &array.speakers[0];
    assert_eq!(speaker1.id, 1);
    assert_eq!(speaker1.position.azimuth, -39.3518);
    assert_eq!(speaker1.position.elevation, 0.0);
    assert_eq!(speaker1.position.distance, 1.29321);
    assert_eq!(speaker1.delay, 0.0186);
    assert_eq!(speaker1.gain, -3.5);

    println!(
        "Successfully parsed OSC config with {} commands and {} speaker arrays",
        config.commands.len(),
        config.speaker_arrays.len()
    );
}

#[test]
fn test_parse_actual_osc_file() {
    let osc_path = Path::new("../../max8_source/AUO_32speakers_45mm.txt");

    if osc_path.exists() {
        let result = parse_osc_config(osc_path);
        assert!(
            result.is_ok(),
            "Failed to parse actual OSC file: {:?}",
            result.err()
        );

        let config = result.unwrap();
        println!(
            "Successfully parsed OSC file with {} commands and {} speaker arrays",
            config.commands.len(),
            config.speaker_arrays.len()
        );

        // Print speaker array info
        for array in &config.speaker_arrays {
            println!(
                "Bus {}: {} ({}) with {} speakers",
                array.bus_id,
                array.name,
                array.format,
                array.speakers.len()
            );
        }

        // Should have found speaker arrays
        assert!(!config.speaker_arrays.is_empty(), "No speaker arrays found");
    } else {
        println!("Skipping actual OSC file test - file not found");
    }
}

#[test]
fn test_parse_osc_value_types() {
    let osc_with_types = r#"/test/string "hello world"
/test/float 3.14159
/test/int 42
/test/mixed 1.0 "test" 2
"#;

    let result = parse_osc_text(osc_with_types);
    assert!(
        result.is_ok(),
        "Failed to parse OSC with different value types"
    );

    let config = result.unwrap();
    assert_eq!(config.commands.len(), 4);

    // Check string value
    let string_cmd = &config.commands[0];
    assert_eq!(string_cmd.address, "/test/string");
    if let max2sc_max_types::OSCValue::String(s) = &string_cmd.args[0] {
        assert_eq!(s, "hello world");
    } else {
        panic!("Expected string value");
    }

    // Check float value
    let float_cmd = &config.commands[1];
    assert_eq!(float_cmd.address, "/test/float");
    if let max2sc_max_types::OSCValue::Float(f) = &float_cmd.args[0] {
        assert!((f - std::f32::consts::PI).abs() < 0.0001);
    } else {
        panic!("Expected float value");
    }

    // Check int value
    let int_cmd = &config.commands[2];
    assert_eq!(int_cmd.address, "/test/int");
    if let max2sc_max_types::OSCValue::Int(i) = &int_cmd.args[0] {
        assert_eq!(*i, 42);
    } else {
        panic!("Expected int value");
    }
}
