<script setup lang="ts">
import { ref, watch } from "vue";
import AppHeader from "./AppHeader.vue";
import ActivityBar from "./ActivityBar.vue";
import AppSidebar from "./AppSidebar.vue";
import TabBar from "../tabs/TabBar.vue";
import { useSettingsStore } from "../../stores/settings";

const settingsStore = useSettingsStore();

const isResizing = ref(false);
const startX = ref(0);
const startWidth = ref(0);
const dpr = () => window.devicePixelRatio || 1;
const snapToPhysicalPixel = (width: number) =>
  Math.round(width * dpr()) / dpr();

const currentWidth = ref(snapToPhysicalPixel(settingsStore.sidebarWidth));

// Sync actual width after settings finish async loading
watch(
  () => settingsStore.sidebarWidth,
  (newWidth) => {
    if (!isResizing.value) {
      currentWidth.value = snapToPhysicalPixel(newWidth);
    }
  },
);

function startResize(e: MouseEvent) {
  e.preventDefault();
  isResizing.value = true;
  startX.value = e.clientX;
  startWidth.value = currentWidth.value;
  document.addEventListener("mousemove", onResize);
  document.addEventListener("mouseup", stopResize);
}

function onResize(e: MouseEvent) {
  if (!isResizing.value) return;
  e.preventDefault();
  const delta = e.clientX - startX.value;
  const newWidth = startWidth.value + delta;
  const clamped = Math.min(
    window.innerWidth * 0.7,
    Math.max(window.innerWidth * 0.1, newWidth),
  );
  // Snap to physical pixel boundary to avoid subpixel rendering on high DPI screens (125%/150% scale)
  currentWidth.value = snapToPhysicalPixel(clamped);
}

async function stopResize() {
  isResizing.value = false;
  document.removeEventListener("mousemove", onResize);
  document.removeEventListener("mouseup", stopResize);
  settingsStore.setSidebarWidth(currentWidth.value);
}
</script>

<template>
  <div class="h-screen flex flex-col bg-surface-deep text-text-primary">
    <AppHeader />
    <div class="flex flex-1 overflow-hidden">
      <!-- Activity Bar (VSCode style) -->
      <ActivityBar />

      <!-- Sidebar Panel (shows content for active activity) -->
      <div
        v-if="settingsStore.activeActivity"
        class="flex-shrink-0 overflow-hidden"
        :style="{ width: currentWidth + 'px' }"
      >
        <AppSidebar />
      </div>

      <!-- Resize Handle -->
      <div
        v-if="settingsStore.activeActivity"
        class="relative z-10 select-none cursor-col-resize"
        style="width: 1px; background: transparent"
        @mousedown="startResize"
      >
        <!-- Visual line (very thin) -->
        <div
          class="absolute left-0 top-0 bottom-0 w-px transition-all duration-150"
          :style="
            isResizing
              ? 'background-color: rgba(0, 122, 204, 0.5); width: 2px'
              : 'background-color: var(--border-default)'
          "
        />

        <!-- Hover zone (transparent 20px wide area) -->
        <div
          class="absolute -left-2 -right-2 top-0 bottom-0"
          :class="{ 'cursor-col-resize': isResizing }"
        />
      </div>

      <!-- Main Content Area -->
      <div class="flex-1 flex flex-col overflow-hidden">
        <TabBar />
        <main class="flex-1 overflow-auto bg-surface-base">
          <slot />
        </main>
      </div>
    </div>
  </div>
</template>
