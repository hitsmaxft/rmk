use crate::{boot::jump_to_bootloader, KeyMap, event::Event, input_device::{InputProcessor, ProcessResult}};
use core::cell::{RefCell};
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};

#[derive(Debug)]
pub struct BootMagicConfig {
    boot_magic_row: u8,
    boot_magic_col: u8,
    boot_magic_timeout: u8,
}

impl BootMagicConfig {
    pub fn new(row: u8, col: u8, timeout: u8) -> Self {
        Self {
            boot_magic_row: row,
            boot_magic_col: col,
            boot_magic_timeout: timeout,
        }
    }

    pub fn get_row(&self) -> u8 {
        self.boot_magic_row
    }

    pub fn get_col(&self) -> u8 {
        self.boot_magic_col
    }

    pub fn get_timeout(&self) -> u8 {
        self.boot_magic_timeout
    }

    pub fn disable(&mut self) {
        self.boot_magic_row = u8::MAX;
        self.boot_magic_col = u8::MAX;
    }
}

pub struct BootMagicTimeout<'a> {
    config: &'a RefCell<BootMagicConfig>,
}


impl<'a> BootMagicTimeout<'a> {

    pub fn new(config: &'a RefCell<BootMagicConfig>) -> Self {
        BootMagicTimeout { config }
    }

    pub async fn wait_for_boot_magic(&mut self) {
        let timeout ;
        {

            timeout =  self.config.borrow().get_timeout();
        }

        Timer::after(Duration::from_secs(timeout.into())).await;

        self.config.borrow_mut().disable();
    }
}


pub struct BootMagicProcessor<'a, const ROW: usize, 
const COL: usize, const NUM_LAYER : usize, const NUM_ENCODER: usize> {
    config: &'a RefCell<BootMagicConfig>,
    keymap: &'a RefCell<KeyMap<'a, ROW, COL, NUM_LAYER, NUM_ENCODER>>,
}

impl<'a, const ROW: usize, const COL: usize, const NUM_LAYER: usize, const NUM_ENCODER: usize>
BootMagicProcessor<'a, ROW, COL, NUM_LAYER, NUM_ENCODER> {
    /// Create a new BootmagicProcessor
    fn new(
        config: &'a RefCell<BootMagicConfig>,
        keymap: &'a RefCell<KeyMap<'a, ROW, COL, NUM_LAYER, NUM_ENCODER>>,
    ) -> Self {
        return BootMagicProcessor{
            config,
            keymap,
        };
    }

    pub fn create_pair(
        config: &'a RefCell<BootMagicConfig>,
        keymap: &'a RefCell<KeyMap<'a, ROW, COL, NUM_LAYER, NUM_ENCODER>>,
    ) -> (BootMagicProcessor<'a, ROW, COL, NUM_LAYER, NUM_ENCODER>, BootMagicTimeout<'a>) {

        let processor = BootMagicProcessor::new(config, keymap); 

        let timeout = BootMagicTimeout::new(config);
        return (
            processor,
            timeout,
        );
    }
}

impl<'a, const ROW: usize, const COL: usize, const NUM_LAYER: usize, const NUM_ENCODER: usize>
    InputProcessor<'a, ROW, COL, NUM_LAYER, NUM_ENCODER>
    for BootMagicProcessor<'a, ROW, COL, NUM_LAYER, NUM_ENCODER>
{

    async fn process(&mut self, event: Event) -> ProcessResult {
        let _c= self.config.try_borrow();
        match event {
            Event::Key(val) => {
                match  _c {
                    Ok(_config) => {
                    if val.row == _config.boot_magic_row
                        && val.col  == _config.boot_magic_col
                    {
                        // If the key is pressed, we send a bootmagic key action
                        // TODO call rmk boot loader jumping
                        if val.pressed {
                            debug!("BootMagic key pressed, jump to bootloader");
                            jump_to_bootloader();
                            return ProcessResult::Stop;
                        }
                    };
                        return ProcessResult::Continue(event);
                    },

                    _ => {
                        panic!("borrower failed");
                    }       
                    
                }
            }
                    _ => {
                        return ProcessResult::Continue(event);
                    }       
        }
    }


    /// Get the current keymap
    fn get_keymap(&self) -> &RefCell<KeyMap<'a, ROW, COL, NUM_LAYER, NUM_ENCODER>> {
        return self.keymap;
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::config::BehaviorConfig;
    use crate::channel::KEY_EVENT_CHANNEL;
    use futures::future::join3;
    use embassy_futures::block_on;
    use crate::action::EncoderAction;
    use crate::k;
    use rusty_fork::rusty_fork_test;

    rusty_fork_test!  {
    #[test]
    fn test_boot_magic_triggered() {
        // Test the bootmagic processor

        block_on(async {
        let config = RefCell::new(BootMagicConfig::new(0, 0, 1));

        let keyMap = RefCell::new(KeyMap::new(
           &mut [[[k!(A); 1]; 1]; 0],
           None::<&mut [[EncoderAction; 0]; 0]>,
            BehaviorConfig::default(),
        ).await);

        let (mut processor, mut timeout) = BootMagicProcessor::create_pair(&config, &keyMap);
        // generate async task to wait for boot magic

            join3(
                timeout.wait_for_boot_magic(),
                async  {
                    let event = crate::event::KeyEvent {
                        row: 0,
                        col: 0,
                        pressed: true,
                    };
                    KEY_EVENT_CHANNEL.send(event).await;
                },

                async {
                    let event = KEY_EVENT_CHANNEL.receiver().receive().await;
                    println!("Received event: {:?}", event);
                    let e = crate::event::Event::Key(event);
                    let result = processor.process(e).await;
                    match result {
                        ProcessResult::Continue(_) => {
                        assert!(false, "BootMagicProcessor should stop the event");
                        }
                        ProcessResult::Stop => {
                            println!("BootMagicProcessor should stop the event");
                        }
                    }   
                }
                ).await;
        });
    }
    #[test]
    fn test_boot_magic_timeout() {
        // Test the boot magic processor

        block_on(async {
        let config = RefCell::new(BootMagicConfig::new(0, 0, 1));

        let keyMap = RefCell::new(KeyMap::new(
           &mut [[[k!(A); 1]; 1]; 0],
           None::<&mut [[EncoderAction; 0]; 0]>,
            BehaviorConfig::default(),
        ).await);

        let (mut processor, mut timeout) = BootMagicProcessor::create_pair(&config, &keyMap);
        // generate async task to wait for boot magic

            join3(
                timeout.wait_for_boot_magic(),
                async  {
                    let event = crate::event::KeyEvent {
                        row: 0,
                        col: 0,
                        pressed: true,
                    };
                    Timer::after(Duration::from_secs(2)).await;
                    KEY_EVENT_CHANNEL.send(event).await;
                },

                async {
                    let event = KEY_EVENT_CHANNEL.receiver().receive().await;
                    println!("Received event: {:?}", event);
                    let e = crate::event::Event::Key(event);
                    let result = processor.process(e).await;
                    match result {
                        ProcessResult::Stop => {
                        assert!(false, "BootMagicProcessor timeout");
                        }
                        _ => {
                            println!("BootMagicProcessor should ignore key press");
                        }
                    }   
                }
                ).await;
        });
    }

}
}