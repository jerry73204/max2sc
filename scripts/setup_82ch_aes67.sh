#!/bin/bash
# Setup script for 82-channel spatial audio with AES67 output
# Supports both JACK+PipeWire hybrid and JACK+external AES67 approaches

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_DIR="$HOME/.config/max2sc"
WINE_PREFIX="$HOME/.wine_max"

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration options
APPROACH="jack_external"  # Options: jack_pipewire, jack_external
AUDIO_INTERFACE="hw:0"
SAMPLE_RATE=48000
BUFFER_SIZE=512
AES67_MULTICAST="239.69.83.67:5004"

print_header() {
    echo -e "${BLUE}=== max2sc 82-Channel AES67 Setup ===${NC}"
    echo
}

print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_dependencies() {
    print_status "Checking dependencies..."
    
    local missing_deps=()
    
    # Check JACK
    if ! command -v jackd >/dev/null 2>&1; then
        missing_deps+=("jackd2")
    fi
    
    # Check WINE
    if ! command -v wine >/dev/null 2>&1; then
        missing_deps+=("wine")
    fi
    
    # Check SuperCollider
    if ! command -v sclang >/dev/null 2>&1; then
        missing_deps+=("supercollider")
    fi
    
    # Check PipeWire (if using hybrid approach)
    if [ "$APPROACH" = "jack_pipewire" ] && ! command -v pw-cli >/dev/null 2>&1; then
        missing_deps+=("pipewire")
    fi
    
    if [ ${#missing_deps[@]} -gt 0 ]; then
        print_error "Missing dependencies: ${missing_deps[*]}"
        echo "Install with: sudo apt install ${missing_deps[*]}"
        exit 1
    fi
    
    print_status "All dependencies found"
}

create_config_dir() {
    mkdir -p "$CONFIG_DIR"
    mkdir -p "$CONFIG_DIR/scripts"
    mkdir -p "$CONFIG_DIR/logs"
}

setup_wine_environment() {
    print_status "Setting up WINE environment..."
    
    export WINEPREFIX="$WINE_PREFIX"
    
    if [ ! -d "$WINE_PREFIX" ]; then
        print_status "Creating WINE prefix for Max MSP..."
        winecfg >/dev/null 2>&1 &
        WINECFG_PID=$!
        print_warning "Configure WINE to Windows XP mode, then close winecfg"
        wait $WINECFG_PID
    fi
    
    # Configure WINE registry for optimal audio
    wine regedit <<EOF >/dev/null 2>&1
[HKEY_CURRENT_USER\Software\Wine\DirectSound]
"HelBuflen"="$BUFFER_SIZE"
"SndQueueMax"="28"
"MaxShadowSize"="0"

[HKEY_CURRENT_USER\Software\Wine\ALSA Driver]
"AutoScanCards"="N"
"AutoScanDevices"="N"
"PeriodsPerBuffer"="2"

[HKEY_CURRENT_USER\Software\Wine\WineASIO]
"Number of inputs"="82"
"Number of outputs"="82"
"Preferred buffer size"="$BUFFER_SIZE"
"Jack client name"="max_msp"
EOF
    
    # Set environment variables for 82-channel support
    export WINEASIO_NUMBER_INPUTS=82
    export WINEASIO_NUMBER_OUTPUTS=82
    
    print_status "WINE environment configured for 82 channels"
}

setup_jack_external_aes67() {
    print_status "Setting up JACK + External AES67 approach..."
    
    # Stop PipeWire to avoid conflicts
    systemctl --user stop pipewire pipewire-pulse 2>/dev/null || true
    
    # Create JACK startup script
    cat > "$CONFIG_DIR/scripts/start_jack_82ch.sh" << EOF
#!/bin/bash
# Start JACK with 82-channel support

# Stop any existing JACK processes
killall jackd 2>/dev/null || true
sleep 1

# Start JACK with professional settings
jackd -R -P70 --port-max 1024 \\
    -d alsa -d $AUDIO_INTERFACE \\
    -r $SAMPLE_RATE -p $BUFFER_SIZE -n 2 \\
    -i 82 -o 82 &

JACK_PID=\$!
echo \$JACK_PID > "$CONFIG_DIR/jack.pid"

# Wait for JACK to start
sleep 3

# Verify JACK is running
if jack_lsp >/dev/null 2>&1; then
    echo "JACK started successfully with 82 channels"
    jack_lsp | grep -c "system:" || echo "Port count check failed"
else
    echo "JACK failed to start"
    exit 1
fi
EOF
    
    chmod +x "$CONFIG_DIR/scripts/start_jack_82ch.sh"
    
    # Create AES67 sender script (placeholder for external tool)
    cat > "$CONFIG_DIR/scripts/start_aes67_sender.sh" << EOF
#!/bin/bash
# Start AES67 sender for network output

# Example using aes67-linux-daemon (install separately)
# Replace with your preferred AES67 implementation

if command -v aes67-daemon >/dev/null 2>&1; then
    aes67-daemon \\
        --input-type jack \\
        --channels 82 \\
        --sample-rate $SAMPLE_RATE \\
        --multicast $AES67_MULTICAST \\
        --ptp-enable &
    
    echo \$! > "$CONFIG_DIR/aes67.pid"
    echo "AES67 sender started"
else
    echo "AES67 daemon not found. Install aes67-linux-daemon or similar."
    echo "Alternatives:"
    echo "  - RAVENNA ALSA driver"
    echo "  - Dante Audio Toolkit"
    echo "  - Custom network audio solution"
fi
EOF
    
    chmod +x "$CONFIG_DIR/scripts/start_aes67_sender.sh"
    
    print_status "JACK + External AES67 configuration created"
}

setup_jack_pipewire_hybrid() {
    print_status "Setting up JACK + PipeWire AES67 hybrid approach..."
    
    # Create PipeWire configuration for multichannel
    mkdir -p "$HOME/.config/pipewire/pipewire.conf.d"
    
    cat > "$HOME/.config/pipewire/pipewire.conf.d/99-max2sc.conf" << EOF
context.properties = {
    default.clock.rate = $SAMPLE_RATE
    default.clock.quantum = $BUFFER_SIZE
    default.clock.min-quantum = 32
    default.clock.max-quantum = 2048
    link.max-buffers = 64
    mem.warn-mlock = false
    mem.allow-mlock = true
}

context.modules = [
    { name = libpipewire-module-rt
        args = {
            nice.level = -11
            rt.prio = 88
            rt.time.soft = 200000
            rt.time.hard = 200000
        }
    }
]
EOF
    
    # Create hybrid startup script
    cat > "$CONFIG_DIR/scripts/start_jack_pipewire_hybrid.sh" << EOF
#!/bin/bash
# Start JACK + PipeWire AES67 hybrid setup

# Start PipeWire
systemctl --user start pipewire pipewire-pulse
sleep 2

# Start JACK with 82 channels using PipeWire bridge
pw-jack jackd -d alsa -d $AUDIO_INTERFACE \\
    -r $SAMPLE_RATE -p $BUFFER_SIZE -n 2 \\
    -i 82 -o 82 &

JACK_PID=\$!
echo \$JACK_PID > "$CONFIG_DIR/jack.pid"
sleep 3

# Create multiple AES67 streams for 82 channels
# Stream 1: Channels 1-64
pw-cli create-node adapter {
    factory.name = audioadapter
    node.name = "AES67_Stream1"
    media.class = "Audio/Sink"
    audio.channels = 64
    audio.rate = $SAMPLE_RATE
    api.aes67.multicast.group = "239.69.83.67"
    api.aes67.multicast.port = 5004
} >/dev/null 2>&1

# Stream 2: Channels 65-82 (18 channels)
pw-cli create-node adapter {
    factory.name = audioadapter
    node.name = "AES67_Stream2"
    media.class = "Audio/Sink"
    audio.channels = 18
    audio.rate = $SAMPLE_RATE
    api.aes67.multicast.group = "239.69.83.68"
    api.aes67.multicast.port = 5005
} >/dev/null 2>&1

sleep 2

# Route JACK outputs to AES67 streams
echo "Configuring audio routing..."
for i in {1..64}; do
    jack_connect "system:playback_\${i}" "AES67_Stream1:input_\${i}" 2>/dev/null || true
done

for i in {65..82}; do
    jack_connect "system:playback_\${i}" "AES67_Stream2:input_\$((i-64))" 2>/dev/null || true
done

echo "Hybrid JACK + PipeWire AES67 setup complete"
echo "Channels 1-64: AES67 stream 239.69.83.67:5004"
echo "Channels 65-82: AES67 stream 239.69.83.68:5005"
EOF
    
    chmod +x "$CONFIG_DIR/scripts/start_jack_pipewire_hybrid.sh"
    
    print_status "JACK + PipeWire hybrid configuration created"
}

create_supercollider_config() {
    print_status "Creating SuperCollider configuration..."
    
    SC_CONFIG_DIR="$HOME/.config/SuperCollider"
    mkdir -p "$SC_CONFIG_DIR"
    
    cat > "$SC_CONFIG_DIR/startup_max2sc.scd" << EOF
// SuperCollider startup configuration for max2sc 82-channel testing

// Configure server for 82 channels
Server.default.options.numOutputBusChannels = 82;
Server.default.options.numInputBusChannels = 82;
Server.default.options.blockSize = $BUFFER_SIZE;
Server.default.options.sampleRate = $SAMPLE_RATE;
Server.default.options.device = "JackRouter";

// Memory and performance settings
Server.default.options.memSize = 16384;  // 16MB memory
Server.default.options.numBuffers = 2048;
Server.default.options.maxNodes = 2048;
Server.default.options.maxSynthDefs = 2048;

// Boot message
Server.default.waitForBoot {
    "max2sc SuperCollider server booted with 82 channels".postln;
    "Output channels: %".format(Server.default.options.numOutputBusChannels).postln;
    "Input channels: %".format(Server.default.options.numInputBusChannels).postln;
    "Sample rate: %".format(Server.default.sampleRate).postln;
    "Block size: %".format(Server.default.options.blockSize).postln;
};
EOF
    
    print_status "SuperCollider configured for 82-channel operation"
}

create_test_scripts() {
    print_status "Creating test scripts..."
    
    # Max MSP launcher script
    cat > "$CONFIG_DIR/scripts/launch_max_msp.sh" << EOF
#!/bin/bash
# Launch Max MSP with 82-channel configuration

export WINEPREFIX="$WINE_PREFIX"
export WINEASIO_NUMBER_INPUTS=82
export WINEASIO_NUMBER_OUTPUTS=82

# Check if JACK is running
if ! jack_lsp >/dev/null 2>&1; then
    echo "JACK is not running. Start with:"
    echo "  $CONFIG_DIR/scripts/start_jack_82ch.sh"
    exit 1
fi

# Launch Max MSP
echo "Launching Max MSP with 82-channel support..."
wine "C:/Program Files/Cycling '74/Max 8/Max.exe"
EOF
    
    chmod +x "$CONFIG_DIR/scripts/launch_max_msp.sh"
    
    # SuperCollider test script
    cat > "$CONFIG_DIR/scripts/test_supercollider.sh" << EOF
#!/bin/bash
# Test SuperCollider 82-channel functionality

echo "Testing SuperCollider 82-channel setup..."

sclang -l "$SC_CONFIG_DIR/startup_max2sc.scd" << 'SCEOF'
// Test 82-channel output
(
    // Simple test: sine waves on all 82 channels
    SynthDef(\test82ch, {
        var freqs = (100..181);  // 82 frequencies
        var sigs = SinOsc.ar(freqs) * 0.01;  // Low volume
        Out.ar((0..81), sigs);
    }).add;
    
    s.sync;
    
    "Playing test tones on all 82 channels...".postln;
    x = Synth(\test82ch);
    
    // Stop after 2 seconds
    SystemClock.sched(2, {
        x.free;
        "82-channel test complete".postln;
        0.exit;
    });
)
SCEOF
EOF
    
    chmod +x "$CONFIG_DIR/scripts/test_supercollider.sh"
    
    # Channel validation script
    cat > "$CONFIG_DIR/scripts/validate_channels.sh" << EOF
#!/bin/bash
# Validate 82-channel setup

echo "=== Channel Validation ==="

# Check JACK ports
if jack_lsp >/dev/null 2>&1; then
    JACK_INPUTS=\$(jack_lsp | grep "system:capture" | wc -l)
    JACK_OUTPUTS=\$(jack_lsp | grep "system:playback" | wc -l)
    
    echo "JACK Input channels: \$JACK_INPUTS"
    echo "JACK Output channels: \$JACK_OUTPUTS"
    
    if [ "\$JACK_OUTPUTS" -eq 82 ]; then
        echo "✅ 82-channel output configuration verified"
    else
        echo "❌ Expected 82 output channels, found \$JACK_OUTPUTS"
    fi
else
    echo "❌ JACK is not running"
fi

# Check WINE configuration
export WINEPREFIX="$WINE_PREFIX"
if [ -d "\$WINEPREFIX" ]; then
    echo "✅ WINE prefix configured: \$WINEPREFIX"
else
    echo "❌ WINE prefix not found: \$WINEPREFIX"
fi

# Check WineASIO
if [ -f "\$WINEPREFIX/drive_c/windows/system32/wineasio.dll" ]; then
    echo "✅ WineASIO installed"
else
    echo "⚠️  WineASIO not found - install with: wine regsvr32 wineasio.dll"
fi

echo "=== Validation Complete ==="
EOF
    
    chmod +x "$CONFIG_DIR/scripts/validate_channels.sh"
}

create_stop_scripts() {
    print_status "Creating stop scripts..."
    
    cat > "$CONFIG_DIR/scripts/stop_all.sh" << EOF
#!/bin/bash
# Stop all audio processes

echo "Stopping max2sc audio services..."

# Stop JACK
if [ -f "$CONFIG_DIR/jack.pid" ]; then
    JACK_PID=\$(cat "$CONFIG_DIR/jack.pid")
    kill \$JACK_PID 2>/dev/null || true
    rm -f "$CONFIG_DIR/jack.pid"
fi

# Stop AES67 sender
if [ -f "$CONFIG_DIR/aes67.pid" ]; then
    AES67_PID=\$(cat "$CONFIG_DIR/aes67.pid")
    kill \$AES67_PID 2>/dev/null || true
    rm -f "$CONFIG_DIR/aes67.pid"
fi

# Kill any remaining processes
killall jackd 2>/dev/null || true
killall aes67-daemon 2>/dev/null || true

# Restart PipeWire if it was stopped
systemctl --user start pipewire pipewire-pulse 2>/dev/null || true

echo "All processes stopped"
EOF
    
    chmod +x "$CONFIG_DIR/scripts/stop_all.sh"
}

show_usage() {
    echo "Usage: $0 [approach] [options]"
    echo
    echo "Approaches:"
    echo "  jack_external   - JACK + External AES67 tool (recommended)"
    echo "  jack_pipewire   - JACK + PipeWire AES67 hybrid"
    echo
    echo "Options:"
    echo "  --interface DEVICE   Audio interface (default: hw:0)"
    echo "  --sample-rate RATE   Sample rate (default: 48000)"
    echo "  --buffer-size SIZE   Buffer size (default: 512)"
    echo "  --multicast ADDR     AES67 multicast address (default: 239.69.83.67:5004)"
    echo
    echo "Examples:"
    echo "  $0 jack_external --interface hw:RME"
    echo "  $0 jack_pipewire --buffer-size 1024"
}

main() {
    print_header
    
    # Parse command line arguments
    if [ $# -gt 0 ]; then
        APPROACH="$1"
        shift
    fi
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --interface)
                AUDIO_INTERFACE="$2"
                shift 2
                ;;
            --sample-rate)
                SAMPLE_RATE="$2"
                shift 2
                ;;
            --buffer-size)
                BUFFER_SIZE="$2"
                shift 2
                ;;
            --multicast)
                AES67_MULTICAST="$2"
                shift 2
                ;;
            --help|-h)
                show_usage
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done
    
    print_status "Configuration:"
    echo "  Approach: $APPROACH"
    echo "  Audio interface: $AUDIO_INTERFACE"
    echo "  Sample rate: $SAMPLE_RATE Hz"
    echo "  Buffer size: $BUFFER_SIZE samples"
    echo "  AES67 multicast: $AES67_MULTICAST"
    echo
    
    check_dependencies
    create_config_dir
    setup_wine_environment
    
    case "$APPROACH" in
        jack_external)
            setup_jack_external_aes67
            ;;
        jack_pipewire)
            setup_jack_pipewire_hybrid
            ;;
        *)
            print_error "Unknown approach: $APPROACH"
            show_usage
            exit 1
            ;;
    esac
    
    create_supercollider_config
    create_test_scripts
    create_stop_scripts
    
    print_status "Setup complete!"
    echo
    echo -e "${GREEN}Next steps:${NC}"
    echo "1. Start audio system:"
    if [ "$APPROACH" = "jack_external" ]; then
        echo "   $CONFIG_DIR/scripts/start_jack_82ch.sh"
        echo "   $CONFIG_DIR/scripts/start_aes67_sender.sh"
    else
        echo "   $CONFIG_DIR/scripts/start_jack_pipewire_hybrid.sh"
    fi
    echo
    echo "2. Validate configuration:"
    echo "   $CONFIG_DIR/scripts/validate_channels.sh"
    echo
    echo "3. Test SuperCollider:"
    echo "   $CONFIG_DIR/scripts/test_supercollider.sh"
    echo
    echo "4. Launch Max MSP:"
    echo "   $CONFIG_DIR/scripts/launch_max_msp.sh"
    echo
    echo "5. Stop all services:"
    echo "   $CONFIG_DIR/scripts/stop_all.sh"
    echo
    echo -e "${BLUE}Configuration files created in: $CONFIG_DIR${NC}"
}

# Run main function
main "$@"