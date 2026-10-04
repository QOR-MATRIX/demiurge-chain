# Building with your own LLM

**For game developers who want to describe a game and have an AI build it.** ARQADE is designed so that an external
model — Claude, or any other — can scaffold, write, test and prepare your game for publishing, while you keep the
signature on everything that is public or costs CGT.

> **Status: designed, not built.** The connection is the Demiurge MCP server ADR-010 decided (M5.3), with ARQADE's tools
> added ([ADR-071](../../../../docs/decisions/ADR-071-arqade-self-publishing.md) decision 7, Proposed). Tool names are
> placeholders.

---

## How your model connects

1. **In QOR ID, authorise an agent key** for your model (ADR-014). You choose what it may do; you can revoke it at any
   moment without touching anything else.
2. **Point your model at the Demiurge MCP server** with that key. One server works for every LLM that speaks MCP; there is
   no per-vendor plugin.
3. The model now works on **your** projects, with **its** key. It never sees your Vault's key.

## What the model can do

| Tool (placeholder) | What it does |
| --- | --- |
| `arqade.scaffold` | Start a project from a template: an arcade game, a card game, a game with an evolving asset |
| `arqade.write` / `arqade.commit` | Edit files and commit them to the project's Qontrol history |
| `arqade.test` | Run the game's tests, and a rehearsal on Demiurge Devnet |
| `arqade.profile` | Fill in and revise the project profile, your store page |
| `arqade.readiness` | Report what the next stage still needs (`checkReadiness`) |
| `arqade.prepare` | Prepare a submission, a price, an item list or a campaign, **for you to sign** |

## What it cannot do alone

Submitting a stage, setting or changing a price, opening a campaign and funding a Game Vault are **public or move CGT**,
so they reach your Vault as a request you read and approve. When the chain enforces delegated spend caps (M5.2), you will
be able to let an agent key do small, capped things on its own, and revoke it at will.

## The code it writes is checked like anyone's

Every build runs in ARQADE's isolated origin with only the network access its manifest declares, no access to wallets or
the page around it, and size and frame-time limits — whether a person or a model wrote it ([publishing.md](publishing.md)).

## Cost

Your own LLM costs ARQADE nothing. If ARQADE ever runs a model for you and charges for it, that is priced in CGT as
agent compute, which is not designed yet (U-9).
