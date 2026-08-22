use embassy_time::Instant;
use rmk_types::action::{Action, KeyAction};

use crate::HELD_BUFFER_SIZE;
use crate::event::{KeyboardEvent, KeyboardEventPos};
use crate::morse::MorsePattern;

/// The buffer of held keys.
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct HeldBuffer {
    pub(crate) keys: heapless::Vec<HeldKey, HELD_BUFFER_SIZE>,
}

impl HeldBuffer {
    /// Create a new held buffer
    pub fn new() -> Self {
        Self {
            keys: heapless::Vec::new(),
        }
    }

    /// Push a new held key into the buffer and then sort by the timeout
    pub fn push(&mut self, key: HeldKey) {
        if let Err(e) = self.keys.push(key) {
            error!("Held buffer overflowed, cannot save: {:?}", e);
        }

        self.sort_by_timeout();
    }

    /// Push a new held key into the buffer
    pub fn push_without_sort(&mut self, key: HeldKey) {
        if let Err(e) = self.keys.push(key) {
            error!("Held buffer overflowed, cannot save: {:?}", e);
        }
    }

    /// Find a held key by the key action
    pub fn find_action(&self, action: &KeyAction) -> Option<&HeldKey> {
        self.keys.iter().find(|x| x.action == *action)
    }

    /// Find a held key by the KeyboardEventPos
    pub fn find_pos(&self, pos: KeyboardEventPos) -> Option<&HeldKey> {
        self.keys.iter().find(|x| x.event.pos == pos)
    }

    /// Find a mutable held key by the KeyboardEventPos
    pub fn find_pos_mut(&mut self, pos: KeyboardEventPos) -> Option<&mut HeldKey> {
        self.keys.iter_mut().find(|x| x.event.pos == pos)
    }

    /// Remove a held key from the buffer, keep the order
    pub fn remove_if<P>(&mut self, predicate: P) -> Option<HeldKey>
    where
        P: FnMut(&HeldKey) -> bool,
    {
        if let Some(i) = self.keys.iter().position(predicate) {
            Some(self.keys.remove(i))
        } else {
            None
        }
    }

    /// Remove a held key from the buffer and then resort the buffer
    pub fn remove(&mut self, pos: KeyboardEventPos) -> Option<HeldKey> {
        let k = self.remove_if(|k| k.event.pos == pos);
        self.sort_by_timeout();
        k
    }

    // `slice::sort_unstable_by_key` uses a general small-sort scratch frame
    // sized for much larger slices. HeldBuffer is deliberately small and
    // almost sorted, so insertion sort preserves the existing ordering with
    // bounded stack usage and no auxiliary storage.
    pub(crate) fn sort_by_timeout(&mut self) {
        for i in 1..self.keys.len() {
            let mut j = i;
            while j > 0 && self.keys[j].timeout_time < self.keys[j - 1].timeout_time {
                self.keys.swap(j, j - 1);
                j -= 1;
            }
        }
    }

    pub(crate) fn sort_by_press_time(&mut self) {
        for i in 1..self.keys.len() {
            let mut j = i;
            while j > 0 && self.keys[j].press_time < self.keys[j - 1].press_time {
                self.keys.swap(j, j - 1);
                j -= 1;
            }
        }
    }

    /// Get the next timeout key in the buffer
    pub fn next_timeout<P>(&self, mut predicate: P) -> Option<HeldKey>
    where
        P: FnMut(&HeldKey) -> bool,
    {
        // Support that the held buffer is already sorted by the timeout time
        self.keys.iter().find(|&x| predicate(x)).copied()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
}

/// The state of a held key.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum KeyState {
    /// The current key is a component of a combo, and it's waiting for other combo components
    WaitingCombo,

    /// After a press event is received.
    /// The data represents the previously completed morse pattern
    Pressed(MorsePattern),

    /// After a press event is received and the hold timeout is reached.
    /// The data represents the previously completed morse pattern
    /// including the current hold
    Holding(MorsePattern),

    /// After a release event is received for a key still kept in the HeldBuffer - so morse pattern may continue
    /// The data represents the already completed morse pattern
    Released(MorsePattern),

    /// The corresponding action is already executed (so the Pressed HID report is sent),
    /// but the release HID report is not sent yet (will be sent only when the corresponding
    /// key is really released).
    ProcessedButReleaseNotReportedYet(Action),
    // The Idle state is represented by the removal from the HeldBuffer
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct HeldKey {
    pub event: KeyboardEvent,
    pub action: KeyAction,
    /// Current state of the held key
    pub state: KeyState,
    /// The press time for the key
    pub press_time: Instant,
    /// The timeout time for the key
    pub timeout_time: Instant,
}

impl HeldKey {
    pub fn new(
        event: KeyboardEvent,
        action: KeyAction,
        state: KeyState,
        press_time: Instant,
        timeout_time: Instant,
    ) -> Self {
        Self {
            event,
            action,
            state,
            press_time,
            timeout_time,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn held_key(row: u8, timeout_ticks: u64) -> HeldKey {
        HeldKey::new(
            KeyboardEvent::key(row, 0, true),
            KeyAction::No,
            KeyState::WaitingCombo,
            Instant::from_ticks(0),
            Instant::from_ticks(timeout_ticks),
        )
    }

    #[test]
    fn push_orders_keys_by_timeout_without_scratch_sort() {
        let mut buffer = HeldBuffer::new();
        buffer.push(held_key(0, 30));
        buffer.push(held_key(1, 10));
        buffer.push(held_key(2, 20));

        let timeouts: heapless::Vec<u64, HELD_BUFFER_SIZE> =
            buffer.keys.iter().map(|key| key.timeout_time.as_ticks()).collect();
        assert_eq!(timeouts.as_slice(), &[10, 20, 30]);
    }
}
