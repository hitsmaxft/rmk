#![no_std]
#![no_main]

#[macro_use]
mod macros;
mod keymap;
mod vial;

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::{
    self as _, bind_interrupts,
    gpio::{AnyPin, Input, Output},
    interrupt::{self, InterruptExt, Priority},
    peripherals::{self, SAADC},
    saadc::{self, AnyInput, Input as _, Saadc},
    usb::{self, vbus_detect::SoftwareVbusDetect, Driver},
};
use core::cell::RefCell;
use panic_probe as _;
use rmk::{
    ble::SOFTWARE_VBUS,
    channel::EVENT_CHANNEL,
    config::{
        BleBatteryConfig, ControllerConfig, KeyboardUsbConfig, RmkConfig, StorageConfig, VialConfig,
    },
    debounce::default_debouncer::DefaultDebouncer,
    futures::future::{join, join3, join5, join4},
    initialize_keymap_and_storage, initialize_nrf_sd_and_flash,
    input_device::{
        adc::{AnalogEventType, NrfAdc}, battery::BatteryProcessor, boot_magic::{self, BootMagicConfig}, rotary_encoder::{DefaultPhase, RotaryEncoder, RotaryEncoderProcessor}, Runnable
    },
    keyboard::Keyboard,
    light::LightController,
    run_devices, run_processor_chain, run_rmk,
    split::central::{run_peripheral_manager, CentralMatrix},
};

use keymap::{COL as ALL_COL, ROW, NUM_LAYER};
const COL : usize = ALL_COL / 2;

use vial::{VIAL_KEYBOARD_DEF, VIAL_KEYBOARD_ID};

bind_interrupts!(struct Irqs {
    USBD => usb::InterruptHandler<peripherals::USBD>;
});


#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello NRF BLE!");
    let mut nrf_config = embassy_nrf::config::Config::default();
    nrf_config.gpiote_interrupt_priority = Priority::P3;
    nrf_config.time_interrupt_priority = Priority::P3;
    interrupt::USBD.set_priority(interrupt::Priority::P2);
    interrupt::CLOCK_POWER.set_priority(interrupt::Priority::P2);
    let p = embassy_nrf::init(nrf_config);
    // Disable external HF clock by default, reduce power consumption
    // info!("Enabling ext hfosc...");
    // ::embassy_nrf::pac::CLOCK.tasks_hfclkstart().write_value(1);
    // while ::embassy_nrf::pac::CLOCK.events_hfclkstarted().read() != 1 {}

    // Usb config
    let software_vbus = SOFTWARE_VBUS.get_or_init(|| SoftwareVbusDetect::new(true, false));
    let driver = Driver::new(p.USBD, Irqs, software_vbus);

    // Keyboard config
    let keyboard_usb_config = KeyboardUsbConfig {
        vid: 0x4c4b,
        pid: 0x4643,
        manufacturer: "bhe",
        product_name: "Gynus Ble",
        serial_number: "vial:f64c2b3c:000001",
    };
    let vial_config = VialConfig::new(VIAL_KEYBOARD_ID, VIAL_KEYBOARD_DEF);

    let storage_config = StorageConfig {
        start_addr: 0,
        num_sectors: 6,
        clear_storage: true,
        ..Default::default()
    };
    let rmk_config = RmkConfig {
        usb_config: keyboard_usb_config,
        vial_config,
        storage_config,
        ..Default::default()
    };

    // default nicenano pin mapping, col2row
    //## promicro pin: 4 5 6 7
    //input_pins = ["P1_00", "P0_11", "P1_04", "P1_06"]
    //promicro pin: 21 20 19 18 15 14
    //output_pins = ["P0_31", "P0_29", "P0_02", "P1_15", "P1_13", "P1_11"]    

    //lin's pcb pin mapping
    let (input_pins, output_pins) = config_matrix_pins_nrf!(peripherals: p, 
        // to row
        input: [P1_00, P0_11, P1_04, P1_06], 
        // from col 
        output:  [
            // P0_31, 
        P0_29, P0_02, P1_15, P1_13, P1_11]
    );

    // Initialize the Softdevice and flash
    let central_addr = [0x18, 0xe2, 0x21, 0x80, 0xc0, 0xc7];
    let peripheral_addr = [0x7e, 0xfe, 0x73, 0x9e, 0x66, 0xe3];

    let (sd, flash) = initialize_nrf_sd_and_flash(
        rmk_config.usb_config.product_name,
        spawner,
        Some(central_addr),
    );

    // Initialize the storage and keymap
    let mut default_keymap = keymap::get_default_keymap();
    let (keymap, storage) = initialize_keymap_and_storage(
        &mut default_keymap,
        flash,
        rmk_config.storage_config,
        rmk_config.behavior_config.clone(),
    )
    .await;
// 
    // Initialize the matrix + keyboard
    let debouncer = DefaultDebouncer::<ROW, ALL_COL>::new();
    let mut matrix = CentralMatrix::<_, _, _, 0, 0, ROW, COL>::new(input_pins, output_pins, debouncer);
    let mut keyboard = Keyboard::new(&keymap, rmk_config.behavior_config.clone());

    // Initialize the light controller
    let light_controller: LightController<Output> =
        LightController::new(ControllerConfig::default().light_config);


    // let bootmagic_config = RefCell::new(boot_magic::BootMagicConfig::new(0, 0, 2));

    // let (mut bootmagic_processor, mut boot_magic_timeout) = boot_magic::BootMagicProcessor::create_pair(
    //     &bootmagic_config, &keymap
    // );

    // Start
    join4(
        // boot_magic_timeout.wait_for_boot_magic(),
        run_devices! (
            (
                matrix
            ) => EVENT_CHANNEL,
        ),
        // run_processor_chain! {
        //     EVENT_CHANNEL => [bootmagic_processor]
        // },
        keyboard.run(),
        run_peripheral_manager::<ROW, COL, 0, 0>(0, peripheral_addr),
        run_rmk(&keymap, driver, storage, light_controller, rmk_config, sd)
    )
    .await;
}
