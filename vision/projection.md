# The Metaphysics Engine: the metaphysical world and the physical world  [PRINCIPLE, David 2026-11-01]

**Official** (David, 2026-11-01): `docs/decisions.md`, "The Metaphysics Engine". This page keeps the
thinking behind it.

David, 2026-11-01:

> "we should be looking closely at how our engine works, considering future needs, and making sure
> that we cleave to the advantage we have - projection and interpretation. what we have actually
> constructed is a metaphysical world, and a physical world. i think if we think of every way we
> implement features in this way, so that 1 always equals 1, that will actually be what makes the
> emergent behaviors that fool the human eye into enough orderly commotion that you can look at and
> touch and it has meaning and reasons. its still chaos, but it's the strict reasoning that enables
> the chaos."

> "*nothing* in my world is truly random, because the 'random' parts have barcodes on their noise
> that lets us run it forwards and backwards at any time. nothing in minecraft is truly random
> either, but it can only go on forever precisely because there is no 'reason' for any of it."

> "i would rather see magic happen, its that simple. good enough won't make magic happens and we need
> magic." And: "the entire project can fail if we don't find a way to make it run like a top."

## The two worlds

- **The metaphysical world** is the form. It's made of:
  - the seed and its barcoded noise;
  - the stored maps (the bake, the climate, the rivers);
  - the plans (the river network, the peaks, the placer's choices, and later cave networks and roads);
  - the recipe.

  It exists everywhere at once, costs almost nothing to keep, and answers "what's here?" for any
  point, at any time, from any chip.
- **The physical world** is that form projected into blocks where it's needed: the chunks near you,
  the far view, the sweep. It's made from the form, then thrown away and made again as needed.
- **Differences on top:** what can't be projected (your edits, the settlers' work, simulations with
  a history) is stored as changes laid over the projection, as edits already are.

## What "1 always equals 1" buys

The same question at the same place always gets the same answer: near or far, CPU or GPU, today or
after a reload. From that follow:
- **The far view agrees with the near world,** because both come from one form.
- **Caching is always safe,** because an answer never goes stale unless its stage changes.
- **Work can move to the GPU,** because each column depends only on the form, not on its neighbours'
  history. The GPU can work out a million columns at once.
- **Last frame's answer is still true,** so a frame can reuse what it already worked out (below).
- **Emergence with reasons:** the chaos you see comes from strict rules crossing, so it holds up when
  you look closely and when you touch it.

## How to apply it (Counsel)

For every new feature, ask:
1. **Can it be projected?** That is, can it be worked out from the position, the seed and earlier
   stages? If so, it's a recipe or a field, never stored blocks.
2. **If it needs a plan** (a network, a shape, a choice made once), make the plan in its stage, store
   it compactly, and let blocks ask "am I inside it?" Rivers set the pattern; caves, badlands, roads
   and towns follow it.
3. **If it has a history** (sims, edits, growth), store it as differences on top. Keep that layer
   small and saved.
4. **Never let a later stage feed an earlier one** (the linear pipeline).
5. **Exact on every chip:** barcodes built from whole numbers, so the CPU and GPU agree to the block.
   Test it the way the placer's stop 3 did ("every block matches").

## Speed: the CPU and the GPU (Counsel, 2026-11-01)

- **The CPU** is fast at one thing right now. **The GPU** is fast at the same sum done a million
  times, but its answers come back later, usually a frame or more. Rule: give the GPU work whose
  answer the CPU never needs back, or needs only later.
- **The split:** the CPU keeps the small world near you (collision, aiming, digging, settlers,
  saves). The GPU takes everything seen out to the horizon (the far view, light, finding faces,
  deciding what's seen). Plans are made once, at world creation.
- **The order:** don't work anything out twice (cache what's projected); only work out what's seen
  and has changed; then move what's GPU-shaped to the GPU. The first finding (2026-10-28: the near
  light window rebuilt on every core as you moved) was projection work done again and again, on the
  wrong chip.
- **Temperature:** many current CPUs boost until about 90–100 °C by design, so frame times and how
  long the cores stay busy are the better measures. See `vision/watch-list.md` item 8 and
  `docs/worldgen.md` path step 8.

Details of the renderer: `vision/proposal-voxel-renderer.md`, including "temporal reuse" below its
kung fu.
