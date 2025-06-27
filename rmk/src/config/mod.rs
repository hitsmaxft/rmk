#[cfg(feature = "_ble")]
mod ble_config;
pub mod macro_config;

#[cfg(feature = "_ble")]
pub use ble_config::BleBatteryConfig;
use embassy_time::Duration;
use embedded_hal::digital::OutputPin;
use heapless::Vec;
use macro_config::KeyboardMacrosConfig;

use crate::combo::Combo;
use crate::event::KeyEvent;
use crate::fork::Fork;
use crate::{COMBO_MAX_NUM, FORK_MAX_NUM};

/// The config struct for RMK keyboard.
///
/// There are 3 types of configs:
/// 1. `ChannelConfig`: Configurations for channels used in RMK.
/// 2. `ControllerConfig`: Config for controllers, the controllers are used for controlling other devices on the board.
/// 3. `RmkConfig`: Tunable configurations for RMK keyboard.
pub struct KeyboardConfig<'a, O: OutputPin> {
    pub controller_config: ControllerConfig<O>,
    pub rmk_config: RmkConfig<'a>,
}

impl<O: OutputPin> Default for KeyboardConfig<'_, O> {
    fn default() -> Self {
        Self {
            controller_config: ControllerConfig::default(),
            rmk_config: RmkConfig::default(),
        }
    }
}

/// Config for controllers.
///
/// Controllers are used for controlling other devices on the board, such as lights, RGB, etc.
pub struct ControllerConfig<O: OutputPin> {
    pub light_config: LightConfig<O>,
}

impl<O: OutputPin> Default for ControllerConfig<O> {
    fn default() -> Self {
        Self {
            light_config: LightConfig::default(),
        }
    }
}

/// Internal configurations for RMK keyboard.
#[derive(Default)]
pub struct RmkConfig<'a> {
    pub usb_config: KeyboardUsbConfig<'a>,
    pub vial_config: VialConfig<'a>,
    #[cfg(feature = "storage")]
    pub storage_config: StorageConfig,
    #[cfg(feature = "_ble")]
    pub ble_battery_config: BleBatteryConfig<'a>,
}

/// Config for configurable action behavior
#[derive(Debug, Default)]
pub struct BehaviorConfig {
    pub tri_layer: Option<[u8; 3]>,
    pub tap_hold: TapHoldConfig,
    pub one_shot: OneShotConfig,
    pub combo: CombosConfig,
    pub fork: ForksConfig,
    pub keyboard_macros: KeyboardMacrosConfig,
}

/// Configurations for tap hold behavior
#[derive(Clone, Copy, Debug)]
pub struct TapHoldConfig {
    pub enable_hrm: bool,
    pub prior_idle_time: Duration,
    pub post_wait_time: Duration,
    pub hold_timeout: Duration,
    /// Same as QMK's permissive hold: https://docs.qmk.fm/tap_hold#tap-or-hold-decision-modes
    pub permissive_hold: bool,
    /// If the previous key is on the same "hand", the current key will be determined as a tap
    pub chordal_hold: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ChordHoldHand {
    Left,
    Right,
}

#[derive(Clone, Debug)]
pub struct ChordHoldState<const COUNT: usize> {
    pub is_vertical_chord: bool,
    pub hand: ChordHoldHand,
}

impl<const COUNT: usize> ChordHoldState<COUNT> {
    // is the key event in the same side of current chord hold
    pub fn is_same(&self, key_event: KeyEvent) -> bool {
        if self.is_vertical_chord {
            return self.is_same_hand(key_event.row as usize);
        } else {
            return self.is_same_hand(key_event.col as usize);
        }
    }

    pub fn is_same_hand(&self, number: usize) -> bool {
        match self.hand {
            ChordHoldHand::Left => number < COUNT / 2,
            ChordHoldHand::Right => number >= COUNT / 2,
        }
    }

    /// Create a new `ChordHoldState` based on the key event and the number of rows and columns.
    /// If the number of columns is greater than the number of rows, it will determine the hand based on the column.
    /// the chordal hold will be determined by user configuration in future.
    pub(crate) fn create(event: KeyEvent, rows: usize, cols: usize) -> Self {
        if cols > rows {
            if (event.col as usize) < (cols / 2) {
                ChordHoldState {
                    is_vertical_chord: false,
                    hand: ChordHoldHand::Left,
                }
            } else {
                ChordHoldState {
                    is_vertical_chord: false,
                    hand: ChordHoldHand::Right,
                }
            }
        } else {
            if (event.row as usize) < (rows / 2) {
                ChordHoldState {
                    is_vertical_chord: true,
                    hand: ChordHoldHand::Left,
                }
            } else {
                ChordHoldState {
                    is_vertical_chord: true,
                    hand: ChordHoldHand::Right,
                }
            }
        }
    }
}

impl Default for TapHoldConfig {
    fn default() -> Self {
        Self {
            enable_hrm: false,
            permissive_hold: false,
            chordal_hold: false,
            prior_idle_time: Duration::from_millis(120),
            post_wait_time: Duration::from_millis(50),
            hold_timeout: Duration::from_millis(250),
        }
    }
}

/// Config for one shot behavior
#[derive(Clone, Copy, Debug)]
pub struct OneShotConfig {
    pub timeout: Duration,
}

impl Default for OneShotConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(1),
        }
    }
}

/// Config for combo behavior
#[derive(Clone, Debug)]
pub struct CombosConfig {
    pub combos: Vec<Combo, COMBO_MAX_NUM>,
    pub timeout: Duration,
}

impl Default for CombosConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_millis(50),
            combos: Vec::new(),
        }
    }
}

/// Config for fork behavior
#[derive(Clone, Debug)]
pub struct ForksConfig {
    pub forks: Vec<Fork, FORK_MAX_NUM>,
}

impl Default for ForksConfig {
    fn default() -> Self {
        Self { forks: Vec::new() }
    }
}

/// Config for storage
#[derive(Clone, Copy, Debug)]
pub struct StorageConfig {
    /// Start address of local storage, MUST BE start of a sector.
    /// If start_addr is set to 0(this is the default value), the last `num_sectors` sectors will be used.
    pub start_addr: usize,
    // Number of sectors used for storage, >= 2.
    pub num_sectors: u8,
    pub clear_storage: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            start_addr: 0,
            num_sectors: 2,
            clear_storage: false,
        }
    }
}

/// Config for lights
pub struct LightConfig<O: OutputPin> {
    pub capslock: Option<LightPinConfig<O>>,
    pub scrolllock: Option<LightPinConfig<O>>,
    pub numslock: Option<LightPinConfig<O>>,
}

pub struct LightPinConfig<O: OutputPin> {
    pub pin: O,
    pub low_active: bool,
}

impl<O: OutputPin> Default for LightConfig<O> {
    fn default() -> Self {
        Self {
            capslock: None,
            scrolllock: None,
            numslock: None,
        }
    }
}

/// Config for [vial](https://get.vial.today/).
///
/// You can generate automatically using [`build.rs`](https://github.com/HaoboGu/rmk/blob/main/examples/use_rust/stm32h7/build.rs).
#[derive(Clone, Copy, Debug, Default)]
pub struct VialConfig<'a> {
    pub vial_keyboard_id: &'a [u8],
    pub vial_keyboard_def: &'a [u8],
}

impl<'a> VialConfig<'a> {
    pub fn new(vial_keyboard_id: &'a [u8], vial_keyboard_def: &'a [u8]) -> Self {
        Self {
            vial_keyboard_id,
            vial_keyboard_def,
        }
    }
}

/// Configurations for usb
#[derive(Clone, Copy, Debug)]
pub struct KeyboardUsbConfig<'a> {
    /// Vender id
    pub vid: u16,
    /// Product id
    pub pid: u16,
    /// Manufacturer
    pub manufacturer: &'a str,
    /// Product name
    pub product_name: &'a str,
    /// Serial number
    pub serial_number: &'a str,
}

impl Default for KeyboardUsbConfig<'_> {
    fn default() -> Self {
        Self {
            vid: 0x4c4b,
            pid: 0x4643,
            manufacturer: "RMK",
            product_name: "RMK Keyboard",
            serial_number: "vial:f64c2b3c:000001",
        }
    }
}

mod tests {
    use super::{ChordHoldHand, ChordHoldState};
    use crate::{event::KeyEvent, heapless::Vec};

    #[test]
    fn test_chordal_hold() {
        assert_eq!(
            ChordHoldState::<6>::create(
                KeyEvent {
                    row: 0,
                    col: 0,
                    pressed: true,
                },
                3,
                6
            )
            .hand,
            ChordHoldHand::Left
        );
        assert_eq!(
            ChordHoldState::<6>::create(
                KeyEvent {
                    row: 3,
                    col: 3,
                    pressed: true,
                },
                4,
                6
            )
            .hand,
            ChordHoldHand::Right
        );
        assert_eq!(
            ChordHoldState::<6>::create(
                KeyEvent {
                    row: 3,
                    col: 3,
                    pressed: true,
                },
                6,
                4
            )
            .hand,
            ChordHoldHand::Right
        );
        assert_eq!(
            ChordHoldState::<6>::create(
                KeyEvent {
                    row: 6,
                    col: 3,
                    pressed: true,
                },
                5,
                3
            )
            .hand,
            ChordHoldHand::Right
        );

        let chord = ChordHoldState::<6> {
            is_vertical_chord: false,
            hand: ChordHoldHand::Left,
        };

        let vec: heapless::Vec<_, 6> = Vec::from_slice(&[0u8, 1, 2, 3, 4, 5]).unwrap();
        let result: heapless::Vec<_, 6> = vec
            .iter()
            .map(|col| {
                chord.is_same(KeyEvent {
                    row: 0,
                    col: 0 + col,
                    pressed: true,
                })
            })
            .collect();

        let result2: heapless::Vec<bool, 6> = Vec::from_slice(&[true, true, true, false, false, false]).unwrap();
        assert_eq!(result, result2);
    }
}
