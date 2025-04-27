extern crate rmk;

use embassy_futures::block_on;
use embassy_futures::select::select;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Timer};
use futures::{join, FutureExt};
use rusty_fork::rusty_fork_test;

use rmk::action::KeyAction;
use rmk::config::{BehaviorConfig, CombosConfig, ForksConfig};
use rmk::{a, k, layer, mo, th};

use rmk::channel::{KEYBOARD_REPORT_CHANNEL, KEY_EVENT_CHANNEL};
use rmk::combo::Combo;
use rmk::event::{KeyEvent, PressedKeyEvent};
use rmk::keycode::{KeyCode, ModifierCombination};

mod common;
use crate::common::{run_key_sequence_test, create_test_keyboard, create_test_keyboard_with_config, TestKeyPress};

// mod key values
const KC_LShift: u8 = 1 << 1;
const KC_LGUI: u8 = 1 << 3;

rusty_fork_test! {

#[test]
fn test_taphold_tap() {
    let main = async {
        let mut keyboard = create_test_keyboard();

        let sequence = key_sequence![
            [2, 1, true, 10],  // Press TH shift A
            //release before hold timeout
            [2, 1, false, 100], // Release A
        ];

        let expected_reports = key_report![
            //should be a tapping A
            [0, [0x04, 0, 0, 0, 0, 0]],
        ];

        run_key_sequence_test(&mut keyboard, &sequence, expected_reports).await;
    };
    block_on(main);
}


#[test]
fn test_taphold_hold() {
    let main = async {
        let mut keyboard = create_test_keyboard();

        let sequence = key_sequence![
            [2, 1, true, 10],  // Press TH shift A
            [2, 1, false, 300], // Release A
        ];

        let expected_reports = key_report![
            //tap on a
            [2, [0, 0, 0, 0, 0, 0]],
        ];

        run_key_sequence_test(&mut keyboard, &sequence, expected_reports).await;
    };
    block_on(main);
}


// test post_wait_time
#[test]
fn test_tap_hold_key_post_wait_timeout() {
    block_on( async {
        let mut keyboard = create_test_keyboard();
        keyboard.keymap.borrow_mut().set_action_at(
            0,
            0,
            0,
            th!(B, LShift),
        );
        keyboard.keymap.borrow_mut().set_action_at(
            0,
            1,
            0,
            k!(A)
        );

        keyboard.keymap.borrow_mut().behavior.tap_hold.post_wait_time = Duration::from_millis(50);

        let sequence = key_sequence![
            [0, 0, true, 10],  // press th b
            [0, 0, false, 300], // Release th b
            [0, 1, true, 10],  // Press a within post wait timeout
            [0, 1, false, 10],  // Press a
            [0, 0, true, 10],  // press th b
            [0, 0, false, 300], // Release th b
            [0, 1, true, 100],  // Press a out of post wait timeout
            [0, 1, false, 10],  // Press a
        ];

        let expected_reports = key_report![
            //tap on a
            [KC_LShift, [0, 0, 0, 0, 0, 0]],
            [KC_LShift, [ kc8!(A) , 0, 0, 0, 0, 0]],
            [0, [ kc!(A) , 0, 0, 0, 0, 0]],
            [0, [ 0, 0, 0, 0, 0, 0]],
            [KC_LShift, [0, 0, 0, 0, 0, 0]],
            [0, [ 0, 0, 0, 0, 0, 0]],
            [0, [ kc!(A) , 0, 0, 0, 0, 0]],
            [0, [ 0, 0, 0, 0, 0, 0]],
        ];

        run_key_sequence_test(&mut keyboard, &sequence, expected_reports).await;
    });
}
}
