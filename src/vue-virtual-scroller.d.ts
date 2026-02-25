declare module 'vue-virtual-scroller' {
  import { DefineComponent } from 'vue'

  export const RecycleScroller: DefineComponent<{
    items: any[]
    itemSize: number | null
    minItemSize?: number | string
    direction?: 'vertical' | 'horizontal'
    keyField?: string
    buffer?: number
    [key: string]: any
  }>

  export const DynamicScroller: DefineComponent<any>
  export const DynamicScrollerItem: DefineComponent<any>
}
