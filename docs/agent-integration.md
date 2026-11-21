# 接入 AI 助教：agent 集成指南

这门课的网页阅读器（`python3 viewer/serve.py`）右侧有一个侧栏，可以直接启动一个 AI 助手陪你学习：它会读你正在看的章节、帮你解释概念、按提示分级辅导练习。这篇文档讲清楚它怎么工作、怎么装好。

先解释两个会反复出现的词：

- **AI CLI**：在终端里运行的 AI 助手程序（CLI 是 Command Line Interface，命令行界面）。Kimi Code、Codex、Claude Code 都是 AI CLI。它们和普通聊天机器人的区别是：它们能在你的电脑上读文件、运行命令，所以能直接看课程代码、帮你跑测试。
- **MCP**：Model Context Protocol（模型上下文协议），一座“桥梁”。AI 助手本身不知道你正在浏览器里看哪一节；阅读器把“你当前在看什么”写进一个状态文件，再通过一个叫 MCP server 的小程序（`viewer/mcp_server.py`）提供给助手。助手通过 MCP 调用两个工具：`get_current_view`（你在看哪篇哪节）和 `get_current_section`（那一节的原文）。这样你说“讲讲这一段”，助手就知道“这一段”是什么。

## 快速开始（Kimi，推荐）

Kimi Code 是本课程的零配置选项：仓库里已经放好了它需要的一切（`.kimi-code/mcp.json` 和助教 profile），装好登录就能用。

```bash
# 1. 安装 Kimi Code CLI（macOS / Linux；Windows 见官网）
curl -fsSL https://code.kimi.com/kimi-code/install.sh | bash

# 2. 登录（任意选一种方式，按提示完成）
kimi login        # 或者先运行 kimi，再在界面里输入 /login

# 3. 回到课程根目录，启动阅读器
python3 viewer/serve.py
```

打开 http://127.0.0.1:8000 ，侧栏会自动选中 Kimi 并启动一个“课程助教”会话。第一次在这个仓库里启动 Kimi 时，它会问你是否信任这个文件夹（因为仓库里有项目级 MCP 配置），选择信任即可；之后它就能通过 `get_current_view` / `get_current_section` 看到“你正在读的内容”。

注意：阅读和做题本身不需要任何 AI；AI CLI 的**安装和登录必须你自己完成**，阅读器不会也不能替你登录。

## 支持矩阵

| AI CLI | 侧栏启动 | 读取“你正在看的章节” | 需要做什么 |
| --- | --- | --- | --- |
| Kimi Code | 自动（默认） | 自动（项目级 `.kimi-code/mcp.json` + `rust-course-mentor` 助教 profile） | 安装并登录；首次信任本仓库 |
| Claude Code | 自动（已安装时在列表中） | 自动（项目级 `.mcp.json` + `CLAUDE.md` 助教说明） | 安装并登录；首次批准项目 MCP server |
| Codex | 自动（已安装时在列表中） | 需要一次性手动配置（见下文） | 安装并登录；按下文改 `~/.codex/config.toml` |
| Grok / OMP | 自动（已安装时在列表中） | 无自动能力（Grok 会收到指向状态文件的规则参数） | 自行参考“其他 agent”一节 |
| 其他任何 CLI | 通过 `RUST_COURSE_AGENT` 自定义命令 | 通过通用 MCP 接入或环境变量降级方案（见下文） | 自行配置 |

侧栏里“启动 CLI”这件事对所有 agent 都是真实可用的：进程在课程根目录以终端（PTY）方式启动，和你自己在终端里运行它等价。区别只在“能不能自动读取当前章节”——这取决于该工具是否支持 MCP。

## Kimi 详细说明

### 默认开箱流程

侧栏为 Kimi 执行的命令是：

```bash
kimi --agent rust-course-mentor
```

- `--agent rust-course-mentor` 选择仓库 `.agents/agents/rust-course-mentor.md` 里定义的助教 profile：它会遵守课程的教学方式（零基础友好、术语附中文解释），辅导作业时按各课 `HINTS.md` 分级提示，不会直接贴答案。
- Kimi 从课程根目录启动时会自动发现 `.kimi-code/mcp.json`，从而连接 `viewer/mcp_server.py`。模型和 thinking 强度不写在 profile 里（写在 frontmatter 的 `model` 字段会被官方忽略），由启动命令选择。

### 显式指定 K3 模型与 high thinking 强度

想让助教用 K3 模型、high 思考强度，在终端里运行：

```bash
KIMI_MODEL_THINKING_EFFORT=high kimi --model kimi-code/k3 --agent rust-course-mentor
```

想让阅读器侧栏也用这套配置启动 Kimi：

```bash
RUST_COURSE_AGENT="kimi --model kimi-code/k3 --agent rust-course-mentor" \
KIMI_MODEL_THINKING_EFFORT=high \
python3 viewer/serve.py
```

（`KIMI_MODEL_THINKING_EFFORT` 会被侧栏启动的子进程继承，所以对侧栏里的 Kimi 同样生效。）

也可以完全不用阅读器：在课程根目录的终端里直接运行上面的 `kimi` 命令，效果一样——MCP 配置是项目级的，不依赖阅读器侧栏。只是这样启动前，需要先让阅读器运行着并打开过页面，MCP 工具才有“当前章节”可读。

## Claude Code

仓库根目录已提供两个项目级适配文件：

- `.mcp.json`：Claude Code 的项目级 MCP 配置，登记了 `viewer` server（`python3 viewer/mcp_server.py`）。首次在这个仓库启动 `claude` 时，它会提示你是否启用这个项目 MCP server，批准后工具 `mcp__viewer__get_current_view` / `mcp__viewer__get_current_section` 可用。
- `CLAUDE.md`：Claude Code 的项目说明文件，写入了助教行为约定（零基础教学方式、按 HINTS.md 分级提示、不直接给答案）。

在侧栏选择 Claude 即可；也可以在课程根目录的终端里直接运行 `claude`。

## Codex

Codex 没有在仓库内登记 MCP server 的项目级入口（它读取的全局配置在你的用户目录），所以需要你手动加一次。编辑 `~/.codex/config.toml`，追加：

```toml
[mcp_servers.viewer]
command = "python3"
args = ["viewer/mcp_server.py"]
```

然后在**课程根目录**启动 `codex`（侧栏启动时工作目录已经是课程根目录；`args` 里的相对路径相对于启动目录解析）。Codex 也会自动读取根目录 `AGENTS.md`，其中已经包含本课程的教学与助教约定。

如果你的 Codex 版本对项目级配置另有支持，以其官方文档为准；上面是经过核对的通用用户级方式。

## 其他 agent：通用 MCP 接入

`viewer/mcp_server.py` 是一个标准的 **stdio MCP server**：它是一个普通子进程，通过标准输入/输出，用“每行一个 JSON-RPC 消息”的方式通信。任何支持 MCP 的 agent，只要把下面这个 server 登记进去就能用：

```json
{
  "mcpServers": {
    "viewer": {
      "command": "python3",
      "args": ["viewer/mcp_server.py"]
    }
  }
}
```

登记后 agent 会获得两个工具：

- `get_current_view`：返回 JSON，含 `currentDocument`（相对课程根目录的路径）、`currentHeading`、`scrollPercent`。
- `get_current_section`：返回当前章节的 Markdown 原文（可选参数 `maxChars`）。

没有 MCP 的 agent 还有一个降级方案：侧栏启动每个会话时，会向子进程注入两个环境变量——

- `RUST_COURSE_VIEW_STATE`：全局“当前阅读位置”JSON 文件的路径；
- `RUST_COURSE_AGENT_CONTEXT`：本会话专属的上下文 JSON 文件路径，内容随你在页面间切换实时更新（含 `currentDocument`、`workspace`、`lessonsDirectory`）。

支持自定义启动规则/系统提示的 agent，可以让它在回答前读这两个文件（侧栏对 Grok 就是这样做的）。

## 环境变量

| 变量 | 作用 | 默认值 |
| --- | --- | --- |
| `RUST_COURSE_AGENT` | 自定义侧栏启动的 agent 命令（含参数），设置后优先于自动检测 | 自动检测，顺序 Kimi → Codex → Claude → Grok → OMP |
| `RUST_COURSE_HOST` | HTTP 监听地址 | `127.0.0.1`（仅本机） |
| `RUST_COURSE_VIEW_STATE` | “当前阅读位置”状态文件的路径 | 临时目录下按用户和项目隔离的路径 |

旧名称 `LEARN_RUST_AGENT` / `LEARN_RUST_HOST` / `LEARN_RUST_VIEW_STATE` 仍作为兼容别名生效；新名称优先。新旧都不要同时依赖，建议只用新名称。

## 监听局域网的风险

默认只绑定 `127.0.0.1`，只有你自己设置的 `RUST_COURSE_HOST` 才会监听其他地址。**不要轻易设成 `0.0.0.0`**：阅读器没有任何登录验证，侧栏接口能启动终端进程。一旦监听局域网，同一网络里的任何人都能读课程文件、并且能在你的电脑上以你的身份驱动 agent 会话。确有需要时（例如虚拟机/WSL 端口转发不工作时），用完立即 Ctrl+C 关掉。

## 状态文件与多实例

- 网页每次滚动/切换文档都会把当前位置写入状态文件（默认在临时目录，文件名含你的用户 ID 和项目路径的哈希，例如 `/tmp/rust-course-viewer-state-1000-1a2b3c4d.json`）。**不同项目副本的状态文件互不相同**，不会出现“在 A 项目里读到 B 项目的章节”。
- 同一个项目同时开多个阅读器实例时，它们共享同一个状态文件，后写入的覆盖先写入的——MCP 读到的总是“最近一次”位置。需要彻底隔离时，给每个实例显式指定不同的 `RUST_COURSE_VIEW_STATE`。
- 状态文件里只有文档路径和滚动位置，没有任何凭据。每个侧栏会话的上下文文件在会话关闭时自动删除，进程也会被终止（先 SIGHUP，拒不退出再升级）。

## 故障排查

**侧栏显示 “no agent”**：没有检测到任何受支持的 CLI。安装其中一个并登录，然后**重启** `python3 viewer/serve.py`（检测只在启动时做一次）再刷新页面。

**侧栏终端显示 “Unable to start agent …” 或立刻退出**：在终端里手动运行同样的命令（例如 `kimi --agent rust-course-mentor`）看真实报错。常见原因：未登录（先 `kimi login` / `claude` 内 `/login` / `codex login`）；命令不在 PATH 上；Kimi 版本太旧不认识 `--agent`（运行 `kimi upgrade`，或把 `RUST_COURSE_AGENT=kimi` 恢复为默认助手）。

**助手说“不知道你在看哪一节”**：Kimi 检查首次启动时的信任提示是否选择了信任（可在 TUI 里用 `/mcp` 查看 `viewer` server 的连接状态）；Claude 检查是否批准了项目 `.mcp.json`；Codex 检查 `~/.codex/config.toml` 是否按上文配置、以及是否在课程根目录启动。也可以直接验证 MCP server 本身：

```bash
printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
  | python3 viewer/mcp_server.py
```

应当输出两行 JSON，第二行列出 `get_current_view` 和 `get_current_section`。

**工具返回 “No view state recorded yet”**：阅读器没在运行，或页面还没上报过位置。启动阅读器并在网页里打开任意一篇文档即可。

**工具结果带 “state is N minutes old” 警告**：页面可能已关闭，读到的是最后已知位置，提醒助手先向你确认。
