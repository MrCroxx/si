<template>
  <ScrollArea>
    <template #top>
      <SidebarSubpanelTitle icon="component">
        <template #label>Assets Available to Install</template>
      </SidebarSubpanelTitle>
      <SiSearch
        ref="searchRef"
        placeholder="search assets"
        @search="onSearch"
      />
    </template>
    <RequestStatusMessage
      v-if="syncModulesReqStatus.isPending"
      :requestStatus="syncModulesReqStatus"
      loadingMessage="Loading modules..."
    />
    <div
      v-else-if="moduleStore.installableModules.length > 0"
      class="flex flex-col dark:text-white text-black dark:bg-neutral-800 py-[1px]"
    >
      <div
        v-for="module in filteredModules"
        :key="module.id"
        :class="
          clsx(
            'text-xs w-full p-2xs truncate flex flex-row items-center gap-1 hover:text-action-500 dark:hover:text-action-300 cursor-pointer',
            'hover:dark:outline-action-300 hover:outline-action-500 hover:outline hover:z-10 hover:-outline-offset-1 hover:outline-1',
            selectedModule &&
              module.id === selectedModule.id &&
              themeClasses('bg-action-100', 'bg-action-900'),
          )
        "
        @click="() => selectModule(module)"
      >
        <Icon name="component" size="sm" />
        <div class="truncate">
          {{ module.name }}
        </div>
      </div>
    </div>
    <EmptyStateCard
      v-else
      iconName="no-assets"
      primaryText="No Installable Assets"
      secondaryText="Check back later when more assets are contributed."
    />
  </ScrollArea>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import clsx from "clsx";
import { ref, computed, onMounted } from "vue";
import {
  ScrollArea,
  RequestStatusMessage,
  themeClasses,
  Icon,
  SiSearch,
} from "@si/vue-lib/design-system";
import { useRoute } from "vue-router";
import router from "@/router";
import { useModuleStore } from "@/store/module.store";
import { LatestModule } from "@/api/sdf/dal/module";
import EmptyStateCard from "./EmptyStateCard.vue";
import SidebarSubpanelTitle from "./SidebarSubpanelTitle.vue";

const moduleStore = useModuleStore();
const route = useRoute();

const searchRef = ref<InstanceType<typeof SiSearch>>();
const searchString = ref("");

const onSearch = (search: string) => {
  searchString.value = search.trim().toLocaleLowerCase();
};

const filteredModules = computed(() =>
  moduleStore.installableModules.filter((m) =>
    m.name.toLocaleLowerCase().includes(searchString.value),
  ),
);

const syncModulesReqStatus = moduleStore.getRequestStatus("SYNC");

const selectedModule = ref<LatestModule | undefined>();

const selectModule = (module: LatestModule) => {
  selectedModule.value = module;
  const newQueryObj = {
    ...{ m: module.id },
  };
  router.replace({
    query: newQueryObj,
  });
};

onMounted(() => {
  if (route.query.s) {
    router.replace({ query: {} });
  }
});

defineExpose({ selectedModule });
</script>
