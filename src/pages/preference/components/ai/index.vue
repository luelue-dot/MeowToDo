<script setup lang="ts">
import { Input, message, Select, Spin } from 'antdv-next'
import { computed, onMounted, ref } from 'vue'

import ProListItem from '@/components/pro-list-item/index.vue'
import ProList from '@/components/pro-list/index.vue'
import { aiService, type AiSettings, type SaveAiSettingsRequest } from '@/services/ai'

const loading = ref(false)
const saving = ref(false)
const testing = ref(false)
const testResult = ref('')
const testInput = ref('你好，请介绍自己。')

const settings = ref<AiSettings>({
  id: 0,
  provider: 'ollama',
  model_name: 'qwen2.5:7b',
  api_key: '',
  base_url: '',
  is_active: true,
})

const providerOptions = [
  { value: 'openai', label: 'OpenAI' },
  { value: 'deepseek', label: 'DeepSeek' },
  { value: 'ollama', label: 'Ollama' },
]

// 模型选择下拉框选项 — 各 provider 的常用模型
const modelSelectOptions = [
  { provider: 'openai', label: 'gpt-4o-mini（OpenAI）', model: 'gpt-4o-mini' },
  { provider: 'openai', label: 'gpt-4o（OpenAI）', model: 'gpt-4o' },
  { provider: 'deepseek', label: 'deepseek-chat（DeepSeek）', model: 'deepseek-chat' },
  { provider: 'deepseek', label: 'deepseek-reasoner（DeepSeek）', model: 'deepseek-reasoner' },
  { provider: 'ollama', label: 'qwen2.5:7b（Ollama）', model: 'qwen2.5:7b' },
  { provider: 'ollama', label: 'llama3:8b（Ollama）', model: 'llama3:8b' },
  { provider: 'ollama', label: 'deepseek-r1:7b（Ollama）', model: 'deepseek-r1:7b' },
]

// Default config presets for each provider
const providerPresets: Record<string, { model_name: string; base_url: string }> = {
  openai: {
    model_name: 'gpt-4o-mini',
    base_url: 'https://api.openai.com/v1',
  },
  deepseek: {
    model_name: 'deepseek-chat',
    base_url: 'https://api.deepseek.com/v1',
  },
  ollama: {
    model_name: 'qwen2.5:7b',
    base_url: 'http://localhost:11434',
  },
}

function onProviderChange(provider: string) {
  const preset = providerPresets[provider]
  const oldProvider = settings.value.provider
  settings.value.provider = provider
  if (preset) {
    if (!settings.value.model_name || settings.value.model_name === providerPresets[oldProvider]?.model_name) {
      settings.value.model_name = preset.model_name
    }
    if (!settings.value.base_url || settings.value.base_url === providerPresets[oldProvider]?.base_url) {
      settings.value.base_url = preset.base_url
    }
  }
}

function onTestModelChange(modelName: string) {
  // 根据选中的模型名找到对应的 provider
  const option = modelSelectOptions.find(o => o.model === modelName)
  if (option && option.provider !== settings.value.provider) {
    // 切换 provider
    settings.value.provider = option.provider
    const preset = providerPresets[option.provider]
    if (preset) {
      settings.value.base_url = preset.base_url
    }
  }
  settings.value.model_name = modelName
}

async function loadSettings() {
  loading.value = true
  try {
    const result = await aiService.getSettings()
    settings.value = result
  } catch (e: any) {
    console.warn('Failed to load AI settings:', e)
  } finally {
    loading.value = false
  }
}

async function handleSave() {
  if (!settings.value.provider) {
    message.warning('请选择 Provider')
    return
  }
  if (!settings.value.model_name) {
    message.warning('请输入模型名称')
    return
  }
  if (settings.value.provider !== 'ollama' && !settings.value.api_key) {
    message.warning('请输入 API Key')
    return
  }

  saving.value = true
  try {
    const request: SaveAiSettingsRequest = {
      provider: settings.value.provider,
      model_name: settings.value.model_name,
      api_key: settings.value.api_key,
      base_url: settings.value.base_url,
    }
    const result = await aiService.saveSettings(request)
    settings.value = result
    message.success('AI 配置已保存')
  } catch (e: any) {
    message.error('保存失败：' + (e?.toString() || '未知错误'))
  } finally {
    saving.value = false
  }
}

async function handleTest() {
  if (!settings.value.provider || !settings.value.model_name) {
    message.warning('请先完成 AI 服务配置并保存')
    return
  }
  if (settings.value.provider !== 'ollama' && !settings.value.api_key) {
    message.warning('请输入 API Key 并保存配置')
    return
  }

  testing.value = true
  testResult.value = ''
  try {
    const request: SaveAiSettingsRequest = {
      provider: settings.value.provider,
      model_name: settings.value.model_name,
      api_key: settings.value.api_key,
      base_url: settings.value.base_url,
    }
    await aiService.saveSettings(request)
    const content = await aiService.generateText(testInput.value)
    testResult.value = content
    message.success('连接测试成功')
  } catch (e: any) {
    testResult.value = e?.toString() || '未知错误'
    message.error('连接测试失败')
  } finally {
    testing.value = false
  }
}

const isApiKeyDisabled = computed(() => settings.value.provider === 'ollama')

onMounted(() => {
  loadSettings()
})
</script>

<template>
  <ProList title="AI 服务配置">
    <ProListItem title="Provider">
      <Select
        :value="settings.provider"
        style="width: 200px"
        :options="providerOptions"
        @update:value="onProviderChange"
      />
    </ProListItem>

    <ProListItem title="Model Name">
      <Input
        :value="settings.model_name"
        placeholder="例如: gpt-4o-mini, deepseek-chat, qwen2.5:7b"
        style="width: 300px"
        @input="(e: any) => settings.model_name = e.target.value"
      />
    </ProListItem>

    <ProListItem title="API Key">
      <Input.Password
        :value="settings.api_key"
        :disabled="isApiKeyDisabled"
        :placeholder="isApiKeyDisabled ? 'Ollama 无需 API Key' : 'sk-...'"
        style="width: 350px"
        @input="(e: any) => settings.api_key = e.target.value"
      />
    </ProListItem>

    <ProListItem title="Base URL">
      <Input
        :value="settings.base_url"
        placeholder="例如: https://api.openai.com/v1"
        style="width: 350px"
        @input="(e: any) => settings.base_url = e.target.value"
      />
    </ProListItem>

    <ProListItem>
      <div class="flex gap-2">
        <button
          class="px-4 py-1.5 bg-primary text-white rounded-lg text-sm hover:opacity-80 transition disabled:opacity-40"
          :disabled="saving"
          @click="handleSave"
        >
          <Spin v-if="saving" class="w-3.5 h-3.5 mr-1" />
          {{ saving ? '保存中...' : '保存配置' }}
        </button>
      </div>
    </ProListItem>
  </ProList>

  <ProList title="AI 测试">
    <ProListItem title="模型选择">
      <div class="flex items-center gap-2 w-full">
        <Select
          :value="settings.model_name"
          style="width: 300px"
          @update:value="onTestModelChange"
        >
          <Select.Option
            v-for="opt in modelSelectOptions"
            :key="opt.model"
            :value="opt.model"
          >
            {{ opt.label }}
          </Select.Option>
        </Select>
        <span class="text-xs text-gray-4">当前 Provider：{{ settings.provider }}</span>
      </div>
    </ProListItem>

    <ProListItem title="测试输入">
      <Input
        :value="testInput"
        style="width: 350px"
        @input="(e: any) => testInput = e.target.value"
      />
    </ProListItem>

    <ProListItem>
      <div class="flex gap-2 items-center">
        <button
          class="px-4 py-1.5 bg-green-5 text-black rounded-lg text-sm hover:opacity-80 transition disabled:opacity-40 flex items-center gap-1"
          :disabled="testing || !settings.model_name"
          @click="handleTest"
        >
          <Spin v-if="testing" class="w-3.5 h-3.5" />
          <span>{{ testing ? '请求中...' : 'AI 测试' }}</span>
        </button>
        <span
          v-if="!settings.model_name"
          class="text-xs text-red-5"
        >⚠ 尚未配置 AI 模型</span>
      </div>
    </ProListItem>

    <ProListItem
      v-if="testResult"
      title="测试结果"
    >
      <div
        class="w-full p-3 bg-white rounded-lg text-sm whitespace-pre-wrap max-h-72 overflow-y-auto border"
        :class="testResult.startsWith('错误') || testResult.startsWith('Failed') || testResult.startsWith('error') ? 'text-red-6' : 'text-gray-8'"
      >
        {{ testResult }}
      </div>
    </ProListItem>
  </ProList>
</template>