<script setup lang="ts">
import { ref, onMounted, onUnmounted, onErrorCaptured } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { FolderOpen, Save, CheckCircle, Undo2, Redo2, Sun, Moon, X, Info } from "lucide-vue-next";

interface AppSettings {
  folder_path: string;
  interval_minutes: number;
  mode: "Random" | "Sequential";
  autostart: boolean;
  theme: "system" | "light" | "dark";
  pause_on_fullscreen: boolean;
  context_rules_enabled: boolean;
  work_folder: string | null;
  personal_folder: string | null;
  work_days: boolean[];
  work_start_hour: number;
  work_end_hour: number;
  force_mode: "Work" | "Personal" | null;
}

interface AppStatus {
  current_image: string | null;
  total_images: number;
  is_paused: boolean;
  paused_fullscreen: boolean;
  time_remaining: number;
  can_previous: boolean;
  active_profile: string;
  last_error: string | null;
}

const defaults: AppSettings = {
  folder_path: "",
  interval_minutes: 15,
  mode: "Random",
  autostart: false,
  theme: "system",
  pause_on_fullscreen: true,
  context_rules_enabled: false,
  work_folder: null,
  personal_folder: null,
  work_days: [true, true, true, true, true, false, false],
  work_start_hour: 9,
  work_end_hour: 18,
  force_mode: null,
};

const settings = ref<AppSettings>({ ...defaults });
const status = ref<AppStatus>({
  current_image: null,
  total_images: 0,
  is_paused: false,
  paused_fullscreen: false,
  time_remaining: 0,
  can_previous: false,
  active_profile: "Automatic",
  last_error: null,
});
const isSaving = ref(false);
const logPath = ref("");
const effectiveTheme = ref<"light" | "dark">("light");
const mainElement = ref<HTMLElement | null>(null);
const activeInfoTip = ref<"fullscreen" | "context" | null>(null);
const infoTooltipPosition = ref({ left: "8px", top: "8px" });
let infoHideTimer: number | undefined;
let statusInterval: number | undefined;
let mediaQuery: MediaQueryList | null = null;

onErrorCaptured((err) => {
  console.error("Vue Error Captured:", err);
  invoke("log_error", { msg: String(err) }).catch(() => {});
});
window.addEventListener("error", (event) => invoke("log_error", { msg: String(event.error) }).catch(() => {}));
window.addEventListener("unhandledrejection", (event) => invoke("log_error", { msg: String(event.reason) }).catch(() => {}));
window.addEventListener("contextmenu", (event) => event.preventDefault());

function getSystemTheme(): "light" | "dark" {
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function applyTheme() {
  const theme = settings.value.theme === "dark" || (settings.value.theme === "system" && getSystemTheme()) ? "dark" : "light";
  document.documentElement.classList.toggle("dark", theme === "dark");
  effectiveTheme.value = theme;
}

function cycleTheme() {
  settings.value.theme = settings.value.theme === "system" ? "light" : settings.value.theme === "light" ? "dark" : "system";
  applyTheme();
}

function onSystemThemeChange() {
  if (settings.value.theme === "system") applyTheme();
}

async function selectFolder(target: "folder_path" | "work_folder" | "personal_folder" = "folder_path") {
  try {
    const folder = await invoke<string | null>("select_folder");
    if (folder) settings.value[target] = folder;
  } catch (error) {
    console.error("Failed to select folder:", error);
  }
}

async function loadSettings() {
  try {
    settings.value = { ...defaults, ...(await invoke<AppSettings>("get_settings")) };
    applyTheme();
  } catch (error) {
    console.error("Failed to load settings:", error);
  }
}

async function loadStatus() {
  try {
    status.value = await invoke<AppStatus>("get_status");
  } catch (error) {
    console.error("Failed to load status:", error);
  }
}

async function saveSettings() {
  isSaving.value = true;
  try {
    await invoke("save_settings", { newSettings: settings.value });
    await loadStatus();
  } catch (error) {
    console.error("Failed to save settings:", error);
  } finally {
    window.setTimeout(() => (isSaving.value = false), 1200);
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

async function previousWallpaper() {
  try {
    await invoke("previous_wallpaper");
    await loadStatus();
  } catch (error) {
    console.error("Failed to restore previous wallpaper:", error);
  }
}

async function setForceMode(mode: "Work" | "Personal" | null) {
  try {
    await invoke("set_force_mode", { mode });
    settings.value.force_mode = mode;
    await loadStatus();
  } catch (error) {
    console.error("Failed to set profile mode:", error);
  }
}

async function openLogs() {
  try {
    logPath.value = await invoke<string>("open_log_file");
  } catch (error) {
    console.error("Failed to open log file:", error);
  }
}

function hideWindow() {
  invoke("hide_window").catch(console.error);
}

function startDrag() {
  invoke("start_drag").catch(console.error);
}

function showInfoTip(type: "fullscreen" | "context", event: MouseEvent | FocusEvent) {
  if (infoHideTimer) window.clearTimeout(infoHideTimer);
  const target = event.currentTarget as HTMLElement;
  const rect = target.getBoundingClientRect();
  const tooltipWidth = Math.min(240, window.innerWidth - 16);
  const tooltipHeight = 72;
  const left = Math.max(8, Math.min(rect.left, window.innerWidth - tooltipWidth - 8));
  const top = rect.bottom + 8 + tooltipHeight <= window.innerHeight
    ? rect.bottom + 8
    : Math.max(8, rect.top - tooltipHeight - 8);
  infoTooltipPosition.value = { left: `${left}px`, top: `${top}px` };
  activeInfoTip.value = type;
}

function hideInfoTip() {
  infoHideTimer = window.setTimeout(() => {
    activeInfoTip.value = null;
  }, 80);
}

onMounted(async () => {
  await loadSettings();
  await loadStatus();
  statusInterval = window.setInterval(loadStatus, 2000);
  mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
  mediaQuery.addEventListener?.("change", onSystemThemeChange);
});

onUnmounted(() => {
  if (statusInterval) clearInterval(statusInterval);
  mediaQuery?.removeEventListener?.("change", onSystemThemeChange);
});

const intervalOptions = [
  { label: "1 Minute", value: 1 },
  { label: "5 Minutes", value: 5 },
  { label: "15 Minutes", value: 15 },
  { label: "30 Minutes", value: 30 },
  { label: "1 Hour", value: 60 },
  { label: "4 Hours", value: 240 },
  { label: "24 Hours", value: 1440 },
];
const days = ["M", "T", "W", "T", "F", "S", "S"];
</script>

<template>
  <main ref="mainElement" class="h-[800px] min-h-[800px] max-h-[800px] w-full box-border bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100 p-6 flex flex-col font-sans overflow-hidden border-2 border-gray-200 dark:border-gray-700 rounded-xl shadow-2xl">
    <header @mousedown="startDrag" class="flex shrink-0 items-center justify-between mb-5 cursor-move">
      <h1 class="text-xl font-bold pointer-events-none">ShufflePaper</h1>
      <div class="flex items-center gap-2 z-10" @mousedown.stop>
        <button @click="cycleTheme" class="p-2 rounded-lg text-gray-600 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700" title="Theme"><component :is="effectiveTheme === 'dark' ? Moon : Sun" class="w-5 h-5" /></button>
        <button @click="hideWindow" class="p-2 rounded-lg text-gray-600 dark:text-gray-300 hover:bg-red-500 hover:text-white" title="Close to tray"><X class="w-5 h-5" /></button>
      </div>
    </header>

    <div class="settings-scroll min-h-0 flex-1 overflow-y-auto space-y-5 pr-1">
      <section class="space-y-2">
        <label class="block text-sm font-semibold">Wallpaper Folder</label>
        <div class="flex gap-2"><input type="text" readonly :value="settings.folder_path" placeholder="No folder selected" class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm bg-white dark:bg-gray-800" /><button @click="selectFolder()" class="px-4 py-2 bg-blue-600 text-white rounded-md flex items-center gap-2 text-sm font-medium"><FolderOpen class="w-4 h-4" /> Browse</button></div>
      </section>

      <section class="space-y-2"><label class="block text-sm font-semibold">Change Interval</label><select v-model="settings.interval_minutes" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm bg-white dark:bg-gray-800"><option v-for="option in intervalOptions" :key="option.value" :value="option.value">{{ option.label }}</option></select></section>
      <section class="space-y-2"><label class="block text-sm font-semibold">Order</label><div class="flex gap-6"><label><input type="radio" value="Random" v-model="settings.mode" /> Random</label><label><input type="radio" value="Sequential" v-model="settings.mode" /> Sequential</label></div></section>

      <section class="space-y-3 border-t border-gray-200 dark:border-gray-700 pt-4">
        <label class="flex items-center gap-2 cursor-pointer"><input type="checkbox" v-model="settings.pause_on_fullscreen" class="rounded text-blue-600" /><span class="text-sm font-medium">Pause on fullscreen apps</span><button type="button" class="info-tip" aria-label="Pause on fullscreen apps information" @click.stop @mouseenter="showInfoTip('fullscreen', $event)" @mouseleave="hideInfoTip" @focus="showInfoTip('fullscreen', $event)" @blur="hideInfoTip"><Info class="h-3.5 w-3.5" /></button></label>
      </section>

      <section class="bg-white dark:bg-gray-800 p-4 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 space-y-3">
        <h3 class="text-xs font-bold text-gray-500 uppercase tracking-wider">Status</h3>
        <div class="text-sm space-y-1">
          <p class="flex justify-between"><span class="font-medium text-gray-600 dark:text-gray-400">Images found:</span><span>{{ status.total_images }}</span></p>
          <p class="flex justify-between items-center gap-4"><span class="font-medium text-gray-600 dark:text-gray-400">Current:</span><span class="truncate" :title="status.current_image || 'None'">{{ status.current_image ? status.current_image.split(/[\\/]/).pop() : 'None' }}</span></p>
          <p class="flex justify-between"><span class="font-medium text-gray-600 dark:text-gray-400">Profile:</span><span>{{ status.active_profile }}</span></p>
          <p class="flex justify-between"><span class="font-medium text-gray-600 dark:text-gray-400">State:</span><span v-if="status.paused_fullscreen" class="text-amber-600 font-medium">Paused (fullscreen)</span><span v-else-if="status.is_paused" class="text-amber-600 font-medium">Paused</span><span v-else class="text-green-600 font-medium">Active</span></p>
        </div>
        <p v-if="status.last_error" class="flex flex-col gap-0.5 rounded border border-red-300 bg-red-50 px-2 py-1.5 text-xs text-red-700 dark:border-red-800 dark:bg-red-950/40 dark:text-red-300">
          <span class="font-semibold uppercase tracking-wider text-[10px]">Last wallpaper error</span>
          <span class="break-all">{{ status.last_error }}</span>
        </p>
        <div class="mt-4 flex gap-2"><button @click="previousWallpaper" :disabled="!status.can_previous" class="flex-1 px-3 py-2 bg-gray-50 dark:bg-gray-700 rounded border text-sm font-medium disabled:opacity-40 flex items-center justify-center gap-2"><Undo2 class="w-4 h-4" /> Previous</button><button @click="nextWallpaper" class="flex-1 px-3 py-2 bg-gray-50 dark:bg-gray-700 rounded border text-sm font-medium flex items-center justify-center gap-2"><Redo2 class="w-4 h-4" /> Skip to Next</button></div>
        <button @click="openLogs" :title="logPath || 'Open the application log file'" class="mt-2 w-full text-xs text-gray-500 dark:text-gray-400 underline hover:text-blue-600 dark:hover:text-blue-400">Open log file</button>
      </section>

      <section class="space-y-3 border-t border-gray-200 dark:border-gray-700 pt-4">
        <label class="flex items-center gap-2 cursor-pointer"><input type="checkbox" v-model="settings.context_rules_enabled" class="rounded text-blue-600" /><span class="text-sm font-medium">Context rules</span><button type="button" class="info-tip" aria-label="Context rules information" @click.stop @mouseenter="showInfoTip('context', $event)" @mouseleave="hideInfoTip" @focus="showInfoTip('context', $event)" @blur="hideInfoTip"><Info class="h-3.5 w-3.5" /></button></label>
        <div v-if="settings.context_rules_enabled" class="space-y-3 pl-1">
          <div><label class="block text-xs font-semibold mb-1">Work folder</label><div class="flex gap-2"><input readonly :value="settings.work_folder || 'Not configured'" class="flex-1 px-2 py-1.5 border rounded text-xs bg-white dark:bg-gray-800" /><button @click="selectFolder('work_folder')" class="px-2 py-1 bg-blue-600 text-white rounded text-xs"><FolderOpen class="w-3 h-3" /></button></div></div>
          <div><label class="block text-xs font-semibold mb-1">Personal folder</label><div class="flex gap-2"><input readonly :value="settings.personal_folder || 'Not configured'" class="flex-1 px-2 py-1.5 border rounded text-xs bg-white dark:bg-gray-800" /><button @click="selectFolder('personal_folder')" class="px-2 py-1 bg-blue-600 text-white rounded text-xs"><FolderOpen class="w-3 h-3" /></button></div></div>
          <div><label class="block text-xs font-semibold mb-1">Work days</label><div class="flex gap-1"><label v-for="(day, index) in days" :key="`${day}-${index}`" class="flex-1 text-center text-xs"><input type="checkbox" v-model="settings.work_days[index]" class="block mx-auto" />{{ day }}</label></div></div>
          <div class="flex items-center gap-2 text-sm"><label>Working hours</label><select v-model.number="settings.work_start_hour" class="px-2 py-1 border rounded bg-white dark:bg-gray-800"><option v-for="hour in 24" :key="`start-${hour}`" :value="hour - 1">{{ String(hour - 1).padStart(2, '0') }}:00</option></select><span>–</span><select v-model.number="settings.work_end_hour" class="px-2 py-1 border rounded bg-white dark:bg-gray-800"><option v-for="hour in 24" :key="`end-${hour}`" :value="hour - 1">{{ String(hour - 1).padStart(2, '0') }}:00</option></select></div>
          <div><label class="block text-xs font-semibold mb-1">Mode</label><select :value="settings.force_mode || 'Automatic'" @change="setForceMode(($event.target as HTMLSelectElement).value === 'Automatic' ? null : ($event.target as HTMLSelectElement).value as 'Work' | 'Personal')" class="w-full px-2 py-1.5 border rounded text-sm bg-white dark:bg-gray-800"><option value="Automatic">Automatic</option><option value="Work">Force Work</option><option value="Personal">Force Personal</option></select></div>
        </div>
      </section>

      <section class="pt-1"><label class="flex items-center gap-2 cursor-pointer"><input type="checkbox" v-model="settings.autostart" class="rounded text-blue-600" /><span class="text-sm font-medium">Start with Windows</span></label></section>
    </div>

    <footer class="shrink-0 pt-4 border-t border-gray-200 dark:border-gray-700 mt-4">
      <button @click="saveSettings" :disabled="isSaving" class="w-full py-3 bg-gray-900 dark:bg-gray-100 text-white dark:text-gray-900 rounded-md flex items-center justify-center gap-2 font-medium disabled:opacity-90"><template v-if="!isSaving"><Save class="w-5 h-5" /> Save Configuration</template><template v-else><CheckCircle class="w-5 h-5 text-green-400" /> Saved!</template></button>
    </footer>
  </main>

  <Teleport to="body">
    <div v-if="activeInfoTip" class="info-tooltip" role="tooltip" :style="infoTooltipPosition">
      {{ activeInfoTip === "fullscreen" ? "Automatically pauses wallpaper changes while a game, video, or presentation is fullscreen." : "Uses the Work folder during working hours and the Personal folder at other times." }}
    </div>
  </Teleport>
</template>

<style>
.settings-scroll {
  scrollbar-width: thin;
  scrollbar-color: #cbd5e1 transparent;
  scrollbar-gutter: stable;
}

.info-tip {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  color: #64748b;
  border: 1px solid currentColor;
  border-radius: 9999px;
  cursor: help;
  flex: 0 0 auto;
}

.info-tip:hover,
.info-tip:focus-visible {
  color: #2563eb;
  outline: none;
}

.info-tooltip {
  position: fixed;
  z-index: 100;
  width: min(240px, calc(100vw - 16px));
  padding: 8px 10px;
  color: #334155;
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  box-shadow: 0 8px 20px rgb(15 23 42 / 12%);
  font-size: 11px;
  line-height: 1.35;
  font-weight: 400;
  pointer-events: none;
}

.dark .info-tooltip {
  color: #e2e8f0;
  background: #1e293b;
  border-color: #475569;
  box-shadow: 0 8px 20px rgb(0 0 0 / 25%);
}

.settings-scroll::-webkit-scrollbar {
  width: 8px;
}

.settings-scroll::-webkit-scrollbar-track {
  background: transparent;
  margin: 4px 0;
}

.settings-scroll::-webkit-scrollbar-thumb {
  background: #cbd5e1;
  border: 2px solid transparent;
  border-radius: 9999px;
  background-clip: padding-box;
}

.settings-scroll::-webkit-scrollbar-thumb:hover {
  background: #94a3b8;
  border: 1px solid transparent;
  background-clip: padding-box;
}

.dark .settings-scroll {
  scrollbar-color: #475569 transparent;
}

.dark .settings-scroll::-webkit-scrollbar-thumb {
  background: #475569;
  border-color: transparent;
}

.dark .settings-scroll::-webkit-scrollbar-thumb:hover {
  background: #64748b;
  border-color: transparent;
}
</style>
