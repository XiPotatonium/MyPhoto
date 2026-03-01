import { ref, watch, onMounted } from 'vue'

type Theme = 'light' | 'dark' | 'system'

const STORAGE_KEY = 'myphoto-theme'

const theme = ref<Theme>('system')
const resolvedTheme = ref<'light' | 'dark'>('light')

function getSystemTheme(): 'light' | 'dark' {
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

function updateResolvedTheme() {
  if (theme.value === 'system') {
    resolvedTheme.value = getSystemTheme()
  } else {
    resolvedTheme.value = theme.value
  }
}

function applyTheme() {
  const root = document.documentElement
  const isDark = resolvedTheme.value === 'dark'
  
  // Add transition class for smooth theme switching
  document.body.classList.add('theme-transitioning')
  
  if (isDark) {
    root.classList.add('dark')
  } else {
    root.classList.remove('dark')
  }
  
  // Remove transition class after animation completes
  setTimeout(() => {
    document.body.classList.remove('theme-transitioning')
  }, 200)
}

function setTheme(newTheme: Theme) {
  theme.value = newTheme
  localStorage.setItem(STORAGE_KEY, newTheme)
  updateResolvedTheme()
  applyTheme()
}

function toggleTheme() {
  const newTheme = resolvedTheme.value === 'light' ? 'dark' : 'light'
  setTheme(newTheme)
}

export function useTheme() {
  onMounted(() => {
    // Load saved theme
    const saved = localStorage.getItem(STORAGE_KEY) as Theme | null
    if (saved && ['light', 'dark', 'system'].includes(saved)) {
      theme.value = saved
    }
    
    updateResolvedTheme()
    applyTheme()
    
    // Listen for system theme changes
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    mediaQuery.addEventListener('change', () => {
      if (theme.value === 'system') {
        updateResolvedTheme()
        applyTheme()
      }
    })
  })
  
  // Watch for theme changes
  watch(theme, () => {
    updateResolvedTheme()
    applyTheme()
  })
  
  return {
    theme,
    resolvedTheme,
    setTheme,
    toggleTheme,
  }
}
