<template>
  <th
    ref="thRef"
    v-tooltip="tooltip"
    :colSpan="header.colSpan"
    :class="
      clsx(
        'h-8 sticky top-0',
        header.id !== 'json' && 'cursor-pointer hover:underline',
        header.id === 'json' && 'w-8 px-2xs',
        themeClasses('bg-neutral-400', 'bg-shade-100'),
      )
    "
    @mousedown="startActive"
    @mouseup="endActive"
    @click.stop="onClick"
  >
    <div
      :class="
        clsx(
          header.id !== 'json' && 'w-full p-xs truncate',
          windowWidth <= 1100 && 'flex flex-row items-center h-8',
        )
      "
    >
      <IconButton
        v-if="header.id === 'json' && anyRowsOpen"
        icon="collapse-row"
        iconTone="neutral"
        size="sm"
        @click.stop="onClick"
      />
      <template v-else>
        <template v-if="windowWidth > 1100">
          <FlexRender
            v-if="!header.isPlaceholder"
            :render="label"
            :props="header.getContext()"
          />
          <IconButton
            v-if="icon !== 'none'"
            ref="iconButtonRef"
            class="absolute right-xs top-2xs"
            :icon="icon"
            iconTone="neutral"
            @click.stop="onClick"
          />
        </template>
        <template v-else>
          <div class="flex-grow">{{ label }}</div>
          <IconButton
            v-if="icon !== 'none'"
            ref="iconButtonRef"
            class="flex-none"
            :icon="icon"
            iconTone="neutral"
            @click.stop="onClick"
          />
        </template>
        <DropdownMenu
          v-if="header.id !== 'timestamp' && header.id !== 'json'"
          ref="dropdownMenuRef"
          :items="dropdownMenuItems"
          :anchorTo="{ $el: thRef }"
          alignCenter
        />
      </template>
    </div>
  </th>
</template>

<script lang="ts" setup>
import {
  DropdownMenu,
  DropdownMenuItemObjectDef,
  IconButton,
  themeClasses,
} from "@si/vue-lib/design-system";
import { FlexRender, Header } from "@tanstack/vue-table";
import clsx from "clsx";
import { computed, onMounted, onUnmounted, ref } from "vue";
import { useLogsStore } from "@/store/logs.store";

const thRef = ref();
const iconButtonRef = ref<InstanceType<typeof IconButton>>();
const dropdownMenuRef = ref<InstanceType<typeof DropdownMenu>>();

const props = defineProps<{
  anyRowsOpen: boolean;
  header: Header<
    {
      title: string;
      userName: string;
      userId?: string;
      userEmail?: string;
      kind: string;
      entityType: string;
      entityName: string;
      timestamp: string;
      changeSetId?: string;
      changeSetName: string;
      metadata: Record<string, unknown>;
    },
    unknown
  >;
}>();

const logsStore = useLogsStore();

const label = computed(() => props.header.column.columnDef.header as string);

const icon = computed(() => {
  // NOTE(nick): restore timestamp sort after audit trail is shipped.
  // if (props.header.id === "timestamp") {
  // if (props.filters.sortTimestampAscending) return "chevron--up";
  // else return "chevron--down";
  // } else if (selectedFilters.value.length > 0) {
  if (selectedFilters.value.length > 0) {
    return "filter";
  }
  return "none";
});

const filterOptions = computed(() => {
  if (props.header.id === "changeSetName") {
    return logsStore.headerOptions.changeSet;
  } else if (props.header.id === "entityName") {
    return logsStore.headerOptions.entityName;
  } else if (props.header.id === "entityType") {
    return logsStore.headerOptions.entityType;
  } else if (props.header.id === "title") {
    return logsStore.headerOptions.title;
  } else if (props.header.id === "userName") {
    return logsStore.headerOptions.user;
  }
  return [];
});

const selectedFilters = computed(() => {
  if (props.header.id === "changeSetName")
    return logsStore.filters.changeSetFilter;
  else if (props.header.id === "entityName")
    return logsStore.filters.entityNameFilter;
  else if (props.header.id === "entityType")
    return logsStore.filters.entityTypeFilter;
  else if (props.header.id === "title") return logsStore.filters.titleFilter;
  else if (props.header.id === "userName") return logsStore.filters.userFilter;
  else return [];
});

const headerText = computed(() => {
  if (label.value === "Time") {
    // NOTE(nick): restore timestamp sort after audit trail is shipped.
    // return `Sorting By Timestamp ${
    //   props.filters.sortTimestampAscending ? "(Oldest)" : "(Newest)"
    // }`;
    return "Sorted by Timestamp (Newest)";
  }
  if (selectedFilters.value.length > 0) {
    return `Filtering by ${selectedFilters.value.length} selection${
      selectedFilters.value.length > 1 ? "s" : ""
    }`;
  }
  return `Filter by ${label.value}`;
});

const tooltip = computed(() => {
  if (props.header.id === "json") {
    if (props.anyRowsOpen) {
      return {
        content: "Collapse All",
        delay: { show: 0, hide: 100 },
        instantMove: true,
      };
    }
    return null;
  }

  return {
    content: headerText.value,
    delay: { show: 0, hide: 100 },
    instantMove: true,
  };
});

const dropdownMenuItems = computed(() => {
  const items: DropdownMenuItemObjectDef[] = [];

  items.push({
    label: headerText.value,
    header: true,
  });

  for (const k of filterOptions.value) {
    items.push({
      label: k.label,
      checkable: true,
      checked: selectedFilters.value.includes(k.value),
      onSelect: () => {
        emit("toggleFilter", k.value);
      },
    });
  }

  if (selectedFilters.value.length > 0) {
    items.unshift({
      label: "Clear Filters",
      onSelect: () => {
        emit("clearFilters");
      },
    });
  }

  return items;
});

const onClick = () => {
  dropdownMenuRef.value?.open();
  emit("select");
};

const active = ref(false);

const startActive = () => {
  active.value = true;
  iconButtonRef.value?.startActive();
};

const endActive = () => {
  active.value = false;
  iconButtonRef.value?.endActive();
};

const windowWidth = ref(window.innerWidth);
const onWidthChange = () => {
  windowWidth.value = window.innerWidth;
};
onMounted(() => window.addEventListener("resize", onWidthChange));
onUnmounted(() => window.removeEventListener("resize", onWidthChange));

const emit = defineEmits<{
  (e: "select"): void;
  (e: "clearFilters"): void;
  (e: "toggleFilter", v: string): void;
}>();
</script>
