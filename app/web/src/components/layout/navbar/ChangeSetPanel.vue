<template>
  <div>
    <div class="flex gap-xs items-end">
      <div>
        <div
          class="text-[11px] mt-[1px] mb-[5px] capsize font-medium text-neutral-300"
        >
          CHANGE SET:
        </div>
        <VormInput
          ref="dropdownRef"
          :modelValue="selectedChangeSetId"
          :options="changeSetDropdownOptions"
          class="flex-grow font-bold mb-[-1px]"
          noLabel
          placeholder="-- select a change set --"
          size="xs"
          type="dropdown"
          @update:model-value="onSelectChangeSet"
        />
      </div>

      <VButton
        v-tooltip="{
          content: 'Create Change Set',
        }"
        icon="git-branch-plus"
        size="sm"
        tone="action"
        variant="ghost"
        @click="openCreateModal"
      />

      <VButton
        v-tooltip="{
          content: 'Abandon Change Set',
        }"
        :disabled="
          !selectedChangeSetName ||
          changeSetsStore.headSelected ||
          changeSetsStore.creatingChangeSet ||
          (featureFlagsStore.REBAC &&
            changeSetsStore.selectedChangeSet?.status ===
              ChangeSetStatus.NeedsApproval)
        "
        icon="trash"
        size="sm"
        tone="action"
        variant="ghost"
        @click="openApprovalFlowModal"
      />
    </div>

    <Modal ref="createModalRef" title="Create Change Set">
      <form @submit.prevent="onCreateChangeSet">
        <Stack>
          <p>
            Modeling a configuration or extending SI happens within
            <b>Change Sets</b>. Think of these like light-weight branches,
            allowing you to experiment freely without risk of impacting
            production systems.
          </p>
          <p>
            Please give your <b>Change Set</b> a name below, and click the
            Create button.
          </p>
          <VormInput
            v-model="createChangeSetName"
            :regex="CHANGE_SET_NAME_REGEX"
            label="Change set name"
            regexMessage="You cannot name a change set 'HEAD' - please choose another name."
            required
            requiredMessage="Please choose a name for your change set!"
          />
          <div class="flex flex-row-reverse gap-sm">
            <VButton
              :disabled="validationState.isError"
              :requestStatus="createChangeSetReqStatus"
              class="flex-grow"
              icon="plus-circle"
              label="Create change set"
              loadingText="Creating Change Set"
              submit
              tone="success"
            />
          </div>
        </Stack>
      </form>
    </Modal>

    <ApprovalFlowModal
      ref="approvalFlowModalRef"
      votingKind="abandon"
      @completeVoting="changeSetsStore.ABANDON_CHANGE_SET"
    />
    <AbandonChangeSetModal ref="abandonModalRef" />
  </div>
</template>

<script lang="ts" setup>
import { onMounted, computed, ref, watch } from "vue";
import * as _ from "lodash-es";
import { useRoute, useRouter } from "vue-router";
import {
  VButton,
  VormInput,
  Stack,
  Modal,
  useValidatedInputGroup,
} from "@si/vue-lib/design-system";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { ChangeSetStatus } from "@/api/sdf/dal/change_set";
import { useAuthStore } from "@/store/auth.store";
import ApprovalFlowModal from "@/components/ApprovalFlowModal.vue";
import AbandonChangeSetModal from "@/components/AbandonChangeSetModal.vue";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";

const CHANGE_SET_NAME_REGEX = /^(?!head).*$/i;

const featureFlagsStore = useFeatureFlagsStore();
const dropdownRef = ref();
const authStore = useAuthStore();
const changeSetsStore = useChangeSetsStore();
const openChangeSets = computed(() => changeSetsStore.openChangeSets);
const selectedChangeSetId = computed(() => changeSetsStore.selectedChangeSetId);
const selectedChangeSetName = computed(
  () => changeSetsStore.selectedChangeSet?.name,
);

const changeSetDropdownOptions = computed(() => {
  return [
    ..._.map(openChangeSets.value, (cs) => ({ value: cs.id, label: cs.name })),
    { value: "NEW", label: "+ Create new change set" },
  ];
});

const router = useRouter();
const route = useRoute();

const approvalFlowModalRef = ref<InstanceType<typeof ApprovalFlowModal> | null>(
  null,
);

const abandonModalRef = ref<InstanceType<typeof AbandonChangeSetModal> | null>(
  null,
);

const openApprovalFlowModal = () => {
  if (!featureFlagsStore.REBAC) {
    approvalFlowModalRef.value?.open();
  } else {
    abandonModalRef.value?.open();
  }
};

const createModalRef = ref<InstanceType<typeof Modal>>();

const changeSetsReqStatus =
  changeSetsStore.getRequestStatus("FETCH_CHANGE_SETS");

const checkFirstLoad = () => {
  if (!changeSetsReqStatus.value.isSuccess || !createModalRef.value) return;

  const isFirstLoad = !window.localStorage.getItem("ran-first-load");
  window.localStorage.setItem("ran-first-load", "true");

  if (isFirstLoad) {
    createModalRef.value?.open();
  }
};

watch([changeSetsReqStatus], checkFirstLoad);

onMounted(() => {
  checkFirstLoad();
});

// The name for a new change set
const createChangeSetName = ref(changeSetsStore.getGeneratedChangesetName());

const { validationState, validationMethods } = useValidatedInputGroup();

function onSelectChangeSet(newVal: string) {
  if (newVal === "NEW") {
    createModalRef.value?.open();
    return;
  }

  if (newVal && route.name) {
    // do not allow people to navigate to a change set that NeedsApproval
    // unless they were the one that initiated the merge request (avoids dead end)
    if (
      changeSetsStore.changeSetsById[newVal]?.status !== ChangeSetStatus.Open &&
      changeSetsStore.changeSetsById[newVal]?.mergeRequestedByUserId !==
        authStore.user?.pk &&
      !featureFlagsStore.REBAC
    ) {
      return;
    }

    // keep everything in the current route except the change set id
    // note - we use push here, so there is a new browser history entry
    let name = route.name;
    if (name === "workspace-compose-view") name = "workspace-compose";
    router.push({
      name,
      params: {
        ...route.params,
        changeSetId: newVal,
      },
      query: route.query,
    });
  }
}

async function onCreateChangeSet() {
  if (validationMethods.hasError()) return;

  const createReq = await changeSetsStore.CREATE_CHANGE_SET(
    createChangeSetName.value,
  );
  createChangeSetName.value = changeSetsStore.getGeneratedChangesetName();

  if (createReq.result.success) {
    // reusing above to navigate to new change set... will probably clean this all up later
    onSelectChangeSet(createReq.result.data.changeSet.id);
    createModalRef.value?.close();
  }
}

const createChangeSetReqStatus =
  changeSetsStore.getRequestStatus("CREATE_CHANGE_SET");

function openCreateModal() {
  createChangeSetName.value = changeSetsStore.getGeneratedChangesetName();
  createModalRef.value?.open();
}
</script>
