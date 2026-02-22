import { reactive } from 'vue'

export interface MenuItemDef {
  label: string
  action: () => void
}

interface MenuState {
  visible: boolean
  x: number
  y: number
  items: MenuItemDef[]
}

export function useContextMenu() {
  const menuState = reactive<MenuState>({
    visible: false,
    x: 0,
    y: 0,
    items: [],
  })

  function showMenu(e: MouseEvent, items: MenuItemDef[]) {
    e.preventDefault()
    menuState.x = e.clientX
    menuState.y = e.clientY
    menuState.items = items
    menuState.visible = true
  }

  function hideMenu() {
    menuState.visible = false
  }

  return { menuState, showMenu, hideMenu }
}
