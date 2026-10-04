<script lang="ts">
  import { CAMPAIGNS } from "../game/campaign-data";
  import { TYPES } from "../game/engine";
  import { artSprite, type ArtStyle } from "../presentation/art";
  import type { CampaignId } from "../game/types";
  let {
    selected = $bindable<CampaignId>("lantern-road"),
    artStyle,
  }: { selected?: CampaignId; artStyle: ArtStyle } = $props();
  const campaigns = Object.values(CAMPAIGNS);
</script>

<fieldset class="campaign-picker">
  <legend>Choose a story</legend>
  {#each campaigns as campaign}
    <label class:selected={selected === campaign.id} class="campaign-choice">
      <input type="radio" name="campaign" value={campaign.id} bind:group={selected} />
      <div class="campaign-portrait">
        {@html artSprite(artStyle, "units", campaign.leader, 0, 0, 70, 90)}
      </div>
      <div>
        <h3>{campaign.name}</h3>
        <p class="campaign-tagline">{campaign.tagline}</p>
        <p>{campaign.description}</p>
        <span class="campaign-length">Three missions as {campaign.race} armies</span>
        {#if selected === campaign.id}
          <ol class="mission-list">
            {#each campaign.missions as mission, index}
              <li>
                <strong>{index + 1}. {mission.map.name}</strong>
                <span>{mission.subtitle}</span>
              </li>
            {/each}
          </ol>
          <p class="campaign-length">Led by a {TYPES[campaign.leader].name.toLowerCase()}</p>
        {/if}
      </div>
    </label>
  {/each}
</fieldset>
