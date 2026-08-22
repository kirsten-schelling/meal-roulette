# Meal Roulette — Roadmap & Feature Ideas

> Working doc. Captures the vision, what's built, and where it could go.
> **Provenance:** unmarked features are the original idea list; items tagged _(suggested)_
> are proposals added on top — react to / cut / keep as you like.

---

## The core loop (MVP — building now)

Roll a week of dinners from recipes in Postgres, weighted so **weeknight meals are
quick/easy** and **weekend meals can run longer/ambitious**, then generate a **shopping
list** for the week.

Already scaffolded: SvelteKit + yarn frontend · Rust/Axum + SQLx backend (Cargo workspace
with a shared `domain` crate) · Postgres. Stretch surface: ESP32 kitchen screen.

Core sub-features:
- Recipe attributes: difficulty, time-to-make, servings/yield, protein, cuisine, tags.
- **Difficulty by day**: defaults to easier dishes on weeknights, harder on weekends —
  but **user-overridable** per day / per plan.
- Shopping list = union of ingredients across the week's recipes.
- _(suggested)_ Seeded, reproducible rolls; **pin** meals you like and **re-roll a single
  day** — small feature, but it's what makes this feel like a product instead of a script.

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
- _(suggested)_ **Dietary & preference constraints** — **[Own]** M. Veg / allergy /
  "no fish for partner" / cuisine rotation, all fed into the solver.
- _(suggested)_ **Ratings & feedback** — **[Own]** M. Rate after cooking; weight future
  rolls toward liked, away from disliked; recency via `last_cooked`.
- _(suggested)_ **Busy-day / calendar awareness** — **[Own→Fiddly]** M. "Eating out
  Friday" → roll 6 days not 7; optional Google Calendar push of the plan.
- _(suggested)_ **Budget as a constraint** — **[Own→External]** M. Keep the week under $X.
  Works today with manual per-recipe cost estimates; the automatic version needs pricing
  (see §4).
- _(suggested)_ **Batch-cook / ingredient-overlap optimisation** — **[Own]** L. Bias the
  week toward recipes that *share* ingredients, to cut cost and waste. A genuinely
  interesting solver objective to write in Rust.

### 2 · Recipes & data entry
- _(suggested, high value)_ **Recipe import from URL** — **[Own]** M. Most recipe sites
  embed `schema.org/Recipe` as JSON-LD; parse title / ingredients / steps / time / yield
  straight out of the page. This kills the single biggest friction of any recipe app —
  hand-entering recipes — so I'd put it very early.
- _(suggested)_ **Cook mode** — **[Own]** S. Step-by-step view, scale servings,
  keep-screen-awake. Shared by the mobile app and the kitchen screen.

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
- **Staples predictor** — **[Own]** M. Track non-recipe items (milk, paper towel, etc.)
  and typical usage cadence; predict run-out and auto-suggest adding to the list.
- _(suggested)_ **"Use it up" bias** — **[Own]** S. Bias the roll toward recipes that use
  soon-to-expire pantry items. Closes the inventory loop cheaply.
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
- **Mobile app** — **[Fiddly]** M. Capacitor wrap of the SvelteKit app. Unlocks the camera
  (OCR / barcode), share-sheet recipe import, push notifications, and an offline shopping
  list for in-store with bad signal.
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

---

## Suggested roadmap ordering

Front-loads value; each step is a clean learning milestone.

0. **Core roller + shopping list** *(in progress)* — includes difficulty-by-day (+ override)
1. **Household + auth foundations** — users + `household_id` ownership in the schema, so the
   collaboration features aren't a painful retrofit later
2. **Recipe import from URL** — removes the data-entry wall so the app is actually usable
3. **Leftovers + dietary constraints** — makes the roll genuinely useful
4. **Pantry + staples predictor + "use it up"** — the inventory loop (with manual override); starts saving real effort
5. **Collaboration: suggest / veto, cook assignment, per-user prefs** — the member-facing side, all feeding the roll
6. **Ratings / feedback** — the roll gets smarter each week
7. **Mobile wrap** — unlocks camera + offline; then receipt OCR / barcode for low-friction inventory
8. **Kitchen screen** — the embedded payoff
9. *(parallel research track, optional)* **pricing + loyalty** — spikes, not promises

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

## Still open (deferred)

- **Kitchen-screen hardware + input method** — a *later* hardware decision; nothing needed
  now. (Which device displays it — e-ink panel vs small TFT/LCD vs a repurposed tablet — and
  how you interact — a few buttons, a rotary encoder, a touchscreen, or just your phone as
  the remote.)
