<script lang="ts">
  import { Award, Download, Mail, Calendar, User } from "lucide-svelte";
  import { t } from "svelte-i18n";
  import type { Attendee } from "$lib/api";

  let { attendees } = $props<{
    attendees: Attendee[];
  }>();

  let completed = $derived(attendees.filter((a) => a.is_completed));
  let searchQuery = $state("");

  let filteredAttendees = $derived(
    completed.filter((a) =>
      a.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      (a.email && a.email.toLowerCase().includes(searchQuery.toLowerCase()))
    )
  );

  function formatCompletedAt(timestamp?: string) {
    if (!timestamp) return "";
    const date = new Date(timestamp);
    if (Number.isNaN(date.getTime())) return timestamp;
    const formatted = date.toLocaleString();
    return formatted;
  }

  function downloadCertificate(attendeeId: string) {
    window.open(`/certificate/${attendeeId}`, '_blank');
  }
</script>

<div class="h-full flex flex-col">
  <div class="bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-2xl shadow-sm p-6 flex flex-col gap-6 h-full">
    <!-- Header -->
    <div class="flex items-center justify-between gap-4 flex-wrap">
      <div>
        <div class="flex items-center gap-2">
          <Award size={24} class="text-primary" />
          <h3 class="text-2xl font-bold text-foreground dark:text-dark-text">
            {$t("certificate.management_title")}
          </h3>
        </div>
        <p class="text-sm text-muted-foreground dark:text-dark-text-muted mt-1">
          {$t("certificate.management_desc")}
        </p>
      </div>
      <div class="flex items-center gap-3">
        <div class="px-4 py-2 rounded-full bg-accent/70 dark:bg-primary/10 border border-primary/20">
          <span class="text-sm font-bold text-primary">
            {completed.length} {$t("certificate.completed_count")}
          </span>
        </div>
      </div>
    </div>

    <!-- Search -->
    <div class="relative">
      <input
        type="text"
        bind:value={searchQuery}
        placeholder={$t("certificate.search_placeholder")}
        class="w-full px-4 py-3 pl-10 bg-muted dark:bg-white/5 border border-border dark:border-dark-border rounded-xl outline-none focus:border-primary focus:ring-2 focus:ring-primary/20 transition-all"
      />
      <User size={18} class="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground/80 dark:text-dark-text-muted" />
    </div>

    <!-- Attendee List -->
    <div class="flex-1 overflow-y-auto space-y-3">
      {#if filteredAttendees.length === 0}
        <div class="flex flex-col items-center justify-center py-12 text-center">
          <div class="w-20 h-20 rounded-full bg-muted dark:bg-white/5 flex items-center justify-center mb-4">
            <Award size={32} class="text-muted-foreground/80 dark:text-dark-text-muted" />
          </div>
          <p class="text-muted-foreground dark:text-dark-text-muted font-medium">
            {searchQuery ? $t("certificate.no_search_results") : $t("certificate.no_completed")}
          </p>
        </div>
      {:else}
        {#each filteredAttendees as attendee}
          <div class="bg-muted dark:bg-white/5 border border-border dark:border-dark-border rounded-xl p-4 hover:border-primary hover:shadow-md transition-all">
            <div class="flex items-start justify-between gap-4">
              <!-- Attendee Info -->
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-3 mb-2">
                  <div class="w-12 h-12 rounded-full bg-primary flex items-center justify-center text-white font-bold text-lg shrink-0">
                    {attendee.name.slice(0, 2).toUpperCase()}
                  </div>
                  <div class="flex-1 min-w-0">
                    <h4 class="text-lg font-bold text-foreground dark:text-dark-text truncate">
                      {attendee.name}
                    </h4>
                    {#if attendee.email}
                      <div class="flex items-center gap-1.5 text-sm text-muted-foreground dark:text-dark-text-muted">
                        <Mail size={14} />
                        <span class="truncate">{attendee.email}</span>
                      </div>
                    {/if}
                  </div>
                </div>

                <!-- Completion Info -->
                {#if attendee.completed_at}
                  <div class="flex items-center gap-2 text-xs text-muted-foreground dark:text-dark-text-muted ml-15">
                    <Calendar size={14} />
                    <span>{$t("certificate.completed_at")}: {formatCompletedAt(attendee.completed_at)}</span>
                  </div>
                {/if}
              </div>

              <!-- Download Button -->
              <button
                onclick={() => downloadCertificate(attendee.id)}
                class="flex items-center gap-2 px-4 py-2.5 bg-primary hover:bg-primary/90 text-white rounded-lg font-bold shadow-sm hover:shadow-md transition-all shrink-0"
              >
                <Download size={16} />
                <span>{$t("certificate.download")}</span>
              </button>
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </div>
</div>
