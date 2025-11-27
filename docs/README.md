# AgentX 文档

本目录包含 AgentX 应用开发过程中的技术文档和最佳实践。

## 📚 文档列表

### [Collapsible 组件与 Entity 生命周期管理](./collapsible-entity-lifecycle.md) ⭐ 最新

**问题**：Collapsible 展开/折叠按钮无响应

**涉及技术**：
- GPUI Entity 生命周期
- Collapsible 组件使用
- 状态管理模式

**核心要点**：
- ❌ 不要在 `render()` 中创建 Entity
- ✅ 在 `new()` 中创建并存储 Entity
- ✅ 在 `render()` 中只引用已存储的 Entity

**适用场景**：
- 使用 Collapsible 组件
- 管理多个可交互的子组件
- 需要持久化组件状态

### [Agent Message 组件文档](./AGENT_MESSAGE.md)

AI 代理消息显示组件的设计和使用说明。

### [User Message 组件文档](./USER_MESSAGE.md)

用户消息显示组件的设计和使用说明，包含资源附件处理。

### [Tool Call Item 组件文档](./TOOL_CALL_ITEM.md)

工具调用项组件的设计和使用说明，支持状态展示和折叠。

---

## 🏗️ 项目架构

AgentX 是一个基于 GPUI 的桌面应用，展示了：
- Dock 布局系统
- 多面板管理
- AI 对话界面
- 代码编辑器集成
- 任务列表管理

### 主要目录结构

```
examples/agentx/
├── src/
│   ├── main.rs              # 应用入口
│   ├── lib.rs               # 核心初始化
│   ├── conversation.rs      # 会话面板（修复后）
│   ├── components/          # UI 组件
│   │   ├── agent_message.rs
│   │   ├── user_message.rs
│   │   ├── tool_call_item.rs
│   │   └── ...
│   ├── fixtures/            # 模拟数据
│   └── ...
├── docs/                    # 📖 技术文档
│   ├── README.md           # 本文件
│   ├── collapsible-entity-lifecycle.md  # ⭐ Entity 生命周期管理
│   ├── AGENT_MESSAGE.md    # Agent 消息组件
│   ├── USER_MESSAGE.md     # 用户消息组件
│   └── TOOL_CALL_ITEM.md   # 工具调用组件
└── ...
```

## 🔗 相关资源

- [GPUI 官方文档](https://www.gpui.rs/)
- [gpui-component 组件库](../../crates/ui/)
- [CollapsibleStory 参考实现](../src/collapsible_story.rs)

## 💡 贡献指南

如果你发现了新的技术问题或最佳实践，欢迎添加文档到此目录：

1. 创建 `your-topic.md` 文件
2. 使用清晰的标题和代码示例
3. 更新本 README 添加索引
4. 包含问题、原因、解决方案三部分

## 📝 文档模板

```markdown
# 标题

## 问题概述
描述问题症状...

## 根本原因
解释为什么会出现这个问题...

## 解决方案
提供正确的代码示例...

## 最佳实践
总结要点...
```
