extern crate rmk;

// a rust macro to map a str to k!(a) as u8
#[macro_export]
macro_rules! kc8 {
    ($key: ident) => {
        rmk::keycode::KeyCode::$key as u8
    };
}

// a rust macro to create a key sequence to simulate key presses
#[macro_export]
macro_rules! key_sequence {
($([$row:expr, $col:expr, $pressed:expr, $delay:expr]),* $(,)?) => {
    vec![
        $(
            $crate::common::TestKeyPress {
                row: $row,
                col: $col,
                pressed: $pressed,
                delay: $delay,
            },
        )*
    ]
};
}

// a rust macro to create a key report that simulates key status change in hid
#[macro_export]
macro_rules! key_report {
( $([$modifier:expr, $keys:expr]),* $(,)? ) => {
    {
    // Count the number of elements at compile time

    const N: usize = {
        let arr = [$((($modifier, $keys)),)*];
        arr.len()
    };


    let mut reports: heapless::Vec<rmk::usb::descriptor::KeyboardReport, N> = heapless::Vec::new();
    $(
        reports.push(rmk::usb::descriptor::KeyboardReport {
            modifier: $modifier,
            keycodes: $keys,
            leds: 0,
            reserved: 0,
        }).unwrap();
    )*
    reports
    }

}}
