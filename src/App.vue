<script setup lang="ts">
import { ref, onMounted, onUnmounted, onErrorCaptured } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { FolderOpen, Save, CheckCircle, RefreshCcw, Sun, Moon } from "lucide-vue-next";

onErrorCaptured((err, instance, info) => {
  console.error("Vue Error Captured:", err, info);
  invoke("log_error", { msg: String(err) }).catch(() => {});
});

window.addEventListener('error', (event) => {
  invoke("log_error", { msg: String(event.error) }).catch(() => {});
});

window.addEventListener('unhandledrejection', (event) => {
  invoke("log_error", { msg: String(event.reason) }).catch(() => {});
});


interface AppSettings {
  folder_path: string;
  interval_minutes: number;
  mode: "Random" | "Sequential";
  autostart: boolean;
  theme: "system" | "light" | "dark";
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
  theme: "system",
});

const status = ref<AppStatus>({
  current_image: null,
  total_images: 0,
  is_paused: false,
  time_remaining: 0,
});

const isSaving = ref(false);
const effectiveTheme = ref<"light" | "dark">("light");
let statusInterval: number;
let mediaQuery: MediaQueryList | null = null;

function getSystemTheme(): "light" | "dark" {
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function getEffectiveTheme(): "light" | "dark" {
  const pref = settings.value.theme;
  if (pref === "light" || pref === "dark") return pref;
  return getSystemTheme();
}

function applyTheme() {
  const html = document.documentElement;
  if (getEffectiveTheme() === "dark") {
    html.classList.add("dark");
  } else {
    html.classList.remove("dark");
  }
  effectiveTheme.value = getEffectiveTheme();
}

function cycleTheme() {
  const current = settings.value.theme;
  if (current === "system") settings.value.theme = "light";
  else if (current === "light") settings.value.theme = "dark";
  else settings.value.theme = "system";
  applyTheme();
}

function onSystemThemeChange() {
  if (settings.value.theme === "system") {
    applyTheme();
  }
}

async function loadSettings() {
  try {
    const loadedSettings = await invoke<AppSettings>("get_settings");
    settings.value = loadedSettings;
    applyTheme();
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
  mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
  if (mediaQuery.addEventListener) {
    mediaQuery.addEventListener("change", onSystemThemeChange);
  }
  applyTheme();
});

onUnmounted(() => {
  if (statusInterval) {
    clearInterval(statusInterval);
  }
  if (mediaQuery && mediaQuery.removeEventListener) {
    mediaQuery.removeEventListener("change", onSystemThemeChange);
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
  <main class="h-screen bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100 p-6 flex flex-col font-sans">
    <header class="flex items-center justify-between mb-5">
      <h1 class="text-xl font-bold text-gray-900 dark:text-gray-100">ShufflePaper</h1>
      <button 
        @click="cycleTheme" 
        class="p-2 rounded-lg text-gray-600 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
        :title="(settings.theme || 'system') === 'system' ? 'System' : (settings.theme || 'system').charAt(0).toUpperCase() + (settings.theme || 'system').slice(1)"
      >
        <component :is="effectiveTheme === 'dark' ? Moon : Sun" class="w-5 h-5" />
      </button>
    </header>
    <div class="flex-grow space-y-5">
      
      <!-- Folder Section -->
      <section class="space-y-2">
        <label class="block text-sm font-semibold text-gray-700 dark:text-gray-300">Wallpaper Folder</label>
        <div class="flex gap-2">
          <input 
            type="text" 
            readonly
            :value="settings.folder_path" 
            placeholder="No folder selected" 
            class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm text-sm bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:outline-none cursor-default"
          />
          <button @click="selectFolder" class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-1 dark:focus:ring-offset-gray-900 flex items-center gap-2 text-sm font-medium transition-colors">
            <FolderOpen class="w-4 h-4" /> Browse
          </button>
        </div>
      </section>

      <!-- Interval Select -->
      <section class="space-y-2">
        <label class="block text-sm font-semibold text-gray-700 dark:text-gray-300">Change Interval</label>
        <select v-model="settings.interval_minutes" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm text-sm bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500">
          <option v-for="option in intervalOptions" :key="option.value" :value="option.value">
            {{ option.label }}
          </option>
        </select>
      </section>

      <!-- Mode Radio -->
      <section class="space-y-2">
        <label class="block text-sm font-semibold text-gray-700 dark:text-gray-300">Order</label>
        <div class="flex gap-6 mt-1">
          <label class="flex items-center gap-2 cursor-pointer">
            <input type="radio" value="Random" v-model="settings.mode" class="text-blue-600 focus:ring-blue-500 w-4 h-4 border-gray-300 dark:border-gray-600" />
            <span class="text-sm text-gray-800 dark:text-gray-200">Random</span>
          </label>
          <label class="flex items-center gap-2 cursor-pointer">
            <input type="radio" value="Sequential" v-model="settings.mode" class="text-blue-600 focus:ring-blue-500 w-4 h-4 border-gray-300 dark:border-gray-600" />
            <span class="text-sm text-gray-800 dark:text-gray-200">Sequential</span>
          </label>
        </div>
      </section>

      <!-- Autostart Checkbox -->
      <section class="pt-1">
        <label class="flex items-center gap-2 cursor-pointer">
          <input type="checkbox" v-model="settings.autostart" class="rounded text-blue-600 focus:ring-blue-500 w-4 h-4 border-gray-300" />
          <span class="text-sm font-medium text-gray-800 dark:text-gray-200">Start with Windows</span>
        </label>
      </section>

      <!-- Status Area -->
      <section class="bg-white dark:bg-gray-800 p-4 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 mt-2 space-y-3">
        <h3 class="text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider">Status</h3>
        <div class="text-sm space-y-1">
          <p class="flex justify-between"><span class="font-medium text-gray-600 dark:text-gray-400">Images found:</span> <span>{{ status.total_images }}</span></p>
          <p class="flex justify-between items-center gap-4">
            <span class="font-medium text-gray-600 dark:text-gray-400 whitespace-nowrap">Current:</span> 
            <span class="truncate text-gray-800 dark:text-gray-200" :title="status.current_image || 'None'">{{ status.current_image ? status.current_image.split(/[\\/]/).pop() : 'None' }}</span>
          </p>
          <p class="flex justify-between pt-1">
            <span class="font-medium text-gray-600 dark:text-gray-400">State:</span>
            <span class="text-amber-600 dark:text-amber-400 font-medium" v-if="status.is_paused">Paused</span>
            <span class="text-green-600 dark:text-green-400 font-medium" v-else>Active</span>
          </p>
        </div>
        <button @click="nextWallpaper" class="mt-4 w-full px-3 py-2 bg-gray-50 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded hover:bg-gray-100 dark:hover:bg-gray-600 border border-gray-300 dark:border-gray-600 flex items-center justify-center gap-2 text-sm font-medium transition-colors">
          <RefreshCcw class="w-4 h-4" /> Skip to Next
        </button>
      </section>

    </div>

    <!-- Save Button -->
    <div class="pt-4 border-t border-gray-200 dark:border-gray-700 mt-auto">
      <button 
        @click="saveSettings" 
        :disabled="isSaving"
        class="w-full py-3 bg-gray-900 dark:bg-gray-100 text-white dark:text-gray-900 rounded-md hover:bg-gray-800 dark:hover:bg-gray-200 focus:outline-none focus:ring-2 focus:ring-gray-900 dark:focus:ring-gray-100 focus:ring-offset-2 dark:focus:ring-offset-gray-900 flex items-center justify-center gap-2 font-medium transition-all"
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
