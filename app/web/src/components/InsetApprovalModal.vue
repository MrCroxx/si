<template>
  <div
    :class="
      clsx(
        'max-w-md flex flex-col gap-sm p-sm shadow-2xl',
        themeClasses('bg-neutral-100 border', 'bg-neutral-900'),
      )
    "
  >
    <div class="flex flex-col gap-2xs">
      <div class="font-bold">{{ modalData.title }}</div>
      <div class="text-sm italic">
        <Timestamp :date="modalData.date" showTimeIfToday size="extended" />
      </div>
    </div>
    <ErrorMessage
      :tone="modalData.messageTone"
      :icon="modalData.messageIcon"
      variant="block"
      class="rounded"
    >
      <template v-if="mode === 'requested'">
        This change set is currently locked until the approval is accepted or
        rejected.
        <template v-if="userIsApprover"
          >You can approve or reject this change set, or you
        </template>
        <template v-else>
          {{
            `${
              requesterIsYou
                ? "You can withdraw the approval request to make more changes and then request approval again, or you"
                : "You"
            } `
          }}
        </template>
        can switch to a different change set using the dropdown at the top of
        the screen.
      </template>
      <template v-else-if="mode === 'approved' || mode === 'rejected'">
        {{ requesterIsYou ? "Your" : "The" }} request to
        <span class="font-bold">Apply</span> change set
        <span class="font-bold">{{ changeSetName }}</span> was {{ mode }} by
        <span class="font-bold">{{ approverEmail + " " }}</span>

        <!-- {{ modalData.date.getTime() === new Date().getTime() ? "" : "on" }} -->
        <span class="font-bold">
          <Timestamp :date="modalData.date" showTimeIfToday size="extended" />
        </span>
        <div
          v-if="!requesterIsYou && !userIsApprover && mode === 'approved'"
          class="pt-xs"
        >
          <span class="font-bold">{{ requesterEmail }}</span> requested this
          <span class="font-bold">Apply</span> and can merge this change set.
          You can switch to a different change set using the dropdown at the top
          of the screen.
        </div>
      </template>
      <template v-else>
        ERROR - this message should not ever show. Something has gone wrong!
      </template>
    </ErrorMessage>
    <div class="text-sm mb-sm">
      These actions will be applied to the real world:
    </div>
    <div
      class="flex-grow overflow-y-auto mb-sm border border-neutral-100 dark:border-neutral-700"
    >
      <ActionsList slim kind="proposed" noInteraction />
    </div>
    <div
      v-if="requesterIsYou || mode === 'rejected' || userIsApprover"
      class="flex flex-row gap-sm"
    >
      <VButton
        v-if="userIsApprover && (mode === 'requested' || mode === 'approved')"
        label="Reject Request"
        tone="destructive"
        variant="ghost"
        @click="rejectHandler"
      />
      <VButton
        v-if="!userIsApprover && mode === 'approved'"
        label="Withdraw Request"
        tone="destructive"
        variant="ghost"
        @click="rejectHandler"
      />
      <VButton
        :label="modalData.buttonText"
        :tone="modalData.buttonTone"
        class="grow"
        :loading="mode === 'approved' ? applyingChangeSet : false"
        loadingText="Applying..."
        @click="confirmHandler"
      />
    </div>
  </div>
</template>

<script lang="ts" setup>
import {
  VButton,
  Timestamp,
  Tones,
  ErrorMessage,
  IconNames,
  themeClasses,
} from "@si/vue-lib/design-system";
import { computed, ref } from "vue";
import clsx from "clsx";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useAuthStore } from "@/store/auth.store";
import { ChangeSetStatus } from "@/api/sdf/dal/change_set";
import ActionsList from "./Actions/ActionsList.vue";

export type InsetApprovalModalMode = "requested" | "approved" | "rejected";

const changeSetsStore = useChangeSetsStore();
const changeSetName = computed(() => changeSetsStore.selectedChangeSet?.name);
const authStore = useAuthStore();
const applyingChangeSet = ref(false);

// TODO(Wendy) - Mock data we need to replace with real data!
const mode = computed(() => {
  if (
    changeSetsStore.selectedChangeSet?.status === ChangeSetStatus.NeedsApproval
  ) {
    return "requested";
  } else if (
    changeSetsStore.selectedChangeSet?.status === ChangeSetStatus.Approved
  ) {
    return "approved";
  } else if (
    changeSetsStore.selectedChangeSet?.status === ChangeSetStatus.Rejected
  ) {
    return "rejected";
  } else return "";
});

const requesterEmail = computed(
  () => changeSetsStore.selectedChangeSet?.mergeRequestedByUser,
);
const requestDate = computed(
  () => changeSetsStore.selectedChangeSet?.mergeRequestedAt as IsoDateString,
);
const requesterIsYou = computed(
  () =>
    changeSetsStore.selectedChangeSet?.mergeRequestedByUserId ===
    authStore.user?.pk,
);
const approverEmail = computed(
  () => changeSetsStore.selectedChangeSet?.reviewedByUser,
);
const approveDate = computed(
  () => changeSetsStore.selectedChangeSet?.reviewedAt as IsoDateString,
);
const userIsApprover = computed(() => changeSetsStore.currentUserIsApprover);

const modalData = computed(() => {
  if (mode.value === "requested") {
    return {
      title: `${changeSetName.value} Approval Requested by ${
        requesterIsYou.value ? "You" : requesterEmail.value
      }`,
      date: requestDate.value,
      buttonText: userIsApprover.value
        ? "Approve Request"
        : "Withdraw Approval Request",
      buttonTone: (userIsApprover.value ? "success" : "action") as Tones,
      messageTone: "warning" as Tones,
      messageIcon: "exclamation-circle" as IconNames,
    };
  } else if (mode.value === "approved") {
    return {
      title: `Approval Granted by ${approverEmail.value}`,
      date: approveDate.value,
      buttonText: "Apply Change Set",
      buttonTone: "success" as Tones,
      messageTone: "success" as Tones,
      messageIcon: "check-circle" as IconNames,
    };
  } else if (mode.value === "rejected") {
    return {
      title: `Approval Rejected by ${approverEmail.value}`,
      date: approveDate.value,
      buttonText: "Make Edits",
      buttonTone: "action" as Tones,
      messageTone: "destructive" as Tones,
      messageIcon: "exclamation-circle" as IconNames,
    };
  }

  return {
    title: "ERROR!",
    date: new Date(),
    buttonText: "Error!",
    buttonTone: "destructive" as Tones,
    messageTone: "destructive" as Tones,
  };
});

const confirmHandler = () => {
  if (mode.value === "requested") {
    if (userIsApprover.value) {
      changeSetsStore.APPROVE_CHANGE_SET_FOR_APPLY();
    } else if (requesterIsYou.value) {
      changeSetsStore.CANCEL_APPROVAL_REQUEST();
    }
  } else if (mode.value === "approved") {
    if (authStore.user) {
      applyingChangeSet.value = true;
      changeSetsStore.APPLY_CHANGE_SET(authStore.user.name);
    }
  } else if (mode.value === "rejected") {
    changeSetsStore.REOPEN_CHANGE_SET();
  }
};

const rejectHandler = () => {
  if (mode.value === "requested") {
    changeSetsStore.REJECT_CHANGE_SET_APPLY();
  } else if (mode.value === "approved" && userIsApprover.value) {
    changeSetsStore.REJECT_CHANGE_SET_APPLY();
  } else if (mode.value === "approved" && requesterIsYou.value) {
    changeSetsStore.CANCEL_APPROVAL_REQUEST();
  }
};
</script>
