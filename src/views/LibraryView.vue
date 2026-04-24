<template>
  <section class="sub-page">
    <div class="page-header">
      <div>
        <h2 class="page-title">材料库</h2>
        <p class="page-desc">按分类整理材料，给每个材料绑定多个文件版本，并指定默认导出版本。</p>
      </div>
      <div class="header-actions">
        <el-button size="small" icon="el-icon-magic-stick" @click="openSmartImport">智能批量导入</el-button>
        <el-button type="primary" icon="el-icon-plus" size="small" @click="openMaterialDialog()">新增材料</el-button>
      </div>
    </div>

    <div class="library-layout">
      <aside class="category-panel">
        <div class="category-header">
          <span>分类</span>
          <el-button size="mini" icon="el-icon-setting" @click="openCategoryDialog">管理</el-button>
        </div>
        <button
          class="category-item"
          :class="{ 'is-active': selectedCategory === '全部' }"
          @click="selectedCategory = '全部'"
        >
          <span>全部</span>
          <em>{{ materials.length }}</em>
        </button>
        <button
          v-for="item in categoryOptions"
          :key="item.name"
          class="category-item"
          :class="{ 'is-active': selectedCategory === item.name }"
          @click="selectedCategory = item.name"
        >
          <span>{{ item.name }}</span>
          <em>{{ item.count }}</em>
        </button>
      </aside>

      <div class="materials-panel">
        <div class="panel-toolbar">
          <el-input
            v-model="keyword"
            prefix-icon="el-icon-search"
            size="small"
            clearable
            placeholder="搜索材料名"
            class="search-input"
          />
          <div class="panel-summary">
            <span>{{ filteredMaterials.length }} 个材料</span>
            <span>{{ totalFiles }} 个文件版本</span>
          </div>
        </div>

        <div v-if="filteredMaterials.length" class="material-list">
          <article
            v-for="material in filteredMaterials"
            :key="material.id"
            class="material-card"
            :class="{ 'is-expanded': expandedMaterialId === material.id }"
          >
            <div class="material-main" @click="toggleMaterial(material.id)">
              <div class="material-meta">
                <div class="material-name-row">
                  <strong>{{ material.name }}</strong>
                  <el-tag size="mini" effect="plain">{{ normalizeCategory(material.category) }}</el-tag>
                </div>
                <div class="material-subline">
                  <span>默认文件：{{ defaultFile(material) ? defaultFile(material).originalFilename : '未设置' }}</span>
                  <span>导出名：{{ material.preferredOutputName || material.name }}</span>
                  <span>{{ (material.files || []).length }} 个版本</span>
                </div>
              </div>
              <div class="material-actions" @click.stop>
                <el-button size="mini" @click="openMaterialDialog(material)">编辑</el-button>
                <el-button size="mini" type="success" plain @click="openFileDialog(material)">添加文件</el-button>
                <el-button size="mini" type="danger" plain @click="removeMaterial(material)">删除</el-button>
              </div>
            </div>

            <div v-if="expandedMaterialId === material.id" class="material-files">
              <div v-if="!material.files || !material.files.length" class="empty-state compact-empty">
                还没有绑定文件，先添加一个文件版本。
              </div>

              <div v-for="file in material.files" :key="file.id" class="file-item">
                <div class="file-item-main">
                  <div class="file-title-row">
                    <el-tag v-if="file.isDefault" size="mini" type="success">默认</el-tag>
                    <strong>{{ file.originalFilename }}</strong>
                    <span class="file-format">{{ file.format }}</span>
                    <span v-if="file.description" class="file-description">{{ file.description }}</span>
                  </div>
                  <div class="file-path">{{ file.filePath }}</div>
                </div>
                <div class="file-actions">
                  <el-button
                    size="mini"
                    :disabled="file.isDefault"
                    @click="setDefault(material.id, file.id)"
                  >
                    设默认
                  </el-button>
                  <el-button size="mini" type="danger" plain @click="removeFile(material.id, file.id)">
                    删除
                  </el-button>
                </div>
              </div>
            </div>
          </article>
        </div>

        <div v-else class="empty-state">当前分类下还没有材料，点击右上角“新增材料”开始维护。</div>
      </div>
    </div>

    <el-dialog :visible.sync="materialDialogVisible" :title="editingMaterial.id ? '编辑材料' : '新增材料'" width="420px" top="10vh">
      <el-form label-width="72px" size="small">
        <el-form-item label="材料名称">
          <el-input v-model="editingMaterial.name" placeholder="例如：毕业证" />
        </el-form-item>
        <el-form-item label="分类">
          <el-select v-model="editingMaterial.category" placeholder="请选择分类" style="width: 100%;">
            <el-option
              v-for="category in availableCategories"
              :key="category"
              :label="category"
              :value="category"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="导出名">
          <el-input v-model.trim="editingMaterial.preferredOutputName" placeholder="例如：身份证、个人简历" />
        </el-form-item>
      </el-form>
      <div slot="footer">
        <el-button size="small" @click="materialDialogVisible = false">取消</el-button>
        <el-button size="small" type="primary" @click="submitMaterial">保存</el-button>
      </div>
    </el-dialog>

    <el-dialog :visible.sync="fileDialogVisible" title="添加文件版本" width="560px" top="10vh">
      <el-form label-width="84px" size="small" class="dialog-form">
        <el-form-item label="文件路径">
          <el-input v-model="editingFile.filePath" placeholder="/Users/yang/Documents/材料/毕业证.pdf">
            <el-button slot="append" icon="el-icon-folder-opened" @click="pickFile">选择</el-button>
          </el-input>
        </el-form-item>
        <el-form-item label="显示文件名">
          <el-input v-model="editingFile.originalFilename" placeholder="留空则从路径自动识别" />
        </el-form-item>
        <el-form-item label="格式">
          <el-input v-model="editingFile.format" placeholder="例如：PDF、PNG、JPG" />
        </el-form-item>
        <el-form-item label="描述">
          <el-input v-model="editingFile.description" placeholder="例如：打印版、高清版" />
        </el-form-item>
        <el-form-item label="默认版本">
          <el-switch v-model="editingFile.isDefault" active-text="设为默认" />
        </el-form-item>
      </el-form>
      <div slot="footer">
        <el-button size="small" @click="fileDialogVisible = false">取消</el-button>
        <el-button size="small" type="primary" @click="submitFile">添加</el-button>
      </div>
    </el-dialog>

    <el-dialog :visible.sync="categoryDialogVisible" title="分类管理" width="420px" top="12vh">
      <div class="category-manage">
        <div class="category-create">
          <el-input
            v-model="newCategoryName"
            size="small"
            placeholder="新增分类名称"
            @keyup.enter.native="createCategory"
          />
          <el-button size="small" type="primary" @click="createCategory">新增</el-button>
        </div>

        <div class="category-manage-list">
          <div v-for="row in categoryDrafts" :key="row.original" class="category-manage-row">
            <el-input v-model="row.name" size="small" />
            <el-button
              size="small"
              :disabled="!row.name.trim() || row.name.trim() === row.original"
              @click="renameCategory(row)"
            >
              改名
            </el-button>
            <el-button
              size="small"
              type="danger"
              plain
              :disabled="categoryCount(row.original) > 0"
              @click="deleteCategory(row.original)"
            >
              删除
            </el-button>
          </div>
        </div>
        <div class="category-manage-tip">只有当前没有材料使用的分类，才允许删除。</div>
      </div>
    </el-dialog>

    <el-dialog :visible.sync="smartImportVisible" title="智能批量导入" width="820px" top="8vh">
      <div class="smart-import">
        <div v-if="!llmEnabled" class="smart-import-disabled">
          <strong>智能批量导入未开启</strong>
          <p>请先在设置中配置并启用 LLM，再回来使用文件夹智能归类导入。</p>
          <el-button size="small" type="primary" plain @click="goToSettings">去开启</el-button>
        </div>

        <template v-else>
          <div class="smart-import-toolbar">
            <el-input :value="importRootDir" readonly size="small" placeholder="请选择要扫描的文件夹" />
            <el-button size="small" icon="el-icon-folder-opened" @click="pickImportFolder">选择文件夹</el-button>
            <el-button size="small" type="primary" :disabled="!importRootDir || !scannedFiles.length" @click="runSmartImport">
              开始识别
            </el-button>
          </div>

          <div v-if="scannedFiles.length" class="scan-summary">
            已扫描 {{ scannedFiles.length }} 个文件，识别结果如下。
          </div>

          <div v-if="importSuggestions.length" class="import-suggestion-list">
            <article v-for="(item, index) in importSuggestions" :key="`${item.materialName}-${index}`" class="import-suggestion-card">
              <div class="import-suggestion-head">
                <div>
                  <strong>{{ item.materialName }}</strong>
                  <div class="import-suggestion-meta">
                    <el-tag size="mini" effect="plain">{{ item.category }}</el-tag>
                    <span>导出名：{{ item.preferredOutputName || item.materialName }}</span>
                    <span>{{ item.filePaths.length }} 个文件</span>
                  </div>
                </div>
                <el-checkbox v-model="item.selected">导入</el-checkbox>
              </div>
              <div v-if="item.reason" class="import-reason">{{ item.reason }}</div>
              <div class="import-file-list">
                <div v-for="filePath in item.filePaths" :key="filePath" class="import-file-row">
                  {{ filePath }}
                </div>
              </div>
            </article>
          </div>

          <div v-else class="empty-state compact-empty">
            选择一个文件夹后，系统会先扫描文件，再通过 LLM 给出归类建议。
          </div>
        </template>
      </div>
      <div slot="footer">
        <el-button size="small" @click="smartImportVisible = false">关闭</el-button>
        <el-button v-if="llmEnabled" size="small" type="primary" :disabled="!selectedImportSuggestions.length" @click="confirmImportSuggestions">
          导入选中项
        </el-button>
      </div>
    </el-dialog>

    <el-dialog
      :visible.sync="progressVisible"
      width="360px"
      :show-close="false"
      :close-on-click-modal="false"
      :close-on-press-escape="false"
      append-to-body
    >
      <div class="progress-box">
        <i class="el-icon-loading progress-icon"></i>
        <strong>{{ progressTitle }}</strong>
        <p>{{ progressText }}</p>
      </div>
      <div slot="footer">
        <el-button size="small" @click="cancelProgress">取消</el-button>
      </div>
    </el-dialog>
  </section>
</template>

<script>
import {
  addMaterialFile,
  cancelLlmRequest,
  deleteMaterial,
  deleteMaterialFile,
  getDefaultFile,
  inferFilename,
  inferFormat,
  finalizeMaterialImportSuggestionsWithLlm,
  listCategories,
  listMaterials,
  loadLlmSettings,
  saveMaterial,
  saveCategories,
  scanImportDirectory,
  suggestMaterialImportsWithLlm,
  setDefaultMaterialFile,
} from '../services/storage';
import { selectDirectoryPath, selectFilePath } from '../services/native';

const CATEGORY_MAP = {
  身份材料: '身份证明',
  身份证明: '身份证明',
  学历材料: '学历学位',
  学历学位: '学历学位',
};

const IMPORT_BATCH_SIZE = 10;

export default {
  name: 'LibraryView',
  data() {
    return {
      keyword: '',
      materials: [],
      categories: [],
      selectedCategory: '全部',
      expandedMaterialId: '',
      materialDialogVisible: false,
      fileDialogVisible: false,
      categoryDialogVisible: false,
      smartImportVisible: false,
      currentMaterialId: '',
      newCategoryName: '',
      categoryDrafts: [],
      llmSettings: null,
      importRootDir: '',
      scannedFiles: [],
      importSuggestions: [],
      progressVisible: false,
      progressTitle: '',
      progressText: '',
      currentRequestId: '',
      editingMaterial: this.createEmptyMaterial(),
      editingFile: this.createEmptyFile(),
    };
  },
  computed: {
    llmEnabled() {
      return Boolean(this.llmSettings && this.llmSettings.enabled);
    },
    availableCategories() {
      const materialCategories = this.materials
        .map((material) => this.normalizeCategory(material.category))
        .filter(Boolean);
      return Array.from(new Set([...this.categories, ...materialCategories]));
    },
    filteredMaterials() {
      const keyword = this.keyword.trim().toLowerCase();
      return this.materials.filter((material) => {
        const normalizedCategory = this.normalizeCategory(material.category);
        const inCategory = this.selectedCategory === '全部' || normalizedCategory === this.selectedCategory;
        if (!inCategory) return false;
        if (!keyword) return true;
        return [material.name, normalizedCategory].join(' ').toLowerCase().includes(keyword);
      });
    },
    categoryOptions() {
      return this.availableCategories.map((name) => ({
        name,
        count: this.materials.filter((material) => this.normalizeCategory(material.category) === name).length,
      }));
    },
    totalFiles() {
      return this.materials.reduce((total, material) => total + (material.files || []).length, 0);
    },
    selectedImportSuggestions() {
      return this.importSuggestions.filter((item) => item.selected);
    },
  },
  async created() {
    this.categories = listCategories();
    const [_, llmSettings] = await Promise.all([this.refresh(), loadLlmSettings()]);
    this.llmSettings = llmSettings;
  },
  methods: {
    createEmptyMaterial() {
      return {
        id: '',
        name: '',
        category: '',
        preferredOutputName: '',
      };
    },
    createEmptyFile() {
      return {
        filePath: '',
        originalFilename: '',
        format: '',
        description: '',
        isDefault: false,
      };
    },
    async refresh() {
      this.materials = await listMaterials();
      if (!this.expandedMaterialId && this.materials.length) {
        this.expandedMaterialId = this.materials[0].id;
      }
    },
    goToSettings() {
      this.$router.push('/settings');
    },
    defaultFile(material) {
      return getDefaultFile(material);
    },
    normalizeCategory(category) {
      return CATEGORY_MAP[category] || category || '其他';
    },
    categoryCount(categoryName) {
      return this.materials.filter((material) => this.normalizeCategory(material.category) === categoryName).length;
    },
    openCategoryDialog() {
      this.newCategoryName = '';
      this.categoryDrafts = this.categories.map((name) => ({ original: name, name }));
      this.categoryDialogVisible = true;
    },
    createCategory() {
      const name = this.newCategoryName.trim();
      if (!name) {
        this.$message.warning('请填写分类名称');
        return;
      }
      if (this.categories.includes(name)) {
        this.$message.warning('该分类已存在');
        return;
      }
      this.categories = saveCategories([...this.categories, name]);
      this.newCategoryName = '';
      this.categoryDrafts = this.categories.map((item) => ({ original: item, name: item }));
      this.$message.success('分类已新增');
    },
    async renameCategory(row) {
      const nextName = row.name.trim();
      if (!nextName) {
        this.$message.warning('分类名称不能为空');
        return;
      }
      if (nextName !== row.original && this.categories.includes(nextName)) {
        this.$message.warning('该分类已存在');
        return;
      }

      const targetMaterials = this.materials.filter(
        (material) => this.normalizeCategory(material.category) === row.original
      );

      for (const material of targetMaterials) {
        await saveMaterial({
          id: material.id,
          name: material.name,
          category: nextName,
          preferredOutputName: material.preferredOutputName || '',
          aliases: material.aliases || [],
          tags: material.tags || [],
          note: material.note || '',
        });
      }

      this.categories = saveCategories(this.categories.map((name) => (name === row.original ? nextName : name)));
      if (this.selectedCategory === row.original) {
        this.selectedCategory = nextName;
      }
      this.categoryDrafts = this.categories.map((item) => ({ original: item, name: item }));
      await this.refresh();
      this.$message.success('分类已改名');
    },
    deleteCategory(categoryName) {
      if (this.categoryCount(categoryName) > 0) {
        this.$message.warning('该分类下还有材料，不能删除');
        return;
      }
      this.categories = saveCategories(this.categories.filter((name) => name !== categoryName));
      if (this.selectedCategory === categoryName) {
        this.selectedCategory = '全部';
      }
      this.categoryDrafts = this.categories.map((item) => ({ original: item, name: item }));
      this.$message.success('分类已删除');
    },
    openSmartImport() {
      this.smartImportVisible = true;
      if (!this.llmEnabled) return;
      this.importSuggestions = [];
      this.scannedFiles = [];
    },
    async pickImportFolder() {
      const rootDir = await selectDirectoryPath();
      if (!rootDir) return;
      this.importRootDir = rootDir;
      this.importSuggestions = [];
      this.scannedFiles = await scanImportDirectory(rootDir);
      if (!this.scannedFiles.length) {
        this.$message.warning('这个文件夹下没有可导入文件');
      } else {
        this.$message.success(`已扫描 ${this.scannedFiles.length} 个文件`);
      }
    },
    async runSmartImport() {
      if (!this.llmEnabled) {
        this.$message.info('请先在设置中开启 LLM');
        return;
      }
      if (!this.importRootDir) {
        this.$message.warning('请先选择文件夹');
        return;
      }
      if (!this.scannedFiles.length) {
        this.$message.warning('当前目录下没有可导入文件');
        return;
      }

      try {
        this.progressVisible = true;
        const batches = this.chunkFiles(this.scannedFiles, IMPORT_BATCH_SIZE);
        let batchSuggestions = [];

        for (let index = 0; index < batches.length; index += 1) {
          const batch = batches[index];
          this.currentRequestId = `llm-import-${Date.now()}-${index}-${Math.random().toString(16).slice(2)}`;
          this.progressTitle = '正在智能归类文件';
          this.progressText = `正在识别第 ${index + 1} / ${batches.length} 批，共 ${batch.length} 个文件，你可以随时取消。`;
          const suggestions = await suggestMaterialImportsWithLlm(this.currentRequestId, this.importRootDir, batch);
          batchSuggestions = batchSuggestions.concat(
            suggestions.map((item) => ({
              ...item,
              filePaths: [...(item.filePaths || [])],
              fileIds: [...(item.fileIds || [])],
            }))
          );
        }

        if (batchSuggestions.length > 1) {
          this.currentRequestId = `llm-import-finalize-${Date.now()}-${Math.random().toString(16).slice(2)}`;
          this.progressTitle = '正在整理全局结果';
          this.progressText = '正在把多批次结果统一命名和归类，避免跨批次错位。';
          batchSuggestions = await finalizeMaterialImportSuggestionsWithLlm(
            this.currentRequestId,
            this.scannedFiles,
            batchSuggestions
          );
        }

        this.importSuggestions = batchSuggestions.map((item) => ({
          ...item,
          selected: true,
        }));
        this.$message.success(`已生成 ${this.importSuggestions.length} 条导入建议`);
      } catch (error) {
        const message = String(error.message || error);
        if (message !== '操作已取消') {
          this.$message.error(message);
        }
      } finally {
        this.progressVisible = false;
        this.currentRequestId = '';
      }
    },
    chunkFiles(files, batchSize) {
      const result = [];
      for (let index = 0; index < files.length; index += batchSize) {
        result.push(files.slice(index, index + batchSize));
      }
      return result;
    },
    async confirmImportSuggestions() {
      if (!this.selectedImportSuggestions.length) {
        this.$message.warning('请先勾选至少一项导入建议');
        return;
      }

      for (const item of this.selectedImportSuggestions) {
        const materialName = item.materialName.trim();
        const category = this.normalizeCategory(item.category);
        const preferredOutputName = (item.preferredOutputName || item.materialName).trim();
        let currentMaterials = await listMaterials();
        let material = currentMaterials.find((entry) => entry.name === materialName && this.normalizeCategory(entry.category) === category);

        if (!material) {
          currentMaterials = await saveMaterial({
            id: '',
            name: materialName,
            category,
            preferredOutputName,
            aliases: [],
            tags: [],
            note: '',
          });
          material = currentMaterials.find((entry) => entry.name === materialName && this.normalizeCategory(entry.category) === category);
        } else if ((material.preferredOutputName || '') !== preferredOutputName) {
          currentMaterials = await saveMaterial({
            id: material.id,
            name: material.name,
            category,
            preferredOutputName,
            aliases: material.aliases || [],
            tags: material.tags || [],
            note: material.note || '',
          });
          material = currentMaterials.find((entry) => entry.id === material.id);
        }

        if (!material) continue;

        const existingPaths = new Set((material.files || []).map((file) => file.filePath));
        let isFirstImportedFile = !(material.files || []).length;
        for (const filePath of item.filePaths) {
          if (existingPaths.has(filePath)) continue;
          currentMaterials = await addMaterialFile(material.id, {
            filePath,
            originalFilename: inferFilename(filePath),
            format: inferFormat(filePath),
            description: '智能导入',
            isDefault: isFirstImportedFile,
          });
          isFirstImportedFile = false;
          material = currentMaterials.find((entry) => entry.id === material.id) || material;
        }
      }

      const nextCategories = Array.from(new Set([
        ...this.categories,
        ...this.selectedImportSuggestions.map((item) => this.normalizeCategory(item.category)).filter(Boolean),
      ]));
      this.categories = saveCategories(nextCategories);
      await this.refresh();
      this.smartImportVisible = false;
      this.$message.success('智能导入完成');
    },
    async cancelProgress() {
      if (!this.currentRequestId) {
        this.progressVisible = false;
        return;
      }
      try {
        await cancelLlmRequest(this.currentRequestId);
      } finally {
        this.progressVisible = false;
      }
    },
    toggleMaterial(materialId) {
      this.expandedMaterialId = this.expandedMaterialId === materialId ? '' : materialId;
    },
    openMaterialDialog(material) {
      this.editingMaterial = material
        ? { ...material, category: this.normalizeCategory(material.category) }
        : this.createEmptyMaterial();
      this.materialDialogVisible = true;
    },
    async submitMaterial() {
      if (!this.editingMaterial.name.trim()) {
        this.$message.warning('请填写材料名称');
        return;
      }

      this.materials = await saveMaterial({
        id: this.editingMaterial.id,
        name: this.editingMaterial.name,
        category: this.normalizeCategory(this.editingMaterial.category),
        preferredOutputName: this.editingMaterial.preferredOutputName || '',
        aliases: [],
        tags: [],
        note: '',
      });
      if (!this.expandedMaterialId && this.materials.length) {
        this.expandedMaterialId = this.materials[0].id;
      }
      this.materialDialogVisible = false;
      this.$message.success('材料已保存');
    },
    removeMaterial(material) {
      this.$confirm(`确定删除“${material.name}”吗？`, '删除材料', { type: 'warning' })
        .then(async () => {
          this.materials = await deleteMaterial(material.id);
          if (this.expandedMaterialId === material.id) {
            this.expandedMaterialId = this.materials[0] ? this.materials[0].id : '';
          }
          this.$message.success('已删除');
        })
        .catch(() => {});
    },
    openFileDialog(material) {
      this.currentMaterialId = material.id;
      this.expandedMaterialId = material.id;
      this.editingFile = this.createEmptyFile();
      this.fileDialogVisible = true;
    },
    async pickFile() {
      const filePath = await selectFilePath();
      if (!filePath) {
        this.$message.info('当前浏览器调试环境不支持系统文件选择器，请在 Tauri 桌面窗口中使用，或先手动填写路径');
        return;
      }

      this.editingFile.filePath = filePath;
      if (!this.editingFile.originalFilename) {
        this.editingFile.originalFilename = inferFilename(filePath);
      }
      if (!this.editingFile.format) {
        this.editingFile.format = inferFormat(filePath);
      }
    },
    async submitFile() {
      if (!this.editingFile.filePath.trim()) {
        this.$message.warning('请填写文件路径');
        return;
      }

      const filePath = this.editingFile.filePath.trim();
      this.materials = await addMaterialFile(this.currentMaterialId, {
        ...this.editingFile,
        filePath,
        originalFilename: this.editingFile.originalFilename || inferFilename(filePath),
        format: this.editingFile.format || inferFormat(filePath),
      });
      this.expandedMaterialId = this.currentMaterialId;
      this.fileDialogVisible = false;
      this.$message.success('文件版本已添加');
    },
    async setDefault(materialId, fileId) {
      this.materials = await setDefaultMaterialFile(materialId, fileId);
      this.expandedMaterialId = materialId;
      this.$message.success('默认文件已更新');
    },
    async removeFile(materialId, fileId) {
      this.materials = await deleteMaterialFile(materialId, fileId);
      this.expandedMaterialId = materialId;
      this.$message.success('文件版本已删除');
    },
  },
};
</script>

<style scoped>
.sub-page {
  padding: 10px 16px 20px;
  font-size: 12px;
}

.header-actions {
  display: flex;
  gap: 8px;
}

.library-layout {
  display: grid;
  grid-template-columns: 200px minmax(0, 1fr);
  gap: 16px;
  align-items: start;
}

.category-panel,
.materials-panel {
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 12px;
}

.category-panel {
  position: sticky;
  top: 10px;
}

.category-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 10px;
  color: var(--muted);
  font-size: 11px;
  font-weight: 600;
}

.category-item {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px;
  margin-bottom: 4px;
  border: 0;
  border-radius: 4px;
  background: transparent;
  color: var(--text);
  font-size: 12px;
  text-align: left;
  cursor: pointer;
}

.category-item:hover {
  background: var(--panel-strong);
}

.category-item.is-active {
  background: var(--text);
  color: #fff;
}

.category-item em {
  font-style: normal;
  color: var(--muted);
}

.category-item.is-active em {
  color: rgba(255, 255, 255, 0.72);
}

.smart-import-toolbar {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto auto;
  gap: 8px;
  align-items: center;
}

.scan-summary {
  margin-top: 10px;
  color: var(--muted);
  font-size: 12px;
}

.smart-import-disabled {
  padding: 12px 0;
}

.smart-import-disabled p {
  margin: 6px 0 12px;
  color: var(--muted);
  line-height: 1.6;
}

.import-suggestion-list {
  margin-top: 12px;
  max-height: 460px;
  overflow: auto;
}

.import-suggestion-card {
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--panel);
  padding: 10px 12px;
  margin-bottom: 10px;
}

.import-suggestion-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.import-suggestion-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 6px;
  color: var(--muted);
  font-size: 11px;
}

.import-reason {
  margin-top: 8px;
  color: var(--muted);
  line-height: 1.5;
}

.import-file-list {
  margin-top: 8px;
  border-top: 1px dashed var(--border);
  padding-top: 8px;
}

.import-file-row {
  font-size: 11px;
  line-height: 1.5;
  color: var(--text);
  word-break: break-all;
}

.panel-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 12px;
}

.search-input {
  width: 280px;
}

.panel-summary {
  display: flex;
  gap: 12px;
  color: var(--muted);
  font-size: 11px;
  white-space: nowrap;
}

.material-list {
  display: grid;
  gap: 10px;
}

.material-card {
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--panel);
}

.material-card.is-expanded {
  border-color: var(--border-strong);
}

.material-main {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px;
  cursor: pointer;
}

.material-meta {
  min-width: 0;
  flex: 1;
}

.material-name-row {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

.material-name-row strong {
  font-size: 13px;
}

.material-subline {
  display: flex;
  gap: 12px;
  margin-top: 4px;
  color: var(--muted);
  font-size: 11px;
}

.material-actions {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.material-files {
  padding: 0 12px 12px;
  border-top: 1px solid var(--border);
  background: var(--panel-strong);
}

.compact-empty {
  margin-top: 10px;
}

.file-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 0;
  border-bottom: 1px solid var(--border);
}

.file-item:last-child {
  border-bottom: 0;
}

.file-item-main {
  min-width: 0;
  flex: 1;
}

.file-title-row {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

.file-title-row strong {
  font-size: 12px;
}

.file-format,
.file-description {
  color: var(--muted);
  font-size: 11px;
}

.file-actions {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.dialog-form {
  margin-top: 8px;
}

.category-manage {
  display: grid;
  gap: 12px;
}

.category-create {
  display: grid;
  grid-template-columns: 1fr auto;
  gap: 8px;
}

.category-manage-list {
  display: grid;
  gap: 8px;
}

.category-manage-row {
  display: grid;
  grid-template-columns: 1fr auto auto;
  gap: 8px;
  align-items: center;
}

.category-manage-tip {
  color: var(--muted);
  font-size: 11px;
}

.progress-box {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 8px 0 4px;
  text-align: center;
}

.progress-icon {
  font-size: 24px;
  color: var(--text);
}

.progress-box p {
  margin: 0;
  color: var(--muted);
  line-height: 1.5;
}

.sub-page >>> .page-title {
  font-size: 18px;
}

.sub-page >>> .page-desc {
  font-size: 12px;
}

.sub-page >>> .el-dialog__title {
  font-size: 14px;
}

@media (max-width: 720px) {
  .library-layout {
    grid-template-columns: 1fr;
  }

  .category-panel {
    position: static;
  }

  .panel-toolbar,
  .material-main,
  .file-item,
  .import-suggestion-head {
    flex-direction: column;
    align-items: stretch;
  }

  .search-input {
    width: 100%;
  }

  .smart-import-toolbar {
    grid-template-columns: 1fr;
  }

  .panel-summary,
  .material-subline,
  .material-actions,
  .file-actions,
  .import-suggestion-meta {
    white-space: normal;
  }
}
</style>
