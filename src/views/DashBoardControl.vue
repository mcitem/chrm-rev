<template>
  <div class="flex h-screen flex-col">
    <v-tabs grow v-model="tab" class="flex-shrink-0 flex-grow-0">
      <v-tab value="1">商品管理</v-tab>
      <v-tab value="2">学生管理</v-tab>
      <v-tab value="3">批量管理</v-tab>
    </v-tabs>

    <v-tabs-window v-model="tab" class="flex-grow overflow-y-auto">
      <v-tabs-window-item value="1">
        <ItemDataTable />
      </v-tabs-window-item>
      <v-tabs-window-item value="2">
        <StuDataTable />
      </v-tabs-window-item>
      <v-tabs-window-item value="3">
        <Management />
      </v-tabs-window-item>
    </v-tabs-window>
  </div>
</template>
<script setup lang="ts">
import ItemDataTable from '@/components/ItemDataTable.vue';
import Management from '@/components/Management.vue';
import StuDataTable from '@/components/StuDataTable.vue';

const tab = ref('1');
</script>

<style scoped>
:deep(.v-tabs-window > div) {
  height: 100% !important;
}
:deep(.v-window__container > div) {
  height: 100% !important;
}
</style>
