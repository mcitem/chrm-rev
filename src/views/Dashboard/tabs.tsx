import MainTab from './MainTab/MainTab.vue'

export const MAIN_TAB_KEY = '__main__'

export const MainTabDefinition = {
  key: MAIN_TAB_KEY,
  label: '首页',
  content: <MainTab />,
  closable: false,
}
