use crate::animations::*;
use crate::control::PwmFreqArg;
use crate::games::game_of_life::GameOfLifeState;
use crate::games::pong::PongState;
use crate::games::snake::SnakeState;

pub const WIDTH: usize = 9;
pub const HEIGHT: usize = 34;
pub const LEDS: usize = WIDTH * HEIGHT;

#[derive(Clone)]
pub struct Grid(pub [[u8; HEIGHT]; WIDTH]);
impl Default for Grid {
    fn default() -> Self {
        Grid([[0; HEIGHT]; WIDTH])
    }
}

impl Grid {
    pub fn rotate(&mut self, rotations: usize) {
        for x in 0..WIDTH {
            self.0[x].rotate_right(rotations);
        }
    }
}

pub struct LedmatrixState {
    /// Currently displayed grid
    pub grid: Grid,
    /// Temporary buffer for building a new grid
    pub col_buffer: Grid,
    /// Whether the grid is currently being animated
    pub animate: bool,
    /// LED brightness out of 255
    pub brightness: u8,
    /// Current sleep state
    pub sleeping: SleepState,
    /// State of the current game, if any
    pub game: Option<GameState>,
    pub animation_period: u64,
    /// Current LED PWM frequency
    pub pwm_freq: PwmFreqArg,
    /// Whether debug mode is active
    ///
    /// In debug mode:
    /// - Startup is instant, no animation
    /// - Sleep/wake transition is instant, no animation/fading
    /// - No automatic sleeping
    pub debug_mode: bool,
    pub upcoming_frames: Option<Animation>,
    /// How much current the LEDs get, 0-255 of `max_scaling` (GlobalCurrent command)
    pub global_current: u8,
    /// The most current (LED scaling) this board allows. Never exceeded.
    pub max_scaling: u8,
}

/// The LED controller's scaling for the current global_current setting, never above max_scaling
pub fn led_scaling(state: &LedmatrixState) -> u8 {
    let scaling = (state.global_current as u16 * state.max_scaling as u16) / 255;
    if state.global_current > 0 && scaling == 0 {
        1
    } else {
        scaling as u8
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
/// Whether asleep or not, if asleep contains data to restore previous LED grid
pub enum SleepState {
    Awake,
    Sleeping((Grid, u8)),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SleepReason {
    Command,
    SleepPin,
    Timeout,
    UsbSuspend,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
/// State that's used for each game
pub enum GameState {
    Snake(SnakeState),
    Pong(PongState),
    GameOfLife(GameOfLifeState),
}
