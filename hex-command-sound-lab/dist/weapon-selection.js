// Explicit selection exceptions; infantry/material and creature defaults stay separate.
export function weaponSelection(entity) {
  const id = entity.id;
  if (
    /^(scout|ranger|shadowblade|elf-archer(?:-l[23])?|elf-flanker(?:-l[23])?)$/.test(
      id,
    )
  )
    return {
      pack: "still-north-media/weapon-textures",
      suggested: /^Dagger Draw Fast$/,
      alternatives: /^(Dagger|Seax) (Draw|Sheath) (Fast|Slow)$/,
      note: "Dagger handling for scouts, elven archers and knife fighters.",
    };
  if (/^(crossbowman|arbalester|dwarf-crossbow(?:-l[23])?)$/.test(id))
    return {
      pack: "still-north-media/weapon-textures",
      suggested: /^(English Longbow|Scythian Recurve) Nock Arrow$/,
      alternatives:
        /^Crossbow (Place Bolt|Set Nut Against Sear|Lever Trigger)$/,
      note: "Arrow nocking, matching the archer selection palette; a crossbow handling proxy.",
    };
  if (/^elf-bladedancer-l[23]$/.test(id))
    return {
      pack: "still-north-media/weapon-textures",
      suggested: /^Sabre Draw Fast$/,
      alternatives: /^Sabre (Draw|Sheath) (Fast|Slow)$/,
      note: "Recorded sword draw for elven swordsmen.",
    };
  if (/^elf-jumper-l[23]$/.test(id))
    return {
      pack: "still-north-media/weapon-textures",
      suggested: /^Spear Swing$/,
      alternatives: /^Spear Swing$/,
      note: "Spear swing as a throwing-motion proxy for jumper selection; no dedicated spear-throw recording is claimed.",
    };
  if (/^dwarf-arquebus-l[23]$/.test(id))
    return {
      pack: "rpg-audio",
      suggested: /^metalLatch$/,
      alternatives: /^metalLatch$|^metalClick$/,
      note: "Metal mechanism handling for arquebusiers; not a recorded firearm mechanism.",
    };
  if (/^(slinger|fireslinger)$/.test(id))
    return {
      pack: "rpg-audio",
      suggested: /^handleSmallLeather2$/,
      alternatives: /^handleSmallLeather[2]?$|^beltHandle[12]$/,
      note: "Leather pouch handling for slingers; a proxy rather than a dedicated sling recording.",
    };
  return null;
}
