#!/bin/bash

# Quick context recovery for Campfire-on-Rust sessions

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SESSION_CONTEXT="$PROJECT_ROOT/SESSION_CONTEXT.md"

echo "🚀 Campfire-on-Rust Context Recovery"
echo "====================================="

if [ ! -f "$SESSION_CONTEXT" ]; then
    echo "❌ SESSION_CONTEXT.md not found!"
    exit 1
fi

echo ""
echo "📋 Live Session Status:"
echo "----------------------"
grep -A 10 "Live Session Status" "$SESSION_CONTEXT"

echo ""
echo "✅ Active Todo List:"
echo "-------------------"
grep -A 15 "Active Todo List" "$SESSION_CONTEXT" | head -20

echo ""
echo "🎯 Priority Tasks:"
echo "-----------------"
grep -A 10 "Current Session Tasks" "$SESSION_CONTEXT"

echo ""
echo "🏗️  5 Critical Gaps Status:"
echo "------------------------"
grep -A 10 "5 Critical Gaps Implementation Status" "$SESSION_CONTEXT" | head -15

echo ""
echo "📊 Architecture Compliance:"
echo "-------------------------"
grep -A 10 "Architecture Compliance Checklist" "$SESSION_CONTEXT" | head -15

echo ""
echo "🔄 Recent Progress:"
echo "-----------------"
grep -A 10 "Recent Progress Log" "$SESSION_CONTEXT" | head -10

echo ""
echo "⚡ Quick Commands:"
echo "----------------"
echo "  cat $SESSION_CONTEXT.md | grep -A 20 'Live Session Status'"
echo "  git status"
echo "  git log --oneline -5"
echo "  ./.scripts/update-context.sh"

echo ""
echo "✅ Context recovery complete!"
echo "Next steps: Check priority tasks above and continue implementation."