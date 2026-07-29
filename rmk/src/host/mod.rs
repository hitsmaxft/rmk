#[cfg(feature = "storage")]
pub(crate) mod storage;
pub mod via;

use core::cell::RefCell;

// TODO: Remove those aliases
pub use via::UsbVialReaderWriter as UsbHostReaderWriter;
#[cfg(feature = "vial")]
pub(crate) use via::VialService as HostService;

#[cfg(feature = "vial")]
use crate::config::VialConfig;
use crate::descriptor::ViaReport;
use crate::hid::{HidReaderTrait, HidWriterTrait};
use crate::keymap::KeyMap;

/// Run the host (Via/Vial) communication task over any `ViaReport` transport.
///
/// This is transport-agnostic on purpose: it only needs a reader/writer of
/// `ViaReport` and is independent of how key reports reach the host. Firmware
/// that composes its own task set — for example a keyboard whose HID output
/// goes over a proprietary radio while Via/Vial stays on USB — can run this
/// directly instead of going through [`crate::run_rmk`].
#[cfg(feature = "vial")]
pub async fn run_host_communicate_task<
    'a,
    Rw: HidReaderTrait<ReportType = ViaReport> + HidWriterTrait<ReportType = ViaReport>,
    const ROW: usize,
    const COL: usize,
    const NUM_LAYER: usize,
    const NUM_ENCODER: usize,
>(
    keymap: &'a RefCell<KeyMap<'a, ROW, COL, NUM_LAYER, NUM_ENCODER>>,
    reader_writer: Rw,
    vial_config: VialConfig<'static>,
) {
    let mut service = HostService::new(keymap, vial_config, reader_writer);
    service.run().await
}

#[cfg(not(feature = "vial"))]
pub async fn run_host_communicate_task<
    'a,
    Rw: HidReaderTrait<ReportType = ViaReport> + HidWriterTrait<ReportType = ViaReport>,
    const ROW: usize,
    const COL: usize,
    const NUM_LAYER: usize,
    const NUM_ENCODER: usize,
>(
    _keymap: &'a RefCell<KeyMap<'a, ROW, COL, NUM_LAYER, NUM_ENCODER>>,
    _reader_writer: Rw,
) {
    todo!()
}
