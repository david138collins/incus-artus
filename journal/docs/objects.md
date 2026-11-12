# Objects: real 3D things in the world (Builder, 2026-11-01, for David and Counsel to choose)

David: "can you send to council an assessment for how to implement what would probably be a tab in
the dock for things like this, and cups, and plates, or loafs of bread, and so on. real 3d objects.
after all we need torches to put them through a cave and see how they look. the braziere would be a
3d object, with a 2d sprite animation located in the cup? thats fine for now and something we may
want to be able to do. so it'd be a complex item, with an attached decoration."

This is an assessment, in the shape of `docs/caves.md` and `docs/quartile.md`. **Nothing here is
built.** The first object is David's brazier (`Art/3d braziere/`, Blockbench), which he's remaking at
1.5 blocks tall with an animated flame (Aseprite, 16 × 16, a few frames).

**The short version:**
- **An object is a model plus attachments.** The model is a glTF from Blockbench. Attachments hang
  on named points in it: a flame (an animated sprite), a light, and later particles. A torch is a
  stick with a flame and a light; a cup is just a model.
- **One new tab in the Registrar** for them, beside the blocks. Its name and place are David's call.
- **Three stops:** see it and place it; its flame; its light (the coloured "placed lights" from
  `docs/caves.md` section 11). Then torches go through a cave.

---

## 1. What exists today

- **Only blocks are registered:** the Registrar has its library (left), the dock (middle) and the
  palettes (right), with no tabs. Each block is a data file (`assets/data/incus/blocks/*.toml`).
- **Bevy reads glTF as it is** (Blockbench's export): models, their textures, and named points.
  Nothing in the game uses it yet.
- **David's brazier** (`braziere.gltf`):
  - It's self-contained, with its two textures packed inside: sharp pixels, cut-out transparency,
    both sides drawn.
  - It's 9 thin parts (a pole, a cup of 4 slats, 4 feet), 3.4 blocks tall as exported (Blockbench
    counts 16 pixels to a block). David means 1.5.
- **Nothing but the pawn lives in the world as a thing:** no items, no inventory, no objects saved in
  chunks.
- **On file already:**
  - quartile: "an octant item would be a cup or a candle", and items placed on quarters and centres;
  - light: placed lights, coloured, their own channel (`docs/caves.md` section 11);
  - the brazier notes there: glTF, a decoration, frames for the flame;
  - Registrar tabs by kind: blocks, items, decorations, Blueprints (`docs/decisions.md`);
  - the clutter or items question (watch list item 19): whether things can be picked up is open.

## 2. What an object is: its data file

`assets/data/incus/objects/<name>.toml`, one per object, as blocks have:

| Setting | What it is |
|---|---|
| `name` | What it's called ("Brazier") |
| `model` | The glTF, in `assets/models/` |
| `height` | How tall it stands, in blocks: the model is scaled to fit, so a model at any size works (David's brazier: 1.5) |
| `sits` | How it's placed: on a floor, a wall or a ceiling, in the middle of a cell or on the octant grid (a cup on a table at a quarter) |
| `solid` | Whether bodies bump into it (a brazier yes, a cup no), as a box round it |
| `[[attach]]` | Things hung on it, each at a named point in the model (a Blockbench **locator**, e.g. `flame`) |

**An attachment** is one of:
- **a sprite:** a strip of frames (one PNG, frames side by side), how many frames a second, its size,
  and that it turns to face you (as Doom's flames do). Drawn at full brightness: it makes its own
  light.
- **a light:** its colour, brightness and reach, and a flicker (brightness wavering by time).
- later, **particles** (the FX Lab), and **sound**.

So a brazier is a model, a flame sprite at `flame`, and an orange light there. A torch is the same
with a stick. Glow lichen could be an object with only a light and a flat model. A loaf, a plate or a
cup is a model alone.

## 3. In the world

- **Placed objects are saved with their chunk:** a list of each one's name, its place (in 16ths of a
  block, so they can sit on the octant grid) and which way it faces. This is chunk format 4: format 3
  still loads, with no objects.
  - Only blueprints and players place them (David: "only blueprint stuff and player stuff"). World
    creation places them only through blueprints.
- **Drawn as models:**
  - Each one is shown when its chunk comes in and taken away when it's dropped.
  - Many of the same object (a city's cups) are drawn in one go by Bevy (instancing), which matters
    later, not for the first few.
- **Lit like the blocks round them:**
  - An object takes the light of the cell it's in (daylight now, placed lights after stop 3), so a
    cup in a dark cave is dark.
  - Its own sprites are full bright.
  - The model's look is flat, as the blocks' is: no Bevy realistic lighting.
- **Placing:** from the hotbar, as blocks are: right click puts it on the face aimed at, as its `sits`
  allows. Left click takes it away.
- **Picking up and holding** (items, an inventory) is later and waits on the clutter or items question.
  The format leaves room: an object is the same thing whether it's on a table or in a hand.
- **Far away:** small objects aren't drawn past the near chunks. Lights that should show for miles (a
  lair's fire at a cave mouth, `docs/caves.md`) come with the far view's points of light, later.

## 4. The Registrar's tab

A tab beside the blocks for objects (name and place: David's call, below):
- **The library:** each object, with a small picture (the isometric camera that's roughed in for
  blueprints, `docs/decisions.md`), grouped by tags as blocks are.
- **The dock:**
  - the object turning slowly in a lit preview;
  - its settings (height, how it sits, solid);
  - its attachments, each picked from the model's named points, with its strip or its light colour,
    shown live in the preview.
- **"Load a model…":** a glTF from `Art/` (or dropped on the window), copied into `assets/models/`,
  with its data file made.
- **Its textures and the palette:** a glTF carries its textures inside it, so the master palette
  doesn't see them yet. Either the Registrar reads them out of the glTF, or models are exported with
  their textures as separate PNGs (Blockbench can do both). That's for later, not stop 1.

## 5. The stops

| Stop | What David sees | About (lines) |
|---|---|---|
| 1. See it, place it | Object files; the brazier loaded and scaled to 1.5 blocks; the new tab listing it with a turning preview; placing it from the hotbar on a block; saved with chunks (format 4); lit by the cell's daylight | 600 |
| 2. Its flame | Sprite attachments at locators: the strip, frames a second, facing you, full bright | 200 |
| 3. Its light | Placed lights (`docs/caves.md` section 11): coloured, spread a short way through air, a flicker; stored in a third word of the vertex; then a torch, the second object, and a walk through a cave with them | 500 |
| Tests throughout | | 250 |
| **Total** | | **about 1,550** |

**After:** picking up and holding (items), placing on the octant grid and on walls, instancing for many,
far lights, and David's own objects (cups, plates, bread).

## 6. Risks

- **Two ways of lighting that must agree:** blocks keep their light in their faces, objects would read
  it from their cell. A brazier on a cave floor must look as dark or as lit as the floor.
- **Placed lights are the biggest piece** (stop 3): a second channel, colour, and making sections again
  when a light is placed or taken. That's the same work torches need, so it's not extra.
- **Blockbench's axes:** its up is Y, the game's is Z (the game already turns Bevy's axes into its own).
  The first model tests it.
- **Many objects:** thousands of separate models (a city's clutter) cost far more than blocks. Instancing
  and leaving small things out at a distance answer it, measured with a fill like the caves' one.
- **Saved worlds:** chunk format 4. Format 3 keeps loading.

## 7. Questions for David

1. **The tab:** one tab for every modelled thing (say **Objects**) for now, or the tabs on file (items,
   decorations) from the start? And its name and place: yours.
2. **The brazier:** solid, so you bump into it? Centred in a cell, or on the octant grid?
3. **The flame:** how many frames a second (Doom's are about 8)? And does it sit in the cup at the size
   of the cup, or bigger?
4. **Torches:** on the floor first, or on walls (which needs "sits on a wall" in stop 1)?

## 8. Stop 1, see it and place it: [BUILT 2026-11-01]

David's answers through Counsel: "youre right, these are all the same thing, swords, armor, brazieres,
torches, bread. blocks are blocks, objects are objects, and blueprints are made of both". The tab is
**Objects**. The brazier is solid, centred in its cell, and stands on the floor. Plants are objects
too ("objects can be 2D, like grass, or 3d, like corn or brazieres or beds"), and they may change over
time.

- **Object files** (`data/objects.rs`): `assets/data/<pack>/objects/<name>.toml`, holding `name`,
  `model` (a glTF in `assets/models/`), `height`, `solid`, `sits` (floor; wall and ceiling later) and
  `tags`. They're reloaded as they change, like blocks.
- **Placed objects are saved with their chunk** (chunk format 4; format 3 still loads). Each keeps
  its name, its foot (16ths of a block), which way it faces, and, kept for things that change with
  time, its stage and since when (the tick it was placed). One the game no longer has is kept, and
  not drawn.
- **Edits:** `PlaceObject` and `BreakObject`, through the edit system as blocks go.
  - An object stands on the floor of the cell in front of the face aimed at, in its middle. It's
    refused if a cell it would stand in is taken (it's 1.5 tall, so it stands in two) or if there's
    nothing solid under it.
  - A block can't be placed into a cell an object stands in.
- **Drawn** (`render/objects.rs`):
  - The model's meshes are drawn one by one, each with its texture, flat (unlit, no realistic
    lighting).
  - It's scaled to its height and stands with its feet's middle on its spot.
  - It's lit by its cell's daylight (the same walk as the blocks', `sky_light.rs`), eased as the
    shader eases it, and lit again when the sections round it are made again.
- **In play:**
  - An object in the hotbar (the Player page's slot picker lists objects too) shows where it would
    stand in magenta. Right click places it.
  - The object under the crosshair (nearer than the block) is outlined in white, and left click
    takes it away.
  - Solid objects stop bodies: their cell, up to their height.
- **The Registrar's tabs:** Blocks, **Objects**, and Blueprints (greyed, later).
  - The Objects tab lists them. Its dock has the name, the model, the height, solid, how it sits and
    the tags, saved to the file. Attachments are greyed until stops 2 and 3.
  - "Load a model…" picks a glTF from `Art/`, copies it into `assets/models/` and makes its file. It
    warns if the textures aren't packed inside.
- **David's brazier** is registered for trying (`incus:braziere`, 1.5 tall, solid), from his first
  export. It's his to replace with the remade one.
- **Not yet:**
  - the turning preview in the dock;
  - the test lantern lighting objects (it's in the terrain shader only);
  - placing on walls;
  - facing other than north.
- **Tests:** the object file read and written back; an object saved with its chunk (and one the game
  no longer has, kept); placing, refusals (taken, nothing under it, a block into its upper cell) and
  taking it away.

## 9. After trying stop 1: sizes and snapping (David, 2026-11-01) [stop 1b BUILT]

David, after placing the brazier: "we would need a simple preview of the object. however, i think an
isometric view would be preferred, so we can wait for that until we build the blueprints section. the
brazier places. i would also need settings for its dimensions, and size of object. an object larger
than 16x16 shouldn't take up more than it's model in collision, but it should count as a 'full block
object', meaning it snaps around like the regular blocks do. blocks that are thinner than 16x16 can
sit on octants, or the center, working like the other blocks. since quarter blocks are 16 wide, you
wouldn't be able to snap an oct sized object to the center against the wall. but you could half a
block forward."

- **The preview waits** for the blueprints section's isometric view.
- **Builder's proposal (stop 1b, before the flame):**
  - **Its size:** the dock shows the model's own size in pixels (16 to a block), width × depth ×
    height. One size setting scales it evenly (stretching would make its pixels uneven), with "as
    made" (Blockbench's 16 to a block, the blocks' own pixel size) as the default.
  - **Bumping:** a solid object stops bodies at the box round its model, not its whole cells.
  - **Two kinds by footprint:**
    - **Wider than half a block** (more than 8 pixels either way): a full-block object. It snaps to
      cells as blocks do, centred, and claims every cell its box reaches, so blocks can't go into it.
    - **Half a block or less both ways** (8 × 8 or smaller): it sits on the octant grid, against a
      wall or half a block forward, or in the middle of a cell, as blocks do (Shift for the octant
      grid). It claims only its octants.
  - **Not in it:** turning (facing), so a bed's long side always points one way for now.

David's answers: "yes, right. i forgot octants are 8 pixels. that is correct. basically we just go by
the models size, or model volume i guess. it can still stick a little over probably and just clip if i
made a larger model? and if it's bigger than that, it can't go on an octant. it needs 4 octants just
like a gabbro block. and, as long as its set by the model size, is hould have no problem making a
64x32x32 bed model and placing it. rotation might as well be B. i think helpers on screen that can be
dimmed will probably stay in some form in the end game, i just think it's good to have." And: "the
collision still needs to be from the models geometry, not its volume".

### Stop 1b, built

- **Its shape, read from its model** (`data/objects.rs`, `read_shape`, with the `gltf` crate Bevy
  already uses):
  - Every mesh in the model, placed as its nodes place it. Each of Blockbench's cubes is the box round
    its own points, turned as its node is.
  - Filled in pixel by pixel (16 to a block): a pixel is filled where its middle is inside a part. A
    part thinner than a pixel (a flat plane) still fills one. The pixels are then joined into boxes.
  - **Its size** is the model's in pixels, across, along and up.
  - It's read again when the object file or its model changes.
- **Its size settings** (the Objects tab):
  - **"as made"**: a glTF's unit is a block, Blockbench's 16 pixels, the blocks' own pixel size.
    It's the default for a model brought in.
  - Or **a height**: scaled evenly to it. The brazier's file still says 1.5.
  - **"may stick over"** (`stick_over`, pixels, default 2): how far it may stick out of its room and
    just clip.
  - The tab shows its size in pixels and what that makes it.
- **Small or full-block, by its size** (`world/objects.rs`):
  - **Small:** 8 pixels across or less both ways, plus what it may stick over. It goes in the middle
    of a cell (holding the four octants round the middle), or with Shift on one octant: against a
    wall, or half a block on.
  - **Full-block:** anything wider. It snaps as blocks do: centred in the cell in front of the face.
    With Shift, it goes on the octant grid. It holds whole blocks across (a bed 64 × 32 holds 4 × 2,
    reaching out of the face aimed at, never into it).
  - Either way it holds octants up its height. Blocks, bars and octants can't go into an object's
    octants, and an object can't go where they are.
  - It needs something solid under at least one of its octants.
- **Bumping into its model's own shape:** bodies stop flush against its boxes, not its room
  (`player/collision.rs` takes boxes as well as blocks). You can stand against the brazier's pole,
  and step up onto a low one as onto a bar. The crosshair meets its boxes too, so it passes through
  the gaps.
- **Turning:** an object in hand faces you, and **B** turns it a quarter at a time. What's drawn,
  bumped into and held all turn with it.
- **Helpers:**
  - The magenta box shows the room it would hold, with an arrow out of its front (the model's north
    in Blockbench).
  - **"how bright the outlines and placing helpers are"** (the Camera page) dims them all, the block
    outline too.
- **David's brazier as it is:**
  - At 1.5 tall it's 6 × 6 × 24 pixels, so it's **small**.
  - As made, it would be 13 × 13 × 55 (3.4 blocks tall).
- **Not yet:**
  - standing on another object (a cup on a table);
  - the isometric preview (with the blueprints);
  - turning a shifted full-block object toward the crosshair's side.
- **Tests:**
  - shapes read from a glTF (as made, scaled, a pole under a plate, a flat plane);
  - small and full-block rooms (a cup, a brazier, a bed turned);
  - snapping (the middle, an octant, half a block on, a bed on the floor and against a wall);
  - turning;
  - collision with boxes (stopped flush, walking past, stepping up onto a low one);
  - cups on two octants of one cell, and a block refused from an octant a cup holds.

## 10. Stop 2, the flame (David, 2026-11-01) [BUILT, with a stand-in glow]

David's flame: `assets/new/green_flame_animation.png`, 112 × 24, 7 frames of 16 × 24, with transparency.
- **The strip marks its own frames** (David: "i put a magenta dot at the corner of each 16 pixel cell,
  i thought maybe it could hook that so mapping them in aseprite would remain a simple png"):
  - The marker colour is whatever colour the image's bottom-left pixel is (his: 254, 0, 191).
  - Each pixel of that colour along the bottom row starts a frame, which runs to the next one (or to
    the edge), so frames could be different widths.
  - The markers are erased when the strip is loaded.
  - With no markers, frames are square, as tall as the strip.
- **Its timing:** milliseconds per frame, one number for now ("probably just set by ms for now").
  Aseprite's sheet data (a time for each frame) is for later.
- **Set in the Objects tab's attachments section** (David's "loader window"): the strip, what was
  found ("7 frames, 16 × 24"), the milliseconds, and a small preview playing.
- **Where it sits:**
  - which point of the frame goes on the spot (David: "a drop down for bottom, bottom center, top
    center"): bottom centre by default, for a flame;
  - the spot itself: a Blockbench locator in the model (listed in a dropdown, and it turns and
    scales with the model), or an offset from the object's feet in pixels, which works with no
    locator.
- **Its size:** one strip pixel is one block pixel by default, with a scale slider and **½ and 2×
  buttons** (David: "a 1/2 / 2x button could be useful").

**Tried (David, 2026-11-01):** "looks good... everything works like i asked it to, i can even jump on
top of them. they place and move correctly." Seen: the first brazier placed takes a second to appear
(its model loads only when it's first needed). Loading every object's model when the game starts
would answer it.

### Stop 2, built

David: "go ahead, i'd love to see a glowing braziere in a cave before bed".

- **The object file** has `[sprite]` (`file`, `frame_ms`, `anchor`, `at`, `offset`, `scale`) and
  `[light]` (`colour`, `reach`, `brightness`). Strips live in `assets/textures/sprites/`, and are
  read again when they change.
- **The flame** (`render/objects.rs`):
  - It's drawn on its spot: the named point, or on top of the model in the middle, moved by its
    offset, turned as the object faces.
  - It's full bright and cut out, and turns about the upright to face you.
  - It plays its frames at its milliseconds. Each one starts on a frame of its own (from its place),
    so a row of braziers doesn't flicker in step.
- **The glow (a stand-in for stop 3):**
  - The nearest 8 glows to the camera go to the terrain shader. Each is in its colour and fades with
    distance (squared) up to its reach, added to the daylight.
  - **It passes through walls.** Light that spreads through the air and stops at them is stop 3,
    which this makes way for.
  - Objects are lit by the glows round them too, their own included, in 16 steps of each colour.
- **The model's named points** (Blockbench's locators, and its groups) are read with its shape and
  listed for the sprite's spot.
- **Every object's model is loaded when the game starts.** The first brazier placed shows at once.
- **The Objects tab's attachments section:**
  - **the sprite:** the strip and what was found in it ("7 frames, 16 × 24 pixels"), the
    milliseconds (and frames a second), the anchor and the spot, the offset, and the size with ½
    and 2×;
  - **the preview:** the strip playing, 4 times its size, with the anchor point as a magenta dot;
  - **the light:** its colour, reach and brightness.
- **David's brazier is set up to try** (his files, uncommitted):
  - the flame is copied to `assets/textures/sprites/`;
  - in `braziere.toml`: the flame at half size, 3 pixels down into the cup, at 110 ms;
  - a green light, `[80, 255, 60]`, reaching 8 blocks.
- **Tests:** the strip's dots found and taken out (even and uneven frames, none), and the sprite and
  light read and written back with the sprite's spot.

**Tried (David, 2026-11-01):** "it looks really good... they look sick though." Seen: "the glow snaps
off the sprite, while its still visible". He'd placed 17 braziers, and only the nearest 8 glows go to
the shader, so the 9th-nearest's glow switches off as you walk. **Left for stop 3** (David: "lets just
wait for objects step 3"), whose light is worked into the faces and has no such limit.

## 10b. Stop 3, light through the air (David, 2026-11-01) [BUILT; David: "excellent. i can see them from far away sparkling."]

David: "you may proceed with step 3, but don't worry about implementing a torch or a wall light just
now, i should be able to build and register them myself from the registrar". He can: a model, its
sprite and its light are all set in the Objects tab. Only placing on walls is still to come.

**Outside in the day (2026-11-02):** David: "the brazieres light doesn't work outside". Placed light is
added to daylight and capped at full, so in full daylight it has nothing to add. It shows outside once
the sky is dim: for now the stand-in night, "how bright the sky is" on the Render page
(`docs/caves.md` 13.14); later the real night.

- **Placed light** (`render/placed_light.rs`), the second kind of light in `docs/caves.md` section
  11:
  - Each object with a light floods it out from its spot through cells that aren't solid: full at
    its spot, an even step darker each block, none at its reach (15 at most). It goes round corners
    and down tunnels, and **stops at walls**. Unloaded chunks count as solid.
  - **In colour:** each cell keeps red, green and blue, 16 levels each (its colour × its brightness
    × how near). Where lights meet, each colour is the brighter.
  - **Stored in the faces:** each face takes the light of the cell in front of it, in the vertex's
    **third word** (12 bytes a corner now, from 8). Faces merge only with the same light.
  - **There's no limit to how many lights there are.** A light costs only the faces it reaches.
    This ends the stand-in glow's "nearest 8" snapping.
  - **Worked out when it changes:**
    - when a light is placed or taken;
    - when blocks change near one (dig a wall away and the light comes through);
    - when chunks round one come in.
    The sections it reaches are made again.
- **In the shader** (`terrain.wesl`):
  - placed light is added to daylight, eased (squared) as daylight is;
  - **a flicker:** a gentle waver over time, from three waves, so it never quite repeats. All placed
    light flickers together for now.
  - On the Render page, under "Light underground": **placed light** on or off, **how much it
    flickers**, and **how fast**.
- **Objects are lit the same way:** daylight and placed light at the cell at their middle. A brazier
  is lit by its own fire, and a cup beside it by the brazier.
- **The stand-in glow is gone:** the nearest-8 shader glows and their arrays.
- **The Objects tab:** a light's reach goes to 15. The note says it spreads through the air and
  flickers.
- **Not yet:**
  - a flicker of each light's own, beside its neighbours';
  - light given by blocks (glow lichen), which is the same flood from a block;
  - moving lights (a held lantern: `docs/caves.md` section 11's third kind);
  - placing on walls.
- **Tests:**
  - a light spreads fading through the air, not into the ground, round a wall rather than through
    it, through once the wall is gone, and away when it's taken;
  - a face carries the light in front of it in its third word.

## 11. Later: overlays, random sets, alt textures (David, through Counsel, 2026-11-01) [NOT BUILT]

David: "we also need decoration type objects that are just a flat overlay that could be an ore or moss,
and we will want randomized set choices for blocks and object components (probably just 2d) so an
object like grass can place one of several sprites, or there are multiple moss patterns. id like to be
able to make different kinds of the same stone too, if that winds up looking cool. i guess itd be alt
textures?"

**Not in a current stop.** Recorded so it's built on what's already decided:
- `docs/decisions.md` (Blocks): "A block = material × inclusion... Inclusions are an optional overlay
  layer (ores first; the same mechanism can carry other overlays later)", and blocks may carry true
  texture variants.
- "Random face variants (e.g. some logs with knots)... picked from the block's position, so it needs no
  storage."
- The grass edge is already one shared overlay drawn over any soil, and `random_rotation`
  (`docs/data-files.md`) already turns faces at random by position.

1. **Overlays: a flat layer drawn over a block's face** (moss, an ore showing in stone).
   - **An ore is still a block of its own,** mined as itself. The overlay is only how it's drawn: the
     stone under it, plus the ore's layer. So one ore layer serves every stone it's found in (Counsel's
     suggestion; it's the "material × inclusion" already decided).
   - **Moss in the natural world** is the same kind of layer, decided by the projection (damp, dark,
     facing which way), not stored. It isn't a sub-block piece ("only blueprint stuff and player
     stuff"): it's a face drawn differently.
   - **Moss someone puts there** (a player or a blueprint) is a 2D object that sits on a face. That's
     the "sits on a wall" placing already on the list, saved like any placed object.
   - **The cost:** a second texture on a face needs room in the face's vertex, and only 7 of its 32
     bits are free (`docs/quartile.md`). The third vertex word planned for coloured light could carry
     it.
2. **Random sets: one of several, picked at random.** A block's face, or an object or a part of one
   (most often 2D), holds a list of choices:
   - grass places one of several sprites;
   - moss has several patterns;
   - a stone has several looks (David's alt textures, below).
   - **Picked from a hash of the position** (Counsel's suggestion, and the decided way): the same
     every time, nothing stored, whole numbers only (the Metaphysics Engine; `worldgen/noise.rs`
     already has `hash32_3`). A placed object's position is saved anyway, so its pick comes back the
     same. Only a choice someone makes by hand ("this one") is stored.
   - **Weights** on the choices (Counsel's suggestion, as Minecraft's resource packs do): a rare
     knotted log, a common plain one.
3. **Alt textures: different kinds of the same stone.** There are two ways, both on file:
   - **Looks of one block:** a random set on its faces (above). It's the same stone, mined and named
     as one.
   - **Blocks of their own:** a new block file (red granite, grey granite), if they should differ in
     more than looks.

   Which is right depends on how they look: David's "if that winds up looking cool".
