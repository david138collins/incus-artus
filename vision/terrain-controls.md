# World Compiler controls, and why each one exists

*(Called the Terrain Compiler until 2026-10-29.)*

For the rework on 2026-10-27 (`docs/worldgen.md`, "controls do what they say"). Loosely organised by
panel. For each control: what it does today, and **why it was asked for**, in David's words where they
exist (from `docs/worldgen.md` and the IDE conversations). Compiled by Counsel from the code
(`src/worldgen/compiler.rs`, `src/world/interpreter.rs`, `src/worldgen/additives.rs`).

**Worth noticing first:** several controls were never asked for by David. Claude added them to make the
first versions work. Those are marked *(Claude's, not David's)*. They're the likeliest to be mysterious,
because no one ever stated what they're for.

---

## Landmass (the compiler's whole-map work)

- **Sea level** (`water`): which of the bake's heights is the coastline; the land share shows.
  *Why:* the first compiler version (2026-10-24): sea at Z 128, mountains to Z 333 [David].
- **Soften coast** (`soften`, 64 blocks): blurs the coast and the heights and tilt beside it, so the
  bake's 32-block pixel staircase becomes a curve. *(Claude's, not David's)*: added with the Interpreter
  to stop one hump per pixel along coasts.
- **Soften mountains** (`soften_mountains`, 64 blocks): the same for the squeeze and push maps.
  *Why:* David's screenshot of rows of notches along range flanks, in step with the pixel grid (2026-10-26).
- **Foothills** (`foothills`, 2,500 blocks): a skirt of lower slopes blurred round each range; mountains
  are never lower than it. *Why:* added with grading and passes (2026-10-24) [David], for the same aim
  as grade: a less abrupt change from mountains to plains.

## World Interpreter (every block)

### The base land and sea
- **Deepest / Highlands** (Z): the deepest sea floor, and the highest land before mountains.
  *Why:* the first compiler version: "how deep the deepest ocean goes, how high highlands go".
- **Land curve / Sea curve** (splines): how land climbs from the coast to the bake's highest ground,
  and how the sea floor falls to the deepest. *Why:* David, 2026-10-24: "i need a curve graph for the
  land and sea curve… a spline generator… where i can add dots and flex the line, in order to create a
  more detailed continental plane. i figure i should try to get it as right as possible in the first
  step so that subsequent passes are simpler to manage."
- **Grade** (`grade`): uses each plate's tilt to steepen its high side and soften its low side (land
  and sea). *Why:* David, same message: "if the tilt map was used to gently grade the landmasses…
  so that the transition between mountains and plains is less abrupt. it'd tend to make smooth coasts
  on one side and steep ones on the other." (The tilt itself came from his picture of "a plate balancing
  on a point in its center, and being tipped slightly in a direction".)

### The Lift
- **Max lift, reach, curve, coast pad (width, falloff), points from (scattered / range middles),
  least / most crush, spacing, range cut, smallest range.** *Why:* David, 2026-10-26: use the mountain
  points "as points with elevation in the compiler, and elevate all exposed land from them using a
  spline graph with dots", "a tablecloth draped over a series of points floating in the area, with the
  edges of the cloth tapering and ending just before the coast lines". The coast pad exists so the Lift
  never moves the coast. Docs note it "may tie in later with reworking hills and ground detail".

### Mountains
- **Mountain curve** (spline): how tall mountains are for each amount of squeeze. *Why:* David,
  2026-10-24: "i can't seem to get the mountains to have a peak as much as be a plateau… a graphical
  curve box where i can drag the end of the line up and down, and flex it into a curve either way."
  (Its start and end sliders went because "these ones are always stuck at one side.")
- **Ridges** (0–2): how much the ridge pattern carves the mountains. *Why:* the first compiler: crushed
  rock gets ridges, lined up across the squeeze. Range doubled after `hope` (2026-10-25), where David's
  best results had every mountain slider pinned at its end.
- **Ridge spacing / ridge length / sharpness:** the ridges' size and crest shape. *Why:* first version;
  ranges re-centred on `hope` ("small, sharp, short ridges").
- **Ridge layers / ridge roughness / ridge alignment:** how many finer layers, how rough, and how
  readily ridges line up across the squeeze. *Why:* these were the ridge pattern's hidden numbers,
  made sliders after `hope` (2026-10-25). *(The numbers were Claude's; David asked for them exposed.)*
- **Steady ridges** (switch) and **ridge wander / wander size:** *Why:* the "giant fingerprint"
  (2026-10-25): ridges crushed into grooves where the squeeze bent. Steady is the fix, kept as an A/B
  switch; wander bends ridges so neighbours stop lining up.
- **Passes / pass spacing:** dips in the crest height along each ridge. *Why:* David, 2026-10-24: "take
  the ridgeline of the deform field mountains, and apply a vertical wobbly edge to its peak height, to
  create mountain passes and a variety along its length."
- **Rift depth:** how deep the most stretched rock sinks. *Why:* the first compiler (stretched rock sinks
  into rift valleys). David, 2026-10-24: "rift depth seemingly does little", which led to it being
  measured against the most stretched rock instead.

### Hills and ground detail
- **Hills** (height, Perlin field, mask) and **Detail** (height, Perlin field, mask): rolling hills and
  small bumps. *(Claude's, not David's)*: in the recipe since the first version. David asked
  (2026-10-24) to be told "where any noise function that i may not have explicity defined as such is
  living in the pipeline… so that i know whats going on."
  - *What David wanted noise to be for:* "your average perlin generated terrain is kind of lame. it
    just makes different kinds of rolls… it should be used as a final smoothing layer that adds some
    detail in between more deliberately evoked transformations to the terrain. we just have to avoid
    any obviously repeating patterns to the eye."
  - *What David finds them good for:* how far inland the coast runs and how irregular it is: rippled
    coasts full of shoals and islets, lowlands full of ponds, and continent-to-islands with a small move
    (2026-10-27).
  - *What he's said against them:* "sometimes they get interesting or good results but they really
    effect the entire map in unintended ways" (2026-10-26); and the distant terrain showed water
    where the ground was stone, which he suspected was hill and detail noise (2026-10-26).
  - *Already planned:* the Additives road step 3, "rework hill and ground detail in light of the
    features we can add in other ways now".
- **Masks** (for hills and detail): fade a layer by coast distance, height, mountains, foothills or
  squeeze, through a curve. *Origin not found in David's messages* (likely Claude's, to tame the
  noise). Related: a mask layer (radial, linear, ring, with a falloff) was wanted in the Graphics Kit.

### Surface blocks
- **Stone from / Gabbro from** (Z): bare stone and gabbro on the heights. *Why:* David, 2026-10-24:
  "apply gabbro to blocks that only mountain peaks reach, stone for anything of a moderate mountainous
  elevation, grass for the rest… so i can pick out peaks at a distance easier." Colour only; they
  don't shape the land.

## Additives (dots only; nothing drives the ground yet)

- **Grain** (grain reach, from squeeze): which way each range runs, and string or knot. *Why:* David,
  2026-10-26: mountain ranges "are generally in strings. in some cases they make bundles without a
  specific ridge direction though", so a detector that works on typical ridges "would freak out on
  less common mountain formations".
- **Grain drift** (spacing, least / most mountains, distance, dots, spread, lean, thin, way / toward):
  *Why:* David's picture (`For Claude/visionofparticlestreaming.png`): seeds along a range, dots
  leaving one side and carried on the way the range heads, scattered and thinning out.
- **Range cut, tip pull:** ranges and their tips. *Why:* David's carrot: its middle sits near the fat
  end, but more of it reaches out the thin end.
- **Coherence, knot scatter, spiral, spiral exponent:** pull dots into one stream; swirl them round
  the map's middle. *Why:* built on the Additives road, step 1 (2026-10-26), "throw dots around
  mountains until it routinely looks right… expect to try and scrap several methods". David picked
  the map's middle and distance travelled for the spiral.
- **Ridge streams** (spacing, distance, dots, bend, sides, least mountains; off by default): the first
  try, seeded on range spines. *Why kept:* David: "ridges were the wrong approach… it wasn't made for
  this"; kept switched off for later use.
- The aim of all of it: "a setting that routinely produces pleasant ranges of fantasy mountains (which
  basically means its taller than it is wide)" [David, 2026-10-26].

---

## Broad to fine: a draft order  [David's idea, 2026-10-27; order is counsel's draft]

David: order controls by how broad and singular their effects are, continental transformations at the
top, smaller and more locally variable ones further down, so tuning can work from top to bottom.

1. **The continents:** sea level · land and sea curves · deepest / highlands
2. **Whole plates:** grade
3. **Mountain regions:** mountain curve · foothills · the Lift
4. **Ranges:** ridges · ridge spacing / length · alignment, steady, wander · passes / pass spacing · rift depth
5. **Within a range:** sharpness · ridge layers / roughness
6. **The ground between:** hills (1,100-block features in `current`) · their mask
7. **The ground you stand on:** detail (~125-block features) · its mask
- **Off the list (fixes, not shaping):** soften coast · soften mountains. Maybe an "advanced" corner.
- **Off the list (colour, not shape):** stone from · gabbro from.

**The test that the order works:** a control low on the list should rarely undo one above it. Today
hills break that: in `current`, hills are ±22.7 blocks, and wherever the land curve keeps lowlands close
to sea level, that's enough to flip whole regions between land and sea. So hills (row 6) are deciding the
continents (row 1). That's the "continent to islands with a small move". The land share shown on the
panel makes this measurable: moving a lower control shouldn't move it much.

Where a control acts at two scales at once (as hills do: rolling ground *and* the coastline), the order
can't place it, and that's the sign it should become two controls.

## David's standing rules for controls (from the same conversations)

- "i like tightening the slider ranges so that they encompass more of the useful area more finely. we
  should keep that as a common design practice at all steps."
- "part of writing better software would be avoiding the necessity of extremes so that behaviors can be
  more closely refined."
- Judge across seeds, not one (Shift+Space), and keep things modular, because "every time we move one
  step forward, we need to assess every single previous level of the software again."
