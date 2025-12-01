import { defineStore } from 'pinia'
import { ref } from 'vue'

interface StabBuilder {
  // 传递给tab组件
  id: number
  // 传递给tab组件的key
  student_no: string
  // tab组件的label
  name: string
}

export const useStore = defineStore('default', () => {
  const tabs = ref<StabBuilder[]>([])

  const addTab = (student: StabBuilder) => {
    tabs.value.push({
      id: student.id,
      name: student.name,
      student_no: student.student_no,
    })
  }

  const pubTab = (student: StabBuilder) => {
    if (!tabs.value.some(tab => tab.student_no === student.student_no)) {
      addTab(student)
    }
  }

  const removeTab = (s_no: string) => {
    tabs.value = tabs.value.filter(tab => tab.student_no !== s_no)
  }

  const clearTabs = () => {
    tabs.value = []
  }

  return { tabs, addTab, pubTab, removeTab, clearTabs }
})

export default useStore
