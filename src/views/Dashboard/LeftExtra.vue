<template>
  <a-dropdown
    :menu="menuPropsWithTheme"
    placement="bottomRight"
    :classes="{
      root: 'leftExtra',
    }"
  >
    <div class="ant-tabs-tab start-0" :style="{ borderRadius: 0, margin: 0 }">
      <span>
        <EllipsisOutlined />
      </span>
    </div>
  </a-dropdown>
</template>

<script setup lang="tsx">
import type { MenuProps } from 'antdv-next'
import type { ExportAllResponse } from '@/bindings/dataManager'
import type { RollBackBackup } from '@/bindings/sys'
import {
  BugOutlined,
  ChromeOutlined,
  DatabaseOutlined,
  EllipsisOutlined,
  ExportOutlined,
  FileZipOutlined,
  FolderOpenOutlined,
  FormatPainterOutlined,
  GithubOutlined,
  ImportOutlined,
  MoonOutlined,
  NumberOutlined,
  QuestionCircleOutlined,
  RollbackOutlined,
  SettingOutlined,
  SunOutlined,
  SyncOutlined,
  TableOutlined,
} from '@antdv-next/icons'
import { open } from '@tauri-apps/plugin-dialog'
import { useColorMode } from '@vueuse/core'
import { App } from 'antdv-next'
import {
  beginDatabaseChange,
  endDatabaseChange,
  resetDatabaseSession,
} from '@/lib/databaseSession'

const { message } = App.useApp()
const queryClient = useQueryClient()
const { store: theme } = useColorMode()

const menuPropsConst: MenuProps = {
  selectable: true,
  items: [
    {
      key: 'docs',
      label: '帮助文档',
      icon: QuestionCircleOutlined,
    },
    {
      key: 'QuickReference',
      label: '商品速查',
      icon: TableOutlined,
    },
    {
      key: 'operation',
      label: '管理',
      icon: DatabaseOutlined,
      children: [
        {
          key: 'DataManager',
          label: '学生与商品管理',
          icon: TableOutlined,
        },
        {
          key: 'DataDir',
          label: '数据目录',
          icon: FolderOpenOutlined,
        },
        {
          key: 'ConfigLoader',
          label: '配置管理',
          icon: SettingOutlined,
        },
        {
          key: 'Export',
          label: '数据导出',
          icon: ExportOutlined,
        },
        {
          key: 'Import',
          label: '数据导入',
          icon: ImportOutlined,
        },
        {
          key: 'Backup',
          label: '数据备份',
          icon: FileZipOutlined,
        },
        {
          key: 'Restore',
          label: '数据恢复',
          icon: RollbackOutlined,
        },
      ],
    },
    {
      key: 'tools',
      label: '开发',
      icon: NumberOutlined,
      children: [
        {
          key: 'DebugTools',
          label: '调试工具',
          icon: BugOutlined,
        },
        {
          key: 'DevTools',
          label: 'DevTools',
          icon: ChromeOutlined,
        },
        {
          key: 'Logs',
          label: '日志目录',
          icon: FolderOpenOutlined,
        },
      ],
    },
    {
      key: 'theme',
      label: '主题',
      icon: FormatPainterOutlined,
      children: [
        {
          key: 'auto',
          label: '跟随系统',
          icon: SyncOutlined,
        },
        {
          key: 'light',
          label: '浅色模式',
          icon: SunOutlined,
        },
        {
          key: 'dark',
          label: '深色模式',
          icon: MoonOutlined,
        },
      ],
    },
    {
      key: 'Github',
      label: 'Github',
      icon: GithubOutlined,
    },
  ],
  onClick: (e) => {
    if (e.key === 'light' || e.key === 'dark' || e.key === 'auto') {
      theme.value = e.key
    }
    else if (e.key === 'docs') {
      instance.post('/sys/open_docs', {})
    }
    else if (e.key === 'QuickReference') {
      instance.post('/sys/open_QuickReference', {})
    }
    else if (e.key === 'ConfigLoader') {
      nextTick(() => {
        router.push({
          path: '/configloader',
        })
      })
    }
    else if (e.key === 'DataManager') {
      router.push('/datamanager')
    }
    else if (e.key === 'Export') {
      instance.post('/sys/data_manager/export_all/check', {}).then(async () => {
        const res = (await instance.post(
          '/sys/data_manager/export_all',
          {},
        )) as ExportAllResponse
        message.success(`文件已保存至：${res.dst}`)
      })
    }
    else if (e.key === 'Import') {
      nextTick(() => {
        router.push({ path: '/dataloader' })
      })
    }
    else if (e.key === 'Backup') {
      instance.post('/sys/backup/create', {}, { timeout: 0 }).then((path) => {
        message.success(`备份已保存至：${path}`)
      })
    }
    else if (e.key === 'Restore') {
      open({
        multiple: false,
        title: '选择数据库备份文件',
        directory: false,
        filters: [
          {
            name: 'SQLite 数据库备份',
            extensions: ['db'],
          },
        ],
      }).then((path) => {
        if (path) {
          beginDatabaseChange()
          instance
            .post(
              '/sys/backup/rollback',
              {
                path: path as string,
              } satisfies RollBackBackup,
              { timeout: 0 },
            )
            .then(async () => {
              await resetDatabaseSession(queryClient)
              message.success('恢复成功')
            })
            .finally(endDatabaseChange)
        }
      })
    }
    else if (e.key === 'DataDir') {
      instance.post('/sys/open_data_dir', {})
    }
    else if (e.key === 'DebugTools') {
      instance.post('/sys/open_debug', {})
    }
    else if (e.key === 'DevTools') {
      instance.post('/sys/open_devtools', {})
    }
    else if (e.key === 'Logs') {
      instance.post('/sys/open_logs', {})
    }
    else if (e.key === 'Github') {
      instance.post('/sys/open_git_repo', {})
    }
  },
}
const menuPropsWithTheme = computed(() => {
  return {
    ...menuPropsConst,
    selectedKeys: [theme.value],
  } satisfies MenuProps
})
</script>
