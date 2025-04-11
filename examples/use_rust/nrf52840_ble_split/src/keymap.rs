use rmk::action::KeyAction;
use rmk::{a, k, mo};
pub(crate) const COL: usize = 10;
pub(crate) const ROW: usize = 4;
pub(crate) const NUM_LAYER: usize = 1;
#[rustfmt::skip]
pub const fn get_default_keymap() -> [[[KeyAction; COL]; ROW]; NUM_LAYER] {
    [
        [
            [k!(Q), k!(W), k!(E), k!(R), k!(T), k!(Y), k!(U), k!(I), k!(O), k!(P)],
            [k!(A), k!(S), k!(D), k!(F), k!(G), k!(H), k!(J), k!(K), k!(L), k!(Semicolon)],
            [k!(Z), k!(X), k!(C), k!(V), k!(B), a!(No), k!(N), k!(M), k!(Comma), k!(Dot)],
            [a!(No),a!(No),k!(Tab), k!(Escape), k!(Space), k!(Enter), k!(Backspace), k!(Quote), a!(No), a!(No)],
        ],
    ]
}
