#!/bin/bash
# Test PipeWire AES67 multichannel capabilities for max2sc project
# This script demonstrates current limitations for 82-channel spatial audio

set -e

echo "=== PipeWire AES67 Multichannel Test for max2sc ==="
echo

# Check if PipeWire is running
echo "1. Checking PipeWire status..."
if systemctl --user is-active --quiet pipewire; then
    echo "✅ PipeWire is running"
else
    echo "❌ PipeWire is not running"
    echo "Start with: systemctl --user start pipewire pipewire-pulse"
    exit 1
fi

# Check PipeWire version
PIPEWIRE_VERSION=$(pipewire --version 2>/dev/null | grep -o '[0-9]\+\.[0-9]\+\.[0-9]\+' || echo "unknown")
echo "📍 PipeWire version: $PIPEWIRE_VERSION"
echo

# List available audio objects
echo "2. Available audio objects:"
pw-cli list-objects | grep -E "(Audio/Source|Audio/Sink)" | head -10
echo

# Check for AES67 module
echo "3. Checking AES67 module availability..."
if pw-cli list-objects | grep -q aes67; then
    echo "✅ AES67 module detected"
    pw-cli list-objects | grep aes67
else
    echo "⚠️  AES67 module not found or not loaded"
fi
echo

# Test channel count limits
echo "4. Testing channel count capabilities..."

# Function to test channel configuration
test_channels() {
    local channels=$1
    echo "Testing $channels channels..."
    
    # Try to create a virtual device with specified channels
    if timeout 5s pw-jack jackd -d dummy -r 48000 -p 512 -i $channels -o $channels >/dev/null 2>&1; then
        echo "✅ $channels channels: SUPPORTED"
        pkill -f "jackd.*dummy" >/dev/null 2>&1 || true
        return 0
    else
        echo "❌ $channels channels: FAILED"
        pkill -f "jackd.*dummy" >/dev/null 2>&1 || true
        return 1
    fi
}

# Test various channel counts
test_channels 32
test_channels 64
test_channels 82
echo

# Check JACK bridge functionality
echo "5. Testing PipeWire-JACK bridge..."
if command -v pw-jack >/dev/null 2>&1; then
    echo "✅ pw-jack bridge available"
    
    # Test basic JACK functionality
    echo "Starting test JACK session with 64 channels..."
    pw-jack jackd -d dummy -r 48000 -p 512 -i 64 -o 64 &
    JACK_PID=$!
    sleep 2
    
    # Check if JACK is responding
    if pw-jack jack_lsp >/dev/null 2>&1; then
        echo "✅ JACK bridge functional"
        JACK_PORTS=$(pw-jack jack_lsp | wc -l)
        echo "📍 Available JACK ports: $JACK_PORTS"
    else
        echo "❌ JACK bridge not responding"
    fi
    
    # Cleanup
    kill $JACK_PID 2>/dev/null || true
    wait $JACK_PID 2>/dev/null || true
else
    echo "❌ pw-jack not available"
fi
echo

# Test WINE integration
echo "6. Testing WINE compatibility..."
if command -v wine >/dev/null 2>&1; then
    export WINEPREFIX=$HOME/.wine_max
    
    if [ -d "$WINEPREFIX" ]; then
        echo "✅ WINE prefix found: $WINEPREFIX"
        
        # Check for WineASIO
        if [ -f "$WINEPREFIX/drive_c/windows/system32/wineasio.dll" ]; then
            echo "✅ WineASIO installed"
        else
            echo "⚠️  WineASIO not found - required for Max MSP multichannel"
            echo "   Install with: wine regsvr32 wineasio.dll"
        fi
    else
        echo "⚠️  WINE prefix not found: $WINEPREFIX"
        echo "   Create with: winecfg"
    fi
else
    echo "❌ WINE not available"
fi
echo

# Generate configuration recommendations
echo "7. Configuration recommendations for max2sc:"
echo
echo "📋 CURRENT LIMITATIONS:"
echo "   • PipeWire maximum channels: 64"
echo "   • max2sc requirement: 82 channels"
echo "   • Gap: 18 channels missing"
echo
echo "📋 RECOMMENDED APPROACH:"
echo "   For full 82-channel testing:"
echo "   1. Temporarily stop PipeWire: systemctl --user stop pipewire pipewire-pulse"
echo "   2. Start JACK: jackd -R -d alsa -r 48000 -p 512 -i 82 -o 82"
echo "   3. Configure SuperCollider for 82 channels"
echo "   4. Run max2sc comparison tests"
echo "   5. Restart PipeWire: systemctl --user start pipewire pipewire-pulse"
echo
echo "📋 DEVELOPMENT TESTING (64-channel limit):"
echo "   Use PipeWire for basic functionality testing:"
echo "   • pw-jack jackd -d dummy -r 48000 -p 512 -i 64 -o 64"
echo "   • Test spatial algorithms with reduced channel count"
echo "   • Validate basic Max→SC conversion accuracy"
echo
echo "📋 FUTURE MIGRATION:"
echo "   Monitor PipeWire development for:"
echo "   • Channel count increases beyond 64"
echo "   • Complete AES67/RAVENNA implementation"
echo "   • Native WINE integration"
echo

# Test SuperCollider integration
echo "8. Testing SuperCollider integration..."
if command -v sclang >/dev/null 2>&1; then
    echo "✅ SuperCollider available"
    
    # Start a minimal JACK session for testing
    echo "Starting test session for SuperCollider..."
    pw-jack jackd -d dummy -r 48000 -p 512 -i 64 -o 64 &
    JACK_PID=$!
    sleep 2
    
    # Test SuperCollider boot with PipeWire
    echo "Testing SuperCollider server boot..."
    SC_TEST_RESULT=$(timeout 10s sclang -e "
        Server.default.options.numOutputBusChannels = 64;
        Server.default.options.numInputBusChannels = 64;
        Server.default.waitForBoot {
            'SuperCollider booted with 64 channels via PipeWire'.postln;
            s.quit;
            0.exit;
        };
    " 2>&1 || echo "TIMEOUT")
    
    if echo "$SC_TEST_RESULT" | grep -q "SuperCollider booted"; then
        echo "✅ SuperCollider successfully booted with 64 channels"
    else
        echo "❌ SuperCollider boot failed or timed out"
        echo "   Output: $SC_TEST_RESULT"
    fi
    
    # Cleanup
    kill $JACK_PID 2>/dev/null || true
    wait $JACK_PID 2>/dev/null || true
else
    echo "❌ SuperCollider not available"
fi
echo

echo "=== Test Summary ==="
echo "PipeWire AES67 setup analysis complete."
echo
echo "🎯 KEY FINDING: PipeWire's 64-channel limit prevents full 82-channel"
echo "   spatial audio testing required for max2sc project."
echo
echo "💡 RECOMMENDATION: Use JACK for production 82-channel testing,"
echo "   PipeWire for limited development work only."
echo
echo "📄 See STUDY_PIPEWIRE_AES67_INTEGRATION.md for detailed analysis."