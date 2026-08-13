use core::sync::atomic::{AtomicU32, Ordering};

const MAGIC: u32 = 0x524d_4241; // "RMBA"

pub(crate) const CONNECTED: usize = 1;
pub(crate) const DISCONNECTED: usize = 2;
pub(crate) const PAIRING_COMPLETE: usize = 3;
pub(crate) const PAIRING_FAILED: usize = 4;
pub(crate) const PASSKEY_INPUT: usize = 5;
pub(crate) const GATT_EVENTS: usize = 6;
pub(crate) const GATT_READS: usize = 7;
pub(crate) const GATT_WRITES: usize = 8;
pub(crate) const HID_CCCD_WRITES: usize = 9;
pub(crate) const HID_NOTIFY_ATTEMPTS: usize = 10;
pub(crate) const HID_NOTIFY_SUCCESSES: usize = 11;
pub(crate) const HID_NOTIFY_FAILURES: usize = 12;
pub(crate) const LAST_SECURITY_LEVEL: usize = 13;
pub(crate) const LED_OUTPUT_WRITES: usize = 14;
pub(crate) const LAST_LED_BITS: usize = 15;
pub(crate) const PROFILE_UPDATES: usize = 16;
pub(crate) const PROFILE_FLASH_SUCCESSES: usize = 17;
pub(crate) const PROFILE_FLASH_FAILURES: usize = 18;
pub(crate) const LOADED_BONDS: usize = 19;
pub(crate) const LOAD_ERRORS: usize = 20;
pub(crate) const PAIRING_LTK_BASE: usize = 21;
pub(crate) const LOADED_LTK_BASE: usize = 25;
pub(crate) const LOADED_IDENTITY_LOW: usize = 29;
pub(crate) const LOADED_METADATA: usize = 30;
pub(crate) const LOAD_EPOCH: usize = 31;

/// Persistent hardware-acceptance counters. The array is NOLOAD so a debugger
/// reset can stop the target without erasing the just-completed connection.
#[used]
#[unsafe(export_name = "RMK_CH582M_BLE_ACCEPTANCE_DIAGNOSTICS")]
#[unsafe(link_section = ".uninit.rmk_ble_acceptance")]
static DIAGNOSTICS: [AtomicU32; 32] = [const { AtomicU32::new(0) }; 32];

#[inline]
fn ensure() {
    if DIAGNOSTICS[0].load(Ordering::Relaxed) != MAGIC {
        for word in &DIAGNOSTICS[1..] {
            word.store(0, Ordering::Relaxed);
        }
        DIAGNOSTICS[0].store(MAGIC, Ordering::Release);
    }
}

#[inline]
pub(crate) fn increment(index: usize) {
    ensure();
    let value = DIAGNOSTICS[index].load(Ordering::Relaxed);
    DIAGNOSTICS[index].store(value.wrapping_add(1), Ordering::Relaxed);
}

#[inline]
pub(crate) fn store(index: usize, value: u32) {
    ensure();
    DIAGNOSTICS[index].store(value, Ordering::Relaxed);
}

pub(crate) fn store_ltk(base: usize, value: trouble_host::LongTermKey) {
    for (index, chunk) in value.to_le_bytes().chunks_exact(4).enumerate() {
        store(base + index, u32::from_le_bytes(chunk.try_into().unwrap()));
    }
}

pub(crate) fn begin_bond_load() {
    ensure();
    for word in &DIAGNOSTICS[LOADED_BONDS..PAIRING_LTK_BASE] {
        word.store(0, Ordering::Relaxed);
    }
    for word in &DIAGNOSTICS[LOADED_LTK_BASE..LOADED_METADATA + 1] {
        word.store(0, Ordering::Relaxed);
    }
    increment(LOAD_EPOCH);
}
