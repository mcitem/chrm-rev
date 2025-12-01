<template>
  <div class="flex flex-wrap gap-2">
    <template v-for="(tag, index) in tags">
      <template v-if="index === editInputIndex">
        <a-input
          :key="tag"
          v-model:value="editInputValue"
          :style="tagInputStyle"
          @blur="handleEditInputConfirm"
          @press-enter="handleEditInputConfirm"
          @vue:mounted="self => self.el?.focus()"
        />
      </template>
      <template v-else>
        <a-tag
          :key="tag"
          class="select-none"
          :styles="tagStyle"
          closable
          @close="() => (tags = tags.filter(i => i !== tag))"
          @dblclick="
            (e: MouseEvent) => {
              editInputIndex = index
              editInputValue = tag
              e.preventDefault()
            }
          "
        >
          <template #closeIcon>
            <CloseOutlined
              :style="
                {
                  fontSize: '14px',
                  paddingLeft: '8px',
                } satisfies CSSProperties
              "
            />
          </template>
          {{ tag }}
        </a-tag>
      </template>
    </template>

    <template v-if="inputVisible">
      <a-input
        v-model:value="inputValue"
        type="text"
        size="large"
        :style="tagInputStyle"
        @blur="handleInputConfirm"
        @press-enter="handleInputConfirm"
        @vue:mounted="self => self.el?.focus()"
      />
    </template>
    <template v-else>
      <a-tag
        class="select-none"
        :styles="tagStyle"
        @click="() => (inputVisible = true)"
      >
        <template #icon>
          <PlusOutlined />
        </template>
        添加
      </a-tag>
    </template>
  </div>
</template>

<script setup lang="ts">
import type { TagStylesType } from 'antdv-next/dist/tag/index'
import type { CSSProperties } from 'vue'
import { CloseOutlined, PlusOutlined } from '@antdv-next/icons'

const tags = defineModel<string[]>({ required: true })

const tagInputStyle = {
  width: '64px',
  height: '28px',
  marginInlineEnd: '8px',
  verticalAlign: 'top',
} satisfies CSSProperties

const tagStyle = {
  root: {
    padding: `4px 8px`,
    fontSize: '14px',
  },
} satisfies TagStylesType

const inputVisible = ref(false)
const inputValue = ref('')

const editInputIndex = ref(-1)
const editInputValue = ref('')

function handleInputConfirm() {
  if (inputValue.value === '') {
    inputVisible.value = false
    return
  }
  if (inputValue.value && !tags.value.includes(inputValue.value)) {
    tags.value = [...tags.value, inputValue.value]
  }
  inputVisible.value = false
  inputValue.value = ''
}
function handleEditInputConfirm() {
  if (editInputValue.value === '') {
    editInputIndex.value = -1
    editInputValue.value = ''
    return
  }
  const newTags = [...tags.value]
  newTags[editInputIndex.value] = editInputValue.value
  tags.value = newTags
  editInputIndex.value = -1
  editInputValue.value = ''
}
</script>
