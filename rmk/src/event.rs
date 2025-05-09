use embassy_time::Instant;
use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

use crate::{action::Action, input_device::rotary_encoder::Direction};
use crate::action::KeyAction;
use crate::keyboard::Keyboard;
use crate::keycode::KeyCode::No;

/// Raw events from input devices and keyboards
///
/// This should be as close to the raw output of the devices as possible.
/// The input processors receives it, processes it,
/// and then converts it to the final keyboard/mouse report.
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Copy, Debug, MaxSize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Event {
    /// Keyboard event
    Key(KeyEvent),
    /// Rotary encoder, ec11 compatible models
    RotaryEncoder(RotaryEncoderEvent),
    /// Multi-touch touchpad
    Touchpad(TouchpadEvent),
    /// Joystick, suppose we have x,y,z axes for this joystick
    Joystick([AxisEvent; 3]),
    /// An AxisEvent in a stream of events. The receiver should keep receiving events until it receives [`Event::Eos`] event.
    AxisEventStream(AxisEvent),
    /// Battery percentage event
    Battery(u16),
    /// Charging state changed event, true means charging, false means not charging
    ChargingState(bool),
    /// End of the event sequence
    ///
    /// This is used with [`Event::AxisEventStream`] to indicate the end of the event sequence.
    Eos,
    /// Custom event
    Custom([u8; 16]),
}

/// Event for rotary encoder
#[derive(Serialize, Deserialize, Clone, Copy, Debug, MaxSize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RotaryEncoderEvent {
    /// The id of the rotary encoder
    pub id: u8,
    /// The direction of the rotary encoder
    pub direction: Direction,
}

/// Event for multi-touch touchpad
#[derive(Serialize, Deserialize, Clone, Copy, Debug, MaxSize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TouchpadEvent {
    /// Finger slot
    pub finger: u8,
    /// X, Y, Z axes for touchpad
    pub axis: [AxisEvent; 3],
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy, MaxSize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AxisEvent {
    /// The axis event value type, relative or absolute
    pub typ: AxisValType,
    /// The axis name
    pub axis: Axis,
    /// Value of the axis event
    pub value: i16,
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy, MaxSize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AxisValType {
    /// The axis value is relative
    Rel,
    /// The axis value is absolute
    Abs,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, MaxSize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Axis {
    X,
    Y,
    Z,
    H,
    V,
    // .. More is allowed
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, MaxSize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct KeyEvent {
    pub row: u8,
    pub col: u8,
    pub pressed: bool,
}


#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum HoldingKey {
    TapHold(PressedTapHold),
    Tapping(BufferedPressEvent),
}


pub trait HoldingKeyTrait {
    fn update_state(&mut self, new_state: TapHoldState);
    fn press_time(& self) -> Instant;
    fn state(&self) -> TapHoldState;
}

impl HoldingKey {
    pub(crate) fn start_time(&self) -> Instant {
        match  self {
            HoldingKey::TapHold(h) => h.pressed_time,
            HoldingKey::Tapping(h) => h.pressed_time,
        }
    }

    pub(crate) fn key_event(&self) -> KeyEvent {

        match  self {
            HoldingKey::TapHold(h) => h.key_event,
            HoldingKey::Tapping(h) => h.key_event,
        }
    }

    pub(crate) fn is_tap_hold(&self) -> bool {

        match  self {
            HoldingKey::TapHold(h) => true,
            _ => false,
        }
    }
}

impl HoldingKeyTrait for HoldingKey {
    fn update_state(&mut self, new_state: TapHoldState) {

        match self {
            HoldingKey::TapHold(h) => {
                h.state = new_state;
            }
            HoldingKey::Tapping(t) => {
                t.state = new_state;
            }
        }
    }

    fn press_time(&self) -> Instant {
        match self {
            HoldingKey::TapHold(h) => {
                h.pressed_time
            }
            HoldingKey::Tapping(t) => {
                t.pressed_time
            }
        }
    }

    fn state(&self) -> TapHoldState {
        match self {
            HoldingKey::TapHold(h) => {
                h.state
            }
            HoldingKey::Tapping(t) => {
                t.state
            }
        }

    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TapHoldState {
    //happen after press event arrives
    Initial,
    //tapping event
    Tap,
    //release tapping event
    PostTap,
    //holding event
    Hold,
    //release holding event
    PostHold,
    //reserved
    Release,
}


// record pressing tap hold keys
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PressedTapHold {
    pub key_event: KeyEvent,
    pub tap_action: Action,
    pub hold_action: Action,
    pub pressed_time: Instant,
    pub deadline: u64,
    pub state: TapHoldState,
}

impl  PressedTapHold {
    const NO: Action = Action::Key(No);

    pub(crate)  fn hold_action(&self) -> Action {
        self.hold_action
    }

    pub(crate) fn tap_action(&self) -> Action {
        self.tap_action
    }
}
impl HoldingKeyTrait for PressedTapHold {
    fn update_state(&mut self, new_state: TapHoldState) {
        self.state = new_state
    }


    fn press_time(&self) -> Instant {
        self.pressed_time
    }

    fn state(&self) -> TapHoldState {
        self.state
    }
}

//buffered pressing key event while TapHolding
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BufferedPressEvent {
    pub key_event: KeyEvent,
    pub key_action: KeyAction,
    pub pressed_time: Instant,
    //initial -> tap -> post tap -> release
    pub state: TapHoldState,
}


impl HoldingKeyTrait for BufferedPressEvent {
    fn update_state(&mut self, new_state: TapHoldState) {
        self.state = new_state
    }

    fn press_time(&self) -> Instant {
        self.pressed_time
    }

    fn state(&self) -> TapHoldState {
        self.state
    }
}