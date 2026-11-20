//! The small pictures, lifted pixel for pixel from his four pictures (`Art/95gui mockups/current/`), which have them as
//! Windows 95 draws them. A letter a pixel (`Sheet::mark`): W white, L light, . face, S grey, K black; a space leaves
//! what's under it.

/// The title bar's minimise mark, at (5, 10) in its button: centred in the button's face, as he asked of all three
/// ("the icons evenly spaced inside the boxes"); his picture has it a pixel to the left.
pub const MINIMISE: [&str; 2] = ["KKKKKK", "KKKKKK"];
pub const MINIMISE_AT: (i32, i32) = (5, 10);

/// The maximise mark, at (3, 3): centred across; its three spare rows can't split evenly, so one above and two below,
/// as he drew it.
pub const MAXIMISE: [&str; 9] = ["KKKKKKKKKK", "KKKKKKKKKK", "K        K", "K        K", "K        K", "K        K", "K        K", "K        K", "KKKKKKKKKK"];
pub const MAXIMISE_AT: (i32, i32) = (3, 3);

/// The close mark, at (4, 4): centred both ways.
pub const CLOSE: [&str; 8] = ["KK    KK", "KKK  KKK", " KKKKKK ", "  KKKK  ", "  KKKK  ", " KKKKKK ", "KKK  KKK", "KK    KK"];
pub const CLOSE_AT: (i32, i32) = (4, 4);

/// The drop-down button's arrow, at (4, 7) in its 16 x 19 button.
pub const DROP_DOWN: [&str; 4] = ["KKKKKKK", " KKKKK ", "  KKK  ", "   K   "];
pub const DROP_DOWN_AT: (i32, i32) = (4, 7);

/// The scrollbar's arrows greyed (nothing to scroll), at (4, 6) in their 16 x 16 buttons: Windows' greyed glyph, grey
/// with white under and to the right of it.
pub const UP_GREYED: [&str; 5] = ["   S   ", "  SSS  ", " SSSSS ", "SSSSSSS", " WWWWWWW"];
pub const DOWN_GREYED: [&str; 5] = ["SSSSSSS ", " SSSSSWW", "  SSSWW ", "   SWW  ", "    W   "];
pub const ARROW_AT: (i32, i32) = (4, 6);

/// The grip in the status bar's corner, 14 x 14.
pub const GRIP: [&str; 14] = [
    "              ",
    "              ",
    "             W",
    "            WS",
    "           WSS",
    "          WSS ",
    "         WSS W",
    "        WSS WS",
    "       WSS WSS",
    "      WSS WSS ",
    "     WSS WSS W",
    "    WSS WSS WS",
    "   WSS WSS WSS",
    "  WSS WSS WSS ",
];
