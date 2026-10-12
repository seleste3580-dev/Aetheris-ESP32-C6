#![no_std]

//! Aetheris device-independent interface and application catalog.

pub const OS_NAME: &str = "Aetheris";
pub const VERSION: &str = "0.1.0";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct App {
    pub name: &'static str,
    pub icon: char,
}

/// Built-in launch catalog. Apps marked as planned in the README have menu
/// entries and UI models; hardware/network-dependent implementations follow.
pub const APPS: [App; 28] = [
    App {
        name: "Piano",
        icon: '♫',
    },
    App {
        name: "Terminal",
        icon: '>',
    },
    App {
        name: "Matrix",
        icon: '#',
    },
    App {
        name: "Clock",
        icon: '◷',
    },
    App {
        name: "Timer",
        icon: '◴',
    },
    App {
        name: "Stopwatch",
        icon: '◉',
    },
    App {
        name: "Notes",
        icon: '▤',
    },
    App {
        name: "Calculator",
        icon: '±',
    },
    App {
        name: "IR Remote",
        icon: '⌁',
    },
    App {
        name: "Buzzer",
        icon: '♪',
    },
    App {
        name: "Wi-Fi",
        icon: '⌁',
    },
    App {
        name: "System Info",
        icon: 'ⓘ',
    },
    App {
        name: "Settings",
        icon: '⚙',
    },
    App {
        name: "Wallpaper",
        icon: '▧',
    },
    App {
        name: "Counter",
        icon: '▣',
    },
    App {
        name: "Coin Flip",
        icon: 'ⓢ',
    },
    App {
        name: "Wi-Fi Scan",
        icon: '⌕',
    },
    App {
        name: "Sound Test",
        icon: '♫',
    },
    App {
        name: "Key Test",
        icon: '⌨',
    },
    App {
        name: "Display Test",
        icon: '▦',
    },
    App {
        name: "Security",
        icon: '▣',
    },
    App {
        name: "Update",
        icon: '↑',
    },
    App {
        name: "Reboot",
        icon: '↻',
    },
    App {
        name: "Power",
        icon: '⏻',
    },
    App {
        name: "Dice",
        icon: '⚄',
    },
    App {
        name: "Metronome",
        icon: '♫',
    },
    App {
        name: "Snake",
        icon: '▰',
    },
    App {
        name: "Reaction Test",
        icon: '!',
    },
];

/// Telephone multi-tap groups for the 3x4 numeric keypad.
pub const T9: [&str; 12] = [
    "", "", "abc", "def", "ghi", "jkl", "mno", "pqrs", "tuv", "wxyz", "*", "#",
];

/// Convert a piano key index into equal-tempered frequency (A4 = 440 Hz).
/// Index zero is C4; indices 0..=23 cover two chromatic octaves, including sharps.
pub fn piano_frequency_hz(index: u8) -> u16 {
    const NOTES: [u16; 24] = [
        262, 277, 294, 311, 330, 349, 370, 392, 415, 440, 466, 494, 523, 554, 587, 622, 659, 698,
        740, 784, 831, 880, 932, 988,
    ];
    NOTES[(index as usize) % NOTES.len()]
}

/// Small deterministic boot animation frame: a digital rain glyph derived from
/// column and frame. The UI can render this in a terminal-style matrix effect.
pub fn matrix_glyph(column: u8, frame: u8) -> char {
    const RAIN: [char; 8] = ['0', '1', '0', '1', 'A', 'B', 'C', 'D'];
    RAIN[((column.wrapping_mul(13).wrapping_add(frame)) as usize) % RAIN.len()]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Login,
    Home,
    Menu,
    Settings,
    Piano,
}

pub struct UiState {
    pub screen: Screen,
    pub selection: usize,
    pub wallpaper: u8,
    pub logged_in: bool,
}

impl UiState {
    pub const fn new() -> Self {
        Self {
            screen: Screen::Login,
            selection: 0,
            wallpaper: 0,
            logged_in: false,
        }
    }

    pub fn move_selection(&mut self, delta: i8) {
        let len = if matches!(self.screen, Screen::Menu) {
            APPS.len()
        } else {
            4
        };
        self.selection = if delta < 0 {
            self.selection.wrapping_add(len - 1) % len
        } else {
            (self.selection + 1) % len
        };
    }
}
