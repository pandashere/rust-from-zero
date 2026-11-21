# 参与维护

## 修改课程

先阅读根目录 `AGENTS.md`、`docs/course-design.md` 和 `lessons/TEMPLATE.md`。新课程或改写必须保持零基础可读：术语就近解释，方案先交代问题背景，示例尽量完整可运行。

每课是独立 Cargo package。讲义、主练习、课后巩固、测试和三级提示要一起修改，不能只更新其中一项。不要在默认学习 target 或提示文件中提交完整参考答案。

## 本地检查

```bash
python3 tools/check_course.py
python3 tools/check_markdown_links.py
python3 -m unittest discover -s tools/tests -v
python3 -m unittest discover -s viewer/tests -v
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --no-run
cargo clippy --workspace --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

`cargo test --workspace` 会执行学习者尚未完成的 `todo!()`，因此失败不代表仓库损坏。持续集成使用 `--no-run` 确认所有行为测试都能编译；维护者还应在 `/tmp` 的独立副本中完成实现并运行行为测试，不将答案带回仓库。

## 修改阅读器或 AI 接入

只使用 Python 标准库。同步更新 `docs/agent-integration.md`，并实际验证首页、文档 API、agent API 与 MCP 当前章节读取流程。默认地址必须保持本机回环地址，不能为了方便默认暴露到公共网络。

## 发布前清理

不要提交 `target/`、Python 缓存、本地学习记录、终端会话、模型密钥或其他凭据。检查讲义和配置中不应出现作者机器的绝对路径。新增文件是否纳入仓库，应与它的课程或工具用途一致。
