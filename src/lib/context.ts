import type { TabsProps } from 'antdv-next'

export interface ImportContext {
  path: Ref<string>
  selectedSheet: Ref<string>
  importStep: Ref<number>
  dataIncludeHeader: Ref<boolean>
  loadContext: Ref<Record<string, number | null>>
}

export const IMPORT_CONTEXT_KEY = 'import-context'

export interface DashboardHomeContext {
  tab: Ref<string>
  tabs: Ref<TabsProps['items']>
}

export const DASHBOARD_HOME_CONTEXT_KEY = 'dashboard-home-context'
