<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { FolderOpen, Save, CheckCircle, RefreshCcw } from "lucide-vue-next";

// Types matching Rust structs
interface AppSettings {
  folder_path: string;
  interval_minutes: number;
  mode: "Random" | "Sequential";
  autostart: boolean;
}

interface AppStatus {
  current_image: string | null;
  total_images: number;
  is_paused: boolean;
  time_remaining: number;
}

const settings = ref<AppSettings>({
  folder_path: "",
  interval_minutes: 15,
  mode: "Random",
  autostart: false,
});

const status = ref<AppStatus>({
  current_image: null,
  total_images: 0,
  is_paused: false,
  time_remaining: 0,
});

const isSaving = ref(false);
let statusInterval: number;

async function loadSettings() {
  try {
    const loadedSettings = await invoke<AppSettings>("get_settings");
    settings.value = loadedSettings;
  } catch (error) {
    console.error("Failed to load settings:", error);
  }
}

async function loadStatus() {
  try {
    const loadedStatus = await invoke<AppStatus>("get_status");
    status.value = loadedStatus;
  } catch (error) {
    console.error("Failed to load status:", error);
  }
}

async function selectFolder() {
  try {
    const folder = await invoke<string | null>("select_folder");
    if (folder) {
      settings.value.folder_path = folder;
    }
  } catch (error) {
    console.error("Failed to select folder:", error);
  }
}

async function saveSettings() {
  isSaving.value = true;
  try {
    await invoke("save_settings", { newSettings: settings.value });
    await loadStatus(); // Reload status as total images might have changed
  } catch (error) {
    console.error("Failed to save settings:", error);
  } finally {
    setTimeout(() => {
      isSaving.value = false;
    }, 1500);
  }
}

async function nextWallpaper() {
  try {
    await invoke("next_wallpaper");
    await loadStatus();
  } catch (error) {
    console.error("Failed to change wallpaper:", error);
  }
}

onMounted(async () => {
  await loadSettings();
  await loadStatus();
  statusInterval = window.setInterval(loadStatus, 2000) as unknown as number;
});

onUnmounted(() => {
  if (statusInterval) {
    clearInterval(statusInterval);
  }
});

const intervalOptions = [
  { label: '1 Minute', value: 1 },
  { label: '5 Minutes', value: 5 },
  { label: '15 Minutes', value: 15 },
  { label: '30 Minutes', value: 30 },
  { label: '1 Hour', value: 60 },
  { label: '4 Hours', value: 240 },
  { label: '24 Hours', value: 1440 },
];
</script>

<template>
  <main class="h-screen bg-gray-50 text-gray-900 p-6 flex flex-col font-sans">
    <div class="flex-grow space-y-5">
      
      <!-- Folder Section -->
      <section class="space-y-2">
        <label class="block text-sm font-semibold text-gray-700">Wallpaper Folder</label>
        <div class="flex gap-2">
          <input 
            type="text" 
            readonly
            :value="settings.folder_path" 
            placeholder="No folder selected" 
            class="flex-1 px-3 py-2 border border-gray-300 rounded-md shadow-sm text-sm bg-white focus:outline-none cursor-default"
          />
          <button @click="selectFolder" class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-1 flex items-center gap-2 text-sm font-medium transition-colors">
            <FolderOpen class="w-4 h-4" /> Browse
          </button>
        </div>
      </section>

      <!-- Interval Select -->
      <section class="space-y-2">
        <label class="block text-sm font-semibold text-gray-700">Change Interval</label>
        <select v-model="settings.interval_minutes" class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm text-sm bg-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500">
          <option v-for="option in intervalOptions" :key="option.value" :value="option.value">
            {{ option.label }}
          </option>
        </select>
      </section>

      <!-- Mode Radio -->
      <section class="space-y-2">
        <label class="block text-sm font-semibold text-gray-700">Order</label>
        <div class="flex gap-6 mt-1">
          <label class="flex items-center gap-2 cursor-pointer">
            <input type="radio" value="Random" v-model="settings.mode" class="text-blue-600 focus:ring-blue-500 w-4 h-4 border-gray-300" />
            <span class="text-sm text-gray-800">Random</span>
          </label>
          <label class="flex items-center gap-2 cursor-pointer">
            <input type="radio" value="Sequential" v-model="settings.mode" class="text-blue-600 focus:ring-blue-500 w-4 h-4 border-gray-300" />
            <span class="text-sm text-gray-800">Sequential</span>
          </label>
        </div>
      </section>

      <!-- Autostart Checkbox -->
      <section class="pt-1">
        <label class="flex items-center gap-2 cursor-pointer">
          <input type="checkbox" v-model="settings.autostart" class="rounded text-blue-600 focus:ring-blue-500 w-4 h-4 border-gray-300" />
          <span class="text-sm font-medium text-gray-800">Start with Windows</span>
        </label>
      </section>

      <!-- Status Area -->
      <section class="bg-white p-4 rounded-lg shadow-sm border border-gray-200 mt-2 space-y-3">
        <h3 class="text-xs font-bold text-gray-500 uppercase tracking-wider">Status</h3>
        <div class="text-sm space-y-1">
          <p class="flex justify-between"><span class="font-medium text-gray-600">Images found:</span> <span>{{ status.total_images }}</span></p>
          <p class="flex justify-between items-center gap-4">
            <span class="font-medium text-gray-600 whitespace-nowrap">Current:</span> 
            <span class="truncate text-gray-800" :title="status.current_image || 'None'">{{ status.current_image ? status.current_image.split(/[\\/]/).pop() : 'None' }}</span>
          </p>
          <p class="flex justify-between pt-1">
            <span class="font-medium text-gray-600">State:</span>
            <span class="text-amber-600 font-medium" v-if="status.is_paused">Paused</span>
            <span class="text-green-600 font-medium" v-else>Active</span>
          </p>
        </div>
        <button @click="nextWallpaper" class="mt-4 w-full px-3 py-2 bg-gray-50 text-gray-700 rounded hover:bg-gray-100 border border-gray-300 flex items-center justify-center gap-2 text-sm font-medium transition-colors">
          <RefreshCcw class="w-4 h-4" /> Skip to Next
        </button>
      </section>

    </div>

    <!-- Save Button -->
    <div class="pt-4 border-t border-gray-200 mt-auto">
      <button 
        @click="saveSettings" 
        :disabled="isSaving"
        class="w-full py-3 bg-gray-900 text-white rounded-md hover:bg-gray-800 focus:outline-none focus:ring-2 focus:ring-gray-900 focus:ring-offset-2 flex items-center justify-center gap-2 font-medium transition-all"
        :class="{ 'opacity-90 cursor-not-allowed': isSaving }"
      >
        <template v-if="!isSaving">
          <Save class="w-5 h-5" /> Save Configuration
        </template>
        <template v-else>
          <CheckCircle class="w-5 h-5 text-green-400" /> Saved!
        </template>
      </button>
    </div>
  </main>
</template>
