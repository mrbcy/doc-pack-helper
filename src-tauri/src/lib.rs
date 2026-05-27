use rusqlite::{params, Connection};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::Manager;
use tokio::time::sleep;

#[derive(Default)]
struct CancelableRequests {
    inner: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct MaterialFile {
    id: String,
    format: String,
    file_path: String,
    original_filename: String,
    description: String,
    is_default: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Material {
    id: String,
    name: String,
    category: String,
    preferred_output_name: String,
    aliases: Vec<String>,
    tags: Vec<String>,
    note: String,
    created_at: String,
    updated_at: String,
    files: Vec<MaterialFile>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MaterialInput {
    id: Option<String>,
    name: String,
    category: String,
    preferred_output_name: String,
    aliases: Vec<String>,
    tags: Vec<String>,
    note: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MaterialFileInput {
    format: String,
    file_path: String,
    original_filename: String,
    description: String,
    is_default: bool,
}

#[derive(Debug, Deserialize)]
struct ExportItem {
    material_name: String,
    source_path: String,
    output_filename: String,
}

#[derive(Debug, Serialize)]
struct ExportResult {
    material_name: String,
    source_path: String,
    output_path: String,
    success: bool,
    message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct LlmSettings {
    enabled: bool,
    base_url: String,
    api_key: String,
    model: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct LlmMaterialCatalogItem {
    id: String,
    name: String,
    category: String,
    aliases: Vec<String>,
    default_filename: String,
    available_formats: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct LlmExtractItem {
    #[serde(default, deserialize_with = "deserialize_string_or_default")]
    required_material: String,
    #[serde(default, deserialize_with = "deserialize_string_or_default")]
    matched_material_id: String,
    #[serde(default, deserialize_with = "deserialize_string_or_default")]
    matched_material_name: String,
    #[serde(default, deserialize_with = "deserialize_string_or_default")]
    suggested_filename: String,
    #[serde(default, deserialize_with = "deserialize_f64_or_default")]
    confidence: f64,
    #[serde(default, deserialize_with = "deserialize_string_or_default")]
    status: String,
    #[serde(default, deserialize_with = "deserialize_string_or_default")]
    reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LlmExtractResponse {
    items: Vec<LlmExtractItem>,
}

fn deserialize_string_or_default<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    Ok(match value {
        Some(Value::String(text)) => text.trim().to_string(),
        Some(Value::Number(number)) => number.to_string(),
        Some(Value::Bool(flag)) => flag.to_string(),
        _ => String::new(),
    })
}

fn deserialize_f64_or_default<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    Ok(match value {
        Some(Value::Number(number)) => number.as_f64().unwrap_or(0.0),
        Some(Value::String(text)) => text.trim().parse::<f64>().unwrap_or(0.0),
        _ => 0.0,
    })
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ImportScanFile {
    id: String,
    path: String,
    relative_path: String,
    filename: String,
    extension: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ImportPromptFileRef {
    id: String,
    relative_path: String,
    filename: String,
    extension: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct LlmImportSuggestion {
    material_name: String,
    category: String,
    preferred_output_name: String,
    #[serde(default)]
    reason: String,
    #[serde(default)]
    file_ids: Vec<String>,
    #[serde(default)]
    file_paths: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LlmImportResponse {
    items: Vec<LlmImportSuggestion>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LlmImportFinalizeResponse {
    items: Vec<LlmImportSuggestion>,
}

#[tauri::command]
fn list_materials(app: tauri::AppHandle) -> Result<Vec<Material>, String> {
    let conn = open_db(&app)?;
    load_materials(&conn)
}

#[tauri::command]
fn save_material(app: tauri::AppHandle, material: MaterialInput) -> Result<Vec<Material>, String> {
    let conn = open_db(&app)?;
    let now = now_text();
    let aliases = serde_json::to_string(&material.aliases).map_err(|error| error.to_string())?;
    let tags = serde_json::to_string(&material.tags).map_err(|error| error.to_string())?;

    match material.id {
        Some(id) if !id.is_empty() => {
            let changed = conn.execute(
                "UPDATE materials SET name = ?1, category = ?2, preferred_output_name = ?3, aliases = ?4, tags = ?5, note = ?6, updated_at = ?7 WHERE id = ?8",
                params![material.name, material.category, material.preferred_output_name, aliases, tags, material.note, now, id],
            )
            .map_err(|error| error.to_string())?;
            if changed == 0 {
                conn.execute(
                    "INSERT INTO materials (id, name, category, preferred_output_name, aliases, tags, note, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    params![id, material.name, material.category, material.preferred_output_name, aliases, tags, material.note, now, now],
                )
                .map_err(|error| error.to_string())?;
            }
        }
        _ => {
            let id = make_id("mat");
            conn.execute(
                "INSERT INTO materials (id, name, category, preferred_output_name, aliases, tags, note, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![id, material.name, material.category, material.preferred_output_name, aliases, tags, material.note, now, now],
            )
            .map_err(|error| error.to_string())?;
        }
    }

    load_materials(&conn)
}

#[tauri::command]
fn delete_material(app: tauri::AppHandle, material_id: String) -> Result<Vec<Material>, String> {
    let conn = open_db(&app)?;
    conn.execute("DELETE FROM material_files WHERE material_id = ?1", params![material_id])
        .map_err(|error| error.to_string())?;
    conn.execute("DELETE FROM materials WHERE id = ?1", params![material_id])
        .map_err(|error| error.to_string())?;
    load_materials(&conn)
}

#[tauri::command]
fn add_material_file(
    app: tauri::AppHandle,
    material_id: String,
    file: MaterialFileInput,
) -> Result<Vec<Material>, String> {
    let conn = open_db(&app)?;
    let existing_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM material_files WHERE material_id = ?1",
            params![material_id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    let should_be_default = existing_count == 0 || file.is_default;

    if should_be_default {
        conn.execute(
            "UPDATE material_files SET is_default = 0 WHERE material_id = ?1",
            params![material_id],
        )
        .map_err(|error| error.to_string())?;
    }

    conn.execute(
        "INSERT INTO material_files (id, material_id, format, file_path, original_filename, description, is_default, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            make_id("file"),
            material_id,
            file.format,
            file.file_path,
            file.original_filename,
            file.description,
            if should_be_default { 1 } else { 0 },
            now_text()
        ],
    )
    .map_err(|error| error.to_string())?;
    touch_material(&conn, &material_id)?;
    load_materials(&conn)
}

#[tauri::command]
fn delete_material_file(
    app: tauri::AppHandle,
    material_id: String,
    file_id: String,
) -> Result<Vec<Material>, String> {
    let conn = open_db(&app)?;
    let was_default: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM material_files WHERE id = ?1 AND is_default = 1",
            params![file_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    conn.execute("DELETE FROM material_files WHERE id = ?1", params![file_id])
        .map_err(|error| error.to_string())?;

    if was_default > 0 {
        let next_file_id: Option<String> = conn
            .query_row(
                "SELECT id FROM material_files WHERE material_id = ?1 ORDER BY created_at ASC LIMIT 1",
                params![material_id],
                |row| row.get(0),
            )
            .ok();
        if let Some(next_id) = next_file_id {
            conn.execute(
                "UPDATE material_files SET is_default = CASE WHEN id = ?1 THEN 1 ELSE 0 END WHERE material_id = ?2",
                params![next_id, material_id],
            )
            .map_err(|error| error.to_string())?;
        }
    }

    touch_material(&conn, &material_id)?;
    load_materials(&conn)
}

#[tauri::command]
fn set_default_material_file(
    app: tauri::AppHandle,
    material_id: String,
    file_id: String,
) -> Result<Vec<Material>, String> {
    let conn = open_db(&app)?;
    conn.execute(
        "UPDATE material_files SET is_default = CASE WHEN id = ?1 THEN 1 ELSE 0 END WHERE material_id = ?2",
        params![file_id, material_id],
    )
    .map_err(|error| error.to_string())?;
    touch_material(&conn, &material_id)?;
    load_materials(&conn)
}

#[tauri::command]
fn copy_export_items(items: Vec<ExportItem>, target_dir: String) -> Result<Vec<ExportResult>, String> {
    let target = PathBuf::from(target_dir);
    if !target.exists() {
        return Err("目标目录不存在".to_string());
    }
    if !target.is_dir() {
        return Err("目标路径不是目录".to_string());
    }

    let mut results = Vec::new();

    for item in items {
        let result = copy_one_item(&item, &target);
        results.push(result);
    }

    Ok(results)
}

#[tauri::command]
fn open_path(path: String) -> Result<(), String> {
    if path.trim().is_empty() {
        return Err("路径为空".to_string());
    }

    #[cfg(target_os = "macos")]
    let mut command = {
        let mut cmd = Command::new("open");
        cmd.arg(path);
        cmd
    };

    #[cfg(target_os = "windows")]
    let mut command = {
        let mut cmd = Command::new("explorer");
        cmd.arg(path);
        cmd
    };

    #[cfg(target_os = "linux")]
    let mut command = {
        let mut cmd = Command::new("xdg-open");
        cmd.arg(path);
        cmd
    };

    command
        .status()
        .map_err(|error| format!("打开路径失败：{}", error))?;
    Ok(())
}

#[tauri::command]
fn load_llm_settings(app: tauri::AppHandle) -> Result<LlmSettings, String> {
    let conn = open_db(&app)?;
    load_llm_settings_from_db(&conn)
}

#[tauri::command]
fn save_llm_settings(app: tauri::AppHandle, settings: LlmSettings) -> Result<LlmSettings, String> {
    let conn = open_db(&app)?;
    save_llm_settings_to_db(&conn, &settings)?;
    load_llm_settings_from_db(&conn)
}

#[tauri::command]
async fn test_llm_connection(
    app: tauri::AppHandle,
    request_id: String,
    requests: tauri::State<'_, CancelableRequests>,
) -> Result<String, String> {
    let conn = open_db(&app)?;
    let settings = load_llm_settings_from_db(&conn)?;
    validate_llm_settings(&settings)?;
    let cancel_flag = register_request(&requests, &request_id)?;
    let result = send_llm_request(
        &settings,
        "你是一个接口连通性测试助手。",
        "只回复 OK",
        cancel_flag,
    )
    .await;
    cleanup_request(&requests, &request_id);
    result
}

#[tauri::command]
async fn extract_materials_with_llm(
    app: tauri::AppHandle,
    request_id: String,
    requirement_text: String,
    materials: Vec<LlmMaterialCatalogItem>,
    requests: tauri::State<'_, CancelableRequests>,
) -> Result<Vec<LlmExtractItem>, String> {
    if requirement_text.trim().is_empty() {
        return Err("材料要求不能为空".to_string());
    }

    let conn = open_db(&app)?;
    let settings = load_llm_settings_from_db(&conn)?;
    validate_llm_settings(&settings)?;
    let cancel_flag = register_request(&requests, &request_id)?;

    let system_prompt = "你是招聘材料整理助手。你的任务是从招聘公告里提取需要提交的材料，并匹配到用户已有材料库。必须只返回 JSON，不要返回 Markdown、解释文字或代码块。";
    let user_prompt = format!(
        "请根据下面的招聘材料要求，返回 JSON 对象，格式固定为 {{\"items\":[...]}}。\n\
items 中每一项都包含以下字段：\n\
- requiredMaterial: 招聘要求里的材料名\n\
- matchedMaterialId: 如果能从材料库中匹配到就填材料 id，否则填空字符串\n\
- matchedMaterialName: 匹配到的材料名，没有就填空字符串\n\
- suggestedFilename: 建议导出文件名，必须带扩展名\n\
- confidence: 0 到 1 之间的小数\n\
- status: matched 或 missing\n\
- reason: 简短原因\n\
要求：\n\
1. 优先匹配材料名、别名、分类语义相近的材料。\n\
2. 如果招聘要求里是“学信网认证/学历备案表/学籍在线验证报告”等，要尽量匹配相关材料。\n\
3. suggestedFilename 尽量使用招聘要求中的自然名称，并参考默认文件扩展名。\n\
4. 不要虚构材料库中不存在的 id。\n\
\n\
招聘要求：\n{}\n\
\n\
材料库：\n{}",
        requirement_text.trim(),
        serde_json::to_string(&materials).map_err(|error| error.to_string())?
    );

    let result = async {
        let raw_content = send_llm_request(&settings, system_prompt, &user_prompt, cancel_flag).await?;
        let parsed = parse_llm_extract_response(&raw_content)?;
        Ok(normalize_llm_items(parsed.items, &materials))
    }
    .await;
    cleanup_request(&requests, &request_id);
    result
}

#[tauri::command]
fn scan_import_directory(root_dir: String) -> Result<Vec<ImportScanFile>, String> {
    let root = PathBuf::from(root_dir.trim());
    if !root.exists() {
        return Err("所选文件夹不存在".to_string());
    }
    if !root.is_dir() {
        return Err("所选路径不是文件夹".to_string());
    }

    let mut files = Vec::new();
    collect_import_files(&root, &root, &mut files)?;
    files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    Ok(files)
}

#[tauri::command]
async fn suggest_material_imports_with_llm(
    app: tauri::AppHandle,
    request_id: String,
    root_dir: String,
    files: Vec<ImportScanFile>,
    requests: tauri::State<'_, CancelableRequests>,
) -> Result<Vec<LlmImportSuggestion>, String> {
    if root_dir.trim().is_empty() {
        return Err("导入目录不能为空".to_string());
    }
    if files.is_empty() {
        return Err("当前目录下没有可导入文件".to_string());
    }

    let conn = open_db(&app)?;
    let settings = load_llm_settings_from_db(&conn)?;
    validate_llm_settings(&settings)?;
    let cancel_flag = register_request(&requests, &request_id)?;

    let category_hints = load_material_categories(&conn)?;
    let prompt_files = build_import_prompt_files(&files);
    let system_prompt = "你是材料整理助手。请根据文件名、相对路径、目录结构，将文件智能归类为可导入的材料。必须只返回 JSON，不要返回 Markdown、解释文字或代码块。";
    let user_prompt = format!(
        "请分析下面目录中的文件，并输出 JSON 对象，格式固定为 {{\"items\":[...]}}。\n\
每个 item 表示一个待导入材料，字段必须包含：\n\
- materialName: 材料名称，简短自然，例如“身份证”“毕业证”“教育部学籍在线验证报告”\n\
- category: 分类名称，优先从已有分类中选择，实在不合适再生成新分类\n\
- preferredOutputName: 导出默认文件名的基础名称，不带扩展名，例如“身份证”“个人简历”\n\
- reason: 简短归类依据\n\
- fileIds: 属于该材料的文件 id 数组，可以包含一个或多个版本文件\n\
要求：\n\
1. 同一种材料的多个版本可以放到同一个 item 中。\n\
2. 不要遗漏文件，也不要虚构不存在的文件 id。\n\
3. 忽略系统文件、缓存文件和明显无关的临时文件。\n\
4. 优先按国企报名材料语境命名和分类。\n\
5. 仅返回 JSON。\n\
\n\
已有分类：\n{}\n\
\n\
扫描目录：\n{}\n\
\n\
文件列表：\n{}",
        serde_json::to_string(&category_hints).map_err(|error| error.to_string())?,
        root_dir.trim(),
        serde_json::to_string(&prompt_files).map_err(|error| error.to_string())?
    );

    let result = async {
        let raw_content = send_llm_request(&settings, system_prompt, &user_prompt, cancel_flag).await?;
        let parsed = parse_llm_import_response(&raw_content)?;
        Ok(normalize_import_suggestions(parsed.items, &files, &category_hints))
    }
    .await;
    cleanup_request(&requests, &request_id);
    result
}

#[tauri::command]
async fn finalize_material_import_suggestions_with_llm(
    app: tauri::AppHandle,
    request_id: String,
    files: Vec<ImportScanFile>,
    suggestions: Vec<LlmImportSuggestion>,
    requests: tauri::State<'_, CancelableRequests>,
) -> Result<Vec<LlmImportSuggestion>, String> {
    if files.is_empty() || suggestions.is_empty() {
        return Ok(suggestions);
    }

    let conn = open_db(&app)?;
    let settings = load_llm_settings_from_db(&conn)?;
    validate_llm_settings(&settings)?;
    let cancel_flag = register_request(&requests, &request_id)?;

    let category_hints = load_material_categories(&conn)?;
    let referenced_file_ids: HashSet<&str> = suggestions
        .iter()
        .flat_map(|item| item.file_ids.iter().map(String::as_str))
        .collect();
    let file_refs: Vec<Value> = files
        .iter()
        .filter(|file| referenced_file_ids.contains(file.id.as_str()))
        .map(|file| {
            json!({
                "id": file.id,
                "relativePath": file.relative_path,
                "filename": file.filename,
                "extension": file.extension
            })
        })
        .collect();
    let suggestion_refs: Vec<Value> = suggestions
        .iter()
        .map(|item| {
            json!({
                "materialName": item.material_name,
                "category": item.category,
                "preferredOutputName": item.preferred_output_name,
                "reason": item.reason,
                "fileIds": item.file_ids
            })
        })
        .collect();

    let system_prompt = "你是材料整理助手。你会对多批次归类结果做第二轮全局整理，统一名称与分类，但不能丢失文件，也不能虚构文件 id。必须只返回 JSON。";
    let user_prompt = format!(
        "请对下面的批量归类结果做一次全局整理，输出 JSON 对象 {{\"items\":[...]}}。\n\
每个 item 字段必须包含：materialName、category、preferredOutputName、reason、fileIds。\n\
要求：\n\
1. 可以把不同批次里其实属于同一材料的项合并。\n\
2. 不要把明显不同的材料错误合并。\n\
3. 统一命名风格，优先使用国企报名常见材料名称。\n\
4. 优先使用已有分类；实在不合适再新增。\n\
5. fileIds 必须来自给定文件列表，不能遗漏也不能虚构。\n\
6. 只返回 JSON。\n\
\n\
已有分类：\n{}\n\
\n\
文件索引：\n{}\n\
\n\
第一轮归类结果：\n{}",
        serde_json::to_string(&category_hints).map_err(|error| error.to_string())?,
        serde_json::to_string(&file_refs).map_err(|error| error.to_string())?,
        serde_json::to_string(&suggestion_refs).map_err(|error| error.to_string())?
    );

    let result = async {
        let raw_content = send_llm_request(&settings, system_prompt, &user_prompt, cancel_flag).await?;
        let parsed = parse_llm_import_finalize_response(&raw_content)?;
        Ok(normalize_import_suggestions(parsed.items, &files, &category_hints))
    }
    .await;
    cleanup_request(&requests, &request_id);
    result
}

#[tauri::command]
fn cancel_llm_request(
    request_id: String,
    requests: tauri::State<'_, CancelableRequests>,
) -> Result<(), String> {
    let inner = requests.inner.lock().map_err(|_| "取消请求失败".to_string())?;
    if let Some(flag) = inner.get(&request_id) {
        flag.store(true, Ordering::SeqCst);
    }
    Ok(())
}

fn copy_one_item(item: &ExportItem, target_dir: &Path) -> ExportResult {
    let output_path = target_dir.join(&item.output_filename);
    let output_path_text = output_path.to_string_lossy().to_string();

    if item.source_path.trim().is_empty() {
        return failed_result(item, output_path_text, "源文件路径为空");
    }

    if item.output_filename.trim().is_empty() {
        return failed_result(item, output_path_text, "导出文件名为空");
    }

    if item.output_filename.contains('/') || item.output_filename.contains('\\') {
        return failed_result(item, output_path_text, "导出文件名不能包含路径分隔符");
    }

    let source = PathBuf::from(&item.source_path);
    if !source.exists() {
        return failed_result(item, output_path_text, "源文件不存在");
    }
    if !source.is_file() {
        return failed_result(item, output_path_text, "源路径不是文件");
    }

    match fs::copy(&source, &output_path) {
        Ok(_) => ExportResult {
            material_name: item.material_name.clone(),
            source_path: item.source_path.clone(),
            output_path: output_path_text,
            success: true,
            message: "复制成功".to_string(),
        },
        Err(error) => failed_result(item, output_path_text, &format!("复制失败：{}", error)),
    }
}

fn failed_result(item: &ExportItem, output_path: String, message: &str) -> ExportResult {
    ExportResult {
        material_name: item.material_name.clone(),
        source_path: item.source_path.clone(),
        output_path,
        success: false,
        message: message.to_string(),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(CancelableRequests::default())
        .invoke_handler(tauri::generate_handler![
            copy_export_items,
            open_path,
            list_materials,
            save_material,
            delete_material,
            add_material_file,
            delete_material_file,
            set_default_material_file,
            load_llm_settings,
            save_llm_settings,
            test_llm_connection,
            extract_materials_with_llm,
            cancel_llm_request,
            scan_import_directory,
            suggest_material_imports_with_llm,
            finalize_material_import_suggestions_with_llm
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn open_db(app: &tauri::AppHandle) -> Result<Connection, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("无法获取应用数据目录：{}", error))?;
    fs::create_dir_all(&app_dir).map_err(|error| format!("无法创建应用数据目录：{}", error))?;
    let db_path = app_dir.join("materials.db");
    let conn = Connection::open(db_path).map_err(|error| error.to_string())?;
    init_db(&conn)?;
    Ok(conn)
}

fn init_db(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS materials (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            category TEXT NOT NULL DEFAULT '',
            preferred_output_name TEXT NOT NULL DEFAULT '',
            aliases TEXT NOT NULL DEFAULT '[]',
            tags TEXT NOT NULL DEFAULT '[]',
            note TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS material_files (
            id TEXT PRIMARY KEY,
            material_id TEXT NOT NULL,
            format TEXT NOT NULL DEFAULT '',
            file_path TEXT NOT NULL,
            original_filename TEXT NOT NULL DEFAULT '',
            description TEXT NOT NULL DEFAULT '',
            is_default INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            FOREIGN KEY(material_id) REFERENCES materials(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        ",
    )
    .map_err(|error| error.to_string())?;

    ensure_column(
        conn,
        "materials",
        "preferred_output_name",
        "ALTER TABLE materials ADD COLUMN preferred_output_name TEXT NOT NULL DEFAULT ''",
    )
}

fn load_materials(conn: &Connection) -> Result<Vec<Material>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, category, preferred_output_name, aliases, tags, note, created_at, updated_at
             FROM materials ORDER BY updated_at DESC",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            let id: String = row.get(0)?;
            let aliases_text: String = row.get(4)?;
            let tags_text: String = row.get(5)?;
            Ok(Material {
                files: load_files(conn, &id).unwrap_or_default(),
                id,
                name: row.get(1)?,
                category: row.get(2)?,
                preferred_output_name: row.get(3)?,
                aliases: serde_json::from_str(&aliases_text).unwrap_or_default(),
                tags: serde_json::from_str(&tags_text).unwrap_or_default(),
                note: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

fn load_files(conn: &Connection, material_id: &str) -> Result<Vec<MaterialFile>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, format, file_path, original_filename, description, is_default
             FROM material_files WHERE material_id = ?1 ORDER BY is_default DESC, created_at ASC",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map(params![material_id], |row| {
            let is_default: i64 = row.get(5)?;
            Ok(MaterialFile {
                id: row.get(0)?,
                format: row.get(1)?,
                file_path: row.get(2)?,
                original_filename: row.get(3)?,
                description: row.get(4)?,
                is_default: is_default == 1,
            })
        })
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

fn load_material_categories(conn: &Connection) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare("SELECT DISTINCT category FROM materials WHERE TRIM(category) <> '' ORDER BY category")
        .map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map([], |row| row.get(0))
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<String>, _>>()
        .map_err(|error| error.to_string())
}

fn touch_material(conn: &Connection, material_id: &str) -> Result<(), String> {
    conn.execute(
        "UPDATE materials SET updated_at = ?1 WHERE id = ?2",
        params![now_text(), material_id],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

fn now_text() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    millis.to_string()
}

fn make_id(prefix: &str) -> String {
    format!("{}-{}", prefix, now_text())
}

fn ensure_column(
    conn: &Connection,
    table_name: &str,
    column_name: &str,
    alter_sql: &str,
) -> Result<(), String> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({})", table_name))
        .map_err(|error| error.to_string())?;
    let mut rows = stmt.query([]).map_err(|error| error.to_string())?;

    while let Some(row) = rows.next().map_err(|error| error.to_string())? {
        let current: String = row.get(1).map_err(|error| error.to_string())?;
        if current == column_name {
            return Ok(());
        }
    }

    conn.execute(alter_sql, []).map_err(|error| error.to_string())?;
    Ok(())
}

fn default_llm_settings() -> LlmSettings {
    LlmSettings {
        enabled: false,
        base_url: "https://api.openai.com/v1".to_string(),
        api_key: String::new(),
        model: "gpt-4.1-mini".to_string(),
    }
}

fn load_llm_settings_from_db(conn: &Connection) -> Result<LlmSettings, String> {
    let stored: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'llm_settings' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .ok();

    match stored {
        Some(value) => serde_json::from_str(&value).map_err(|error| error.to_string()),
        None => Ok(default_llm_settings()),
    }
}

fn save_llm_settings_to_db(conn: &Connection, settings: &LlmSettings) -> Result<(), String> {
    let value = serde_json::to_string(settings).map_err(|error| error.to_string())?;
    conn.execute(
        "INSERT INTO settings (key, value, updated_at)
         VALUES ('llm_settings', ?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        params![value, now_text()],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

fn validate_llm_settings(settings: &LlmSettings) -> Result<(), String> {
    if !settings.enabled {
        return Err("LLM 尚未开启，请先在设置中启用".to_string());
    }
    if settings.base_url.trim().is_empty() {
        return Err("Base URL 不能为空".to_string());
    }
    if settings.api_key.trim().is_empty() {
        return Err("API Key 不能为空".to_string());
    }
    if settings.model.trim().is_empty() {
        return Err("模型名称不能为空".to_string());
    }
    Ok(())
}

async fn send_llm_request(
    settings: &LlmSettings,
    system_prompt: &str,
    user_prompt: &str,
    cancel_flag: Arc<AtomicBool>,
) -> Result<String, String> {
    let url = format!("{}/chat/completions", settings.base_url.trim_end_matches('/'));
    let body = json!({
        "model": settings.model,
        "temperature": 0.2,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": user_prompt }
        ]
    });

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|error| format!("创建 LLM 客户端失败：{}", error))?;

    let response = tokio::select! {
        response = client
            .post(url)
            .bearer_auth(settings.api_key.trim())
            .header("Content-Type", "application/json")
            .json(&body)
            .send() => response.map_err(|error| format!("请求 LLM 失败：{}", error))?,
        _ = wait_for_cancellation(cancel_flag.clone()) => return Err("操作已取消".to_string()),
    };

    let status = response.status();
    let response_text = tokio::select! {
        text = response.text() => text.map_err(|error| format!("读取 LLM 响应失败：{}", error))?,
        _ = wait_for_cancellation(cancel_flag) => return Err("操作已取消".to_string()),
    };

    if !status.is_success() {
        return Err(format!("LLM 请求失败（{}）：{}", status.as_u16(), response_text));
    }

    let value: Value =
        serde_json::from_str(&response_text).map_err(|error| format!("解析 LLM 响应失败：{}", error))?;
    extract_message_content(&value).ok_or_else(|| format!("未能从响应中提取内容：{}", response_text))
}

async fn wait_for_cancellation(cancel_flag: Arc<AtomicBool>) {
    loop {
        if cancel_flag.load(Ordering::SeqCst) {
            break;
        }
        sleep(Duration::from_millis(120)).await;
    }
}

fn register_request(
    requests: &tauri::State<'_, CancelableRequests>,
    request_id: &str,
) -> Result<Arc<AtomicBool>, String> {
    if request_id.trim().is_empty() {
        return Err("请求标识不能为空".to_string());
    }
    let mut inner = requests.inner.lock().map_err(|_| "创建请求失败".to_string())?;
    let flag = Arc::new(AtomicBool::new(false));
    inner.insert(request_id.to_string(), flag.clone());
    Ok(flag)
}

fn cleanup_request(requests: &tauri::State<'_, CancelableRequests>, request_id: &str) {
    if let Ok(mut inner) = requests.inner.lock() {
        inner.remove(request_id);
    }
}

fn extract_message_content(value: &Value) -> Option<String> {
    let content = value
        .get("choices")?
        .get(0)?
        .get("message")?
        .get("content")?;

    match content {
        Value::String(text) => Some(text.clone()),
        Value::Array(parts) => {
            let mut combined = String::new();
            for part in parts {
                if let Some(text) = part.get("text").and_then(Value::as_str) {
                    combined.push_str(text);
                }
            }
            if combined.trim().is_empty() {
                None
            } else {
                Some(combined)
            }
        }
        _ => None,
    }
}

fn parse_llm_extract_response(content: &str) -> Result<LlmExtractResponse, String> {
    let cleaned = strip_code_fence(content);
    if let Ok(parsed) = serde_json::from_str::<LlmExtractResponse>(&cleaned) {
        return Ok(parsed);
    }

    let candidate = extract_json_block(&cleaned).ok_or_else(|| "LLM 没有返回合法 JSON".to_string())?;
    serde_json::from_str::<LlmExtractResponse>(&candidate)
        .map_err(|error| format!("解析识别结果失败：{}", error))
}

fn strip_code_fence(content: &str) -> String {
    let trimmed = content.trim();
    if trimmed.starts_with("```") {
        let lines: Vec<&str> = trimmed.lines().collect();
        if lines.len() >= 3 {
            return lines[1..lines.len() - 1].join("\n");
        }
    }
    trimmed.to_string()
}

fn extract_json_block(content: &str) -> Option<String> {
    let object_start = content.find('{');
    let object_end = content.rfind('}');
    if let (Some(start), Some(end)) = (object_start, object_end) {
        if start < end {
            return Some(content[start..=end].to_string());
        }
    }
    None
}

fn parse_llm_import_response(content: &str) -> Result<LlmImportResponse, String> {
    let cleaned = strip_code_fence(content);
    if let Ok(parsed) = serde_json::from_str::<LlmImportResponse>(&cleaned) {
        return Ok(parsed);
    }

    let candidate = extract_json_block(&cleaned).ok_or_else(|| "LLM 没有返回合法 JSON".to_string())?;
    serde_json::from_str::<LlmImportResponse>(&candidate)
        .map_err(|error| format!("解析导入建议失败：{}", error))
}

fn parse_llm_import_finalize_response(content: &str) -> Result<LlmImportFinalizeResponse, String> {
    let cleaned = strip_code_fence(content);
    if let Ok(parsed) = serde_json::from_str::<LlmImportFinalizeResponse>(&cleaned) {
        return Ok(parsed);
    }

    let candidate = extract_json_block(&cleaned).ok_or_else(|| "LLM 没有返回合法 JSON".to_string())?;
    serde_json::from_str::<LlmImportFinalizeResponse>(&candidate)
        .map_err(|error| format!("解析导入整理结果失败：{}", error))
}

fn normalize_llm_items(
    items: Vec<LlmExtractItem>,
    materials: &[LlmMaterialCatalogItem],
) -> Vec<LlmExtractItem> {
    items.into_iter()
        .filter_map(|item| normalize_llm_item(item, materials))
        .collect()
}

fn normalize_llm_item(
    mut item: LlmExtractItem,
    materials: &[LlmMaterialCatalogItem],
) -> Option<LlmExtractItem> {
    let required = item.required_material.trim().to_string();
    if required.is_empty() {
        return None;
    }

    let matched_material = if !item.matched_material_id.trim().is_empty() {
        materials
            .iter()
            .find(|material| material.id == item.matched_material_id)
            .cloned()
    } else {
        find_catalog_match(&required, materials)
    };

    if let Some(material) = matched_material {
        item.matched_material_id = material.id.clone();
        item.matched_material_name = material.name.clone();
        item.status = "matched".to_string();
        if item.suggested_filename.trim().is_empty() {
            item.suggested_filename = build_suggested_filename(&required, &material.default_filename);
        }
        if item.reason.trim().is_empty() {
            item.reason = "已匹配材料库".to_string();
        }
        if !(0.0..=1.0).contains(&item.confidence) {
            item.confidence = 0.9;
        }
    } else {
        item.matched_material_id = String::new();
        item.matched_material_name = String::new();
        item.status = "missing".to_string();
        if item.suggested_filename.trim().is_empty() {
            item.suggested_filename = build_suggested_filename(&required, "");
        }
        if item.reason.trim().is_empty() {
            item.reason = "材料库中未找到匹配项".to_string();
        }
        if !(0.0..=1.0).contains(&item.confidence) {
            item.confidence = 0.3;
        }
    }

    item.required_material = required;
    Some(item)
}

fn find_catalog_match(required_name: &str, materials: &[LlmMaterialCatalogItem]) -> Option<LlmMaterialCatalogItem> {
    let required = required_name.to_lowercase();
    materials.iter().find_map(|material| {
        let candidates = std::iter::once(&material.name)
            .chain(material.aliases.iter())
            .chain(std::iter::once(&material.category));

        let matched = candidates
            .filter(|text| !text.trim().is_empty())
            .any(|text| {
                let normalized = text.to_lowercase();
                required.contains(&normalized) || normalized.contains(&required)
            });

        if matched {
            Some(material.clone())
        } else {
            None
        }
    })
}

fn build_suggested_filename(required_material: &str, default_filename: &str) -> String {
    let ext = default_filename
        .rsplit('.')
        .next()
        .filter(|value| value != &default_filename && !value.trim().is_empty())
        .unwrap_or("pdf");
    format!("{}.{}", required_material, ext)
}

fn collect_import_files(root: &Path, current: &Path, output: &mut Vec<ImportScanFile>) -> Result<(), String> {
    let entries = fs::read_dir(current).map_err(|error| format!("读取目录失败：{}", error))?;
    for entry in entries {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();

        if should_skip_path(&file_name) {
            continue;
        }

        if path.is_dir() {
            collect_import_files(root, &path, output)?;
            continue;
        }

        if !path.is_file() {
            continue;
        }

        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("")
            .to_lowercase();
        if should_skip_extension(&extension) {
            continue;
        }

        let relative_path = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");

        output.push(ImportScanFile {
            id: format!("file-{}", output.len() + 1),
            path: path.to_string_lossy().to_string(),
            relative_path,
            filename: file_name,
            extension,
        });
    }

    Ok(())
}

fn should_skip_path(file_name: &str) -> bool {
    let lower = file_name.to_lowercase();
    lower == ".ds_store" || lower == "thumbs.db" || lower.starts_with("~$")
}

fn should_skip_extension(ext: &str) -> bool {
    matches!(ext, "db" | "tmp" | "temp")
}

fn build_import_prompt_files(files: &[ImportScanFile]) -> Vec<ImportPromptFileRef> {
    files
        .iter()
        .map(|file| ImportPromptFileRef {
            id: file.id.clone(),
            relative_path: file.relative_path.clone(),
            filename: file.filename.clone(),
            extension: file.extension.clone(),
        })
        .collect()
}

fn normalize_import_suggestions(
    items: Vec<LlmImportSuggestion>,
    files: &[ImportScanFile],
    category_hints: &[String],
) -> Vec<LlmImportSuggestion> {
    let file_map: HashMap<&str, &ImportScanFile> = files.iter().map(|file| (file.path.as_str(), file)).collect();
    let id_map: HashMap<&str, &ImportScanFile> = files.iter().map(|file| (file.id.as_str(), file)).collect();
    let mut normalized = Vec::new();

    for item in items {
        let material_name = item.material_name.trim().to_string();
        if material_name.is_empty() {
            continue;
        }

        let mut file_ids = Vec::new();
        let mut file_paths = Vec::new();
        for file_id in item.file_ids {
            if let Some(file) = id_map.get(file_id.as_str()) {
                if !file_ids.iter().any(|current| current == &file.id) {
                    file_ids.push(file.id.clone());
                }
                if !file_paths.iter().any(|current| current == &file.path) {
                    file_paths.push(file.path.clone());
                }
            }
        }
        for path in item.file_paths {
            if file_map.contains_key(path.as_str()) && !file_paths.iter().any(|current| current == &path) {
                file_paths.push(path);
            }
        }
        if file_paths.is_empty() {
            continue;
        }

        let mut category = item.category.trim().to_string();
        if category.is_empty() {
            category = infer_category_from_path(&material_name, &file_paths, files)
                .unwrap_or_else(|| "其他".to_string());
        } else if let Some(matched) = category_hints.iter().find(|hint| hint == &&category) {
            category = matched.clone();
        }

        let preferred_output_name = if item.preferred_output_name.trim().is_empty() {
            material_name.clone()
        } else {
            item.preferred_output_name.trim().to_string()
        };

        normalized.push(LlmImportSuggestion {
            material_name,
            category,
            preferred_output_name,
            reason: item.reason.trim().to_string(),
            file_ids,
            file_paths,
        });
    }

    normalized
}

fn infer_category_from_path(
    material_name: &str,
    file_paths: &[String],
    files: &[ImportScanFile],
) -> Option<String> {
    let mut candidates = String::new();
    candidates.push_str(material_name);
    for path in file_paths {
        if let Some(file) = files.iter().find(|item| &item.path == path) {
            candidates.push(' ');
            candidates.push_str(&file.relative_path);
        }
    }
    let text = candidates.to_lowercase();
    if text.contains("简历") {
        return Some("简历".to_string());
    }
    if text.contains("学信") {
        return Some("学信网".to_string());
    }
    if text.contains("身份证") || text.contains("户口") {
        return Some("身份证明".to_string());
    }
    if text.contains("毕业证") || text.contains("学位证") || text.contains("成绩单") || text.contains("录取") || text.contains("学生证") {
        return Some("学历学位".to_string());
    }
    if text.contains("一寸照") || text.contains("证件照") || text.contains("形象照") {
        return Some("照片".to_string());
    }
    if text.contains("证明") || text.contains("征信") || text.contains("社保") || text.contains("离职") {
        return Some("证明材料".to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_llm_extract_response_accepts_null_string_fields() {
        let content = r#"{
            "items": [
                {
                    "requiredMaterial": "身份证",
                    "matchedMaterialId": null,
                    "matchedMaterialName": null,
                    "suggestedFilename": "身份证.pdf",
                    "confidence": 0,
                    "status": "missing",
                    "reason": "未匹配"
                }
            ]
        }"#;

        let parsed = parse_llm_extract_response(content).expect("response should parse");
        assert_eq!(parsed.items.len(), 1);
        assert_eq!(parsed.items[0].required_material, "身份证");
        assert_eq!(parsed.items[0].matched_material_id, "");
        assert_eq!(parsed.items[0].matched_material_name, "");
    }
}
