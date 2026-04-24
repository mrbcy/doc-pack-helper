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

## GitHub 自动打包

仓库已经预留 GitHub Actions 自动发布流程，文件在 `.github/workflows/release.yml`。

- 正常发布：推送版本 tag 后会自动构建 macOS 和 Windows 安装包，并上传到对应 GitHub Release
- 补发已有 tag：在 GitHub 的 `Actions -> Release Desktop App -> Run workflow` 中填入 `tag_name`，例如 `0.1`

建议发布方式：

```bash
git tag 0.1.1
git push origin main
git push origin 0.1.1
```

说明：

- 当前 workflow 使用 GitHub 自带的 `GITHUB_TOKEN` 创建或更新 Release
- 目前未配置 macOS 签名、公证和 Windows 代码签名，因此安装时仍可能看到系统安全提示
- Windows 正式发布请优先使用 GitHub Actions 生成的安装包；macOS 上交叉编译得到的 `x86_64-pc-windows-gnu` 版本可能出现快捷方式或任务栏图标异常
- GitHub Actions 中的 Windows 自动发布当前只构建 NSIS `setup.exe`，不走 WiX MSI，主要是为了避开 `light.exe` / VBSCRIPT 这一类不稳定因素
