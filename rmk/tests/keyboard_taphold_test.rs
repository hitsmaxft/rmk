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


/**
* hrm config
*/
fn get_th_config_for_permissive_hold_test(
) -> TapHoldConfig {
    TapHoldConfig {
        enable_hrm: true,
        permissive_hold : true,
        ..TapHoldConfig::default ()
    }
}
/**
* hrm config
*/
fn get_th_config_for_test(
) -> TapHoldConfig {
    TapHoldConfig {
        enable_hrm: true,
        permissive_hold : false,
        chord_hold : true,
        ..TapHoldConfig::default ()
    }
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
                [0, [0, 0, 0, 0, 0, 0]],
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


    //normal tap hold tests
    #[test]
    fn test_tap_hold_key_release_rolling() {
        // eager hold
        let main = async {
            let mut keyboard = create_test_keyboard_with_config(BehaviorConfig {
                //perfer hold
                tap_hold:get_th_config_for_permissive_hold_test(),
                    .. BehaviorConfig::default()
                });

            let sequence = key_sequence![
                [2, 1, true, 10], // Press th!(A,shift)
                [2, 2, true, 30], //  press th!(S,lgui)
                [2, 3, true, 30],  //  press d
                // eager hold and output
                [2, 1, false, 50], // Release A
                [2, 2, false, 100], // Release s
                [2, 3, false, 100],  //  press d
            ];
            let expected_reports = key_report![
                [0, [kc8!(A), 0, 0, 0, 0, 0]],
                [0, [0, 0, 0, 0, 0, 0]],
                [0, [kc8!(S), 0, 0, 0, 0, 0]],
                [0, [0, 0, 0, 0, 0, 0]],
                [0, [kc8!(D), 0, 0, 0, 0, 0]],
                [0, [0, 0, 0, 0, 0, 0]],
            ];

            run_key_sequence_test(&mut keyboard, &sequence, expected_reports).await;
        };
        block_on(main);
    }


    //permissive hold test cases
    #[test]
    fn test_tap_hold_hold_on_other_release() {
            // eager hold
        let main = async {
            let mut keyboard = create_test_keyboard_with_config(BehaviorConfig {
                tap_hold: get_th_config_for_permissive_hold_test(),
                    .. BehaviorConfig::default()
                });

            let sequence = key_sequence![
                [2, 1, true, 10], // Press th!(A,shift)
                [2, 2, true, 30], //  press th!(S,lgui)
                [2, 3, true, 30],  //  press d
                [2, 3, false, 10],  // Release d
                // eager hold and output
                
                [2, 1, false, 50], // Release A
                [2, 2, false, 100], // Release s
            ];
            let expected_reports = key_report![
                [KC_LSHIFT, [0, 0, 0, 0, 0, 0]], // #0
                [KC_LSHIFT | KC_LGUI, [0, 0, 0, 0, 0, 0]],
                [KC_LSHIFT | KC_LGUI,  [kc8!(D), 0, 0, 0, 0, 0]],
                [KC_LSHIFT | KC_LGUI, [0, 0, 0, 0, 0, 0]],
                [KC_LGUI, [ 0, 0, 0, 0, 0, 0]],
                [0, [0, 0, 0, 0, 0, 0]],
            ];

            run_key_sequence_test(&mut keyboard, &sequence, expected_reports).await;

        };
        block_on(main);
    }
    #[test]
    fn test_tap_hold_key_mixed_release_hold() {
            // eager hold
        let main = async {
            let mut keyboard = create_test_keyboard_with_config(BehaviorConfig {
                tap_hold: get_th_config_for_permissive_hold_test(),
                    .. BehaviorConfig::default()
                });

            let sequence = key_sequence![
                [2, 1, true, 10], // Press th!(A,shift)
                [2, 2, true, 30], //  press th!(S,lgui)
                [2, 3, true, 30],  //  press d
                // eager hold and output
                [2, 1, false, 50], // Release A
                [2, 2, false, 100], // Release s
                [2, 3, false, 100],  // Release d
            ];
            let expected_reports = key_report![
                [KC_LSHIFT, [0, 0, 0, 0, 0, 0]], // #0
                [KC_LSHIFT | KC_LGUI, [0, 0, 0, 0, 0, 0]],
                [KC_LSHIFT | KC_LGUI,  [kc8!(D), 0, 0, 0, 0, 0]],

                [KC_LGUI , [ kc8!(D), 0, 0, 0, 0, 0]],
                [0 , [ kc8!(D), 0, 0, 0, 0, 0]],
                [0, [0, 0, 0, 0, 0, 0]],
            ];

            run_key_sequence_test(&mut keyboard, &sequence, expected_reports).await;
        };
        block_on(main);
    }

    #[test]
    fn test_tap_hold_key_chord() {
        let main = async {
            let mut keyboard = create_test_keyboard_with_config(
                BehaviorConfig {
                    tap_hold: get_th_config_for_permissive_hold_test(),
                    ..BehaviorConfig::default()
                }
            );

            // rolling A , then ctrl d
            let sequence = key_sequence![
                [2, 1, true, 200], // +th!(A,shift)
                [2, 8, true, 50],  // +k
                [2, 1, false, 20], // -A
                [2, 8, false, 50],  // -k

            ];
            let expected_reports = key_report![
                // chord hold , should become (shift x)
                [KC_LSHIFT, [0, 0, 0, 0, 0, 0]],
                [KC_LSHIFT, [kc8!(K), 0, 0, 0, 0, 0]],
                [0, [kc8!(K), 0, 0, 0, 0, 0]],
                [0, [0, 0, 0, 0, 0, 0]],

            ];

            run_key_sequence_test(&mut keyboard, &sequence, expected_reports).await;
        };
        block_on(main);
    }

    #[test]
    fn test_tap_hold_key_chord_same_hand() {
        //core case
        //should buffer next key and output
        let main = async {
            let mut keyboard = create_test_keyboard_with_config(
                BehaviorConfig {
                    tap_hold: get_th_config_for_permissive_hold_test(),
                    ..BehaviorConfig::default()
                }
            );

            // rolling A , then ctrl d
            let sequence = key_sequence![
                [2, 1, true, 200], // +th!(A,shift)
                [2, 5, true, 50],  // +g
                [2, 1, false, 20], // -A
                [2, 5, false, 50],  // -g

            ];
            let expected_reports = key_report![
                // non chord hold
                [0, [kc8!(A), 0, 0, 0, 0, 0]], //4
                [0, [0, 0, 0, 0, 0, 0]],
                [0, [kc8!(G), 0, 0, 0, 0, 0]],
                [0, [0, 0, 0, 0, 0, 0]],
            ];

            run_key_sequence_test(&mut keyboard, &sequence, expected_reports).await;
        };
        block_on(main);
    }

    #[test]
    fn test_tap_hold_key_chord_2() {
        let main = async {
            let mut keyboard = create_test_keyboard_with_config(
                BehaviorConfig {
                    tap_hold: get_th_config_for_test(),
                    ..BehaviorConfig::default()
                }
            );

            // rolling A , then ctrl d
            let sequence = key_sequence![
                [2, 1, true, 200],  // +th!(A,shift)
                [2, 2, true, 10], // +th!(S,lgui)
                // cross hand , fire hold
                [2, 8, true, 50],  // +k
                [2, 1, false, 20], // -A
                [2, 8, false, 50], // -k
                [2, 2, false, 400], // -s
            ];
            let expected_reports = key_report![
                //multi mod chord hold
                [KC_LSHIFT, [0, 0, 0, 0, 0, 0]],
                [KC_LSHIFT| KC_LGUI, [0, 0, 0, 0, 0, 0]],
                [KC_LSHIFT| KC_LGUI, [kc8!(K), 0, 0, 0, 0, 0]],
                [ KC_LGUI, [kc8!(K), 0, 0, 0, 0, 0]],
                [KC_LGUI, [0, 0, 0, 0, 0, 0]],
                [0, [0, 0, 0, 0, 0, 0]],
            ];

            run_key_sequence_test(&mut keyboard, &sequence, expected_reports).await;
        };
        block_on(main);
    }

    #[test]
    fn test_tap_hold_key_tap_timeout() {
        let main = async {
            let mut keyboard = create_test_keyboard_with_config(
                BehaviorConfig {
                    tap_hold: get_th_config_for_test(),
                    ..BehaviorConfig::default()
                }
            );

            let sequence = key_sequence![
                [2, 3, true, 30],  //  press d
                [2, 3, false, 30], // release d
                [2, 1, true, 10], // Press th!(A,shift)
                [2, 2, true, 10], //  press th!(S,lgui)
                //hold timeout
                [2, 1, false, 40], // Release A
                [2, 2, false, 10], // Release s
            ];
            let expected_reports = key_report![
                [0, [kc8!(D), 0, 0, 0, 0, 0]],                          //shift
                [0, [0, 0, 0, 0, 0, 0]],
                [0, [kc8!(A), 0, 0, 0, 0, 0]],                          //shift
                [0, [0, 0, 0, 0, 0, 0]],
                [0, [kc8!(S), 0, 0, 0, 0, 0]],                          //shift
                [0, [0, 0, 0, 0, 0, 0]],
            ];

            run_key_sequence_test(&mut keyboard, &sequence, expected_reports).await;
        };
        block_on(main);
    }

}// forks end
