#!/bin/bash
set -euo pipefail

# Mock JSON payloads
MOCK_LOW='{"agent_state": "thinking", "context_window": {"used_percentage": 20.0}, "vcs": {"branch": "main", "dirty": false}, "sandbox": {"enabled": true}, "artifact_count": 3, "subagents": [], "task_count": 1, "model": {"display_name": "gemini-3.5-flash"}, "terminal_width": 120, "rate_limits": {"five_hour": {"used_percentage": 10.0}, "seven_day": {"used_percentage": 15.0}}}'
MOCK_WARN='{"agent_state": "working", "context_window": {"used_percentage": 65.0}, "vcs": {"branch": "feature/ui", "dirty": true}, "sandbox": {"enabled": true}, "artifact_count": 0, "subagents": [], "task_count": 0, "model": {"display_name": "gemini-3.5-flash"}, "terminal_width": 120, "rate_limits": {"five_hour": {"used_percentage": 60.0}, "seven_day": {"used_percentage": 55.0}}}'
MOCK_CRIT='{"agent_state": "tool_use", "context_window": {"used_percentage": 92.0}, "vcs": {"branch": "main", "dirty": false}, "sandbox": {"enabled": false}, "artifact_count": 2, "subagents": [], "task_count": 2, "model": {"display_name": "gemini-3.5-flash"}, "terminal_width": 120, "rate_limits": {"five_hour": {"used_percentage": 85.0}, "seven_day": {"used_percentage": 90.0}}}'

echo "=== Running statusline tests ==="

# Test 1: Low usage states and emojis
out_low=$(echo "$MOCK_LOW" | bash statusline.sh)
echo "$out_low"
if [[ ! "$out_low" =~ "🧠 THINKING" ]]; then
  echo "FAIL: Missing thinking state emoji"
  exit 1
fi
if [[ ! "$out_low" =~ "🤖 gemini-3.5-flash" ]]; then
  echo "FAIL: Missing model emoji"
  exit 1
fi
if [[ ! "$out_low" =~ "🎋 main" ]]; then
  echo "FAIL: Missing branch emoji"
  exit 1
fi
if [[ ! "$out_low" =~ "⏱️ 5h 10%" ]]; then
  echo "FAIL: Missing 5-hour limit 10%"
  exit 1
fi
if [[ ! "$out_low" =~ "📅 wk 15%" ]]; then
  echo "FAIL: Missing weekly limit 15%"
  exit 1
fi

# Test 2: Warning state and dirty branch
out_warn=$(echo "$MOCK_WARN" | bash statusline.sh)
echo "$out_warn"
if [[ ! "$out_warn" =~ "⚡ WORKING" ]]; then
  echo "FAIL: Missing working state emoji"
  exit 1
fi
if [[ ! "$out_warn" =~ "🎋 feature/ui" || ! "$out_warn" =~ "⚠️" ]]; then
  echo "FAIL: Missing dirty branch indicator"
  exit 1
fi

# Test 3: Critical state and alert formatting
out_crit=$(echo "$MOCK_CRIT" | bash statusline.sh)
echo "$out_crit"
if [[ ! "$out_crit" =~ "🛠️ TOOL" ]]; then
  echo "FAIL: Missing tool state emoji"
  exit 1
fi
if [[ ! "$out_crit" =~ "⚠️ ⚡ 5h 85%" ]]; then
  echo "FAIL: Missing critical 5h alert"
  exit 1
fi
if [[ ! "$out_crit" =~ "⚠️ 📅 wk 90%" ]]; then
  echo "FAIL: Missing critical wk alert"
  exit 1
fi

echo "ALL TESTS PASSED!"
