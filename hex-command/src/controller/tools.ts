import { BATTLE_MAPS } from "../game/maps";
import { TYPES } from "../game/catalog";
import type { RecruitType, CampaignId, UnitType } from "../game/types";
import { CAMPAIGNS } from "../game/campaign-data";
const recruitable = Object.keys(TYPES);
import type { BattleController } from "./battle";

interface InputProperty {
  type: "string";
  enum?: string[];
  description?: string;
}

interface BattleTool {
  name: string;
  description: string;
  inputSchema: {
    type: "object";
    properties: Record<string, InputProperty>;
    required?: string[];
    additionalProperties: false;
  };
  annotations?: { readOnlyHint: true };
  execute: (input?: unknown) => unknown;
}

declare global {
  interface Document {
    modelContext?: {
      registerTool: (tool: BattleTool) => unknown;
      unregisterTool?: (name: string) => unknown;
    };
  }
}

function inputs(value: unknown): Record<string, unknown> {
  if (value === undefined) return {};
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new Error("Expected tool inputs to be an object.");
  }
  return value as Record<string, unknown>;
}

function stringInput(input: Record<string, unknown>, key: string): string {
  const value = input[key];
  if (typeof value !== "string") throw new Error(`Expected ${key} to be a string.`);
  return value;
}

const schema = (
  properties: Record<string, InputProperty> = {},
  required?: string[],
): BattleTool["inputSchema"] => ({
  type: "object",
  properties,
  ...(required ? { required } : {}),
  additionalProperties: false,
});

/** The browser tool boundary delegates to the same controller as the interface. */
export function registerBattleTools(controller: BattleController): () => void {
  const registry = document.modelContext;
  if (!registry?.registerTool) return () => {};
  const tools: BattleTool[] = [
    {
      name: "advance_unit",
      description:
        "Advance a ready unit after it earns enough experience. Preserves wounds and consumes this turn.",
      inputSchema: schema(
        { unitId: { type: "string" }, type: { type: "string", enum: Object.keys(TYPES) } },
        ["unitId", "type"],
      ),
      execute: (value) => {
        const input = inputs(value),
          type = stringInput(input, "type");
        if (!Object.hasOwn(TYPES, type)) throw new Error("Unknown advancement.");
        return controller.advance(stringInput(input, "unitId"), type as UnitType);
      },
    },
    {
      name: "start_campaign",
      description: "Replace the current battle with a new story campaign.",
      inputSchema: schema({ id: { type: "string", enum: Object.keys(CAMPAIGNS) } }, ["id"]),
      execute: (value) => {
        const id = stringInput(inputs(value), "id");
        if (!Object.hasOwn(CAMPAIGNS, id)) throw new Error("Unknown campaign.");
        return controller.startCampaign(id as CampaignId);
      },
    },
    {
      name: "advance_dialogue",
      description: "Read the next line in an active campaign conversation.",
      inputSchema: schema(),
      execute: () => controller.advanceDialogue(),
    },
    {
      name: "get_battle_state",
      description: "Read units, gold, village ownership, the current turn, and victory state.",
      inputSchema: schema(),
      annotations: { readOnlyHint: true },
      execute: () => controller.snapshot(),
    },
    {
      name: "get_unit_actions",
      description: "Read legal moves, attacks, and recruiting locations for a unit.",
      inputSchema: schema({ unitId: { type: "string" } }, ["unitId"]),
      annotations: { readOnlyHint: true },
      execute: (value) => controller.actions(stringInput(inputs(value), "unitId")),
    },
    {
      name: "move_unit",
      description: "Move one of your units to a legal hex. Captures a village when entered.",
      inputSchema: schema({ unitId: { type: "string" }, tileId: { type: "string" } }, [
        "unitId",
        "tileId",
      ]),
      execute: (value) => {
        const input = inputs(value);
        return controller.move(stringInput(input, "unitId"), stringInput(input, "tileId"));
      },
    },
    {
      name: "attack_unit",
      description: "Attack an enemy in range and resolve damage and retaliation.",
      inputSchema: schema(
        {
          unitId: { type: "string" },
          targetId: { type: "string" },
          attackId: {
            type: "string",
            description:
              "Weapon id from get_unit_actions; omit to choose the safest effective attack.",
          },
        },
        ["unitId", "targetId"],
      ),
      execute: (value) => {
        const input = inputs(value);
        const attackId = input.attackId === undefined ? undefined : stringInput(input, "attackId");
        return controller.attack(
          stringInput(input, "unitId"),
          stringInput(input, "targetId"),
          attackId,
        );
      },
    },
    {
      name: "recruit_unit",
      description:
        "Spend gold to recruit a troop from this mission’s pool adjacent to your leader on the keep.",
      inputSchema: schema(
        {
          commanderId: { type: "string" },
          type: { type: "string", enum: recruitable },
          tileId: { type: "string" },
        },
        ["commanderId", "type", "tileId"],
      ),
      execute: (value) => {
        const input = inputs(value);
        const type = stringInput(input, "type");
        if (!recruitable.includes(type)) throw new Error("Unknown recruit type.");
        return controller.recruit(
          stringInput(input, "commanderId"),
          type as RecruitType,
          stringInput(input, "tileId"),
        );
      },
    },
    {
      name: "undo_move",
      description:
        "Undo the latest move in the current player turn, restoring its village ownership changes. Attacking, recruiting, or ending the turn clears earlier moves.",
      inputSchema: schema(),
      execute: () => controller.undoMove(),
    },
    {
      name: "end_turn",
      description: "End your turn, let the enemy army act, and return the resulting battle state.",
      inputSchema: schema(),
      execute: () => controller.endTurn(),
    },
    {
      name: "start_new_battle",
      description: "Replace the current battle with a fresh battle.",
      inputSchema: schema({
        mapId: { type: "string", enum: BATTLE_MAPS.map((map) => map.id) },
      }),
      execute: (value) => {
        const input = inputs(value);
        return controller.reset(input.mapId === undefined ? "mirror" : stringInput(input, "mapId"));
      },
    },
  ];
  for (const tool of tools) {
    try {
      Promise.resolve(registry.registerTool(tool)).catch(() => {});
    } catch {
      /* Optional browser tooling must never prevent a battle from loading. */
    }
  }
  return () => {
    if (!registry.unregisterTool) return;
    for (const tool of tools) {
      try {
        Promise.resolve(registry.unregisterTool(tool.name)).catch(() => {});
      } catch {
        /* Tool cleanup is optional in experimental browser implementations. */
      }
    }
  };
}
