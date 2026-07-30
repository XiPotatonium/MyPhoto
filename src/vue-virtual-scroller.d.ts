declare module 'vue-virtual-scroller' {
  import { DefineComponent } from 'vue'

  export const RecycleScroller: DefineComponent<{
    items: unknown[]
    itemSize: number | null
    minItemSize?: number | string
    direction?: 'vertical' | 'horizontal'
    keyField?: string
    buffer?: number
  }>

  export const DynamicScroller: DefineComponent<any>
  export const DynamicScrollerItem: DefineComponent<any>
}
