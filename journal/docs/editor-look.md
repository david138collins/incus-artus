# The editor's look: Windows 95's, drawn by us

Counsel's brief for Builder, 2026-11-19, at David's asking: "please compile for builder all the technical
information re. win95 shell implementation we've come to discuss about the win95 look. i'm going to mock up what
the editor should look like after going through old incus and we'll start from there."

**Standing:** a leaning when written (`journal/docs/decisions.md`, the rebuild, 2026-11-18); later the same
day he spoke of the method as decided ("what we've decided technically on how to implement a win95 gui"), which
Counsel reads as his go on the method. Nothing is built
and nothing is priced. His mock-ups come first; they overrule anything here. What is asked of Builder now is a
price and her own reading of section 8, not a build.

What is measured here is marked *measured*; what is Counsel's suggestion or a thing nobody has tried is marked so.

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

**Drawn by Counsel to the same rules** (in `Art/editor reference/control sheet drawer.cs`; the 95 originals were
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
| 21, 22 | Tooltip, status bar | added in the redraw; not yet spoken to |
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

- `Art/editor reference/` (off GitHub with the rest of `Art/`): the control sheet at one screen pixel to one
  sheet pixel, in Ironquill at 15 and in MS Sans Serif, each numbered and clean (680 pixels wide); the drawer's
  source; the probe that printed the appendix.
- `Art/font reference/`: every font at its true size; title bases; the menu mock-up's pieces; his HUD at three
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
| Tooltip, status bar and the richer list (21 to 23): wanted as drawn? | David, in his mock-ups |

## Appendix: the pixels, as Windows' classic painter gives them

Printed by `Art/editor reference/edge and mark probe.cs` on 2026-11-19 (`DrawEdge` and `DrawFrameControl` onto a
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
