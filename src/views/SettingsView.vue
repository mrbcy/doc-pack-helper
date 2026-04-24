<template>
  <section class="sub-page">
    <div class="page-header">
      <div>
        <h2 class="page-title">设置</h2>
        <p class="page-desc">配置并测试 OpenAI-compatible LLM 接口，导出页会根据这里的开关自动联动。</p>
      </div>
      <div class="header-actions">
        <el-button
          size="small"
          :loading="testing"
          @click="testConnection"
        >
          测试连通性
        </el-button>
        <el-button
          type="primary"
          icon="el-icon-check"
          size="small"
          :loading="saving"
          @click="save"
        >
          保存设置
        </el-button>
      </div>
    </div>

    <div class="settings-container" v-loading="loading">
      <el-form label-width="110px" size="small">
        <el-form-item label="启用 LLM">
          <el-switch v-model="settings.enabled" />
        </el-form-item>
        <el-form-item label="Base URL">
          <el-input v-model.trim="settings.baseUrl" placeholder="https://api.openai.com/v1" />
        </el-form-item>
        <el-form-item label="API Key">
          <el-input v-model.trim="settings.apiKey" show-password placeholder="sk-..." />
        </el-form-item>
        <el-form-item label="模型">
          <el-input v-model.trim="settings.model" placeholder="例如：gpt-4.1-mini" />
        </el-form-item>
      </el-form>

      <el-alert
        type="info"
        show-icon
        :closable="false"
        title="建议先点一次“测试连通性”，确认可用后再在导出页使用智能识别。"
        class="alert-box"
      />
    </div>

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
import { cancelLlmRequest, loadLlmSettings, saveLlmSettings, testLlmConnection } from '../services/storage';

function createDefaultSettings() {
  return {
    enabled: false,
    baseUrl: 'https://api.openai.com/v1',
    apiKey: '',
    model: 'gpt-4.1-mini',
  };
}

export default {
  name: 'SettingsView',
  data() {
    return {
      loading: false,
      saving: false,
      testing: false,
      progressVisible: false,
      progressTitle: '',
      progressText: '',
      currentRequestId: '',
      settings: createDefaultSettings(),
    };
  },
  async created() {
    this.loading = true;
    try {
      this.settings = {
        ...createDefaultSettings(),
        ...(await loadLlmSettings()),
      };
    } finally {
      this.loading = false;
    }
  },
  methods: {
    validateSettings() {
      if (!this.settings.baseUrl) {
        throw new Error('请填写 Base URL');
      }
      if (!this.settings.apiKey) {
        throw new Error('请填写 API Key');
      }
      if (!this.settings.model) {
        throw new Error('请填写模型名称');
      }
    },
    async save() {
      try {
        this.saving = true;
        await saveLlmSettings(this.settings);
        this.$message.success('设置已保存');
      } catch (error) {
        this.$message.error(String(error.message || error));
      } finally {
        this.saving = false;
      }
    },
    async testConnection() {
      try {
        this.validateSettings();
        this.testing = true;
        this.currentRequestId = `llm-test-${Date.now()}-${Math.random().toString(16).slice(2)}`;
        this.progressTitle = '正在测试连通性';
        this.progressText = '正在请求 LLM 接口，请稍候。';
        this.progressVisible = true;
        await saveLlmSettings(this.settings);
        const message = await testLlmConnection(this.currentRequestId);
        this.$message.success(message || '连通性测试成功');
      } catch (error) {
        const message = String(error.message || error);
        if (message !== '操作已取消') {
          this.$message.error(message);
        }
      } finally {
        this.testing = false;
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
  },
};
</script>

<style scoped>
.sub-page {
  padding: 8px 16px 24px;
  font-size: 12px;
}

.header-actions {
  display: flex;
  gap: 8px;
}

.settings-container {
  max-width: 720px;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 16px;
}

.alert-box {
  margin-top: 16px;
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

.sub-page >>> .el-form-item {
  margin-bottom: 10px;
}
</style>
