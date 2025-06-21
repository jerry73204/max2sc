//! OSC router generation

use crate::CodegenError;
use max2sc_max_types::{MaxPatch, OSCConfig};
use max2sc_sc_types::OSCResponder;
use std::collections::HashMap;

/// Generate OSC routers from Max patch
pub fn generate_osc_routers(
    patch: &MaxPatch,
    osc_config: Option<&OSCConfig>,
) -> Result<Vec<OSCResponder>, CodegenError> {
    let mut responders = Vec::new();
    let mut osc_routes = HashMap::new();

    // Find all spat5.osc.route objects in the patch
    for box_container in &patch.patcher.boxes {
        let text = box_container.content.text.as_deref().unwrap_or("");

        if text.starts_with("spat5.osc.route") {
            // Extract OSC addresses from the object
            let addresses = extract_osc_addresses(text);
            for addr in addresses {
                osc_routes.insert(addr.clone(), box_container.content.id.clone());
            }
        }
    }

    // Generate responders for common SPAT5 OSC addresses
    responders.push(generate_source_responder());
    responders.push(generate_speaker_responder());
    responders.push(generate_reverb_responder());
    responders.push(generate_master_responder());

    // Add custom responders from patch analysis
    for (address, obj_id) in osc_routes {
        responders.push(OSCResponder {
            address: address.clone(),
            action: format!("{{ |msg| ~handle_{}.value(msg) }}", sanitize_id(&obj_id)),
            params: vec![],
        });
    }

    Ok(responders)
}

/// Extract OSC addresses from spat5.osc.route text
fn extract_osc_addresses(text: &str) -> Vec<String> {
    let mut addresses = Vec::new();
    let parts: Vec<&str> = text.split_whitespace().collect();

    // Skip "spat5.osc.route" and collect addresses
    for part in parts.iter().skip(1) {
        if part.starts_with('/') {
            addresses.push(part.to_string());
        }
    }

    addresses
}

/// Generate source position responder
fn generate_source_responder() -> OSCResponder {
    OSCResponder {
        address: "/source/*/xyz".to_string(),
        action: r#"{ |msg|
    var sourceID = msg[0].asString.split($/).at(2).asInteger;
    var x = msg[1];
    var y = msg[2];
    var z = msg[3];
    
    if(~sources.notNil and: { ~sources[sourceID].notNil }, {
        ~sources[sourceID].set(\x, x, \y, y, \z, z);
    });
}"#
        .to_string(),
        params: vec![],
    }
}

/// Generate speaker configuration responder
fn generate_speaker_responder() -> OSCResponder {
    OSCResponder {
        address: "/speaker/*/gain".to_string(),
        action: r#"{ |msg|
    var speakerID = msg[0].asString.split($/).at(2).asInteger;
    var gain = msg[1];
    
    if(~speakers.notNil and: { ~speakers[speakerID].notNil }, {
        ~speakers[speakerID].set(\gain, gain.dbamp);
    });
}"#
        .to_string(),
        params: vec![],
    }
}

/// Generate reverb control responder
fn generate_reverb_responder() -> OSCResponder {
    OSCResponder {
        address: "/reverb/*".to_string(),
        action: r#"{ |msg|
    var param = msg[0].asString.split($/).last;
    var value = msg[1];
    
    switch(param,
        "roomsize", { ~reverb.set(\roomsize, value) },
        "rt60", { ~reverb.set(\rt60, value) },
        "damp", { ~reverb.set(\damp, value) },
        "dry", { ~reverb.set(\dry, value) },
        "early", { ~reverb.set(\early, value) },
        "tail", { ~reverb.set(\tail, value) }
    );
}"#
        .to_string(),
        params: vec![],
    }
}

/// Generate master control responder
fn generate_master_responder() -> OSCResponder {
    OSCResponder {
        address: "/master/*".to_string(),
        action: r#"{ |msg|
    var param = msg[0].asString.split($/).last;
    var value = msg[1];
    
    switch(param,
        "gain", { ~masterBus.set(\gain, value.dbamp) },
        "mute", { ~masterBus.set(\mute, value) },
        "bypass", { ~masterBus.set(\bypass, value) }
    );
}"#
        .to_string(),
        params: vec![],
    }
}

/// Sanitize object ID for use in variable names
fn sanitize_id(id: &str) -> String {
    id.replace('-', "_").replace(' ', "_")
}

/// Generate OSC router setup code
pub fn generate_osc_router_code(responders: &[OSCResponder]) -> String {
    let mut code = String::from(
        r#"// OSC Router Setup
// Auto-generated from Max patch analysis

(
    var setupOSC;
    
    setupOSC = {
        "Setting up OSC routers...".postln;
        
        // Clear existing OSC responders
        if(~oscResponders.notNil, {
            ~oscResponders.do(_.free);
        });
        
        ~oscResponders = List.new;
        
"#,
    );

    // Add each responder
    for responder in responders {
        code.push_str(&format!(
            r#"        // {}
        ~oscResponders.add(
            OSCFunc({}, '{}', recvPort: 57120)
        );
        
"#,
            "OSC Route", responder.action, responder.address
        ));
    }

    code.push_str(
        r#"        "OSC routers initialized:".postln;
        ~oscResponders.do { |resp, i|
            "  [%] %".format(i, resp.path).postln;
        };
    };
    
    // Initialize on load
    setupOSC.value;
    
    // Export setup function
    ~setupOSC = setupOSC;
)
"#,
    );

    code
}
