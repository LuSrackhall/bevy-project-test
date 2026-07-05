import re

with open('openspec/changes/session-bootstrap-layer/brainstorm-spec.md', 'r') as f:
    content = f.read()

old = '    }\n}\n\n```\n\nSessionBootstrap 分两阶段：dispatch（类型安全）→ wire：'

new = '''    }
}

### D4.1 SessionArtifacts Ownership

`SessionArtifacts` 是初始化阶段的一次性所有权对象（one-shot ownership）：

- **Initializer 创建它**
- **`wire()` 消费它**——将内部资源分别注册到 Driver、Bevy Resources、Recorder
- **`wire()` 完成后，`SessionArtifacts` 不再存在**

因此：
- 不应 `Clone`
- 不应 `Rc`/`Arc` 跨线程共享
- 不应跨 runtime 生命周期保存

此原则作用于 session-bootstrap-layer 变更引入的抽象，非全局宪法。

SessionBootstrap 分两阶段：dispatch（类型安全）→ wire：'''

if old in content:
    content = content.replace(old, new, 1)
    with open('openspec/changes/session-bootstrap-layer/brainstorm-spec.md', 'w') as f:
        f.write(content)
    print('OK')
else:
    print('NOT FOUND')
