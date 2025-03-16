<script lang="ts" module>
  import { createId } from "@paralleldrive/cuid2";
  import type { Snippet } from "svelte";

  export interface Notification {
    id: string;
    title: string | Snippet;
    message?: string | Snippet;
    dismissTimeout?: number | null;
    status?: "success" | "warning" | "error" | "info";
  }

  export function createNotificationsStore() {
    let notifications = $state<Notification[]>([]);

    const addNotification = (
      notification: Omit<Notification, "id">
    ): Notification => {
      const dismissTimeout =
        typeof notification.dismissTimeout === "undefined"
          ? 5000
          : notification.dismissTimeout;

      const newNotification = {
        ...notification,
        id: createId(),
        dismissTimeout,
      };

      if (typeof dismissTimeout === "number") {
        setTimeout(() => {
          removeNotification(newNotification.id);
        }, dismissTimeout);
      }

      notifications = [...notifications, newNotification];
      return newNotification;
    };

    const removeNotification = (id: string) =>
      (notifications = notifications.filter((it) => it.id !== id));

    return {
      get notifications() {
        return notifications;
      },
      addNotification,
      removeNotification,
    };
  }

  export const notificationsStore = createNotificationsStore();
</script>

{#if notificationsStore.notifications.length > 0}
  <div class="notifications">
    {#each notificationsStore.notifications as notification}
      {#key notification.id}
        <div
          class="notifications__item"
          class:notifications--success={notification.status === "success"}
          class:notifications--error={notification.status === "error"}
          class:notifications--warning={notification.status === "warning"}
          class:notifications--info={notification.status === "info"}
        >
          <span class="notifications__title">
            {#if typeof notification.title === "string"}
              {notification.title}
            {:else}
              {@render notification.title()}
            {/if}
          </span>

          {#if notification.message}
            <p class="notifications__message">
              {#if typeof notification.message === "string"}
                {notification.message}
              {:else}
                {@render notification.message()}
              {/if}
            </p>
          {/if}
        </div>
      {/key}
    {/each}
  </div>
{/if}

<style lang="scss">
  .notifications {
    position: fixed;
    z-index: 10;
    bottom: 0;
    right: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 1.5rem;
    font-family: var(--mono-font-family);

    &__item {
      padding: 0.8rem 1rem;
      box-shadow: var(--shadow);
      background-color: var(--color-grey);
      border-radius: var(--border-radius);
    }

    &__title {
      font-weight: 600;
    }

    &__message {
      margin: 0;
      margin-top: 0.3rem;
      padding-top: 0.3rem;
      border-top: solid 1px black;
      line-clamp: 5;
      display: -webkit-box;
      -webkit-line-clamp: 7;
      -webkit-box-orient: vertical;
      overflow: hidden;
      text-overflow: ellipsis;
      max-width: 50rem;
    }

    &--success {
      background-color: rgb(186, 255, 177);
    }

    &--error {
      background-color: rgb(255, 177, 177);
    }

    &--warning {
      background-color: rgb(255, 243, 177);
    }

    &--info {
      background-color: rgb(177, 224, 255);
    }
  }
</style>
