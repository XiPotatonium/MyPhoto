import { computed } from 'vue'
import { useColorMode } from '@vueuse/core'

type Theme = 'light' | 'dark' | 'auto'

export function useTheme() {
  const mode = useColorMode({
    attribute: 'class',
    storageKey: 'myphoto-theme',
    modes: {
      light: '',
      dark: 'dark',
    },
  })

  // 当前实际生效的主题（解析 auto 后的值）
  const resolvedTheme = computed(() => mode.value as 'light' | 'dark')

  // 当前存储的模式（light / dark / auto）
  const theme = computed({
    get: () => mode.store.value as Theme,
    set: (value: Theme) => {
      mode.store.value = value
    },
  })

  function setTheme(newTheme: Theme) {
    mode.store.value = newTheme
  }

  function toggleTheme() {
    const newTheme = resolvedTheme.value === 'light' ? 'dark' : 'light'
    setTheme(newTheme)
  }

  return {
    theme,
    resolvedTheme,
    setTheme,
    toggleTheme,
  }
}
