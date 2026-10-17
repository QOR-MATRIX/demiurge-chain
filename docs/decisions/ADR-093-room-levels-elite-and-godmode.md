# ADR-093: Room levels, Elite accounts and godmode

**Status:** **Accepted**, 16 October 2026, by the owner, who wrote: "godmode is the account that moderates chat. we
should create hierarchy status levels for chatrooms which can be set by the room owners and "Elite" accounts (Elite
accounts are settable by only godmode." Asked four questions, the owner chose the recommended option in each: levels
**"Owner, Admin, Mod, VIP, Member"**; Elite as **"Staff everywhere"**; godmode **"Above everyone"**; and **"Badge beside
the name"**. **Extends** [ADR-091](ADR-091-social.md) (whose rooms had an owner, moderators and members) and
[ADR-092](ADR-092-world-friends-and-translation.md) (World's moderators and mutes). It decides no economic value.

## Context

Checked on 16 October 2026 on branch `session/social-world` (pull request #35): a Social room has three roles, an owner
who makes moderators and hands the room on (becoming a moderator), moderators who delete others' messages and remove
members, and members. World's moderators come from `SOCIAL_WORLD_MODERATORS`, re-applied at every hello, and only they
mute, only in World. No account has a standing beyond the rooms it is in, and no role shows beside a name.

## Decision

1. **A room's levels, highest first: Owner, Admin, Mod, VIP, Member** (`owner`, `admin`, `moderator`, `vip`,
   `member`). **Each acts only on those below it**, and gives only levels below its own.
   - **Owner:** everything in the room; the only one who hands it on, and who then becomes an Admin. One Owner per room.
   - **Admin:** the room's settings (name, topic, ambience); sets levels (Mod, VIP, Member) of those below them.
   - **Mod:** deletes the messages of those below them, mutes them (ten minutes, an hour, a day or a week), removes
     them from the room, and invites to a private room.
   - **VIP:** a badge, and World's slower pace does not apply to them.
   - **Member:** talks.
2. **Elite accounts are staff everywhere.** An Elite account acts as an Admin in every room it is in, World included, on
   top of its level there, but never above a room's Owner, and never on godmode. Only godmode makes an account Elite or
   takes Elite away.
3. **godmode stands above everyone.** One account, named by its QOR ID username in Social's configuration
   (`SOCIAL_GODMODE`; set to `godmode` on 16 October 2026), stands above every room's Owner: it does anything any level
   does, in any room it is in, hands any room on, and alone grants and removes Elite. Nobody mutes, removes or demotes
   godmode. A username is fixed once chosen (ADR-075, ADR-091 decision 5), so the name is the account.
4. **Badges beside the name.** On every message and in member lists: godmode, Elite, then the room level (Owner, Admin,
   Mod, VIP); a Member shows none. A profile card shows godmode or Elite.
5. **No level and no standing reads a room it is not in.** godmode and Elite act in the rooms they are in; a private
   room's words stay its members', and a direct conversation stays two people's. (This keeps ADR-091's privacy as it
   was; widening it is a later record of its own.)
6. **World has no Owner.** Its levels are set by Elite accounts and godmode. The names in `SOCIAL_WORLD_MODERATORS` are
   at least Mods there at every hello, and a level appointed since is kept rather than reset.
7. **A mute applies in any room but a direct one**, given by a Mod or above to someone below them; the room list says
   until when the reader is muted in each room.
8. **Names** (plain, under ADR-032): the levels above; the methods `rooms.mute | unmute` and `elite.grant | revoke |
   list`; the event `room.muted` (World's keeps `world.muted`); a profile's `status` (`godmode`, `elite` or none); a
   history page's `levels`, a new message's `level`, and a room summary's `muted_until`; the configuration
   `SOCIAL_GODMODE`; migration 003. The protocol stays at version 1.

## Consequences

- A moderator under ADR-091 could change a room's name and topic; now that is an Admin's. A room's existing roles keep
  their names; an Owner who handed a room on before this change stays a Mod until someone above sets otherwise.
- godmode's powers rest on one configuration value and one account's sign-in. Whoever holds the `godmode` QOR ID holds
  every room; its key and sessions deserve the care the owner's wallet gets.
- A report still goes to a room's moderators and Social's operators; the content policy reports are judged against is
  still the owner's (L8.7).
- Nothing here touches CGT, the chain or DRC-369 state.
