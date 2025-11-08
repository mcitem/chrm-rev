<template>
  <div class="relative flex h-full flex-col">
    <div class="flex-shrink-0 flex-grow-0 py-3 pl-8">
      <span class="text-2xl font-semibold">
        {{ student?.name }}
      </span>
      <span class="ml-1"> ({{ student?.sex }}) </span>
      <v-chip style="vertical-align: top">
        {{ student?.difficulty_level }}
      </v-chip>
    </div>

    <v-row class="flex-shrink-0 flex-grow-0 px-2 lg:w-[66%]">
      <v-col cols="5"> 院系: {{ student?.secondary_school }} </v-col>
      <v-col cols="5"> 班级: {{ student?.class }} </v-col>
      <v-col cols="5"> 学号: {{ student?.student_no }} </v-col>

      <v-col cols="5" v-if="conf.data.value?.balance_config.active || false">
        余额: {{ student?.balance ?? '0.00' }}
      </v-col>
    </v-row>

    <div class="relative flex-shrink-0 flex-grow-0">
      <v-text-field
        ref="inputRef"
        aria-autocomplete="none"
        v-model="v"
        style="margin-top: 1rem"
        placeholder="添加商品 支持 0xx（添加指定编号） =xx（倍数添加上一个）"
        clearable
        hide-details
        @keydown.esc="
          () => {
            v = '';
          }
        "
      />

      <transition
        enter-active-class="transition-all duration-200"
        enter-from-class="opacity-0 -translate-y-2"
        enter-to-class="opacity-100 translate-y-0"
        leave-active-class="transition-all duration-200"
        leave-from-class="opacity-100 translate-y-0"
        leave-to-class="opacity-0 -translate-y-2"
      >
        <div
          v-if="
            Params.q &&
            !Params.q.startsWith('0') &&
            !Params.q.startsWith('=') &&
            Params.q !== '' &&
            items?.length === 0
          "
          class="absolute z-10 w-full bg-[#f3f4f5] px-4 py-2 text-center shadow-md dark:bg-[#1a1a1a]"
        >
          无数据
        </div>
        <div
          v-else-if="
            Params.q &&
            !Params.q.startsWith('0') &&
            !Params.q.startsWith('=') &&
            Params.q.trim() !== '' &&
            items?.length
          "
          class="absolute z-10 w-full bg-[#f3f4f5] shadow-md dark:bg-[#1a1a1a]"
        >
          <button
            v-for="(item, index) in items.data"
            :key="index"
            class="block w-full border-b border-gray-200 px-4 py-2 text-start"
            @click="
              () => {
                let exists =
                  record_item.data.value?.find(ri => ri.item_id === item.id)
                    ?.quantity ?? 0;

                set_item.mutate({
                  item_id: item.id,
                  quantity: exists + 1,
                });
                v = '';
                inputRef?.focus();
              }
            "
          >
            #{{ item.id }} - {{ item.name }} - {{ item.spec }}
          </button>
        </div>
        <!-- 不能移除的v-else!!!!,会导致keepalive出错 -->
        <div v-else></div>
      </transition>
    </div>
    <!-- 已购视图 -->

    <!-- <div>{{ record_item.data.value }}</div> -->
    <div class="flex flex-grow flex-col overflow-hidden">
      <div class="flex-grow overflow-hidden">
        <v-data-table
          ref="tableRef"
          :items="record_item.data.value"
          fixed-header
          class="h-full whitespace-nowrap"
          items-per-page="-1"
          :sort-by="[{ key: 'id', order: 'asc' }]"
          :headers="
            [
              // { key: `id`, title: `record序号` },
              { key: `item_id`, title: `商品编号` },
              { key: `item_name`, title: `名称` },
              { key: `item_spec`, title: `规格` },
              // { key: `student_no`, title: `学号` },
              // { key: `stu_d_level`, title: `认定级别` },
              { key: `discount_price`, title: `折后价` },
              { key: `original_price`, title: `原价` },
              { key: `quantity`, title: `数量` },
              { key: `action`, title: `操作`, sortable: false },
            ].map(h => ({ ...h, sortable: false, align: 'center' }))
          "
        >
          <template #bottom>
            <div class="flex justify-end px-4 pb-4 pt-3">
              共计 {{ summary.item_count }} 种商品，总共
              {{ summary.total_quantity }} 件商品，总价（折后价）
              {{ summary.total_discount_price.toFixed(2) }} 元
            </div>
          </template>
          <template #item.action="{ item }">
            <v-btn
              icon
              size="small"
              density="comfortable"
              variant="flat"
              @click="
                () => {
                  let exists =
                    record_item.data.value?.find(
                      ri => ri.item_id === item.item_id
                    )?.quantity ?? 1;

                  set_item.mutate({
                    item_id: item.item_id,
                    quantity: exists + 1,
                  });
                }
              "
            >
              <v-icon>mdi-plus</v-icon>
            </v-btn>
            <v-btn
              icon
              size="small"
              density="comfortable"
              variant="flat"
              @click="
                () => {
                  let exists =
                    record_item.data.value?.find(
                      ri => ri.item_id === item.item_id
                    )?.quantity ?? 1;

                  if (exists <= 1) {
                    delete_item.mutate(item.item_id);
                  } else {
                    set_item.mutate({
                      item_id: item.item_id,
                      quantity: exists - 1,
                    });
                  }
                }
              "
            >
              <v-icon>mdi-minus</v-icon>
            </v-btn>
            <v-btn
              icon
              size="small"
              density="comfortable"
              variant="flat"
              @click="
                () => {
                  let exists =
                    record_item.data.value?.find(
                      ri => ri.item_id === item.item_id
                    )?.quantity ?? 1;

                  delete_item.mutate(item.item_id);

                  toast(`删除了${student?.name}-${item.item_name}`, {
                    action: {
                      label: '反悔',
                      onClick: () => {
                        set_item.mutate({
                          item_id: item.item_id,
                          quantity: exists,
                        });
                      },
                    },
                  });
                }
              "
            >
              <v-icon color="red">mdi-delete</v-icon>
            </v-btn>
          </template>
        </v-data-table>
      </div>
    </div>

    <!-- 余额不足提示-->
    <div
      v-if="ifBalanceLimit"
      class="absolute inset-0 z-50 flex h-full w-full select-none items-center justify-center bg-black/30"
    >
      <div class="rounded-md bg-red-800 p-6">
        <div>余额不足提示</div>
        <div></div>
        <div>
          <v-btn
            @click="
              () => {
                ifBalanceLimit = false;
              }
            "
            >已阅</v-btn
          >
        </div>
      </div>
    </div>
    <!--  -->
  </div>
</template>
<script setup lang="ts">
import type { Record, Student } from '@/bindings/entity';
import { OrderPagination } from '@/bindings/utils';
import { GetItems, useConfig } from '@/lib/service';
import { StuTab } from '@/lib/stores/default';
import Decimal from 'decimal.js';
import { toast } from 'vue-sonner';
import { VDataTable } from 'vuetify/components';

const conf = useConfig();

// 点击列表后自动聚焦输入支持
const inputRef = useTemplateRef('inputRef');

// 搜索提示支持
const Params = reactive<OrderPagination>({
  page: 1,
  per_page: 5,
  order_by_key: null,
  order_by_type: null,
  q: '',
});

const { data: items } = GetItems(Params);

// 滚动支持，自动滚动表格
const tableRef = useTemplateRef('tableRef');
const queryClient = useQueryClient();

const props = defineProps<{
  stu: StuTab;
}>();

interface Summary {
  item_count: number;
  total_quantity: number;
  total_discount_price: Decimal;
}
const summary = computed<Summary>(() => {
  const items = record_item.data.value ?? [];
  return {
    item_count: items.length,
    total_quantity: items.reduce((acc, item) => acc + item.quantity, 0),
    total_discount_price: items.reduce(
      (acc, item) =>
        acc.plus(new Decimal(item.discount_price).times(item.quantity)),
      new Decimal(0)
    ),
  };
});

const { data: student } = useQuery<Student>({
  queryKey: ['student', props.stu.id],
  queryFn: () => instance.get(`/student/${props.stu.id}`),
});

// 余额不足提示
const ifBalanceLimit = ref(false);
watch(
  () => student.value?.balance,
  (newValue, oldValue) => {
    // 确保余额功能启用
    if (!conf.data.value?.balance_config.active) return;
    if (newValue === undefined || newValue === null) return;
    if (oldValue === undefined || oldValue === null) return;
    if (conf.data.value?.balance_config.active) {
      let newBalance = new Decimal(newValue);
      let oldBalance = new Decimal(oldValue);

      let limit = new Decimal(
        conf.data.value?.balance_config.tolerance ?? '0.00'
      )
        .abs()
        .neg();

      if (
        newBalance.lessThan(limit) &&
        oldBalance.greaterThanOrEqualTo(limit)
      ) {
        ifBalanceLimit.value = true;

        nextTick(() => {
          inputRef?.value?.blur();
        });
      }
    }
  }
);

/**
 * 已购(记录)商品列表
 */
const record_item = useQuery<Record[]>({
  queryKey: ['record', props.stu.id],
  queryFn: () => instance.get(`/record/${props.stu.id}/item`),
});

watch(record_item.data, (newValue, oldValue) => {
  // 自动滚动到底部
  if (tableRef.value === null) return;

  const tableEl = tableRef.value.$el;

  if (
    newValue === undefined ||
    oldValue === undefined ||
    !(tableEl instanceof HTMLElement)
  )
    return;

  const scrollEl = tableEl.querySelector('.v-table__wrapper');

  if (!(scrollEl instanceof HTMLDivElement)) return;

  if (newValue.length > oldValue.length) {
    // 新增了商品，滚动到底部

    nextTick(() => {
      scrollEl.scrollTo({
        top: scrollEl.scrollHeight,
        behavior: 'smooth',
      });
    });
  }
});

const set_item = useMutation({
  mutationFn: (data: { item_id: number; quantity: number }) =>
    instance.post(`/record/${props.stu.id}/item/${data.item_id}`, {
      quantity: data.quantity,
    }),
  onMutate(variables) {
    info(
      `${student.value?.name} 设置商品: 编号${variables.item_id} - 数量${variables.quantity}`
    );
  },
  onSuccess(_, variables) {
    lastItemId.value = variables.item_id;

    queryClient.invalidateQueries({ queryKey: ['record', props.stu.id] });
    queryClient.invalidateQueries({ queryKey: ['student', props.stu.id] });
    queryClient.invalidateQueries({ queryKey: ['record', 'get_all_record'] });
  },
});

const delete_item = useMutation({
  mutationFn: (item_id: number) =>
    instance.delete(`/record/${props.stu.id}/item/${item_id}`),
  onMutate(variables) {
    info(`${student.value?.name} 删除商品，id: ${variables}`);
  },
  onSuccess() {
    queryClient.invalidateQueries({ queryKey: ['record', props.stu.id] });
    queryClient.invalidateQueries({ queryKey: ['student', props.stu.id] });
    queryClient.invalidateQueries({ queryKey: ['record', 'get_all_record'] });
  },
});

const lastItemId = ref(1);
const v = ref('');
watch(v, newValue => {
  // 以防vuetify的clearable功能
  if (newValue === null || newValue === undefined) {
    // 同步到搜索参数
    Params.q = '';
    return;
  }

  // 同步到搜索参数
  Params.q = newValue;

  if (newValue === '') return;

  // 假定编号范围000-099
  if (newValue.startsWith('0') && /^\d{3,}$/.test(newValue)) {
    const item_id = parseInt(newValue);
    const quantity =
      (record_item.data.value?.find(item => item.item_id === item_id)
        ?.quantity ?? 0) + 1;
    set_item.mutate({
      item_id,
      quantity,
    });
    v.value = '';
  }

  if (newValue.startsWith('=') && /^\d{2,}$/.test(newValue.slice(1))) {
    set_item.mutate({
      item_id: lastItemId.value,
      quantity: parseInt(newValue.slice(1)),
    });
    v.value = '';
  }

  // if (newValue.startsWith('-') && /^\d{2,}$/.test(newValue.slice(1))) {
  //   delete_item.mutate(parseInt(newValue.slice(1)));
  //   v.value = '';
  // }
});
</script>
