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

## 9. After trying stop 1: sizes and snapping (David, 2026-11-01) [PROPOSED, not built]

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
