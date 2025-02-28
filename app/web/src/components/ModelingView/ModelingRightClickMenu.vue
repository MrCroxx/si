<template>
  <DropdownMenu
    v-if="selectedEdge"
    ref="contextMenuRef"
    :items="rightClickMenuItemsEdge"
    variant="editor"
  />
  <DropdownMenu
    v-else
    ref="contextMenuRef"
    :items="rightClickMenuItems"
    variant="editor"
  />
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import {
  DropdownMenu,
  DropdownMenuItemObjectDef,
} from "@si/vue-lib/design-system";
import { storeToRefs } from "pinia";
import { computed, ref } from "vue";
// import plur from "plur";
import { RouteLocationRaw } from "vue-router";
import { useToast } from "vue-toastification";
import { IRect } from "konva/lib/types";
import { ComponentType } from "@/api/sdf/dal/schema";
import { useComponentsStore } from "@/store/components.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import {
  BindingWithDisplayName,
  useFuncStore,
  MgmtPrototype,
} from "@/store/func/funcs.store";
import { useActionsStore } from "@/store/actions.store";
import { useComponentAttributesStore } from "@/store/component_attributes.store";
import { useViewsStore } from "@/store/views.store";
import { ComponentId } from "@/api/sdf/dal/component";
import { ViewId } from "@/api/sdf/dal/views";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import {
  DiagramGroupData,
  DiagramNodeData,
  DiagramNodeDef,
  DiagramViewData,
} from "../ModelingDiagram/diagram_types";

const contextMenuRef = ref<InstanceType<typeof DropdownMenu>>();

const toast = useToast();
const changeSetsStore = useChangeSetsStore();
const componentsStore = useComponentsStore();
const funcStore = useFuncStore();
const actionsStore = useActionsStore();
const viewStore = useViewsStore();
const ffStore = useFeatureFlagsStore();

const {
  selectedComponentId,
  selectedComponentIds,
  selectedComponent,
  selectedComponents,
  selectedComponentsAndChildren,
  deletableSelectedComponents,
  restorableSelectedComponents,
  erasableSelectedComponents,
  selectedEdge,
} = storeToRefs(viewStore);

const attributesStore = computed(() =>
  selectedComponentId.value
    ? useComponentAttributesStore(selectedComponentId.value)
    : undefined,
);

function typeDisplayName() {
  if (selectedComponentId.value && selectedComponent.value) {
    if (selectedComponent.value.def.componentType === ComponentType.Component)
      return "COMPONENT";
    else if (
      selectedComponent.value.def.componentType ===
      ComponentType.ConfigurationFrameUp
    )
      return "UP FRAME";
    else if (
      selectedComponent.value.def.componentType ===
      ComponentType.ConfigurationFrameDown
    )
      return "DOWN FRAME";
    else if (selectedComponent.value.def.componentType === ComponentType.View)
      return "VIEW";
    else return "ASSET";
  } else if (selectedComponentIds.value.length) {
    for (const c of selectedComponents.value) {
      if (c.def.componentType === ComponentType.Component) return "COMPONENTS"; // if we have both frames and components, just use the word component
    }
    return "ASSETS";
  } else {
    return "ASSET";
  }
}

const bindings = computed(() => funcStore.actionBindingsForSelectedComponent);
const canRefresh = computed(
  () =>
    selectedComponent.value?.def &&
    "hasResource" in selectedComponent.value.def &&
    selectedComponent.value.def.hasResource &&
    changeSetsStore.selectedChangeSetId === changeSetsStore.headChangeSetId,
);
const getActionToggleState = (id: string) => {
  if (!selectedComponentId.value) return false;

  const a = actionsStore.listActionsByComponentId
    .get(selectedComponentId.value)
    .find((a) => a.prototypeId === id);
  return !!a;
};

const removeFromView = () => {
  if (viewStore.selectedViewId) {
    const componentIds = viewStore.selectedComponents
      .filter((c) => c.def.componentType !== ComponentType.View)
      .map((c) => c.def.id);
    if (componentIds.length > 0)
      viewStore.REMOVE_FROM(viewStore.selectedViewId, componentIds);
    const viewIds = viewStore.selectedComponents
      .filter((c) => c.def.componentType === ComponentType.View)
      .map((c) => c.def.id);
    if (viewIds.length > 0)
      viewStore.REMOVE_VIEW_FROM(viewStore.selectedViewId, viewIds);
  }
};

const viewsSubitems = (add: (viewId: ViewId) => void) => {
  // dont show the view you're in b/c you cannot copy or move things to it
  return viewStore.viewList
    .filter((v) => v.id !== viewStore.selectedViewId)
    .map((v) => {
      return {
        label: v.name,
        onSelect: () => add(v.id),
      };
    });
};

const viewAdd = (remove: boolean) => {
  return (viewId: ViewId) => {
    const components: Record<ComponentId, IRect> = {};
    selectedComponents.value
      .filter(
        (c): c is DiagramGroupData | DiagramNodeData =>
          c.def.componentType !== ComponentType.View,
      )
      .forEach((c) => {
        const geo = c.def.isGroup
          ? viewStore.groups[c.def.id]
          : viewStore.components[c.def.id];
        if (geo) components[c.def.id] = geo;
      });

    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    viewStore.ADD_TO(viewStore.selectedViewId!, components, viewId, remove);
  };
};

const rightClickMenuItemsEdge = computed(() => {
  const items: DropdownMenuItemObjectDef[] = [];
  const disabled = false;
  // single selected edge
  items.push({
    label: "EDGE",
    header: true,
  });

  if (selectedEdge.value?.changeStatus === "deleted") {
    items.push({
      label: "Restore",
      icon: "trash-restore",
      onSelect: triggerRestoreSelection,
      disabled,
    });
  } else {
    items.push({
      label: "Delete",
      shortcut: "⌫",
      icon: "trash",
      onSelect: triggerDeleteSelection,
      disabled,
    });
  }
  return items;
});

const anyViews = computed(() =>
  selectedComponents.value.some((c) => c instanceof DiagramViewData),
);

/**
 * HERE IS THE APPROACH IN GENERAL
 * Make sure every "action" (i.e. onSelect) operates on the whole list of selectedComponents
 * Unless it is disallowed from doing so
 *
 * Don't duplicate `items.push`, only add a thing one time, and focus on the conditions by which it should be added
 */
const rightClickMenuItems = computed(() => {
  const items: DropdownMenuItemObjectDef[] = [];
  const disabled = false;

  if (ffStore.OUTLINER_VIEWS) {
    items.push({
      label: "VIEWS",
      header: true,
    });

    // you can do these operations no matter how many elements selected
    if (!anyViews.value) {
      items.push({
        label: "Move to",
        icon: "arrows-out",
        submenuItems: viewsSubitems(viewAdd(true)),
      });
      items.push({
        label: "Copy to",
        icon: "clipboard-copy",
        submenuItems: viewsSubitems(viewAdd(false)),
      });
    }
    items.push({
      label: "Remove",
      icon: "x-circle",
      onSelect: removeFromView,
    });
  }

  // if you've selected a view, you can't do anything else
  if (anyViews.value) return items;

  items.push({
    label: typeDisplayName(),
    header: true,
  });

  // if only one element you can do these operations
  if (selectedComponentId.value && selectedComponent.value) {
    items.push({
      label: "Rename",
      shortcut: "N",
      icon: "cursor",
      onSelect: renameComponent,
    });

    // set component type
    const updateComponentType = (componentType: ComponentType) => {
      if (selectedComponentId.value && attributesStore.value) {
        attributesStore.value.SET_COMPONENT_TYPE({
          componentId: selectedComponentId.value,
          componentType,
        });
      }
    };

    const submenuItems: DropdownMenuItemObjectDef[] = [];
    submenuItems.push({
      label: "Component",
      icon: "component",
      checkable: true,
      checked:
        selectedComponent.value.def.componentType === ComponentType.Component,
      onSelect: () => {
        updateComponentType(ComponentType.Component);
      },
    });
    submenuItems.push({
      label: "Up Frame",
      icon: "frame-up",
      checkable: true,
      checked:
        selectedComponent.value.def.componentType ===
        ComponentType.ConfigurationFrameUp,
      onSelect: () => {
        updateComponentType(ComponentType.ConfigurationFrameUp);
      },
    });
    submenuItems.push({
      label: "Down Frame",
      icon: "frame-down",
      checkable: true,
      checked:
        selectedComponent.value.def.componentType ===
        ComponentType.ConfigurationFrameDown,
      onSelect: () => {
        updateComponentType(ComponentType.ConfigurationFrameDown);
      },
    });

    items.push({
      label: "Set Type",
      icon: "component",
      submenuItems,
    });
  }

  // management funcs for a single selected component
  if (
    funcStore.managementFunctionsForSelectedComponent.length > 0 &&
    ffStore.MANAGEMENT_FUNCTIONS
  ) {
    const submenuItems: DropdownMenuItemObjectDef[] = [];
    funcStore.managementFunctionsForSelectedComponent.forEach((fn) => {
      submenuItems.push({
        label: fn.label,
        icon: "play",
        onSelect: () => {
          runManagementFunc(fn);
        },
      });
    });
    items.push({
      label: "Management",
      icon: "func",
      submenuItems,
    });
  }

  // you copy, restore, delete,
  items.push({
    label: `Copy`,
    shortcut: "⌘C",
    icon: "clipboard-copy",
    onSelect: triggerCopySelection,
    disabled,
  });
  if (
    restorableSelectedComponents.value.length > 0 &&
    restorableSelectedComponents.value.length ===
      selectedComponentsAndChildren.value.length
  ) {
    items.push({
      label: `Restore`,
      icon: "trash-restore",
      onSelect: triggerRestoreSelection,
      disabled,
    });
  } else if (
    deletableSelectedComponents.value.length > 0 &&
    deletableSelectedComponents.value.length ===
      selectedComponentsAndChildren.value.length
  ) {
    items.push({
      label: `Delete`,
      shortcut: "⌫",
      icon: "trash",
      onSelect: triggerDeleteSelection,
      disabled,
    });
  }

  // can erase so long as you have not selected a view
  if (
    erasableSelectedComponents.value.length > 0 &&
    erasableSelectedComponents.value.length ===
      selectedComponentsAndChildren.value.length
  ) {
    items.push({
      label: "Erase",
      shortcut: "⌘E",
      icon: "erase",
      onSelect: triggerWipeFromDiagram,
      disabled,
    });
  }

  // can only refresh a single component
  if (bindings.value.length > 0 || canRefresh.value) {
    items.push({
      label: "RESOURCE",
      header: true,
    });

    if (canRefresh.value) {
      items.push({
        label: "Refresh",
        shortcut: "R",
        icon: "refresh",
        onSelect: () => {
          // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
          if (selectedComponent.value)
            componentsStore.REFRESH_RESOURCE_INFO(
              selectedComponent.value.def.id,
            );
        },
        disabled,
      });
    }

    // actions limited to a single component
    if (bindings.value.length > 0 && selectedComponentId.value) {
      const def = selectedComponent.value?.def as DiagramNodeDef;
      const submenuItems: DropdownMenuItemObjectDef[] = [];

      bindings.value.forEach((binding: BindingWithDisplayName) => {
        const componentId = selectedComponentId.value as string;

        const action = computed(() => {
          const a = actionsStore.listActionsByComponentId
            .get(componentId)
            .find((a) => a.prototypeId === binding.actionPrototypeId);
          return a;
        });

        submenuItems.push({
          label: binding.displayName || binding.name,
          toggleIcon: true,
          checked: binding.actionPrototypeId
            ? getActionToggleState(binding.actionPrototypeId)
            : false,
          onSelect: () => {
            if (action.value?.id) {
              actionsStore.CANCEL([action.value.id]);
            } else if (binding.actionPrototypeId) {
              actionsStore.ADD_ACTION(componentId, binding.actionPrototypeId);
            }
          },
          endLinkTo: {
            name: "workspace-lab-assets",
            query: {
              s: `a_${def.schemaVariantId}|f_${binding.funcId}`,
            },
          } as RouteLocationRaw,
          endLinkLabel: "view",
        });
      });

      items.push({
        label: "Actions",
        submenuItems,
      });
    }
  }

  return items;
});

const runManagementFunc = async (prototype: MgmtPrototype) => {
  if (!selectedComponent.value) return;
  if (!viewStore.selectedViewId) return;

  const result = await funcStore.RUN_MGMT_PROTOTYPE(
    prototype.managementPrototypeId,
    selectedComponent.value.def.id,
    viewStore.selectedViewId,
  );

  if (result.result.success && result.result.data.message) {
    const toastOptions = {
      pauseOnHover: true,
      timeout: 5000,
    };
    if (result.result.data.status === "ok") {
      toast.success(
        `${prototype.label}: ${result.result.data.message}`,
        toastOptions,
      );
    } else {
      toast.warning(
        `${prototype.label}: ${result.result.data.message}`,
        toastOptions,
      );
    }
  }
};

function triggerCopySelection() {
  componentsStore.copyingFrom = elementPos.value;
  elementPos.value = null;
}

const modelingEventBus = componentsStore.eventBus;

function triggerDeleteSelection() {
  modelingEventBus.emit("deleteSelection");
  elementPos.value = null;
}

function triggerRestoreSelection() {
  modelingEventBus.emit("restoreSelection");
  elementPos.value = null;
}

function triggerWipeFromDiagram() {
  modelingEventBus.emit("eraseSelection");
  elementPos.value = null;
}

function renameComponent() {
  if (selectedComponentId.value) {
    componentsStore.eventBus.emit("rename", selectedComponentId.value);
  }
}

const elementPos = ref<{ x: number; y: number } | null>(null);

function open(
  e: MouseEvent,
  anchorToMouse: boolean,
  elementPosition?: { x: number; y: number },
) {
  if (elementPosition) elementPos.value = elementPosition;
  contextMenuRef.value?.open(e, anchorToMouse);
}

const isOpen = computed(() => contextMenuRef.value?.isOpen);

defineExpose({ open, isOpen });
</script>
