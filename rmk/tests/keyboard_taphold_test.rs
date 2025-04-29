extern crate rmk;

use embassy_futures::block_on;
use embassy_time::Duration;
use rusty_fork::rusty_fork_test;
use std::cell::RefCell;

use rmk::{
    config::{BehaviorConfig, TapHoldConfig},
    k,
    th,
    keyboard::Keyboard,
    keycode::KeyCode,
    keymap::KeyMap
};

mod common;
pub(crate) use crate::common::*;

// Init logger for tests
#[ctor::ctor]
pub fn init_log() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();
}

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
            let config =BehaviorConfig {
                    tap_hold: TapHoldConfig {
                        post_wait_time: Duration::from_millis(50),
                        ..Default::default()
                    },
                    ..Default::default()
                };
        let keymap:&mut RefCell<KeyMap<1, 2, 1>> = wrap_keymap(
                [[[
                    th!(B, LShift),
                k!(A)
                ]]]
                ,
                config.clone()
            );
        let mut keyboard = Keyboard::new(keymap, config);


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
            [KC_LSHIFT, [0, 0, 0, 0, 0, 0]],
            [KC_LSHIFT, [ kc8!(A) , 0, 0, 0, 0, 0]],
            [0, [ kc8!(A) , 0, 0, 0, 0, 0]],
            [0, [ 0, 0, 0, 0, 0, 0]],
            [KC_LSHIFT, [0, 0, 0, 0, 0, 0]],
            [0, [ 0, 0, 0, 0, 0, 0]],
            [0, [ kc8!(A) , 0, 0, 0, 0, 0]],
            [0, [ 0, 0, 0, 0, 0, 0]],
        ];

        run_key_sequence_test(&mut keyboard, &sequence, expected_reports).await;
    });
}
}
#[test]
fn test_tap_hold_key_multi_hold() {
    let main = async {
        let mut keyboard = create_test_keyboard();

        let sequence = key_sequence![
            [2, 1, true, 10], // Press th!(A,shift)
            [2, 2, true, 10], //  press th!(S,lgui)
            //hold timeout
            [2, 3, true, 270],  //  press d
            [2, 3, false, 290], // release d
            [2, 1, false, 380], // Release A
            [2, 2, false, 400], // Release s
        ];
        let expected_reports = key_report![
            [KC_LSHIFT, [0, 0, 0, 0, 0, 0]],                          //shift
            [KC_LSHIFT | KC_LGUI, [0, 0, 0, 0, 0, 0]],                //shift
            [KC_LSHIFT | KC_LGUI, [KeyCode::D as u8, 0, 0, 0, 0, 0]], // 0x7
            [KC_LSHIFT | KC_LGUI, [0, 0, 0, 0, 0, 0]],                //shift
            [KC_LGUI, [0, 0, 0, 0, 0, 0]],                            //shift and gui
            [0, [0, 0, 0, 0, 0, 0]],
        ];

        run_key_sequence_test(&mut keyboard, &sequence, expected_reports).await;
    };
    block_on(main);
}
