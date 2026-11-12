# The FX Lab (was the Toy Factory)  [PROPOSED by counsel, 2026-10-26]

A proposal for growing the Graphics Kit into the place where the game's small, cheap, reactive effects
are made: fluid in the spheres, glass cracking, lightning, blood spray, sparks, drips. Nothing here is
decided. David picks what to keep, and where it lives in the Editor Suite (`docs/worldgen.md` says to ask
David before naming or placing any new tab or panel).

---

## The shape: three devices, one engine  [David's direction, 2026-10-26]

**Retrofit the Graphics Kit, don't build a second container beside it.** Separate tools would mean more
containers for things that must work closely together, and copies of code doing the same job. The kit still
serves its first purposes (grading PNGs, rendering graded noise for textures, feeding World Gen).

Think of it as a **modular synthesizer**: one engine, with devices you move between in the same window.

**Working names (David, 2026-10-26): Graphics Kit → Sequencer → FX Lab.**

| Device | Synth equivalent | What it does | What it makes |
|---|---|---|---|
| **Graphics Kit** (exists) | the sound sources and filters | The static processes, set by hand while working on a still: noise, layers, grading, **filters, HSL**, distortions. Its settings are the knobs. | Stills: graded images, textures |
| **Sequencer** | the modulators | Its own viewport, and the movement: **oscillators, envelopes, keyframes, and multi-flex knobs** (one knob driving several settings in set ratios, and itself drivable by an oscillator or envelope; synths call these *macros*). Patches into any Graphics Kit setting. Records the result. | Plain animations: loops, sprite sheets, GIFs |
| **FX Lab** | the self-contained instruments | Discrete objects with behaviours: lightning, particles, pixel sims, the fluid spheres. Its own viewport, **2D or 3D**, for positioning objects and particle generator points. | Live objects for the game, and recorded effects |

**How they pass things to each other ("bounce"):**
- Sequencer → Graphics Kit: a single frame, as a still.
- FX Lab → Sequencer: an effect set up and recorded as an animation (a spell icon's flash).
- Sequencer → FX Lab: an animation used inside an effect.
- Graphics Kit → both: its settings are what the Sequencer moves and what toys are built from.

David doesn't expect to use them at the same time; each is its own page in the same window.

**Notes for building it (counsel):**
- **Every Graphics Kit setting needs a stable address** ("this layer → HSL → hue"), so the Sequencer can
  aim at it. Address layers by an ID, not their position in the stack, so reordering layers doesn't
  break a patch.
- **Base value plus movement:** when the Sequencer moves a knob, the Graphics Kit should still show the
  knob's own setting, with the movement drawn around it (the way synths show a modulation ring), so
  going back to work on a still never finds settings mysteriously changed.
- **"Scripting" without a scripting language, at first.** Toy behaviours (emit, branch, crack, flow,
  react) are modules Claude writes in Rust; the FX Lab configures them and wires them together, and
  that wiring *is* the script. Embedding a real scripting language is a big project of its own. Only do it
  if wiring stops being enough.
- **The model is the StarCraft editor's triggers** [David 2026-10-30], for the FX Lab and the zone
  designer alike: "an intimidating page for designing triggers", but everything it can do combines
  flexibly, and if you can find or make a script, you can extend its behaviour. Counsel's reading of what
  made it work:
  - a trigger is **conditions, then actions**, each picked from a list (StarCraft II added the
    **event** that wakes it up);
  - each line **reads as a sentence with blanks to fill in** ("Player 1 brings at least 1 Marine to
    Location"), which is what makes the intimidating page legible;
  - **"find a script":** a set of triggers saves and loads, so other people's can be imported;
  - **"make a script":** StarCraft II let you save your own actions and functions as new pieces, used
    like the built-in ones.
  For us: hooks are the events and conditions, behaviours are the actions, the Rust modules are the
  list, and a saved script is a piece too.
- **2D first.** The demo's toys are UI (the spheres, icons). The 3D viewport (lightning between two points
  in the world, dust from a falling tree) can come after.

**The test for where something belongs:** what does it ship as?
- a still → Graphics Kit;
- movement over time, recorded → the animation page;
- a live object that runs in the game and reacts → the effects processor.

**Live objects have inputs, like a synth's control jacks.** The game patches its own numbers into them, and
in the editor you fake those numbers to tune. Example (David): an equipped fireball's icon in the low-res
UI runs a flash of light through itself, and the flash also shows the spell's cooldown. The icon is an
effect with one input, `cooldown` (0 to 1). It costs almost nothing to run.

**Naming:** "toys" in David's vocabulary are the reactive things in the game, which are what the effects
processor makes. **2026-10-30, David:** "toy factory is whimsical but we'll probably go with FX Lab." The
effects processor is the **FX Lab** (was Toy Factory); this file keeps its old name so links to it hold.

---

## The one idea that matters most

**A toy in the game should be the same thing as the toy in the factory, not a picture of it.**

There are two ways to bring an effect into the game, and the factory should offer both:

| | Baked | Live |
|---|---|---|
| What goes into the game | Frames: a palette-indexed sprite sheet, plus a small file saying fps, loop and pivot | A **recipe**: the toy's settings as a data file, which the game runs itself |
| Good for | Fixed animations: a torch flame, a spell icon, a title-screen shimmer | Anything that **reacts**: the spheres sloshing, the glass cracking where you were hit, lightning between two real points |
| Cost | Cheap to draw, big files | Tiny files, a little work every frame |

Live recipes are how presets already work in the suite, and how the terrain lab shares the World
Interpreter with the game. Same principle: **tuning in the factory is tuning in the game**, with no
export step to forget.

---

## What it is, in David's words (2026-10-26)

**Not a drawing program: Aseprite is perfect for drawing, and assets aren't made here.** But it will
have nearly all the *peripheral* tools a drawing program has, because those are the tools David wants to
automate. "Many times I have sat there moving a slider back and forth in an image program thinking,
that'd be cool animated just like that." So the factory is **an image editor's adjustments, where every
slider can move by itself.**

The palette grading and pixelizing settings already in the Graphics Kit are something David couldn't
find in any commercial software: in Photoshop, GIMP or their plugins, grading an image to a palette with
control over how it's done is a tremendous ordeal. That's a real asset (see `vision/marketing.md`).

## What it would need

### 0. The image editor's peripheral tools, all automatable

Every one of these is a layer or adjustment in the stack, and every one of its sliders can be driven by
time (section 1):

- **Tone:** levels, curves, brightness and contrast, threshold, posterize, invert, gradient map.
- **Colour:** hue, saturation and lightness (the HSL filter exists), colour balance, palette grading and
  pixelizing (exist), palette cycling (the classic 256-colour trick of rotating a run of palette entries:
  water, fire and glow animated with no new pixels).
- **Filters:** blur, sharpen, glow, emboss and bevel, edge detect, outline, drop shadow, noise and grain.
- **Distort:** offset and scroll, rotate, scale, shear, ripple, twirl, displace by another layer
  (the warp modes exist), mirror and tile.
- **Masks and selections:** a layer's alpha or brightness as a mask, feathering, grow and shrink.
- **Mix:** the mix modes and opacity (exist).

### 1. Time  (unlocks everything else)

The kit's tools are all still pictures today. Give every slider a way to move:
- **Oscillators** that can drive any slider: sine, saw, square, smooth noise, random steps; rate, depth, offset.
- **Envelopes** (attack, decay, sustain, release) for one-shot effects. The game already has these
  (`src/kinematics/envelope.rs`, used for sprinting), so this is reuse.
- **Keyframes** for hand-timed moves.
- **The loop recorder** (an idea David already noted): record N frames and check that the loop is seamless.

### 2. Generators

- **Bolts:** lightning from point A to point B. Jaggedness, branch chance, how fast branches thin out,
  glow, flicker. The kit already has midpoint displacement (plate lines), which is the classic bolt maths.
- **Cracks:** growing from an impact point, branching, stopping at a mask's edge (the glass of a sphere).
  Worley edges already make static cracks. This adds cracks that *grow*.
- **Particles:** an emitter with rate, speed, spread, gravity, drag and lifetime, coloured along a
  palette ramp over its life. Sparks, smoke, blood spray, dust, falling leaves.
- **Pixel sims** ("falling sand"): elements like liquid, powder, gas, fire and smoke, each a rule about how
  it moves and what it turns into when it touches another (potion + blood → glowing → more blood; lightning
  turns a pixel into 2 fire and 2 smoke). The elements live in data files, like blocks. **This is the
  spheres' engine:** build it once as a toy, and the HUD runs the same code.
- **Flow:** push particles and pixels along a field (the kit's warp modes: swirl, push, flow) for smoke curls and eddies.

### 3. Shapes and masks

Circles, rings, radial gradients, polygons, and image masks: the sphere's glass, a window's border, a
blade's arc. Sims and particles stay inside a mask or bounce off it.

### 4. Drawing stays in Aseprite

No brushes here. Instead, **open Aseprite files directly** (layers and frames), so a hand-drawn sphere,
bottle or border drops straight in as a layer or a mask, ready for the adjustments above to move.

### 5. Palette discipline

Every output snaps to the game's palette, as indexed colour. Colour over time is a **ramp**: a run of
palette entries, like Grafx2's spread. No programmatic dithering (`docs/worldgen.md`, decided 2026-10-23).

### 6. Poke it

Toys have to be tuned while they react, so the factory needs fake game events: click to strike the
sphere, drag to slosh it, a button for "lightning hit", "healing potion", "fast regeneration". Show it
on a **640×480 stage**, at the game's real scale and whole-number upscaling, over a screenshot of the HUD
if wanted.

### 7. Bounce

- **To the game, live:** save the recipe into the game's data (a `toys/` folder beside `blocks/`).
- **To the game, baked:** a palette-indexed PNG sprite sheet plus its small description file.
- **To the world:** **GIF and video export.** Every toy is a marketing clip too (`vision/marketing.md`).

### 8. A cost meter

Each toy shows what it costs per frame (the suite already has a processing meter), with a budget, so a
screen full of toys stays cheap. Pixel sims are capped by their mask's size; 64-pixel spheres are tiny.

---

## A suggested order

Build each piece when something in the game needs it, not all at once. That keeps the factory from
pulling time away from the dwarf (`vision/watch-list.md`, item 1):

1. **Time:** oscillators, envelopes and the loop recorder, with PNG-sequence and GIF export. Small, and
   it brings everything the kit already has to life. Then the peripheral tools (section 0) one at a
   time, each animatable from the day it's added.
2. **Particles,** with poking. Wanted the moment there's a hit, a dig or a falling tree.
3. **Cracks and the glass mask,** for the sphere HUD.
4. **Pixel sims,** for the sphere's blood: the demo's HUD (`watch-list.md` item 3, demo scope).
5. **Live recipes in the game,** once the first toy is good enough to ship.
6. **Bolts,** with magic, after the demo.
