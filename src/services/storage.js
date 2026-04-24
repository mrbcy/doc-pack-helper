import {
  addMaterialFileToDb,
  cancelLlmRequestFromNative,
  deleteMaterialFileFromDb,
  deleteMaterialFromDb,
  extractMaterialsWithLlmFromNative,
  finalizeMaterialImportSuggestionsWithLlmFromNative,
  isTauriRuntime,
  listMaterialsFromDb,
  loadLlmSettingsFromNative,
  saveMaterialToDb,
  saveLlmSettingsToNative,
  scanImportDirectoryFromNative,
  suggestMaterialImportsWithLlmFromNative,
  setDefaultMaterialFileInDb,
  testLlmConnectionFromNative,
} from './native';

const STORAGE_KEY = 'doc-pack-helper-state-v1';
const CATEGORY_STORAGE_KEY = 'doc-pack-helper-categories-v1';
export const DEFAULT_CATEGORIES = [
  '身份证明',
  '学历学位',
  '学信网',
  '简历',
  '照片',
  '证明材料',
  '其他',
];

const sampleState = {
  materials: [
    {
      id: 'mat-id-card',
      name: '身份证',
      category: '身份材料',
      aliases: ['居民身份证', '身份证扫描件'],
      tags: ['常用', '身份'],
      note: '可维护正反面合并 PDF 或图片版本。',
      createdAt: now(),
      updatedAt: now(),
      files: [
        {
          id: 'file-id-card-pdf',
          format: 'PDF',
          filePath: '/Users/yang/Documents/证件/身份证.pdf',
          originalFilename: '身份证.pdf',
          description: '正反面合并版',
          isDefault: true,
        },
      ],
    },
    {
      id: 'mat-degree',
      name: '学位证',
      category: '学历材料',
      aliases: ['学士学位证书'],
      tags: ['学历'],
      note: '',
      createdAt: now(),
      updatedAt: now(),
      files: [],
    },
  ],
  llmSettings: {
    enabled: false,
    baseUrl: 'https://api.openai.com/v1',
    apiKey: '',
    model: 'gpt-4.1-mini',
  },
};

function now() {
  return new Date().toISOString();
}

function clone(value) {
  return JSON.parse(JSON.stringify(value));
}

function makeId(prefix) {
  return `${prefix}-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

export function loadState() {
  const raw = localStorage.getItem(STORAGE_KEY);
  if (!raw) {
    saveState(sampleState);
    return clone(sampleState);
  }

  try {
    return JSON.parse(raw);
  } catch (error) {
    console.warn('Failed to parse app state. Falling back to sample data.', error);
    saveState(sampleState);
    return clone(sampleState);
  }
}

export function saveState(state) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
}

export function listCategories() {
  const raw = localStorage.getItem(CATEGORY_STORAGE_KEY);
  if (!raw) {
    localStorage.setItem(CATEGORY_STORAGE_KEY, JSON.stringify(DEFAULT_CATEGORIES));
    return [...DEFAULT_CATEGORIES];
  }

  try {
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed) || !parsed.length) {
      return [...DEFAULT_CATEGORIES];
    }
    return parsed.filter(Boolean);
  } catch (error) {
    console.warn('Failed to parse categories. Falling back to defaults.', error);
    return [...DEFAULT_CATEGORIES];
  }
}

export function saveCategories(categories) {
  const next = Array.from(new Set((categories || []).map((item) => String(item || '').trim()).filter(Boolean)));
  localStorage.setItem(CATEGORY_STORAGE_KEY, JSON.stringify(next.length ? next : DEFAULT_CATEGORIES));
  return listCategories();
}

export async function listMaterials() {
  if (isTauriRuntime()) {
    const materials = await listMaterialsFromDb();
    if (materials.length > 0) return materials;
    return migrateLocalStorageMaterials();
  }
  return loadState().materials || [];
}

async function migrateLocalStorageMaterials() {
  const raw = localStorage.getItem(STORAGE_KEY);
  if (!raw) return [];

  let oldState;
  try {
    oldState = JSON.parse(raw);
  } catch (error) {
    console.warn('Failed to migrate localStorage materials.', error);
    return [];
  }

  const oldMaterials = oldState.materials || [];
  if (!oldMaterials.length) return [];

  for (const material of oldMaterials) {
    await saveMaterialToDb({
      id: material.id || null,
      name: material.name,
      category: material.category || '',
      preferredOutputName: material.preferredOutputName || '',
      aliases: normalizeList(material.aliases),
      tags: normalizeList(material.tags),
      note: material.note || '',
    });

    for (const file of material.files || []) {
      await addMaterialFileToDb(material.id, {
        format: file.format || inferFormat(file.filePath),
        filePath: file.filePath,
        originalFilename: file.originalFilename || inferFilename(file.filePath),
        description: file.description || '',
        isDefault: Boolean(file.isDefault),
      });
    }
  }

  return listMaterialsFromDb();
}

export async function saveMaterial(material) {
  if (isTauriRuntime()) {
    return saveMaterialToDb({
      id: material.id || null,
      name: material.name,
      category: material.category || '',
      preferredOutputName: material.preferredOutputName || '',
      aliases: normalizeList(material.aliases),
      tags: normalizeList(material.tags),
      note: material.note || '',
    });
  }

  const state = loadState();
  const next = {
    ...material,
    aliases: normalizeList(material.aliases),
    tags: normalizeList(material.tags),
    updatedAt: now(),
  };

  if (next.id) {
    const index = state.materials.findIndex((item) => item.id === next.id);
    if (index >= 0) {
      state.materials.splice(index, 1, {
        ...state.materials[index],
        ...next,
        files: state.materials[index].files || [],
      });
    }
  } else {
    state.materials.unshift({
      ...next,
      id: makeId('mat'),
      createdAt: now(),
      files: [],
    });
  }

  saveState(state);
  return state.materials;
}

export async function deleteMaterial(materialId) {
  if (isTauriRuntime()) {
    return deleteMaterialFromDb(materialId);
  }

  const state = loadState();
  state.materials = state.materials.filter((item) => item.id !== materialId);
  saveState(state);
  return state.materials;
}

export async function addMaterialFile(materialId, file) {
  if (isTauriRuntime()) {
    return addMaterialFileToDb(materialId, {
      format: file.format || inferFormat(file.filePath),
      filePath: file.filePath,
      originalFilename: file.originalFilename || inferFilename(file.filePath),
      description: file.description || '',
      isDefault: Boolean(file.isDefault),
    });
  }

  const state = loadState();
  const material = state.materials.find((item) => item.id === materialId);
  if (!material) return state.materials;

  const files = material.files || [];
  const shouldBeDefault = files.length === 0 || file.isDefault;
  if (shouldBeDefault) {
    files.forEach((item) => {
      item.isDefault = false;
    });
  }

  files.push({
    ...file,
    id: makeId('file'),
    format: file.format || inferFormat(file.filePath),
    originalFilename: file.originalFilename || inferFilename(file.filePath),
    isDefault: shouldBeDefault,
  });

  material.files = files;
  material.updatedAt = now();
  saveState(state);
  return state.materials;
}

export async function deleteMaterialFile(materialId, fileId) {
  if (isTauriRuntime()) {
    return deleteMaterialFileFromDb(materialId, fileId);
  }

  const state = loadState();
  const material = state.materials.find((item) => item.id === materialId);
  if (!material) return state.materials;

  const wasDefault = (material.files || []).some((item) => item.id === fileId && item.isDefault);
  material.files = (material.files || []).filter((item) => item.id !== fileId);
  if (wasDefault && material.files.length > 0) {
    material.files[0].isDefault = true;
  }

  material.updatedAt = now();
  saveState(state);
  return state.materials;
}

export async function setDefaultMaterialFile(materialId, fileId) {
  if (isTauriRuntime()) {
    return setDefaultMaterialFileInDb(materialId, fileId);
  }

  const state = loadState();
  const material = state.materials.find((item) => item.id === materialId);
  if (!material) return state.materials;

  material.files = (material.files || []).map((file) => ({
    ...file,
    isDefault: file.id === fileId,
  }));
  material.updatedAt = now();
  saveState(state);
  return state.materials;
}

export async function loadLlmSettings() {
  if (isTauriRuntime()) {
    return loadLlmSettingsFromNative();
  }

  return loadState().llmSettings || sampleState.llmSettings;
}

export async function saveLlmSettings(settings) {
  if (isTauriRuntime()) {
    return saveLlmSettingsToNative(settings);
  }

  const state = loadState();
  state.llmSettings = settings;
  saveState(state);
  return settings;
}

export async function testLlmConnection(requestId) {
  if (isTauriRuntime()) {
    return testLlmConnectionFromNative(requestId);
  }

  throw new Error('请在桌面版中测试 LLM 连通性');
}

export async function extractMaterialsWithLlm(requestId, requirementText, materials) {
  if (isTauriRuntime()) {
    return extractMaterialsWithLlmFromNative(requestId, requirementText, materials);
  }

  throw new Error('请在桌面版中使用 LLM 智能识别');
}

export async function cancelLlmRequest(requestId) {
  if (isTauriRuntime()) {
    return cancelLlmRequestFromNative(requestId);
  }

  return null;
}

export async function scanImportDirectory(rootDir) {
  if (isTauriRuntime()) {
    return scanImportDirectoryFromNative(rootDir);
  }

  throw new Error('请在桌面版中使用智能批量导入');
}

export async function suggestMaterialImportsWithLlm(requestId, rootDir, files) {
  if (isTauriRuntime()) {
    return suggestMaterialImportsWithLlmFromNative(requestId, rootDir, files);
  }

  throw new Error('请在桌面版中使用智能批量导入');
}

export async function finalizeMaterialImportSuggestionsWithLlm(requestId, files, suggestions) {
  if (isTauriRuntime()) {
    return finalizeMaterialImportSuggestionsWithLlmFromNative(requestId, files, suggestions);
  }

  throw new Error('请在桌面版中使用智能批量导入');
}

export function getDefaultFile(material) {
  const files = material.files || [];
  return files.find((file) => file.isDefault) || files[0] || null;
}

export function inferExtension(filePath) {
  const filename = inferFilename(filePath);
  return filename.includes('.') ? filename.split('.').pop() : '';
}

export function inferBaseName(filename) {
  if (!filename) return '';
  const clean = inferFilename(filename);
  const lastDot = clean.lastIndexOf('.');
  if (lastDot <= 0) return clean;
  return clean.slice(0, lastDot);
}

export function buildOutputFilename(material, file) {
  if (!file) {
    return material.preferredOutputName || `${material.name}.pdf`;
  }

  const ext = inferExtension(file.originalFilename || file.filePath) || inferExtension(file.filePath) || 'pdf';
  const preferred = String(material.preferredOutputName || '').trim();
  if (!preferred) {
    return inferFilename(file.filePath);
  }

  const baseName = inferBaseName(preferred) || material.name;
  return `${baseName}.${ext}`;
}

export function inferFilename(filePath) {
  if (!filePath) return '';
  return filePath.split(/[\\/]/).filter(Boolean).pop() || filePath;
}

export function inferFormat(filePath) {
  const filename = inferFilename(filePath);
  const ext = filename.includes('.') ? filename.split('.').pop() : '';
  return ext ? ext.toUpperCase() : '未知';
}

export function normalizeList(value) {
  if (Array.isArray(value)) return value.filter(Boolean);
  return String(value || '')
    .split(/[,，\n]/)
    .map((item) => item.trim())
    .filter(Boolean);
}
