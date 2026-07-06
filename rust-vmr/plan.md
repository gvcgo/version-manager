# Rust 改写计划

## 已完成

- [x] `vmr-config` — 配置管理 (TOML + 工作目录 + 环境变量)
- [x] `vmr-utils` — 工具函数 (文件操作 + 解压 + 版本排序 + shell 辅助)
- [x] `vmr-shell` — Shell 环境集成 (bash/zsh/fish)

## 待完成

- [ ] `vmr-download` — SDK 文件下载 (多线程 + 校验和)
- [ ] `vmr-lua` — Lua 插件系统 (mlua 绑定, 兼容现有 .lua 插件)
- [ ] `vmr-install` — 安装器后端 (archive/coursier/executable/rustup, 不含 conda)
- [ ] `vmr-tui` — ratatui TUI 界面
- [ ] `vmr-cli` — clap CLI 入口
- [ ] `vmr-self` — VMR 自身安装/更新/卸载
- [ ] Shell 集成增强 (Windows PowerShell 支持)
- [ ] 集成测试 & 与 Go 版本的行为对比
