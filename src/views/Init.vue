<template>
  <div className="pl-4">
    <div>初始化导入</div>
    <div>{{ MigrateUpBody.path ?? '' }}</div>
    <div className="space-x-3 pt-3">
      <v-btn
        @click="
          () => {
            MigrateUpBody = { ...MigrateUpBody, atype: 'Skip' };
            // console.log(MigrateUpBody);
            MigrateUpMutation.mutate();
          }
        "
      >
        仅初始化，暂不导入数据
      </v-btn>
      <v-btn @click="onDownloadTemplate"> 下载模板 </v-btn>
      <v-btn @click="onTryOpenFile" variant="outlined"> 选择文件 </v-btn>
      <v-btn
        @click="
          () => {
            MigrateUpMutation.mutate();
          }
        "
        :disabled="MigrateUpBody.path === placeholder"
      >
        {{ MigrateUpBody.atype === 'AnalyzeOnly' ? '解析' : '导入' }}
      </v-btn>
    </div>
    <div class="grid grid-cols-2 pt-4">
      <div>
        解析的商品第一行数据：

        <div>商品名称 {{ item0?.name ?? '无' }}</div>
        <div>规格 {{ item0?.spec ?? '无' }}</div>
        <div>3折价 {{ item0?.p_hard ?? '无' }}</div>
        <div>5折价 {{ item0?.p_easy ?? '无' }}</div>
        <div>7折价 {{ item0?.p_normal ?? '无' }}</div>
        <div>原价 {{ item0?.price ?? '无' }}</div>
        <div>积分 {{ item0?.p_score ?? '无' }}</div>
      </div>
      <div>
        解析的学生第一行数据
        <div>姓名 {{ stu0?.name ?? '无' }}</div>
        <div>学号 {{ stu0?.student_no ?? '无' }}</div>
        <div>认定级别 {{ stu0?.difficulty_level ?? '无' }}</div>
        <div>学院 {{ stu0?.secondary_school ?? '无' }}</div>
        <div>班级 {{ stu0?.class ?? '无' }}</div>
        <div>性别 {{ stu0?.sex ?? '无' }}</div>
        <div>余额 {{ stu0?.balance ?? '无' }}</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Item, Student } from '@/bindings/entity';
import type { MigrateUp } from '@/bindings/mutation';
import { listen, TauriEvent } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import { writeFile, BaseDirectory } from '@tauri-apps/plugin-fs';

const queryClient = useQueryClient();
const placeholder = '将需要导入的数据文件拖入窗口';
const err = '请选择一个有效的 Excel(.xls/.xlsx) 文件';
const MigrateUpBody = ref<MigrateUp>({
  path: placeholder,
  atype: 'AnalyzeOnly',
});
// 路径被替换时重置状态
watch(
  () => MigrateUpBody.value.path,
  (newValue, oldValue) => {
    if (newValue !== oldValue) {
      MigrateUpBody.value.atype = 'AnalyzeOnly';
      item0.value = undefined;
      stu0.value = undefined;
    }
  }
);

const stu0 = ref<Student | undefined>();

const item0 = ref<Item | undefined>();

// 监听文件拖放
let unlisten = () => {};
listen<{ paths: string[] }>(TauriEvent.DRAG_DROP, e => {
  const path = e.payload.paths[0];
  if (path.endsWith('.xlsx') || path.endsWith('.xls')) {
    MigrateUpBody.value.path = path;
  } else {
    toast.error(err);
  }
}).then(un => {
  unlisten = un;
});
onUnmounted(() => {
  unlisten();
});

const MigrateUpMutation = useMutation<{
  item?: Item;
  student?: Student;
  i_count?: number;
  s_count?: number;
}>({
  mutationKey: ['/migration/up', MigrateUpBody.value],
  mutationFn: () => instance.post('/migration/up', MigrateUpBody.value),
  onSuccess(data) {
    const atype = MigrateUpBody.value.atype;

    item0.value = data.item;
    stu0.value = data.student;

    if (atype === 'AnalyzeOnly') {
      MigrateUpBody.value.atype = 'Import';
      return;
    }

    if (
      atype === 'Import' &&
      data.i_count !== undefined &&
      data.s_count !== undefined
    ) {
      toast.success(
        `导入成功！商品 ${data.i_count} 条，学生 ${data.s_count} 条`
      );
    }

    if (atype === 'Import' || atype === 'Skip') {
      router.push('/');
    }
    queryClient.invalidateQueries();
  },
});

const onDownloadTemplate = async () => {
  const res = await fetch('/chrm-rev导入模板.xlsx');
  const arrayBuffer = await res.arrayBuffer();
  await writeFile('chrm-rev数据导入模板.xlsx', new Uint8Array(arrayBuffer), {
    baseDir: BaseDirectory.Desktop,
  });
  toast.success('模板已保存至桌面');
};

const onTryOpenFile = () => {
  open({ multiple: false }).then(path => {
    if (path && (path.endsWith('.xlsx') || path.endsWith('.xls'))) {
      MigrateUpBody.value.path = path;
    } else {
      toast.error(err);
    }
  });
};
</script>
