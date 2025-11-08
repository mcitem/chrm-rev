<template>
  <div class="flex h-screen w-full flex-col">
    <v-tabs
      class="flex-shrink-0 flex-grow-0"
      :model-value="tab"
      @update:model-value="
        val => {
          tab =
            typeof val === 'string'
              ? val
              : store.students.length
                ? store.students[store.students.length - 1].student_no
                : '登录';
        }
      "
    >
      <v-tab value="登录" text="学生登录"></v-tab>
      <v-tab
        v-for="stu in store.students"
        :key="stu.student_no"
        :value="stu.student_no"
        :text="stu.name"
      >
        <template v-slot:append>
          <v-btn
            @click="
              () => {
                store.removeStudent(stu);
                store.$tauri.saveNow();
                tab = `登录`;
              }
            "
            icon
            size="small"
            density="comfortable"
            variant="plain"
          >
            <v-icon>mdi-close</v-icon>
          </v-btn>
        </template>
      </v-tab>
    </v-tabs>
    <v-tabs-window :model-value="tab" class="flex-grow">
      <v-tabs-window-item value="登录" class="flex h-full flex-col">
        <!-- 页头 -->
        <div
          class="mr-4 flex flex-shrink-0 items-center justify-center gap-2 pb-3 pt-3 text-3xl"
        >
          <div class="h-15 font-mc10 text-5xl">CHRM REV</div>
        </div>
        <!-- 搜索框 -->
        <div
          :class="
            cn(
              `relative mx-3 flex-shrink-0 pt-2 dark:bg-[#1a1a1a]`,
              Params.q ? `rounded-t-md` : `rounded-md`
            )
          "
        >
          <v-text-field
            aria-autocomplete="none"
            hide-details
            v-model="Params.q"
            placeholder="搜索： 学号 或 姓名"
            clearable
            autofocus
          ></v-text-field>

          <transition
            enter-active-class="transition-all duration-200"
            enter-from-class="opacity-0 -translate-y-2"
            enter-to-class="opacity-100 translate-y-0"
            leave-active-class="transition-all duration-200"
            leave-from-class="opacity-100 translate-y-0"
            leave-to-class="opacity-0 -translate-y-2"
          >
            <div
              v-if="students?.length === 0 && Params.q"
              class="absolute z-10 w-full bg-[#f3f4f5] px-4 py-2 text-center shadow-md dark:bg-[#1a1a1a]"
            >
              无数据
            </div>

            <div
              v-else-if="students?.length && Params.q"
              class="absolute z-10 w-full bg-[#f3f4f5] shadow-md dark:bg-[#1a1a1a]"
            >
              <!-- 搜索学生列表 -->
              <button
                v-for="(item, index) in students.data"
                :key="index"
                class="block w-full border-b border-gray-200 px-4 py-2 text-start"
                @click="
                  () => {
                    if (
                      !store.students.find(
                        stu => stu.student_no === item.student_no
                      )
                    ) {
                      store.addStudent(item);
                      info(
                        `添加学生: ${item.name} - ${item.student_no} - ${item.difficulty_level}`
                      );
                      store.$tauri.saveNow();
                    }
                    tab = item.student_no;
                  }
                "
              >
                {{ item.name }} - {{ item.student_no }} -
                {{ item.difficulty_level }}
              </button>
            </div>
            <!-- 不能移除的v-else!!!!,会导致keepalive出错 -->
            <div v-else></div>
          </transition>
        </div>

        <!-- 内容 -->
        <div class="flex flex-1 flex-col overflow-hidden">
          <div
            class="flex flex-shrink-0 flex-grow-0 items-center gap-2 px-3 pb-2 pt-3"
          >
            <span class="text-2xl font-bold">电子表</span>
            <v-btn
              size="small"
              variant="tonal"
              @click="
                () => {
                  let newGroupView = groupView + 1;
                  groupView = newGroupView > 2 ? 0 : newGroupView;
                }
              "
              >{{
                groupView === 0
                  ? '分组查看'
                  : groupView === 1
                    ? '按学生分组'
                    : '按商品分组'
              }}</v-btn
            >
            <v-btn size="small" color="secondary" @click="open_csv">
              电子表
            </v-btn>
            <v-btn
              size="small"
              color="primary"
              @click="
                () => {
                  exportDialog = true;
                }
              "
              :disabled="exportMutation.isPending.value"
            >
              导出
            </v-btn>
          </div>
          <div class="flex-1 overflow-hidden">
            <v-data-table
              fixed-header
              class="h-full whitespace-nowrap"
              :items="all_record.data.value"
              items-per-page="-1"
              :group-by="
                groupView === 1
                  ? [{ key: 'student_no', order: 'asc' }]
                  : groupView === 2
                    ? [{ key: 'item_id', order: 'asc' }]
                    : []
              "
              :headers="
                [
                  {
                    key: `id`,
                    title: `序号`,
                  },
                  {
                    key: 'student_no',
                    title: '学号',
                  },
                  {
                    key: 'stu_d_level',
                    title: '认定级别',
                  },
                  {
                    key: 'item_id',
                    title: '商品编号',
                  },
                  {
                    key: 'discount_price',
                    title: '折后价',
                  },
                  {
                    key: 'original_price',
                    title: '原价',
                  },
                  {
                    key: 'quantity',
                    title: '数量',
                  },
                ].map(h => ({ ...h, align: 'center', sortable: false }))
              "
            >
              <template #bottom>
                <div class="flex justify-end px-4 py-3 pb-3">
                  共 {{ summary.stu_count }} 人，{{ summary.item_count }}
                  种商品，总共
                  {{ summary.total_quantity }} 件商品，总计（折后价）
                  {{ summary.total_discount_price.toFixed(2) }} 元
                </div>
              </template>
            </v-data-table>
          </div>
        </div>
      </v-tabs-window-item>
      <v-tabs-window-item
        v-for="stu in store.students"
        :value="stu.student_no"
        :key="stu.student_no"
      >
        <StuTab :stu="stu" />
      </v-tabs-window-item>
    </v-tabs-window>
  </div>

  <!-- 导出弹窗 -->
  <v-dialog v-model="exportDialog" max-width="500">
    <v-card
      title="导出电子表"
      :subtitle="`共 ${summary.total_quantity} 件商品，总价（折后价） ${summary.total_discount_price.toFixed(2)} 元`"
    >
      <div class="px-6 pb-3">
        <v-chip-group v-model="sign_time" selected-class="text-primary">
          <v-chip
            v-for="tag in config?.export_config.time_template ?? []"
            :key="tag"
            :text="tag"
            :value="tag"
          />
        </v-chip-group>

        <v-chip-group
          selected-class="text-primary"
          multiple
          v-model="sign_members"
        >
          <v-chip
            v-for="tag in config?.export_config.template ?? []"
            :key="tag"
            :text="tag"
            :value="tag"
          />
        </v-chip-group>

        <v-text-field
          label="导出签名"
          hide-details
          v-model="sign_string"
          @keydown.enter="
            () => {
              exportFn();
            }
          "
        />
      </div>
      <v-card-actions>
        <v-btn
          text="取消"
          variant="plain"
          @click="
            () => {
              exportDialog = false;
            }
          "
        ></v-btn>

        <v-btn
          text="确认"
          color="primary"
          @click="
            () => {
              exportFn();
            }
          "
          :disabled="exportMutation.isPending.value || !sign_string"
        ></v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { Record } from '@/bindings/entity';
import { ExportRequest } from '@/bindings/mutation';
import type { OrderPagination } from '@/bindings/utils';
import StuTab from '@/components/StuTab.vue';
import { GetStudents, useConfig, useSysTime } from '@/lib/service';
import { useStore } from '@/lib/stores';
import { cn } from '@/lib/utils';
import { invoke } from '@tauri-apps/api/core';
import { info } from '@tauri-apps/plugin-log';
import { onKeyStroke } from '@vueuse/core';
import { AxiosError } from 'axios';
import { Decimal } from 'decimal.js';

const { data: config } = useConfig();

const open_csv = () => {
  const path = config.value?.export_config.path;
  if (!path) {
    toast.error('未配置导出路径，请前往设置配置');
    return;
  }
  invoke('open_web_url', {
    url: path,
  });
};

const queryClient = useQueryClient();

const sign_string = ref('');
const sign_time = ref<string>();
const sign_members = ref<string[]>([]);
const sys_time = useSysTime();
// 生成签名
watch(
  [sign_members, sign_time],
  ([newMembers, newTime]) => {
    const members = [...newMembers];
    if (newTime) {
      let timeStr = sys_time.data.value?.date + ' ' + newTime;
      members.push(timeStr);
    }
    sign_string.value = members.reverse().join(' ');
  },
  { immediate: true, deep: true }
);

const exportDialog = ref(false);

const exportFn = () => {
  return toast.promise(() => exportMutation.mutateAsync(), {
    loading: '正在导出电子表，请勿关闭软件...',
    success: '导出成功',
    error(err: string | AxiosError<{ message: string }>) {
      if (typeof err === 'object' && 'response' in err && err.response) {
        return `导出失败: ${err.response.data.message}`;
      }
      return `导出失败: ${err}`;
    },
  });
};

const exportMutation = useMutation({
  mutationFn: async () => {
    const path = config.value?.export_config.path;
    if (!path) {
      // toast.error('未配置导出路径');
      exportDialog.value = false;
      return Promise.reject('未配置导出路径');
    }

    await invoke('backup');

    await instance.post('/record/export', {
      sign: sign_string.value,
    } satisfies ExportRequest);

    await instance.delete('/record');

    return;
  },
  onSuccess() {
    exportDialog.value = false;
    sign_string.value = '';
    store.clearStudents();
    queryClient.invalidateQueries();
  },
});

// 0: 不分组 1: 按学生分组 2：按商品分组
const groupView = ref(0);

const all_record = useQuery<Record[]>({
  queryKey: ['record', 'get_all_record'],
  queryFn: () => instance.get(`/record`),
});

interface Summary {
  stu_count: number;
  item_count: number;
  total_quantity: number;
  total_discount_price: Decimal;
}

const summary = computed<Summary>(() => {
  const data = all_record.data.value ?? [];

  const stuSet = new Set<string>();
  const itemSet = new Set<number>();
  let total_quantity = 0;
  let total_discount_price = new Decimal(0);
  data.forEach(record => {
    stuSet.add(record.student_no);
    itemSet.add(record.item_id);
    total_quantity += record.quantity;
    total_discount_price = total_discount_price.plus(
      new Decimal(record.discount_price).times(record.quantity)
    );
  });
  return {
    stu_count: stuSet.size,
    item_count: itemSet.size,
    total_quantity,
    total_discount_price,
  };
});

const store = useStore();

// 学号
const tab = ref<string>('登录');

const Params = reactive<OrderPagination>({
  page: 1,
  per_page: 5,
  order_by_key: null,
  order_by_type: null,
  q: '',
});

// 由于keep-alive存在，应避免多处注册esc快捷键
onKeyStroke('Escape', e => {
  e.preventDefault();
  if (Params.q !== '' && tab.value === '登录') {
    Params.q = '';
  }
});

const { data: students } = GetStudents(Params);
</script>

<style scoped>
:deep(.v-tabs-window > div) {
  height: 100% !important;
}

:deep(.v-window__container > div) {
  height: 100% !important;
}
:deep(.v-slide-group__container > .v-slide-group__content) {
  flex-wrap: wrap;
  flex-shrink: 1;
}
</style>
