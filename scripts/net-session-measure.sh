#!/usr/bin/env bash
# 本地双客户端联机「推进节奏」测量 —— 自动化自测用（人工验收见 net-acceptance.sh）。
#
# 用法:
#   scripts/net-session-measure.sh [latency_ms] [jitter_ms] [loss_pct] [seconds] [sampler_cmd]
#
# 参数说明:
#   latency_ms/jitter_ms/loss_pct  relay 广播出口的链路注入（0 0 0 = 真实链路）
#   seconds                        采样窗口秒数（默认 20）
#   sampler_cmd                    可选：采样窗口开始前执行（用于让外部注入操作，如 UI 点击）
#
# 输出：人读摘要 + 一行 JSON（c0/c1 的逐帧 tick 直方图、定格/跳变/阻塞帧、同步检查）。
# 退出码：0 = 两侧同步；1 = 不同步或未进局；2 = 环境错误。
#
# 依赖：已构建的 relay 与 city-conquest（--features remote）。构建由调用方负责，避免本脚本
# 在测量中悄悄重编译。

set -uo pipefail
cd "$(dirname "$0")/.." || exit 2

LAT=${1:-0}; JIT=${2:-0}; LOSS=${3:-0}; SECS=${4:-20}
TARGET=${CARGO_TARGET_DIR:-/Volumes/SSD980/bevy-cache/target}/debug
RELAY_ID=${RELAY_ID:-7}
BRP0=${BRP0:-15702}; BRP1=${BRP1:-15712}
EXTRA_CLIENT_ARGS=${EXTRA_CLIENT_ARGS:-}

LOG_DIR=$(mktemp -d /tmp/netmeasure.XXXXXX)
PORT=$(python3 -c 'import socket;s=socket.socket();s.bind(("127.0.0.1",0));p=s.getsockname()[1];s.close();print(p)')

RELAY_PID=""; C0_PID=""; C1_PID=""
cleanup() {
  for p in "$C0_PID" "$C1_PID" "$RELAY_PID"; do
    [ -n "$p" ] && kill "$p" 2>/dev/null
  done
  sleep 0.5
  for p in "$C0_PID" "$C1_PID" "$RELAY_PID"; do
    [ -n "$p" ] && kill -9 "$p" 2>/dev/null
  done
  echo "  （日志: ${LOG_DIR}）"
}
trap cleanup EXIT INT TERM

echo "=== 链路注入: ${LAT}ms ±${JIT}ms, loss ${LOSS}% | 端口 $PORT | 窗口 ${SECS}s ==="
"$TARGET/relay" --port "$PORT" --seed 42 --players 2 --relay-id "$RELAY_ID" --loopback \
  --latency-ms "$LAT" --jitter-ms "$JIT" --loss-pct "$LOSS" > "$LOG_DIR/relay.log" 2>&1 &
RELAY_PID=$!
sleep 1

BEVY_ASSET_ROOT="$PWD" "$TARGET/city-conquest" --windowed \
  --relay "127.0.0.1:$PORT" --player-id 0 --players 2 --relay-id "$RELAY_ID" \
  --brp-port "$BRP0" $EXTRA_CLIENT_ARGS > "$LOG_DIR/c0.log" 2>&1 &
C0_PID=$!
BEVY_ASSET_ROOT="$PWD" "$TARGET/city-conquest" --windowed \
  --relay "127.0.0.1:$PORT" --player-id 1 --players 2 --relay-id "$RELAY_ID" \
  --brp-port "$BRP1" $EXTRA_CLIENT_ARGS > "$LOG_DIR/c1.log" 2>&1 &
C1_PID=$!

python3 - "$BRP0" "$BRP1" "$SECS" <<'PY'
import json, sys, time, urllib.request

p0, p1, secs = int(sys.argv[1]), int(sys.argv[2]), float(sys.argv[3])

def brp(port, method, params=None, timeout=5):
    body = json.dumps({"jsonrpc":"2.0","id":1,"method":method,"params":params or {}}).encode()
    req = urllib.request.Request(f"http://127.0.0.1:{port}/", data=body,
                                 headers={"content-type":"application/json"})
    with urllib.request.urlopen(req, timeout=timeout) as r:
        resp = json.loads(r.read().decode())
    if "error" in resp:
        raise RuntimeError(resp["error"])
    return resp.get("result")

def wait_playing(port, secs=150):
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
    return {"tick": r["tick"], "hash": r["world_hash"], "soldiers": r.get("total_soldiers", 0),
            "frames": p.get("frames", 0), "ticks": p.get("ticks_total", 0),
            "zero": p.get("frames_zero", 0), "burst": p.get("frames_burst", 0),
            "blocked": p.get("frames_blocked", 0), "max": p.get("max_ticks_in_frame", 0),
            "acc": p.get("accumulator_peak_ms", 0.0),
            "hist": p.get("histogram_ticks_per_frame", [])}

print("=== 等待进局 ===")
wait_playing(p0); wait_playing(p1)
print(f"=== 采样 {secs:.0f}s ===")
a0, a1 = snap(p0), snap(p1)
time.sleep(secs)
b0, b1 = snap(p0), snap(p1)

def win(x, y, label):
    df = max(y["frames"] - x["frames"], 1)
    dt = y["ticks"] - x["ticks"]
    dz = y["zero"] - x["zero"]; db = y["burst"] - x["burst"]; dk = y["blocked"] - x["blocked"]
    tpf = dt / df
    print(f"  {label}: 帧={df} tick={dt} ({tpf:.3f} tick/帧) | 定格={dz} 跳变={db} 阻塞={dk}")
    print(f"      单帧最大={y['max']} 累加器峰值={y['acc']:.0f}ms | 阻塞率={dk/df*100:.1f}% 跳变率={db/df*100:.1f}%")
    print(f"      直方图(tick/帧)={y['hist']}")
    return {"frames": df, "ticks": dt, "ticks_per_frame": tpf, "zero": dz, "burst": db,
            "blocked": dk, "blocked_pct": dk/df*100, "burst_pct": db/df*100,
            "max_single_frame": y["max"], "acc_peak_ms": y["acc"], "histogram": y["hist"]}

w0 = win(a0, b0, "c0")
w1 = win(a1, b1, "c1")
sync = b0["hash"] == b1["hash"] and b0["tick"] == b1["tick"]
print(f"=== 同步: c0 tick={b0['tick']} / c1 tick={b1['tick']} → {'一致 ✓' if sync else '不一致 ✗'}")
print(json.dumps({"c0": w0, "c1": w1, "sync_ok": sync}, ensure_ascii=False))
sys.exit(0 if sync else 1)
PY
RC=$?
echo "=== 采样退出码=$RC ==="
exit $RC
