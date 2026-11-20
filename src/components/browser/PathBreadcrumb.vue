<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { ChevronDown, ChevronLeft, ChevronRight, Folder, HardDrive, Monitor } from 'lucide-vue-next'

const props = withDefaults(defineProps<{
  rootPath: string | null
  currentPath: string | null
  /** 每级目录名最多显示的「等效全角字数」，超出部分在该级内以 … 收尾 */
  maxLabelChars?: number
}>(), {
  maxLabelChars: 14,
})

const emit = defineEmits<{
  navigate: [path: string]
}>()

interface Segment {
  label: string
  path: string
}

/* ------------------------------------------------------------------ *
 * 路径解析：从文件系统根 / 盘符开始，保留完整层级。
 * 旧实现以 rootPath 为边界，把盘符和所有上级目录都丢掉了，
 * 导致刚打开目录时只显示一节、且没有可返回的父级。
 * ------------------------------------------------------------------ */

/** Windows canonicalize 会给出 `\\?\E:\...` 这样的 NT 扩展路径。
 * 后端已统一清理，这里再做一层兜底，避免历史路径或其它来源把它显示成
 * 「\\?\E:」甚至误判成 UNC 网络路径。 */
function normalizePath(raw: string): string {
  if (raw.startsWith('\\\\?\\UNC\\')) return '\\\\' + raw.slice(8)
  if (raw.startsWith('\\\\?\\')) return raw.slice(4)
  return raw
}

/** 盘符段（如 `E:\`）：用硬盘图标和普通目录区分 */
function isDriveSegment(seg: Segment): boolean {
  return /^[A-Za-z]:[\\/]$/.test(seg.path)
}

function buildSegments(raw: string): Segment[] {
  const normalized = normalizePath(raw)
  const p = normalized.replace(/[\\/]+$/, '')
  if (!p) return []

  // UNC：\\server\share\dir\...
  if (/^\\\\/.test(normalized)) {
    const parts = p.slice(2).split(/[\\/]+/).filter(Boolean)
    if (parts.length < 2) return [{ label: p, path: p }]
    let acc = `\\\\${parts[0]}\\${parts[1]}`
    const segs: Segment[] = [{ label: `\\\\${parts[0]}\\${parts[1]}`, path: acc }]
    for (const name of parts.slice(2)) {
      acc += '\\' + name
      segs.push({ label: name, path: acc })
    }
    return segs
  }

  // Windows 盘符：C:\Users\...\Album
  if (/^[A-Za-z]:[\\/]/.test(p)) {
    const drive = p.slice(0, 2)
    const sep = p.includes('\\') ? '\\' : '/'
    const root = drive + sep
    const parts = p.slice(2).split(/[\\/]+/).filter(Boolean)
    const segs: Segment[] = [{ label: drive, path: root }]
    let acc = root
    for (const name of parts) {
      acc = acc.endsWith(sep) ? acc + name : acc + sep + name
      segs.push({ label: name, path: acc })
    }
    return segs
  }

  // POSIX 绝对路径：/home/user/.../Album
  const parts = p.split(/[\\/]+/).filter(Boolean)
  const segs: Segment[] = [{ label: '/', path: '/' }]
  let acc = ''
  for (const name of parts) {
    acc += '/' + name
    segs.push({ label: name, path: acc })
  }
  return segs
}

const segments = computed<Segment[]>(() =>
  props.currentPath ? buildSegments(props.currentPath) : [],
)

/** 真实父目录：只要还没到盘符 / 文件系统根，就一直能返回上一级 */
const parentPath = computed<string | null>(() => {
  const segs = segments.value
  return segs.length >= 2 ? segs[segs.length - 2].path : null
})

/** 「打开目录」设定的图库根，在完整路径中标记出来，方便一键回去 */
const rootSegmentPath = computed(() => {
  const root = props.rootPath
    ? normalizePath(props.rootPath).replace(/[\\/]+$/, '').toLowerCase()
    : null
  if (!root) return null
  return segments.value.find(
    s => s.path.replace(/[\\/]+$/, '').toLowerCase() === root,
  )?.path ?? null
})

function isRootSegment(seg: Segment) {
  return rootSegmentPath.value !== null && seg.path === rootSegmentPath.value
}

/* ------------------------------------------------------------------ *
 * 名称截断：全角 / CJK 记 1 个字，半角 ASCII 记 0.55 个字，
 * 中英混排时视觉宽度才一致。超出 N 的部分在本级内用 … 收尾，
 * 完整名称始终可通过 title 与下拉菜单查看。
 * ------------------------------------------------------------------ */
const WIDE_CHAR = /[\u1100-\u115F\u2E80-\u303E\u3041-\u33FF\u3400-\u4DBF\u4E00-\u9FFF\uA000-\uA4CF\uAC00-\uD7A3\uF900-\uFAFF\uFE10-\uFE19\uFE30-\uFE6F\uFF00-\uFF60\uFFE0-\uFFE6]/

function charUnits(ch: string): number {
  return WIDE_CHAR.test(ch) ? 1 : 0.55
}

function textUnits(text: string): number {
  let u = 0
  for (const ch of text) u += charUnits(ch)
  return u
}

function fitLabel(label: string, maxUnits: number): string {
  if (textUnits(label) <= maxUnits) return label
  let u = 0
  let out = ''
  for (const ch of label) {
    const du = charUnits(ch)
    if (u + du > maxUnits) break
    u += du
    out += ch
  }
  return out + '…'
}

/* ------------------------------------------------------------------ *
 * 宽度自适应：优先把高层级折叠成「…」，至少保留末尾 3 段
 * （当前 + 父 + 祖父 → 两级可向上跳转）；仍放不下时才逐级缩短
 * 每级名称的字数。折叠掉的层级在下拉里完整列出，随时可跳。
 * ------------------------------------------------------------------ */
const FONT_PX = 14      // --font-size-sm = 0.875rem
const BTN_PAD = 12      // .path-crumb 左右 padding 6 + 6
const SEP_PX = 18       // chevron 14 + flex gap
const NAV_PX = 30       // 返回按钮 26 + margin
const MACHINE_PX = 82   // 此电脑：图标 + 文字
const ELLIPSIS_PX = 40  // 「…」+ 下拉箭头 + padding
const DRIVE_ICON_PX = 17 // 盘符段的硬盘图标
const BAR_PADDING = 24
const SAFETY = 24       // 估算误差余量
// 末尾至少保留 3 段（当前 + 父 + 祖父），其中两级可直接向上跳转；
// 恒显示的盘符 / 文件系统根是额外的一段，不计入这 3 段。
const KEEP_TAIL = 3

interface Layout {
  visible: Segment[]
  collapsed: Segment[]
  chars: number
}

function visibleSegments(segs: Segment[], keep: number): Segment[] {
  return [segs[0], ...segs.slice(segs.length - keep + 1)]
}

function estimateWidth(segs: Segment[], keep: number, chars: number): number {
  const collapsed = segs.length > keep
  const visible = visibleSegments(segs, keep)
  const nodes = 1 /* 此电脑 */ + visible.length + (collapsed ? 1 : 0)
  let w = NAV_PX + MACHINE_PX + BAR_PADDING + SAFETY + SEP_PX * (nodes - 1)
  for (const s of visible) w += textUnits(fitLabel(s.label, chars)) * FONT_PX + BTN_PAD
  if (isDriveSegment(visible[0])) w += DRIVE_ICON_PX
  if (collapsed) w += ELLIPSIS_PX
  return w
}

const barEl = ref<HTMLElement | null>(null)
const barWidth = ref(0)
let resizeObserver: ResizeObserver | null = null

const layout = computed<Layout>(() => {
  const segs = segments.value
  if (segs.length === 0) {
    return { visible: [], collapsed: [], chars: props.maxLabelChars }
  }

  const width = barWidth.value || 900
  // 首段（盘符 / 根）恒可见 + 末尾至少 KEEP_TAIL 段
  const minKeep = Math.min(segs.length, 1 + KEEP_TAIL)
  // 字数候选：先按设定值，宽度不够再逐级收紧
  const candidates = [props.maxLabelChars, 12, 10, 8, 6, 5, 4]
    .filter((c, i, a) => c >= 4 && a.indexOf(c) === i)

  for (const chars of candidates) {
    // 基准形态（首段 + 末尾 minKeep-1 段 + 省略号）必须放得下
    if (estimateWidth(segs, minKeep, chars) > width) continue
    // 再尽量把折叠的高层级释放回来，从最靠近当前目录的一段开始
    let keep = minKeep
    for (let k = minKeep + 1; k <= segs.length; k++) {
      if (estimateWidth(segs, k, chars) > width) break
      keep = k
    }
    return {
      visible: visibleSegments(segs, keep),
      collapsed: segs.slice(1, segs.length - keep + 1),
      chars,
    }
  }

  // 极窄窗口兜底：末尾保留两级 + 最小字数
  return {
    visible: visibleSegments(segs, minKeep),
    collapsed: segs.slice(1, segs.length - minKeep + 1),
    chars: 4,
  }
})

/* -------- 折叠层级的下拉（Teleport 到 body，避免被 path-bar 裁切） -------- */
const menuOpen = ref(false)
const menuPos = ref({ x: 0, y: 0 })
const ellipsisBtn = ref<HTMLElement | null>(null)
const menuRef = ref<HTMLElement | null>(null)

function toggleMenu() {
  if (menuOpen.value) {
    menuOpen.value = false
    return
  }
  const el = ellipsisBtn.value
  if (!el) return
  const rect = el.getBoundingClientRect()
  menuPos.value = {
    x: Math.max(8, Math.min(rect.left, window.innerWidth - 280)),
    y: rect.bottom + 4,
  }
  menuOpen.value = true
}

function onDocMouseDown(e: MouseEvent) {
  if (!menuOpen.value) return
  const target = e.target as Node
  if (menuRef.value?.contains(target) || ellipsisBtn.value?.contains(target)) return
  menuOpen.value = false
}

// 捕获阶段拦截：下拉打开时 Esc 只关下拉，不会连带触发 App.vue 里
// 「收起详情面板 / 退出大图」的全局快捷键（两者都挂在 window 上）。
function onGlobalKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && menuOpen.value) {
    menuOpen.value = false
    e.preventDefault()
    e.stopPropagation()
  }
}

function go(path: string) {
  menuOpen.value = false
  emit('navigate', path)
}

watch(menuOpen, open => {
  if (open) document.addEventListener('mousedown', onDocMouseDown)
  else document.removeEventListener('mousedown', onDocMouseDown)
})

watch(() => props.currentPath, () => { menuOpen.value = false })

function measure() {
  if (barEl.value) barWidth.value = barEl.value.getBoundingClientRect().width
}

onMounted(() => {
  measure()
  if (typeof ResizeObserver !== 'undefined' && barEl.value) {
    resizeObserver = new ResizeObserver(entries => {
      const w = entries[0]?.contentRect.width
      if (w) barWidth.value = w
    })
    resizeObserver.observe(barEl.value)
  }
  window.addEventListener('resize', measure)
  window.addEventListener('keydown', onGlobalKeydown, true)
})

onBeforeUnmount(() => {
  resizeObserver?.disconnect()
  window.removeEventListener('resize', measure)
  window.removeEventListener('keydown', onGlobalKeydown, true)
  document.removeEventListener('mousedown', onDocMouseDown)
})
</script>

<template>
  <div ref="barEl" class="path-bar">
    <button
      v-if="parentPath"
      type="button"
      class="path-nav-btn"
      title="返回上一级"
      aria-label="返回上一级"
      @click="emit('navigate', parentPath ?? '')"
    >
      <ChevronLeft class="h-4 w-4" />
    </button>

    <template v-if="layout.visible.length">
      <span class="path-machine" title="此电脑">
        <Monitor class="h-4 w-4" />
        <span>此电脑</span>
      </span>
      <ChevronRight class="path-sep" />

      <!-- 首段（盘符 / 文件系统根）恒可见 -->
      <button
        type="button"
        class="path-crumb"
        :class="[
          layout.visible.length === 1 && 'path-crumb--current',
          isRootSegment(layout.visible[0]) && 'path-crumb--root',
        ]"
        :title="layout.visible[0].path"
        @click="go(layout.visible[0].path)"
      >
        <HardDrive v-if="isDriveSegment(layout.visible[0])" class="path-drive-icon" />
        {{ fitLabel(layout.visible[0].label, layout.chars) }}
      </button>

      <!-- 高层级折叠入口：点开可看到并跳到任意被折叠的目录 -->
      <template v-if="layout.collapsed.length">
        <ChevronRight class="path-sep" />
        <button
          ref="ellipsisBtn"
          type="button"
          class="path-ellipsis"
          :class="menuOpen && 'path-ellipsis--open'"
          :title="`展开 ${layout.collapsed.length} 个上层目录`"
          :aria-label="`展开 ${layout.collapsed.length} 个上层目录`"
          @click="toggleMenu"
        >
          <span>…</span>
          <ChevronDown class="h-3 w-3" />
        </button>
      </template>

      <!-- 末尾层级：至少保留两级可向上跳转 -->
      <template v-for="(seg, i) in layout.visible.slice(1)" :key="seg.path">
        <ChevronRight class="path-sep" />
        <button
          type="button"
          class="path-crumb"
          :class="[
            i === layout.visible.length - 2 && 'path-crumb--current',
            isRootSegment(seg) && 'path-crumb--root',
          ]"
          :title="isRootSegment(seg) ? `图库根目录 · ${seg.path}` : seg.path"
          @click="go(seg.path)"
        >
          {{ fitLabel(seg.label, layout.chars) }}
        </button>
      </template>
    </template>

    <span v-else class="path-placeholder">未打开目录</span>
  </div>

  <Teleport to="body">
    <Transition name="path-menu">
      <div
        v-if="menuOpen"
        ref="menuRef"
        class="path-menu"
        :style="{ left: menuPos.x + 'px', top: menuPos.y + 'px' }"
      >
        <div class="path-menu-caption">完整路径</div>
        <div class="path-menu-full" :title="currentPath ?? ''">{{ currentPath }}</div>

        <template v-if="layout.collapsed.length">
          <div class="path-menu-divider" />
          <div class="path-menu-caption">上层目录</div>
          <button
            v-for="seg in layout.collapsed"
            :key="seg.path"
            type="button"
            class="path-menu-item"
            :title="seg.path"
            @click="go(seg.path)"
          >
            <Folder class="h-3.5 w-3.5" />
            <span>{{ seg.label }}</span>
          </button>
        </template>

        <template v-if="rootPath">
          <div class="path-menu-divider" />
          <button
            type="button"
            class="path-menu-item path-menu-item--root"
            :title="rootPath"
            @click="go(rootPath)"
          >
            <Folder class="h-3.5 w-3.5" />
            <span>图库根目录</span>
          </button>
        </template>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.path-bar {
  position: relative;
  display: flex;
  align-items: center;
  gap: 2px;
  flex-shrink: 0;
  height: 40px;
  padding: 0 var(--spacing-sm);
  border-bottom: 1px solid var(--border);
  background: var(--background);
  overflow: hidden;
  white-space: nowrap;
}

.path-nav-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  margin-right: 2px;
  flex-shrink: 0;
  border: none;
  border-radius: calc(var(--radius) - 4px);
  background: transparent;
  color: var(--foreground);
  cursor: pointer;
}

.path-nav-btn:hover {
  background: var(--accent);
}

.path-machine {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
  color: var(--muted-foreground);
  font-size: var(--font-size-sm);
}

.path-sep {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
  color: var(--muted-foreground);
  opacity: 0.65;
}

.path-crumb {
  flex: 0 0 auto;
  max-width: 240px;
  padding: 4px 6px;
  border: none;
  border-radius: calc(var(--radius) - 4px);
  background: transparent;
  color: var(--muted-foreground);
  font-size: var(--font-size-sm);
  cursor: pointer;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.path-crumb:hover {
  background: var(--accent);
  color: var(--foreground);
}

.path-crumb--current {
  color: var(--foreground);
  font-weight: 600;
}

/* 「打开目录」设定的图库根，用一个小圆点标出，跳出后也能一眼找到 */
.path-crumb--root::before {
  content: '';
  display: inline-block;
  width: 5px;
  height: 5px;
  margin-right: 5px;
  border-radius: 50%;
  background: var(--primary);
  vertical-align: middle;
}

.path-drive-icon {
  display: inline-block;
  width: 13px;
  height: 13px;
  margin-right: 4px;
  vertical-align: -2px;
  opacity: 0.75;
  flex-shrink: 0;
}

.path-ellipsis {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 2px;
  flex: 0 0 auto;
  height: 24px;
  padding: 0 5px;
  border: none;
  border-radius: calc(var(--radius) - 4px);
  background: transparent;
  color: var(--muted-foreground);
  font-size: var(--font-size-sm);
  line-height: 1;
  cursor: pointer;
}

.path-ellipsis:hover,
.path-ellipsis--open {
  background: var(--accent);
  color: var(--foreground);
}

.path-placeholder {
  color: var(--muted-foreground);
  font-size: var(--font-size-sm);
}

/* 折叠层级下拉 */
.path-menu {
  position: fixed;
  z-index: var(--z-popover);
  width: 260px;
  max-height: 340px;
  overflow-y: auto;
  padding: var(--spacing-xs);
  border: 1px solid var(--border);
  border-radius: calc(var(--radius) - 2px);
  background: var(--popover);
  color: var(--popover-foreground);
  box-shadow: var(--shadow-lg);
}

.path-menu-caption {
  padding: 2px var(--spacing-sm) 4px;
  color: var(--muted-foreground);
  font-size: var(--font-size-xs);
}

.path-menu-full {
  padding: 4px var(--spacing-sm) 6px;
  font-size: var(--font-size-xs);
  line-height: 1.5;
  word-break: break-all;
  white-space: normal;
  user-select: text;
  cursor: text;
}

.path-menu-divider {
  height: 1px;
  margin: 4px 0 6px;
  background: var(--border);
}

.path-menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 6px var(--spacing-sm);
  border: none;
  border-radius: calc(var(--radius) - 4px);
  background: transparent;
  color: inherit;
  font-size: var(--font-size-sm);
  text-align: left;
  cursor: pointer;
  overflow: hidden;
}

.path-menu-item > span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.path-menu-item:hover {
  background: var(--accent);
  color: var(--accent-foreground);
}

.path-menu-item--root {
  color: var(--primary);
}

.path-menu-enter-active,
.path-menu-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}

.path-menu-enter-from,
.path-menu-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
