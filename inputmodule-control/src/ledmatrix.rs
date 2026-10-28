use clap::Parser;

#[derive(Clone, Copy, Debug, PartialEq, clap::ValueEnum)]
#[repr(u8)]
pub enum Pattern {
    Percentage = 0,
    Gradient = 1,
    DoubleGradient = 2,
    LotusSideways = 3,
    Zigzag = 4,
    AllOn = 5,
    Panic = 6,
    LotusTopDown = 7,
    //AllBrightnesses
}

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Debug, PartialEq, clap::ValueEnum)]
#[repr(u8)]
pub enum Game {
    Snake = 0,
    Pong = 1,
    Tetris = 2,
    GameOfLife = 3,
}

#[derive(Copy, Clone, Debug, PartialEq, clap::ValueEnum)]
pub enum GameOfLifeStartParam {
    CurrentMatrix = 0x00,
    Pattern1 = 0x01,
    Blinker = 0x02,
    Toad = 0x03,
    Beacon = 0x04,
    Glider = 0x05,
    BeaconToadBlinker = 0x06,
    RPentomino = 0x07,
    Acorn = 0x08,
    Diehard = 0x09,
    LightweightSpaceship = 0x0A,
    MiddleweightSpaceship = 0x0B,
    HeavyweightSpaceship = 0x0C,
    SpaceshipFleet = 0x0D,
    Octagon = 0x0E,
    Tumbler = 0x0F,
    Caterer = 0x10,
    Pattern2 = 0x11,
    Pattern3 = 0x12,
    Pattern4 = 0x13,
    Pattern5 = 0x14,
}

/// Animation to show when the LED Matrix powers on
#[derive(Copy, Clone, Debug, PartialEq, clap::ValueEnum)]
pub enum StartupAnimation {
    /// A random one of the animations below
    Random,
    /// No animation
    None,
    Percentage,
    ZigZag,
    GolPattern1,
    GolBeaconToadBlinker,
    GolGlider,
    Breathing,
    Pong,
    Snake,
    GolRPentomino,
    GolAcorn,
    GolDiehard,
    GolLightweightSpaceship,
    GolMiddleweightSpaceship,
    GolHeavyweightSpaceship,
    GolSpaceshipFleet,
    GolOctagon,
    GolTumbler,
    GolCaterer,
    GolPattern2,
    GolPattern3,
    GolPattern4,
    GolPattern5,
}

impl StartupAnimation {
    pub const ANIMATIONS: [StartupAnimation; 22] = [
        StartupAnimation::Percentage,
        StartupAnimation::ZigZag,
        StartupAnimation::GolPattern1,
        StartupAnimation::GolBeaconToadBlinker,
        StartupAnimation::GolGlider,
        StartupAnimation::Breathing,
        StartupAnimation::Pong,
        StartupAnimation::Snake,
        StartupAnimation::GolRPentomino,
        StartupAnimation::GolAcorn,
        StartupAnimation::GolDiehard,
        StartupAnimation::GolLightweightSpaceship,
        StartupAnimation::GolMiddleweightSpaceship,
        StartupAnimation::GolHeavyweightSpaceship,
        StartupAnimation::GolSpaceshipFleet,
        StartupAnimation::GolOctagon,
        StartupAnimation::GolTumbler,
        StartupAnimation::GolCaterer,
        StartupAnimation::GolPattern2,
        StartupAnimation::GolPattern3,
        StartupAnimation::GolPattern4,
        StartupAnimation::GolPattern5,
    ];

    /// ID of the animation in the firmware, if it's a single animation
    pub fn id(self) -> Option<u8> {
        Self::ANIMATIONS
            .iter()
            .position(|a| *a == self)
            .map(|i| i as u8)
    }
}

/// LED Matrix
#[derive(Parser, Debug)]
#[command(arg_required_else_help = true)]
pub struct LedMatrixSubcommand {
    /// Set LED max brightness percentage or get, if no value provided
    #[arg(long)]
    pub brightness: Option<Option<u8>>,

    /// Set sleep status or get, if no value provided
    #[arg(long)]
    pub sleeping: Option<Option<bool>>,

    /// Jump to the bootloader
    #[arg(long)]
    pub bootloader: bool,

    /// Display a percentage (0-100)
    #[arg(long)]
    pub percentage: Option<u8>,

    /// Start/stop animation
    #[arg(long)]
    pub animate: Option<Option<bool>>,

    /// Display a pattern
    #[arg(long)]
    #[clap(value_enum)]
    pub pattern: Option<Pattern>,

    /// Show every brightness, one per pixel
    #[arg(long)]
    pub all_brightnesses: bool,

    /// Blink the current pattern once a second
    #[arg(long)]
    pub blinking: bool,

    /// Breathing brightness of the current pattern
    #[arg(long)]
    pub breathing: bool,

    /// Display black&white image (9x34px)
    #[arg(long)]
    pub image_bw: Option<String>,

    /// Display grayscale image
    #[arg(long)]
    pub image_gray: Option<String>,

    /// Random EQ
    #[arg(long)]
    pub random_eq: bool,

    /// Display EQ of microphone input
    #[cfg(feature = "audio-visualizations")]
    #[arg(long)]
    pub input_eq: bool,

    /// EQ with custom values
    #[arg(long, num_args(9))]
    pub eq: Option<Vec<u8>>,

    /// Clock
    #[arg(long)]
    pub clock: bool,

    /// Display a string (max 5 chars)
    #[arg(long)]
    pub string: Option<String>,

    /// Display a string (max 5 symbols)
    #[arg(long, num_args(0..6))]
    pub symbols: Option<Vec<String>>,

    /// Start a game
    #[arg(long)]
    #[clap(value_enum)]
    pub start_game: Option<Game>,

    /// Paramater for starting the game. Required for some games
    #[arg(long)]
    #[clap(value_enum)]
    pub game_param: Option<GameOfLifeStartParam>,

    /// Stop the currently running game
    #[arg(long)]
    #[clap(value_enum)]
    pub stop_game: bool,

    /// Set/get animation FPS
    #[arg(long)]
    pub animation_fps: Option<Option<u16>>,

    /// Set/get PWM Frequency in Hz
    #[arg(long)]
    #[clap(value_enum)]
    pub pwm_freq: Option<Option<u16>>,

    /// Set debug mode or get current mode, if no value provided
    #[arg(long)]
    pub debug_mode: Option<Option<bool>>,

    /// Set the animation to show on startup or get current setting, if no value provided
    #[arg(long)]
    #[clap(value_enum)]
    pub startup_animation: Option<Option<StartupAnimation>>,

    /// On startup, show a random animation out of this comma separated list
    #[arg(long, value_delimiter = ',', conflicts_with = "startup_animation")]
    #[clap(value_enum)]
    pub startup_animation_set: Option<Vec<StartupAnimation>>,

    /// Crash the firmware (TESTING ONLY!)
    #[arg(long)]
    pub panic: bool,

    /// Get the device version
    #[arg(short, long)]
    pub version: bool,
}
