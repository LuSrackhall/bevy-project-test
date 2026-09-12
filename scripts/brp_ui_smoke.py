#!/usr/bin/env python3
"""BRP 驱动的 UI 端到端验收（agent 原生，无需人眼看屏幕）。

它验证的是"agent 能否自己操作并检查 UI"这条闭环：

  1. 连上运行中客户端的 BRP（Bevy Remote Protocol，JSON-RPC）
  2. 读仿真状态（自定义方法 `city_conquest/probe`）
  3. 枚举 UI 文本（`world.query`），确认主菜单可见
  4. **注入点击**：向地图大小按钮"小 (2000)"触发 `bevy_ui_widgets::Activate`
     （选它而不是"局域网模式"：这条路径不启动 LAN 发现监听器，
     因此不会在 macOS 上触发防火墙的入站授权询问）
  5. 断言**对局真的跑起来了**：轮询 `city_conquest/probe` 直到 `tick > 0`
  6. **请求截图落盘**（`city_conquest/screenshot`），产物可被 agent 直接读取

用法：
    # 终端 A：启动客户端（需 GPU/窗口）
    #   直接跑二进制时必须给 BEVY_ASSET_ROOT，否则 Bevy 会以可执行文件所在目录
    #   作为资源根，找不到 assets/（用 `cargo run` 时由 CARGO_MANIFEST_DIR 兜底）。
    BEVY_ASSET_ROOT="$PWD" /path/to/city-conquest --windowed
    # 终端 B：
    python3 scripts/brp_ui_smoke.py

退出码：0 全部通过 / 1 断言失败 / 2 环境错误（BRP 未就绪等）
"""

from __future__ import annotations

import argparse
import json
import pathlib
import sys
import time
import urllib.error
import urllib.request

ENDPOINT = "http://127.0.0.1:15702/"
MENU_LABEL = "城池争霸"
START_LABEL = "小 (2000)"


def brp(method: str, params: dict | None = None, timeout: float = 10.0) -> dict:
    body = json.dumps(
        {"jsonrpc": "2.0", "id": 1, "method": method, "params": params or {}}
    ).encode()
    req = urllib.request.Request(
        ENDPOINT, data=body, headers={"content-type": "application/json"}
    )
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        payload = json.loads(resp.read().decode())
    if "error" in payload:
        raise RuntimeError(f"BRP 错误 ({method}): {payload['error']}")
    return payload.get("result")


def wait_for_brp(seconds: float) -> dict:
    """等客户端启动并连上 BRP；返回一次 probe 结果。"""
    deadline = time.time() + seconds
    last_err = ""
    while time.time() < deadline:
        try:
            return brp("city_conquest/probe")
        except Exception as exc:  # noqa: BLE001 - 启动期任意失败都应重试
            last_err = str(exc)
            time.sleep(0.5)
    raise SystemExit(f"[exit 2] {seconds:.0f}s 内未能连上 BRP（{ENDPOINT}）：{last_err}")


def find_type_path(fragment: str) -> str:
    """在 BRP 的类型注册表里找类型全路径（不硬编码 Bevy 内部路径）。"""
    schema = brp("registry.schema")
    types = schema if isinstance(schema, list) else schema.get("types", schema)
    candidates = []
    if isinstance(types, dict):
        candidates = list(types.keys())
    elif isinstance(types, list):
        candidates = [t.get("typePath") or t.get("type_path") or "" for t in types]
    for path in candidates:
        if path.endswith(fragment):
            return path
    raise SystemExit(f"[exit 2] 注册表中找不到以 {fragment} 结尾的类型")


def query(components: list[str]) -> list[dict]:
    result = brp("world.query", {"data": {"components": components}})
    return result if isinstance(result, list) else []


def text_entities(text_path: str) -> list[tuple[int, str]]:
    """返回 [(entity_id, 文本内容)]。

    BRP `world.query` 的返回形状是
    `[{"entity": <int>, "components": {"<type path>": <value>}}, ...]`；
    这里只对 `components` 下的目标类型取值，再递归抽取字符串，
    以免绑定到某个具体 Bevy 版本的序列化细节。
    """

    def collect_strings(node) -> list[str]:
        if isinstance(node, str):
            return [node]
        if isinstance(node, dict):
            out = []
            for value in node.values():
                out += collect_strings(value)
            return out
        if isinstance(node, list):
            out = []
            for value in node:
                out += collect_strings(value)
            return out
        return []

    found = []
    for row in query([text_path]):
        entity = row.get("entity")
        if not isinstance(entity, int):
            continue
        raw = (row.get("components") or {}).get(text_path)
        texts = collect_strings(raw)
        if texts:
            found.append((entity, texts[0]))
    return found


def parent_of(entity: int, childof_path: str):
    """取该实体的父实体 id（ChildOf 的值形状随版本而变，这里取第一个整数）。"""
    result = brp("world.get_components", {"entity": entity, "components": [childof_path]})
    blob = result.get("components", result) if isinstance(result, dict) else result

    def first_int(node):
        if isinstance(node, bool):
            return None
        if isinstance(node, int):
            return node
        if isinstance(node, dict):
            for value in node.values():
                got = first_int(value)
                if got is not None:
                    return got
        if isinstance(node, list):
            for value in node:
                got = first_int(value)
                if got is not None:
                    return got
        return None

    return first_int(blob)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--wait", type=float, default=90.0, help="等待 BRP 的秒数")
    parser.add_argument(
        "--shot", default="/tmp/city-conquest-shot.png", help="截图输出路径"
    )
    args = parser.parse_args()

    checks: list[dict] = []

    def record(name: str, ok: bool, detail: str) -> None:
        checks.append({"check": name, "ok": ok, "detail": detail})
        print(f"  {'PASS' if ok else 'FAIL'}  {name}: {detail}")

    print("== BRP UI 端到端验收 ==")
    probe = wait_for_brp(args.wait)
    record("BRP 就绪 + 仿真探针", True, f"tick={probe.get('tick')} hash={probe.get('world_hash')}")

    text_path = find_type_path("::Text")
    print(f"  UI 文本类型: {text_path}")

    # 菜单在启动后的最初若干帧内构建完成，而 BRP 会先于它可用 → 轮询到有文本为止。
    before: list[tuple[dict, str]] = []
    labels_before: list[str] = []
    ui_deadline = time.time() + 20
    while time.time() < ui_deadline:
        before = text_entities(text_path)
        labels_before = [t for _, t in before]
        if labels_before:
            break
        time.sleep(0.3)
    record(
        "主菜单可见",
        MENU_LABEL in labels_before,
        f"'{MENU_LABEL}' {'在' if MENU_LABEL in labels_before else '不在'} UI 文案中"
        f"（共 {len(labels_before)} 条）",
    )

    # 找到按钮实体：文本"小 (2000)"所在实体的父实体
    target_parent = None
    childof_path = find_type_path("::ChildOf")
    for entity, text in before:
        if text == START_LABEL:
            target_parent = parent_of(entity, childof_path)
            if target_parent is None:
                print(f"  （未能从实体 {entity} 解析出父按钮）")
            break

    if target_parent is None:
        record("定位按钮实体", False, "未能从文本实体解析出父按钮")
        print(
            "  调试：UI 文案样本 =",
            labels_before[:12],
        )
        print(json.dumps({"verdict": "fail", "checks": checks}, ensure_ascii=False))
        return 1

    activate_path = find_type_path("::Activate")
    brp("world.trigger_event", {"event": activate_path, "value": {"entity": target_parent}})
    record("注入点击 Activate", True, f"event={activate_path} entity={target_parent}")

    # 断言对局真的开始：tick 从 0 开始推进（点击 → NeedsGameReset + GameState::Playing）
    deadline = time.time() + 20
    tick_after = 0
    while time.time() < deadline:
        try:
            tick_after = brp("city_conquest/probe").get("tick", 0)
        except Exception:  # noqa: BLE001 - 状态切换瞬间 probe 可能短暂报错
            tick_after = 0
        if tick_after and tick_after > 0:
            break
        time.sleep(0.3)
    record("点击后对局开始推进", tick_after > 0, f"tick={tick_after}（点击前 tick={probe.get('tick')}）")

    shot_path = pathlib.Path(args.shot)
    if shot_path.exists():
        shot_path.unlink()
    brp("city_conquest/screenshot", {"path": str(shot_path)})
    shot_deadline = time.time() + 15
    while time.time() < shot_deadline and not shot_path.exists():
        time.sleep(0.3)
    shot_ok = shot_path.exists() and shot_path.stat().st_size > 0
    record(
        "截图落盘",
        shot_ok,
        f"{shot_path}（{shot_path.stat().st_size if shot_ok else 0} 字节）",
    )

    ok = all(c["ok"] for c in checks)
    print(json.dumps({"verdict": "pass" if ok else "fail", "checks": checks}, ensure_ascii=False, indent=2))
    return 0 if ok else 1


if __name__ == "__main__":
    try:
        sys.exit(main())
    except urllib.error.URLError as exc:
        print(f"[exit 2] 无法连接 BRP（{ENDPOINT}）：{exc}", file=sys.stderr)
        sys.exit(2)
