<template>
  <div id="app">
    <div class="ide-shell">
      <aside class="sidebar">
        <div class="brand">
          <div class="brand-logo">
            <div class="logo-rect logo-rect-1"></div>
            <div class="logo-rect logo-rect-2"></div>
          </div>
          <div class="brand-text">
            <h1>材料助手</h1>
            <p>智能提取与打包</p>
          </div>
        </div>
        <nav class="side-nav">
          <router-link to="/export" class="nav-item" active-class="is-active">
            <i class="el-icon-finished"></i> 导出工作台
          </router-link>
          <router-link to="/library" class="nav-item" active-class="is-active">
            <i class="el-icon-folder-opened"></i> 材料库
          </router-link>
          <router-link to="/settings" class="nav-item" active-class="is-active">
            <i class="el-icon-setting"></i> 设置
          </router-link>
        </nav>
      </aside>
      <main class="main-content">
        <router-view />
      </main>
    </div>
  </div>
</template>

<script>
export default {
  name: 'App',
};
</script>