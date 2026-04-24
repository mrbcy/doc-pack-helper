# 材料助手

一个基于 Vue 2 + Tauri 的跨平台桌面工具，用来整理报名、招聘、入职等场景常用材料，并按本次要求一键复制到目标目录。

## 当前功能

- 材料库管理：按分类维护材料，每个材料可绑定多个文件版本，并指定默认文件
- 本次导出：手动选择材料、修改本次导出文件名、选择导出目录后批量复制
- 智能识别：粘贴招聘材料要求后，调用兼容 OpenAI `/chat/completions` 的 LLM 自动识别材料清单
- 智能批量导入：选择文件夹后，借助 LLM 自动建议分类并导入材料库
- 本地持久化：通过 Tauri + SQLite 保存材料、分类和设置
- 跨平台打包：支持 macOS 与 Windows 安装包构建

## 技术栈

- Vue 2
- Element UI
- Vite
- Tauri 2
- SQLite

## 本地开发

```bash
npm install
npm run dev
```

启动 Tauri 桌面调试：

```bash
source "$HOME/.cargo/env"
npx tauri dev
```

## 打包

macOS：

```bash
source "$HOME/.cargo/env"
npx tauri build
```

Windows 交叉编译：

```bash
source "$HOME/.cargo/env"
CC_x86_64_pc_windows_gnu=/opt/homebrew/bin/x86_64-w64-mingw32-gcc \
CXX_x86_64_pc_windows_gnu=/opt/homebrew/bin/x86_64-w64-mingw32-g++ \
AR_x86_64_pc_windows_gnu=/opt/homebrew/bin/x86_64-w64-mingw32-ar \
CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=/opt/homebrew/bin/x86_64-w64-mingw32-gcc \
npx tauri build --target x86_64-pc-windows-gnu
```
