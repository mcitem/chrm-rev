import type { Student } from '@/bindings/entity';
import { defineStore } from 'pinia';
import { ref } from 'vue';

/**
 * 只提供部分必要属性，让tab自行去拉取新数据
 */
export interface StuTab {
  id: number;
  name: string;
  student_no: string;
}

export const useStore = defineStore('default', () => {
  /**
   * 当前打开的学生tab列表
   */
  const students = ref<StuTab[]>([]);

  const addStudent = (student: Student) => {
    if (student.balance === undefined || student.balance === null) {
      instance.get(`/student/${student.id}/touch`).then(res => {
        students.value.push(res as unknown as Student);
      });
    } else {
      students.value.push({
        id: student.id,
        name: student.name,
        student_no: student.student_no,
      });
    }
  };

  const removeStudent = (student: StuTab) => {
    students.value = students.value.filter(s => s.id !== student.id);
  };

  const clearStudents = () => {
    students.value = [];
  };
  return { students, addStudent, removeStudent, clearStudents };
});

export default useStore;
