<template>
  <div class="flex h-full flex-col">
    <v-text-field
      aria-autocomplete="none"
      class="block flex-shrink-0 flex-grow-0"
      hide-details
      v-model="Params.q"
      placeholder="搜索 支持姓名、学号、学院、班级  eg: 物理科学与技术学院"
    ></v-text-field>
    <div class="flex-1 overflow-hidden">
      <v-data-table-server
        class="h-full whitespace-nowrap"
        fixed-header
        fixed-footer
        :headers="Head"
        v-model:items-per-page="Params.per_page"
        v-model:page="Params.page"
        @update:sort-by="
          (v: { key: string; order: Order }[]) => {
            if (v.length > 0) {
              Params.order_by_key = v[0].key;
              Params.order_by_type = v[0].order;
            } else {
              Params.order_by_key = null;
              Params.order_by_type = null;
            }
          }
        "
        :items="students?.data ?? []"
        :items-length="students?.length ?? 0"
        :items-per-page-options="[10, 15, 20, 50, 100]"
        :loading="isFetching"
      >
        <!-- 默认隐藏的主键列 -->
        <template v-slot:item.id="{ value }">
          <v-chip border="thin opacity-25" :text="value" label />
        </template>

        <template v-slot:item.balance="{ item }">
          {{ item.balance ?? '未使用过' }}
        </template>

        <!-- 操作列 -->
        <template v-slot:item.action="{ item }">
          <!-- 编辑 -->
          <v-btn
            size="small"
            density="comfortable"
            variant="flat"
            icon
            @click="
              () => {
                sudo().then(() => {
                  formModel = JSON.parse(JSON.stringify(item));
                  isEditing = true;
                  dialog = true;
                });
              }
            "
          >
            <v-icon>mdi-pencil</v-icon>
          </v-btn>
          <!-- 删除 -->
          <v-btn
            icon
            size="small"
            density="comfortable"
            variant="flat"
            @click="
              () => {
                sudo().then(() => {
                  formModel = JSON.parse(JSON.stringify(item));
                  deleteConfirmDialog = true;
                });
              }
            "
          >
            <v-icon color="red">mdi-delete</v-icon>
          </v-btn>
        </template>

        <template v-slot:footer.prepend>
          <v-btn
            v-tooltip:top="`添加学生`"
            icon
            density="comfortable"
            variant="flat"
            class="mr-2"
            @click="
              () => {
                sudo().then(() => {
                  dialog = true;
                  isEditing = false;
                  formModel = JSON.parse(JSON.stringify(defaultFormModel));
                });
              }
            "
          >
            <v-icon>mdi-plus</v-icon>
          </v-btn>
          <v-menu
            location="right"
            v-model="menu"
            :close-on-content-click="false"
          >
            <template v-slot:activator="{ props }">
              <v-btn
                v-tooltip:top="`隐藏列`"
                icon
                density="comfortable"
                variant="flat"
                class="mr-2"
                v-bind="props"
              >
                <v-icon>mdi-view-week</v-icon>
              </v-btn>
            </template>
            <v-card>
              <v-checkbox
                style="--v-input-control-height: 0px"
                hide-details
                v-for="item in HeadRaw"
                :key="item.key"
                :label="item.title"
                v-model="item.hide"
              ></v-checkbox>
              <v-btn
                style="width: 100%"
                @click="
                  () => {
                    HeadRaw.forEach(h => (h.hide = true));
                  }
                "
              >
                全选
              </v-btn>
              <v-btn
                style="width: 100%"
                @click="
                  () => {
                    HeadRaw.forEach(h => (h.hide = false));
                  }
                "
                >重置
              </v-btn>
            </v-card>
          </v-menu>
        </template>
      </v-data-table-server>
    </div>
  </div>

  <!-- 编辑新增 弹出对话框 -->
  <v-dialog v-model="dialog" max-width="500">
    <v-card :title="`${isEditing ? '编辑' : '添加'}学生`">
      <template v-slot:text>
        <v-row>
          <v-col cols="6">
            <v-text-field v-model="formModel.name" label="姓名" />
          </v-col>
          <v-col cols="6">
            <v-text-field v-model="formModel.student_no" label="学号" />
          </v-col>
          <v-col cols="6">
            <v-text-field v-model="formModel.sex" label="性别" />
          </v-col>
          <v-col>
            <v-text-field
              v-model="formModel.secondary_school"
              label="二级学院"
            />
          </v-col>
          <v-col cols="6">
            <v-text-field v-model="formModel.class" label="班级" />
          </v-col>

          <v-col cols="6">
            <v-select
              v-model="formModel.difficulty_level"
              :items="
                [
                  { title: '不困难', value: 'Peaceful' },
                  { title: '一般困难', value: 'Easy' },
                  { title: '困难', value: 'Normal' },
                  { title: '特别困难', value: 'Hard' },
                ] satisfies { title: string; value: Difficulty }[]
              "
              label="认定级别"
            ></v-select>
          </v-col>
          <v-col cols="9">
            <v-text-field
              v-model="formModel.balance"
              label="余额"
              placeholder="建议留空，'学生登录'时自动根据配置生成"
            />
          </v-col>
        </v-row>
      </template>

      <v-divider></v-divider>

      <v-card-actions class="bg-surface-light">
        <v-btn text="取消" variant="plain" @click="dialog = false"></v-btn>

        <v-spacer></v-spacer>

        <v-btn
          text="保存"
          @click="
            () => {
              if (isEditing) {
                isEditing = false;

                if (formModel.balance === ``) {
                  formModel.balance = null;
                }

                updateStudent.mutate(formModel);
              } else {
                addStudent.mutate(formModel);
              }
              dialog = false;
            }
          "
        ></v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

  <v-dialog v-model="deleteConfirmDialog" max-width="500">
    <v-card
      title="删除确认"
      :subtitle="`确定要删除 ${formModel.name}(${formModel.student_no}) 吗？`"
    >
      <v-card-text> 删除后将无法恢复 </v-card-text>
      <v-card-actions>
        <v-btn
          text="取消"
          variant="plain"
          @click="
            () => {
              deleteConfirmDialog = false;
            }
          "
        ></v-btn>

        <v-spacer></v-spacer>

        <v-btn
          text="删除"
          color="red"
          @click="
            () => {
              deleteConfirmDialog = false;
              deleteStudent.mutate(formModel);
            }
          "
        ></v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import type { Student } from '@/bindings/entity';
import type { Difficulty } from '@/bindings/types';
import type { Order, OrderPagination } from '@/bindings/utils';
import { GetStudents } from '@/lib/service';
import { sudo } from '@/lib/utils';
import type { DataTableHeader } from 'vuetify';
import type { VDataTableServer } from 'vuetify/components';

const menu = ref(false);
const dialog = ref(false);
const isEditing = ref(false);

const deleteConfirmDialog = ref(false);
const defaultFormModel: Student = {
  id: -1,
  student_no: '',
  name: '',
  difficulty_level: 'Peaceful',
  secondary_school: '',
  class: '',
  sex: '',
  balance: null,
};

const formModel = ref<Student>(defaultFormModel);

/**
 * 用于打开编辑对话框
 * NOTE
 */

const queryClient = useQueryClient();

const updateStudent = useMutation({
  mutationFn: (stu: Student) => instance.put(`/student/${stu.id}`, stu),
  onSuccess: (_, stu) => {
    toast.success(`更新 ${stu.name}(${stu.student_no}) 成功`);
    queryClient.invalidateQueries({
      queryKey: ['/student/list'],
    });
    queryClient.invalidateQueries({
      queryKey: ['student', stu.id],
    });
  },
  onError: (_, stu) => {
    toast.error(`更新 ${stu.name}(${stu.student_no}) 失败`);
  },
});
const deleteStudent = useMutation({
  mutationFn: (stu: Student) => instance.delete(`/student/${stu.id}`),
  onSuccess: (_, stu) => {
    toast.success(`删除 ${stu.name}(${stu.student_no}) 成功`);
    queryClient.invalidateQueries({
      queryKey: ['/student/list'],
    });
    queryClient.invalidateQueries({
      queryKey: ['/student', stu.id],
    });
  },
  onError: (_, stu) => {
    toast.error(`删除 ${stu.name}(${stu.student_no}) 失败`);
  },
});
const addStudent = useMutation({
  mutationFn: (stu: Student) => instance.post(`/student`, stu),
  onSuccess: (_, stu) => {
    toast.success(`添加 ${stu.name}(${stu.student_no}) 成功`);
    queryClient.invalidateQueries({
      queryKey: ['/student/list'],
    });
  },
  onError: (_, stu) => {
    toast.error(`添加 ${stu.name}(${stu.student_no}) 失败`);
  },
});

const HeadRaw: Ref<
  (DataTableHeader<Student> & {
    hide?: boolean;
  })[]
> = ref([
  {
    key: 'action',
    title: '编辑',
    sortable: false,
  },
  { key: 'name', title: '姓名', fixed: 'start' },
  {
    key: 'difficulty_level',
    title: '认定级别',
  },
  {
    key: 'student_no',
    title: '学号',
  },
  {
    key: 'balance',
    title: '余额',
  },
  { key: 'sex', title: '性别', align: 'center' },
  {
    key: 'secondary_school',
    title: '二级学院',
  },
  {
    key: 'class',
    title: '班级',
  },
  {
    key: 'id',
    title: 'ID',
    hide: true,
  },
]);

const Head = computed(() => {
  const tmp = HeadRaw.value
    .filter(h => h.hide !== true)
    .map(
      h =>
        ({
          ...h,
          align: 'center',
        }) satisfies DataTableHeader<Student>
    );
  return tmp;
});

const Params = reactive<OrderPagination>({
  page: 1,
  per_page: 10,
  order_by_key: null,
  order_by_type: null,
  q: null,
});

const { data: students, isFetching } = GetStudents(Params);
</script>
