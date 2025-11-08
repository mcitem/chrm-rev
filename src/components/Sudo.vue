<!-- sudo授权模态框(弹窗) -->
<template>
  <Transition
    enter-active-class="transition-opacity duration-300"
    leave-active-class="transition-opacity duration-300"
    enter-from-class="opacity-0"
    leave-to-class="opacity-0"
  >
    <div
      v-if="sudoRequest"
      class="fixed inset-0 z-[888] flex select-none items-center justify-center bg-[#00000077]"
    >
      <Transition
        appear
        enter-active-class="transition-all duration-300 ease-[cubic-bezier(0.4,0,0.2,1)]"
        leave-active-class="transition-all duration-300 ease-[cubic-bezier(0.4,0,0.2,1)]"
        enter-from-class="scale-90 opacity-0"
        leave-to-class="scale-90 opacity-0"
      >
        <div
          ref="target"
          class="max-h-[80vh] w-[50vw] overflow-y-auto rounded bg-[#eeeeee] p-6 shadow-lg dark:bg-[#2c2c2c]"
        >
          <h2 class="mb-1 text-2xl font-bold">管理员验证</h2>
          <p class="mb-2 font-medium"></p>
          <v-text-field
            v-model="password"
            label="请输入管理员密码"
            placeholder="输入管理员密码以继续操作"
            outlined
            dense
            class="w-full"
            autofocus
            @keyup.enter="confirm"
          />
          <div class="flex justify-center">
            <v-checkbox
              v-model="rememberMe"
              label="下次启动前不再询问"
              class="mr-auto"
              density="compact"
              hide-details
            />
            <v-btn color="primary" class="ml-2" @click="confirm">确认</v-btn>
            <v-btn color="secondary" class="ml-2" @click="reject">取消</v-btn>
          </div>
        </div>
      </Transition>
    </div>
    <div v-else></div>
  </Transition>
</template>

<script setup lang="ts">
import { SudoResponse } from '@/bindings/sudo';
import { emit, listen } from '@tauri-apps/api/event';
import { onKeyStroke } from '@vueuse/core';
import { onClickOutside } from '@vueuse/core';
import { useTemplateRef } from 'vue';

const password = ref('');
const rememberMe = ref(false);

// 监听sudo授权请求
const sudoRequest = ref(false);
listen('mcitem://sudo/request', () => {
  sudoRequest.value = true;
});

const confirm = () => {
  sudoRequest.value = false;
  emit('mcitem://sudo/response', {
    password: password.value,
    remember: rememberMe.value,
  } satisfies SudoResponse);
  password.value = '';
};

const reject = () => {
  sudoRequest.value = false;
  // 不提交内容，表示拒绝
  emit('mcitem://sudo/response');
};

// 监听Esc键关闭
onKeyStroke('Escape', e => {
  e.preventDefault();
  reject();
});

// 点击外部关闭
const target = useTemplateRef<HTMLDivElement>('target');
onClickOutside(target, () => {
  reject();
});
</script>
