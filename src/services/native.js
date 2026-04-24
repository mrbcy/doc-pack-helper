import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

export function isTauriRuntime() {
  return Boolean(window.__TAURI_INTERNALS__);
}

export async function selectFilePath() {
  if (!isTauriRuntime()) {
    return '';
  }

  const selected = await open({
    multiple: false,
    directory: false,
  });

  return typeof selected === 'string' ? selected : '';
}

export async function selectDirectoryPath() {
  if (!isTauriRuntime()) {
    return '';
  }

  const selected = await open({
    multiple: false,
    directory: true,
  });

  return typeof selected === 'string' ? selected : '';
}

export async function copyExportItems(items, targetDir) {
  if (!isTauriRuntime()) {
    return null;
  }

  return invoke('copy_export_items', {
    items: items.map((item) => ({
      material_name: item.materialName,
      source_path: item.sourcePath,
      output_filename: item.outputFilename,
    })),
    targetDir,
  });
}

export async function openPath(path) {
  if (!isTauriRuntime()) {
    return false;
  }

  await invoke('open_path', { path });
  return true;
}

export async function listMaterialsFromDb() {
  return invoke('list_materials');
}

export async function saveMaterialToDb(material) {
  return invoke('save_material', { material });
}

export async function deleteMaterialFromDb(materialId) {
  return invoke('delete_material', { materialId });
}

export async function addMaterialFileToDb(materialId, file) {
  return invoke('add_material_file', { materialId, file });
}

export async function deleteMaterialFileFromDb(materialId, fileId) {
  return invoke('delete_material_file', { materialId, fileId });
}

export async function setDefaultMaterialFileInDb(materialId, fileId) {
  return invoke('set_default_material_file', { materialId, fileId });
}

export async function loadLlmSettingsFromNative() {
  return invoke('load_llm_settings');
}

export async function saveLlmSettingsToNative(settings) {
  return invoke('save_llm_settings', { settings });
}

export async function testLlmConnectionFromNative(requestId) {
  return invoke('test_llm_connection', { requestId });
}

export async function extractMaterialsWithLlmFromNative(requestId, requirementText, materials) {
  return invoke('extract_materials_with_llm', {
    requestId,
    requirementText,
    materials,
  });
}

export async function cancelLlmRequestFromNative(requestId) {
  return invoke('cancel_llm_request', { requestId });
}

export async function scanImportDirectoryFromNative(rootDir) {
  return invoke('scan_import_directory', { rootDir });
}

export async function suggestMaterialImportsWithLlmFromNative(requestId, rootDir, files) {
  return invoke('suggest_material_imports_with_llm', {
    requestId,
    rootDir,
    files,
  });
}

export async function finalizeMaterialImportSuggestionsWithLlmFromNative(requestId, files, suggestions) {
  return invoke('finalize_material_import_suggestions_with_llm', {
    requestId,
    files,
    suggestions,
  });
}
