<script lang="ts">
  import type { BattleView, BattleCommands } from "../controller/view";
  import { dialoguePending } from "../game/campaign-progress";
  import { artSprite } from "../presentation/art";
  let { view, actions }: { view: BattleView; actions: BattleCommands } = $props();
  let dialog: HTMLDialogElement;
  const campaign = $derived(view.game.campaign);
  const line = $derived(campaign?.dialogue[campaign.dialogueIndex]);
  const pending = $derived(dialoguePending(campaign));
  $effect(() => {
    if (pending && !dialog.open) dialog.showModal();
    else if (!pending && dialog.open) dialog.close();
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
    <p class="story-heading">A conversation</p>
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
        {campaign.dialogueIndex + 1 === campaign.dialogue.length ? "Return to battle" : "Continue"}
      </button>
    </div>
  {/if}
</dialog>
