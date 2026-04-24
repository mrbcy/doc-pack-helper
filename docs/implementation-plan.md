# 实现计划

## 当前阶段：Vue 2 前端业务闭环

- 使用 Vue 2、Vite、Element UI 搭建 Mac 本地调试版本。
- 使用 `localStorage` 临时保存材料库和 LLM 配置。
- 每个材料支持多个文件版本，并且最多指定一个默认文件。
- 本次导出页面默认使用材料的默认文件，允许手动切换版本和修改导出文件名。
- 智能识别合并到本次导出页面：粘贴招聘要求后，识别结果直接加入导出清单，未匹配材料同页提示。
- 智能识别当前先使用本地规则模拟提取，保留 OpenAI-compatible JSON 响应格式。

## 下一阶段：Tauri 原生能力

当前机器还没有 Rust/Cargo，安装 Rust 后接入 Tauri：

```bash
rustup-init
npm install @tauri-apps/cli @tauri-apps/api
npx tauri init
```

需要实现的 Tauri 命令：

- `select_file()`: 打开系统文件选择器，返回文件路径。
- `select_directory()`: 打开系统目录选择器，返回目录路径。
- `copy_export_items(items, target_dir)`: 将导出清单中的源文件复制到目标目录，并使用用户指定的新文件名。
- `open_in_finder(path)`: 在 Finder 中定位导出目录或源文件。

当前已接入：

- Tauri dialog 插件提供系统文件和目录选择器。
- Rust 命令 `copy_export_items` 执行真实文件复制，并返回逐项成功/失败结果。

## 数据存储迁移

当前已接入 Tauri 后端 SQLite。`src/services/storage.js` 会在 Tauri 环境优先调用 SQLite，在浏览器调试环境保留 `localStorage` fallback。

首次进入 Tauri 且 SQLite 为空时，会尝试把旧 `localStorage` 里的材料库自动迁移到 SQLite。

建议 SQLite 表：

- `materials`: 材料基础信息。
- `material_files`: 材料绑定的文件版本。
- `export_jobs`: 每次导出记录，待实现。
- `export_items`: 每次导出的具体文件，待实现。
- `settings`: LLM 配置和应用设置，待迁移。

## 导出体验

当前已实现：

- Tauri 后端真实复制文件到目标目录。
- 逐项返回复制成功或失败结果。
- 导出完成后以表格展示结果。
- 支持从结果弹窗打开目标目录。

## LLM 接入

LLM 使用 OpenAI-compatible Chat Completions 或 Responses API 风格。

输入：

- 招聘公告材料要求文本。
- 当前材料库名称、别名、默认文件。

输出：

```json
{
  "items": [
    {
      "required_material": "身份证",
      "matched_material_id": "mat-id-card",
      "matched_material_name": "身份证",
      "suggested_filename": "身份证.pdf",
      "confidence": 0.95,
      "reason": "根据材料名匹配"
    }
  ]
}
```

安全规则：

- LLM 只能返回材料名、匹配 ID 和建议文件名。
- 不允许 LLM 返回或决定真实本地文件路径。
- 真实源文件始终来自本地材料库。
