# Meal Roulette — Roadmap & Feature Ideas

> Working doc. Captures the vision, what's built, and where it could go.
> **Provenance:** unmarked features are the original idea list; items tagged _(suggested)_
> are proposals added on top — react to / cut / keep as you like.

---

## The core loop (MVP — building now)

Roll a week of dinners from recipes in Postgres, weighted so **weeknight meals are
quick/easy** and **weekend meals can run longer/ambitious**, then generate a **shopping
list** for the week.

Already scaffolded: Dioxus (Rust) fullstack frontend · Rust/Axum + SQLx backend (Cargo
workspace with a shared `domain` crate) · Postgres. Stretch surface: ESP32 kitchen screen.

Core sub-features:
- Recipe attributes: difficulty, time-to-make, servings/yield, **main protein**, cuisine,
  tags, **healthiness**.
- **Difficulty by day**: defaults to easier dishes on weeknights, harder on weekends —
  but **user-overridable** per day / per plan.
- Shopping list = union of ingredients across the week's recipes.
- **Sleep ingredients (shopping list)** — **MVP**. The list still returns *every* ingredient
  the week needs, but you can choose **not to add specific items to the shopping list** when
  you know you don't need to buy them for a while (e.g. soy sauce). **Multi-select** items
  and **sleep them for N weeks**, or **set the number of weeks to wait per item**.
- _(suggested)_ Seeded, reproducible rolls; **pin** meals you like, **re-roll a single day**,
  or **swap a day for a recipe you pick** — all before the shopping list is generated. Small
  feature, but it's what makes this feel like a product instead of a script.

---

## Legend

| Tag | Meaning |
|-----|---------|
| **[Own]** | Fully in your control — buildable anytime. |
| **[Fiddly]** | Feasible but messy — parsing, mobile, or hardware work. |
| **[External]** | Depends on third parties with **no official API** — research-grade, ToS caveats. |

Size = rough effort (S / M / L).

---

## The three loops (the spine)

The individual features get much more powerful when they close into loops:

1. **Inventory loop** — purchases (receipt scan / barcode / confirm) → **pantry state** →
   shopping list subtracts what you already have → "use it up" biases the roller → cooking
   decrements the pantry. Nail this and the app stops being a randomizer and starts
   *managing your kitchen*.
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
  rolls toward liked, away from disliked; recency via `last_cooked`.
- **Min/max cook interval** — **[Own]** M. Per-recipe **minimum gap** ("don't repeat within
  N weeks") and **maximum gap** ("cook at least every M weeks"), on top of the recency
  weighting. Default to friendly presets (e.g. *rarely / sometimes / often*), with an
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
  week toward recipes that *share* ingredients, to cut cost and waste. A genuinely
  interesting solver objective to write in Rust.
- _(suggested)_ **Prep-ahead / defrost awareness** — **[Own]** M. Flag recipes that need an
  overnight step (marinate / soak / defrost) and surface a "do this tonight" nudge the day
  before. Reuses time-to-make data.
- _(suggested)_ **Seasonality bias** — **[Own→External]** M. Prefer in-season produce
  (cheaper, fresher) via a season→ingredient map; a deeper version leans on scraped pricing
  (§4).
- _(suggested)_ **Exploration ↔ comfort dial** — **[Own]** S. A "surprise me ↔ safe" slider
  trading novelty against known favourites — leans into the *roulette* identity; pairs with
  ratings.
- **Constraint-conflict prompts** — **[Own]** M. When honouring a request collides with a
  rule — e.g. the only recipe with an ingredient you want to use up is still inside its **min
  interval**, or every candidate is filtered out by available time — surface a **warning with
  choices** instead of silently failing: cook it anyway, add a new recipe, or pick a
  different existing one.

### 2 · Recipes & data entry
- **Main protein** — **[Own]** S. Record each recipe's main protein as a first-class
  attribute (chicken / beef / tofu / …). The roller uses it to **rotate proteins across the
  week** (no chicken four nights running).
- **Healthiness rating** — **[Own]** S. A per-meal healthiness rating (simple scale for
  now); the roller can **cap or balance less-healthy meals per week**. *Future:* derive the
  rating from the recipe's **ingredients** rather than a manual per-meal score — ideally
  without making the user rate every ingredient by hand — via the **base ingredient library**
  (below).
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
  point is building the list from *your* version of the selected recipes.
- _(suggested)_ **Cooking notes over time** — **[Own]** S. A per-recipe running log of
  timestamped notes ("used less chili last time"). Separate from editing the recipe itself
  (users can also just adjust a recipe's ingredients) — this is the history, not the recipe.
- _(suggested)_ **Nutrition (macros / calories)** — **[Own→External]** M. Beyond the health
  *rating* above, per-recipe macros / calories derived from the **base ingredient library**.

### 3 · Pantry & inventory
- **Pantry tracking** — **[Own]** M. Track what you have based on confirmed purchases
  (user confirms they bought for a given recipe); the shopping list then subtracts pantry
  stock so you don't re-buy.
  - **Decrement on cooking**, driven by the recipe's ingredients.
  - **Always allow manual override** — the user can add an item to the list even when the
    app thinks it's still stocked.
  - **Correctable estimates** — the pantry count is an *estimate* (it assumes you cooked
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
  apps are already structured** — a cleaner data source than OCR *if* you can get at them
  (see §4).
- _(suggested)_ **Barcode scanning** — **[Fiddly]** M. Camera barcode + **Open Food Facts**
  (free, open product database with an API) for reliable per-item pantry adds. More accurate
  than OCR for individual items; complements it.

### 4 · Money & shopping integrations — *the hard bucket*
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
  - **Display:** e-ink is lovely for a *passive, always-on "this week"* panel (low power)
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
  planned meals (excludes them / triggers a re-roll of that slot). Requires users + auth and
  a `household_id` on the core tables. **Bake this into the schema early** — even if you run
  solo at first, retrofitting ownership onto existing data is painful.
- **Per-day cook assignment** — **[Own]** M. Assign who's cooking on which days ("Tuesday
  is Sam's night"); the roll respects it.
- **Per-user recipe affinity** — **[Own]** M. Track which recipes each member *likes to
  make* / is comfortable cooking. When a slot is assigned to a member, the roller prefers
  from that member's recipes.

> These, plus difficulty-by-day, leftovers, dietary constraints, ratings and vetoes, all
> become weighted **inputs to the roller** — which is where the interesting Rust logic lives.

### 7 · Shopping list
> The list is generated from the week's recipes; **sleep ingredients** (MVP, core loop)
> already lets you suppress items you don't need to buy yet.

- _(suggested)_ **Quantity aggregation + unit math** — **[Own]** L. Sum quantities for the
  same ingredient across the week's recipes and reconcile units. Density makes ml↔g
  **product-specific** (water 1:1, flour not), so two modes:
  - **Simple (default):** the user picks one unit per ingredient and enters *everything* —
    recipe amounts and the ingredient itself — in that unit, so no conversion is needed.
  - **Advanced:** ship a set of standard conversions, and let users add their own
    **per-product** conversions where the standard ones don't fit.
- _(suggested)_ **Customisable list display** — **[Own]** M. Let users define **their own
  sections and grouping** for the list — one person groups into *Longlife / Fresh / Meat*,
  another wants different buckets. Ingredient categories from the library feed this, but the
  grouping stays user-configurable.
- _(suggested)_ **Ad-hoc / non-recipe items** — **[Own]** S. Manually add items that aren't
  tied to any recipe (bin bags, foil) to the week's list.

### 8 · Insights & goals
- _(suggested)_ **Insights dashboard** — **[Own]** M. Trends that make the attributes pay
  off: protein mix, cuisine spread, healthiness over time, weekly spend, most / least cooked.
- **Goals & targets** — **[Own]** M. Let the user set **per-meal or per-week goals** — e.g.
  *max one Chinese dish per week*, *≥ 40 g protein per meal* — that constrain the roll. All
  **overridable in advanced settings when they spin the wheel**. (A roller input, surfaced
  here alongside the insights that track progress against it.)

---

## Suggested roadmap ordering

Front-loads value; each step is a clean learning milestone. Ordering is **author-first** —
built for personal use, so it leads with what the author values most.

0. **Core roller + shopping list** *(in progress)* — includes difficulty-by-day (+ override)
   and sleep-ingredients on the shopping list
1. **Household + auth foundations** — users + `household_id` ownership in the schema, so the
   collaboration features aren't a painful retrofit later
2. **Recipe import from URL** — removes the data-entry wall so the app is actually usable
3. **Leftovers + dietary constraints** — makes the roll genuinely useful
4. **Pantry + staples predictor + "use it up"** — the inventory loop (with manual override); starts saving real effort
5. **Collaboration: suggest / veto, cook assignment, per-user prefs** — the member-facing side, all feeding the roll
6. **Ratings / feedback** — the roll gets smarter each week
7. **Mobile wrap** — unlocks camera + offline; then receipt OCR / barcode for low-friction inventory
8. **Kitchen screen** — the embedded payoff
9. *(parallel research track, optional)* **base ingredient library + pricing + loyalty** —
   multi-currency, scraped ingredient/price data; spikes, not promises

---

## Decisions

- **Household, not single-user.** Several members; the main cook plans; others can suggest
  recipes and veto. **Per-day cook assignment** and **per-user recipe affinity** feed the
  roll (whose night it is → prefer recipes they like to make). → build users/auth + household
  ownership into the schema from the start (§6).
- **Difficulty by day, overridable.** Easier weeknights, harder weekends by default; the
  user can override per day / per plan.
- **Recipe sources: both** — hand-entered *and* imported from URL.
- **Pantry decrement: recipe-driven, with user override.** Cooking a planned recipe
  decrements its ingredients; the user can always add an item to the list even when the app
  thinks it's stocked, and can correct counts when they cooked unplanned things.
- **Pricing: manual entry for now.** Automatic pricing (§4) deferred — decide when you reach it.
- **Dinners first, me-centric.** The MVP rolls **dinners only** — this is built for the
  author's own use first, so the roadmap is prioritised by what the author values most, with
  breakfast / lunch and the broader add-ons sequenced after. Rolling other meals is an
  **option to add later**.

## Still open (deferred)

- **Kitchen-screen hardware + input method** — a *later* hardware decision; nothing needed
  now. (Which device displays it — e-ink panel vs small TFT/LCD vs a repurposed tablet — and
  how you interact — a few buttons, a rotary encoder, a touchscreen, or just your phone as
  the remote.)
