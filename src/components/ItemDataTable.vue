<template>
  <div class="flex h-full flex-col">
    <v-text-field
      class="block flex-shrink-0 flex-grow-0"
      aria-autocomplete="none"
      hide-details
      v-model="Params.q"
      placeholder="搜索"
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
        :items="items?.data ?? []"
        :items-length="items?.length ?? 0"
        :items-per-page-options="[10, 15, 20, 50, 100]"
        :loading="isFetching"
      >
        <!-- 主键列 -->
        <template v-slot:item.id="{ value }">
          <v-chip
            border="thin opacity-25"
            :text="String(value).padStart(3, '0')"
            label
          />
        </template>

        <!-- <template v-slot:item.credit="{ item }">
      {{ item.credit ?? '未使用过' }}
    </template> -->

        <!-- 操作列 -->
        <template v-slot:item.action="{ item }">
          <v-btn
            size="small"
            density="comfortable"
            variant="flat"
            icon
            @click="
              () => {
                // 深拷贝一份商品数据
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
            v-tooltip:top="`添加商品`"
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
  <!-- 编辑弹出对话框 -->
  <v-dialog v-model="dialog" max-width="500">
    <v-card :title="`${isEditing ? '编辑' : '添加'}商品`">
      <template v-slot:text>
        <v-row>
          <v-col cols="6">
            <v-text-field v-model="formModel.name" label="商品名称" />
          </v-col>
          <v-col cols="3">
            <v-text-field v-model="formModel.spec" label="规格" />
          </v-col>
          <v-col cols="3">
            <v-text-field v-model="formModel.p_score" label="积分" />
          </v-col>
          <v-col cols="3">
            <v-text-field v-model="formModel.price" label="原价" />
          </v-col>
          <v-col cols="3">
            <v-text-field v-model="formModel.p_easy" label="一般困难" />
          </v-col>
          <v-col cols="3">
            <v-text-field v-model="formModel.p_normal" label="困难" />
          </v-col>
          <v-col cols="3">
            <v-text-field v-model="formModel.p_hard" label="特别困难" />
          </v-col>
          <v-col cols="12">
            <v-text-field
              :model-value="formModel.tags"
              @update:model-value="
                value => {
                  formModel.tags = value;
                }
              "
              label="标签"
            />
          </v-col>
          <v-col cols="12">
            <v-text-field
              :model-value="formModel.id"
              @update:model-value="
                value => {
                  formModel.id = Number(value);
                }
              "
              label="ID"
              hint="请确保ID唯一，否则这将将覆盖其他已有项目的数据"
              persistent-hint
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
      :subtitle="`确定要删除 #${formModel.id} ${formModel.name}(${formModel.spec}) 吗？`"
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
import type { Item } from '@/bindings/entity';
import type { Order, OrderPagination } from '@/bindings/utils';
import { GetItems } from '@/lib/service';
import { sudo } from '@/lib/utils';
import type { DataTableHeader } from 'vuetify';
import type { VDataTableServer } from 'vuetify/components';

const menu = ref(false);
const dialog = ref(false);
const isEditing = ref(false);
const deleteConfirmDialog = ref(false);

const defaultFormModel: Item = {
  id: -1,
  name: '',
  spec: '',
  price: '',
  p_easy: '',
  p_normal: '',
  p_hard: '',
  p_score: '',
  tags: '',
};

const formModel = ref<Item>(defaultFormModel);

const queryClient = useQueryClient();

const updateStudent = useMutation({
  mutationFn: (stu: Item) => instance.put(`/item/${stu.id}`, stu),
  onSuccess: (_, stu) => {
    toast.success(`更新 ${stu.name}(${stu.spec}) 成功`);
    queryClient.invalidateQueries({
      queryKey: ['/item/list'],
    });
  },
  onError: (_, stu) => {
    toast.error(`更新 ${stu.name}(${stu.spec}) 失败`);
  },
});
const deleteStudent = useMutation({
  mutationFn: (stu: Item) => instance.delete(`/item/${stu.id}`),
  onSuccess: (_, stu) => {
    toast.success(`删除 ${stu.name}(${stu.spec}) 成功`);
    queryClient.invalidateQueries({
      queryKey: ['/item/list'],
    });
  },
  onError: (_, stu) => {
    toast.error(`删除 ${stu.name}(${stu.spec}) 失败`);
  },
});
const addStudent = useMutation({
  mutationFn: (stu: Item) => instance.post(`/item`, stu),
  onSuccess: (_, stu) => {
    toast.success(`添加 ${stu.name}(${stu.spec}) 成功`);
    queryClient.invalidateQueries({
      queryKey: ['/item/list'],
    });
  },
  onError: (_, stu) => {
    toast.error(`添加 ${stu.name}(${stu.spec}) 失败`);
  },
});

type HR<T> = (DataTableHeader<T> & {
  hide?: boolean;
  key: keyof T | 'action';
})[];
const HeadRaw: Ref<HR<Item>> = ref([
  {
    key: 'action',
    title: '编辑',
    sortable: false,
  },
  {
    key: 'id',
    title: 'ID',
  },
  { key: 'name', title: '名称', fixed: 'start' },
  {
    key: 'spec',
    title: '规格',
  },
  {
    key: 'price',
    title: '原价',
  },
  {
    key: 'p_easy',
    title: '一般困难',
  },
  {
    key: 'p_normal',
    title: '困难',
  },
  {
    key: 'p_hard',
    title: '特别困难',
  },
  {
    key: 'p_score',
    title: '积分',
  },
  {
    key: 'tags',
    title: '标签',
  },
] satisfies HR<Item>);

const Head = computed(() => {
  const tmp = HeadRaw.value
    .filter(h => h.hide !== true)
    .map(
      h =>
        ({
          ...h,
          align: 'center',
        }) satisfies DataTableHeader<Item>
    );
  return tmp;
});

const Params = reactive<OrderPagination>({
  page: 1,
  per_page: 50,
  order_by_key: null,
  order_by_type: null,
  q: null,
});

const { data: items, isFetching } = GetItems(Params);
</script>
