<template>
  <div>{{ version }}</div>
  <div class="flex justify-start gap-2 py-2">
    <v-btn-toggle
      v-tooltip="`切换主题模式`"
      v-model="store"
      color="#075cb2"
      rounded
      divided
      mandatory
      density="compact"
    >
      <v-btn value="auto">自动</v-btn>
      <v-btn value="light">浅色</v-btn>
      <v-btn value="dark">深色</v-btn>
    </v-btn-toggle>
  </div>

  <div class="flex justify-start gap-2 py-2">
    <!-- <v-btn
      v-tooltip:top="`立即备份数据库，(按当前时间命名)`"
      @click="
        () => {
          // invoke('backup');
        }
      "
    >
      立即备份
    </v-btn> -->
    <v-btn
      v-tooltip:top="`立即从文件读取配置文件并应用`"
      @click="
        () => {
          instance.get('/sys/config/reload').then(() => {
            queryClient.invalidateQueries();
          });
        }
      "
    >
      重新加载配置
    </v-btn>
    <!-- <v-btn @click="() => {}" color="red" v-tooltip:top="`清空电子表中的数据`">
      清空电子表
    </v-btn> -->
    <v-btn
      @click="on_reset_db"
      color="red"
      v-tooltip:top="`重置数据库到未初始化状态`"
    >
      重置数据库
    </v-btn>
  </div>
  <div class="flex justify-start gap-2 py-2">
    <v-btn
      v-tooltip:top="`打开配置、数据库存储的数据目录`"
      @click="
        () => {
          invoke('open_data_dir');
        }
      "
    >
      数据目录
    </v-btn>
    <v-btn
      v-tooltip:top="`打开exe程序所在目录`"
      @click="
        () => {
          invoke('open_core_dir');
        }
      "
    >
      核心目录
    </v-btn>
    <v-btn
      v-tooltip:top="`打开日志文件存储目录`"
      @click="
        () => {
          invoke('open_logs_dir');
        }
      "
    >
      日志目录
    </v-btn>

    <v-btn
      v-tooltip:top="`打开windows系统时间设置界面`"
      @click="
        () => {
          invoke('open_timedate_cpl');
        }
      "
    >
      修改系统时间
    </v-btn>
    <v-btn
      v-tooltip:top="`打开F12开发者工具`"
      @click="
        () => {
          invoke('open_devtools');
        }
      "
    >
      devtools
    </v-btn>
  </div>
  <div>
    {{ config ?? '加载中...' }}
  </div>
  <!-- <div>设置导出路径</div> -->
  <!-- <div>
    <v-text-field v-model="darftEXportPath" label="导出路径" />
  </div> -->
  <!-- <div>
    <v-btn
      @click="
        () => {
          instance.delete('/sys/config/exportPath').then(() => {
            queryClient.invalidateQueries({ queryKey: ['/sys/config'] });
          });
        }
      "
    >
      清除
    </v-btn>
    <v-btn
      @click="
        () => {
          setEXportPathMutation.mutate(true);
        }
      "
    >
      设置增强路径
    </v-btn>
    <v-btn
      @click="
        () => {
          setEXportPathMutation.mutate(false);
        }
      "
    >
      设置
    </v-btn>
  </div> -->
</template>

<script setup lang="ts">
// const darftEXportPath = ref('C:/Users/mcitem/Desktop/1.xlsx');
// const setEXportPathMutation = useMutation({
//   mutationFn: (advance: boolean) => {
//     return instance.post(
//       advance ? '/sys/config/AdvanceExportPath' : '/sys/config/exportPath',
//       {
//         path: darftEXportPath.value,
//       }
//     );
//   },
//   onSuccess: () => {
//     toast.success('设置成功');
//     queryClient.invalidateQueries({ queryKey: ['/sys/config'] });
//   },
//   onError: (err: AxiosError<{ message: string }>) => {
//     toast.error(err?.response?.data?.message || '设置失败');
//   },
// });
import { type AppContext } from '@/App.vue';
import { useConfig } from '@/lib/service';
import { instance } from '@/lib/service';
import { sudo } from '@/lib/utils';
import { version } from '@root/package.json';
import { invoke } from '@tauri-apps/api/core';
import { useColorMode } from '@vueuse/core';

const queryClient = useQueryClient();
// import { AxiosError } from 'axios';

const { store } = useColorMode();

// const queryClient = useQueryClient();
const { data: config } = useConfig();

const appContext = inject<AppContext>('AppContext')!;

const on_reset_db = () => {
  sudo().then(() => {
    instance.get(`/migration/down`).then(() => {
      queryClient.clear();
      appContext.refetch();
    });
  });
};
</script>
