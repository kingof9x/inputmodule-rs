//! White Rabbit startup animations
//!
//! Six extra startup animations: Rabbit hole, Digital rain, Hourglass, Warp speed, Power on and
//! Campfire. They only use whole-number maths, since the RP2040 has no floating point unit.
//! Each one plays for 4 to 6 seconds at the normal 32 frames per second and ends on a dark panel.
use crate::matrix::{Grid, HEIGHT, WIDTH};

/// Small pseudo random number generator (xorshift32)
pub struct Rng(u32);

impl Rng {
    pub fn new(seed: u8) -> Self {
        // Never 0, xorshift would get stuck there
        Self(0x9E37_79B9 ^ ((seed as u32) << 8) ^ seed as u32)
    }

    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }

    /// A number from 0 up to, but not including, `n`
    pub fn below(&mut self, n: u32) -> u32 {
        (self.next_u32() >> 8) % n
    }

    /// A number from 0 to 1024
    pub fn unit(&mut self) -> i32 {
        self.below(1025) as i32
    }
}

/// Whole-number square root
fn isqrt(n: u32) -> u32 {
    if n < 2 {
        return n;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

/// sin(i * 2pi / 32) * 256
const SINE: [i32; 32] = [
    0, 50, 98, 142, 181, 213, 237, 251, 256, 251, 237, 213, 181, 142, 98, 50, 0, -50, -98, -142,
    -181, -213, -237, -251, -256, -251, -237, -213, -181, -142, -98, -50,
];

const RABBIT: [&[u8; 7]; 10] = [
    b"0110110", b"0110110", b"0110110", b"0110110", b"0111110", b"1111111", b"1101011", b"1111111",
    b"0111110", b"0011100",
];

/// Draw the rabbit with its top left corner at (x, y). Parts outside the panel are left out.
fn stamp_rabbit(grid: &mut Grid, x: i32, y: i32, level: u8, hide_from_row: i32) {
    for (dy, row) in RABBIT.iter().enumerate() {
        let yy = y + dy as i32;
        if yy < 0 || yy >= HEIGHT as i32 || yy >= hide_from_row {
            continue;
        }
        for (dx, c) in row.iter().enumerate() {
            let xx = x + dx as i32;
            if *c == b'1' && (0..WIDTH as i32).contains(&xx) {
                let led = &mut grid.0[xx as usize][yy as usize];
                *led = (*led).max(level);
            }
        }
    }
}

fn is_rabbit(x: usize, y: usize, top: usize) -> bool {
    // The rabbit drawn at (1, top)
    y >= top && y < top + RABBIT.len() && (1..8).contains(&x) && RABBIT[y - top][x - 1] == b'1'
}

/// The grid at `num` / `den` of its brightness
fn scaled(grid: &Grid, num: u32, den: u32) -> Grid {
    let mut out = grid.clone();
    for col in out.0.iter_mut() {
        for led in col.iter_mut() {
            *led = (*led as u32 * num / den) as u8;
        }
    }
    out
}

/// Frame `i` of a fade to dark that takes `n` frames
fn fade(grid: &Grid, i: u32, n: u32) -> Grid {
    scaled(grid, n - 1 - i.min(n - 1), n)
}

fn filled(level: u8) -> Grid {
    Grid([[level; HEIGHT]; WIDTH])
}

/// The rabbit hops down the panel and dives into a hole. Rings ripple out and fade.
#[derive(Default)]
pub struct RabbitHoleIterator {
    frame: u32,
}

const HOLE_Y: i32 = 29;
const HOPS: [i32; 5] = [-9, 2, 9, 16, 22];

impl RabbitHoleIterator {
    fn hole(open: u32) -> Grid {
        // An oval 3.2 LEDs wide and 0.6 times as tall, opening over 10 frames
        let r10 = (open as i32 + 1) * 32 / 10;
        let mut grid = Grid::default();
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                let dx = x as i32 - 4;
                let dy = y as i32 - HOLE_Y;
                if 3600 * dx * dx + 10000 * dy * dy <= 36 * r10 * r10 {
                    grid.0[x][y] = 64;
                }
            }
        }
        grid
    }

    fn ripples(k: i32) -> Grid {
        let hole = Self::hole(9);
        let mut grid = scaled(&hole, (20 - k.min(20)) as u32, 20);
        for ring in 0..3 {
            let r10 = k * 11 - ring * 60;
            if r10 <= 0 {
                continue;
            }
            for x in 0..WIDTH {
                for y in 0..HEIGHT {
                    let dx = (x as i32 - 4) * 16;
                    let dy = (y as i32 - HOLE_Y) * 10;
                    let d10 = isqrt((dx * dx + dy * dy) as u32) as i32;
                    let near = (12 - (d10 - r10).abs()).max(0);
                    let far = (340 - r10).max(0);
                    let v = (255 * near * far / (12 * 340)) as u8;
                    grid.0[x][y] = grid.0[x][y].max(v);
                }
            }
        }
        grid
    }

    fn draw(t: u32) -> Option<Grid> {
        // Hole opens: 10 frames, hops: 4 x 12, dive: 12, ripples: 36, fade: 6, dark: 8
        if t < 10 {
            return Some(Self::hole(t));
        }
        let t = t - 10;
        if t < 48 {
            let hop = (t / 12) as usize;
            // 10 frames in the air, 2 frames sitting
            let k = (t % 12).min(9) as i32;
            let (start, end) = (HOPS[hop], HOPS[hop + 1]);
            // Straight line plus an arc 3 LEDs high, in hundredths
            let y100 = start * 100 + (end - start) * (k + 1) * 10 - 12 * (k + 1) * (9 - k);
            let mut grid = Self::hole(9);
            stamp_rabbit(
                &mut grid,
                1,
                (y100 + 50).div_euclid(100),
                255,
                HEIGHT as i32,
            );
            return Some(grid);
        }
        let t = t - 48;
        if t < 12 {
            let mut grid = Self::hole(9);
            stamp_rabbit(&mut grid, 1, HOPS[4] + t as i32, 255, HOLE_Y - 1);
            return Some(grid);
        }
        let t = t - 12;
        if t < 36 {
            return Some(Self::ripples(t as i32));
        }
        let t = t - 36;
        if t < 6 {
            return Some(fade(&Self::ripples(35), t, 6));
        }
        if t < 6 + 8 {
            return Some(Grid::default());
        }
        None
    }
}

impl Iterator for RabbitHoleIterator {
    type Item = Grid;

    fn next(&mut self) -> Option<Grid> {
        let grid = Self::draw(self.frame);
        self.frame += 1;
        grid
    }
}

/// Code rain falls down the panel. Drops that cross the rabbit stay lit until the rabbit shows.
pub struct DigitalRainIterator {
    frame: u32,
    rng: Rng,
    /// x, y in 1/256 LEDs, speed in 1/256 LEDs per frame; y < 0 means the slot is free
    drops: [(u8, i32, i32); 96],
    trail: Grid,
    lit: Grid,
}

const RAIN_RABBIT_TOP: usize = 12;

impl DigitalRainIterator {
    pub fn new(seed: u8) -> Self {
        Self {
            frame: 0,
            rng: Rng::new(seed),
            drops: [(0, -1, 0); 96],
            trail: Grid::default(),
            lit: Grid::default(),
        }
    }

    fn rabbit(level: u8) -> Grid {
        let mut grid = Grid::default();
        stamp_rabbit(&mut grid, 1, RAIN_RABBIT_TOP as i32, level, HEIGHT as i32);
        grid
    }
}

impl Iterator for DigitalRainIterator {
    type Item = Grid;

    fn next(&mut self) -> Option<Grid> {
        // Rain: 110 frames, the rabbit glows: 16, fades: 14, dark: 6
        let t = self.frame;
        self.frame += 1;
        if t < 110 {
            if t < 80 {
                for _ in 0..(if t < 50 { 2 } else { 3 }) {
                    if self.rng.below(10) < 8 {
                        let x = self.rng.below(WIDTH as u32) as u8;
                        let speed = 128 + self.rng.below(205) as i32;
                        if let Some(slot) = self.drops.iter_mut().find(|d| d.1 < 0) {
                            *slot = (x, 0, speed);
                        }
                    }
                }
            }
            for col in self.trail.0.iter_mut() {
                for led in col.iter_mut() {
                    *led = (*led as u32 * 184 / 256) as u8;
                }
            }
            for drop in self.drops.iter_mut() {
                if drop.1 < 0 {
                    continue;
                }
                drop.1 += drop.2;
                let y = (drop.1 >> 8) as usize;
                if y >= HEIGHT {
                    drop.1 = -1;
                    continue;
                }
                let x = drop.0 as usize;
                self.trail.0[x][y] = 255;
                if t > 20 && is_rabbit(x, y, RAIN_RABBIT_TOP) {
                    self.lit.0[x][y] = 230;
                }
            }
            let mut grid = scaled(&self.trail, 4, 5);
            for x in 0..WIDTH {
                for y in 0..HEIGHT {
                    grid.0[x][y] = grid.0[x][y].max(self.lit.0[x][y]);
                }
            }
            return Some(grid);
        }
        let t = t - 110;
        if t < 16 {
            // 0.9 + 0.1 * sin(t / 2)
            const GLOW: [u8; 16] = [
                230, 242, 251, 255, 253, 245, 233, 221, 210, 205, 205, 212, 223, 235, 247, 254,
            ];
            self.trail = scaled(&self.trail, 3, 5);
            let mut grid = scaled(&self.trail, 3, 5);
            let rabbit = Self::rabbit(GLOW[t as usize]);
            for x in 0..WIDTH {
                for y in 0..HEIGHT {
                    grid.0[x][y] = grid.0[x][y].max(rabbit.0[x][y]);
                }
            }
            return Some(grid);
        }
        let t = t - 16;
        if t < 14 {
            return Some(fade(&Self::rabbit(230), t, 14));
        }
        if t < 14 + 6 {
            return Some(Grid::default());
        }
        None
    }
}

/// Sand runs from the top of an hourglass through the narrow middle and piles up at the bottom.
pub struct HourglassIterator {
    frame: u32,
    rng: Rng,
    sand: [[bool; HEIGHT]; WIDTH],
    /// Frame at which the sand finished running, then it holds and fades
    done_at: Option<u32>,
    last: Grid,
}

const NECK: usize = 17;

fn is_glass(x: usize, y: usize) -> bool {
    // Glass wherever |x - 4| > |y - 17| / 17 * 4.6 + 0.6
    170 * (x as i32 - 4).abs() > 46 * (y as i32 - NECK as i32).abs() + 102
}

impl HourglassIterator {
    pub fn new(seed: u8) -> Self {
        let mut sand = [[false; HEIGHT]; WIDTH];
        for x in 0..WIDTH {
            for y in 2..NECK - 2 {
                sand[x][y] = !is_glass(x, y);
            }
        }
        Self {
            frame: 0,
            rng: Rng::new(seed),
            sand,
            done_at: None,
            last: Grid::default(),
        }
    }

    fn free(&self, x: usize, y: usize) -> bool {
        !self.sand[x][y] && !is_glass(x, y)
    }

    fn step(&mut self, t: u32) {
        let mut order = [0, 1, 2, 3, 4, 5, 6, 7, 8];
        for i in (1..WIDTH).rev() {
            order.swap(i, self.rng.below(i as u32 + 1) as usize);
        }
        for y in (0..HEIGHT - 1).rev() {
            for &x in order.iter() {
                if !self.sand[x][y] {
                    continue;
                }
                // The neck only lets a grain through every other frame
                if y == NECK - 1 && t % 2 == 1 {
                    continue;
                }
                if self.free(x, y + 1) {
                    self.sand[x][y] = false;
                    self.sand[x][y + 1] = true;
                    continue;
                }
                let sides = if self.rng.below(2) == 0 {
                    [-1, 1]
                } else {
                    [1, -1]
                };
                for dx in sides {
                    let nx = x as i32 + dx;
                    if !(0..WIDTH as i32).contains(&nx) {
                        continue;
                    }
                    let nx = nx as usize;
                    if self.free(nx, y + 1) && !is_glass(nx, y) {
                        self.sand[x][y] = false;
                        self.sand[nx][y + 1] = true;
                        break;
                    }
                }
            }
        }
    }

    fn draw(&self) -> Grid {
        let mut grid = Grid::default();
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                grid.0[x][y] = if self.sand[x][y] {
                    255
                } else if is_glass(x, y) {
                    31
                } else {
                    0
                };
            }
        }
        grid
    }
}

impl Iterator for HourglassIterator {
    type Item = Grid;

    fn next(&mut self) -> Option<Grid> {
        let t = self.frame;
        self.frame += 1;
        match self.done_at {
            None => {
                self.step(t);
                self.last = self.draw();
                let top_empty = (0..WIDTH).all(|x| (0..NECK).all(|y| !self.sand[x][y]));
                if (t > 40 && top_empty) || t >= 260 {
                    self.done_at = Some(t + 1);
                }
                Some(self.last.clone())
            }
            Some(done) => {
                // Hold: 10 frames, fade: 12, dark: 6
                let t = t - done;
                if t < 10 {
                    Some(self.last.clone())
                } else if t < 22 {
                    Some(fade(&self.last, t - 10, 12))
                } else if t < 28 {
                    Some(Grid::default())
                } else {
                    None
                }
            }
        }
    }
}

/// Stars rush out from the middle, faster and faster, then the panel flashes white.
pub struct WarpSpeedIterator {
    frame: u32,
    rng: Rng,
    /// Direction (index into SINE) and distance from the middle in 1/256 LEDs; 0 means a free slot
    stars: [(u8, u32); 96],
    trail: Grid,
}

impl WarpSpeedIterator {
    pub fn new(seed: u8) -> Self {
        Self {
            frame: 0,
            rng: Rng::new(seed),
            stars: [(0, 0); 96],
            trail: Grid::default(),
        }
    }
}

impl Iterator for WarpSpeedIterator {
    type Item = Grid;

    fn next(&mut self) -> Option<Grid> {
        // Stars: 90 frames, flash up: 6, flash down: 18, dark: 6
        let t = self.frame;
        self.frame += 1;
        if t < 90 {
            // Speed in 1/10000, from 0.02 to 0.14
            let speed = 200 + t * 1200 / 90;
            for _ in 0..1 + t / 25 {
                let direction = self.rng.below(32) as u8;
                if let Some(slot) = self.stars.iter_mut().find(|s| s.1 == 0) {
                    *slot = (direction, 128);
                }
            }
            let keep = if t < 70 { 141 } else { 192 };
            self.trail = scaled(&self.trail, keep, 256);
            for star in self.stars.iter_mut() {
                if star.1 == 0 {
                    continue;
                }
                star.1 += star.1 * speed * 22 / 100_000 + speed * 512 / 10_000;
                let r = star.1 as i32;
                let sin = SINE[star.0 as usize];
                let cos = SINE[(star.0 as usize + 8) % 32];
                // Squashed sideways, since the panel is so narrow
                let x = (4 * 256 + 128 + cos * r * 3 / 10 / 256) >> 8;
                let y = (16 * 256 + 128 + 128 + sin * r / 256) >> 8;
                if (0..WIDTH as i32).contains(&x) && (0..HEIGHT as i32).contains(&y) {
                    // Brighter the further out it is
                    let level = (77 + r * 255 / (14 * 256)).min(255) as u8;
                    self.trail.0[x as usize][y as usize] = level;
                } else {
                    star.1 = 0;
                }
            }
            return Some(self.trail.clone());
        }
        let t = t - 90;
        if t < 6 {
            return Some(filled(((t + 1) * 255 / 6) as u8));
        }
        let t = t - 6;
        if t < 18 {
            return Some(filled((255 - (t + 1) * 255 / 18) as u8));
        }
        if t < 18 + 6 {
            return Some(Grid::default());
        }
        None
    }
}

/// An old TV switching on: a dot, a line, the picture opens, a scan line rolls down twice,
/// then it switches off again.
#[derive(Default)]
pub struct PowerOnIterator {
    frame: u32,
}

fn fill_rect(grid: &mut Grid, x0: usize, x1: usize, y0: usize, y1: usize, level: u8) {
    for col in grid.0[x0..x1].iter_mut() {
        for led in col[y0..y1].iter_mut() {
            *led = level;
        }
    }
}

impl PowerOnIterator {
    fn draw(t: u32) -> Option<Grid> {
        let mut grid = Grid::default();
        // Dot: 8 frames, line: 6, open: 10, scan lines: 68, close: 10, dot fades: 8, dark: 6
        if t < 8 {
            fill_rect(&mut grid, 4, 5, 16, 18, ((t + 1) * 255 / 8) as u8);
            return Some(grid);
        }
        let t = t - 8;
        if t < 6 {
            let w = ((t + 1) * 4 + 3) / 6;
            fill_rect(&mut grid, 4 - w as usize, 5 + w as usize, 16, 18, 255);
            return Some(grid);
        }
        let t = t - 6;
        if t < 10 {
            let h = (((t + 1) * 17 + 5) / 10) as usize;
            fill_rect(&mut grid, 0, WIDTH, 17 - h, 17 + h, 140);
            fill_rect(&mut grid, 0, WIDTH, 16, 18, 255);
            return Some(grid);
        }
        let t = t - 10;
        if t < 68 {
            let y = (t % 34) as usize;
            grid = filled(89);
            fill_rect(&mut grid, 0, WIDTH, y, y + 1, 255);
            if y > 0 {
                fill_rect(&mut grid, 0, WIDTH, y - 1, y, 179);
            }
            if y > 1 {
                fill_rect(&mut grid, 0, WIDTH, y - 2, y - 1, 128);
            }
            return Some(grid);
        }
        let t = t - 68;
        if t < 10 {
            let h = (((9 - t) * 17 + 5) / 10) as usize;
            fill_rect(&mut grid, 0, WIDTH, 17 - h, 17 + h, 89);
            fill_rect(&mut grid, 3, 6, 16, 18, 255);
            return Some(grid);
        }
        let t = t - 10;
        if t < 8 {
            fill_rect(&mut grid, 4, 5, 16, 18, (255 - (t + 1) * 255 / 8) as u8);
            return Some(grid);
        }
        if t < 8 + 6 {
            return Some(grid);
        }
        None
    }
}

impl Iterator for PowerOnIterator {
    type Item = Grid;

    fn next(&mut self) -> Option<Grid> {
        let grid = Self::draw(self.frame);
        self.frame += 1;
        grid
    }
}

/// Flames light at the bottom, roar up the panel, then die down to glowing embers.
pub struct CampfireIterator {
    frame: u32,
    rng: Rng,
    /// Heat from 0 to 1024, with two extra rows of fuel below the panel
    heat: [[i32; HEIGHT + 2]; WIDTH],
    last: Grid,
}

impl CampfireIterator {
    pub fn new(seed: u8) -> Self {
        Self {
            frame: 0,
            rng: Rng::new(seed),
            heat: [[0; HEIGHT + 2]; WIDTH],
            last: Grid::default(),
        }
    }

    fn burn(&mut self, t: i32) -> Grid {
        let power = if t < 95 {
            (t * 1024 / 25).min(1024)
        } else {
            (1024 - (t - 95) * 1024 / 40).max(0)
        };
        for x in 0..WIDTH {
            for y in HEIGHT..HEIGHT + 2 {
                let fuel = isqrt(self.rng.unit() as u32 * 1024) as i32;
                self.heat[x][y] = fuel * power / 1024;
            }
        }
        let old = self.heat;
        for x in 0..WIDTH {
            let left = x.saturating_sub(1);
            let right = (x + 1).min(WIDTH - 1);
            for y in 0..HEIGHT {
                let below = old[x][y + 1] * 2
                    + old[left][y + 1]
                    + old[right][y + 1]
                    + old[x][(y + 2).min(HEIGHT + 1)];
                let u = self.rng.unit();
                let cool = (u * u / 1024 * u / 1024) * 10 / 100 + 4 + (1024 - power) * 5 / 100;
                self.heat[x][y] = (below / 5 - cool).clamp(0, 1024);
            }
        }
        let mut grid = Grid::default();
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                let mut v = self.heat[x][y];
                if t > 105 && y == HEIGHT - 1 && self.rng.below(2) == 0 {
                    // Embers
                    let ember = self.rng.unit() / 2 * (45 - (t - 105)).max(0) / 45;
                    v = v.max(ember);
                }
                // Brighter, with a curve so the flame tips stay dim
                let v = (v * 5 / 4).min(1024) as u32;
                grid.0[x][y] = (255 * v * isqrt(v * 1024) / (1024 * 1024)) as u8;
            }
        }
        grid
    }
}

impl Iterator for CampfireIterator {
    type Item = Grid;

    fn next(&mut self) -> Option<Grid> {
        // Fire: 150 frames, fade: 6, dark: 6
        let t = self.frame;
        self.frame += 1;
        if t < 150 {
            self.last = self.burn(t as i32);
            return Some(self.last.clone());
        }
        let t = t - 150;
        if t < 6 {
            return Some(fade(&self.last, t, 6));
        }
        if t < 12 {
            return Some(Grid::default());
        }
        None
    }
}
