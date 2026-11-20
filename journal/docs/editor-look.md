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

## Where we stand (the shared page, kept by Counsel; last brought up to date 2026-11-20)

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
| **The mock-ups are handed to Builder: four pictures** (the Registrar, the Software map, the Tectonic Sim's Ocean plates, World Gen's Climate), in `Art/95gui mockups/current/`, to be studied "for the rules and conventions" | to Builder: "please take a moment to study these for the rules and conventions." | 2026-11-20 |
| **The first job is egui out:** the debug panel replaced, and the Software map | to Builder: "lets start the day by kicking e-gui to the curb ... we'll just get started by replacing the debug panel, as you see fit, using tabs and the collapsing sections, as well as the software map." | 2026-11-20 |
| The debug panel's layout is Builder's, with tabs and folding sections (this overtakes his word of 2026-11-19 that a mock-up of his own would come first) | "as you see fit, using tabs and the collapsing sections" | 2026-11-20 |
| Pages begin as blank panels in the window; buttons and features go in one at a time | to Builder: "we'll start with blank panels in the window, and add buttons and features one at a time." | 2026-11-20 |
| Old's parameters come across one by one, each weighed, the process remade to reach the same results, more refined | to Builder: "all the parameters and variables from the old incus will be evaluated singly alongside a recreative process that should get the same results, just mroe refined." | 2026-11-20 |
| The Software map: the 95 scheme on teal; every module the same white box, in groups; every wire dark grey behind the boxes; the clicked box's wires blue and on top; a card panel on the left | to Counsel, then his picture (`current/software map.png`, Counsel's example 10 taken whole) | 2026-11-20 |
| The Registrar's library: Find, a Type drop-down, a Sort drop-down, a count and an "unregulated only" tick over one list; no tags in the filter; rows 40 tall with one 32 x 32 thumbnail, the name, and a second line that differs by type | his picture (`current/registrar.png`: the third of Counsel's four ways, example 9 C) | 2026-11-20 |
| The toolbar's first pieces, on the menu's row after the page's drop-down: Save, Load, a presets drop-down ("Default"); in the Tectonic Sim also four buttons not yet named, and at the right a Feed slider and a Seed field | his pictures (`current/tectonics.png`, `current/world gen.png`) | 2026-11-20 |
| Folding sections are the bars as drawn (a plus box and a name on a raised bar) | his picture (`current/world gen.png`) | 2026-11-20 |
| The wheel does nothing over the Software map | to Builder: "the mouse wheel shouldn't do anything over the software map." | 2026-11-20 |
| The title's lettering, where it has to be written: Darinia | to Builder: "If you need to write INCUS ARTUS in the upper left, the font is darinia." | 2026-11-20 |
| Describing text is left out everywhere but the Software map (so the text at the head of the Ocean plates panel goes; the Software map's card keeps its "What it does") | to Builder: "the describing text in general should be omitted, except from the software map." | 2026-11-20 |
| **Standing: his drawings are compared against the control sheets, and he is asked about every difference**, "in a pixel, a shade, a line, a spacing, etc." Neither the picture nor the sheet wins unasked (this sharpens "his mock-ups overrule this brief", above) | to Builder: "make sure that you compare my drawings against the stylesheets and ask if mine differs from it, in a pixel, a shade, a line, a spacing, etc." | 2026-11-20 |
| The title's three buttons are his: a pixel of navy between each, the marks evenly spaced inside (Builder: his minimise bar sits 2 from the left of its face and 4 from the right, so she centres it and tells him; the square and the X as drawn) | to Builder: "1. go with mine, with the boxes evenly spaced and the icons evenly spaced inside the boxes." | 2026-11-20 |
| The Tectonic Sim's panel sits at World Gen's height, and its chosen tab gets its top edge back | to Builder: "2. conform to world gen height"; "2b. replace the lost edge" | 2026-11-20 |
| Sliders carry three tick marks: one at each end and one at the centre (neither the picture's none nor the sheet's twelve) | to Builder: "3. please a tik mark only at the ends and the center of the sliders." | 2026-11-20 |
| Number boxes are plain, without the small arrows. A click in one selects its whole contents, typing replaces them, and Enter keeps the number and leaves the box | to Builder: "3b. no arrows, scroll wheel or typing is enough. speaking of which, clicking in the number box, should highlight the contents, and enter should 'leave' the number box as is, so you can click, enter numbers, press enter. some programs have annoying behaviors where you click in there and always get a cursor after the decimal or in the middle and have to fuss with it or get used to just clicking and pressing ctrl+a to select it all." | 2026-11-20 |
| Radio buttons sit where the sheet puts them (y + 1) | to Builder: "4. go with default" | 2026-11-20 |
| Lists: rows 15 apart, words 4 in (the sheet's) | to Builder: "5. go with 15 and 4" | 2026-11-20 |
| **Standing: everything as tight as it can go, by one consistent logic for gaps and borders; no dead space, nothing unnecessary on screen** | to Builder: "we should always squeeze things as tight as possible while maintaining a consistent logic for gaps and borders. we don't want dead space or anything unnecessary on the screen." | 2026-11-20 |
| ~~The viewport's frame is his (raised: white, light and face at top and left; 808080, 3A3A42, 282830 at bottom and right).~~ **Overtaken by his own word later the same day** (the frame's row below). Standing from it: the magenta is the viewport's own picture, an exception to the 95 style | to Builder: "6 go with my version. obviously the magenta is part of the actual viewport which is an exception to our 95 style." | 2026-11-20 |
| Whatever a display shows is centred in it, in the status bar and in the second bar from the top | to Builder: "7. just center anything in those displays in the bottom bar or in the 2nd from top bar" | 2026-11-20 |
| **egui goes all at once**; the new system from the first build (this overtakes "today's F3 panel stays until then", 2026-11-19) | to Builder: "you can just get rid of it all at once, there's no need to go in and use it except to check it in the near future, so we can just start with the new system when you're ready." | 2026-11-20 |
| The debug panel is in the game only, never in the editor. Builder's suggestions for it taken: it starts from example 4, with its scale and "game in a window" on its Overview tab | to Builder: "the debug panel should not appear in the editor, it should just be in the game. yes, go with your suggestions for it." | 2026-11-20 |
| The menus, to start: File has Return to game and Quit; Edit has Settings, greyed; Window has the scale and windowed menus. A tooltip mode: the cursor becomes a question mark, tooltips show over everything, any key ends it (put as a question; its menu not named). Builder is to suggest what else could live in the menus instead of elsewhere, to save space | to Builder: "for the menus, i know we don't have much to put in there now, but yeah, try to suggest things to me that can go there instead of somewhere else, to save space. return to game and quit under file, greyed out 'settings' under edit, the scale and windowed menus under window, and tooltip mode that turns the cursor into a question mark to reveal tooltips over everything, any key to exit? im sure they'll fill up over time." | 2026-11-20 |
| The page's drop-down is 128 wide on every page | to Builder: "2. 128" | 2026-11-20 |
| The viewport's frame ends at x 1,276 on the right, level with everything else; the Software map's two dark lines outside its sunken frame are replaced (Builder's reading, put to him "say if not": they become the window's grey and the sunken frame runs to 1,276) | to Builder: "3. pull it in, replace them" | 2026-11-20 |
| The temperatures in the editor sit at the right end of the status bar (in the game they stay top right, in our letters) | to Builder: "4. put them at the bottom bar, to the right." | 2026-11-20 |
| **The viewport's frame is Windows 95's thick border frame in the standard colours, the same on the Tectonic Sim and World Gen** whatever picture is inside (this overtakes "go with my version"). Sharpened by his answer 7, the next row but one: sunk, and on the Software map's board as well; Builder's A and B, both raised, fall away | to Builder: "6. im not sure what i was trying to do, but derive win95's thick borderframe logic and use the standard colors, make sure its the same between the tectonics and the world gen, even though they show different pictures" | 2026-11-20 |
| A chosen tab is no taller than the rest; it is drawn in front of its neighbours' borders (Builder: covering 2 pixels of each; where it has no neighbour it doesn't reach out, so the row stays at x 4 and "2 pixels in" falls away) | to Builder: "1. make it not get taller, just be up front as re the borders between neighbors." | 2026-11-20 |
| **One frame for the three views: thick and sunk**, on the Software map's board, the Tectonic Sim's viewport and World Gen's (against Builder's and Counsel's, who had the Software map keep its plain sunken frame). Which order its rings go in is under Open | to Builder: "7. make it thick frame, sunk, on both the software map board, the tectonic viewport, and the world gen viewport." | 2026-11-20 |
| The thick sunk frame's rings: the sunken edge outside, then two face rings (S/W, K/L, face, face) | picked in Builder's question box: "Sunken edge outside (Recommended)" | 2026-11-20 |
| **Standing: at least 2 pixels between elements** (with his rule of the same day, as tight as possible: so 2 is the gap wherever nothing else sets it). Builder's reading of it for the row of tabs, told to him: the row moves up 2 (tabs at y 50 to 70, the page from 71), leaving exactly 2 pixels of face (48 and 49) under the menus' etched line, the same as above the frames; this reverses "conform to world gen height" for the row | asked whether the row should move up 2: "keep at least a 2 pixel border between elements" | 2026-11-20 |
| The status bar has 2 pixels of spacing round it, as a panel set into the window. Builder's reading, told to him: the page's bottom comes down to y 772, level with the frames, leaving 773 and 774 above the status bar, as on its other sides | asked whether the page's bottom should come down 2: "give the bottom bar 2 pixels of spacing where its a panel element integrated into the window." | 2026-11-20 |
| The debug panel starts as example 4 draws it, by his approval of Builder's suggestions: "Debug [F3]" in Ironquill on its title bar; tabs Overview, Camera and Render, none rising (his tab rule) | to Builder: "yes, go with your suggestions for it." | 2026-11-20 |
| **Standing: every row as dense as possible, nothing touching within 2 pixels.** Tabs first. Builder's readings, built into 228: a tab is its name's ink plus 2 clear of its two-pixel edges on each side (the sheet gave 7 a side); the status bar's small panes have their words 2 clear of their line and the last pane 2 clear of the grip; the Software map's boxes are to follow at build 230 (his own picture draws them 7 a side: his to be asked there) | to Builder, during the build of 228: "you dont need so much spacing between names and bars, and it doesnt necessarily need to be quite so wide"; "just go a shade denser were nothing touches within 2px"; asked which names and bars: "Tabs, generally all rows should be as dense as possible and you can narrow the whole by about a quarter" | 2026-11-20 |
| "Narrow the whole by about a quarter": read by Builder as the debug panel, 303 wide in 228 where example 4 had 404 (the width was Counsel's example's, not his). He may mean the editor's tabbed panel (549, his own drawing): she points it out at his look. **At the look he chose neither** ("dont worry about it right now"): the debug panel stays 303 as built and the question rests | to Builder: "you can narrow the whole by about a quarter" | 2026-11-20 |
| The 2 pixels under the menus' etched line stand: the tabs and the Software map's board sit 2 below it (Builder's reading of "keep at least a 2 pixel border between elements", confirmed) | at his look at build 228, to her second point: "yes" | 2026-11-20 |
| The window gets its black outer line on the right and bottom, which his pictures lack (they are a 1,281 x 801 window cut to 1,280 x 800); drawn whole, everything on the right and bottom comes in a pixel. In build 229 | at his look at build 228, to her third point: "yes" | 2026-11-20 |
| The X on the editor's title bar quits, the same as File > Quit and Alt+F4 | to Builder's first question for 229: "1. x quits" | 2026-11-20 |
| The middle button follows Windows: the restore mark (two small windows) while the editor fills the screen, the maximise mark once the window is smaller | to her second: "2. follow windows" | 2026-11-20 |
| The menus take Builder's additions: Edit > Undo (Ctrl+Z) and Redo (Ctrl+Y); View > Status Bar and Parameters Panel, ticks that hide or show; Help > What's This? (Shift+F1), which is his tooltip mode, and Keys.... Anything not working yet is greyed | to her third: "3. yes" | 2026-11-20 |
| Editor Scale: Auto and 1x to 5x, a scale the screen can't hold greyed. Panel Scale: greyed until the panel has its first controls, its rules settled with him then | to her fourth: "4. yes" | 2026-11-20 |
| **No wrapper: Windows' own frame never shows round ours, in any mode; the whole window is 95. And the scale never changes with the window's mode or size.** Builder's account of what he saw in 228: F11 gave a maximised window with Windows' title bar, and Auto, worked out from the window's size, fell from 2x to 1x; in 229 Auto comes from the screen's full size | to Builder: "we do need to make sure that the game window doesn't scale inside another window. there should be no wrapper at all, so the entire thing is 95gui. when i maximized it last time, it changed scale." | 2026-11-20 |
| While the editor fills the screen (maximised above the taskbar, or by F11) its border stays all round, with the black line, as his pictures have it; not Windows 95's way of pushing it past the screen's edges | to Builder: "1 border all around" | 2026-11-20 |
| While the editor fills the screen the size grip is gone; a smaller window keeps it. What fills its place he didn't say: Builder's reading, from her question and in 229's pass mark, is that the small panes move 14 into its room and "Ready" is 14 longer; his to object to at the look | to Builder: "2 grip gone"; then "proceed" | 2026-11-20 |

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
- **They came on 2026-11-20, and his word that day changes her order:** the first job is egui out, the debug
  panel and the Software map, where her order above had the debug panel last. She is briefing him ("give me
  a briefing on how we'll carry out the replacement, any details i should be deciding on, and how you're going
  to set up our windows95 drawing") and builds nothing until he answers it. Her new order is hers to write
  here once he has.
- **Her order as briefed to him, 2026-11-20 (proposed; not yet his go):** build 228 the bare frame; 229 the
  frame's behaviour; 230 the Software map; 231 to 234 the debug panel, a tab at a time; 235 egui out.
- **What she asked him in the briefing** (his answers will be set down above as they come):
  1. Keep egui's F3 panel working until ours has everything, or go blank from the first build?
  2. The title's lettering: cut from his picture, or a font he names?
  3. The debug panel from example 4 as a starting point: does it also float over the editor, and may its
     scale setting and the game-windowed setting sit on its Overview tab?
  4. The menus to start: Exit under File; the two scales and Full screen under View; Game (F2) and Debug
     panel (F3) under Window; Edit and Help greyed.
  5. The Software map exactly as his picture, the hover tooltips gone since the card replaces them.
- The handover's other questions (the wheel over the Software map, the debug panel's title bar font, the
  Ocean plates' describing text, Save held down, the Tags field) she will put to him at the build each
  belongs to, one or two at a time.
- **He answered the briefing on 2026-11-20** (his words are in "Decided by him", above): 1, egui out all at
  once; 2, Darinia (she takes his lettering as a picture); 3, the debug panel in the game only, her suggestions
  taken; 4, the menus as he named them; 5 stands as she put it. He may want to look at the old panel "in the
  near future": she is offering to run build 227 from a folder of its own, the way `old/` runs.
- **Her order now, since egui goes at once (proposed; his only word on it is "start with the new system when
  you're ready"):** build 228, egui out, with the frame, the blank pages, the blank floating debug panel and
  the temperatures; 229, the frame working, and the menus; 230, the Software map; 231 to 234, the debug
  panel's tabs. Nothing is built or committed yet.
- **The menus as she sketched them to him (hers where starred; unanswered):** File: Return to game (F2),
  Quit (Alt+F4). Edit: Settings... greyed; Undo* (Ctrl+Z) and Redo* (Ctrl+Y), greyed. View: Status Bar* and
  Parameters Panel*, ticked. Window: Editor scale, Panel scale, Full screen (F11). Help: What's This?*
  (Shift+F1), his tooltip mode; Keys...*. Later, maybe: Save Preset (Ctrl+S) and Save Preset As... under
  File instead of the top bars' Save and Load, decided when the Tectonic Sim is built. The game's "in a
  window" stays on the debug panel's Overview.
- **Her six questions to him, numbered as he saw them (his answers will come back by these numbers;
  unanswered; each is hers until he answers):**
  1. Her row 3, the first tab when chosen. Hers: start the row of tabs 2 pixels in.
  2. Her row 10, the page's drop-down's width. Hers: 128 everywhere.
  3. His frame's two pixels: its outer line on the right at x 1,277, one past everything else (hers: pull it
     to 1,276, so the picture inside is 716 wide); and the Software map's two dark lines (hers: leftovers,
     drop them, the sunken frame to 1,276).
  4. The temperatures in the editor. Hers: a pane at the right end of the status bar; in the game they stay
     top right, in our letters.
  5. Tooltips outside his mode. Hers: none; inside it, a click also ends the mode, without pressing anything.
  6. The frame's shades, 3A3A42 and 282830: keep them (hers, as she reads "go with my version"), or black for
     both, the nearest in the scheme. Put after Counsel's question: her table's row 8 had told him "raised,
     three dark lines, two in shades outside the scheme", but the picture's item 6 was captioned only "the
     viewport's frame, bottom right corner", and he answered by the picture.
- **What she told him she takes as he said, "answer only if wrong" (her readings; his to object to):**
  - The minimise bar centred, 1 pixel right of his drawing; the square and the X as drawn.
  - Three tick marks drawn like the sheet's; nothing moves.
  - The number box: his click, type, Enter; and, hers, Esc restores the old number, a click elsewhere keeps
    what was typed, and a second click places the cursor. What happens to a number typed past a slider's
    ends is held for the number box's build.
  - The centring: words centred in the status bar's small panes and in the Seed box; the long "Ready" pane
    stays left, as the message line; drop-downs keep their words left, in line with their lists.
  - His spacing rule as standing.
- She writes build 228's pass mark and starts once he has answered 1 to 6.
- **His answers to the six, 2026-11-20** (his words are in "Decided by him" and `journal/docs/decisions.md`):
  1 came back as a question, "i don't think a chosen tab should 'grow', should it? it should just leaf to the
  front of any adjacent tabs", and she is answering it with a picture; 2, 128; 3, "pull it in, replace
  them"; 4, the status bar's right end; 5, not answered, and she has told him it can wait until the menus
  are built; 6, Windows 95's thick border frame in the standard colours, the same on both pages.
- **Her picture of the two choices left** (`Art/References/editor reference/choices - the chosen tab and the viewport frame (2026-11-20).png`;
  hers; awaiting his pick):
  - The chosen tab. In Windows 95 it does grow: 2 pixels taller and covering 2 pixels of each neighbour,
    which is how it comes to the front; his own pictures already draw it so (Climate, Ocean plates). Three
    ways for the first tab: Windows' way with the row where he drew it (the first tab pokes past the panel's
    edge); Windows' way with the row 2 pixels in (hers); not growing.
  - The viewport's frame, both at x 555 to 1,276, y 50 to 772. A (hers): the thick frame, the same as the
    window's own edge, rings from outside in L/K, W/S, face, face; the picture inside 714 x 715. B: the
    raised edge and one face ring (W/K, L/S, face), a pixel thinner and nearer his drawing; the picture
    inside 716 x 717.
- **His answers to 1 and 7, the same day, and her second picture** (`Art/References/editor reference/choices 2 - the tab in front, the row up 2, the sunk frame (2026-11-20).png`;
  hers; awaiting his word on three things):
  - The tab, as she drew it from his answer: the chosen tab as tall as the rest, in front of its neighbours'
    edges, covering 2 pixels of each.
  - With nothing rising, the 2 rows above the tabs are empty. Hers: move the row up 2 (tabs at y 50 to 70, the
    page from 71), where his Tectonic Sim drawing had them, the tabs' tops in line with the frame's top. This
    reverses his "conform to world gen height" of that morning for the row, since the rise it made room for
    is gone.
  - The page's bottom is at y 770 in both pictures, 2 above the frame's 772. Hers: take it down to 772, so
    both sit 2 pixels above the status bar.
  - "Thick frame, sunk", two readings: hers, the sunken edge outside and then two face rings (S/W, K/L, face,
    face), the thick frame's own order with its edge sunk; or two face rings and then the sunken edge (face,
    face, S/W, K/L), which reads as the ordinary sunken frame with a wider gap. All three boxes run y 50 to
    772; the views x 555 to 1,276 (inside 714 x 715); the board x 306 to 1,276 (inside 963 x 715, its
    scrollbar inside the frame).
- **He answered the three the same day, in a question box** (his words in "Decided by him"): the row up 2 and
  the page's bottom to 772 by her readings of his two spacing answers, told to him as readings; the sunken
  edge outside, picked.
- **Everything build 228 needs is decided (2026-11-20).** She writes 228's pass mark, then builds, and tells
  Counsel before she hands him a launch. Open for later builds: tooltips outside his mode (her 5), her menu
  suggestions, and numbers typed past a slider's ends.
- **Build 228's pass mark** is in `journal/docs/experiments.md` (812312d, hers). Two things in it that are
  hers, not his word:
  - The Registrar is drawn blank in 228: an empty page, no panes. His picture's panes don't follow the
    sheet: the left pane has a raised edge on three sides and none on the right, and the panes are split by
    2-pixel bars with odd caps (x 260 to 261 and 875 to 876). She puts those to him when the page is built
    (3.3), not now.
  - The debug panel takes example 4's place and size (at 16, 16; 404 x 432), but its inside follows his
    rules of 2026-11-20 and not the example's 4-pixel margins: the tabs 2 pixels under the title bar, and
    the page flush with the window's own grey band, as the editor's pages are.
- **Build 228 built (f9a8376, hers; her entry "Build 228 built" in `journal/docs/experiments.md`).** His
  words during the build made the tabs and the status bar's panes denser and the debug panel 303 wide (rows
  in "Decided by him"). The page's drop-down sits 10 pixels after "Help", as in his pictures (her first
  drawing had 12). Her comparison: on all four pictures every differing pixel is in a place named with its
  ruling, and none differs anywhere else; all 68 tests pass.
- **His look at build 228, 2026-11-20** (her entry "His look at build 228" in `journal/docs/experiments.md`,
  9dc9464): "1. dont worry about it right now 2. yes 3. yes" to her three points, in order: whether "the whole" was the debug
  panel or the tabbed panel; the 2 pixels under the menus' line; the window's black outer line on the right
  and bottom. He said nothing on anything else in 228. The session ran 95 minutes and closed normally, no
  warnings or errors (`logs/session-1795194177.log`). Next is 229, the frame working and the menus: she is
  reading this page for what it still needs from him before its pass mark.
- **Launches from 229 on (hers, at Counsel's suggestion):** the compile and the tests run as their own steps
  once the build's commit is in; she launches when the CPU area reads under 80 C and tells him the wait. 228's
  log read 105 C at 3 s and 61 s and 104 C at 108 s, the compile running straight into the making of the
  world, the same pattern as 104 C on 2026-11-18.
- **Build 229's four questions, put to him 2026-11-20 (hers; answered the same day: "1. x quits 2. follow
  windows 3. yes 4. yes", in "Decided by him"; nothing built):**
  1. The X on the editor's title bar. Hers: it quits, Windows 95's way, the same as File > Quit and Alt+F4.
     The other way: it returns to the game, like F2.
  2. The middle button while the editor fills the screen above the taskbar. Hers: Windows 95's restore mark
     (two small windows), and the maximise mark once the window is smaller; his pictures show the maximise
     mark. The border stays in view either way, by his "yes" to the black line (Windows 95 would push it off
     the screen's edges).
  3. The menus, what he named and her additions (starred): File: Return to Game (F2), Quit (Alt+F4). Edit:
     Settings..., Undo* (Ctrl+Z), Redo* (Ctrl+Y). View: Status Bar*, Parameters Panel* (ticks that hide or
     show). Window: Editor Scale, Panel Scale (each a submenu), Full Screen (F11). Help: What's This?*
     (Shift+F1, his tooltip mode), Keys...*. Anything not working yet is greyed. Hers: take all the starred
     ones, which also places his tooltip mode. Her question 5 (tooltips outside the mode) waits for the
     build that brings tooltips.
  4. The scales. Editor Scale: Auto and 1x to 5x, the debug panel's list; Auto is the largest whole scale at
     which 1,280 x 800 fits the screen; a scale too large for the screen to hold the frame is greyed. Panel
     Scale: greyed until the panel has its first controls, its rules settled with him then.
- **Hers for 229's pass mark, told to him as hers:** the drag, resizing by the edges and the grip,
  double-click on the title bar to fill the screen, the smallest size. Two things he will see: windowed
  above the taskbar the editor is shorter than his pictures by the taskbar's height (F11 gives the full
  1,280 x 800); and F11 works only in the editor until the game has its windowed setting.
- **Build 229 built (3ac63d6, hers; pass mark df165f3; her entry in `journal/docs/experiments.md`).** What
  is in it, by her account:
  - No Windows frame in any mode; Windows 11's rounded corners and coloured edge are off. Auto's scale comes
    from the screen, never the window. The black outer line is in.
  - Three ways of being shown: filling the screen above the taskbar; a smaller window (the first time, three
    quarters of the screen, centred); full screen (F11, the editor only). F2 brings it back the way it was
    left.
  - The title buttons act when let go on them; the middle one wears the restore mark while the editor fills
    the screen; the X quits. The title bar moves the window, and a double click on it fills the screen or
    makes it smaller. The edges and the grip size it, with the matching pointer. None of that works while it
    fills the screen.
  - While it fills the screen: the border stays, the grip goes, and the small panes move 14 into its room.
  - The menus as he took them. Working: Return to Game, Quit, Status Bar, Parameters Panel (greyed on the
    Software map and the Registrar), Editor Scale (Auto, 1x to 5x; 4x and 5x greyed on the laptop), Full
    Screen. Greyed: Undo, Redo, Settings..., Panel Scale, What's This?, Keys.... Drawn by Windows 95's
    recipe, with Wine's numbers sized to Ironquill's line.
  - The smallest frame is 739 x 136 sheet pixels.
  - Tests: all pass (75, 2 ignored). His pictures differ only in 228's ruled places.
  - Two things she raises at his look: every menu keeps Windows' tick column and arrow column even where
    nothing uses them, against his "no dead space"; and the menus' look itself.
- **His look at build 229, 2026-11-20** (her entry "His look at build 229" in `journal/docs/experiments.md`,
  5f8255e): "nice", after eight minutes in it. He had been given the first two steps: F2 (the editor filling
  the screen above the taskbar in its own frame, the restore mark, no grip), then the menus (sliding along
  the row, and View's two ticks). It is his only word on 229 so far. She has asked him two things: whether
  he got to the smaller window (moving, sizing) and F11, with another launch offered; and the menus' empty
  columns. The plan's line for 229 stays unticked until he says, since it names drag, resize, maximise and
  F11. The session: 8 minutes, quit, no warnings or errors, memory level (`logs/session-1795214649.log`).
- **The cooler launch, tried at 229, and what it showed (corrected in the open: Counsel's suggestion above
  took little off the peak).** She launched with the CPU area at 78 C; it still read 103 C at 2.5 s and at
  60 s, against 228's 105 and 105. So the making of the world at the start takes it past 100 by itself. Where
  it did differ: at about 105 s, 92 C against 228's 104. She keeps the habit, since it costs him little. A
  cooler start would be a change to the loading (fewer threads making the world at the start), his call; it
  has not been proposed to him by her.

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
| Which panel scales count as unavailable at each editor scale (the rule behind the greying); what "reset" sets the panel's scale to; ~~whether the editor's scale has the debug panel's steps (1x to 5x and Auto)~~ (answered 2026-11-20: yes, Auto from the screen's full size; the panel's rules wait for its first controls); what Auto means for the debug panel; ~~where the debug panel's scale setting sits~~ (answered 2026-11-20: its Overview tab) | Builder to put to him when the menu is built; David |
| ~~What the wheel does when the pointer is inside a number box~~ | Answered 2026-11-19: it steps the number up or down (the row in "Decided by him") |
| The wheel's step in a number box (how much one notch changes each setting); whether the new viewports keep old's zooms as they were ("the zoom was ..."), ~~what the wheel does over the Software map~~ (answered 2026-11-20: nothing); what it does over the other pages' views | Builder to propose from old; David |
| The Software map: ~~black and magenta as now (example 6), or the 95 scheme on teal (example 7); every wire drawn dimmed, or only the chosen box's~~ answered by his mock-up and his words of 2026-11-20 (the 95 scheme on teal; every wire dark grey behind the boxes, the chosen box's blue on top). The card keeps its describing text, by his word of 2026-11-20 ("except from the software map"); example 10 he took whole into his own picture | David |
| ~~Where the two settings go: the game windowed; the scale~~ | Answered 2026-11-20: the editor's "scale and windowed menus under window"; the debug panel's scale and "game in a window" on its Overview tab ("yes, go with your suggestions for it"). What each is called on screen is still his |
| Still wanted from him, by Builder's answer to this brief: ~~the mock-ups as lossless 1,280 x 800 PNGs~~ (they came 2026-11-20, in `Art/95gui mockups/current/`); ~~the title's lettering~~ (answered 2026-11-20: Darinia; Builder finds his own lettering set closer than the font sets it, below); the toolbar's four unnamed buttons; what fills the status bar's panes; ~~the open menus~~ (begun 2026-11-20: File, Edit and Window as he named them; the rest as they fill); which of old's viewport readouts go | David |
| Screens with less room than 1,280 x 800 | David, near shipping |
| ~~A smaller font for a floating window's short title bar (Ironquill has one size; Antiquity Print at 13 is his smallest), or the full-height bar~~ | Answered for the debug panel 2026-11-20, by his approval of example 4: Ironquill on its title bar. Open again if another floating window wants a shorter bar |
| Tooltips: old's descriptions run to five lines and a 95 tooltip was one short line: wrapped tooltips, or the texts cut down. His tooltip mode of 2026-11-20 (a question-mark cursor that shows tooltips over everything) is where they will show: ~~which menu it sits in~~ (Help > What's This?, Shift+F1, 2026-11-20); whether any tooltip shows outside it (Builder reads none); whether a click ends it as well as a key | David; Builder has asked |
| ~~Folding sections: the bars as drawn, or a tree that shows one section at a time~~ | Answered by his mock-up, 2026-11-20: the bars as drawn |
| Where the Field size, pixel size and bake controls of old's top bar go in World Gen (Counsel assumed a Main tab) | David |
| ~~The describing text at the head of the Ocean plates panel in `current/tectonics.png`: kept, or a leftover?~~ | Answered 2026-11-20: left out ("the describing text in general should be omitted, except from the software map") |
| "Save" is drawn held down (checkered) in both toolbars: a state it has, or an accident of pasting? | David |
| The Registrar: the Tags field is still in the dock though the library no longer filters by tags: are tags kept on an asset for later, or gone? | David |
| Ironquill reaching the build: its licence forbids the font's files in a public repository (a built game may be public). Is the GitHub repository private? Until he says, the sheet stays off GitHub and the game reads it from a folder on this machine (Builder's plan) | David |
| The debug panel: its tabs and sections (Builder to propose from today's F3 panel and example 4, which he took as the start, 2026-11-20); ~~its title bar's font~~ (Ironquill, as example 4); ~~where its scale setting sits~~ (its Overview tab); its first tabs Overview, Camera and Render | Builder to propose; David |
| ~~Build 229's four questions: the title bar's X; the middle button's mark; her additions to the menus; the scales~~ | Answered 2026-11-20: "1. x quits 2. follow windows 3. yes 4. yes" |
| ~~Two things Windows 95 hides while a window is maximised, which his pictures show: the border and the size grip~~ | Answered 2026-11-20: "1 border all around 2 grip gone" |
| The menus' tick column and arrow column, kept by Windows even where nothing in the menu uses them: as Windows has them, or trimmed where unused (his "no dead space")? Until he answers they stay | David; Builder has asked |
| Build 229: whether he got to the smaller window (moving, sizing) and F11; another launch offered | David; Builder has asked |
| The start's heat: the CPU area reads 103 to 105 C for the first minute of every launch, while the world is made, whatever it read before. Left as it is, or the world made on fewer threads at the start (a slower load)? | David, if he wants it looked at; not proposed by Builder |
| For later, Counsel's: once presets can be saved, what an X that quits does about unsaved work | David, when saving is built |
| The Registrar's panes in `current/registrar.png` against the sheet: the left pane raised on three sides with none on the right; the panes split by 2-pixel bars with odd caps (x 260 to 261, 875 to 876). Found by Builder; the page is blank until then | Builder to put to him when the Registrar is built (3.3); David |
| ~~Her row 3, the first tab when chosen; whether a chosen tab grows~~ ~~Her row 10, the page's drop-down's width~~ | Answered 2026-11-20: "make it not get taller, just be up front as re the borders between neighbors"; 128 |
| ~~His viewport frame's outer dark line on the right at x 1,277; the two dark lines outside the Software map's sunken frame~~ | Answered 2026-11-20: "pull it in, replace them" |
| ~~His viewport frame's two dark shades, 3A3A42 and 282830, outside the 95 scheme: kept, or the scheme's?~~ | Answered 2026-11-20: the standard colours ("im not sure what i was trying to do, but derive win95's thick borderframe logic and use the standard colors") |
| ~~The temperatures in the editor, once egui (which draws them) is gone~~ | Answered 2026-11-20: "put them at the bottom bar, to the right." |
| ~~The viewport's frame, which drawing: A or B~~ | Overtaken 2026-11-20 by his answer 7 ("thick frame, sunk"): both were raised |
| ~~The Software map's view: its plain sunken frame, or one frame for all three (Counsel's question; Builder's question 7)~~ | Answered 2026-11-20: one frame, "thick frame, sunk, on both the software map board, the tectonic viewport, and the world gen viewport" (Builder and Counsel had both said keep it plain sunken) |
| ~~The thick sunk frame's order of rings~~ | Answered 2026-11-20: the sunken edge outside, then two face rings |
| ~~The row of tabs, now that nothing rises: up 2, or left with 2 empty rows above~~ | Answered 2026-11-20: "keep at least a 2 pixel border between elements"; read by Builder as up 2, which leaves exactly 2. Confirmed at his look at build 228: "yes" |
| ~~The page's bottom: down from y 770 to 772~~ | Answered 2026-11-20: "give the bottom bar 2 pixels of spacing where its a panel element integrated into the window."; read by Builder as down to 772 |
| ~~The menus as Builder sketched them to him~~ (taken 2026-11-20: "3. yes"); her additions to the number box (Esc restores, a click elsewhere keeps, a second click places the cursor) | David, if he objects |
| "The 2nd from top bar": Builder reads it as the bar under the title bar, and told him her reading, "answer only if wrong": words centred in the status bar's small panes and in the Seed box; the "Ready" pane left; drop-downs' words left, in line with their lists | David, if he objects |
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

### Handed over 2026-11-20: `Art/95gui mockups/current/` (four pictures; they are the specification)

His words to Builder with them (relayed by her; whole in `journal/docs/decisions.md`, the rebuild, 2026-11-20):
"Good morning builder! lets start the day by kicking e-gui to the curb. please take a moment to study these for the rules and conventions. we'll just get started by replacing the debug panel, as you see fit, using tabs and the collapsing sections, as well as the software map. please ask any questions about something you're not sure of, and give me a briefing on how we'll carry out the replacement, any details i should be deciding on, and how you're going to set up our windows95 drawing. we'll start with blank panels in the window, and add buttons and features one at a time. all the parameters and variables from the old incus will be evaluated singly alongside a recreative process that should get the same results, just mroe refined."

What each shows, as Counsel reads it (where this page and a picture differ, the picture wins):

- **All four:** the title bar with "INCUS ARTUS" in lettering of his own and "Build 238" before the three
  buttons; the menus (File, Edit, View, Window, Help) and, on the same row, the page's drop-down as a true
  drop-down with its arrow button; a status bar of a wide pane and two small ones.
  - *Corrected by Builder, 2026-11-20:* that holds for three of the pictures. The Tectonic Sim's has three
    small panes: "Land 23%" (x 1,048 to 1,113), "Tick 4,812" (1,118 to 1,182), "N 204, E 88" (1,187 to 1,268).
- **`software map.png`:** Counsel's example 10 taken whole (only the page's drop-down differs). So the group
  captions, the arrowheads, the dotted focus line, navy for the blue and 404040 for the grey, which were
  Counsel's stand-ins yesterday, are in his picture now.
- **`registrar.png`:** the library in the third of the four ways (example 9 C): "Find", "Type" ("All types"),
  "Sort" ("Name"), "412 of 412 shown", an "unregulated only" tick; rows 40 tall. The dock and the palette as
  example 8 had them.
- **`tectonics.png`:** the Ocean plates tab of five (Main settings, Plate settings, Land plates, Ocean plates,
  Simulation), two-line settings in group boxes, the panel's scrollbar; the toolbar with Save, Load, the
  presets drop-down, four unnamed buttons, Feed and Seed.
- **`world gen.png`:** the Climate tab of six (Main, Landmass, Topography, Climate, Mineralogy, Interpreter), a
  "Show over the map" drop-down over folding sections, greyed settings where they don't apply; the toolbar
  with Save, Load and the presets drop-down.

### The handover: what the four pictures don't show (Counsel, 2026-11-20, as he asked: "a prompt from you about any details i forgot")

**1. Left in the pictures that isn't his choice. Ask him; don't copy it.**

- **The viewports' pictures** in the Tectonic Sim and World Gen are pasted from old: the magenta ring and
  line, "YOU ARE HERE", the stray dark columns at the right edge. The viewport's own frame is the clean sunken
  one of the Software map's picture. What goes inside is old's final viewports with the leftover readouts
  dropped (his word of 2026-11-19); which readouts go is open.
- **The window's frame is cut one pixel short** at the right and the bottom in all four (Builder's finding;
  the same in the picture he pasted to Counsel: the last column and row are the grey 808080, the black outer
  line is off the canvas). Build the whole raised edge to section 4's recipe.
- **Every number, name and count** is a stand-in: the values in the fields, the seed, the splines' names,
  "412", "115 colours", the times under "Now", and "Build 238" itself (the engine's last build is 227).
- **The describing text** at the head of the Ocean plates panel, and **"Save" drawn held down**: both are in
  the Open table above. The Software map's card keeps its "What it does" text: that one is in his own
  picture of the page and is the one place a panel carries such text.

**2. Behaviour, decided by him, that no picture can show** (each is a row in "Decided by him" with his words):

- The wheel by where the pointer is: over a number box it steps the number; elsewhere in the panel it scrolls
  the panel; in the Tectonic Sim's or World Gen's viewport it zooms.
- Mouseover information in yellow tooltips; no describing text in the panels.
- A control that doesn't apply is greyed, never hidden (as the two greyed settings in `world gen.png`).
- One program, F2 both ways; the editor windowed above the taskbar in our own title bar and borders, F11
  full screen for the editor only; the game borderless full screen, with a setting to window it in the frame.
- The scales: probably two in the editor's menu (the editor's and the panel's), the debug panel's own
  (1x to 5x and Auto); whole numbers only, every pixel a whole pixel.
- The tabbed panel fixed on the left, between a half and a quarter of the width; the viewport square at
  1,280 x 800 and free to change shape when the window is resized.
- Our own text placer and text-entry box, for the editor and the game.
- **The Software map:** a click on any box, pass or store, turns its wires blue and brings them on top, turns
  the boxes it is wired to navy, and fills the card on the left; the name box's drop-down jumps to any box;
  the boxes are laid out from the code's own list (`src/wiring.rs`) and wrap to the width, so the page keeps
  itself true as the code changes. Example 10c is a store chosen.
- **The Registrar:** block textures are 32 x 32 and a thumbnail is one of them at one pixel to one; the plus
  mark on an entry of several textures shows the rest in a tooltip on hover; asset types are hardcoded; the
  palette's box on the right is fixed at 400 wide and must not scroll as a whole (only the list of splines
  scrolls); "the size of the 3 box areas on the left can be messed with".

**3. The measures behind the look** (further down this brief): the palette (section 3), the edges (4), the
marks (5 and the appendix), the sizes with Ironquill (6), the controls he picked (7). Ironquill at 15 pixels,
at x2 for a large name (the Registrar's "Grass"), never bold.

**4. Still his to decide.** The Open table above; the ones today's job touches are the debug panel's rows,
the wheel over the Software map, the menus' contents, the status bar's panes and the title's lettering.

**Checked by Counsel, of Builder's findings:** Ironquill's package (in his Downloads, `ironquill-fonts.zip`;
not in `Art/fonts/`) holds `Ironquill.png`, a sheet of white letters, 16 a row in cells of 9 x 15, 196
letters, with `Ironquill.fnt` giving each letter's place and width, so no sheet has to be made. Its
`LICENSE.txt`: "Put the font files in a public repository (GitHub and similar)" is under "You CAN'T"; "Ship
it inside your project's files: a game build" is under "You CAN". `Art/fonts/` already stays off GitHub.

**Measured by Builder the same day** (hers; the second also by Counsel):

- Ironquill's own sheet gives the same letters as the drawers' text, pixel for pixel: four of the pictures'
  texts laid from `Ironquill.png` with the `.fnt`'s widths at the drawer's places differ from his pictures in
  0 pixels. The `.fnt` has no kerning pairs and the sheet's alpha is 0 or 255 only. So text can be inside
  the pixel-for-pixel checks, and the drawers' measures carry over as they are (a box as wide as its name
  plus 14).
- `current/software map.png` is example 10 except the page's drop-down (x 192 to 317, y 25 to 46: 563
  pixels, the same count by Counsel's own comparison). The title bar and the menus' words are identical in
  all four pictures and in `blank.png`.
- `blank.png`'s body is his older working frame (teal, magenta dividers, old's toolbar buttons): it is not a
  specification of a blank page.
- One more stray from old: a row of 282830 at y 772, under the viewport's sunken frame, in the Software
  map's picture (Counsel saw the same row in the picture he pasted).
- **How she will test the frame against the cut-short pictures:** they read as a window of 1,281 x 801 with
  its last column and row off the canvas. The check draws our frame for 1,281 x 801 and compares the
  1,280 x 800 the picture has; a real 1,280 x 800 window then has its stretching parts one pixel smaller,
  with the whole raised edge in view.

### His pictures against the control sheet: Builder's comparison (2026-11-20, at his standing word; hers; his answers at its end)

His word: "make sure that you compare my drawings against the stylesheets and ask if mine differs from it, in a pixel, a shade, a line, a spacing, etc." She compared the four pictures, by pixel, with the Ironquill
control sheet and its drawer. His answer to each difference goes here as it comes.

**Match the sheet exactly:** the menu row (0 pixels differ over x 0 to 189, rows 24 to 47); the minimise and
close buttons; every tick box, radio, drop-down button, slider thumb, scroll arrow and the size grip, found
by search in all four; the panels of the Tectonic Sim, World Gen and the Software map hold only the scheme's
colours.

**Differ, in his own hand** (all four title bars):

1. The title's three buttons: a 1-pixel navy gap between each (minimise at x 1,225, maximise 1,242, close
   1,259), where the sheet has the first two touching and 2 pixels before close; his maximise square sits a
   pixel further right in its button.
2. The title's lettering is Darinia at 20 pixels, each letter's shape exact, but set closer than the font
   sets it: 139 wide against 151, every gap 1 pixel where the font gives 2, A and R touching, T over R by 2.
   She means to take his lettering as a picture.

**Differ between his own pictures:**

3. `tectonics.png` is example 2 with everything from y 52 down moved up 2 rows: the tabs' tops at y 50 where
   World Gen's are at 52, the chosen tab without its top edge and rounded corners, 2 rows of face under the
   scrollbar.
4. The page's drop-down is 128 wide in three pictures and 150 in the Registrar's.
5. The status bar's small panes: words 4 in on some and 5 in on others (the sheet: 5, and 4 for the first
   pane); in `tectonics.png` the Tick pane starts at x 1,117, not 1,116.

**Counsel's drawers against the sheet's own drawer** (Counsel's slips, carried into his pictures with the
examples he took; the sheet is what he chose from):

6. Radio buttons a pixel low beside their words (drawer 3 puts them at y + 2, the sheet at y + 1).
7. The lists in the Software map's card: rows 16 apart and words 6 in, where the sheet's list box has rows 15
   apart and words 4 in.
8. Sliders drawn without tick marks (the sheet has 12 under the channel); number boxes drawn as plain fields
   (the sheet's no. 12 has the two small arrows).
9. The Registrar's push buttons 23 tall with words 4 down (the sheet, with Ironquill: 25, words 5 down).

**Left from his first frame:**

10. The viewport's frame in the Tectonic Sim and World Gen is raised, with three bottom-right lines in 808080,
    3A3A42 and 282830 (the last two outside the scheme), spanning x 555 to 1,277 and y 50 to 772. Counsel's
    sunken frame in the Software map ends at x 1,275, a pixel short of the title bar's right end, which
    leaves those lines showing.

**The Tectonic Sim's toolbar, for its own build:**

11. The presets drop-down has lost its right edge; the Feed slider's thumb is 20 tall, not 21; Save is held
    down with a checker; the buttons are words, 51 x 22, where the sheet's are pictures, 24 x 22.

**One edge case:**

12. His tabs start level with the page's left edge, so a chosen first tab would stick out 2 pixels to the
    left; the sheet's chosen first tab is level with the edge.

**As Builder put them to him: a table of ten rows** (he will most likely answer by her rows; her
recommendation beside each is hers, not his word):

| Her row | The difference | The point above | She recommended |
|---|---|---|---|
| 1 | The title buttons' gaps and the maximise square | 1 | his |
| 2 | The Tectonic Sim's panel 2 pixels high, its chosen tab's top lost | 3 | World Gen's |
| 3 | The first tab when chosen | 12 | start the row of tabs 2 pixels in |
| 4 | Sliders without tick marks | 8 | no ticks |
| 5 | Number boxes without arrows | 8 | plain, since the wheel steps the number |
| 6 | Radio buttons a pixel low | 6 | the sheet's |
| 7 | Lists 16 apart and 6 in | 7 | the sheet's (she first said the picture's, and changed it since it wasn't his choice) |
| 8 | The viewport's frame | 10 | the sheet's sunken frame |
| 9 | The status bar's small panes | 5 | the sheet's 5 in |
| 10 | The page's drop-down's width | 4 | 128 everywhere |

Not in her table, noted for their own pages: the Tectonic Sim's toolbar (point 11) and the Registrar's buttons
(point 9). The title's lettering (point 2) she told him she will take as his picture. She told him that
points 6 to 9 came through Counsel's drawers and are not choices of his.

**His answers, 2026-11-20.** He numbered by her picture (1, 2, 2b, 3, 3b, 4, 5, 6, 7; kept as
`Art/References/editor reference/comparison - yours against the style sheet (2026-11-20).png`), not by the
table; the mapping is hers. His words are whole in "Decided by him" and in
`journal/docs/decisions.md`.

| His number | Her row | The point | His answer | What it comes to |
|---|---|---|---|---|
| 1 | 1 | 1 | "go with mine, with the boxes evenly spaced and the icons evenly spaced inside the boxes" | his: the buttons a pixel apart; the minimise bar centred (in his drawing 2 from the left of its face, 4 from the right); the square and the X as drawn |
| 2 | 2, first half | 3 | "conform to world gen height" | the Tectonic Sim's panel at World Gen's height |
| 2b | 2, second half | 3 | "replace the lost edge" | the chosen tab's top edge back |
| 3 | 4 | 8 | "a tik mark only at the ends and the center of the sliders" | three tick marks; the sheet's recipe for one (1 x 3, black, 2 below the thumb) fits the 50-pixel two-line setting, by her measure |
| 3b | 5 | 8 | "no arrows, scroll wheel or typing is enough" | plain number boxes; a click selects all, Enter keeps and leaves |
| 4 | 6 | 6 | "go with default" | the sheet's radio buttons |
| 5 | 7 | 7 | "go with 15 and 4" | the sheet's lists; and his standing rule on tightness |
| 6 | 8 | 10 | "go with my version"; later the same day, "derive win95's thick borderframe logic and use the standard colors" | Windows 95's thick frame in the standard colours, ending at x 1,276; then, by his answer 7, sunk and on all three views; its order of rings under Open |
| 7 | 9 | 5 | "just center anything in those displays" | words centred in the status bar's panes and the second bar's displays |
| (none) | 3 | 12 | asked again: "i don't think a chosen tab should 'grow', should it?"; then "make it not get taller, just be up front as re the borders between neighbors" | the chosen tab no taller, in front of its neighbours' edges; the row's height under Open |
| (none) | 10 | 4 | asked again: "128" | 128 on every page |

Where the answers fell against the recommendations: with Builder on her rows 1, 2, 5, 6 and 7; his own on 4
(three ticks, where she and Counsel both came to none), on 8 (his frame, where both said the sheet's) and on 9
(centred, where both said 5 in). Checked by Counsel on `current/tectonics.png`: the minimise bar at x 1,229 to
1,234 in a face of x 1,227 to 1,238; the square with 1 clear row above and 2 below; the frame's rings at x 555
to 557 (FFFFFF, DFDFDF, C0C0C0) and x 1,275 to 1,277 (808080, 3A3A42, 282830), the same at y 50 to 52 and
770 to 772.

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
- `comparison - yours against the style sheet (2026-11-20).png`: Builder's picture of the differences, the one
  his numbered answers of that day refer to.
- `choices - the chosen tab and the viewport frame (2026-11-20).png`: Builder's picture of the first tab three
  ways and the viewport's frame two ways (A and B) beside his own.
- `choices 2 - the tab in front, the row up 2, the sunk frame (2026-11-20).png`: Builder's picture from his
  answers 1 and 7: the tab in front and no taller, the row of tabs up 2 or left, the thick sunk frame two ways.
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

The first conventions he has given (2026-11-20): a click in a number box selects its whole contents, typing
replaces them, and Enter keeps the number and leaves the box ("some programs have annoying behaviors where you
click in there and always get a cursor after the decimal or in the middle"); the wheel steps a number box and
does nothing over the Software map; a tooltip mode with a question-mark cursor.

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
