<template>
  <a-descriptions :title="title" :items="items" />
</template>

<script setup lang="tsx">
import { useStudent } from '@/lib/service'

const props = defineProps<{
  id: number
}>()

const { data: student } = useStudent(props.id)

const title = computed(() => (
  <>
    {student.value?.name ?? '姓名'}
    (
    {student.value?.sex ?? '性别'}
    )
  </>
))
const items = computed(() => {
  return [
    {
      key: '1',
      label: '学号',
      content: student.value?.student_no ?? '-',
    },
    {
      key: '2',
      label: '认定级别',
      content: student.value?.difficulty_level ?? '-',
    },
    {
      key: '3',
      label: '院系',
      content: student.value?.secondary_school ?? '-',
    },
    {
      key: '4',
      label: '专业',
      content: student.value?.major ?? '-',
    },
    {
      key: '5',
      label: '班级',
      content: student.value?.class ?? '-',
    },
    {
      key: '6',
      label: '余额',
      content: student.value?.balance ?? '未使用过',
    },
  ]
})
</script>
