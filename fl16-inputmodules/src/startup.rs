//! Startup animation setting, persisted in flash
//!
//! The setting lives in its own 4K flash sector, right below the serial number.
//! See `flash_layout.md`.
use crate::animations::*;
use crate::control::GameOfLifeStartParam;
use crate::games::pong_animation::*;
use crate::games::snake_animation::*;
use num::FromPrimitive;

const FLASH_OFFSET: usize = 0x10000000;
/// Offset of the settings sector, relative to the start of flash
const SETTINGS_BLOCK: u32 = 0xfe000;
const SECTOR_SIZE: u32 = 0x1000;
/// Smallest amount of flash that can be programmed at once
const PAGE_SIZE: usize = 256;
const MAGIC: [u8; 4] = *b"FWST";
const RECORD_VERSION: u8 = 1;

/// Number of built-in startup animations
pub const STARTUP_ANIMATION_COUNT: u8 = 8;

#[derive(Copy, Clone, PartialEq, Eq, num_derive::FromPrimitive)]
pub enum StartupMode {
    /// Randomly pick any of the animations (default)
    RandomAll = 0x00,
    /// Always show the same animation
    Fixed = 0x01,
    /// Randomly pick one of the animations in the set
    RandomSet = 0x02,
    /// Don't show any animation
    None = 0x03,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct StartupAnimationConfig {
    pub mode: StartupMode,
    /// Animation to show in `Fixed` mode
    pub animation: u8,
    /// Bitmask of animations to choose from in `RandomSet` mode
    pub set: u8,
}

impl Default for StartupAnimationConfig {
    fn default() -> Self {
        Self {
            mode: StartupMode::RandomAll,
            animation: 0,
            set: 0xFF,
        }
    }
}

impl StartupAnimationConfig {
    pub fn from_args(mode: u8, animation: u8, set: u8) -> Option<Self> {
        let mode = FromPrimitive::from_u8(mode)?;
        if mode == StartupMode::Fixed && animation >= STARTUP_ANIMATION_COUNT {
            return None;
        }
        if mode == StartupMode::RandomSet && set == 0 {
            return None;
        }
        Some(Self {
            mode,
            animation,
            set,
        })
    }

    pub fn to_bytes(self) -> [u8; 3] {
        [self.mode as u8, self.animation, self.set]
    }

    /// Pick the animation to show according to this setting
    pub fn choose(&self, random: u8) -> Option<Animation> {
        match self.mode {
            StartupMode::RandomAll => Some(startup_animation(random % STARTUP_ANIMATION_COUNT)),
            StartupMode::Fixed => Some(startup_animation(self.animation)),
            StartupMode::RandomSet => {
                let count = self.set.count_ones() as u8;
                if count == 0 {
                    return Some(startup_animation(random % STARTUP_ANIMATION_COUNT));
                }
                // Find the n-th animation that's in the set
                let mut n = random % count;
                for i in 0..STARTUP_ANIMATION_COUNT {
                    if self.set & (1 << i) != 0 {
                        if n == 0 {
                            return Some(startup_animation(i));
                        }
                        n -= 1;
                    }
                }
                None
            }
            StartupMode::None => None,
        }
    }
}

pub fn startup_animation(index: u8) -> Animation {
    match index {
        0 => Animation::Percentage(StartupPercentageIterator::default()),
        1 => Animation::ZigZag(ZigZagIterator::default()),
        2 => Animation::Gof(GameOfLifeIterator::new(GameOfLifeStartParam::Pattern1, 200)),
        3 => Animation::Gof(GameOfLifeIterator::new(
            GameOfLifeStartParam::BeaconToadBlinker,
            128,
        )),
        4 => Animation::Gof(GameOfLifeIterator::new(GameOfLifeStartParam::Glider, 128)),
        5 => Animation::Breathing(BreathingIterator::default()),
        6 => Animation::Pong(PongIterator::default()),
        _ => Animation::Snake(SnakeIterator::default()),
    }
}

fn checksum(data: &[u8]) -> u32 {
    let crc: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
    crc.checksum(data)
}

/// Read the setting from flash. Falls back to the default if nothing valid was saved.
pub fn load_startup_config() -> StartupAnimationConfig {
    // Flash is mapped into memory, just read it from there
    let ptr = (FLASH_OFFSET + SETTINGS_BLOCK as usize) as *const [u8; 12];
    let raw = unsafe { core::ptr::read_volatile(ptr) };

    let actual_checksum = u32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]);
    if raw[0..4] != MAGIC || raw[4] != RECORD_VERSION || checksum(&raw[0..8]) != actual_checksum {
        return StartupAnimationConfig::default();
    }
    StartupAnimationConfig::from_args(raw[5], raw[6], raw[7]).unwrap_or_default()
}

/// Save the setting to flash
///
/// Takes about 50ms, during which interrupts are disabled.
pub fn save_startup_config(config: &StartupAnimationConfig) {
    if load_startup_config() == *config {
        // Avoid needlessly wearing out the flash
        return;
    }

    let mut page = [0xFFu8; PAGE_SIZE];
    page[0..4].copy_from_slice(&MAGIC);
    page[4] = RECORD_VERSION;
    page[5..8].copy_from_slice(&config.to_bytes());
    let crc = checksum(&page[0..8]);
    page[8..12].copy_from_slice(&crc.to_le_bytes());

    // Nothing may read from flash while it's being written.
    // The second core is never started, so it's still waiting in ROM.
    cortex_m::interrupt::free(|_cs| unsafe {
        rp2040_flash::flash::flash_range_erase(SETTINGS_BLOCK, SECTOR_SIZE, true);
        rp2040_flash::flash::flash_range_program(SETTINGS_BLOCK, &page, true);
    });
}
