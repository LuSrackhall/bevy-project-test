#!/usr/bin/env bash
# 本地 N 人联机「推进节奏」测量 —— 自动化自测用（人工验收见 net-acceptance.sh）。
#
# 用法:
#   scripts/net-session-measure.sh [latency_ms] [jitter_ms] [loss_pct] [seconds]
#
# 环境变量:
#   PLAYERS=2|3|4     联机人数（默认 2；规格上限为 4 —— 不止双人，修复必须按 N 人验证）
#   SCREEN_W/SCREEN_H 铺窗用的可用屏幕尺寸（默认 1440x900，单显示器也要能把 N 个窗口摆开）
#   RELAY_ID          固定 relay 身份（默认 7；客户端必须传同一个值）
#
# 说明:
#   latency_ms/jitter_ms/loss_pct 是 relay 广播出口的链路注入（0 0 0 = 真实链路）。
#   loopback 上 RTT≈0、几乎不丢包，本地稳态测不出真实 WiFi/公网症状，必须显式注入。
#
# 输出：人读摘要 + 一行 JSON（每个客户端的逐帧 tick 直方图、定格/跳变/阻塞帧、
#      累加器峰值，以及全体同步检查）。
# 退出码：0 = 全体同步；1 = 不同步或未进局；2 = 环境错误。

set -uo pipefail
cd "$(dirname "$0")/.." || exit 2

LAT=${1:-0}; JIT=${2:-0}; LOSS=${3:-0}; SECS=${4:-20}
PLAYERS=${PLAYERS:-2}
if [ "$PLAYERS" -lt 2 ] || [ "$PLAYERS" -gt 4 ]; then
  echo "[exit 2] PLAYERS 必须是 2..4（规格上限为 4）" >&2
  exit 2
fi
SCREEN_W=${SCREEN_W:-1440}; SCREEN_H=${SCREEN_H:-900}
TARGET=${CARGO_TARGET_DIR:-/Volumes/SSD980/bevy-cache/target}/debug
RELAY_ID=${RELAY_ID:-7}
BRP_BASE=${BRP_BASE:-15702}
BRP_STEP=${BRP_STEP:-10}
EXTRA_CLIENT_ARGS=${EXTRA_CLIENT_ARGS:-}

LOG_DIR=$(mktemp -d /tmp/netmeasure.XXXXXX)
PORT=$(python3 -c 'import socket;s=socket.socket();s.bind(("127.0.0.1",0));p=s.getsockname()[1];s.close();print(p)')

PIDS=()
cleanup() {
  for p in "${PIDS[@]:-}"; do [ -n "$p" ] && kill "$p" 2>/dev/null; done
  sleep 0.5
  for p in "${PIDS[@]:-}"; do [ -n "$p" ] && kill -9 "$p" 2>/dev/null; done
  echo "  （日志: ${LOG_DIR}）"
}
trap cleanup EXIT INT TERM

# ── 铺窗：N=2 一行两个；N=3 一行三个；N=4 两行两列（单显示器可一眼看全）──
if [ "$PLAYERS" -le 3 ]; then COLS=$PLAYERS; else COLS=2; fi
ROWS=$(( (PLAYERS + COLS - 1) / COLS ))
GAP=20
WIN_W=$(( (SCREEN_W - GAP * (COLS + 1)) / COLS ))
WIN_H=$(( (SCREEN_H - 80 - GAP * (ROWS + 1)) / ROWS ))

echo "=== 链路注入: ${LAT}ms ±${JIT}ms, loss ${LOSS}% | ${PLAYERS} 人 | 端口 $PORT | 窗口 ${SECS}s ==="
"$TARGET/relay" --port "$PORT" --seed 42 --players "$PLAYERS" --relay-id "$RELAY_ID" --loopback \
  --latency-ms "$LAT" --jitter-ms "$JIT" --loss-pct "$LOSS" > "$LOG_DIR/relay.log" 2>&1 &
PIDS+=("$!")
sleep 1

BRP_PORTS=()
for i in $(seq 1 "$PLAYERS"); do
  idx=$((i - 1))
  col=$((idx % COLS)); row=$((idx / COLS))
  x=$((GAP + col * (WIN_W + GAP)))
  y=$((80 + row * (WIN_H + GAP)))
  brp=$((BRP_BASE + idx * BRP_STEP))
  BRP_PORTS+=("$brp")
  BEVY_ASSET_ROOT="$PWD" "$TARGET/city-conquest" --windowed \
    --relay "127.0.0.1:$PORT" --player-id "$idx" --players "$PLAYERS" --relay-id "$RELAY_ID" \
    --brp-port "$brp" --window-pos "${x},${y}" --window-size "${WIN_W},${WIN_H}" \
    $EXTRA_CLIENT_ARGS > "$LOG_DIR/c${idx}.log" 2>&1 &
  PIDS+=("$!")
done
echo "  已启动 ${PLAYERS} 个客户端（BRP: ${BRP_PORTS[*]}）"

python3 - "${BRP_PORTS[@]}" "$SECS" <<'PY'
import json, sys, time, urllib.request

ports = [int(x) for x in sys.argv[1:-1]]
secs = float(sys.argv[-1])

def brp(port, method, params=None, timeout=5):
    body = json.dumps({"jsonrpc": "2.0", "id": 1, "method": method, "params": params or {}}).encode()
    req = urllib.request.Request(f"http://127.0.0.1:{port}/", data=body,
                                 headers={"content-type": "application/json"})
    with urllib.request.urlopen(req, timeout=timeout) as r:
        resp = json.loads(r.read().decode())
    if "error" in resp:
        raise RuntimeError(resp["error"])
    return resp.get("result")

def wait_playing(port, secs=180):
    end = time.time() + secs
    while time.time() < end:
        try:
            r = brp(port, "city_conquest/probe")
            if r.get("tick", 0) > 100:
                return r
        except Exception:
            pass
        time.sleep(0.5)
    raise SystemExit(f"[exit 2] 端口 {port} 未在 {secs}s 内进入对局")

def snap(port):
    r = brp(port, "city_conquest/probe")
    p = r.get("pacing") or {}
    return {"port": port, "tick": r["tick"], "hash": r["world_hash"],
            "soldiers": r.get("total_soldiers", 0),
            "frames": p.get("frames", 0), "ticks": p.get("ticks_total", 0),
            "zero": p.get("frames_zero", 0), "burst": p.get("frames_burst", 0),
            "blocked": p.get("frames_blocked", 0), "max": p.get("max_ticks_in_frame", 0),
            "acc": p.get("accumulator_peak_ms", 0.0),
            "hist": p.get("histogram_ticks_per_frame", [])}

print(f"=== 等待 {len(ports)} 个客户端进局 ===")
for p in ports:
    wait_playing(p)
print(f"=== 采样 {secs:.0f}s ===")
before = [snap(p) for p in ports]
time.sleep(secs)
after = [snap(p) for p in ports]

results = []
for i, (x, y) in enumerate(zip(before, after)):
    df = max(y["frames"] - x["frames"], 1)
    dt = y["ticks"] - x["ticks"]
    dz = y["zero"] - x["zero"]; db = y["burst"] - x["burst"]; dk = y["blocked"] - x["blocked"]
    print(f"  c{i}: 帧={df} tick={dt} ({dt/df:.3f} tick/帧 = {dt/secs:.1f}Hz 名义 20Hz)"
          f" | 定格={dz} 跳变={db} 阻塞={dk} ({dk/df*100:.1f}%)")
    print(f"      单帧最大={y['max']} 累加器峰值={y['acc']:.0f}ms 直方图={y['hist']}")
    results.append({"frames": df, "ticks": dt, "hz": dt/secs, "ticks_per_frame": dt/df,
                    "zero": dz, "burst": db, "blocked": dk, "blocked_pct": dk/df*100,
                    "max_single_frame": y["max"], "acc_peak_ms": y["acc"], "histogram": y["hist"],
                    "tick": y["tick"], "hash": y["hash"], "soldiers": y["soldiers"]})

ticks = {r["tick"] for r in results}
hashes = {r["hash"] for r in results}
sync = len(hashes) == 1 and (max(ticks) - min(ticks)) <= 1
print(f"=== 同步: ticks={sorted(ticks)} hashes={len(hashes)} 种 → {'一致 ✓' if sync else '不一致 ✗'}"
      f"（tick 允许 ±1 采样竞态）")
print(json.dumps({"players": len(results), "sync_ok": sync, "clients": results}, ensure_ascii=False))
sys.exit(0 if sync else 1)
PY
RC=$?
echo "=== 采样退出码=$RC ==="
exit $RC
