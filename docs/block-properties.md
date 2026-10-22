# What a Block Needs to Know

David's list (2026-10-21), with where each piece of information will live in the code.

## Where information lives

| Tier | Meaning | Cost |
|---|---|---|
| **Definition** | Written once in a TOML data file, shared by every block of that type | Free per block |
| **Dense** | Stored for *every* block in the world | Expensive: each byte here ≈ 400 KB per loaded chunk |
| **Sparse** | Stored only for the few blocks that need it | Cheap |
| **Not per-block** | Lives in its own system (regions, fields, visibility maps) | Varies |

## The list

| # | Property | David's description | Tier |
|---|---|---|---|
| 1 | Material | Stone/soil/wood type (granite, gabbro, silty loam, rich loam, white sand…). Hardness, sounds and colour derive from it. Many blocks share a texture in different colours. | Definition (material file) |
| 2 | Inclusions | Optional ore inside the block (hematite, native gold…) with density and grade. Changes mining and drops. Drawn as an overlay layer, like Minecraft vines, or an alternate texture. | Sparse, or derived from world gen |
| 3 | Yield | What drops when you do something to it. Sophisticated: resources are physical and take up space. | Definition |
| 4 | Hardness / tools | Mining time, tool types or tiers. Same rules for NPCs and player. | Definition |
| 5 | Structural | Gravity and support behaviour. Later. | Definition |
| 6 | Placement | Orientation within the octants. | Sparse (quarter-block cells) |
| 7 | Constructed | Natural or placed by someone. Useful for blueprints and overlays. | Dense (1 bit) |
| 8 | Haulable | How it must be carried or stored (wheelbarrow, cart, barrel, bag…). | Definition |
| 9 | Building ownership | Links to a building/region (field, house, workshop, pasture). | Not per-block: region system |
| 10 | Light | Emissivity and opacity. Later. | Definition (emission/opacity); computed light cached for rendering |
| 11 | Temperature | Emergent environment like the "hell of sand" browser games, Noita as the aspiration. | Not per-block: coarse field + sparse active cells |
| 12 | Growth | Dirt types and what grows on them. | Definition (rules) + Sparse (growth stage) |
| 13 | Inventory | For container blocks. | Definition (capacity) + Sparse (contents) |
| 14 | Discovered | Revealed on maps and in isometric view. | Dense (1 bit), or per-column |
| 15 | Seen | Fog of war shared by everyone in the settlement; fades after a few moments with no observer. RTS influence. | Not per-block: live visibility map from observers |
