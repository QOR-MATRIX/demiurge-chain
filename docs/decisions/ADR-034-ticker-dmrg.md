# ADR-034: The ticker is DMRG; the name stays Creator-God Token

**Status:** Accepted, 17 September 2026, by the project owner. **Superseded by
[ADR-045](ADR-045-the-ticker-returns-to-cgt.md)** (21 September 2026): the ticker returned to `CGT`. The text below is
kept as accepted.
**Follows:** [ADR-008](ADR-008-language-discipline.md) (the currency is infrastructure, not an instrument),
[ADR-013](ADR-013-polkadot-sdk-migration.md) (the custom chain is being replaced),
[ADR-024](ADR-024-ss58-addresses-prefix-42-until-mainnet.md) (where the collision was found).
**Supersedes nothing.** It settles U-13 in [`docs/economics/OPEN_QUESTIONS.md`](../economics/OPEN_QUESTIONS.md), which
holds the full research.

## Context

The ticker was `CGT`, the initials of Creator-God Token. The `paritytech/ss58-registry` records `CGT` for Curio, at
prefix 777 with eighteen decimals, found on 15 September 2026 while researching address prefixes. Researched again on
17 September at the owner's instruction, to establish whether sharing the symbol causes real problems.

What is **not** at stake was established first, because it bounded the question: nothing reserves a ticker, no
aggregator requires uniqueness, no listing policy was found that refuses a collision, and in Substrate tooling a
shared symbol cannot produce a wrong balance, because chains are identified by genesis hash and wallets read symbol and
decimals from the chain.

What is at stake is paid on surfaces the project does not control:

- **Symbol-keyed price lookup silently resolves to the larger asset.** CoinGecko's `/simple/price` accepts a symbol and
  returns the top-ranked match, with no error. That asset is not ours and need never be.
- **An exchange renames the later arrival, unilaterally.** MEXC runs a standing category for it; Binance has renamed an
  incumbent for a newcomer. Which side is renamed is not ours to decide.
- **The symbol carries another project's story.** Curio's `CGT` was exploited in March 2024 for about $16M through a
  voting access-control flaw that minted roughly a billion tokens, and relaunched as "CGT 2.0". For a currency whose
  argument is that it is not a speculative instrument (ADR-008), inheriting a governance-exploit narrative is a poor
  trade for three letters.
- **The collision is live in shipped software,** not only in an archived registry: SubWallet's chain list carries `CGT`
  today.

Changing after listings exist is the expensive path, evidenced by MATIC→POL and FTM→S: migration contracts,
per-venue suspension windows, a phishing surface, and at least one delisting. Changing before launch costs a constant.
Nothing is listed, no SDK is published and no genesis exists.

## Decision

1. **The ticker is `DMRG`.** It is Demiurge compressed, so it points at the ecosystem rather than at the token's name,
   and it survives a later change of the token's name.
2. **The name stays Creator-God Token.** Only the three-letter symbol changes.
3. **The change is made now, before launch,** for the three reasons above: the Curio collision, the March 2024 exploit
   attached to that symbol, and aggregator symbol resolution to the higher-ranked asset.
4. **The swap covers what a person sees, and nothing else.** The declared symbol, the hard-coded fallbacks behind it,
   and the ticker written into copy.
5. **Identifiers, commands, RPC names and chain constants are deliberately left reading `cgt`.** `SPARKS_PER_CGT`,
   `format_cgt`, `parse_cgt`, `amount_cgt`, the launcher commands `cgt_send`, `cgt_balance`, `cgt_nonce`,
   `cgt_history`, `cgt_claim_starter`, `cgt_parse_amount`, the RPC name `admin_mintCgt`, `ActionType::TransferCgt` and
   the `CGT` unit constant all stay. They live in code the Substrate implementation replaces wholesale (ADR-013), so
   renaming them now is churn on names that disappear.
6. **The Substrate implementation uses `DMRG` from the start,** in identifiers, commands, RPC names and constants.
   There is no second rename pass, because those names are written fresh. This is recorded as a requirement in
   [`MIGRATION_INVENTORY.md`](../architecture/MIGRATION_INVENTORY.md).

## Clearance research

`DMRG` was researched on 17 September 2026 before it was adopted, against the rule that a candidate must not reproduce
the problem being escaped and must be clear on trademark, crowding, Substrate tooling and this system's own names. The
full findings, sources and method are in U-13. In summary, nothing bars it:

- **No live trademark on the string, anywhere, in any class.** Through TMview, over the EU, US, UK, WIPO and national
  registers: a Turkish class 9 mark has ended, and the two other hits are different strings. Nothing in classes 36 or 42.
- **No crypto asset uses it.** Absent from CoinGecko's active universe of 21,260 coins and from every chain DexScreener
  indexes, so a symbol lookup resolves to nothing.
- **Absent from Substrate tooling:** the ss58 registry, Talisman's chain data and its symbol-keyed icon slots, and
  SubWallet's chain list. All three carry `CGT`.
- **No regulatory action, exploit or collapse** attaches to the string. Its search results are the density-matrix
  renormalization group, a 1992 physics algorithm: a cost in search visibility, not a conflict.
- **No collision inside this system.** The atomic unit is the Spark and the whole unit is CGT; `DMRG` is neither. This
  check disqualified `SPRK`, whose four letters would have sat beside a Spark worth `10^-18` of one.

The method was validated before it was trusted: the same query reproduces EUTM 018611688, "EMBER TOKEN", Apex Group
ApS, classes 9, 36 and 42, the registered mark that disqualified the previous candidate.

## What this decision does not clear

**The project's name, Demiurge, carries live trademark exposure.** The string `DEMIURGE` is live in classes 9, 41 and
42: a US application by Demiurge Studios, Inc. (98610467), past publication and notice of allowance with a first
statement-of-use extension granted, and a composite mark registered in Switzerland and as international registration
1305546 by Demiurge Technologies AG.

The owner weighed this and it does not block the ticker, on three distinctions: the exposure is on the project's name
rather than on the ticker; it exists today whatever the ticker becomes; the goods are games, not currency; and **no one
holds either string in class 36**, the class a currency occupies. That is what separates it from the rejected candidate
Ember, where the registered phrase was the one our own copy would have used, and it covered class 36.

It is carried consciously rather than discovered at mainnet:

- The research is a screening for **identical marks only**. No register was searched for merely similar marks, and
  confusing similarity is the ground most refusals rest on.
- **No attorney opinion was obtained.**
- **A clearance opinion on the project name is a Public Release gate criterion** (`docs/GATES.toml`,
  `public-release.name-clearance`).

## Consequences

- **The devnet reports `DMRG`.** `framework/primitives/src/denomination.rs` declares it, so the chain's properties, the
  amounts it formats and its error messages all read `DMRG`. The chain is otherwise untouched; this is a display
  constant, not a change to consensus, networking, storage or RPC (ADR-013).
- **The launcher shows `DMRG`,** both where it reads the symbol from the host and where it had fallen back to a
  hard-coded string. The fallbacks were found during this work: the record had said the views always read the symbol
  from the host, and nine of them did not.
- **`docs/economics/CGT.md` is now `docs/economics/DMRG.md`,** so no current document is named for a retired ticker.
- **The earlier ADRs are not rewritten.** ADR-002 to ADR-008, ADR-029 and ADR-030 discuss the currency as `CGT`; they
  are records of decisions taken then, and this ADR is what changes the symbol.
- **One test was repaired rather than broken.** `registration-mints-nothing.mjs` asserted that a registration response
  never mentions `cgt`. After the rename that assertion would have passed while testing nothing, so it now checks every
  name the currency answers to, with a note to extend it if the ticker changes again.
- **No migration, no holders, no listings.** Nothing exists to convert, which is the whole reason for doing it now.

---

## Appendix: the research behind this decision

This was U-13 in [`docs/economics/OPEN_QUESTIONS.md`](../economics/OPEN_QUESTIONS.md) until this ADR settled it. It is
carried here in full, rather than deleted with the item, so that the ticker is not re-litigated. Two corrections the
research made to its own earlier claims are kept with it, because the corrections are the part most easily lost.

### A.1 What is not at stake

Verified.
- **Nothing reserves a ticker.** No authority grants exclusivity. CoinGecko states it does not allow any team to
  reserve a symbol, because it supports several tokens on one. CoinMarketCap: "An asset does not need to have a unique
  ticker symbol in order to be listed." Its one hard bar is colliding with a *stablecoin* ticker, which `CGT` does not.
- **Duplicates are the normal condition.** ISO 24165 exists precisely because "token names and tickers are not unique";
  its answer is a nine-character meaningless identifier, not a naming rule. In April 2018 twenty CoinMarketCap symbols
  each mapped to two coins at once.
- **In Substrate tooling a collision is harmless.** Chains are identified by genesis hash, and decimals, symbol and
  prefix are read from the chain's own `system_properties`. Talisman states it fetches all of them from the chain;
  SubWallet keys on slug and genesis hash. So a second `CGT` cannot produce a wrong balance or wrong decimals in our
  wallets. No such incident was found.
- **Asset Hub and XCM are designed for it.** The Polkadot wiki: "The `AssetId` should be the canonical identifier for
  an asset, as the chain does not enforce the uniqueness of metadata like 'name' and 'symbol'." Foreign assets are
  keyed by XCM location. No failed transfer from a symbol collision was found.
- **The archived registry never required unique symbols** either; it enforced uniqueness only on prefix and network
  name, and already carries duplicates (`XOR`, three SORA networks).

**What is at stake. Verified unless marked.**
- **Symbol-keyed price lookup silently resolves to the bigger asset.** CoinGecko's `/simple/price` accepts a symbol,
  and that lookup defaults to the top-ranked token by market cap or volume. An integrator who queries by symbol rather
  than by id gets whichever `CGT` is larger, with no error. Today that is not ours, and it need never be.
- **Exchanges rename the newcomer, unilaterally.** MEXC runs a standing "token renaming" category and has repeatedly
  suffixed the later arrival (`AI`→`AICOIN1`, `SYNC`→`SYNCHRONI`, `ANON`→`SUPERANON`). Binance renamed the incumbent
  `BTT`→`BTTOLD` for the newcomer. Coinbase was described by its own spokesperson as first come, first served. Which
  side gets renamed is not ours to decide. (Inferred from these cases: a listing would probably carry `CGT`, but a
  suffix is a live possibility.)
- **A few surfaces key on the symbol string.** Talisman's token icons live at `assets/tokens/${symbol}.svg`, so two
  chains contend for one icon path. Verified on 17 September 2026: the directory holds 139 files, each named for a
  lower-cased symbol. The contention is a risk rather than a fact here, because `cgt.svg` is unclaimed; Curio never
  submitted one. Subscan requires submitted token info to match on-chain data and reviews it by hand, so a second
  `CGT` is arbitrated by a person.
- **Client libraries have broken on this repeatedly.** ccxt once dropped twenty coins entirely because each symbol
  mapped to two assets; rotki showed `SOL` holdings under the wrong project's name; Portfolio Performance returned
  incorrect quotes for `BIT`. These are third-party bugs we would inherit, not bugs we could fix.
- **`CGT` is crowded, and Curio is not the incumbent that matters.** At least ten distinct assets use it: Curio (two or
  three identities of its own), CACHE Gold (gold-backed, with a Coinbase price page), Collision Token, Clinq.Gold
  Token, Coin Gabbar, CZ GROK, Crypto Gaming Token, CryptoGamesToken, Cryptogene Token, ChainGuard. Curio's own CGT is
  a sub-$1M asset whose published market cap is variously $56K to $651K.
  **Re-checked 17 September 2026, and narrowed.** That count came from aggregator pages and search. It could not be
  reproduced from CoinGecko's API, whose active universe of 21,260 coins now carries no `CGT` at all, inactive and
  delisted assets being behind its paid plan; so the ten are not all live today and the figure should not be repeated
  as a count of live assets. What is verified live is narrower and enough: DexScreener still serves Curio Governance
  Token on BSC under `CGT`, with about $13 of liquidity, and SubWallet's shipped chain list carries `CGT` among its
  1,180 assets. The symbol is taken in the tooling our own users would run.
- **Association is the underrated cost.** Curio's `CGT` was exploited in March 2024 for about $16M through a voting
  access-control flaw that minted roughly a billion tokens, and relaunched as "CGT 2.0". A search for our ticker
  surfaces that story and a gold-backed token. For a currency whose whole argument is that it is not a speculative
  instrument (ADR-008), inheriting a governance-exploit narrative is a poor trade for a three-letter string.
- **Nobody can stop us, and we cannot stop anyone.** No trademark on `CGT` in this space was found. Trademark fights
  over tickers exist and are expensive: Telegram paid about $625,000 in fees to drop its `GRAM` claim.

### A.2 Why changing later is the expensive path

Verified. MATIC→POL (September 2024) needed a migration contract for every
holder outside Polygon's own chain, per-exchange suspension windows set independently (about nine days of degraded
service at one venue), and a phishing surface during the window; a year on, a co-founder publicly floated reverting
because users had lost the thread. Fantom→Sonic (2025) saw at least one venue delist rather than migrate, and warned
that depositing the old ticker afterwards could lose funds.

### A.3 What the swap actually costs, corrected

**Changing now is cheap, but it is not two constants.** Corrected on 17 September 2026 against the repository. U-13 had said the
ticker was two constants and that the launcher's views read the symbol from the host. The first half is the smallest
part of the truth and the second is only half true. The symbol is declared twice,
`framework/primitives/src/denomination.rs:72` and `tools/qor-launcher/src-tauri/src/cgt.rs:50`, and ten places in the
launcher's views do read it from the host, but each of those reads falls back to a hard-coded `'CGT'`, and about
fifteen further places write the ticker into prose that no host value reaches ("CGT Vault", "settled in CGT", the
seed-phrase warnings). Beyond the displayed string, the currency's name is built into identifiers on both sides:
`SPARKS_PER_CGT`, `format_cgt`, `parse_cgt`, `amount_cgt`, `cgt_balance` and the launcher commands `cgt_send`,
`cgt_nonce`, `cgt_history`, `cgt_claim_starter`, `cgt_parse_amount`, and on the chain the RPC names `transferCGT` and
`admin_mintCGT`. The repository carries 1,643 occurrences of the string across 284 files, most of them in
documents, in decision records that are never rewritten, and in frozen scope that is not touched. By zone, in
occurrences: `framework` 146, `tools/qor-launcher` 82, `services/qor-auth` 16, `docs` 366, and 942 in the frozen
`apps`, `client`, `cli`, `packages` and `sdk`. **What must change
for the ticker to change, and what must not, is an owner's decision recorded when the swap is made.** Nothing is
listed, no SDK is published, no genesis exists, and the framework chain is being replaced anyway, so the moment is
still the cheap one.

### A.4 The candidates rejected before DMRG

The rule applied to each: a candidate must not reproduce the problem being escaped, and must be clear on trademark,
crowding and Substrate tooling. Each was researched before any code changed.

- **Ember (ticker SPRK), proposed 17 September 2026, rejected the same day.**
  - **Trademark, the disqualifying finding.** EUTM 018611688, the registered word mark **"EMBER TOKEN"**, owner Apex
    Group ApS, registered 22 March 2022, in force across the EU until 30 November 2031, in Nice classes 9, 36 and 42:
    software, financial services, computer services. That is the phrase our documents would use, in the classes a
    currency occupies. Mitigations exist and none is a clearance: the owner is the defunct Ember Sword studio, an EU
    non-use revocation window opens around March 2027, and EUIPO does not refuse later marks on its own motion.
  - **Ember.js is not the conflict**, which was checked specifically. Tilde Inc.'s two US registrations (4462784,
    4462786) are live, renewed and incontestable, but class 9 only and scoped to "computer programs that implement a
    computer programming language", with no enforcement found.
  - **It would not have escaped the problem.** Four CoinGecko coins carry the symbol EMBER, among them CoinGecko's own
    collision suffixes `ember-3` and `ember-4`; DexScreener lists fourteen distinct EMBER tokens. Coinbase already
    serves both `/price/ember` and `/price/ember-base-0x7ffbe850…`, the later arrival carrying a contract address in
    its URL. A symbol lookup resolves to embercurve, an anonymous-team Solana launchpad of about $12.4M.
  - Beyond crypto the name is busy in finance and hardware: Ember (UK accounting, acquired by Starling Bank in August
    2025), Ember Fund, ember.so, Ember Technologies.
  - **SPRK** itself was mechanically clean — no stablecoin, no top-300 asset, no live class-36 mark, nothing in the
    SS58 registry or either wallet's chain data — with one live incumbent (SparkDEX on Flare, about $5.6M) and one
    blemish: Sparkster, whose "SPRK tokens" are named in the SEC's September 2022 settlement over a $35M unregistered
    offering, still ranking in search.
  - **SPRK also collided inside this system.** The atomic unit is already the Spark, so `1 CGT = 10^18 Sparks`
    (`framework/primitives/src/denomination.rs:45-51`). A ticker of `SPRK` for one whole unit, beside a Spark worth
    `10^-18` of one, would differ by a factor of a billion billion in every amount shown, logged or parsed. Verified
    (repo). Any future candidate is checked against the system's own names for this reason.

### A.5 When this would have been foreclosed

**Foreclosed by:** the first exchange listing, SDK publication and mainnet genesis, whichever comes first. The ticker
enters chain properties, wallets' network lists and every client that displays amounts. It remains a Public Release
criterion in `docs/GATES.toml`, as a backstop; the owner intends to decide it sooner (17 September 2026).

### A.6 Sources

**Sources** (read 17 September 2026):
[CoinMarketCap listings criteria](https://support.coinmarketcap.com/hc/en-us/articles/360043659351-Listings-Criteria) ·
[CoinGecko on reserving a symbol](https://support.coingecko.com/hc/en-us/articles/4498962550681-Can-I-reserve-a-token-symbol-or-use-another-token-s-symbol) ·
[CoinGecko `/simple/price`](https://docs.coingecko.com/reference/simple-price) ·
[Asset Hub: AssetId is canonical](https://wiki.polkadot.com/learn/learn-assets/) ·
[Talisman chaindata](https://github.com/TalismanSociety/chaindata) ·
[MEXC token renamings](https://www.mexc.com/announcements/tag/token-renaming-35) ·
[MATIC→POL migration](https://support.coingecko.com/hc/en-us/articles/36751952559385-Migration-of-MATIC-to-POL-Completed) ·
[Polygon co-founder on reverting](https://cointelegraph.com/news/polygon-founder-matic-poll-revert-debate) ·
[FTM→S, and a delisting](https://www.coinjar.com/uk/learn/fantom-to-sonic) ·
[the Curio exploit](https://www.halborn.com/blog/post/explained-the-curio-hack-march-2024) ·
[ISO 24165 digital token identifiers](https://www.iso.org/standard/85546.html) ·
[Telegram's `GRAM` settlement](https://www.financemagnates.com/cryptocurrency/news/telegram-drops-gram-trademark-lawsuit-paying-625k-to-defendant/)
