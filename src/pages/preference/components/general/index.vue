<script setup lang="ts">
import { disable, enable, isEnabled } from '@tauri-apps/plugin-autostart'
import { Switch, message } from 'antdv-next'
import { onMounted, ref, watch } from 'vue'

import ProListItem from '@/components/pro-list-item/index.vue'
import ProList from '@/components/pro-list/index.vue'
import ReportExport from '@/components/report-export/index.vue'
import { analyticsService } from '@/services/analytics'
import { useGeneralStore } from '@/stores/general'
import { isMac, isWindows } from '@/utils/platform'

import Language from './components/language/index.vue'
import MacosPermissions from './components/macos-permissions/index.vue'
import ThemeMode from './components/theme-mode/index.vue'
import WindowsPermissions from './components/windows-permissions/index.vue'

const generalStore = useGeneralStore()
const testingEnabled = ref(false)
const reportExportRef = ref<InstanceType<typeof ReportExport> | null>(null)

onMounted(async () => {
  try {
    testingEnabled.value = await analyticsService.getTestingMode()
  } catch (e) {
    console.warn('Failed to load testing mode:', e)
  }
})

async function handleTestingToggle(checked: boolean) {
  try {
    await analyticsService.setTestingMode(checked)
    testingEnabled.value = checked
    message.success(checked ? 'User testing mode enabled' : 'User testing mode disabled')
  } catch (e: any) {
    message.error(`Failed to toggle testing mode: ${e}`)
  }
}

watch(() => generalStore.app.autostart, async (value) => {
  const enabled = await isEnabled()

  if (value && !enabled) {
    return enable()
  }

  if (!value && enabled) {
    disable()
  }
}, { immediate: true })
</script>

<template>
  <MacosPermissions v-if="isMac" />

  <WindowsPermissions v-if="isWindows" />

  <ProList :title="$t('pages.preference.general.labels.appSettings')">
    <ProListItem :title="$t('pages.preference.general.labels.launchOnStartup')">
      <Switch v-model:checked="generalStore.app.autostart" />
    </ProListItem>

    <ProListItem
      :description="$t('pages.preference.general.hints.showTaskbarIcon')"
      :title="$t('pages.preference.general.labels.showTaskbarIcon')"
    >
      <Switch v-model:checked="generalStore.app.taskbarVisible" />
    </ProListItem>

    <ProListItem
      :description="$t('pages.preference.general.hints.showTrayIcon')"
      :title="$t('pages.preference.general.labels.showTrayIcon')"
    >
      <Switch v-model:checked="generalStore.app.trayVisible" />
    </ProListItem>
  </ProList>

  <ProList :title="$t('pages.preference.general.labels.appearanceSettings')">
    <ThemeMode />

    <Language />
  </ProList>

  <ProList :title="$t('pages.preference.general.labels.updateSettings')">
    <ProListItem :title="$t('pages.preference.general.labels.autoCheckUpdate')">
      <Switch v-model:checked="generalStore.update.autoCheck" />
    </ProListItem>
  </ProList>

  <ProList :title="$t('pages.preference.general.labels.testingSettings')">
    <ProListItem
      :description="$t('pages.preference.general.hints.testingModeHint')"
      :title="$t('pages.preference.general.labels.testingMode')"
    >
      <Switch :checked="testingEnabled" @change="handleTestingToggle" />
    </ProListItem>
    <ProListItem :title="$t('pages.preference.general.labels.exportReport')">
      <a @click="reportExportRef?.open()">{{ $t('pages.preference.general.labels.export') }}</a>
    </ProListItem>
  </ProList>

  <ReportExport ref="reportExportRef" />
</template>
