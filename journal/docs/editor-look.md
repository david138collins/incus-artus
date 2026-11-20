# The editor's look: Windows 95's, drawn by us

Counsel's brief for Builder, 2026-11-19, at David's asking: "please compile for builder all the technical
information re. win95 shell implementation we've come to discuss about the win95 look. i'm going to mock up what
the editor should look like after going through old incus and we'll start from there."

**Standing:** a leaning from 2026-11-18 (`journal/docs/decisions.md`, the rebuild). On 2026-11-19, after this
brief was written, he spoke of the method as decided ("what we've decided technically on how to implement a
win95 gui"), which Counsel reads as his go on the method. Nothing is built. Builder has since answered this brief and given her order
of building (under "Builder's plan" below); she builds nothing until his mock-ups come. His mock-ups come first;
they overrule anything here.

What is measured here is marked *measured*; what is Counsel's suggestion or a thing nobody has tried is marked so.

## Where we stand (the shared page, kept by Counsel; last brought up to date 2026-11-19)

David, 2026-11-19: "At this point, i'd like for you to reach out to builder and put together a doc with what
we've decided for you to go off of as i work up these mockups, so you two are on the same page starting now."
This section is that page. The sections under it are the reference it rests on. Builder's corrections go in
here in the open.

### Decided by him

| What | His words | When |
|---|---|---|
| The editor looks like a Windows 95 program, drawn by us to its recipes; every control's look is his | "id rather have something of my own"; then "what we've decided technically on how to implement a win95 gui" | 2026-11-18, 19 |
| His mock-ups come first and overrule this brief | "i'm going to mock up what the editor should look like after going through old incus and we'll start from there" | 2026-11-19 |
| The editor is mocked up at 1,280 x 800 (his laptop at x2); the editor's scale is its own | chosen of two offered | 2026-11-19 |
| The frame, from his first mock-up: a title bar; the menus, the page's drop-down and the toolbar's buttons on one row; a tabbed panel on the left (x 4 to 552); the viewport on the right; a status bar | his picture, in `Art/References/editor reference/` | 2026-11-19 |
| Pages are reached by the drop-down at the top; inside a page the panel has tabs | "I think we'd do the tabs in the panel like before" | 2026-11-19 |
| Tab names can shrink to one word if more tabs are needed | "we can always go to 1 word labels if we need more i suppose. if good sets of words for them are to be found" | 2026-11-19 |
| The Tectonic Sim's pages: Main settings, Plate settings, Land plates, Ocean plates, Simulation | his notes, whole in `journal/docs/decisions.md` (the rebuild, 2026-11-19) | 2026-11-19 |
| No real water in the Tectonic Sim: an estimated sea level shown in the viewport; water is World Gen's | "water will be placed entirely by the world gen section, ultimately" | 2026-11-19 |
| A setting is two lines: its name and tick boxes above; its slider, number and spread field below | asked for "the variables arranged like so for each one rather than in long rows"; of the redraw, "very nice" | 2026-11-19 |
| A panel longer than its page scrolls, the scrollbar on its right | "with a scrollbar on the right" | 2026-11-19 |
| No description text in the panels; mouseover information in yellow tooltips | "these are unnecessary going forward ... we can use the yellow tooltip boxes for any mouseover info" | 2026-11-19 |
| The controls: the plain 1995 set, the small pictures on buttons his; only the viewports special | section 7 | 2026-11-18 |
| Ironquill at 15 pixels is the editor's font on every sheet and mock-up | "a wonderfull standard font" | 2026-11-18 |
| The game's own interface is a separate thing: mocked up at 640 x 480, anchored, x2 at 1080p, x3 at 2K, x4 at 4K | section 10 | 2026-11-18 |
| **egui leaves the project.** Not kept as plumbing either: clicks, typing, scrolling and layout are ours | to Builder, who had suggested it: "yeah! lets trash egui." | 2026-11-19 |
| One text placer and one text-entry box of our own, shared by the editor and the game | to Builder: "i am confident you can spin up a text placer and entry device for both the editor and game." | 2026-11-19 |
| The viewports are old's final ones (their results, not their code), the leftover readouts dropped; square at 1,280 x 800, free to change shape when the window is resized or on other screens | to Builder: "the viewports we had in the end in old incus were good, there were some vestigial readouts and displays left over for precious iterations of them. theyll just slot into the 95gui as squares (at 1280x800, it doesnt have to stay square as the window is resized or on other screens)" | 2026-11-19 |
| The tabbed panel is fixed on the left, between a half and a quarter of the width, in the Tectonic Sim and World Gen | to Builder: "the panel with tabs will be a fixed 1/2 to 1/4 size on the left in tectonics and world gen." | 2026-11-19 |
| One program, F2 both ways. The editor runs windowed, filling the screen above the taskbar, in our own 95 title bar and borders; F11 makes it full screen, the editor only. The game is borderless full screen, with a setting to run it windowed in the same 95 frame | to Builder: "the editor will run windowed, so it stays snapped above the taskbar unless i hit f11. the game will be borderless fullscreen like it is now ... when you hit f2 itll go back to the editor, windowed or fullscreen, and f2 back again."; "f11 would only effect the editor, the game will stay borderless fullscreen. we can put a setting somewhere to enable it to be windowed just like the editor, with its 95 title bar and borders." | 2026-11-19 |
| The debug panel becomes a floating 95 window over the game, from a mock-up of his own; today's F3 panel stays until then | to Builder: "youll have a 95 style floating debug panelmockup when im done" (as he typed it; first set down here with a space, Builder's slip in the relay, found by her check); "ill have a mockup for the debug panel too when its time to switch." | 2026-11-19 |
| A scale setting, Auto by default; the general rule is enough until shipping, when each standard screen's default is picked from a set of pictures Counsel makes | to Builder: "there should also be a setting for the scale of the panel, with it being set to auto for standard ratios."; "we will figure out scaling for different cases when its important, the general rule with a scale setting should be good enough until shipping. we can always just have counsel make a whole set of images to select the default scale for each standard screen size/res" | 2026-11-19 |
| That scale setting is the editor's, for the parameters panel on the left, picked from the menu; the debug panel has a scale setting of its own somewhere, Auto its standard | to Counsel, asked which panel he meant: "the scale setting i mentioned was for inside the editor, referring to the parameters panel on the left. if someone doesn't like the scale, or wants to use half a screen or something, they can select a different scale from the menu. The debug panel will also have a scale setting somewhere, with the auto being standard." | 2026-11-19 |
| Probably two scales in the editor's menu, the editor's and the panel's: changing the editor's resets the panel's; panel scales that can't be had at the editor's scale are greyed, unless it is Auto. The debug panel has one scale of its own: 1x, 2x, 3x, 4x, 5x, Auto | to Counsel, asked whether the setting sizes the panel alone or the frame with it: "probably both have their own scale in the menu, with the editor scale resetting the panel scale when its changed, and unavailable scales greyed based on the editor scale if not auto. the debug panel would just have its own scale, 1x 2x 3x 4x 5x auto" | 2026-11-19 |
| The mouse wheel scrolls the panel, unless the pointer is inside a number box | to Counsel: "additionally note i need the mousewheel to scroll the panel unless the mouse is inside a number box." | 2026-11-19 |
| The wheel, by where the pointer is: over a number box it steps the number up or down; elsewhere in the panel it scrolls the panel; in the viewport of the Tectonic Sim or World Gen it zooms. In old the zoom was pixels per chunk in the Tectonic Sim, and 1/2x, 1x, 2x, 4x, 8x in World Gen | to Counsel: "oh, when its over a number box, it increments/decrements. otherwise, when its in the panel, it scrolls the panel. when its in the viewport, for tectonics or world gen, it zooms. the zoom was pixels per chunk in tectonics, and 1/2x, 1x, 2x, 4x, 8x for the world gen." | 2026-11-19 |
| A thorough set of mock-ups is on its way | to Builder: "i am working on a thorough set of mockups now that should make it easy for you." | 2026-11-19 |

### Drawn by Counsel at his asking, not yet answered by him (guesses; nothing here is decided)

- **The World Gen page** (example 3): the stages as one-word tabs (Main, Landmass, Topography, Climate,
  Mineralogy, Interpreter); sections that fold, each a bar with a tree's plus-or-minus box; "Show over the map"
  as one drop-down; old's "show ... on the map" buttons as tick boxes; land and compile time in the status bar;
  the settings that don't apply greyed, not hidden. He asked for the description box out and said no more of it.
- **The F3 debug panel as a floating window** (example 4; only an offer: his own mock-up of it is to come): tabs Overview, Camera, Render, its names from
  `src/panel.rs`; the passes as a list with column heads; Render's nine sections as folding bars.
- **The master palette** (example 5): splines as named rows of sunken wells, darkest first, in folding groups;
  "Not on a spline" as a box of loose wells; picked colours framed; a right-click menu on a spline; old's counts
  in a status bar; a Crush dialog after Paint's Edit Colors with old's own controls; a curve window.
- **The Software map** (examples 6 and 7), asked for "for studying, with the highlighting and hierarchy it has now,
  possibly with a panel on the left, a bit narrower than most, that shows the info about each module when it is
  clicked". Both are laid out from the code's own list (`src/wiring.rs`: 4 lanes, 32 passes, 27 stores, every
  wire). A panel 300 wide holds a name box (its drop-down jumps to any box) and the chosen box's card in folding
  sections: What it does, Wires (two lists), Now, Switches, Files. The chosen box wears eight handles; its wires
  are drawn over the rest with arrowheads the way the data goes; the status bar counts passes and stores.
  - Example 6 keeps the map's look as it is (his word of 2026-11-12: a black screen, magenta boxes).
  - Example 7, at his asking ("try to redraw the software map using the basic 95 scheme colors and box styles,
    with a teal background"): each lane a small window with a title bar (navy for the chosen pass's lane, grey
    for the rest; its minimise button would fold it), the passes push buttons, the stores sunken white wells,
    the chosen pass held down, its stores picked out in navy, on the 95 desktop's teal. The dimmed wires are a
    dark teal, the one colour outside the scheme.
  - Counsel's own in them, not his: white for the wires a pass writes; the folding of lanes; a click in the
    Reads or Writes list jumping to that store; the times (stand-ins from a flight).
  - **His direction of 2026-11-20, on his own mock-up of the page** (the 95 scheme on teal, the card panel as
    example 7 had it, the lanes' windows blanked out and only the stores' white boxes left): "Can you redo the software map page with the modules all in the grouped boxes like these? the lines can be dark gray and behind the boxes, until one is clicked, at which point its lines become blue and on top"
    So: every module the same white sunken box as the stores; the boxes in groups; every wire dark grey and
    behind the boxes; a click turns that box's wires blue and brings them on top. The lanes as small windows
    with title bars, and the passes as push buttons, fall with it.
  - **Example 10, at that asking, not yet answered** (drawn onto his picture; only the map's own area repainted):
    five groups under white captions (the four lanes by name and when they run, then the stores), the boxes in
    wrapped rows in the order the code lists them; 103 wires in a dark grey (404040) from box to box behind
    them; the chosen box navy with a dotted focus line, the boxes it is wired to navy, its wires navy and two
    pixels wide from edge to edge over everything, an arrowhead the way the data goes. 10b is the map with
    nothing chosen, 10c with a store chosen (a store's wires are the passes that read or write it).
    Counsel's own in it, not his: the captions and their words; the arrowheads; the focus line; navy for
    "blue" (the selection's own colour, the only blue in his `95.gpl`); the grey's exact value (404040; the
    scheme's own 808080 read as pink against the teal).
- In all of them the arrangement, the numbers, and in the palette the splines' names and the menu's words are
  Counsel's stand-ins.

### What the rows above overrule further down this brief

- Section 2, way 3: "egui is kept only as unseen plumbing". **Overruled:** egui goes.
- Section 8: its three levels (written for egui) and its questions on egui as plumbing, on the pixel font
  through egui, and on the outer title bar. **Answered:** our own controls; our own text; and, by his word, our
  own 95 title bar and borders on the window ("with its 95 title bar and borders"). How that is done is
  Builder's plan, below. Its other questions stand.
- Section 10: "the editor's scale is not chosen" on other screens. **Answered until shipping:** a scale
  setting, Auto by default.
- Section 13: the rows on the price of egui against her own controls, on the pixel font through egui, and on
  where the editor begins (his mock-ups come first).
- His words to Builder are relayed by her (her message to Counsel, 2026-11-19) and are whole in
  `journal/docs/decisions.md` (the rebuild, 2026-11-19).

### Builder's plan from them (hers; not his word, and not yet his go)

- **Text:** each font drawn from a glyph sheet of our own at its true size (Ironquill at 15) and whole
  multiples. A baked glyph sheet may also be how bought fonts reach the build without their files: his and
  hers to confirm.
- **The window:** made without Windows' frame; the dragging, the resizing by the edges and maximise above the
  taskbar are ours; the editor's mode (windowed or full screen) is remembered across F2.
- **Auto scale:** the largest whole number at which 1,280 x 800 fits: x2 on his laptop, x1 on 1080p and 1440p.
- **The panel's width:** at 1,280 x 800 a square viewport under the bars is about 720 across, leaving the panel
  about 560, just under half; his mock-ups set it exactly (his first has the panel at x 4 to 552 and the
  viewport's frame from 555, about 721 for the viewport: the same reading).
- **Her order, proposed:** A, the painter and the bare frame, compared pixel for pixel with his picture; B, the
  frame's behaviour (drag, resize, maximise above the taskbar, F11, F2); C1, the controls drawn, compared with
  the control sheet; C2, the controls working, the text-entry box among them; D, the viewports. Then the debug
  panel from its own mock-up, and egui out of `Cargo.toml`.
- **She builds nothing until his mock-ups come.**

### The master palette in the Registrar: his direction (2026-11-19; a contemplation, its words may shift)

Not decided, and Counsel draws nothing of it for now ("for now, don't try to mock anything up yourself"). His
mock-up of the Registrar's right-hand palette panel is in hand. What he has said, whole in
`journal/docs/decisions.md`:

- **Why "spline":** "a spline implies it arches? and we can show the arch procedurally with hue, sat, or lum."
- **The look:** "a compromise between the logic of late 70s-late 80s hardware-based utilities/editors, and
  high-density conventions that more modern software uses".
- **A dense view of every regulated colour, not by spline,** arranged by orders he can switch with tick boxes:
  "like hue by lum by sat, so that it lists each hue in order in rows and groups colors down the rows by lum
  and sat comes last when deciding by some override-ratio ... because lum and sat are sort of siblings"; "see
  the 'dense' palette arrange itself in patterns readable to the eye, so i can bounce that to gpl and paint
  with it and not have to run between splines".
- **Old's Registrar's explicit features stay,** "for the most part".
- **The panel doesn't scroll as a whole:** "we need to consider ways to make the larger part of the right-hand
  palette panel in the registrar ... not scroll as a whole ... there will need to be some scrolling internally".
- **The method:** "working through this stuff to draw perfect mockups for builder seems to me the soundest way
  to make sure we get it just right, so some features, terminology, and behavior may derive backwards from how
  we go about researching and designing these mockups"; "some of the lexicon we've used on the master palette
  may shift as we rework it".
- He likes NEOchrome Master's colour grid: "the hue-by-lum-by-sat in a sort of diamond pattern into a grid".

**The references** (114 pictures gathered for him that day, with a manifest for each group and one index page):
`Art/References/palette reference/index.html` (off GitHub; on this machine). Five groups: Atari ST and Amiga; professional
and workstation; Japan and Asia; Europe, the Soviet bloc and South America; PC game art and modern tools. What
they show that bears on his direction, each from a source named in its manifest:

- NEOchrome Master's diamond is the machine's whole colour space laid out by one fixed rule and dragged about
  (black in the middle, brightness in rings outward, hue round each ring); it does not sort a picture's own
  colours. Spectrum 512 drew all 512 colours as eight hexagons, the cube seen corner-on.
- Sorting a palette in place while the picture keeps its look: Deluxe PhotoLab's Colors (by red, green, blue,
  hue, saturation, value or pixel count), Sega Painter (by use, unused colours to the end), Pro Motion.
- Ramps as units: Autodesk Animator's clusters; Pro Motion's named gradients and its palette shown as rows of
  ramps; Deluxe Paint IV's ranges; Animo's named wells in a character's colour model.
- A whole palette re-plotted many ways on one screen (hue against brightness among them): DawnBringer's
  Analyze Palette for GrafX2.
- Several banks at once: Graph Saurus 2's eight palette tracks; EDGE2's palette banks; NEOchrome Master's up
  to 200 palettes a picture by scan line; Graph2Font's palette for every raster line.
- One palette for many pictures: Animator's One Palette; Personal Paint's merge of two pictures' and nine
  brushes' palettes; Wally's one master palette a game.
- **The "orphan colors" bin he had heard was SGI's** (with "fit to nearest ramp" and "promote to new ramp"):
  no source found for the term, the bin or the buttons; SGI's tools worked on raw runs of colour-table entries.
  The nearest real things: Animator's Unused Colors and Fit; Animo's choice to discard or create wells that
  don't match. The idea stands on its own; it has no precedent to lean on.
- Not found at all: HAM-E's software; Alias and Softimage ramps before 1998; anything from Korea, Taiwan,
  China or South America; period screens of DeBabelizer and Director's palette window.

### For later: creator windows that nest, after StarCraft's trigger editor (his reference, 2026-11-19)

Not for now. He installed the original StarCraft and shared five pictures of its editor's trigger windows
(`Art/References/editor reference/StarCraft triggers (his reference)/`): "I found and installed the original starcraft, and have been looking through its editor. I wanted to share the trigger window. I think that may be useful at the point that we're setting the GUI up to add new placement fields in ecology or civilization, where it shouldn't be hardcoded so i can keep expanding the content. Some types of features, like the way a tree type is placed, may get pretty complex to make a new type of rather than just adjusting a given type, so we may wind up with nesting creator windows for big things like placement fields, or behaviors, as we'd apply them to a entity/mob/actor in the registrar when im making bad guys and types of villagers."

What the pictures show, for when that point comes (Counsel's reading of them):

- Four windows deep, each small. **Triggers:** a list in which each trigger is a block of plain sentences
  under CONDITIONS and ACTIONS, with New, Modify, Copy, Delete, Load and Save, and Move Up and Move Down for
  their order. **Trigger Properties:** three tabs, Players, Conditions, Actions. **New Condition:** one
  drop-down of condition types, and under it the chosen type as a sentence with underlined words ("Player
  accumulates quantity resources"). **Qualified Number:** the small window an underlined word opens (At
  least, At most, Exactly; a number).
- The part that answers "it shouldn't be hardcoded": a type is a sentence with blanks, and each blank opens a
  small window for its own kind of value. A new type is a new sentence and its blanks, with no window drawn
  for it by hand.
- OK stays greyed until every blank is filled.

### The Registrar's library panel: his direction (2026-11-19; he is pasting it together in Aseprite)

Not decided; his mock-up is to come. With a sixth StarCraft picture (its Unit and Hero Settings window: a tree
on the left, a unit's numbers on the right, greyed while "Use defaults" is ticked), which he shows as what
does NOT suit: "this pic is the window where you can adjust the stats of units and buildings. you weren't able to create new ones like our registrar can, and the file tree doesn't really suit for a big library/asset window on the left that can be sorted by tags that we can expand on. Asset types would be hardcoded, i think; blocks, compound blocks, animated blocks, objects, fx, blueprints, weapons, armor, clothing, etc etc. types would be tags but tags wouldn't be types, for the purpose of library sorting, vs just alphabetically scrolling through every asset in the game. think of it like a really well made online shopping filter, compact and sitting on top of the list. Not sure if the library panel, on the left, would need for tabs at all, or just search filters on top of a list. im working through pasting things together until i get the idea in aseprite."

- **The library is on the left:** a big list of every asset, with filters sitting compactly on top of it, "like
  a really well made online shopping filter". Not a tree of folders.
- **Asset types are hardcoded:** blocks, compound blocks, animated blocks, objects, fx, blueprints, weapons,
  armor, clothing, and so on. **A type is a tag, but a tag is not a type:** tags are his to add to; types are
  the program's.
- **New assets are made in the Registrar,** which StarCraft's window could not do.
- **Tabs in the library panel, or only filters over one list:** he isn't sure.
- An older line of the spec has the Registrar's tabs as Blocks, Objects, Blueprints. If types become filter
  tags over one list, that line is overtaken; his to say when his mock-up shows it.
- Of creator windows that nest (the block above): "right, some things would be complex enough that making a new one takes you through a russian doll of menus."

### Open

| Question | Whose |
|---|---|
| ~~Which panel "the scale of the panel" is~~ | Answered 2026-11-19: the parameters panel on the left, inside the editor; the debug panel has its own (the row above in "Decided by him") |
| ~~Whether the editor's scale setting sizes the panel alone or the frame's bars with it~~ | Answered 2026-11-19, with a "probably": each has its own scale in the menu (the row in "Decided by him") |
| Which panel scales count as unavailable at each editor scale (the rule behind the greying); what "reset" sets the panel's scale to; whether the editor's scale has the debug panel's steps (1x to 5x and Auto); what Auto means for the debug panel; where the debug panel's scale setting sits | Builder to put to him when the menu is built; David |
| ~~What the wheel does when the pointer is inside a number box~~ | Answered 2026-11-19: it steps the number up or down (the row in "Decided by him") |
| The wheel's step in a number box (how much one notch changes each setting); whether the new viewports keep old's zooms as they were ("the zoom was ..."), and what the wheel does over the Software map and the other pages' views | Builder to propose from old; David |
| The Software map: ~~black and magenta as now (example 6), or the 95 scheme on teal (example 7); every wire drawn dimmed, or only the chosen box's~~ answered by his mock-up and his words of 2026-11-20 (the 95 scheme on teal; every wire dark grey behind the boxes, the chosen box's blue on top). Still open: whether a box's describing text stays in its card (his mock-up keeps it, the one place a panel carries such text); his answer to example 10 | David |
| Where the two settings go and what they are called: the game windowed; the scale | David: his mock-ups, or Builder asks |
| Still wanted from him, by Builder's answer to this brief: the mock-ups as lossless 1,280 x 800 PNGs in `Art/References/editor reference/`; the title font; the toolbar's five buttons; what fills the status bar's panes; the open menus and the Tectonic Sim page; which of old's viewport readouts go | David |
| Screens with less room than 1,280 x 800 | David, near shipping |
| A smaller font for a floating window's short title bar (Ironquill has one size; Antiquity Print at 13 is his smallest), or the full-height bar | David |
| Tooltips: old's descriptions run to five lines and a 95 tooltip was one short line: wrapped tooltips, or the texts cut down | David, when it comes up |
| Folding sections: the bars as drawn, or a tree that shows one section at a time | David |
| Where the Field size, pixel size and bake controls of old's top bar go in World Gen (Counsel assumed a Main tab) | David |
| The rest are in section 13 | |

### His mock-ups: `Art/95gui mockups/` (his own folder, from 2026-11-19; in progress, not yet handed over)

He rearranged `Art/` on 2026-11-19: his mock-ups are in `Art/95gui mockups/`, the references in
`Art/References/` (`editor reference`, `font reference`, `palette reference`, `menu mockup`), his palettes in
`Art/palettes/` (`95.gpl` and its Aseprite file among them). As seen there on 2026-11-19, each 1,280 x 800:

- `tectonics.png`: the Ocean plates page with two-line settings and a scrollbar (Counsel's example 2, his now).
- `world gen.png`: the Climate tab with folding sections (example 3, his now).
- `software map.png`: the 95 scheme on teal (example 7, his now), its left panel blanked to a dark box for
  him to fill.
- `registrar.png`: his own frame for the Registrar (below).
- `blank.png`: the bare frame. `old*.png`: his screenshots of old's pages.
- The title bar now carries the build's number at its right, before the buttons ("Build 238").
- That he took examples 2, 3 and 7 into his folder says they are his starting points, not that every part of
  them is answered: until he hands a mock-up over, what is unanswered above stays unanswered.

### The Registrar's frame: his mock-up in progress (2026-11-19)

`Art/95gui mockups/registrar.png`. Three boxes side by side under the bars, outlined in magenta as placeholders:

- **Left, about 255 wide:** on top a box 96 tall for the search filter; under it the library's list with its
  scrollbar. A row as he has it: the name large, a line of tags greyed ("rock, block, natural"), a greyed
  "Regulated" tick box, a thumbnail of 32 x 32 in a sunken frame; rows 51 tall, twelve showing.
- **Middle, about 615 wide:** the registry editor.
- **Right, 400 wide:** the palette, fixed: "I'd reserve probably 400 pixels like shown for the palette so it can
  be immobile, but the size of the 3 box areas on the left can be messed with."
- His question to Counsel, with it: "I can't think of anything the list needs to show except for the name, thumbnails (how many? if it's multiple can they be leafed?) and the relevant tags each block can be found by. And the type, too i guess. the small space above that list can be for the search filter. the middle box is obviously the registry editor and the right is the palette. I'd reserve probably 400 pixels like shown for the palette so it can be immobile, but the size of the 3 box areas on the left can be messed with. Got any ideas?"
- Counsel's ideas, offered in answer and none of them his yet: one thumbnail a row (a block as a small cube
  seen corner-on, its top and two sides in one picture; leafing left to the middle box); rows of two lines
  (name; type then tags) with the thumbnail at the left edge, about seventeen showing; "regulated" shown only
  as a mark on the rows that aren't, and as a filter; the filter box as four rows (find by name, type, tags,
  unregulated only, with a count); a second view of the list as a grid of thumbnails; the bar between list
  and editor draggable.

- **Later the same day, his words on the list and its filter:**
  - "note our block textures are 32x32." So a thumbnail of 32 x 32 is a texture at one pixel to one, and Counsel's idea of a small cube
    falls (it would shrink the textures).
  - **One thumbnail an entry, with a plus mark** on blocks of several textures, which "expands them in a tooltip
    on hover" (his).
  - **The filter, two ways he is weighing:** type as buttons, one held at a time and released by a second click,
    with a drop-down of tags grouped one level deep, "so you could select a type and a tag"; or no tags at all,
    "just use types, and have types be a drop down menu", which "would allow us to use different types of
    thumbnails or text for different types of assets". The library shows everything by default for now.
  - **Counsel's examples 8 and 9, at his asking, none answered:** the whole Registrar page in his frame (the
    library; the dock as boilerplate from old's Registrar; a stab at the palette), and four ways of the library
    panel alone: A, type buttons and a tags drop-down, his large names; B, the same with the tags open as
    one-level groups with tick boxes; C, types as a drop-down and no tags, short rows whose second line differs
    by type; D, the list as a grid of thumbnails. The pictures in them are the game's own textures
    (`old/assets/textures/`); the palette's colours are counted from those textures (115 across thirteen, 19
    for grass, as old's Registrar says). The tags, each block's type and the counts are stand-ins.
  - In the palette stab, what answers "not scroll as a whole": a dense view of every colour (a row a hue,
    darkest to lightest across, two drop-downs for the order), the "Not on a spline" strip and the buttons stay
    put; only the list of splines scrolls.

### The pictures, all in `Art/References/editor reference/` (off GitHub; on this machine)

- The control sheets (Ironquill and the plain font, numbered and clean) and the probe's pixel maps (the appendix).
- `David's mock-up as pasted 2026-11-19.png`: his frame. The truth where it differs from anything else.
- Example 1: the Ocean plates page in long rows. **Overtaken** by example 2.
- Example 2: the Ocean plates page, two-line settings, scrolling: the whole mock-up, the panel alone (549 x 723,
  at 4, 48), and the page's whole contents (529 x 1,036).
- Example 3: World Gen, the Climate tab, folding sections: the same three pictures (contents 529 x 882).
- Example 4: the F3 debug panel, floating. Example 5: the master palette, crush and curve.
- Example 6: the Software map, black and magenta, with a card panel. Example 7: the same in the 95 scheme on teal.
- Example 8: the Registrar page (library, dock, palette). Example 9: four ways of the library panel.
- Example 10: the Software map with every module a box in its group, grey wires behind and the chosen box's in
  blue on top (10b nothing chosen, 10c a store chosen); its drawer is `software map drawer 3 (grouped boxes).cs`.
  (Example 7's picture he took into his own folder as `software map.png`.)
- Each drawer's source beside its pictures (`*.cs`, run from PowerShell); every edge in them is drawn to
  section 4's recipes, so they are also a worked example of the recipes in code.

### How the two of us work from this

- His mock-ups are the specification. Counsel reads each one he hands over, records what it decides in his
  words, and brings this page up to date in the same commit.
- Where a mock-up and this page differ, the mock-up wins and this page is corrected.
- Counsel's examples are offers for him to mark up. Builder builds nothing from an example he hasn't answered.
- Builder prices and builds from his mock-ups and section 4's recipes, a pass mark first as always, and asks him
  where each control goes and what it is called before placing it.
- Either of us who hears a word of his on the editor passes it to the other whole.
- **The handover, his words of 2026-11-19:** "i think what in gonna wind up doing is making the mockups as best as i can, handing them to builder with a prompt from you about any details i forgot, and then letting builder go from there and micromanaging anything beyond the overall setup with her." So when he
  hands the mock-ups over, Counsel writes the prompt that goes with them: what the pictures don't show (the
  decided rows above, the behaviour, the sizes and recipes) and what is still open. Past the overall setup, the
  details are his and Builder's, worked out between them as she builds; she passes his words to Counsel for
  the record, as before.


## 1. What he wants, in his words

- "how much of a pain would it be to make the new editor look like a windows 95 app"
- "its not just a look; its how i understand complicated software working, and gives me a handle for how to think
  about organizing features for ergonomics and power."
- "i don't really love E-gui and *would* probably love win95 .net style debug panel and even desktop app wrapping
  for the game before it launches ... like dungeon keeper did. idk if that's necessary at all but that's how i'd
  like to set up the editor."
- "how do you emulate the look exactly, so i can use my own behaviors. could we just gank the way the apps are
  drawn, and call it from rust as we please? and yes, we'd have windows in the app for our own things like
  previews and the map."
- Asked whether it is egui's look or its behaviour he doesn't love: "part of it is the generic look which is half
  vanity, and half just reacting to something *i didnt choose* ... genuine will to make everything per-fectus, and
  not slop. id rather have something of my own just because basically".

So: it is the look, and that he chose it. The behaviour is ours to choose with him, one convention at a time.
Every control's look is his to change; the recipes below are the starting point he picked from, not a rule.

## 2. The three ways weighed, and where it rests

1. **Windows' own controls** (user32 and comctl32 called from Rust, the classic theme: what a 1995 programmer
   used, still shipped in Windows 11). Genuine controls and all their behaviour. Set aside: Windows only for good;
   controls placed by hand; sharpness on a 2,560 x 1,600 screen unknown (the real bitmap font comes out tiny or
   smeared, by how Windows is told to scale); our card-drawn views would live in child windows and need a shared
   message loop; the outer title bar stays today's unless the program draws its own. He stepped back from it
   himself: "i think i may overstate how deep i recognize the .net conventions".
2. **Windows' own painter, called by us** (`DrawEdge`, `DrawFrameControl`). It paints the genuine edges and marks,
   but onto Windows' own drawing surfaces, not the card's picture: a copy across each frame, and still Windows
   only. Set aside for the program. **Used once, as the reference:** every edge and mark in this brief and in
   the control sheets came out of it (section 4, the appendix).
3. **Our own drawing to the same recipes.** The look is a palette, a pixel font and about a dozen recipes of
   one-pixel lines, so drawing them ourselves gives the same pixels on any system, doubled whole for sharpness.
   **This is where it rests.** egui is kept only as unseen plumbing (clicks, typing, scrolling, layout) unless
   Builder prices her own as worth it.

Nothing of Microsoft's is taken: no files, no font. MS Sans Serif appears in one reference sheet only and cannot
ship with the game.

## 3. The palette

*Measured:* counted on the two clean sheets, which hold these colours, the focus dots' `#FFFF7F` and the richer
list's six sample swatches, and no others.

| Name here | Colour | Used for |
|---|---|---|
| face (F) | `#C0C0C0` | every window and control surface |
| white (W) | `#FFFFFF` | highlights; the inside of fields and lists |
| light (L) | `#DFDFDF` | the second highlight line |
| shadow (S) | `#808080` | the first shadow line; greyed text; an inactive title bar |
| black (K) | `#000000` | the outer shadow line; text; marks |
| navy | `#000080` | the active title bar; a selected row; progress blocks |
| tooltip | `#FFFFE1` | a tooltip's fill |
| teal | `#008080` | the 95 desktop behind the window, on the sheets only |

## 4. The edges (*measured* from Windows' classic painter; the pictures are in the appendix)

Every edge is two one-pixel lines, an outer and an inner, each split into a top-left colour and a bottom-right
colour. The bottom-right line owns both far corners.

| Edge | Outer: top-left, bottom-right | Inner: top-left, bottom-right | Used for |
|---|---|---|---|
| Raised, window kind | L, K | W, S | the window's frame, a subwindow's frame, scrollbar arrows and thumb, the drop-down's and number field's small buttons |
| Raised, button kind | W, K | L, S | push buttons, the title bar's buttons, a tab page |
| Sunken | S, W | K, L | text fields, lists, the viewport's frame, check boxes, the slider's channel |
| Etched | S, W | W, S | group boxes, dividing lines (a groove: a shadow rectangle, and a white one a pixel down and right) |
| Sunken, one line | S, W | none | status bar panes, the progress bar's frame |
| Held down (toolbar) | K, W | S, L | a toolbar button held in |

The two raised kinds differ only in which of white and light is outermost. It is easy to miss and shows at once
beside a real one.

## 5. The marks and hand-drawn parts

**Painted by Windows (*measured*, pixel maps in the appendix):** the check mark (in a 13 x 13 box) and its greyed
form; the radio button (12 x 12, a shaded ring, a 4 x 4 dot with its corners off); the four scrollbar arrows
(a black triangle 7, 5, 3, 1 pixels, in a 16 x 16 raised button); the drop-down's arrow; the number field's two
small arrows (3 and 1 pixels); the title bar's close, maximise and minimise marks (16 x 14 buttons); the size
grip (14 x 14, diagonal lines of white, shadow, shadow).

**Drawn by Counsel to the same rules** (in `Art/References/editor reference/control sheet drawer.cs`; the 95 originals were
followed from memory and from the 98.css page, so check these against his mock-ups first):

- **A button held down** (a dialog's kind): flat; a black box, a shadow box inside it, the label moved one pixel
  right and down.
- **The default button:** a black box round a button one pixel smaller on each side. **The focus box:** a dotted
  rectangle, every other pixel, four pixels inside the button's edge. On a selected navy row the dots are
  `#FFFF7F` (navy turned inside out), as Windows gets by flipping the colours under it.
- **Greyed text:** the label in white one pixel right and down, then again in shadow on top.
- **A greyed check box:** its inside face-coloured, its mark shadow.
- **The scrollbar's track:** a checkerboard of white and face, white where column plus row is even.
- **The slider:** a sunken channel four pixels tall; a thumb 11 wide and 21 tall, pointed downward over its last
  five rows, its left edge white, its right edge black with shadow inside it; tick marks 1 x 3, black.
- **Tabs:** each with its two top corners rounded over two pixels; white then light down the left and across the
  top, shadow then black down the right. The chosen tab is two pixels taller and two wider, lying over its
  neighbour, and rubs out the page's top edge under itself; the others' labels sit a pixel lower. The page is a
  raised edge of the button kind.
- **A group box:** an etched rectangle whose top line runs through the middle of its label's height; the label
  on a face-coloured patch that breaks the line, starting seven pixels in.
- **The progress bar:** a one-line sunken frame; navy blocks 8 wide, 2 apart, 2 in from the frame.
- **A tooltip:** tooltip fill, a black one-pixel box, the text 4 in and 2 down.
- **A selected row:** navy fill, white text.
- **The title bar:** a navy bar inside the window's frame (the two-pixel raised edge, then two pixels of face);
  the title in white, bold where the font has a bold; its buttons 2 down from the bar's top, close at the right,
  maximise and minimise together to its left with a 2-pixel gap between them and close. An inactive window's bar
  is shadow with face-coloured text. (95's bar is one flat navy; the fade from navy to blue is 98's.)
- **The menu bar:** words 12 apart; the letter that Alt picks underlined by a one-pixel line a pixel under the
  text's baseline.
- **The toolbar:** an etched line above it (shadow, then white); buttons 24 x 22.
- **The status bar:** one-line sunken panes; the size grip in the bottom right corner.

## 6. Sizes (*measured* on the sheets; each follows the font's line height)

| Part | Rule on the sheets | Ironquill at 15 (line 15) | MS Sans Serif at 11 (line 13) |
|---|---|---|---|
| Title bar's height | the larger of 18 and line + 5 | 20 | 18 |
| Menu bar's height | the larger of 19 and line + 6 | 21 | 19 |
| Push button's height | the larger of 23 and line + 10 | 25 | 23 |
| Push button's width | the larger of 75 and "Cancel" + 24 | 75 | 75 |
| Field, drop-down, number field | the larger of 21 and line + 8 | 23 | 21 |
| A list's row | the larger of 13 and line | 15 | 13 |
| Status bar's height | the larger of 20 and line + 7 | 22 | 20 |
| Check box, radio | 13 x 13, 12 x 12, the label 18 from the box's left | the same | the same |
| Scrollbar | 16 wide; arrows 16 x 16 | the same | the same |
| Drop-down's button | 16 wide, the field's height less 4 | 16 x 19 | 16 x 17 |
| Number field's two buttons | 15 wide, sharing the field's height less 4, the upper the taller by the odd pixel | 15 x 10, 15 x 9 | 15 x 9, 15 x 8 |
| Title bar's buttons | 16 wide, the bar's height less 4 | 16 x 16 | 16 x 14 |
| Toolbar's buttons | 24 x 22 | the same | the same |

The second column is 95's own sizes with its own font. With Ironquill each text-bearing control is two pixels
taller. Spacing between controls on the sheets (6 between buttons, 18 between rows) is Counsel's and means
nothing: the layout is his mock-ups'.

## 7. The controls he picked (the sheet's numbers)

His words: "i like 4, 5, 6, 7, 8, 9, 10, 11 but i'll get the right button, 12, 13, 14, 15, 16, 17, 18, 19 (but
i'll get the right buttons), 20 but it'd be fixed in the window no title bar. i'd get the right buttons for 1, 2
looks right. 3, i'd get the right buttons; and we'd need some rich features done our way up there *possibly*. i
think only the viewports need to be special, and possibly richer boxes in a window list with a scrollbar, that
contain a preview and multiple fields like in the registrar."

| No. | Control | His word |
|---|---|---|
| 1 | Title bar with its three buttons | wanted; the buttons' pictures his |
| 2 | Menu bar | "looks right" |
| 3 | Toolbar, with the page's drop-down | wanted; the buttons' pictures his; perhaps richer things of our own there |
| 4 to 7 | Push button: plain, held down, the default one, greyed | liked |
| 8, 9 | Check boxes, radio buttons | liked |
| 10 | Text field | liked |
| 11 | Drop-down | liked; its button's picture his |
| 12 | Number field with up and down arrows | liked |
| 13 | Slider | liked |
| 14 | Scrollbar | liked |
| 15 | Progress bar | liked |
| 16, 17 | Tabs, group box | liked |
| 18 | List, one row selected | liked |
| 19 | Subwindow with its own title bar | liked; its buttons' pictures his |
| 20 | Viewport | fixed in the window, a sunken frame, no title bar; "only the viewports need to be special" |
| 21 | Tooltip | wanted, 2026-11-19: "we can use the yellow tooltip boxes for any mouseover info" |
| 22 | Status bar | in his own first mock-up, 2026-11-19 |
| 23 | A list of richer rows: a preview, a name, fields, a check box, with a scrollbar | his idea, "like in the registrar"; drawn once as a guess |

Where a small picture is his to supply, the sheets carry a dotted placeholder or Windows' stock mark.

## 8. Under the look: what is known, and what is Builder's to price

**As the code stands** (read 2026-11-19): the F3 panel is egui 0.36.2, drawn on the card by egui-wgpu with its
input from egui-winit (`Cargo.toml`); `src/panel.rs` is 481 lines; `src/runner.rs` holds the egui state, the
renderer and `pixels_per_point`. Nothing in it is styled, so there is nothing to undo.

**Counsel's first sketch of the work, in three levels** (told to David 2026-11-18; a guess, not a price):

1. *The feel:* silver panels, square corners, navy title bars, black text, the pixel font. egui keeps its look in
   one block of settings, so this is one build and one look.
2. *The bevels:* egui draws no two-tone edges, so a small kit of our own controls drawn to sections 4 and 5, used
   by every page. Built once and shared; cheapest before the Tectonic Sim and World Gen pages exist.
3. *Exact to the pixel everywhere,* menus that behave as 95's did in every case: open-ended, not recommended.

**Questions for Builder, none of them tried:**

- **egui as plumbing, or controls of her own?** With egui kept, what of the 95 kit can be painted through its
  painter (plain rectangles and one-pixel lines at whole pixels), and where does egui's own drawing leak through
  (rounded corners, its text cursor and selection, its scroll areas, its window frames and shadows, its menus)?
  If her own, the long part is the text field (the caret, selection, the clipboard, key repeat).
- **Every line on a whole pixel.** The recipes are one-pixel lines. egui lays out in points and smooths edges
  that fall between pixels; the kit needs its rectangles snapped to whole pixels at a whole scale, and textures
  read without blending. To be tested on his screen, not assumed.
- **The pixel font, crisp.** The sheets got it from Windows' text drawing with smoothing off, at exactly the
  font's true size (section 9). egui rasterises a font's outlines with smoothing, so a pixel font may come out
  with grey pixels or uneven stems. Two ways to price: egui's own text at the true size times the whole scale,
  looked at closely; or a glyph sheet of our own, each letter a one-bit picture at the true size, drawn as plain
  squares. Counsel's drawer can make that glyph sheet for any of his fonts.
- **The outer title bar.** He wants control 1, a 95 title bar with his own buttons. Windows draws the real
  window's title bar itself and it stays today's unless the window is made without one and we draw ours, which
  means doing the dragging, the resizing by the edges, maximise, and the snap to screen edges ourselves (winit has
  calls for starting a drag and a resize; check them at the version in use). The cheaper cut: Windows' own frame
  outside, the 95 title bars on the subwindows inside. His to choose once priced.
- **Subwindows** (control 19): draggable windows inside the editor for previews and the map. egui has them;
  under a 95 skin their frame, title bar and close button would be ours.
- **The viewports** (control 20): the card's own pictures inside a sunken frame, fixed in the window. How a
  page's view gets its rectangle and its clicks.
- **The game's interface** (HUD, menus, in section 10) is a different thing from the editor and may want the
  same whole-pixel drawing underneath: one piece or two?

## 9. Fonts

- **Ironquill at 15 pixels is the editor's font on his sheets** (Francisco Cidade; bought by him). It has no
  bold: Windows' made-up bold smeared it, so title bars and names are regular weight on the Ironquill sheet.
  Line height 15, ascent 12, descent 3 (*measured*).
- **A pixel font has one true size:** the height at which one font pixel is one screen pixel. It is drawn only
  at that size and whole multiples of it; anything else smears. *Measured* for his fonts (the size whose drawing
  at twice the height is the same drawing doubled): Antiquity Print 13, Ironquill 15, Troll 16, Sorcery Code 16,
  Darinia 20, RVSTICA 26, Abracadabra 28, Decree 32.
- **Which font takes which part is not chosen:** one as the basis of the logo splash, one for important text
  and menus, one for small functional text. The editor's sheets use Ironquill; that is the nearest thing to a
  choice so far.
- **Licences, as read 2026-11-18:** Abracadabra, Darinia, RVSTICA (Valiegraphie): use in a sold game and
  changing them allowed, the files not to be passed on, credit welcome. Antiquity Print: CC BY 4.0, credit
  required. Troll (somepx): credit asked. Decree: no licence file in his folder, to be read before it ships.
  **The font files are in `Art/fonts/`, which is off GitHub;** how they reach the build without being passed on
  (baked into a glyph sheet, or packed) is a thing to settle with him.

## 10. Sharpness and scale

- **Whole numbers only.** Drawn at its own size and every pixel doubled, tripled or quadrupled; never a
  fraction.
- **The game's interface, decided 2026-11-18:** mocked up on 640 x 480, its pieces each anchored to a corner or
  edge, "2x @ 1080p, 3x @ 2k, and 4x @ 4k". The room it is laid out in is then 960 x 540, 853 x 480 and
  960 x 540, and 853 x 533 on his laptop (2,560 x 1,600 at x3). Screens under 1,080 tall are open.
- **The editor is mocked up at 1,280 x 800** (his choice, 2026-11-19, asked which of 1,280 x 800 and 1,280 x
  720). That is his laptop at x2; the interface's rule would have left the editor 853 x 533 there, small for an
  editor, so the editor's scale is its own. *Counsel's reading, not yet his word:* the editor is shown at the
  largest whole scale that leaves at least 1,280 x 800 of room: x2 on his laptop and on a 4K screen, x1 on
  1,920 x 1,080 and 2,560 x 1,440. What the editor does on a screen with less room than that is open.

## 11. Behaviour: his, a convention at a time

Not 95's thousand small rules; the ones he notices, added one by one as he asks: a menu bar, subwindows with
title bars, a status bar, tabs, OK / Cancel / Apply, Alt-letter menus, Tab between fields, Enter and Esc. Each
is asked of him before it goes in.

The standing habits apply to every piece of it: where each setting, control, window and page goes and what it is
called is asked of him first; a control that doesn't apply yet is greyed, never hidden; nothing visible is a
stand-in he didn't choose. The pages already named: Software map, Tectonic Sim, World Gen, Registrar, reached by
a drop-down at the top (`master/spec.md`, the Editor Suite).

An idea beside it, not asked for: a small desktop window before the game starts, "like dungeon keeper did".

## 12. Where the references are

- `Art/References/editor reference/` (off GitHub with the rest of `Art/`): the control sheet at one screen pixel to one
  sheet pixel, in Ironquill at 15 and in MS Sans Serif, each numbered and clean (680 pixels wide); the drawer's
  source; the probe that printed the appendix.
- Also in `Art/References/editor reference/`, from 2026-11-19: his first mock-up as he pasted it (1,280 x 800: the menus,
  the page's drop-down and the toolbar on one row; a tabbed panel on the left, x 4 to 552; the viewport on the
  right; a status bar); Counsel's example of one page drawn into it, the Tectonic Sim's Ocean plates page, with
  the panel alone as a second picture and its drawer's source. The example's arrangement is a guess for him to
  mark up; his notes on the Tectonic Sim's five pages are in `journal/docs/decisions.md` (the rebuild,
  2026-11-19).
- `Art/References/font reference/`: every font at its true size; title bases; the menu mock-up's pieces; his HUD at three
  scales on three screens.
- *The Windows Interface Guidelines for Software Design* (Microsoft, 1995; on the Internet Archive): spacing,
  when edges are raised or sunken, status bars, subwindows.
- The 98.css demonstration page (jdan.github.io/98.css): every control live and close to exact. It is 98's
  look; the title bar's fade is the main difference from 95's.
- His mock-ups, to come, after he has gone through old's editor (`old/`). They are the truth; this brief is the
  ground under them.

## 13. Open, and whose

| Question | Whose |
|---|---|
| His go on the look itself (a leaning until then) | David, after his mock-ups |
| The price: egui as plumbing against controls of her own; the outer title bar both ways | Builder |
| ~~The size he mocks the editor up at~~ | Answered 2026-11-19: 1,280 x 800 |
| The editor's scale on each screen (Counsel's reading: the largest whole scale leaving 1,280 x 800), and on screens with less room | David, when it is built |
| Where the editor begins: Tectonic Sim, or the frame and look first | David (Builder is asking) |
| The pixel font crisp through egui, or a glyph sheet of our own | Builder, by a test |
| How bought fonts reach the build without their files being passed on | David and Builder |
| The richer list (23): wanted as drawn? | David, in his mock-ups |

## Appendix: the pixels, as Windows' classic painter gives them

Printed by `Art/References/editor reference/edge and mark probe.cs` on 2026-11-19 (`DrawEdge` and `DrawFrameControl` onto a
face-coloured picture; today's system colours read back as 1995's). F face, W white, L light, S shadow, K black.
Inside a check box and a radio button the W is the white of the field.

```
Raised, window kind           Raised, button kind           Sunken
(the window's frame)          (a tab page)                  (a field)
    LLLLLLLLLK                    WWWWWWWWWK                    SSSSSSSSSW
    LWWWWWWWSK                    WLLLLLLLSK                    SKKKKKKKLW
    LWFFFFFFSK                    WLFFFFFFSK                    SKFFFFFFLW
    LWFFFFFFSK                    WLFFFFFFSK                    SKFFFFFFLW
    LWFFFFFFSK                    WLFFFFFFSK                    SKFFFFFFLW
    LWFFFFFFSK                    WLFFFFFFSK                    SKFFFFFFLW
    LSSSSSSSSK                    WSSSSSSSSK                    SLLLLLLLLW
    KKKKKKKKKK                    KKKKKKKKKK                    WWWWWWWWWW

Etched                        Sunken, one line              Push button      Held down (toolbar)
(a group box)                 (a status pane)
    SSSSSSSSSW                    SSSSSSSSSW                    WWWWWWWWWWWK     KKKKKKKKKKKW
    SWWWWWWWSW                    SFFFFFFFFW                    WLLLLLLLLLSK     KSSSSSSSSSLW
    SWFFFFFFSW                    SFFFFFFFFW                    WLFFFFFFFFSK     KSFFFFFFFFLW
    SWFFFFFFSW                    SFFFFFFFFW                    WLFFFFFFFFSK     KSFFFFFFFFLW
    SWFFFFFFSW                    SFFFFFFFFW                    WLFFFFFFFFSK     KSFFFFFFFFLW
    SWFFFFFFSW                    SFFFFFFFFW                    WLFFFFFFFFSK     KSFFFFFFFFLW
    SSSSSSSSSW                    SFFFFFFFFW                    WSSSSSSSSSSK     KLLLLLLLLLLW
    WWWWWWWWWW                    WWWWWWWWWW                    KKKKKKKKKKKK     WWWWWWWWWWWW

Check box, checked   Check box, greyed    Radio, chosen    Radio, empty
    SSSSSSSSSSSSW        SSSSSSSSSSSSW        FFFFSSSSFFFF     FFFFSSSSFFFF
    SKKKKKKKKKKLW        SKKKKKKKKKKLW        FFSSKKKKSSFF     FFSSKKKKSSFF
    SKWWWWWWWWWLW        SKFFFFFFFFFLW        FSKKWWWWKKWF     FSKKWWWWKKWF
    SKWWWWWWWKWLW        SKFFFFFFFSFLW        FSKWWWWWWLWF     FSKWWWWWWLWF
    SKWWWWWWKKWLW        SKFFFFFFSSFLW        SKWWWKKWWWLW     SKWWWWWWWWLW
    SKWKWWWKKKWLW        SKFSFFFSSSFLW        SKWWKKKKWWLW     SKWWWWWWWWLW
    SKWKKWKKKWWLW        SKFSSFSSSFFLW        SKWWKKKKWWLW     SKWWWWWWWWLW
    SKWKKKKKWWWLW        SKFSSSSSFFFLW        SKWWWKKWWWLW     SKWWWWWWWWLW
    SKWWKKKWWWWLW        SKFFSSSFFFFLW        FSKWWWWWWLWF     FSKWWWWWWLWF
    SKWWWKWWWWWLW        SKFFFSFFFFFLW        FSLLWWWWLLWF     FSLLWWWWLLWF
    SKWWWWWWWWWLW        SKFFFFFFFFFLW        FFWWLLLLWWFF     FFWWLLLLWWFF
    SLLLLLLLLLLLW        SLLLLLLLLLLLW        FFFFWWWWFFFF     FFFFWWWWFFFF
    WWWWWWWWWWWWW        WWWWWWWWWWWWW

Scroll arrow, up      Scroll arrow, down    Scroll arrow, left    Scroll arrow, right
    LLLLLLLLLLLLLLLK      LLLLLLLLLLLLLLLK      LLLLLLLLLLLLLLLK      LLLLLLLLLLLLLLLK
    LWWWWWWWWWWWWWSK      LWWWWWWWWWWWWWSK      LWWWWWWWWWWWWWSK      LWWWWWWWWWWWWWSK
    LWFFFFFFFFFFFFSK      LWFFFFFFFFFFFFSK      LWFFFFFFFFFFFFSK      LWFFFFFFFFFFFFSK
    LWFFFFFFFFFFFFSK      LWFFFFFFFFFFFFSK      LWFFFFFFFFFFFFSK      LWFFFFFFFFFFFFSK
    LWFFFFFFFFFFFFSK      LWFFFFFFFFFFFFSK      LWFFFFFFKFFFFFSK      LWFFFFKFFFFFFFSK
    LWFFFFFFFFFFFFSK      LWFFFFFFFFFFFFSK      LWFFFFFKKFFFFFSK      LWFFFFKKFFFFFFSK
    LWFFFFFKFFFFFFSK      LWFFKKKKKKKFFFSK      LWFFFFKKKFFFFFSK      LWFFFFKKKFFFFFSK
    LWFFFFKKKFFFFFSK      LWFFFKKKKKFFFFSK      LWFFFKKKKFFFFFSK      LWFFFFKKKKFFFFSK
    LWFFFKKKKKFFFFSK      LWFFFFKKKFFFFFSK      LWFFFFKKKFFFFFSK      LWFFFFKKKFFFFFSK
    LWFFKKKKKKKFFFSK      LWFFFFFKFFFFFFSK      LWFFFFFKKFFFFFSK      LWFFFFKKFFFFFFSK
    LWFFFFFFFFFFFFSK      LWFFFFFFFFFFFFSK      LWFFFFFFKFFFFFSK      LWFFFFKFFFFFFFSK
    LWFFFFFFFFFFFFSK      LWFFFFFFFFFFFFSK      LWFFFFFFFFFFFFSK      LWFFFFFFFFFFFFSK
    LWFFFFFFFFFFFFSK      LWFFFFFFFFFFFFSK      LWFFFFFFFFFFFFSK      LWFFFFFFFFFFFFSK
    LWFFFFFFFFFFFFSK      LWFFFFFFFFFFFFSK      LWFFFFFFFFFFFFSK      LWFFFFFFFFFFFFSK
    LSSSSSSSSSSSSSSK      LSSSSSSSSSSSSSSK      LSSSSSSSSSSSSSSK      LSSSSSSSSSSSSSSK
    KKKKKKKKKKKKKKKK      KKKKKKKKKKKKKKKK      KKKKKKKKKKKKKKKK      KKKKKKKKKKKKKKKK

Number field, up     Number field, down
(15 x 9)             (15 x 8)
    LLLLLLLLLLLLLLK      LLLLLLLLLLLLLLK
    LWWWWWWWWWWWWSK      LWWWWWWWWWWWWSK
    LWFFFFFFFFFFFSK      LWFFFFFFFFFFFSK
    LWFFFFFFFFFFFSK      LWFFFFKKKFFFFSK
    LWFFFFFKFFFFFSK      LWFFFFFKFFFFFSK
    LWFFFFKKKFFFFSK      LWFFFFFFFFFFFSK
    LWFFFFFFFFFFFSK      LSSSSSSSSSSSSSK
    LSSSSSSSSSSSSSK      KKKKKKKKKKKKKKK
    KKKKKKKKKKKKKKK

Title bar: close      maximise              minimise              Size grip (14 x 14)
(16 x 14)
    WWWWWWWWWWWWWWWK      WWWWWWWWWWWWWWWK      WWWWWWWWWWWWWWWK      FFFFFFFFFFFFFF
    WLLLLLLLLLLLLLSK      WLLLLLLLLLLLLLSK      WLLLLLLLLLLLLLSK      FFFFFFFFFFFFFF
    WLFFFFFFFFFFFFSK      WLFKKKKKKKKKFFSK      WLFFFFFFFFFFFFSK      FFFFFFFFFFFFFW
    WLFFKKFFFFKKFFSK      WLFKKKKKKKKKFFSK      WLFFFFFFFFFFFFSK      FFFFFFFFFFFFWS
    WLFFFKKFFKKFFFSK      WLFKFFFFFFFKFFSK      WLFFFFFFFFFFFFSK      FFFFFFFFFFFWSS
    WLFFFFKKKKFFFFSK      WLFKFFFFFFFKFFSK      WLFFFFFFFFFFFFSK      FFFFFFFFFFWSSF
    WLFFFFFKKFFFFFSK      WLFKFFFFFFFKFFSK      WLFFFFFFFFFFFFSK      FFFFFFFFFWSSFW
    WLFFFFKKKKFFFFSK      WLFKFFFFFFFKFFSK      WLFFFFFFFFFFFFSK      FFFFFFFFWSSFWS
    WLFFFKKFFKKFFFSK      WLFKFFFFFFFKFFSK      WLFFFFFFFFFFFFSK      FFFFFFFWSSFWSS
    WLFFKKFFFFKKFFSK      WLFKFFFFFFFKFFSK      WLFFKKKKKKFFFFSK      FFFFFFWSSFWSSF
    WLFFFFFFFFFFFFSK      WLFKKKKKKKKKFFSK      WLFFKKKKKKFFFFSK      FFFFFWSSFWSSFW
    WLFFFFFFFFFFFFSK      WLFFFFFFFFFFFFSK      WLFFFFFFFFFFFFSK      FFFFWSSFWSSFWS
    WSSSSSSSSSSSSSSK      WSSSSSSSSSSSSSSK      WSSSSSSSSSSSSSSK      FFFWSSFWSSFWSS
    KKKKKKKKKKKKKKKK      KKKKKKKKKKKKKKKK      KKKKKKKKKKKKKKKK      FFWSSFWSSFWSSF
```

A drop-down's button is the scroll arrow, down, in a button 16 wide and as tall as its field less 4, the arrow
kept in the middle. With Ironquill the title bar's buttons are 16 x 16 and their marks a pixel heavier; the
probe prints that one too.
