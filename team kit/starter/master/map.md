# The software map

What exists and how it's wired. Once there's code, this file is drawn from the code's own list of its parts by its
test on every build: change the list, never this file. Until then it's empty.

The words (each keeps one meaning; David's to change):
- a **lane** is a line of work with its own pace (the frame is one; loading is another);
- a **pass** is one piece of a lane's work, on the processor or the card, timed by its own clock;
- a **stage** is one step of a recipe that makes data, in one linear chain;
- a **store** is something held, with where it lives, its size and its fixed limits.

Each box gets a card: what it is, what it reads and writes, its switches, its limits, the tests that keep it true,
its files, and its cost from the newest kept flight, with the build and the date.
