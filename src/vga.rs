#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Colour {
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    LightGray = 0x7,
    DarkGray = 0x8,
    LightBlue = 0x9,
    LightGreen = 0xa,
    LightCyan = 0xb,
    LightRed = 0xc,
    Pink = 0xd,
    Yellow = 0xe,
    White = 0xf,
}

fn colour_as_byte(fg: Colour, bg: Colour) -> u8 {
    fg as u8 | ((bg as u8) << 4)
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ScreenChar {
    character: u8,
    colour: u8,
}

impl ScreenChar {
    fn new(c: u8, fg: Colour, bg: Colour) -> Self {
        Self {
            character: c,
            colour: colour_as_byte(fg, bg),
        }
    }
}

const WIDTH: usize = 80;
const HEIGHT: usize = 25;
type VGABuffer = [[ScreenChar; WIDTH]; HEIGHT];

pub struct Screen {
    buffer: &'static mut VGABuffer,
    foreground: Colour,
    background: Colour,
    x: usize,
    y: usize,
}

impl Default for Screen {
    fn default() -> Self {
        Self::with_colours(Colour::White, Colour::Black)
    }
}

impl Screen {
    pub fn with_colours(fg: Colour, bg: Colour) -> Self {
        let mut s = Self {
            buffer: unsafe { &mut *(0xb8000 as *mut VGABuffer) },
            foreground: fg,
            background: bg,
            x: 0,
            y: 0,
        };
        s.clear();
        s
    }

    pub fn set_colours<FG, BG>(&mut self, fg: FG, bg: BG)
    where
        FG: Into<Option<Colour>>,
        BG: Into<Option<Colour>>,
    {
        if let Some(fg) = fg.into() {
            self.foreground = fg
        }
        if let Some(bg) = bg.into() {
            self.background = bg
        }
    }

    fn screen_char(&self, c: u8) -> ScreenChar {
        ScreenChar::new(c, self.foreground, self.background)
    }

    pub fn clear(&mut self) {
        let sc = self.screen_char(b' ');
        let len = (WIDTH * HEIGHT) as isize;
        let buf: *mut ScreenChar = self.buffer as *mut VGABuffer as *mut ScreenChar;

        for i in 0..len {
            unsafe {
                *buf.offset(i) = sc;
            }
        }

        self.x = 0;
        self.y = 0;
    }

    pub fn scroll_down(&mut self) {
		// move all rows up
		for row in 1..HEIGHT {
			for col in 0..WIDTH {
				self.buffer[row-1][col] = self.buffer[row][col];
			}
		}

		// clear bottom row
		let blank = self.screen_char(b' ');
		for col in 0..WIDTH {
			self.buffer[HEIGHT-1][col] = blank;
		}

		if self.y > 0 {
			self.y -= 1;
		}
    }

    pub fn new_line(&mut self) {
        self.x = 0;
        self.y += 1;
        if self.y >= HEIGHT {
            self.scroll_down();
        }
    }

    pub fn write_byte(&mut self, b: u8) {
        // limit to ascii
        let b = match b {
            32...176 | b'\n' => b,
            _ => b'?',
        };

        let new_line = b == b'\n';

        // print char
        if !new_line {
            let c = self.screen_char(b);
            self.buffer[self.y][self.x] = c;
            self.x += 1;
        }

        // wrap
        if new_line || self.x >= WIDTH {
            self.new_line();
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for b in s.bytes() {
            self.write_byte(b);
        }
    }
}
