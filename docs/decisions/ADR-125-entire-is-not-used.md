# ADR-125: Entire is not used

**Status:** **Accepted**, 29 October 2026, by the owner, on the assistant's recommendation. The owner asked for it: "Go
with your best recommendation on this one. [...] If you are not seeing any real positive benefit to using Entire, then
let's forget about it and keep on trekking the way we already were." It decides **no economic value**, and loosens no
evidence. **Supersedes [ADR-124](ADR-124-github-stays-the-source-of-truth-entire-mirrors-it.md).**

## Context

ADR-124, the same day, kept GitHub as the source of truth and let Entire, the Git forge launched on 26 October 2026,
mirror this repository through its GitHub app, which had been installed on the organisation on 28 October for all its
repositories; the owner then allowed the app in full. Weighed for this repository on 29 October 2026:

- **Hosting and review** duplicate GitHub, which holds the code, the pull requests, CI (ADR-063), the releases and the
  launcher's update channel, and from which the homepage is deployed (ADR-100).
- **CI through Entire's partners** would replace GitHub Actions, which already runs every pull request here.
- **Agents' sessions recorded beside the commits** would publish transcripts in a public repository, where AGENTS.md
  section 9 keeps tokens, keys, email addresses and other secrets out of every log.
- **Its "Entire Gates" check** needed an approval on Entire, so it marked pull requests neutral or failed (#110).
- **The app** held write access to the organisation's repositories, under terms that said nothing about pricing or
  private repositories.

## Decision

1. **This repository does not use Entire.** GitHub alone, as before 28 October 2026. This supersedes ADR-124.
2. **Entire's GitHub app is removed from the QOR-MATRIX organisation**, from the organisation's settings, the only place
   GitHub lets an owner remove an app.
3. **Entire is looked at again only in a later record.**

## Consequences

- Nothing here read from or depended on Entire, so nothing else changes.
- Pull requests no longer carry the "Entire Gates" check once the app is removed.
