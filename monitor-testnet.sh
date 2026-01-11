#!/bin/bash

# Casper Testnet RPC Monitor
# Checks testnet RPC every 5 minutes and alerts when it's back online

echo "======================================"
echo "Casper Testnet RPC Monitor"
echo "======================================"
echo "Starting at: $(date)"
echo "Checking every 5 minutes..."
echo ""

# RPC endpoints to test
ENDPOINTS=(
    "https://rpc.testnet.casperlabs.io"
    "http://65.109.101.173:7777"
    "http://195.201.174.222:7777"
)

check_count=0

while true; do
    check_count=$((check_count + 1))
    echo "Check #$check_count at $(date)"
    echo "---"

    found_working=false

    for endpoint in "${ENDPOINTS[@]}"; do
        result=$(curl -s -X POST "$endpoint/rpc" \
            -H "Content-Type: application/json" \
            -d '{"jsonrpc":"2.0","method":"chain_get_state_root_hash","params":[],"id":1}' \
            --connect-timeout 5 --max-time 10 2>&1)

        if echo "$result" | grep -q "state_root_hash"; then
            echo "✅✅✅ TESTNET IS BACK UP! ✅✅✅"
            echo "Working endpoint: $endpoint"
            state_root=$(echo "$result" | jq -r '.result.state_root_hash' 2>/dev/null)
            echo "State root hash: $state_root"
            echo ""
            echo "🚀 READY TO DEPLOY!"
            echo ""
            echo "Run deployment script:"
            echo "  cd /Users/janhv/Desktop/StakeFlow/stakeflow"
            echo "  ./deploy-v3.sh"
            echo ""

            # Play alert sound (macOS)
            afplay /System/Library/Sounds/Glass.aiff 2>/dev/null || true

            found_working=true
            break
        fi
    done

    if [ "$found_working" = true ]; then
        break
    fi

    echo "❌ All endpoints still down"
    echo ""
    echo "Next check in 5 minutes..."
    echo "Press Ctrl+C to stop monitoring"
    echo "======================================"
    echo ""

    sleep 300  # Wait 5 minutes
done

echo ""
echo "Monitoring stopped."
