The far march's compiled program by its parts (Builder, 2026-11-16, at David's word)
====================================================================================

What's here, and what it's for, is written up in journal/docs/march-by-parts.md: a tree of the march's program
by its parts with each part's size in instructions, and the values the march holds in hand round its loop (its
size on the card). For a later review looking for parts that do little.

march_parts.py   the script: compiles a program's text as the game's compiler does (the Windows SDK's dxc.exe,
                 cs_6_7, -HV 2018 -no-warnings -Ges), again with debug marks, checks the two are the same program
                 instruction for instruction, and prints the tree and the held values.
b194/            build 194's march once the textures are in, as the card's compiler was given it, with cover by
                 faces on and off, and the script's output for each (4,548 and 4,436 instructions; 98 and 97 held
                 values).

To take it again for a later build:

  FAR_TEST_SHADER_TEXT="<a folder>" FAR_TEST_BLEND=1 cargo test --release -j 4 the_far_eye_draws_ground_and_sky
  python "kept logs/march/march_parts.py" "<the folder>" march_all.compute.hlsl

(The script writes two listings beside the text, <name>.ll and <name>.debug.ll, about 300 KB each: not kept.)
