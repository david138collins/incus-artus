# Data Files

Game content is defined in plain text files that you write and edit. The game reads them at
startup, and while it's running it checks them about once a second: **save a file and the
change appears in the game**, no restart needed. (The same goes for block texture PNGs.)

Files are [TOML](https://toml.io): `setting = value`, one per line. Anything after `#` is a
comment and is ignored, so use comments freely.

## Where files go

```
assets/
  data/
    incus/              ← a "pack": the base game. A mod would add its own folder here.
      blocks/
        stone.toml      ← defines the block  incus:stone
        grass.toml      ← defines the block  incus:grass
  textures/
    blocks/
      stone.png         ← textures, 32 × 32, named in the block files
```

The block's ID is `<pack>:<file name>`. Pack and file names use **lowercase letters, digits
and `_` only** (e.g. `red_granite.toml` → `incus:red_granite`). IDs are what saves use, so
renaming a file later means old saves won't find that block.

## Blocks

Every setting is optional; leave out anything you don't need.

| Setting | What it does | Default |
|---|---|---|
| `name` | The name players see. | From the file name: `red_granite` → "Red granite" |
| `look` | `"solid"` (an ordinary block), `"grid"` (see-through, every cell drawn), or `"outline"` (see-through, only the outside of a volume drawn). | `"solid"` |
| `textures` | Which texture goes on which face (see *Faces* below). Texture names are PNG file names without `.png`. | none (magenta checkerboard) |
| `random_rotation` | A list of faces whose texture is turned at random per block, to hide repetition. Only for textures with no "up", and the texture must tile in all 4 turns. | none |
| `friction` | How grippy the top is to walk on. 1 = ordinary ground, lower = slipperier (0.1 ≈ ice), higher = grippier. | 1.0 |
| `solid` | Does it stop bodies? | `true` for solid, `false` for grid/outline |
| `targetable` | Can the crosshair stop on it (to break it or build against it)? | `true` |
| `replaceable` | Does placing a block into it just replace it, like air? | `false` for solid, `true` for grid/outline |
| `tags` | Labels the Registrar groups and sorts blocks by, as many as you like, like being in several folders at once. A `/` nests one tag inside another, as folders do: `tags = ["rock/igneous", "ore"]`. | none |

### Faces

Use these words in `textures` and `random_rotation`:

- `all`: all six faces
- `sides`: north, south, east and west
- `top`, `bottom`, `north`, `south`, `east`, `west`: one face each

More specific words win, so you can set `all` and then override just one face. On side faces
the top of the image points up; on the top and bottom faces it points north.

### Examples

```toml
# A plain block, the same on every face.
name = "Stone"
textures = { all = "stone" }
```

```toml
# Different top, sides and bottom; the top turned at random.
name = "Grass"
textures = { top = "grass_top", sides = "grass_side", bottom = "dirt" }
random_rotation = ["top"]
```

```toml
# A slippery block.
name = "Ice"
textures = { all = "ice" }
friction = 0.1
```

## When something's wrong

Mistakes never crash the game. They're listed in pink on the **Blocks** debug page, with the
file name (and the line, for typos):

- **A misspelt setting** (`frictoin = 2`): the file is refused and the block keeps its last good
  version, until you fix it and save again.
- **An unknown face word** (`roof = "..."`): a warning; the rest of the file still works.
- **A missing texture**: that face shows the magenta checkerboard.
- **A deleted file**: the block stays until the game restarts.

Changes made with the sliders on the Blocks page are temporary: saving that block's file replaces
them. To keep a tweak, write it into the file.
