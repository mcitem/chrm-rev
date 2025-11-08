<template>
  <div class="px-2 py-2">
    <v-btn
      color="primary"
      @click="
        () => {
          export_all.mutate();
        }
      "
      >导出所有</v-btn
    >
  </div>
  <!-- <div class="px-2">
    以下功能尚不可用
    <div class="flex justify-start gap-2 py-2">
      <v-btn>导出学生</v-btn>
      <v-btn>导出商品</v-btn>
    </div>

    <div class="flex justify-start gap-2 py-2">
      <v-btn color="primary">下载导入模版</v-btn>
    </div>
    <div class="flex justify-start gap-2 py-2">
      <v-btn>导入学生（增量）</v-btn>
      <v-btn>导入商品（增量）</v-btn>
      <v-btn>导入学生（替换）</v-btn>
      <v-btn>导入商品（替换）</v-btn>
    </div>

    <div class="flex justify-start gap-2 py-2">
      <v-btn>清空学生</v-btn>
      <v-btn>清空商品</v-btn>
    </div>

    <div class="flex justify-start gap-2 py-2">
      <v-btn color="red">重置所有学生余额</v-btn>
    </div>

    <div>执行SQL（Sqlite）</div>
  </div> -->
</template>

<script setup lang="ts">
import { useSysTime } from '@/lib/service';
import { desktopDir } from '@tauri-apps/api/path';

const sysTime = useSysTime();
//
const export_all = useMutation({
  mutationFn: async () => {
    const desktop_path = await desktopDir();
    const rawTime = sysTime.data.value?.time ?? '';
    const time = rawTime.replace(/\D+/g, '');
    const path = desktop_path + '\\' + time + '主动导出.xlsx';

    return instance.post('/sys/export_all', {
      path,
    });
  },
  onSuccess: () => {
    toast.success('导出成功，文件已保存到桌面');
  },
});
</script>
