<template>
  <section class="export-page">
    <div class="workspace-grid">
      <div class="left-rail">
        <el-card v-if="llmEnabled" class="card smart-card" shadow="never">
          <div slot="header" class="smart-header">
            <div>
              <strong>智能识别</strong>
              <p>把公告里的材料要求粘进来，自动补到右侧清单。</p>
            </div>
            <el-tag type="success" size="mini">LLM 已开启</el-tag>
          </div>
          <el-input
            v-model="requirementText"
            type="textarea"
            :rows="5"
            placeholder="粘贴招聘公告中的提交材料要求，例如：请上传身份证、毕业证、学位证、学历证书电子注册备案表。"
          />
          <div class="smart-actions">
            <el-button
              type="primary"
              icon="el-icon-magic-stick"
              :loading="extracting"
              @click="extractToExport"
            >
              识别并加入清单
            </el-button>
            <el-button plain @click="clearRequirement">清空</el-button>
          </div>

          <div v-if="missingItems.length" class="missing-box">
            <strong>未匹配材料</strong>
            <div v-for="item in missingItems" :key="item.requiredMaterial" class="missing-row">
              <span>{{ item.requiredMaterial }}</span>
              <el-tag type="warning" size="mini">{{ item.reason }}</el-tag>
            </div>
          </div>
        </el-card>

        <el-card v-else class="card smart-card smart-disabled-card" shadow="never">
          <div class="smart-disabled">
            <div>
              <strong>智能识别未开启</strong>
              <p>在设置中配置并启用 LLM 后，这里会直接显示招聘要求输入框。</p>
            </div>
            <el-button size="mini" type="primary" plain @click="goToSettings">去开启</el-button>
          </div>
        </el-card>

        <el-card class="card" shadow="never">
          <div slot="header">
            <div class="section-title">
              <strong>手动添加</strong>
              <span>{{ filteredMaterials.length }} 项可选</span>
            </div>
          </div>
          <el-input v-model="keyword" clearable prefix-icon="el-icon-search" placeholder="搜索并添加材料" />
          <div class="material-picker">
            <div v-for="material in filteredMaterials" :key="material.id" class="pick-row" @click="addToExport(material)">
              <div class="pick-main">
                <strong>{{ material.name }}</strong>
              </div>
              <el-button class="pick-action" size="mini" type="success" plain @click.stop="addToExport(material)">加入</el-button>
            </div>
            <div v-if="!filteredMaterials.length" class="empty-state">没有匹配材料。</div>
          </div>
        </el-card>
      </div>

      <div class="main-board">
        <el-card class="card export-card" shadow="never">
          <div slot="header" class="export-header">
            <div>
              <strong>导出清单</strong>
              <p>默认使用每个材料设置的默认文件，必要时可切换版本。</p>
            </div>
            <div class="export-header-actions">
              <el-button size="small" type="danger" plain icon="el-icon-delete" @click="clearItems" :disabled="!items.length">清空</el-button>
            </div>
          </div>

          <div v-if="!items.length" class="empty-state">
            <strong>清单还是空的</strong>
            <br />
            可以先粘贴公告智能识别，也可以从左侧手动添加材料。
          </div>

          <div v-for="(item, index) in items" :key="item.uid" class="export-item">
            <div class="export-row-header">
              <div class="export-index">{{ index + 1 }}</div>
              <div class="export-title">
                <strong>{{ item.material.name }}</strong>
                <el-tag v-if="!item.file" type="warning" size="mini">缺少文件</el-tag>
                <el-tag v-if="item.source === 'smart'" type="success" size="mini">智能识别</el-tag>
              </div>
              <el-button class="delete-btn" icon="el-icon-close" type="text" @click="removeItem(item.uid)" />
            </div>
            <div class="export-row-body">
              <div class="export-field">
                <span class="field-label">版本</span>
                <el-select
                  v-model="item.fileId"
                  class="version-select"
                  size="small"
                  placeholder="请选择文件版本"
                  @change="onFileChange(item)"
                >
                  <el-option
                    v-for="file in item.material.files"
                    :key="file.id"
                    :label="fileLabel(file)"
                    :value="file.id"
                  />
                </el-select>
              </div>
              <div class="export-field">
                <span class="field-label">命名</span>
                <el-input
                  v-model="item.outputFilename"
                  class="filename-input"
                  size="small"
                  placeholder="例如：01_身份证.pdf"
                />
              </div>
            </div>
          </div>

          <div v-if="items.length" class="export-footer">
            <el-button type="primary" icon="el-icon-folder-opened" @click="pickDirAndExport" style="width: 100%;">
              选择目录并开始复制
            </el-button>
          </div>
        </el-card>
      </div>
    </div>

    <el-dialog :visible.sync="manifestVisible" title="导出结果" width="860px">
      <el-alert
        type="warning"
        show-icon
        :closable="false"
        :title="manifestNotice"
      />
      <el-table v-if="exportResults.length" class="result-table" :data="exportResults" size="small">
        <el-table-column label="状态" width="90">
          <template slot-scope="{ row }">
            <el-tag :type="row.success ? 'success' : 'danger'" size="mini">
              {{ row.success ? '成功' : '失败' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="material_name" label="材料" width="150" />
        <el-table-column prop="message" label="结果" width="180" />
        <el-table-column prop="output_path" label="输出路径" show-overflow-tooltip />
      </el-table>
      <pre v-else class="manifest">{{ manifestText }}</pre>
      <div slot="footer">
        <el-button v-if="targetDir" icon="el-icon-folder-opened" @click="openTargetDir">打开目标目录</el-button>
        <el-button @click="manifestVisible = false">关闭</el-button>
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
import { buildOutputFilename, cancelLlmRequest, extractMaterialsWithLlm, getDefaultFile, listMaterials, loadLlmSettings } from '../services/storage';
import { copyExportItems, isTauriRuntime, openPath, selectDirectoryPath } from '../services/native';

export default {
  name: 'ExportView',
  data() {
    return {
      keyword: '',
      requirementText: '',
      targetDir: '',
      materials: [],
      items: [],
      missingItems: [],
      manifestVisible: false,
      manifestText: '',
      manifestNotice: '',
      exportResults: [],
      llmSettings: null,
      extracting: false,
      progressVisible: false,
      progressTitle: '',
      progressText: '',
      currentRequestId: '',
    };
  },
  computed: {
    llmEnabled() {
      return Boolean(this.llmSettings && this.llmSettings.enabled);
    },
    filteredMaterials() {
      const keyword = this.keyword.trim().toLowerCase();
      if (!keyword) return this.materials;
      return this.materials.filter((material) => {
        const text = [material.name, material.category, ...(material.aliases || []), ...(material.tags || [])]
          .join(' ')
          .toLowerCase();
        return text.includes(keyword);
      });
    },
  },
  async created() {
    const [materials, llmSettings] = await Promise.all([listMaterials(), loadLlmSettings()]);
    this.materials = materials;
    this.llmSettings = llmSettings;
  },
  methods: {
    defaultFile(material) {
      return getDefaultFile(material);
    },
    goToSettings() {
      this.$router.push('/settings');
    },
    addToExport(material) {
      this.addMaterialToExport(material);
    },
    addMaterialToExport(material, options = {}) {
      const existing = this.items.find((item) => item.material.id === material.id);
      if (existing) {
        existing.outputFilename = options.outputFilename || existing.outputFilename;
        existing.source = options.source || existing.source;
        existing.requiredMaterial = options.requiredMaterial || existing.requiredMaterial;
        return false;
      }

      const file = getDefaultFile(material);
      this.items.push({
        uid: `${material.id}-${Date.now()}-${Math.random().toString(16).slice(2)}`,
        material,
        fileId: file ? file.id : '',
        file,
        outputFilename: options.outputFilename || buildOutputFilename(material, file),
        source: options.source || 'manual',
        requiredMaterial: options.requiredMaterial || material.name,
      });
      return true;
    },
    async extractToExport() {
      if (!this.requirementText.trim()) {
        this.$message.warning('请先粘贴材料要求');
        return;
      }

      if (!this.llmEnabled) {
        this.$message.info('请先在设置中开启 LLM');
        this.goToSettings();
        return;
      }

      try {
        this.extracting = true;
        this.currentRequestId = `llm-extract-${Date.now()}-${Math.random().toString(16).slice(2)}`;
        this.progressTitle = '正在识别材料';
        this.progressText = '正在分析招聘要求并匹配材料库，你可以随时取消。';
        this.progressVisible = true;
        const results = await extractMaterialsWithLlm(this.currentRequestId, this.requirementText, this.buildLlmCatalog());
        const matchedItems = results.filter((item) => item.status === 'matched');
        this.missingItems = results.filter((item) => item.status !== 'matched');

        const addedCount = matchedItems.reduce((count, result) => {
          const material = this.materials.find((item) => item.id === result.matchedMaterialId);
          if (!material) return count;

          const added = this.addMaterialToExport(material, {
            outputFilename: result.suggestedFilename,
            source: 'smart',
            requiredMaterial: result.requiredMaterial,
          });
          return added ? count + 1 : count;
        }, 0);

        if (!results.length) {
          this.$message.info('没有识别出可处理的材料');
        } else {
          this.$message.success(`已加入 ${addedCount} 个材料，${this.missingItems.length} 个未匹配`);
        }
      } catch (error) {
        const message = String(error.message || error);
        if (message !== '操作已取消') {
          this.$message.error(message);
        }
      } finally {
        this.extracting = false;
        this.progressVisible = false;
        this.currentRequestId = '';
      }
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
    buildLlmCatalog() {
      return this.materials.map((material) => {
        const file = getDefaultFile(material);
        return {
          id: material.id,
          name: material.name,
          category: material.category || '',
          aliases: material.aliases || [],
          defaultFilename: file ? file.originalFilename : '',
          availableFormats: (material.files || []).map((item) => item.format).filter(Boolean),
        };
      });
    },
    clearRequirement() {
      this.requirementText = '';
      this.missingItems = [];
    },
    onFileChange(item) {
      item.file = (item.material.files || []).find((file) => file.id === item.fileId) || null;
      if (item.file) {
        item.outputFilename = buildOutputFilename(item.material, item.file);
      }
    },
    fileLabel(file) {
      return `${file.isDefault ? '默认 · ' : ''}${file.originalFilename} · ${file.format}`;
    },
    removeItem(uid) {
      this.items = this.items.filter((item) => item.uid !== uid);
    },
    clearItems() {
      this.$confirm('确定要清空当前清单吗？', '清空清单', { type: 'warning' })
        .then(() => {
          this.items = [];
        })
        .catch(() => {});
    },
    async pickDirAndExport() {
      if (!this.items.length) {
        this.$message.warning('请先添加需要导出的材料');
        return;
      }

      const targetDir = await selectDirectoryPath();
      if (!targetDir) {
        this.$message.info('当前浏览器调试环境不支持系统目录选择器，请在 Tauri 桌面窗口中使用');
        // Optional: you can still allow simulated export without a real directory in dev mode
        if (isTauriRuntime()) return;
      }

      this.targetDir = targetDir;
      this.exportManifest();
    },
    async exportManifest() {
      if (!this.items.length) {
        this.$message.warning('请先添加需要导出的材料');
        return;
      }

      const manifest = {
        targetDir: this.targetDir,
        items: this.items.map((item) => ({
          materialName: item.material.name,
          requiredMaterial: item.requiredMaterial,
          sourcePath: item.file ? item.file.filePath : '',
          outputFilename: item.outputFilename,
          fileVersion: item.file ? item.file.description || item.file.format : '',
          ready: Boolean(item.file && item.outputFilename),
        })),
      };

      if (isTauriRuntime()) {
        if (!this.targetDir.trim()) {
          this.$message.warning('请选择目标目录');
          return;
        }

        try {
          const results = await copyExportItems(manifest.items, this.targetDir);
          this.exportResults = results;
          this.manifestNotice = '文件复制已执行，请检查每项结果。';
          this.manifestText = '';
        } catch (error) {
          this.exportResults = [];
          this.manifestNotice = '文件复制失败，下面是错误信息和导出清单。';
          this.manifestText = JSON.stringify({ error: String(error), manifest }, null, 2);
        }
      } else {
        this.exportResults = [];
        this.manifestNotice = '当前是浏览器调试环境，仅生成导出清单预览；Tauri 桌面窗口中会直接复制文件。';
        this.manifestText = JSON.stringify(manifest, null, 2);
      }
      this.manifestVisible = true;
    },
    async openTargetDir() {
      const opened = await openPath(this.targetDir);
      if (!opened) {
        this.$message.info('当前浏览器调试环境不支持打开目录');
      }
    },
  },
};
</script>

<style scoped>
.export-page {
  height: 100vh;
  display: flex;
  flex-direction: column;
}

.workspace-grid {
  display: flex;
  height: 100vh;
}

.left-rail {
  width: 380px;
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--border);
  background: var(--panel-strong);
  overflow-y: auto;
}

.smart-card {
  border: 0;
  border-radius: 0;
  border-bottom: 1px solid var(--border);
  background: transparent;
}

.smart-card >>> .el-card__header {
  background: transparent;
}

.smart-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.smart-header p,
.export-header p {
  margin: 3px 0 0;
  color: var(--muted);
  font-size: 11px;
  line-height: 1.35;
}

.section-title {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
}

.section-title span {
  color: var(--muted);
  font-size: 11px;
}

.smart-actions {
  display: grid;
  grid-template-columns: 1fr auto;
  gap: 8px;
  margin-top: 8px;
}

.smart-disabled-card {
  display: flex;
  align-items: center;
}

.smart-disabled {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.smart-disabled p {
  margin: 4px 0 0;
  color: var(--muted);
  font-size: 11px;
  line-height: 1.45;
}

.missing-box {
  margin-top: 8px;
  padding: 8px;
  border-radius: 4px;
  background: var(--panel);
  border: 1px dashed var(--border-strong);
  font-size: 12px;
}

.missing-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  margin-top: 6px;
}

.material-picker {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 6px;
  margin-top: 8px;
  max-height: 340px;
  overflow: auto;
}

.pick-row {
  display: flex;
  justify-content: space-between;
  gap: 6px;
  align-items: center;
  width: 100%;
  min-width: 0;
  padding: 6px 8px;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: var(--panel);
  cursor: pointer;
  position: relative;
  overflow: hidden;
}

.pick-main {
  min-width: 0;
  padding-right: 58px;
}

.pick-row strong {
  display: block;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 12px;
}

.pick-action {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  opacity: 0;
  pointer-events: none;
  transition: opacity 0.16s ease;
}

.pick-row:hover .pick-action,
.pick-row:focus-within .pick-action {
  opacity: 1;
  pointer-events: auto;
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
  font-size: 12px;
}

.export-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.left-rail .card {
  border: 0;
  border-radius: 0;
  background: transparent;
}

.left-rail .card >>> .el-card__header {
  background: transparent;
}

.main-board {
  flex: 1;
  overflow-y: auto;
  background: var(--panel);
}

.export-card {
  border: 0;
  border-radius: 0;
  min-height: 100%;
  display: flex;
  flex-direction: column;
}

.export-card >>> .el-card__body {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding-bottom: 0;
}

.export-card >>> .el-card__header {
  border-bottom: 1px solid var(--border);
  position: sticky;
  top: 0;
  z-index: 10;
  background: var(--panel-strong);
}

.export-item {
  margin-bottom: 8px;
  padding: 10px 12px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--panel);
}

.export-item:last-child {
  margin-bottom: 0;
}

.export-row-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.export-index {
  width: 20px;
  height: 20px;
  border-radius: 4px;
  display: grid;
  place-items: center;
  color: var(--text);
  background: var(--panel-strong);
  font-weight: 700;
  font-size: 11px;
  border: 1px solid var(--border);
  flex-shrink: 0;
}

.export-title {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

.export-title strong {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 12px;
}

.delete-btn {
  padding: 0;
  color: var(--muted);
}

.delete-btn:hover {
  color: #e00;
}

.export-row-body {
  display: grid;
  grid-template-columns: minmax(0, 1.2fr) minmax(0, 1.8fr);
  gap: 12px;
  padding-left: 28px;
}

.export-field {
  display: flex;
  align-items: center;
  gap: 6px;
}

.field-label {
  font-size: 11px;
  color: var(--muted);
  white-space: nowrap;
}

.version-select,
.filename-input {
  flex: 1;
  min-width: 0;
}

.manifest {
  max-height: 420px;
  overflow: auto;
  margin: 16px 0 0;
  padding: 16px;
  border-radius: 8px;
  background: #172026;
  color: #e6fffb;
}

.result-table {
  margin-top: 14px;
}

.export-footer {
  position: sticky;
  bottom: 0;
  padding: 12px 16px;
  border-top: 1px solid var(--border);
  background: var(--panel);
  box-shadow: 0 -4px 12px rgba(0, 0, 0, 0.03);
  margin-top: auto;
}

@media (max-width: 980px) {
  .workspace-grid {
    grid-template-columns: 1fr;
  }

  .export-header {
    grid-template-columns: 1fr;
  }

  .material-picker {
    grid-template-columns: 1fr 1fr;
  }
}
</style>
