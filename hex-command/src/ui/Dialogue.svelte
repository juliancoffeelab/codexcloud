<script lang="ts">
  import { tick } from "svelte";
  import type { BattleView, BattleCommands } from "../controller/view";
  import { campaignMission, dialoguePending } from "../game/campaign-progress";
  import { artSprite } from "../presentation/art";
  let { view, actions }: { view: BattleView; actions: BattleCommands } = $props();
  let dialog: HTMLDialogElement;
  const campaign = $derived(view.game.campaign);
  const line = $derived(campaign?.dialogue[campaign.dialogueIndex]);
  const pending = $derived(dialoguePending(campaign));
  const mission = $derived(campaign ? campaignMission(campaign) : null);
  $effect(() => {
    if (pending && !dialog.open) dialog.showModal();
    else if (!pending && dialog.open) {
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
</script>

<dialog
  class="story-dialog"
  bind:this={dialog}
  oncancel={(event) => {
    event.preventDefault();
    actions.advanceDialogue(true);
  }}
  aria-labelledby="dialogue-speaker"
>
  {#if campaign && line}
    <p class="story-heading">{mission?.map.name}{view.game.winner ? " — After the battle" : ""}</p>
    <div class="story-line">
      <div class="story-portrait">
        {@html artSprite(view.artStyle, "units", line.portrait, 0, 0, 90, 120)}
      </div>
      <div>
        <h2 id="dialogue-speaker">{line.speaker}</h2>
        <p class="story-text">{line.text}</p>
      </div>
    </div>
    <p class="story-count">{campaign.dialogueIndex + 1} / {campaign.dialogue.length}</p>
    <div class="dialog-actions">
      <button class="quiet" onclick={() => actions.advanceDialogue(true)}>Skip conversation</button>
      <button class="primary" onclick={() => actions.advanceDialogue()}>
        {campaign.dialogueIndex + 1 === campaign.dialogue.length
          ? view.game.winner
            ? "Review mission result"
            : "Return to battle"
          : "Continue"}
      </button>
    </div>
  {/if}
</dialog>
