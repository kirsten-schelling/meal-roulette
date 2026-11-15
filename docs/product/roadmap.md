# Meal Roulette — Roadmap & Feature Ideas

> Working doc. Captures the vision, what's built, and where it could go.
> **Provenance:** unmarked features are the original idea list; items tagged _(suggested)_
> are proposals added on top — react to / cut / keep as you like.

---

## The core loop (MVP — mostly built)

Roll a week of dinners from recipes in Postgres, limited per day so **you choose
how easy or ambitious each day is**, then put them on days and generate a **shopping
list** for the week.

Built: Dioxus (Rust) fullstack frontend · Rust/Axum + SQLx backend (Cargo workspace with a
shared `domain` crate) · Postgres, fully migrated · the C solver, wired into `POST /roll` and
the single-day re-roll. Stretch surface: ESP32 kitchen screen.

**Status on `main`:**

- Built: roll a week (`/roll`) under week-level rules (cuisine, protein, must-use ingredient and
  difficulty-band counts, each Required or a High / Medium / Low preference), with unmet
  preferences reported back; seeded rolls; re-roll a day; swap a day for a chosen recipe; pin
  dinners on an unsaved roll so rolling again keeps them; roll presets (one default per
  household); shopping-list generation and check-off; sleep for N weeks and wake; plan history
  (favourite, archive, delete); Library page with the manage-lists dialog; recipe create, edit
  in place and copy; toasts.
- Partial: constraint-conflict prompts (see §1).
- Not built: pinning on a saved plan (`planned_meal.is_pinned` exists, but nothing sets it:
  a saved plan has no whole-week re-roll for a pin to survive); cooking log (`/cooking-log` is
  a placeholder page; `cook_log` has REST CRUD only, nothing writes to it from the app, and the
  roller does not use recency).

Core sub-features:

- Recipe attributes: difficulty, time-to-make, servings/yield, **main protein**, cuisine,
  tags, **healthiness**.
- **Roll, then schedule**: _built._ The roll asks only for how many dinners; the result is
  an unscheduled set, Day 1 to Day N in roll order. A **Put on days** step on the plan page
  takes a start date and lays the dinners on consecutive dates, then any single dinner can be
  moved to another date (a date picker, or drag on a desktop pointer; a date another dinner
  holds is traded) or taken off the calendar. Home shows the dinner scheduled for today, or
  a prompt to put the week on days. A scheduled dinner is labelled with its weekday and date,
  an unscheduled one with "Day N", and History and the shopping list name a plan by its name
  or by when it was rolled.
- **Roll page**: _built._ The page is a roll first: a wheel, "Plan N dinners" and a big Roll
  button (sticky at the bottom on a phone), with the rules folded under it into one line.
  A roll is a preview shown in place and saved only when kept, so rolling again does not add
  plans to History; each dinner can be re-rolled on its own, and preferences the roll could
  not meet are listed in an amber alert. There is one Rules panel with a type-first "Add
  rule" builder: pick Protein, Cuisine, Difficulty, Ingredient or Tag, then the value, then at
  least / at most / exactly and a count ("At least 2 Quick dinners"); each rule is an editable row. A
  rule has one Must toggle, off by default: off is sent as the middle importance level, on as
  the hard one (the four levels stay in the data; saved presets with other levels load as Must
  off). The same builder adds "Every dinner" caps on cook time and difficulty. A failed roll
  shows a persistent error; rules that cannot all hold are flagged before rolling, with a
  "Turn off Must for these" fix; a live count shows how many recipes fit the Must rules.
  Presets are a chip that applies on pick, with a menu to save, update, make default and
  delete, and an "(edited)" marker. The seed is under Advanced.
- **Difficulty by day**: _built, then narrowed._ The solver and the plan still take
  difficulty and max cook time limits per day number, each a hard rule. The Roll page no longer
  sets per-day overrides: its "Every dinner" caps (cook time at most N min, difficulty at most
  a band) are sent as one limit for all days. Presets saved with per-day limits load with the
  all-days limit as the caps; their per-day overrides and easiest band are dropped. Count rules
  per difficulty band still work alongside.
- Shopping list = union of ingredients across the week's recipes.
- **Sleep ingredients (shopping list)** — **MVP**. The list still returns _every_ ingredient
  the week needs, but you can choose **not to add specific items to the shopping list** when
  you know you don't need to buy them for a while (e.g. soy sauce). **Multi-select** items
  and **sleep them for N weeks**, or **set the number of weeks to wait per item**. _Status:
  built._
- _(suggested)_ Seeded, reproducible rolls; **pin** meals you like, **re-roll a single day**,
  or **swap a day for a recipe you pick** — all before the shopping list is generated. Small
  feature, but it's what makes this feel like a product instead of a script. _Status: seed,
  re-roll and swap are built; pin is not._

---

## Legend

| Tag            | Meaning                                                                          |
| -------------- | -------------------------------------------------------------------------------- |
| **[Own]**      | Fully in your control — buildable anytime.                                       |
| **[Fiddly]**   | Feasible but messy — parsing, mobile, or hardware work.                          |
| **[External]** | Depends on third parties with **no official API** — research-grade, ToS caveats. |

Size = rough effort (S / M / L).

---

## The three loops (the spine)

The individual features get much more powerful when they close into loops:

1. **Inventory loop** — purchases (receipt scan / barcode / confirm) → **pantry state** →
   shopping list subtracts what you already have → "use it up" biases the roller → cooking
   decrements the pantry. Nail this and the app stops being a randomizer and starts
   _managing your kitchen_.
2. **Preference loop** — cook → rate → the roller weights future weeks. Turns randomness
   into recommendation.
3. **Cost loop** — pricing → budget constraint + ingredient-overlap batch-cooking → cheaper,
   lower-waste weeks.

---

## Feature catalog

### 1 · Planning & the roller

- **Leftovers-aware planning** — **[Own]** M. Recipes declare a yield; mark surplus
  servings as weekday lunches; the roller guarantees enough coverage for the week.
- **Roll from a chosen ingredient** — **[Own]** M. The user names an ingredient they want to
  use this run (e.g. sour cream) and the roller **guarantees a recipe that uses it** — or
  **N recipes** if they ask for several. Supports **multiple ingredients** and can span
  **multiple weeks** in one run. Like "use it up" (§3) but **user-driven and a hard
  requirement**, not a soft bias.
- _(suggested)_ **Dietary & preference constraints** — **[Own]** M. Veg / allergy /
  "no fish for partner" / cuisine rotation, all fed into the solver.
- _(suggested)_ **Ratings & feedback** — **[Own]** M. Rate after cooking; weight future
  rolls toward liked, away from disliked; recency derives from `cook_log`.
- **Min/max cook interval** — **[Own]** M. Per-recipe **minimum gap** ("don't repeat within
  N weeks") and **maximum gap** ("cook at least every M weeks"), on top of the recency
  weighting. Default to friendly presets (e.g. _rarely / sometimes / often_), with an
  **advanced setting** to specify exact weeks per recipe.
- _(suggested)_ **Busy-day / calendar awareness** — **[Own→Fiddly]** M. "Eating out
  Friday" → roll 6 days not 7; optional Google Calendar push of the plan.
- _(suggested)_ **Budget as a constraint** — **[Own→External]** M. Keep the week under $X.
  Works today with manual per-recipe cost estimates; the automatic version needs pricing
  (see §4).
- **Per-meal cost from ingredient costs** — **[Own]** M. Enter a cost per ingredient; a
  meal's cost = the sum of its ingredient costs, and a week's cost rolls up from the meals.
  This is the manual foundation the budget constraint uses; automatic pricing — the **base
  ingredient library** (§2) fed by scraped data (§4) — would replace the manual entry later.
- _(suggested)_ **Batch-cook / ingredient-overlap optimisation** — **[Own]** L. Bias the
  week toward recipes that _share_ ingredients, to cut cost and waste. A genuinely
  interesting solver objective to write in Rust.
- _(suggested)_ **Prep-ahead / defrost awareness** — **[Own]** M. Flag recipes that need an
  overnight step (marinate / soak / defrost) and surface a "do this tonight" nudge the day
  before. Reuses time-to-make data.
- _(suggested)_ **Seasonality bias** — **[Own→External]** M. Prefer in-season produce
  (cheaper, fresher) via a season→ingredient map; a deeper version leans on scraped pricing
  (§4).
- _(suggested)_ **Exploration ↔ comfort dial** — **[Own]** S. A "surprise me ↔ safe" slider
  trading novelty against known favourites — leans into the _roulette_ identity; pairs with
  ratings.
- **Constraint-conflict prompts** — **[Own]** M. When honouring a request collides with a
  rule — e.g. the only recipe with an ingredient you want to use up is still inside its **min
  interval**, or every candidate is filtered out by available time — surface a **warning with
  choices** instead of silently failing: cook it anyway, add a new recipe, or pick a
  different existing one. _Status: partial. The roll reports unmet or loosened rules back to
  the Roll page; the "cook it anyway / add a recipe" choices are not built._

### 2 · Recipes & data entry

- **Main protein** — **[Own]** S. Record each recipe's main protein as a first-class
  attribute (chicken / beef / tofu / …). The roller uses it to **rotate proteins across the
  week** (no chicken four nights running).
- **Healthiness rating** — **[Own]** S. A per-meal healthiness rating (simple scale for
  now); the roller can **cap or balance less-healthy meals per week**. _Future:_ derive the
  rating from the recipe's **ingredients** rather than a manual per-meal score — ideally
  without making the user rate every ingredient by hand — via the **base ingredient library**
  (below).
- **Dishes rating** — **[Own]** S. How much washing-up a recipe creates (one-pot →
  every-pan-you-own), as a simple per-recipe scale. A roll setting ("keep the dishes down")
  biases or caps high-cleanup meals — per plan or per day, in the same spirit as
  difficulty-by-day.
- **Flavour profile** - **[Own]** S. A flavour profile per ingredient (eg Sugar = Sweet,
  Tomato = Acidic) to classify flavour profiles of recipes. Can be used for future recipe
  recommendations, based on a user's favourite or commonly cooked dishes.
- **Base ingredient library** — **[Own→External]** L. A canonical, app-maintained set of
  common ingredients that users **extend with their own** when something's missing. Each
  ingredient carries structured data — category, **main-protein** tag, **health** data and
  **price** — so recipes reference one shared source instead of everyone re-entering the same
  facts. This is the backbone behind **per-meal cost** (§1), the **ingredient-derived
  healthiness** upgrade (above) and protein tagging.
  - **Multi-country / multi-currency** — prices (and product availability) vary by region, so
    the library must model **currency and locale** from the start, not bolt it on.
  - **Data sourcing: scrape, don't trust user input.** Hand-entered per-ingredient prices
    won't stay accurate or consistent, so the realistic path is **scraping** price/product
    data — which lands squarely in the §4 reality-check (no official supermarket APIs,
    undocumented endpoints, ToS-grey, store-location dependent). User-added ingredients still
    carry their own health rating and can fall back to manual price where no scrape exists.
  - **Scope:** genuinely a **big project** / its own track — sequence it after the core loop
    and the manual-cost foundation, alongside the §4 pricing research.
- _(suggested)_ **Library housekeeping card** — **[Own]** S. Home shows a small card only when
  the library needs attention — unused or near-duplicate cuisines / proteins / units,
  ingredients with missing data — and clicking a finding jumps into the fix (the Library
  page's manage dialog). Invisible when everything is clean: the app points at problems
  instead of offering admin lists to patrol. _Status: not built (#17)._
- _(suggested, high value)_ **Recipe import from URL** — **[Own]** M. Most recipe sites
  embed `schema.org/Recipe` as JSON-LD; parse title / ingredients / steps / time / yield
  straight out of the page. This kills the single biggest friction of any recipe app —
  hand-entering recipes — so I'd put it very early.
- _(suggested)_ **Cook mode** — **[Own]** S. Step-by-step view, scale servings,
  keep-screen-awake. Shared by the mobile app and the kitchen screen.
- _(suggested)_ **Ingredient substitutions (at plan time)** — **[Own]** M. When a meal is
  selected for the week, let the user **swap one of its ingredients** for another; the change
  flows straight into the shopping list. Start with users defining their own subs; how this
  generalises depends on the **base ingredient library** direction (above). The immediate
  point is building the list from _your_ version of the selected recipes.
- _(suggested)_ **Cooking notes over time** — **[Own]** S. A per-recipe running log of
  timestamped notes ("used less chili last time"). Separate from editing the recipe itself
  (users can also just adjust a recipe's ingredients) — this is the history, not the recipe.
- _(suggested)_ **Nutrition (macros / calories)** — **[Own→External]** M. Beyond the health
  _rating_ above, per-recipe macros / calories derived from the **base ingredient library**.

### 3 · Pantry & inventory

- **Pantry tracking** — **[Own]** M. Track what you have based on confirmed purchases
  (user confirms they bought for a given recipe); the shopping list then subtracts pantry
  stock so you don't re-buy.
  - **Decrement on cooking**, driven by the recipe's ingredients.
  - **Always allow manual override** — the user can add an item to the list even when the
    app thinks it's still stocked.
  - **Correctable estimates** — the pantry count is an _estimate_ (it assumes you cooked
    what was planned), so let the user fix counts when they cooked something unplanned since
    the last purchase.
- **Sleep as lightweight tracking** — **[Own]** S. Full pantry tracking (above) is
  inherently laborious and hard to make effortless/automated. **Sleeping ingredients** (MVP,
  see the core loop) is the low-effort middle ground: for users who don't want detailed
  manual tracking, choosing to sleep an item — and later **accepting it back onto the list**
  when you actually need it — is itself a simple form of "do I have this?" tracking.
- **Staples predictor** — **[Own]** M. Track non-recipe items (milk, paper towel, etc.)
  and typical usage cadence; predict run-out and auto-suggest adding to the list.
- _(suggested)_ **"Use it up" bias** — **[Own]** S. Bias the roll toward recipes that use
  soon-to-expire pantry items. Closes the inventory loop cheaply.
- _(suggested)_ **Ingredient shelf-life & use-up suggestions** — **[Own]** M. A "how long
  this lasts" value on the **ingredient table** predicts when a bought item will go off; use
  it to **suggest ingredients to use up** based on recent weeks' shopping (feeds "roll from a
  chosen ingredient", §1). **Use-by is optional** per ingredient and off by default — keep
  the noise down.
- **Receipt OCR (mobile)** — **[Fiddly]** L. Camera → OCR (Google ML Kit on-device, or a
  cloud Vision API) → parse line items → update pantry. Parsing is the hard part
  (each retailer's receipt format differs). Note: the **digital receipts inside the loyalty
  apps are already structured** — a cleaner data source than OCR _if_ you can get at them
  (see §4).
- _(suggested)_ **Barcode scanning** — **[Fiddly]** M. Camera barcode + **Open Food Facts**
  (free, open product database with an API) for reliable per-item pantry adds. More accurate
  than OCR for individual items; complements it.

### 4 · Money & shopping integrations — _the hard bucket_

> **Reality check:** none of the AU supermarkets or loyalty programs offer an official
> public API for any of this. Everything here is research-grade — treat as spikes, not
> commitments, and check the ToS.

- **Loyalty link — Everyday Rewards / Flybuys** — **[External]** ?. No official third-party
  API for purchase history. The only routes are **unofficial/reverse-engineered app
  endpoints** (fragile, likely against ToS) or **manual import/export**. Aspirational — but
  the structured digital receipts these apps hold are the real prize if a route exists.
- **Price comparison — Woolworths / Coles / Aldi** — **[External]** L. No official pricing
  API. Woolworths & Coles run **undocumented internal JSON APIs** (the ones their own
  sites/apps call) that people scrape: undocumented, ToS-grey, rate-limited, and
  **store-location dependent**. **Aldi is the hardest** (limited online catalogue). Before
  building your own scraper, check whether an existing AU grocery price-comparison service
  already exposes usable data.

### 5 · Surfaces

- **Mobile app** — **[Fiddly]** M. Native Dioxus mobile target (same codebase as web/desktop).
  Unlocks the camera (OCR / barcode), share-sheet recipe import, push notifications, and an
  offline shopping list for in-store with bad signal.
- **Kitchen screen** — **[Fiddly / Hardware]** L. Show day + planned meal; display the
  recipe while cooking; add recently-made recipes.
  - **Display:** e-ink is lovely for a _passive, always-on "this week"_ panel (low power)
    but refreshes too slowly to scroll a recipe — a small **TFT/LCD** or a repurposed
    **tablet** is better for interactive cook mode. Possibly two modes / two devices.
  - **Input:** a full keyboard is awkward on embedded. Simpler: a **rotary encoder +
    button**, a few **GPIO buttons**, or a **touchscreen** — or use your **phone as the
    remote** (the mobile app drives the screen). Voice is a stretch goal.
  - Reuses the shared **`domain` crate** — the device deserializes the exact types the API
    serializes. This is the whole reason the backend is Rust.
- _(suggested)_ **Physical re-roll button / rotary control** — **[Fiddly / Hardware]** S. A
  satisfying tactile "spin" on the kitchen screen to re-roll a day — small, fun, and real
  embedded practice.
- _(suggested)_ **NFC / RFID pantry tap** — **[Fiddly / Hardware]** L. Tap an item when you
  use it to decrement the pantry — an embedded alternative to barcode / OCR (§3).

### 6 · Household & collaboration

- **Household with multiple members** — **[Own]** M. One household, several members. The
  **main cook** does most of the planning; other members can **suggest recipes** (into a
  shared pool the cook can pull into the roll or pin) and **veto** recipes or specific
  planned meals (excludes them / triggers a re-roll of that slot). Requires users + auth;
  `household_id` is already on the core tables, so ownership isn't a retrofit.
- **Per-day cook assignment** — **[Own]** M. Assign who's cooking on which days ("Tuesday
  is Sam's night"); the roll respects it.
- **Per-user recipe affinity** — **[Own]** M. Track which recipes each member _likes to
  make_ / is comfortable cooking. When a slot is assigned to a member, the roller prefers
  from that member's recipes.
- **Public recipes & recommended periods** — **[Own]** L. _Future._ Let users make recipes
  **publicly available** beyond their household. On top of that pool, the app recommends a
  **full week (or any chosen period) in one shot** from the household's settings, goals and
  history — drawing on both the own library and public recipes, so a plan arrives ready to
  accept or tweak instead of being assembled by hand. The endgame of the **preference loop**.

> These, plus difficulty-by-day, leftovers, dietary constraints, ratings and vetoes, all
> become weighted **inputs to the roller** — which is where the interesting Rust logic lives.

### 7 · Shopping list

> The list is generated from the week's recipes; **sleep ingredients** (MVP, core loop)
> already lets you suppress items you don't need to buy yet.

- _(suggested)_ **Quantity aggregation + unit math** — **[Own]** L. Sum quantities for the
  same ingredient across the week's recipes and reconcile units. Density makes ml↔g
  **product-specific** (water 1:1, flour not), so two modes:
  - **Simple (default):** the user picks one unit per ingredient and enters _everything_ —
    recipe amounts and the ingredient itself — in that unit, so no conversion is needed.
  - **Advanced:** ship a set of standard conversions, and let users add their own
    **per-product** conversions where the standard ones don't fit.
- _(suggested)_ **Customisable list display** — **[Own]** M. Let users define **their own
  sections and grouping** for the list — one person groups into _Longlife / Fresh / Meat_,
  another wants different buckets. Ingredient categories from the library feed this, but the
  grouping stays user-configurable.
- _(suggested)_ **Ad-hoc / non-recipe items** — **[Own]** S. Manually add items that aren't
  tied to any recipe (bin bags, foil) to the week's list.

### 8 · Insights & goals

- _(suggested)_ **Insights dashboard** — **[Own]** M. Trends that make the attributes pay
  off: protein mix, cuisine spread, healthiness over time, weekly spend, most / least cooked.
- **Goals & targets** — **[Own]** M. Let the user set **per-meal or per-week goals** — e.g.
  _max one Chinese dish per week_, _≥ 40 g protein per meal_ — that constrain the roll. All
  **overridable in advanced settings when they spin the wheel**. (A roller input, surfaced
  here alongside the insights that track progress against it.)

---

## Suggested roadmap ordering

Front-loads value; each step is a clean learning milestone. Ordering is **author-first** —
built for personal use, so it leads with what the author values most.

0. **Core roller + shopping list** — includes difficulty-by-day (+ override)
   and sleep-ingredients on the shopping list. _Roller, shopping list and sleep are built;
   difficulty-by-day as specified is not (see above)._
1. **Household + auth foundations** — users + auth on top of the `household_id` ownership
   already in the schema
2. **Recipe import from URL** — removes the data-entry wall so the app is actually usable
3. **Leftovers + dietary constraints** — makes the roll genuinely useful
4. **Pantry + staples predictor + "use it up"** — the inventory loop (with manual override); starts saving real effort
5. **Collaboration: suggest / veto, cook assignment, per-user prefs** — the member-facing side, all feeding the roll
6. **Ratings / feedback** — the roll gets smarter each week
7. **Mobile wrap** — unlocks camera + offline; then receipt OCR / barcode for low-friction inventory
8. **Kitchen screen** — the embedded payoff
9. _(parallel research track, optional)_ **base ingredient library + pricing + loyalty** —
   multi-currency, scraped ingredient/price data; spikes, not promises

---

## Decisions

- **Household, not single-user.** Several members; the main cook plans; others can suggest
  recipes and veto. **Per-day cook assignment** and **per-user recipe affinity** feed the
  roll (whose night it is → prefer recipes they like to make). → build users/auth + household
  ownership into the schema from the start (§6).
- **Difficulty by day, chosen at roll time.** The user sets difficulty and max cook time
  per day number on the roll form (a default for all days plus per-day overrides); nothing
  is derived from the weekday.
- **Recipe sources: both** — hand-entered _and_ imported from URL.
- **Pantry decrement: recipe-driven, with user override.** Cooking a planned recipe
  decrements its ingredients; the user can always add an item to the list even when the app
  thinks it's stocked, and can correct counts when they cooked unplanned things.
- **Pricing: manual entry for now.** Automatic pricing (§4) deferred — decide when you reach it.
- **Dinners first, me-centric.** The MVP rolls **dinners only** — this is built for the
  author's own use first, so the roadmap is prioritised by what the author values most, with
  breakfast / lunch and the broader add-ons sequenced after. Rolling other meals is an
  **option to add later**.
- **Lookup management: Library page + manage dialog, not per-entity pages.** Ingredients get
  the full page (they grow into the base ingredient library, §2); cuisines / proteins / units
  are thin name lists managed in a tabbed dialog, reached from the Library page and from
  "Manage …" portals in the recipe form's comboboxes. Creation stays inline at the point of
  use (type + Enter in a recipe field); cleanup work surfaces through the housekeeping card
  (§2) rather than admin browsing. Built: rename and delete in the dialog (#16), usage counts (#15),
  inline unit create (#20), a 409 on deleting an in-use entry (#14). Open follow-ups: merge
  duplicates (#18), cascade delete (#23), a shared in-use message (#26), ingredient editing on
  the Library page (#19), a recipe search bar (#22).

## Still open (deferred)

- **Kitchen-screen hardware + input method** — a _later_ hardware decision; nothing needed
  now. (Which device displays it — e-ink panel vs small TFT/LCD vs a repurposed tablet — and
  how you interact — a few buttons, a rotary encoder, a touchscreen, or just your phone as
  the remote.)
