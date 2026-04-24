<template>
  <section>
    <div class="page-header">
      <div>
        <h2 class="page-title">智能识别</h2>
        <p class="page-desc">粘贴招聘公告中的材料要求，自动提取本次需要提交的材料。</p>
      </div>
      <el-button type="primary" icon="el-icon-magic-stick" @click="extract">识别材料</el-button>
    </div>

    <el-row :gutter="18">
      <el-col :span="12">
        <el-card class="card" shadow="never">
          <div slot="header">
            <strong>招聘材料要求</strong>
          </div>
          <el-input
            v-model="requirementText"
            type="textarea"
            :rows="18"
            placeholder="例如：请上传身份证、毕业证、学位证、教育部学历证书电子注册备案表、个人简历等材料。"
          />
        </el-card>
      </el-col>

      <el-col :span="12">
        <el-card class="card" shadow="never">
          <div slot="header" class="result-header">
            <strong>识别结果</strong>
            <el-tag type="info">当前为本地规则模拟</el-tag>
          </div>

          <div v-if="!results.length" class="empty-state">
            识别后会在这里显示匹配到的材料、建议文件名和缺失项。
          </div>

          <div v-for="item in results" :key="item.requiredMaterial" class="result-row">
            <div>
              <strong>{{ item.requiredMaterial }}</strong>
              <div class="muted">
                {{ item.status === 'matched' ? `匹配：${item.matchedMaterialName}` : item.reason }}
              </div>
              <div class="file-path">建议文件名：{{ item.suggestedFilename }}</div>
            </div>
            <el-tag :type="item.status === 'matched' ? 'success' : 'warning'">
              {{ item.status === 'matched' ? '已匹配' : '缺失' }}
            </el-tag>
          </div>
        </el-card>

        <el-card class="card prompt-card" shadow="never">
          <div slot="header">
            <strong>LLM 响应格式草案</strong>
          </div>
          <pre class="prompt">{{ promptText }}</pre>
        </el-card>
      </el-col>
    </el-row>
  </section>
</template>

<script>
import { buildLlmPrompt, extractByRules } from '../services/matcher';
import { listMaterials } from '../services/storage';

export default {
  name: 'SmartExtractView',
  data() {
    return {
      requirementText: '',
      materials: [],
      results: [],
      promptText: '',
    };
  },
  async created() {
    this.materials = await listMaterials();
    this.updatePrompt();
  },
  watch: {
    requirementText() {
      this.updatePrompt();
    },
  },
  methods: {
    extract() {
      if (!this.requirementText.trim()) {
        this.$message.warning('请先粘贴材料要求');
        return;
      }

      this.results = extractByRules(this.requirementText, this.materials);
      this.updatePrompt();
      if (!this.results.length) {
        this.$message.info('没有识别到常见材料，后续接入 LLM 后会更灵活');
      }
    },
    updatePrompt() {
      this.promptText = JSON.stringify(buildLlmPrompt(this.requirementText, this.materials), null, 2);
    },
  },
};
</script>

<style scoped>
.result-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.result-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  padding: 16px 0;
  border-bottom: 1px solid var(--border);
}

.result-row:last-child {
  border-bottom: 0;
}

.prompt-card {
  margin-top: 18px;
}

.prompt {
  max-height: 320px;
  overflow: auto;
  padding: 14px;
  border-radius: 14px;
  background: var(--text);
  color: var(--panel);
  font-size: 12px;
}
</style>
