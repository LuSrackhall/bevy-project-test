#!/usr/bin/env python3
"""宪法 §10.2 / §22.1：hash_world_state 覆盖率守卫。

问题：新增仿真组件后若忘记同步更新 `hash_world_state`，确定性黄金测试会
**漏检**确定性回退（两次运行仍然相等，但状态的一部分从未参与比对）。

做法：枚举 `crates/simulation/src` 中所有 `#[derive(... Component ...)]` 的结构体，
要求每个组件要么出现在 `golden_test.rs` 的 `get::<T>()` 覆盖里，要么在本文件的
`EXEMPT` 名单中——豁免必须写明理由，让豁免在代码审查中可见。

用法：`python3 scripts/check-hash-coverage.py`（CI 在 Linux job 中调用）
退出码：0 = 覆盖完整；1 = 存在未覆盖组件
"""

import re
import sys
import pathlib

ROOT = pathlib.Path(__file__).resolve().parent.parent
SRC = ROOT / "crates" / "simulation" / "src"
GOLDEN = SRC / "golden_test.rs"

# 显式豁免：键为组件名，值为豁免理由（必须具体，不得留空）
EXEMPT = {
    "UnitIdComponent": "身份组件，已按 UnitId 单独哈希（见 hash_world_state 中的 uid.0）",
    "SoldierMarker": "零尺寸标记组件，不携带状态",
    "CityMarker": "零尺寸标记组件，不携带状态",
    "WaypointMarker": "零尺寸标记组件，不携带状态",
    "ArrowMarker": "零尺寸标记组件，不携带状态",
}


def collect_components():
    """返回 {组件名: 相对路径}，仅统计带 Component 派生的 pub struct。"""
    found = {}
    pattern = re.compile(
        r"#\[derive\(([^)]*)\)\]\s*(?:#\[[^\]]*\]\s*)*pub struct ([A-Za-z0-9_]+)"
    )
    for path in sorted(SRC.rglob("*.rs")):
        text = path.read_text(encoding="utf-8")
        for derives, name in pattern.findall(text):
            if re.search(r"\bComponent\b", derives):
                found[name] = path.relative_to(ROOT)
    return found


def main() -> int:
    golden = GOLDEN.read_text(encoding="utf-8")
    components = collect_components()

    stale = [name for name in EXEMPT if name not in components]
    if stale:
        print(f"::warning::EXEMPT 中存在已不存在的组件，请清理：{', '.join(sorted(stale))}")

    required = sorted(n for n in components if n not in EXEMPT)
    missing = [n for n in required if f"get::<{n}>()" not in golden]

    print(
        f"扫描到 {len(components)} 个仿真组件；豁免 {len(EXEMPT)} 个；要求覆盖 {len(required)} 个"
    )
    if missing:
        print("::error::hash_world_state 未覆盖以下仿真组件（宪法 §10.2）：", file=sys.stderr)
        for name in missing:
            print(f"  - {name}  ({components[name]})", file=sys.stderr)
        print(
            "请更新 hash_world_state 补齐字段，或在 scripts/check-hash-coverage.py 的 "
            "EXEMPT 中显式豁免并写明理由。",
            file=sys.stderr,
        )
        return 1

    print("PASS: hash_world_state 覆盖完整")
    return 0


if __name__ == "__main__":
    sys.exit(main())
