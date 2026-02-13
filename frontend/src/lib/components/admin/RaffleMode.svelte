<script lang="ts">
  import { Award, RefreshCcw, Shuffle, Trophy, Users, Download } from "lucide-svelte";
  import { t } from "svelte-i18n";
  import type { Attendee } from "$lib/api";

  let { attendees, onRefresh } = $props<{
    attendees: Attendee[];
    onRefresh: () => void;
  }>();

  const segmentColors = [
    "var(--color-chart-1)",
    "var(--color-chart-2)",
    "var(--color-chart-3)",
    "var(--color-chart-4)",
    "var(--color-chart-5)",
    "var(--color-primary)",
    "var(--color-ring)",
  ];

  let spinning = $state(false);
  let rotation = $state(0);
  let winner = $state<Attendee | null>(null);
  let history = $state<{ id: string; name: string; time: string }[]>([]);

  let eligible = $derived(attendees.filter((a: Attendee) => a.is_completed));
  let segmentAngle = $derived(eligible.length ? 360 / eligible.length : 0);
  let gradient = $derived(
    eligible.length
      ? eligible
          .map((_: Attendee, idx: number) => {
            const start = (idx / eligible.length) * 100;
            const end = ((idx + 1) / eligible.length) * 100;
            return `${segmentColors[idx % segmentColors.length]} ${start}% ${end}%`;
          })
          .join(", ")
      : ""
  );

  function formatCompletedAt(timestamp?: string) {
    if (!timestamp) return "";
    const date = new Date(timestamp);
    if (Number.isNaN(date.getTime())) return timestamp;
    return date.toLocaleString();
  }

  function downloadCertificate(attendeeId: string) {
    window.open(`/certificate/${attendeeId}`, '_blank');
  }

  function spinWheel() {
    if (!eligible.length || spinning) return;

    spinning = true;
    winner = null;

    const winnerIndex = Math.floor(Math.random() * eligible.length);
    const extraTurns = Math.floor(Math.random() * 3) + 4;
    const targetRotation =
      rotation + extraTurns * 360 + winnerIndex * segmentAngle + segmentAngle / 2;

    setTimeout(() => {
      const picked = eligible[winnerIndex];
      winner = picked;
      history = [
        { id: picked.id, name: picked.name, time: new Date().toLocaleTimeString() },
        ...history,
      ].slice(0, 6);
      spinning = false;
    }, 3200);

    requestAnimationFrame(() => {
      rotation = targetRotation;
    });
  }
</script>

<div class="grid grid-cols-1 xl:grid-cols-2 gap-6 sm:gap-8 h-full">
  <div class="bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-2xl shadow-sm p-6 sm:p-8 flex flex-col gap-6">
    <div class="flex items-center justify-between gap-3">
      <div>
        <p class="text-xs uppercase font-bold text-muted-foreground dark:text-dark-text-muted tracking-widest">
          {$t("raffle.tab_label")}
        </p>
        <h3 class="text-2xl font-bold text-foreground dark:text-dark-text mt-1 flex items-center gap-2">
          <Trophy size={22} class="text-amber-500" />
          {$t("raffle.title")}
        </h3>
        <p class="text-sm text-muted-foreground dark:text-dark-text-muted mt-1">
          {$t("raffle.description")}
        </p>
      </div>
      <button
        onclick={onRefresh}
        class="inline-flex items-center gap-2 px-3 py-2 text-sm font-bold text-primary bg-accent/70 dark:bg-primary/10 rounded-full hover:bg-accent dark:hover:bg-primary/20 transition-all"
        title={$t("raffle.refresh")}
      >
        <RefreshCcw size={16} class="shrink-0" />
        <span class="hidden sm:inline">{$t("raffle.refresh")}</span>
      </button>
    </div>

      <div class="flex flex-col items-center gap-6">
      <div class="relative w-full max-w-[420px] aspect-square mx-auto">
        <div
          class="wheel absolute inset-0 rounded-full flex items-center justify-center overflow-hidden border-[8px] border-white dark:border-dark-bg shadow-xl transition-transform duration-[3200ms] ease-[cubic-bezier(0.22,1,0.36,1)]"
          style={`background: ${
            eligible.length ? `conic-gradient(${gradient})` : "var(--color-border)"
          }; transform: rotate(${rotation}deg);`}
        >
          {#if eligible.length === 0}
            <div class="flex flex-col items-center justify-center text-center gap-2 text-muted-foreground dark:text-dark-text-muted">
              <Award size={32} />
              <p class="text-sm font-medium">{$t("raffle.no_eligible")}</p>
            </div>
          {/if}

          {#each eligible as attendee, idx}
            <span
              class="segment-label absolute left-1/2 top-1/2 origin-center text-[11px] font-semibold text-foreground dark:text-dark-text"
              style={`white-space: nowrap; transform: rotate(${idx * segmentAngle + segmentAngle / 2}deg) translateY(-46%) rotate(-${idx * segmentAngle + segmentAngle / 2}deg);`}
            >
              {attendee.name}
            </span>
          {/each}
        </div>

        <div class="pointer absolute left-1/2 -translate-x-1/2 -top-3 w-0 h-0">
          <div class="pointer-tip w-0 h-0 border-l-6 border-r-6 border-b-8 border-l-transparent border-r-transparent border-b-red-500 drop-shadow-lg"></div>
        </div>
      </div>

      <div class="flex flex-col sm:flex-row items-center gap-3 w-full justify-center">
        <button
          onclick={spinWheel}
        class="inline-flex items-center justify-center gap-2 px-5 py-3 rounded-full text-white font-bold shadow-md bg-red-600 hover:bg-red-700 active:scale-95 transition-all disabled:opacity-50"
        disabled={!eligible.length || spinning}
      >
          {#if spinning}
            <span class="flex items-center gap-2">
              <Shuffle size={18} class="animate-spin" />
              {$t("raffle.spinning")}
            </span>
          {:else}
            <Shuffle size={18} />
            {$t("raffle.spin")}
          {/if}
        </button>
        <div class="px-4 py-2 rounded-full bg-accent/60 dark:bg-white/5 border border-border dark:border-dark-border text-xs font-bold text-muted-foreground dark:text-dark-text-muted">
          {$t("raffle.eligible_count", { values: { count: eligible.length } })}
        </div>
      </div>

      {#if winner}
        <div class="w-full bg-emerald-50 dark:bg-emerald-500/10 border border-emerald-200 dark:border-emerald-500/30 rounded-2xl p-4 sm:p-5 flex items-center gap-3">
          <div class="w-12 h-12 rounded-full bg-white dark:bg-dark-bg shadow-sm flex items-center justify-center text-emerald-600 font-black text-lg">
            <Award size={22} />
          </div>
          <div class="flex-1 min-w-0">
            <p class="text-xs uppercase font-bold text-emerald-700 dark:text-emerald-200 tracking-widest">
              {$t("raffle.winner")}
            </p>
            <p class="text-lg font-bold text-foreground dark:text-white truncate">
              {winner.name}
            </p>
            <p class="text-sm text-muted-foreground dark:text-dark-text-muted truncate">
              {$t("raffle.certificate_badge")}
            </p>
          </div>
        </div>
      {/if}
    </div>
  </div>

  <div class="bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-2xl shadow-sm p-6 sm:p-8 flex flex-col gap-4">
    <div class="flex items-center justify-between gap-3">
      <div class="flex items-center gap-2">
        <Users size={18} class="text-primary" />
        <h4 class="text-lg font-bold text-foreground dark:text-dark-text">
          {$t("raffle.eligible_title")}
        </h4>
      </div>
      <span class="text-xs font-semibold px-3 py-1 rounded-full bg-accent/60 dark:bg-white/5 text-muted-foreground dark:text-dark-text-muted">
        {$t("raffle.completed_only")}
      </span>
    </div>

    <div class="space-y-2 max-h-[520px] overflow-y-auto pr-1">
      {#each eligible as attendee}
        <div class="flex items-center justify-between p-3 rounded-xl border border-border dark:border-dark-border bg-accent/60 dark:bg-white/5">
          <div class="flex items-center gap-3 min-w-0 flex-1">
            <div class="w-10 h-10 rounded-full bg-accent/80 dark:bg-white/10 flex items-center justify-center text-sm font-bold text-foreground dark:text-dark-text">
              {attendee.name.slice(0, 2).toUpperCase()}
            </div>
            <div class="min-w-0 flex-1">
              <p class="font-bold text-foreground dark:text-dark-text truncate">{attendee.name}</p>
              {#if attendee.completed_at}
                <p class="text-xs text-muted-foreground dark:text-dark-text-muted">
                  {$t("raffle.completed_at", { values: { time: formatCompletedAt(attendee.completed_at) } })}
                </p>
              {/if}
            </div>
          </div>
          <div class="flex items-center gap-2">
            <button
              onclick={() => downloadCertificate(attendee.id)}
              class="p-2 rounded-lg bg-white dark:bg-dark-bg hover:bg-accent/70 dark:hover:bg-primary/20 text-primary border border-border dark:border-dark-border transition-all"
              title="Download Certificate"
            >
              <Download size={16} />
            </button>
            <div class="flex items-center gap-1 text-emerald-600 dark:text-emerald-300 text-xs font-bold uppercase tracking-wide">
              <Award size={16} />
              <span class="hidden sm:inline">{$t("raffle.eligible_badge")}</span>
            </div>
          </div>
        </div>
      {:else}
        <div class="p-4 rounded-xl border border-dashed border-border dark:border-dark-border text-center text-sm text-muted-foreground dark:text-dark-text-muted bg-accent/60 dark:bg-white/5">
          {$t("raffle.waiting_for_completion")}
        </div>
      {/each}
    </div>

    {#if history.length}
      <div class="pt-2 border-t border-border dark:border-dark-border">
        <div class="flex items-center gap-2 mb-2">
          <Trophy size={16} class="text-amber-500" />
          <p class="text-sm font-bold text-foreground dark:text-dark-text">
            {$t("raffle.recent_winners")}
          </p>
        </div>
        <div class="space-y-2">
          {#each history as item}
            <div class="flex items-center justify-between p-2 rounded-lg bg-accent/60 dark:bg-white/5 border border-border dark:border-dark-border text-sm">
              <span class="font-semibold text-foreground dark:text-dark-text truncate">{item.name}</span>
              <span class="text-[11px] text-muted-foreground dark:text-dark-text-muted font-mono">{item.time}</span>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>
</div>
