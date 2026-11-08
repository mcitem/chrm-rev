<template>
  <main
    class="manager"
    :style="{
      '--line': token.colorBorderSecondary,
      '--surface': token.colorBgContainer,
      '--head': token.colorFillAlter,
      '--changed': token.colorWarningBg,
      '--text': token.colorText,
      '--muted': token.colorTextSecondary,
      '--primary': token.colorPrimary,
    }"
  >
    <header>
      <div class="title-row">
        <a-button :disabled="saving" @click="router.push('/dashboard')">
          返回业务首页
        </a-button>
        <h1>学生与商品管理</h1>
        <a-button class="help-button" @click="helpOpen = true">
          帮助
        </a-button>
      </div>
      <div class="toolbar">
        <a-radio-group
          :value="kind"
          option-type="button"
          button-style="solid"
          :disabled="saving"
          @change="changeKind($event.target.value)"
        >
          <a-radio-button value="item">
            商品
          </a-radio-button>
          <a-radio-button value="student">
            学生
          </a-radio-button>
        </a-radio-group>
        <a-input-search
          v-model:value="search"
          :disabled="saving"
          :placeholder="
            kind === 'student'
              ? '搜索姓名、学号、院系、班级'
              : '搜索商品名称、编号'
          "
          style="width: 300px"
          allow-clear
          @search="searchRows"
        />
        <span class="grow" />
        <a-button :disabled="saving || loading || !!loadError" @click="addRow">
          新增{{ kind === 'student' ? '学生' : '商品' }}
        </a-button>
        <span role="status">{{
          drafts.size ? `${drafts.size} 行待保存` : '所有修改已保存'
        }}</span>
        <a-button :disabled="saving || loading" @click="discard">
          {{ drafts.size ? '放弃修改并刷新' : '刷新' }}
        </a-button>
        <a-button
          type="primary"
          :loading="saving"
          :disabled="!drafts.size || loading"
          @click="save"
        >
          保存全部修改
        </a-button>
      </div>
    </header>
    <div v-if="loadError" role="alert" class="empty">
      {{ loadError }} <a-button @click="load">
        重试
      </a-button>
    </div>
    <div
      v-else
      ref="grid"
      class="grid-wrap"
      :aria-busy="loading || saving"
      :inert="loading || saving"
    >
      <table aria-label="可编辑资料表格">
        <thead>
          <tr>
            <th class="id-column">
              编号
            </th>
            <th v-for="column in columns" :key="column.key">
              {{ column.label }}
            </th>
            <th class="status-column">
              状态
            </th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="(row, r) in displayRows"
            :key="rowKey(row)"
            :class="{ deleted: findEdit(row)?.action === 'delete' }"
          >
            <th class="id-column">
              {{ row.__new ? '待分配' : row.id }}
            </th>
            <td
              v-for="(column, c) in columns"
              :key="column.key"
              :class="{
                changed:
                  findEdit(row)?.after && column.key in findEdit(row)!.after,
                invalid: !!cellError(row, column),
              }"
              :title="cellError(row, column)"
              :inert="findEdit(row)?.action === 'delete'"
            >
              <select
                v-if="column.options"
                :value="valueOf(row, column)"
                :aria-label="`${row.id} ${column.label}`"
                :data-cell="`${r}:${c}`"
                @change="
                  editCell(
                    row,
                    column,
                    ($event.target as HTMLSelectElement).value,
                  )
                "
                @keydown="keydown($event, r, c)"
                @paste="paste($event, r, c)"
              >
                <option v-for="option in column.options" :key="option">
                  {{ option }}
                </option>
              </select>
              <input
                v-else
                :value="valueOf(row, column)"
                :aria-label="`${row.id} ${column.label}`"
                :aria-invalid="!!cellError(row, column)"
                :inputmode="column.money ? 'decimal' : 'text'"
                :data-cell="`${r}:${c}`"
                @input="
                  editCell(
                    row,
                    column,
                    ($event.target as HTMLInputElement).value,
                  )
                "
                @keydown="keydown($event, r, c)"
                @paste="paste($event, r, c)"
              >
            </td>
            <td class="status-column">
              <span v-if="findEdit(row)?.action === 'delete'">待删除</span>
              <span v-else-if="row.__new">待新增</span>
              <a-button
                v-if="findEdit(row)"
                size="small"
                :disabled="saving"
                @click="drafts.delete(rowKey(row))"
              >
                撤销此行
              </a-button>
              <a-button
                v-if="findEdit(row)?.action !== 'delete'"
                size="small"
                danger
                :disabled="saving"
                @click="deleteRow(row)"
              >
                删除行
              </a-button>
            </td>
          </tr>
        </tbody>
      </table>
      <div v-if="!displayRows.length" class="empty">
        {{ loading ? '正在加载…' : '没有匹配的数据' }}
      </div>
    </div>
    <footer>
      <span>{{ loading ? '正在加载…' : `共 ${total} 条` }}</span>
      <a-pagination
        :current="params.page"
        :page-size="params.per_page"
        :total="total"
        :disabled="loading || saving"
        show-size-changer
        :page-size-options="['20', '50', '100', '200']"
        @change="changePage"
      />
    </footer>
    <a-modal
      v-model:open="helpOpen"
      title="学生与商品管理 · 帮助"
      :footer="null"
      :width="640"
    >
      <div class="help-content">
        <p>仅在首页没有购买记录时，才能进入学生与商品管理。</p>
        <h2>编辑与快捷键</h2>
        <ul>
          <li>点击单元格直接编辑，困难级别使用下拉选择。</li>
          <li>
            Tab 切换单元格，Enter / Shift+Enter 上下移动，Esc 撤销当前格，Ctrl+S
            保存。
          </li>
          <li>支持从 Excel 粘贴多行多列，粘贴范围需在当前页内。</li>
        </ul>
        <h2>新增与删除</h2>
        <ul>
          <li>
            新增行保存后自动分配编号，已有编号不可修改，删除不会重排其他编号。
          </li>
          <li>
            删除行先标记，保存后生效，保存前可撤销；删除新增草稿行会直接移除该行。
          </li>
          <li>有购买记录关联的学生或商品不能直接删除。</li>
        </ul>
        <h2>保存修改</h2>
        <ul>
          <li>
            搜索、翻页和切换学生／商品时保留草稿，点击“保存全部修改”统一提交。
          </li>
          <li>
            姓名、商品名称和学号不能为空，学号不能重复；金额最多两位小数，商品价格不能为负数。
          </li>
          <li>
            任一行保存失败时整批回滚并保留草稿。遇到数据冲突，请记下修改，再放弃修改并刷新后重试。
          </li>
        </ul>
      </div>
    </a-modal>
  </main>
</template>

<script setup lang="ts">
import type { OrderPagination, PaginateData } from '@/bindings/utils'
import { isTauri } from '@tauri-apps/api/core'
import { App, theme } from 'antdv-next'
import { onBeforeRouteLeave } from 'vue-router'
import { databaseChanging, databaseGeneration } from '@/lib/databaseSession'
import { instance } from '@/lib/service'
import { useStore } from '@/lib/stores'

type ManagementKind = 'student' | 'item'
type CellValue = string | number | null
interface ManagementRow {
  id: number
  [key: string]: CellValue
}
interface ManagementColumn {
  key: string
  label: string
  money?: boolean
  optional?: boolean
  options?: string[]
}
interface RowEdit {
  action?: 'update' | 'create' | 'delete'
  kind: ManagementKind
  id: number
  before: Record<string, CellValue>
  after: Record<string, CellValue>
}

const managementColumns: Record<ManagementKind, ManagementColumn[]> = {
  student: [
    { key: 'name', label: '姓名' },
    { key: 'student_no', label: '学号' },
    {
      key: 'difficulty_level',
      label: '困难级别',
      options: ['不困难', '一般困难', '困难', '特别困难'],
    },
    { key: 'secondary_school', label: '院系', optional: true },
    { key: 'major', label: '专业', optional: true },
    { key: 'class', label: '班级', optional: true },
    { key: 'sex', label: '性别', optional: true },
    { key: 'balance', label: '余额', money: true },
  ],
  item: [
    { key: 'name', label: '商品名称' },
    { key: 'spec', label: '规格' },
    { key: 'price', label: '原价', money: true },
    { key: 'p_easy', label: '一般困难价', money: true },
    { key: 'p_normal', label: '困难价', money: true },
    { key: 'p_hard', label: '特别困难价', money: true },
    { key: 'p_score', label: '积分', money: true },
  ],
}

function createManagementRow(kind: ManagementKind, id: number): RowEdit {
  return {
    action: 'create',
    kind,
    id,
    before: {},
    after: Object.fromEntries(
      managementColumns[kind].map(column => [
        column.key,
        column.options?.[0]
        ?? (column.money ? '0' : column.optional ? null : ''),
      ]),
    ),
  }
}

function validateCell(
  column: ManagementColumn,
  value: string,
): string | undefined {
  if ((column.key === 'name' || column.key === 'student_no') && !value.trim())
    return `${column.label}不能为空`
  if (column.options && !column.options.includes(value))
    return `请选择有效的${column.label}`
  if (
    column.money
    && (!/^-?\d{1,8}(?:\.\d{1,2})?$/.test(value)
      || (column.key !== 'balance' && value.startsWith('-')))
  ) {
    return `${column.label}需为最多两位小数的金额（商品价格不能为负数）`
  }
}

// A rectangle is validated in full before any draft is changed.
function parsePaste(
  text: string,
  rowCount: number,
  columnCount: number,
  row: number,
  column: number,
) {
  const lines = text
    .replace(/\r\n?/g, '\n')
    .replace(/\n$/, '')
    .split('\n')
    .map(line => line.split('\t'))
  if (
    row + lines.length > rowCount
    || lines.some(line => column + line.length > columnCount)
  ) {
    throw new Error('粘贴范围超出当前页，请减少行列或调整每页行数')
  }
  return lines.flatMap((line, r) =>
    line.map((value, c) => ({ row: row + r, column: column + c, value })),
  )
}

const { message, modal } = App.useApp()
const { token } = theme.useToken()
const router = useRouter()
const queryClient = useQueryClient()
const kind = ref<ManagementKind>('item')
const search = ref('')
const helpOpen = ref(false)
const params = reactive<OrderPagination>({
  page: 1,
  per_page: 50,
  order_by_key: 'id',
  order_by_type: 'asc',
  q: null,
})
const drafts = reactive(new Map<string, RowEdit>())
const rows = ref<ManagementRow[]>([])
const total = ref(0)
const loading = ref(false)
const saving = ref(false)
const loadError = ref('')
const grid = ref<HTMLElement>()
const columns = computed(() => managementColumns[kind.value])
const displayRows = computed<ManagementRow[]>(() =>
  loading.value
    ? []
    : [
        ...Array.from(drafts.values())
          .filter(edit => edit.kind === kind.value && edit.action === 'create')
          .reverse()
          .map(edit => ({ ...edit.after, id: edit.id, __new: 1 })),
        ...rows.value,
      ],
)
let temporaryId = -1
let loadRevision = 0

async function addRow() {
  const draft = createManagementRow(kind.value, temporaryId--)
  drafts.set(`${kind.value}:new:${draft.id}`, draft)
  await nextTick()
  grid.value?.scrollTo({ top: 0 })
  grid.value?.querySelector<HTMLElement>('[data-cell="0:0"]')?.focus()
}
function deleteRow(row: ManagementRow) {
  const key = rowKey(row)
  if (row.__new) {
    drafts.delete(key)
    return
  }
  drafts.set(key, {
    action: 'delete',
    kind: kind.value,
    id: row.id,
    before: Object.fromEntries(
      columns.value.map(column => [column.key, row[column.key] ?? null]),
    ),
    after: {},
  })
}

async function load() {
  const revision = ++loadRevision
  const generation = databaseGeneration.value
  loading.value = true
  // Never render the previous dataset against a newly selected column schema.
  rows.value = []
  loadError.value = ''
  try {
    const result = (await instance.get(`/biz/${kind.value}/list`, {
      params: { ...params },
    })) as PaginateData<ManagementRow[]>
    if (revision !== loadRevision || generation !== databaseGeneration.value)
      return
    rows.value = result.data
    total.value = result.length
    const lastPage = Math.max(1, Math.ceil(result.length / params.per_page))
    if (params.page > lastPage)
      params.page = lastPage
  }
  catch {
    if (revision === loadRevision) {
      rows.value = []
      loadError.value = '加载失败，请重试'
    }
  }
  finally {
    if (revision === loadRevision)
      loading.value = false
  }
}
watch([kind, () => params.page, () => params.per_page, () => params.q], load, {
  immediate: true,
})
function searchRows() {
  params.page = 1
  params.q = search.value.trim() || null
}
function changeKind(value: ManagementKind) {
  if (saving.value)
    return
  kind.value = value
  search.value = ''
  params.q = null
  params.page = 1
}
function changePage(page: number, size: number) {
  params.page = size === params.per_page ? page : 1
  params.per_page = size
}
function rowKey(row: ManagementRow) {
  return `${kind.value}:${row.__new ? 'new:' : ''}${row.id}`
}
function findEdit(row: ManagementRow) {
  return drafts.get(rowKey(row))
}
function valueOf(row: ManagementRow, column: ManagementColumn) {
  const draft = findEdit(row)
  return String(
    draft && column.key in draft.after
      ? (draft.after[column.key] ?? '')
      : (row[column.key] ?? ''),
  )
}
function editCell(row: ManagementRow, column: ManagementColumn, text: string) {
  const key = rowKey(row)
  const draft = drafts.get(key) ?? {
    kind: kind.value,
    id: row.id,
    before: {},
    after: {},
  }
  if (draft.action === 'delete')
    return
  if (draft.action === 'create') {
    draft.after[column.key] = column.optional && text === '' ? null : text
    return
  }
  if (!(column.key in draft.before))
    draft.before[column.key] = row[column.key] ?? null
  const value = column.optional && text === '' ? null : text
  if (String(draft.before[column.key] ?? '') === String(value ?? '')) {
    delete draft.before[column.key]
    delete draft.after[column.key]
  }
  else {
    draft.after[column.key] = value
  }
  if (Object.keys(draft.after).length)
    drafts.set(key, draft)
  else drafts.delete(key)
}
function cellError(row: ManagementRow, column: ManagementColumn) {
  if (findEdit(row)?.action === 'delete')
    return
  return validateCell(column, valueOf(row, column))
}
function keydown(event: KeyboardEvent, r: number, c: number) {
  if (event.key === 'Escape') {
    const original = findEdit(displayRows.value[r])?.before[
      columns.value[c].key
    ]
    if (original !== undefined)
      editCell(displayRows.value[r], columns.value[c], String(original ?? ''))
    return
  }
  if (event.key !== 'Enter' || event.isComposing)
    return
  event.preventDefault()
  const next = r + (event.shiftKey ? -1 : 1)
  grid.value?.querySelector<HTMLElement>(`[data-cell="${next}:${c}"]`)?.focus()
}
function paste(event: ClipboardEvent, r: number, c: number) {
  const text = event.clipboardData?.getData('text/plain') ?? ''
  if (!text.includes('\t') && !text.includes('\n'))
    return
  event.preventDefault()
  try {
    const cells = parsePaste(
      text,
      displayRows.value.length,
      columns.value.length,
      r,
      c,
    )
    for (const cell of cells) {
      if (findEdit(displayRows.value[cell.row])?.action === 'delete')
        throw new Error('粘贴范围包含待删除行，请先撤销删除')
      const error = validateCell(columns.value[cell.column], cell.value)
      if (error)
        throw new Error(`第 ${cell.row + 1} 行：${error}`)
    }
    for (const cell of cells) {
      editCell(
        displayRows.value[cell.row],
        columns.value[cell.column],
        cell.value,
      )
    }
  }
  catch (error) {
    message.error(String(error))
  }
}
async function save() {
  if (saving.value || !drafts.size || databaseChanging.value)
    return
  const edits = Array.from(drafts.values())
  for (const edit of edits) {
    for (const [key, value] of Object.entries(edit.after)) {
      const column = managementColumns[edit.kind].find(
        column => column.key === key,
      )!
      const error = validateCell(column, String(value ?? ''))
      if (error) {
        message.error(`编号 ${edit.id}：${error}`)
        return
      }
    }
  }
  saving.value = true
  try {
    const result = (await instance.put(
      '/biz/manage/save',
      edits,
    )) as unknown as {
      created: { kind: ManagementKind, temporary_id: number, id: number }[]
    }
    if (isTauri()) {
      const store = useStore()
      store.tabs = store.tabs.filter(
        tab =>
          !edits.some(
            edit =>
              edit.kind === 'student'
              && edit.action === 'delete'
              && edit.id === tab.id,
          ),
      )
      const tabs = store.tabs
      for (const edit of edits) {
        const tab
          = edit.kind === 'student' && edit.action !== 'create'
            ? tabs.find(tab => tab.id === edit.id)
            : undefined
        if (tab) {
          if (typeof edit.after.name === 'string')
            tab.name = edit.after.name
          if (typeof edit.after.student_no === 'string')
            tab.student_no = edit.after.student_no
        }
      }
    }
    drafts.clear()
    const assigned = result.created
      ?.map(row => `${row.kind === 'student' ? '学生' : '商品'} #${row.id}`)
      .join('、')
    message.success(
      `已保存 ${edits.length} 行修改${assigned ? `，新增编号：${assigned}` : ''}`,
    )
    await Promise.all([load(), queryClient.invalidateQueries()])
  }
  catch {
    message.error(
      '保存失败，修改仍保留。请根据错误提示检查；如有数据冲突，请放弃修改并刷新后重试。',
    )
  }
  finally {
    saving.value = false
  }
}
function confirmDiscard(): Promise<boolean> {
  return new Promise(resolve =>
    modal.confirm({
      title: '放弃未保存的修改？',
      content: `有 ${drafts.size} 行新增、修改或删除尚未保存。`,
      okText: '放弃修改',
      cancelText: '继续编辑',
      onOk: () => resolve(true),
      onCancel: () => resolve(false),
    }),
  )
}
async function discard() {
  if (!drafts.size || (await confirmDiscard())) {
    drafts.clear()
    await load()
  }
}
onBeforeRouteLeave(
  async () => !saving.value && (!drafts.size || (await confirmDiscard())),
)
function beforeUnload(event: BeforeUnloadEvent) {
  if (drafts.size || saving.value) {
    event.preventDefault()
    event.returnValue = ''
  }
}
function shortcut(event: KeyboardEvent) {
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 's') {
    event.preventDefault()
    void save()
  }
}
watch(databaseGeneration, () => {
  drafts.clear()
  rows.value = []
  loadRevision++
})
onMounted(() => {
  window.addEventListener('beforeunload', beforeUnload)
  window.addEventListener('keydown', shortcut)
})
onUnmounted(() => {
  loadRevision++
  window.removeEventListener('beforeunload', beforeUnload)
  window.removeEventListener('keydown', shortcut)
})
</script>

<style scoped>
.manager {
  height: 100vh;
  display: flex;
  flex-direction: column;
  padding: 20px;
  gap: 16px;
  color: var(--text);
  background: var(--surface);
}
.title-row,
.toolbar,
footer {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}
h1 {
  font-size: 22px;
  font-weight: 600;
  margin: 0;
}
.toolbar {
  margin-top: 20px;
}
.grow {
  flex: 1;
}
.help-button {
  margin-left: auto;
}
.help-content h2 {
  margin: 20px 0 8px;
  font-size: 14px;
  font-weight: 600;
}
.help-content ul {
  margin: 0;
  padding-left: 20px;
  list-style: disc;
}
.help-content li {
  margin: 6px 0;
  line-height: 1.7;
}
.grid-wrap {
  flex: 1;
  min-height: 0;
  overflow: auto;
  border: 1px solid var(--line);
}
table {
  border-collapse: separate;
  border-spacing: 0;
  width: 100%;
  table-layout: fixed;
  min-width: 1250px;
}
th,
td {
  border-right: 1px solid var(--line);
  border-bottom: 1px solid var(--line);
  padding: 0;
  text-align: left;
}
thead th {
  position: sticky;
  top: 0;
  z-index: 2;
  background: linear-gradient(var(--head), var(--head)), var(--surface);
  padding: 12px 10px;
  font-weight: 500;
}
.id-column {
  width: 68px;
  padding: 10px;
  color: var(--muted);
  background: linear-gradient(var(--head), var(--head)), var(--surface);
}
.status-column {
  width: 225px;
  padding: 4px 10px;
  color: var(--muted);
}
.status-column > * + * {
  margin-left: 6px;
}
.deleted td:not(.status-column) {
  opacity: 0.5;
  text-decoration: line-through;
  background: var(--head);
}
input,
select {
  width: 100%;
  height: 38px;
  padding: 8px 10px;
  border: 0;
  border-radius: 0;
  background: transparent;
  color: inherit;
  outline: none;
  font: inherit;
}
select option {
  background: var(--surface);
  color: var(--text);
}
input:focus,
select:focus {
  outline: 2px solid var(--primary);
  outline-offset: -2px;
}
.changed {
  background: var(--changed);
}
.invalid {
  box-shadow: inset 0 -2px #ff4d4f;
}
.empty {
  padding: 40px;
  text-align: center;
  color: var(--muted);
}
footer {
  justify-content: space-between;
  font-size: 12px;
  color: var(--muted);
}
</style>
