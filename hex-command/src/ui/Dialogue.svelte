<script lang="ts">
  import { tick } from "svelte";
  import type { BattleView, BattleCommands } from "../controller/view";
  import { campaignMission, dialoguePending } from "../game/campaign-progress";
  import { artSprite } from "../presentation/art";
  let { view, actions }: { view: BattleView; actions: BattleCommands } = $props();
  let dialog: HTMLDialogElement;
  let nextButton = $state<HTMLButtonElement>();
  const campaign = $derived(view.game.campaign);
  const line = $derived(campaign?.dialogue[campaign.dialogueIndex]);
  const pending = $derived(dialoguePending(campaign));
  const mission = $derived(campaign ? campaignMission(campaign) : null);
  $effect(() => {
    if (pending && !dialog.open) {
      dialog.showModal();
      nextButton?.focus();
    } else if (!pending && dialog.open) {
      dialog.close();
      if (view.game.winner) {
        const finishedGame = view.game;
        void tick().then(() => {
          if (view.game !== finishedGame || dialoguePending(view.game.campaign)) return;
          const result = document.getElementById("mission-result");
          result?.focus({ preventScroll: true });
          result?.scrollIntoView({ block: "start" });
        });
      }
    }
  });

  function navigate(event: KeyboardEvent): void {
    if (event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
    if (event.key === "ArrowLeft") {
      event.preventDefault();
      if (!event.repeat) actions.previousDialogue();
    } else if (event.key === "ArrowRight") {
      event.preventDefault();
      if (!event.repeat) actions.advanceDialogue();
    }
  }
</script>

<dialog
  class="story-dialog"
  bind:this={dialog}
  oncancel={(event) => {
    event.preventDefault();
  }}
  onkeydown={navigate}
  aria-labelledby="dialogue-speaker"
>
  {#if campaign && line}
    <p class="story-heading">{mission?.map.name}{view.game.winner ? " — After the battle" : ""}</p>
    <div class="story-line" aria-live="polite" aria-atomic="true">
      <div class="story-portrait">
        {@html artSprite(view.artStyle, "units", line.portrait, 0, 0, 90, 120)}
      </div>
      <div>
        <h2 id="dialogue-speaker">{line.speaker}</h2>
        <p class="story-text">{line.text}</p>
      </div>
    </div>
    <nav class="story-navigation" aria-label="Conversation">
      <button
        class="story-back"
        disabled={campaign.dialogueIndex === 0}
        onclick={() => actions.previousDialogue()}
        aria-label="Previous line"
        title="Previous line (Left arrow)"
      >
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="m15 5-7 7 7 7" /></svg>
      </button>
      <span class="story-count">{campaign.dialogueIndex + 1} / {campaign.dialogue.length}</span>
      <button
        class="story-next"
        bind:this={nextButton}
        onclick={() => actions.advanceDialogue()}
        title="Next line (Right arrow)"
      >
        <span>
          {campaign.dialogueIndex + 1 === campaign.dialogue.length
            ? view.game.winner
              ? "Review mission result"
              : "Return to battle"
            : "Continue"}
        </span>
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="m9 5 7 7-7 7" /></svg>
      </button>
    </nav>
  {/if}
</dialog>

<style>
  .story-navigation {
    display: flex;
    align-items: center;
    gap: 12px;
    border-top: 1px solid var(--line);
    padding-top: 16px;
    margin-top: 24px;
  }
  .story-navigation button {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    min-height: 44px;
    padding: 8px;
  }
  .story-navigation svg {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
    fill: none;
    stroke: currentColor;
    stroke-width: 2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .story-back {
    width: 44px;
    flex-shrink: 0;
  }
  .story-count {
    margin-right: auto;
    white-space: nowrap;
  }
  .story-next {
    color: var(--gold);
    border-color: var(--gold);
    text-align: right;
  }
</style>
