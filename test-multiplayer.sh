#!/usr/bin/env bash
# 一键启动 3 人联机测试
# Usage: ./test-multiplayer.sh [--release]
#
# 分别打开 4 个终端窗口：relay + Player 0/1/2
# macOS 专用 (使用 osascript 控制终端)

set -e

cd "$(dirname "$0")"

BUILD_MODE=""
PROFILE="dev"
PROFILE_DIR="debug"
if [ "$1" = "--release" ]; then
    BUILD_MODE="--release"
    PROFILE="release"
    PROFILE_DIR="release"
fi

echo "=== 编译中 (${PROFILE})... ==="
cargo build $BUILD_MODE --workspace

BINARY="./target/$PROFILE_DIR/city-conquest"
RELAY_BINARY="./target/$PROFILE_DIR/relay"

echo "=== 启动 relay (端口 9876, 3 人) ==="
osascript -e "
tell application \"Terminal\"
    activate
    set newTab to do script \"cd '$(pwd)' && $RELAY_BINARY --port 9876 --seed 42 --players 3
\"
    set custom title of front window to \"[Relay] :9876\"
end tell
"

sleep 1

echo "=== 启动 Player 0 ==="
osascript -e "
tell application \"Terminal\"
    activate
    set newTab to do script \"cd '$(pwd)' && $BINARY --relay 127.0.0.1:9876 --player-id 0 --players 3 --windowed
\"
    set custom title of front window to \"[Player 0]\"
end tell
"

sleep 1

echo "=== 启动 Player 1 ==="
osascript -e "
tell application \"Terminal\"
    activate
    set newTab to do script \"cd '$(pwd)' && $BINARY --relay 127.0.0.1:9876 --player-id 1 --players 3 --windowed
\"
    set custom title of front window to \"[Player 1]\"
end tell
"

sleep 1

echo "=== 启动 Player 2 ==="
osascript -e "
tell application \"Terminal\"
    activate
    set newTab to do script \"cd '$(pwd)' && $BINARY --relay 127.0.0.1:9876 --player-id 2 --players 3 --windowed
\"
    set custom title of front window to \"[Player 2]\"
end tell
"

echo ""
echo "=== 全部已启动 ==="
echo "每个窗口按 Ctrl+C 可单独停止"
