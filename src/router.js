import Vue from 'vue';
import Router from 'vue-router';
import LibraryView from './views/LibraryView.vue';
import ExportView from './views/ExportView.vue';
import SettingsView from './views/SettingsView.vue';

Vue.use(Router);

export default new Router({
  mode: 'hash',
  routes: [
    { path: '/', redirect: '/export' },
    { path: '/export', name: 'export', component: ExportView },
    { path: '/library', name: 'library', component: LibraryView },
    { path: '/settings', name: 'settings', component: SettingsView },
  ],
});
