pub mod test_macro;

use embassy_futures::block_on;
use embassy_futures::select::select;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Timer};
use futures::{join, FutureExt};
use rusty_fork::rusty_fork_test;

use rmk::action::KeyAction;
use rmk::config::{BehaviorConfig, CombosConfig, ForksConfig};
use rmk::fork::Fork;
use rmk::hid_state::HidModifiers;
use rmk::{a, k, layer, mo, th};

use rmk::hid::Report;
use rmk::channel::{KEYBOARD_REPORT_CHANNEL, KEY_EVENT_CHANNEL};
use rmk::combo::Combo;
use rmk::event::{KeyEvent, PressedKeyEvent};
use rmk::input_device::Runnable;
use rmk::keycode::{KeyCode, ModifierCombination};
use rmk::keymap::KeyMap;

#[derive(Debug, Clone)]
pub struct TestKeyPress {
    row: u8,
    col: u8,
    pressed: bool,
    delay: u64, // Delay before this key event in milliseconds
}

pub async fn run_key_sequence_test<'a, const N: usize>(
    keyboard: &mut Keyboard<'a, 5, 14, 2>,
    key_sequence: &[TestKeyPress],
    expected_reports: Vec<KeyboardReport, N>,
) {
    static REPORTS_DONE: Mutex<CriticalSectionRawMutex, bool> = Mutex::new(false);

    KEY_EVENT_CHANNEL.clear();
    KEYBOARD_REPORT_CHANNEL.clear();
    static MAX_TEST_TIMEOUT: Duration = Duration::from_secs(5);

    join!(
        // Run keyboard until all reports are received
        async {
            select(keyboard.run(), async {
                select(
                    Timer::after(MAX_TEST_TIMEOUT).then(|_| async {
                        panic!("Test time out reached");
                    }),
                    async {
                        while !*REPORTS_DONE.lock().await {
                            // polling reports
                            Timer::after(Duration::from_millis(5)).await;
                        }
                    },
                )
                .await;
            })
            .await;
        },
        // Send all key events with delays
        async {
            for key in key_sequence {
                Timer::after(Duration::from_millis(key.delay)).await;
                KEY_EVENT_CHANNEL
                    .send(KeyEvent {
                        row: key.row,
                        col: key.col,
                        pressed: key.pressed,
                    })
                    .await;
            }
        },
        // Verify reports
        async {
            let mut report_index = -1;
            for expected in expected_reports {
                match KEYBOARD_REPORT_CHANNEL.receive().await {
                    Report::KeyboardReport(report) => {
                        report_index += 1;
                        // println!("Received {}th report from channel: {:?}", report_index, report);
                        assert_eq!(
                            expected, report,
                            "on {}th reports, expected left but actually right",
                            report_index
                        );
                    }
                    _ => panic!("Expected a KeyboardReport"),
                }
            }
            // Set done flag after all reports are verified
            *REPORTS_DONE.lock().await = true;
        }
    );
}

    // Init logger for tests
    #[ctor::ctor]
    pub fn init_log() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .try_init();
    }

    #[rustfmt::skip]
    pub const fn get_keymap() -> [[[KeyAction; 14]; 5]; 2] {
        [
            layer!([
                [k!(Grave), k!(Kc1), k!(Kc2), k!(Kc3), k!(Kc4), k!(Kc5), k!(Kc6), k!(Kc7), k!(Kc8), k!(Kc9), k!(Kc0), k!(Minus), k!(Equal), k!(Backspace)],
                [k!(Tab), k!(Q), k!(W), k!(E), k!(R), k!(T), k!(Y), k!(U), k!(I), k!(O), k!(P), k!(LeftBracket), k!(RightBracket), k!(Backslash)],
                [k!(Escape), th!(A, LShift), th!(S, LGui), k!(D), k!(F), k!(G), k!(H), k!(J), k!(K), k!(L), k!(Semicolon), k!(Quote), a!(No), k!(Enter)],
                [k!(LShift), k!(Z), k!(X), k!(C), k!(V), k!(B), k!(N), k!(M), k!(Comma), k!(Dot), k!(Slash), a!(No), a!(No), k!(RShift)],
                [k!(LCtrl), k!(LGui), k!(LAlt), a!(No), a!(No), k!(Space), a!(No), a!(No), a!(No), mo!(1), k!(RAlt), a!(No), k!(RGui), k!(RCtrl)]
            ]),
            layer!([
                [k!(Grave), k!(F1), k!(F2), k!(F3), k!(F4), k!(F5), k!(F6), k!(F7), k!(F8), k!(F9), k!(F10), k!(F11), k!(F12), k!(Delete)],
                [a!(No), a!(Transparent), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No)],
                [k!(CapsLock), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No)],
                [a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), k!(Up)],
                [a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), k!(Left), a!(No), k!(Down), k!(Right)]
            ]),
        ]
    }

    #[rustfmt::skip]
    pub fn get_combos_config() -> CombosConfig {
        // Define the function to return the appropriate combo configuration
        CombosConfig {
            combos: Vec::from_iter([
                Combo::new(
                    [
                        k!(V), //3,4
                        k!(B), //3,5
                    ]
                    .to_vec(),
                    k!(LShift),
                    Some(0),
                ),
                Combo::new(
                    [
                        k!(R), //1,4
                        k!(T), //1,5
                    ]
                    .to_vec(),
                    k!(LAlt),
                    Some(0),
                ),
            ]),
            timeout: Duration::from_millis(100),
        }
    }

    pub fn create_test_keyboard_with_config(config: BehaviorConfig) -> Keyboard<'static, 5, 14, 2> {
        // Box::leak is acceptable in tests
        let keymap = Box::new(get_keymap());
        let leaked_keymap = Box::leak(keymap);

        let keymap = block_on(KeyMap::new(leaked_keymap, None, config.clone()));
        let keymap_cell = RefCell::new(keymap);
        let keymap_ref = Box::leak(Box::new(keymap_cell));

        Keyboard::new(keymap_ref, config)
    }

    pub fn create_test_keyboard() -> Keyboard<'static, 5, 14, 2> {
        create_test_keyboard_with_config(BehaviorConfig::default())
    }

    pub fn key_event(row: u8, col: u8, pressed: bool) -> KeyEvent {
        KeyEvent { row, col, pressed }
    }