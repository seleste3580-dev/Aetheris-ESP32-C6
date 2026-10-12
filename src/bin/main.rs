#![no_std]
#![no_main]
#![deny(clippy::mem_forget, clippy::large_stack_frames)]

use core::cell::RefCell;
use embassy_executor::Spawner;
use embassy_time::{Instant, Timer};
use embedded_graphics::{
    mono_font::{
        MonoTextStyle,
        ascii::{FONT_5X7, FONT_6X10},
    },
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, Ellipse, Line, PrimitiveStyle, Rectangle},
    text::{Alignment, Text},
};
use embedded_hal_bus::i2c::RefCellDevice;
use esp_hal::{
    clock::CpuClock,
    gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull},
    i2c::master::{Config as I2cConfig, I2c},
    time::Rate,
    timer::timg::TimerGroup,
};
use esp_radio::wifi::{
    AuthenticationMethodConfig, Config as WifiConfig, Ssid, WifiController, scan::ScanConfig,
    sta::StationConfig,
};
use log::{error, info};
use ssd1306::{I2CDisplayInterface, Ssd1306, prelude::*};

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("Aetheris panic: {}", info);
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

// Generated from esp-generate 1.4.0 for ESP32-C6 WROOM-1, Embassy, alloc,
// Wi-Fi and log. BLE from the starter template was intentionally removed.

const KEYMAP: [[u8; 4]; 4] = [*b"123A", *b"456B", *b"789C", *b"*0#D"];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Login,
    Home,
    Menu,
    Settings,
    WifiSetup,
    Piano,
    App,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EditField {
    Username,
    Password,
}

struct State {
    screen: Screen,
    username: [u8; 16],
    username_len: usize,
    password: [u8; 16],
    password_len: usize,
    account_username: [u8; 16],
    account_username_len: usize,
    account_password: [u8; 16],
    account_password_len: usize,
    edit_field: EditField,
    tap_key: u8,
    tap_index: usize,
    last_tap_ms: u64,
    uppercase: bool,
    selection: usize,
    wallpaper: u8,
    wifi_connected: bool,
    networks: [[u8; 32]; 8],
    network_lens: [u8; 8],
    network_count: usize,
    network_selected: usize,
    wifi_password: [u8; 64],
    wifi_password_len: usize,
    wifi_edit_password: bool,
    piano_octave: u8,
    piano_chord: bool,
    app_value: u32,
    app_accum: u32,
    app_op: u8,
    app_running: bool,
    app_started_ms: u64,
    app_counter: u32,
    note: [u8; 80],
    note_len: usize,
    last_key: u8,
    ir_code: u32,
    ir_count: u32,
    screen_off: bool,
    dice_seed: u32,
    dice_sides: u8,
    dice_result: u8,
    dice_rolling: bool,
    dice_started_ms: u64,
    dice_sound_step: u64,
    coin_flipping: bool,
    coin_started_ms: u64,
    coin_sound_step: u64,
    metronome_bpm: u16,
    metronome_running: bool,
    metronome_next_ms: u64,
    metronome_beat: u8,
    snake_body: [u8; 40],
    snake_len: u8,
    snake_direction: u8,
    snake_food: u8,
    snake_running: bool,
    snake_over: bool,
    snake_ate: bool,
    snake_last_ms: u64,
    reaction_phase: u8,
    reaction_started_ms: u64,
    reaction_ready_ms: u64,
    reaction_wait_ms: u64,
    reaction_result_ms: u32,
}

impl State {
    fn new() -> Self {
        let mut username = [0; 16];
        username[..8].copy_from_slice(b"aetheris");
        let mut password = [0; 16];
        password[..4].copy_from_slice(b"1234");
        let mut account_username = [0; 16];
        account_username[..8].copy_from_slice(b"aetheris");
        let mut account_password = [0; 16];
        account_password[..4].copy_from_slice(b"1234");
        Self {
            screen: Screen::Login,
            username,
            username_len: 8,
            password,
            password_len: 4,
            account_username,
            account_username_len: 8,
            account_password,
            account_password_len: 4,
            edit_field: EditField::Password,
            tap_key: 0,
            tap_index: 0,
            last_tap_ms: 0,
            uppercase: false,
            selection: 0,
            wallpaper: 0,
            wifi_connected: false,
            networks: [[0; 32]; 8],
            network_lens: [0; 8],
            network_count: 0,
            network_selected: 0,
            wifi_password: [0; 64],
            wifi_password_len: 0,
            wifi_edit_password: false,
            piano_octave: 0,
            piano_chord: false,
            app_value: 60,
            app_accum: 0,
            app_op: 0,
            app_running: false,
            app_started_ms: 0,
            app_counter: 0,
            note: [0; 80],
            note_len: 0,
            last_key: 0,
            ir_code: 0,
            ir_count: 0,
            screen_off: false,
            dice_seed: 0xA37E_2159,
            dice_sides: 6,
            dice_result: 1,
            dice_rolling: false,
            dice_started_ms: 0,
            dice_sound_step: u64::MAX,
            coin_flipping: false,
            coin_started_ms: 0,
            coin_sound_step: u64::MAX,
            metronome_bpm: 120,
            metronome_running: false,
            metronome_next_ms: 0,
            metronome_beat: 0,
            snake_body: [0; 40],
            snake_len: 0,
            snake_direction: 1,
            snake_food: 0,
            snake_running: false,
            snake_over: false,
            snake_ate: false,
            snake_last_ms: 0,
            reaction_phase: 0,
            reaction_started_ms: 0,
            reaction_ready_ms: 0,
            reaction_wait_ms: 0,
            reaction_result_ms: 0,
        }
    }
}

impl State {
    fn active_text(&mut self) -> (&mut [u8], &mut usize) {
        match self.edit_field {
            EditField::Username => (&mut self.username, &mut self.username_len),
            EditField::Password => (&mut self.password, &mut self.password_len),
        }
    }

    fn insert_phone_key(&mut self, key: u8, now_ms: u64) {
        let upper = self.uppercase;
        match self.edit_field {
            EditField::Username => t9_insert(
                &mut self.username,
                &mut self.username_len,
                key,
                upper,
                &mut self.tap_key,
                &mut self.tap_index,
                &mut self.last_tap_ms,
                now_ms,
            ),
            EditField::Password => t9_insert(
                &mut self.password,
                &mut self.password_len,
                key,
                upper,
                &mut self.tap_key,
                &mut self.tap_index,
                &mut self.last_tap_ms,
                now_ms,
            ),
        }
    }

    fn delete_char(&mut self) {
        let (buf, len) = self.active_text();
        if *len > 0 {
            *len -= 1;
            buf[*len] = 0;
        }
        self.tap_key = 0;
    }

    fn clear_field(&mut self) {
        let (buf, len) = self.active_text();
        *len = 0;
        buf.fill(0);
        self.tap_key = 0;
    }
}

fn t9_insert(
    buffer: &mut [u8],
    len: &mut usize,
    key: u8,
    uppercase: bool,
    tap_key: &mut u8,
    tap_index: &mut usize,
    last_ms: &mut u64,
    now_ms: u64,
) {
    const GROUPS: [&[u8]; 10] = [
        b" ", b".,?!1", b"abc2", b"def3", b"ghi4", b"jkl5", b"mno6", b"pqrs7", b"tuv8", b"wxyz9",
    ];
    if !(b'0'..=b'9').contains(&key) {
        return;
    }
    let group = GROUPS[(key - b'0') as usize];
    if *tap_key == key && now_ms.saturating_sub(*last_ms) < 900 && *len > 0 {
        *tap_index = (*tap_index + 1) % group.len();
        let mut ch = group[*tap_index];
        if uppercase {
            ch = ch.to_ascii_uppercase();
        }
        buffer[*len - 1] = ch;
    } else if *len < buffer.len() {
        *tap_index = 0;
        let mut ch = group[0];
        if uppercase {
            ch = ch.to_ascii_uppercase();
        }
        buffer[*len] = ch;
        *len += 1;
    }
    *tap_key = key;
    *last_ms = now_ms;
}

fn draw<D>(display: &mut D, state: &State, wifi: bool)
where
    D: DrawTarget<Color = BinaryColor>,
{
    let fg = if state.wallpaper == 1 {
        BinaryColor::Off
    } else {
        BinaryColor::On
    };
    let bg = if state.wallpaper == 1 {
        BinaryColor::On
    } else {
        BinaryColor::Off
    };
    let _ = display.clear(bg);
    if state.wallpaper % 4 == 2 {
        let _ = Line::new(Point::new(0, 15), Point::new(0, 63)).into_styled(PrimitiveStyle::with_stroke(fg, 1)).draw(display);
        let _ = Line::new(Point::new(127, 15), Point::new(127, 63)).into_styled(PrimitiveStyle::with_stroke(fg, 1)).draw(display);
    } else if state.wallpaper % 4 == 3 {
        for y in [19, 31, 43, 55] {
            let _ = Circle::new(Point::new(0, y), 2).into_styled(PrimitiveStyle::with_fill(fg)).draw(display);
            let _ = Circle::new(Point::new(125, y), 2).into_styled(PrimitiveStyle::with_fill(fg)).draw(display);
        }
    }
    let text = MonoTextStyle::new(&FONT_6X10, fg);
    let small = MonoTextStyle::new(&FONT_5X7, fg);
    let large = MonoTextStyle::new(&embedded_graphics::mono_font::ascii::FONT_10X20, fg);

    // A clean status/header strip leaves most of the 128x64 panel for content.
    let title = match state.screen {
        Screen::Login => "AETHERIS",
        Screen::Home => "HOME",
        Screen::Menu => "APPS",
        Screen::Settings => "SETTINGS",
        Screen::WifiSetup => "WI-FI",
        Screen::Piano => "PIANO",
        Screen::App => aetheris::APPS[state.selection % aetheris::APPS.len()].name,
    };
    let _ = Text::new(title, Point::new(2, 10), text).draw(display);
    draw_wifi_icon(display, fg, wifi);
    let _ = Line::new(Point::new(0, 13), Point::new(127, 13))
        .into_styled(PrimitiveStyle::with_stroke(fg, 1))
        .draw(display);

    match state.screen {
        Screen::Login => {
            let _ = Text::new("SIGN IN", Point::new(2, 25), text).draw(display);
            let _ = Text::new("USER", Point::new(2, 37), small).draw(display);
            let _ = Text::new("PASS", Point::new(2, 50), small).draw(display);
            let active_y = if state.edit_field == EditField::Username { 28 } else { 41 };
            let _ = Rectangle::new(Point::new(31, active_y), Size::new(95, 13))
                .into_styled(PrimitiveStyle::with_stroke(fg, 1))
                .draw(display);
            if let Ok(s) = core::str::from_utf8(&state.username[..state.username_len.min(15)]) {
                let _ = Text::new(s, Point::new(34, 38), text).draw(display);
            }
            let stars = [b'*'; 16];
            if let Ok(s) = core::str::from_utf8(&stars[..state.password_len.min(15)]) {
                let _ = Text::new(s, Point::new(34, 51), text).draw(display);
            }
            let _ = Text::new("A FIELD B DEL D CASE # OK", Point::new(1, 63), small).draw(display);
        }
        Screen::Home => {
            let _ = Text::with_alignment("AETHERIS", Point::new(64, 36), large, Alignment::Center)
                .draw(display);
            let _ = Text::with_alignment("Powered by Seleste", Point::new(64, 46), small, Alignment::Center)
                .draw(display);
            let _ = Text::with_alignment("A APPS   B SETUP", Point::new(64, 55), small, Alignment::Center)
                .draw(display);
            let _ = Text::with_alignment("D PIANO  # DICE", Point::new(64, 63), small, Alignment::Center)
                .draw(display);
        }
        Screen::Menu => {
            let app_index = state.selection % aetheris::APPS.len();
            draw_app_icon(display, fg, app_index, Point::new(8, 22));
            let app = aetheris::APPS[app_index].name;
            let _ = Text::new(app, Point::new(38, 36), text).draw(display);
            let mut count = [b'0'; 5];
            let n = app_index + 1;
            count[0] = b'0' + ((n / 10) % 10) as u8;
            count[1] = b'0' + (n % 10) as u8;
            count[2] = b'/';
            count[3] = b'0' + ((aetheris::APPS.len() / 10) % 10) as u8;
            count[4] = b'0' + (aetheris::APPS.len() % 10) as u8;
            if let Ok(s) = core::str::from_utf8(&count) {
                let _ = Text::new(s, Point::new(38, 49), small).draw(display);
            }
            let _ = Text::new("2/8 NEXT  # OPEN  * BACK", Point::new(1, 63), small).draw(display);
        }
        Screen::Settings => {
            draw_app_icon(display, fg, 12, Point::new(5, 20));
            let _ = Text::new("A WALLPAPER", Point::new(34, 29), text).draw(display);
            let _ = Text::new("B WI-FI SETUP", Point::new(34, 41), text).draw(display);
            let _ = Text::new("C REBOOT   * BACK", Point::new(1, 63), small).draw(display);
            let wallpaper = match state.wallpaper % 4 {
                0 => "MATRIX",
                1 => "INVERT",
                2 => "GRID",
                _ => "DOTS",
            };
            let _ = Text::new(wallpaper, Point::new(34, 55), small).draw(display);
        }
        Screen::WifiSetup => {
            if !state.wifi_edit_password {
                let _ = Text::new("A/B SELECT  D SCAN", Point::new(1, 25), small).draw(display);
                if state.network_count == 0 {
                    let _ = Text::with_alignment("NO NETWORKS", Point::new(64, 43), text, Alignment::Center).draw(display);
                    let _ = Text::new("# SCAN  * BACK", Point::new(1, 63), small).draw(display);
                } else {
                    let selected = state.network_selected % state.network_count;
                    if let Ok(name) = core::str::from_utf8(&state.networks[selected][..state.network_lens[selected] as usize]) {
                        let _ = Text::with_alignment(name, Point::new(64, 43), text, Alignment::Center).draw(display);
                    }
                    let _ = Text::new("# CONNECT  D RESCAN  * BACK", Point::new(1, 63), small).draw(display);
                }
            } else {
                let _ = Text::new("PASSWORD (PHONE T9)", Point::new(1, 26), small).draw(display);
                let stars = [b'*'; 20];
                if let Ok(s) = core::str::from_utf8(&stars[..state.wifi_password_len.min(20)]) {
                    let _ = Text::new(s, Point::new(2, 43), text).draw(display);
                }
                let _ = Text::new("A JOIN  B DEL  # BACK", Point::new(1, 63), small).draw(display);
            }
        }
        Screen::Piano => {
            let _ = Text::new("C4-B5  1-9,0,A,B", Point::new(1, 27), small).draw(display);
            let _ = Text::with_alignment("C C# D D# E F", Point::new(64, 41), text, Alignment::Center).draw(display);
            let _ = Text::with_alignment("F# G G# A A# B", Point::new(64, 52), text, Alignment::Center).draw(display);
            let _ = Text::new("C/D OCT  # CHORD  * BACK", Point::new(1, 63), small).draw(display);
        }
        Screen::App => {
            let index = state.selection % aetheris::APPS.len();
            if index != 26 { draw_app_icon(display, fg, index, Point::new(2, 17)); }
            match index {
                2 => {
                    for x in 0..18 {
                        let ch = aetheris::matrix_glyph(x, (Instant::now().as_millis() / 100) as u8);
                        let mut b = [0u8; 1]; b[0] = ch as u8;
                        if let Ok(s) = core::str::from_utf8(&b) { let _ = Text::new(s, Point::new(x as i32 * 7, 43), small).draw(display); }
                    }
                }
                3 => {
                    let secs = Instant::now().as_secs() % 86_400;
                    let hour = secs / 3600; let minute = (secs / 60) % 60;
                    let _ = Text::with_alignment("UPTIME CLOCK", Point::new(64, 31), small, Alignment::Center).draw(display);
                    let mut buf = [b'0'; 5]; buf[0]=b'0'+(hour/10) as u8; buf[1]=b'0'+(hour%10) as u8; buf[2]=b':'; buf[3]=b'0'+(minute/10) as u8; buf[4]=b'0'+(minute%10) as u8;
                    if let Ok(s)=core::str::from_utf8(&buf) { let _=Text::with_alignment(s,Point::new(64,52),large,Alignment::Center).draw(display); }
                }
                4 | 5 => {
                    let elapsed = (Instant::now().as_millis().saturating_sub(state.app_started_ms) / 1000) as u32;
                    let value = if state.app_running && index == 4 { state.app_counter.saturating_sub(elapsed) } else if state.app_running { state.app_counter.saturating_add(elapsed) } else { state.app_counter };
                    let mut buf=[b'0';5]; let mm=value/60; let ss=value%60;
                    buf[0]=b'0'+((mm/10)%10) as u8;buf[1]=b'0'+(mm%10) as u8;buf[2]=b':';buf[3]=b'0'+(ss/10) as u8;buf[4]=b'0'+(ss%10) as u8;
                    if let Ok(s)=core::str::from_utf8(&buf) { let _=Text::with_alignment(s,Point::new(64,45),large,Alignment::Center).draw(display); }
                    let _=Text::new("A/B SET  # START  C RESET",Point::new(1,63),small).draw(display);
                }
                6 => {
                    let start=state.note_len.saturating_sub(14);
                    if let Ok(s)=core::str::from_utf8(&state.note[start..state.note_len]) { let _=Text::new(s,Point::new(2,42),text).draw(display); }
                    let _=Text::new("2-9 TYPE  B DEL  C CLEAR",Point::new(1,63),small).draw(display);
                }
                7 => {
                    let mut buf=[b'0';8];let mut n=state.app_value;
                    for i in (0..8).rev(){buf[i]=b'0'+(n%10) as u8;n/=10;}
                    if let Ok(s)=core::str::from_utf8(&buf){let _=Text::with_alignment(s,Point::new(64,45),text,Alignment::Center).draw(display);}
                    let _=Text::new("A +  B -  C x  D /  # =",Point::new(1,63),small).draw(display);
                }
                8 => {
                    let mut c=[b'0';4];let mut n=state.ir_count;for i in (0..4).rev(){c[i]=b'0'+(n%10) as u8;n/=10;}
                    if let Ok(s)=core::str::from_utf8(&c){let _=Text::with_alignment(s,Point::new(64,45),large,Alignment::Center).draw(display);}
                    let _=Text::with_alignment("IR REMOTE PRESSES",Point::new(64,59),small,Alignment::Center).draw(display);
                }
                9 => {
                    let _=Text::with_alignment("1-9,0,A,B PLAY TONES",Point::new(64,42),text,Alignment::Center).draw(display);
                    let _=Text::with_alignment("C/D LOW / HIGH",Point::new(64,56),small,Alignment::Center).draw(display);
                }
                11 => {
                    let _=Text::new("ESP32-C6  RISC-V",Point::new(34,30),text).draw(display);
                    let _=Text::new(if wifi{"WI-FI CONNECTED"}else{"WI-FI OFFLINE"},Point::new(34,43),small).draw(display);
                }
                14 => {
                    let _=Text::new("A +1  B -1",Point::new(34,30),small).draw(display);
                    let _=Text::new("C RESET",Point::new(34,41),small).draw(display);
                    let mut b=[b'0';8];let mut n=state.app_value;for i in (0..8).rev(){b[i]=b'0'+(n%10) as u8;n/=10;}
                    if let Ok(s)=core::str::from_utf8(&b){let _=Text::with_alignment(s,Point::new(64,55),text,Alignment::Center).draw(display);}
                }
                15 => {
                    draw_coin(display, fg, state, large, text);
                    let face=if state.app_value & 1 == 0 {"HEADS"} else {"TAILS"};
                    let _=Text::with_alignment(if state.coin_flipping {"FLIPPING"} else {face},Point::new(64,55),text,Alignment::Center).draw(display);
                    let _=Text::with_alignment("# FLIP COIN",Point::new(64,63),small,Alignment::Center).draw(display);
                }
                20 => {
                    let _=Text::new("A FIELD  B DELETE",Point::new(34,30),small).draw(display);
                    let _=Text::new("C CLEAR  D CASE",Point::new(34,41),small).draw(display);
                    let _=Text::new("# SAVE LOGIN",Point::new(34,53),small).draw(display);
                }
                22 => { let _=Text::with_alignment("# REBOOT DEVICE",Point::new(64,44),text,Alignment::Center).draw(display); }
                23 => { let _=Text::with_alignment("A SCREEN OFF",Point::new(64,41),text,Alignment::Center).draw(display); let _=Text::with_alignment("IR REMOTE TO WAKE",Point::new(64,55),small,Alignment::Center).draw(display); }
                24 => {
                    let elapsed=Instant::now().as_millis().saturating_sub(state.dice_started_ms);
                    let frame=(elapsed/55) as i32 % 32;
                    let travel=if frame<16 {frame} else {32-frame};
                    let bounce=((elapsed/45)%8) as i32;
                    let bounce=if bounce<4 {bounce} else {7-bounce};
                    let face=if state.dice_rolling {((elapsed/70)%6+1) as u8} else {state.dice_result.min(6)};
                    let x=6+travel*5;
                    let y=17+bounce*2;
                    draw_dice_face(display, fg, face, Point::new(x,y));
                    let _=Text::with_alignment("A SIDES  # ROLL",Point::new(64,53),text,Alignment::Center).draw(display);
                    let sides=match state.dice_sides { 4=>"D4",6=>"D6",8=>"D8",10=>"D10",12=>"D12",_=>"D20" };
                    let mut result=[b'0';2]; result[0]=b'0'+(state.dice_result/10) as u8; result[1]=b'0'+(state.dice_result%10) as u8;
                    if let Ok(s)=core::str::from_utf8(&result){let _=Text::with_alignment(if state.dice_rolling {"..."} else {s},Point::new(64,43),text,Alignment::Center).draw(display);}
                    let _=Text::with_alignment(sides,Point::new(64,63),small,Alignment::Center).draw(display);
                }
                25 => {
                    let _=Text::with_alignment("BPM",Point::new(64,30),small,Alignment::Center).draw(display);
                    let mut bpm=[b'0';3];bpm[0]=b'0'+((state.metronome_bpm/100)%10) as u8;bpm[1]=b'0'+((state.metronome_bpm/10)%10) as u8;bpm[2]=b'0'+(state.metronome_bpm%10) as u8;
                    if let Ok(s)=core::str::from_utf8(&bpm){let _=Text::with_alignment(s,Point::new(64,50),large,Alignment::Center).draw(display);}
                    for i in 0..4 {let x=46+i*12;let style=if state.metronome_running && i==state.metronome_beat as i32 {PrimitiveStyle::with_fill(fg)} else {PrimitiveStyle::with_stroke(fg,1)};let _=Circle::new(Point::new(x,53),6).into_styled(style).draw(display);}
                    let _=Text::with_alignment("A/B TEMPO  # START",Point::new(64,63),small,Alignment::Center).draw(display);
                }
                26 => {
                    let _=Rectangle::new(Point::new(14,16),Size::new(100,39)).into_styled(PrimitiveStyle::with_stroke(fg,1)).draw(display);
                    for i in 0..state.snake_len as usize {let cell=state.snake_body[i] as i32;let x=17+(cell%16)*6;let y=18+(cell/16)*6;let _=Rectangle::new(Point::new(x,y),Size::new(4,4)).into_styled(PrimitiveStyle::with_fill(fg)).draw(display);}
                    let fx=17+(state.snake_food as i32%16)*6;let fy=18+(state.snake_food as i32/16)*6;let _=Circle::new(Point::new(fx,fy),4).into_styled(PrimitiveStyle::with_fill(fg)).draw(display);
                    if !state.snake_running {let _=Text::with_alignment(if state.snake_over {"GAME OVER  # RETRY"} else {"# START SNAKE"},Point::new(64,42),text,Alignment::Center).draw(display);}
                    let _=Text::new("A/B/C/D MOVE  # PAUSE",Point::new(1,63),small).draw(display);
                }
                27 => {
                    let label=match state.reaction_phase {0=>"READY?",1=>"WAIT...",2=>"GO!",3 if state.reaction_result_ms==0=>"TOO SOON",_=>"RESULT"};
                    let _=Text::with_alignment(label,Point::new(64,39),large,Alignment::Center).draw(display);
                    if state.reaction_phase==3 && state.reaction_result_ms>0 {
                        let mut buf=[b'0';4];let mut n=state.reaction_result_ms.min(9999);for i in (0..4).rev(){buf[i]=b'0'+(n%10) as u8;n/=10;}
                        if let Ok(s)=core::str::from_utf8(&buf){let _=Text::with_alignment(s,Point::new(53,54),text,Alignment::Center).draw(display);}
                        let _=Text::new("MS",Point::new(72,54),small).draw(display);
                    }
                    let _=Text::with_alignment("# START / TAP ON GO",Point::new(64,63),small,Alignment::Center).draw(display);
                }
                _ => {
                    let _=Text::with_alignment("# SELECT  * BACK",Point::new(64,52),small,Alignment::Center).draw(display);
                }
            }
            let _=Text::new("* BACK",Point::new(94,10),small).draw(display);
        }
    }
}

fn draw_wifi_icon<D>(display: &mut D, fg: BinaryColor, connected: bool)
where D: DrawTarget<Color = BinaryColor>,
{
    for i in 0..4 {
        let h=3+i*2;
        let style=PrimitiveStyle::with_fill(if connected {fg} else {BinaryColor::Off});
        let outline=PrimitiveStyle::with_stroke(fg,1);
        let _=Rectangle::new(Point::new(104+i*5, 12-h),Size::new(3,h as u32)).into_styled(if connected {style} else {outline}).draw(display);
    }
}

fn draw_app_icon<D>(display: &mut D, fg: BinaryColor, app: usize, at: Point)
where D: DrawTarget<Color = BinaryColor>,
{
    let outline=PrimitiveStyle::with_stroke(fg,1);
    let fill=PrimitiveStyle::with_fill(fg);
    match app {
        0 | 17 => {
            let _=Rectangle::new(at,Size::new(24,18)).into_styled(outline).draw(display);
            for x in [6,12,18] { let _=Line::new(Point::new(at.x+x,at.y),Point::new(at.x+x,at.y+17)).into_styled(outline).draw(display); }
            let _=Line::new(Point::new(at.x,at.y+9),Point::new(at.x+23,at.y+9)).into_styled(outline).draw(display);
        }
        3 | 4 | 5 => {
            let _=Circle::new(at,22).into_styled(outline).draw(display);
            let _=Line::new(Point::new(at.x+11,at.y+11),Point::new(at.x+11,at.y+4)).into_styled(outline).draw(display);
            let _=Line::new(Point::new(at.x+11,at.y+11),Point::new(at.x+17,at.y+14)).into_styled(outline).draw(display);
        }
        7 => {
            let _=Rectangle::new(at,Size::new(22,24)).into_styled(outline).draw(display);
            for y in [8,14] { let _=Line::new(Point::new(at.x+1,at.y+y),Point::new(at.x+20,at.y+y)).into_styled(outline).draw(display); }
            for x in [7,14] { let _=Line::new(Point::new(at.x+x,at.y+1),Point::new(at.x+x,at.y+22)).into_styled(outline).draw(display); }
        }
        10 | 16 => {
            for i in 0..4 { let _=Rectangle::new(Point::new(at.x+i*5,at.y+20-i*5),Size::new(3,((i+1)*5) as u32)).into_styled(fill).draw(display); }
        }
        12 => {
            let _=Circle::new(Point::new(at.x+4,at.y+4),16).into_styled(outline).draw(display);
            let _=Circle::new(Point::new(at.x+9,at.y+9),6).into_styled(outline).draw(display);
            for (x1,y1,x2,y2) in [(11,0,11,4),(11,20,11,24),(0,11,4,11),(20,11,24,11),(3,3,6,6),(17,17,20,20),(17,5,20,2),(3,19,6,16)] {
                let _=Line::new(Point::new(at.x+x1,at.y+y1),Point::new(at.x+x2,at.y+y2)).into_styled(outline).draw(display);
            }
        }
        24 => draw_dice_face(display,fg,5,at),
        23 => {
            let _=Circle::new(Point::new(at.x+2,at.y+2),20).into_styled(outline).draw(display);
            let _=Line::new(Point::new(at.x+12,at.y),Point::new(at.x+12,at.y+12)).into_styled(outline).draw(display);
        }
        25 => {
            let _=Line::new(Point::new(at.x+4,at.y+21),Point::new(at.x+20,at.y+21)).into_styled(outline).draw(display);
            let _=Line::new(Point::new(at.x+7,at.y+18),Point::new(at.x+12,at.y+2)).into_styled(outline).draw(display);
            let _=Line::new(Point::new(at.x+17,at.y+18),Point::new(at.x+12,at.y+2)).into_styled(outline).draw(display);
            let _=Line::new(Point::new(at.x+12,at.y+4),Point::new(at.x+18,at.y+15)).into_styled(outline).draw(display);
        }
        26 => {
            let _=Rectangle::new(Point::new(at.x,at.y+14),Size::new(7,7)).into_styled(fill).draw(display);
            let _=Rectangle::new(Point::new(at.x+8,at.y+7),Size::new(7,7)).into_styled(fill).draw(display);
            let _=Rectangle::new(Point::new(at.x+16,at.y),Size::new(7,7)).into_styled(fill).draw(display);
        }
        27 => {
            for (x1,y1,x2,y2) in [(13,0,5,12),(5,12,12,12),(12,12,7,23)] {
                let _=Line::new(Point::new(at.x+x1,at.y+y1),Point::new(at.x+x2,at.y+y2)).into_styled(PrimitiveStyle::with_stroke(fg,2)).draw(display);
            }
        }
        _ => {
            let _=Rectangle::new(at,Size::new(22,22)).into_styled(outline).draw(display);
            let ch=aetheris::APPS[app % aetheris::APPS.len()].name.as_bytes().first().copied().unwrap_or(b'?');
            let mut b=[0u8;1];b[0]=ch;
            if let Ok(s)=core::str::from_utf8(&b){let _=Text::with_alignment(s,Point::new(at.x+11,at.y+16),MonoTextStyle::new(&FONT_6X10,fg),Alignment::Center).draw(display);}
        }
    }
}

fn draw_dice_face<D>(display: &mut D, fg: BinaryColor, value: u8, at: Point)
where D: DrawTarget<Color = BinaryColor>,
{
    let _=Rectangle::new(at,Size::new(26,26)).into_styled(PrimitiveStyle::with_stroke(fg,2)).draw(display);
    let pips: &[(i32,i32)] = match value {
        1=>&[(12,12)],
        2=>&[(6,6),(18,18)],
        3=>&[(6,6),(12,12),(18,18)],
        4=>&[(6,6),(18,6),(6,18),(18,18)],
        5=>&[(6,6),(18,6),(12,12),(6,18),(18,18)],
        _=>&[(6,6),(12,6),(18,6),(6,18),(12,18),(18,18)],
    };
    for (x,y) in pips { let _=Circle::new(Point::new(at.x+x-2,at.y+y-2),4).into_styled(PrimitiveStyle::with_fill(fg)).draw(display); }
}

fn draw_coin<D>(display: &mut D, fg: BinaryColor, state: &State, large: MonoTextStyle<'_, BinaryColor>, text: MonoTextStyle<'_, BinaryColor>)
where D: DrawTarget<Color = BinaryColor>,
{
    let (width, label) = if state.coin_flipping {
        let frame=(Instant::now().as_millis().saturating_sub(state.coin_started_ms)/45) as u32 % 16;
        let phase=if frame<8 {frame} else {15-frame};
        (4+(phase*3).min(24), if frame%2==0 {"H"} else {"T"})
    } else {
        (26, if state.app_value & 1 == 0 {"H"} else {"T"})
    };
    let x=64-(width as i32/2);
    let _=Ellipse::new(Point::new(x,17),Size::new(width,28)).into_styled(PrimitiveStyle::with_stroke(fg,2)).draw(display);
    if width>=20 {let _=Text::with_alignment(label,Point::new(64,37),large,Alignment::Center).draw(display);}
    else {let _=Text::with_alignment(label,Point::new(64,36),text,Alignment::Center).draw(display);}
}

fn random_next(state: &mut State) -> u32 {
    let mut x = state.dice_seed ^ Instant::now().as_millis() as u32;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    if x == 0 { x = 0xA37E_2159; }
    state.dice_seed = x;
    x
}

fn start_dice_roll(state: &mut State) {
    state.dice_result = (random_next(state) % state.dice_sides as u32 + 1) as u8;
    state.dice_rolling = true;
    state.dice_started_ms = Instant::now().as_millis();
    state.dice_sound_step = u64::MAX;
}

fn start_coin_flip(state: &mut State) {
    state.app_value = random_next(state) & 1;
    state.coin_flipping = true;
    state.coin_started_ms = Instant::now().as_millis();
    state.coin_sound_step = u64::MAX;
}

fn snake_reset(state: &mut State) {
    state.snake_body.fill(0);
    state.snake_body[0] = 53; // row 3, column 5; head faces right
    state.snake_body[1] = 52;
    state.snake_body[2] = 51;
    state.snake_len = 3;
    state.snake_direction = 1;
    snake_place_food(state);
    state.snake_running = false;
    state.snake_over = false;
    state.snake_ate = false;
    state.snake_last_ms = Instant::now().as_millis();
    state.app_counter = 0;
}

fn snake_place_food(state: &mut State) {
    for _ in 0..96 {
        let candidate=(random_next(state)%96) as u8;
        if !state.snake_body[..state.snake_len as usize].contains(&candidate) {
            state.snake_food=candidate;
            return;
        }
    }
    state.snake_food=0;
}

fn snake_turn(state: &mut State, direction: u8) {
    if (direction + 2) % 4 != state.snake_direction {
        state.snake_direction = direction;
    }
}

fn snake_step(state: &mut State) {
    let head = state.snake_body[0] as i16;
    let x = head % 16;
    let y = head / 16;
    let (nx, ny) = match state.snake_direction {
        0 => (x, y - 1),
        1 => (x + 1, y),
        2 => (x, y + 1),
        _ => (x - 1, y),
    };
    if !(0..16).contains(&nx) || !(0..6).contains(&ny) {
        state.snake_over = true;
        state.snake_running = false;
        return;
    }
    let next = (ny * 16 + nx) as u8;
    let ate = next == state.snake_food;
    let len = state.snake_len as usize;
    let collision_len = if ate { len } else { len.saturating_sub(1) };
    if state.snake_body[..collision_len].contains(&next) {
        state.snake_over = true;
        state.snake_running = false;
        return;
    }
    if ate && len < state.snake_body.len() {
        state.snake_body.copy_within(0..len, 1);
        state.snake_len += 1;
        state.app_counter = state.app_counter.saturating_add(1);
        state.snake_ate = true;
        snake_place_food(state);
    } else {
        state.snake_body.copy_within(0..len.saturating_sub(1), 1);
    }
    state.snake_body[0] = next;
}

fn scan_key(rows: &mut [Output<'static>; 4], cols: &[Input<'static>; 4]) -> Option<u8> {
    for r in 0..4 {
        for row in rows.iter_mut() {
            row.set_high();
        }
        rows[r].set_low();
        for c in 0..4 {
            if cols[c].is_low() {
                return Some(KEYMAP[r][c]);
            }
        }
    }
    None
}

async fn wait_ir_level(pin: &Input<'static>, high: bool, limit_us: u32) -> Option<u32> {
    let start = Instant::now();
    loop {
        if pin.is_high() == high {
            return Some(start.elapsed().as_micros() as u32);
        }
        if start.elapsed().as_micros() >= limit_us as u64 {
            return None;
        }
        Timer::after_micros(40).await;
    }
}

/// Decode a standard 32-bit NEC frame from a demodulating IR receiver.
async fn read_nec(pin: &Input<'static>) -> Option<u32> {
    let leader_mark = wait_ir_level(pin, true, 12_000).await?;
    if !(8_000..=10_500).contains(&leader_mark) {
        return None;
    }
    let leader_space = wait_ir_level(pin, false, 6_000).await?;
    if !(3_000..=5_500).contains(&leader_space) {
        return None;
    }
    let mut frame = 0u32;
    for bit in 0..32 {
        let mark = wait_ir_level(pin, true, 1_200).await?;
        if !(300..=900).contains(&mark) {
            return None;
        }
        let space = wait_ir_level(pin, false, 2_300).await?;
        if space > 1_000 {
            frame |= 1 << bit;
        }
    }
    Some(frame)
}

fn draw_aux<D>(display: &mut D, state: &State)
where
    D: DrawTarget<Color = BinaryColor>,
{
    let fg = if state.wallpaper & 1 == 0 {
        BinaryColor::On
    } else {
        BinaryColor::Off
    };
    let bg = if state.wallpaper & 1 == 0 {
        BinaryColor::Off
    } else {
        BinaryColor::On
    };
    let _ = display.clear(bg);
    let title = MonoTextStyle::new(&FONT_6X10, fg);
    let small = MonoTextStyle::new(&FONT_6X10, fg);
    let _ = Text::with_alignment("AETHERIS", Point::new(64, 17), title, Alignment::Center)
        .draw(display);
    let _ = Line::new(Point::new(0, 22), Point::new(127, 22)).into_styled(PrimitiveStyle::with_stroke(fg, 1)).draw(display);
    let _ = Text::new(
        if state.wifi_connected {
            "WIFI CONNECTED ~))"
        } else {
            "WIFI DISCONNECTED"
        },
        Point::new(2, 38),
        small,
    )
    .draw(display);
    let app = match state.screen {
        Screen::Login => "LOGIN",
        Screen::Home => "HOME",
        Screen::Menu => aetheris::APPS[state.selection % aetheris::APPS.len()].name,
        Screen::Settings => "SETTINGS",
        Screen::WifiSetup => "NETWORK SETUP",
        Screen::Piano => "PIANO",
        Screen::App => aetheris::APPS[state.selection % aetheris::APPS.len()].name,
    };
    let _ = Text::with_alignment(app, Point::new(64, 56), small, Alignment::Center).draw(display);
    let _ = Text::with_alignment("SELESTE TECHNOLOGIES", Point::new(64, 63), MonoTextStyle::new(&FONT_5X7, fg), Alignment::Center).draw(display);
}

async fn scan_wifi(controller: &mut WifiController<'_>, state: &mut State) -> bool {
    match controller
        .scan_async(&ScanConfig::default().with_max(8))
        .await
    {
        Ok(aps) => {
            state.network_count = 0;
            for ap in aps.iter().take(8) {
                let i = state.network_count;
                let bytes = ap.ssid.as_str().as_bytes();
                let n = bytes.len().min(32);
                state.networks[i][..n].copy_from_slice(&bytes[..n]);
                state.network_lens[i] = n as u8;
                state.network_count += 1;
                info!("Wi-Fi: {} ({} dBm)", ap.ssid.as_str(), ap.signal_strength);
            }
            state.network_selected = 0;
            true
        }
        Err(e) => {
            error!("Wi-Fi scan error: {:?}", e);
            false
        }
    }
}

async fn connect_wifi(controller: &mut WifiController<'_>, state: &mut State) -> bool {
    if state.network_count == 0 {
        return false;
    }
    let i = state.network_selected % state.network_count;
    let Ok(ssid_str) = core::str::from_utf8(&state.networks[i][..state.network_lens[i] as usize])
    else {
        return false;
    };
    let Ok(ssid) = Ssid::try_from(ssid_str) else {
        return false;
    };
    let auth = if state.wifi_password_len == 0 {
        AuthenticationMethodConfig::Open
    } else {
        let Ok(password_str) =
            core::str::from_utf8(&state.wifi_password[..state.wifi_password_len])
        else {
            return false;
        };
        let Ok(password) = password_str.try_into() else {
            return false;
        };
        AuthenticationMethodConfig::Wpa2Personal(password)
    };
    let config = WifiConfig::Station(
        StationConfig::default()
            .with_ssid(ssid)
            .with_authentication(auth),
    );
    if let Err(e) = controller.set_config(&config) {
        error!("Wi-Fi configuration failed: {:?}", e);
        return false;
    }
    match controller.connect_async().await {
        Ok(info) => {
            info!("Connected to Wi-Fi: {}", info.ssid.as_str());
            true
        }
        Err(e) => {
            error!("Wi-Fi connection failed: {:?}", e);
            false
        }
    }
}

async fn beep(buzzer: &mut Output<'static>, hz: u16, duration_ms: u16) {
    if hz == 0 {
        return;
    }
    let half_us = 500_000u64 / hz as u64;
    let toggles = duration_ms as u64 * hz as u64 / 500;
    for _ in 0..toggles {
        buzzer.toggle();
        Timer::after_micros(half_us).await;
    }
    buzzer.set_low();
}

#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    esp_println::logger::init_logger_from_env();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let p = esp_hal::init(config);
    esp_alloc::heap_allocator!(size: 64 * 1024);
    let mut led = Output::new(p.GPIO22, Level::Low, OutputConfig::default());
    let mut buzzer = Output::new(p.GPIO23, Level::Low, OutputConfig::default());
    let mut rows = [
        Output::new(p.GPIO0, Level::High, OutputConfig::default()),
        Output::new(p.GPIO1, Level::High, OutputConfig::default()),
        Output::new(p.GPIO2, Level::High, OutputConfig::default()),
        Output::new(p.GPIO3, Level::High, OutputConfig::default()),
    ];
    let col_config = InputConfig::default().with_pull(Pull::Up);
    let cols = [
        Input::new(p.GPIO10, col_config),
        Input::new(p.GPIO11, col_config),
        Input::new(p.GPIO18, col_config),
        Input::new(p.GPIO19, col_config),
    ];
    let ir = Input::new(p.GPIO4, InputConfig::default().with_pull(Pull::Up));

    let timg0 = TimerGroup::new(p.TIMG0);
    esp_rtos::start(timg0.timer0, p.FROM_CPU_INTR0);
    let mut wifi = WifiController::new(p.WIFI, Default::default()).expect("Wi-Fi init failed");
    wifi.set_config(&WifiConfig::Station(StationConfig::default()))
        .expect("Wi-Fi station config failed");
    let i2c = I2c::new(
        p.I2C0,
        I2cConfig::default().with_frequency(Rate::from_khz(400)),
    )
    .expect("I2C configuration failed")
    .with_sda(p.GPIO6)
    .with_scl(p.GPIO7);
    let shared_i2c = RefCell::new(i2c);
    let interface = I2CDisplayInterface::new(RefCellDevice::new(&shared_i2c));
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display
        .init()
        .expect("OLED initialization failed; check SDA/SCL and panel type");
    // If the secondary module is strapped to 0x3D it gets a separate status
    // dashboard. If both modules are fixed at 0x3C they mirror OLED 1.
    let interface2 = I2CDisplayInterface::new_alternate_address(RefCellDevice::new(&shared_i2c));
    let mut display2 = Ssd1306::new(interface2, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    let second_display_available = display2.init().is_ok();

    info!("AETHERIS | Powered by Seleste Technologies | ESP32-C6 R8N16");
    let mut state = State::new();
    for frame in 0..12 {
        let _ = display.clear(BinaryColor::Off);
        let white = MonoTextStyle::new(&FONT_5X7, BinaryColor::On);
        for x in 0..20 {
            let ch = aetheris::matrix_glyph(x, frame);
            let mut b = [0u8; 1];
            b[0] = ch as u8;
            if let Ok(s) = core::str::from_utf8(&b) {
                let y = ((x as u32 * 11 + frame as u32 * 7) % 53) as i32 + 7;
                let _ = Text::new(s, Point::new(x as i32 * 6, y), white).draw(&mut display);
            }
        }
        let _ = display.flush();
        if second_display_available {
            let _ = display2.clear(BinaryColor::Off);
            let _ = Text::with_alignment(
                "AETHERIS",
                Point::new(64, 26),
                MonoTextStyle::new(&FONT_6X10, BinaryColor::On),
                Alignment::Center,
            )
            .draw(&mut display2);
            let _ = Text::with_alignment(
                "Powered by Seleste Technologies",
                Point::new(64, 42),
                MonoTextStyle::new(&FONT_5X7, BinaryColor::On),
                Alignment::Center,
            )
            .draw(&mut display2);
            let _ = display2.flush();
        }
        led.toggle();
        Timer::after_millis(120).await;
    }
    let _ = display.clear(BinaryColor::Off);
    let title = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    let _ = Text::with_alignment("AETHERIS", Point::new(64, 25), title, Alignment::Center)
        .draw(&mut display);
    let _ = Text::with_alignment(
        "Powered by Seleste",
        Point::new(64, 39),
        MonoTextStyle::new(&FONT_5X7, BinaryColor::On),
        Alignment::Center,
    )
    .draw(&mut display);
    let _ = Text::with_alignment(
        "Technologies",
        Point::new(64, 51),
        MonoTextStyle::new(&FONT_5X7, BinaryColor::On),
        Alignment::Center,
    )
    .draw(&mut display);
    let _ = display.flush();
    beep(&mut buzzer, 880, 100).await;
    beep(&mut buzzer, 1175, 140).await;
    led.set_high();

    loop {
        state.wifi_connected = wifi.is_connected();
        if state.wifi_connected {
            led.set_high();
        } else {
            led.set_low();
        }
        if state.screen == Screen::App && state.selection == 4 && state.app_running {
            let elapsed = (Instant::now()
                .as_millis()
                .saturating_sub(state.app_started_ms)
                / 1000) as u32;
            if elapsed >= state.app_counter {
                state.app_counter = 0;
                state.app_value = 0;
                state.app_running = false;
                beep(&mut buzzer, 1047, 160).await;
                beep(&mut buzzer, 1319, 220).await;
            }
        }
        let now_ms = Instant::now().as_millis();
        if state.screen == Screen::App && state.selection == 24 && state.dice_rolling {
            let elapsed = now_ms.saturating_sub(state.dice_started_ms);
            if elapsed >= 1_200 {
                state.dice_rolling = false;
                beep(&mut buzzer, 1320, 90).await;
            } else {
                let step = elapsed / 100;
                if state.dice_sound_step != step {
                    state.dice_sound_step = step;
                    beep(&mut buzzer, if step % 2 == 0 { 1760 } else { 1320 }, 18).await;
                }
            }
        }
        if state.screen == Screen::App && state.selection == 15 && state.coin_flipping {
            let elapsed = now_ms.saturating_sub(state.coin_started_ms);
            if elapsed >= 1_000 {
                state.coin_flipping = false;
                beep(&mut buzzer, if state.app_value & 1 == 0 { 1047 } else { 784 }, 100).await;
            } else {
                let step = elapsed / 90;
                if state.coin_sound_step != step {
                    state.coin_sound_step = step;
                    beep(&mut buzzer, if step % 2 == 0 { 1480 } else { 990 }, 18).await;
                }
            }
        }
        if state.screen == Screen::App && state.selection == 25 && state.metronome_running
            && now_ms >= state.metronome_next_ms {
            let beat_ms = 60_000 / state.metronome_bpm as u64;
            state.metronome_next_ms = now_ms + beat_ms;
            state.metronome_beat = (state.metronome_beat + 1) % 4;
            beep(&mut buzzer, if state.metronome_beat == 0 { 1760 } else { 880 }, 35).await;
        }
        if state.screen == Screen::App && state.selection == 26 && state.snake_running
            && now_ms.saturating_sub(state.snake_last_ms)
                >= 350u64.saturating_sub((state.app_counter as u64).min(200)) {
            state.snake_last_ms = now_ms;
            let was_running = state.snake_running;
            snake_step(&mut state);
            if state.snake_ate {
                state.snake_ate = false;
                beep(&mut buzzer, 1320, 70).await;
            } else if was_running && state.snake_over {
                beep(&mut buzzer, 220, 220).await;
            }
        }
        if state.screen == Screen::App && state.selection == 27 && state.reaction_phase == 1
            && now_ms.saturating_sub(state.reaction_started_ms) >= state.reaction_wait_ms {
            state.reaction_phase = 2;
            state.reaction_ready_ms = now_ms;
            beep(&mut buzzer, 1760, 110).await;
        }
        if !state.screen_off {
            draw(&mut display, &state, state.wifi_connected);
            let _ = display.flush();
        }
        if second_display_available && !state.screen_off {
            draw_aux(&mut display2, &state);
            let _ = display2.flush();
        }
        if let Some(key) = scan_key(&mut rows, &cols) {
            Timer::after_millis(35).await;
            if scan_key(&mut rows, &cols) == Some(key) {
                let waking = state.screen_off;
                if waking {
                    state.screen_off = false;
                }
                state.last_key = key;
                match state.screen {
                    Screen::Login => match key {
                        b'0'..=b'9' => {
                            state.insert_phone_key(key, Instant::now().as_millis());
                            beep(&mut buzzer, 660, 25).await;
                        }
                        b'A' => {
                            state.edit_field = if state.edit_field == EditField::Username {
                                EditField::Password
                            } else {
                                EditField::Username
                            };
                            state.tap_key = 0;
                        }
                        b'B' | b'*' => state.delete_char(),
                        b'C' => state.clear_field(),
                        b'D' => {
                            state.uppercase = !state.uppercase;
                            state.tap_key = 0;
                        }
                        b'#' => {
                            let valid_user = state.username_len == state.account_username_len
                                && &state.username[..state.username_len]
                                    == &state.account_username[..state.account_username_len];
                            let valid_pass = state.password_len == state.account_password_len
                                && &state.password[..state.password_len]
                                    == &state.account_password[..state.account_password_len];
                            if valid_user && valid_pass {
                                state.screen = Screen::Home;
                                beep(&mut buzzer, 880, 60).await;
                            } else {
                                beep(&mut buzzer, 180, 160).await;
                            }
                        }
                        _ => {}
                    },
        Screen::Home => match key {
                        b'A' => {
                            state.screen = Screen::Menu;
                            state.selection = 0;
                        }
                        b'B' => state.screen = Screen::Settings,
                        b'D' => state.screen = Screen::Piano,
                        b'#' => {
                            state.selection = 24;
                            state.screen = Screen::App;
                            state.dice_rolling = false;
                        }
                        b'*' => {
                            state.screen = Screen::Menu;
                            state.selection = 0;
                        }
                        _ => {}
                    },
                    Screen::Menu => match key {
                        b'A' | b'2' => {
                            state.selection =
                                (state.selection + aetheris::APPS.len() - 1) % aetheris::APPS.len()
                        }
                        b'B' | b'8' => {
                            state.selection = (state.selection + 1) % aetheris::APPS.len()
                        }
                        b'#' => {
                            state.screen = if state.selection == 0 {
                                Screen::Piano
                            } else {
                                Screen::App
                            }
                            ;
                            match state.selection {
                                24 => state.dice_rolling = false,
                                25 => { state.metronome_running = false; state.metronome_beat = 0; }
                                26 => snake_reset(&mut state),
                                27 => { state.reaction_phase = 0; state.reaction_result_ms = 0; }
                                _ => {}
                            }
                        }
                        b'*' | b'D' => state.screen = Screen::Home,
                        _ => {}
                    },
                    Screen::Settings => match key {
                        b'A' => state.wallpaper = state.wallpaper.wrapping_add(1) % 4,
                        b'B' | b'#' => {
                            state.screen = Screen::WifiSetup;
                            state.wifi_edit_password = false;
                            if scan_wifi(&mut wifi, &mut state).await {
                                beep(&mut buzzer, 880, 80).await;
                            } else {
                                beep(&mut buzzer, 180, 160).await;
                            }
                        }
                        b'C' => esp_hal::system::software_reset(),
                        b'*' | b'D' => state.screen = Screen::Home,
                        _ => {}
                    },
                    Screen::WifiSetup => {
                        if !state.wifi_edit_password {
                            match key {
                                b'A' => {
                                    if state.network_count > 0 {
                                        state.network_selected =
                                            (state.network_selected + state.network_count - 1)
                                                % state.network_count;
                                    }
                                }
                                b'B' => {
                                    if state.network_count > 0 {
                                        state.network_selected =
                                            (state.network_selected + 1) % state.network_count;
                                    }
                                }
                                b'D' => {
                                    let _ = scan_wifi(&mut wifi, &mut state).await;
                                }
                                b'#' if state.network_count > 0 => {
                                    state.wifi_edit_password = true;
                                    state.wifi_password_len = 0;
                                    state.wifi_password.fill(0);
                                    state.tap_key = 0;
                                }
                                b'*' => state.screen = Screen::Settings,
                                _ => {}
                            }
                        } else {
                            match key {
                                b'0'..=b'9' => t9_insert(
                                    &mut state.wifi_password,
                                    &mut state.wifi_password_len,
                                    key,
                                    state.uppercase,
                                    &mut state.tap_key,
                                    &mut state.tap_index,
                                    &mut state.last_tap_ms,
                                    Instant::now().as_millis(),
                                ),
                                b'A' => {
                                    state.wifi_connected =
                                        connect_wifi(&mut wifi, &mut state).await;
                                    if state.wifi_connected {
                                        led.set_high();
                                        beep(&mut buzzer, 1100, 100).await;
                                        state.screen = Screen::Home;
                                    } else {
                                        beep(&mut buzzer, 180, 160).await;
                                    }
                                }
                                b'B' | b'*' => {
                                    if state.wifi_password_len > 0 {
                                        state.wifi_password_len -= 1;
                                        state.wifi_password[state.wifi_password_len] = 0;
                                    }
                                    state.tap_key = 0;
                                }
                                b'C' => {
                                    state.wifi_password.fill(0);
                                    state.wifi_password_len = 0;
                                    state.tap_key = 0;
                                }
                                b'D' => {
                                    state.uppercase = !state.uppercase;
                                    state.tap_key = 0;
                                }
                                b'#' => {
                                    state.wifi_edit_password = false;
                                }
                                _ => {}
                            }
                        }
                    }
                    Screen::Piano => match key {
                        b'1'..=b'9' | b'0' | b'A' | b'B' => {
                            let key_index = match key {
                                b'1'..=b'9' => key - b'1',
                                b'0' => 9,
                                b'A' => 10,
                                _ => 11,
                            };
                            let root = state.piano_octave * 12 + key_index;
                            beep(&mut buzzer, aetheris::piano_frequency_hz(root), 180).await;
                            if state.piano_chord {
                                beep(&mut buzzer, aetheris::piano_frequency_hz(root + 4), 180)
                                    .await;
                                beep(&mut buzzer, aetheris::piano_frequency_hz(root + 7), 220)
                                    .await;
                            }
                        }
                        b'C' => state.piano_octave = 0,
                        b'D' => state.piano_octave = 1,
                        b'#' => state.piano_chord = !state.piano_chord,
                        b'*' => state.screen = Screen::Home,
                        _ => {}
                    },
                    Screen::App => match state.selection {
                        1 => match key {
                            b'1' => info!(
                                "Aetheris shell: Rust/Embassy running; {} apps",
                                aetheris::APPS.len()
                            ),
                            b'2' => info!(
                                "Aetheris shell commands: 1=status, 2=help, 3=Wi-Fi scan, #=reboot"
                            ),
                            b'3' => {
                                let _ = scan_wifi(&mut wifi, &mut state).await;
                            }
                            b'#' => esp_hal::system::software_reset(),
                            b'*' => state.screen = Screen::Menu,
                            _ => info!("Aetheris terminal key: {}", key as char),
                        },
                        4 => match key {
                            b'A' => state.app_value = state.app_value.saturating_add(10),
                            b'B' => state.app_value = state.app_value.saturating_sub(10),
                            b'C' => {
                                state.app_value = 60;
                                state.app_counter = 0;
                                state.app_running = false;
                            }
                            b'#' => {
                                if state.app_running {
                                    let elapsed = (Instant::now()
                                        .as_millis()
                                        .saturating_sub(state.app_started_ms)
                                        / 1000)
                                        as u32;
                                    state.app_counter = state.app_counter.saturating_sub(elapsed);
                                    state.app_value = state.app_counter;
                                    state.app_running = false;
                                } else {
                                    state.app_counter = state.app_value;
                                    state.app_started_ms = Instant::now().as_millis();
                                    state.app_running = true;
                                }
                            }
                            b'*' => {
                                state.app_running = false;
                                state.screen = Screen::Menu;
                            }
                            _ => {}
                        },
                        5 => match key {
                            b'#' => {
                                if state.app_running {
                                    state.app_counter += (Instant::now()
                                        .as_millis()
                                        .saturating_sub(state.app_started_ms)
                                        / 1000)
                                        as u32;
                                    state.app_running = false;
                                } else {
                                    state.app_started_ms = Instant::now().as_millis();
                                    state.app_running = true;
                                }
                            }
                            b'C' => {
                                state.app_counter = 0;
                                state.app_running = false;
                            }
                            b'*' => {
                                state.app_running = false;
                                state.screen = Screen::Menu;
                            }
                            _ => {}
                        },
                        6 => match key {
                            b'0'..=b'9' => t9_insert(
                                &mut state.note,
                                &mut state.note_len,
                                key,
                                false,
                                &mut state.tap_key,
                                &mut state.tap_index,
                                &mut state.last_tap_ms,
                                Instant::now().as_millis(),
                            ),
                            b'B' => {
                                if state.note_len > 0 {
                                    state.note_len -= 1;
                                    state.note[state.note_len] = 0;
                                }
                                state.tap_key = 0;
                            }
                            b'C' => {
                                state.note.fill(0);
                                state.note_len = 0;
                            }
                            b'*' => state.screen = Screen::Menu,
                            _ => {}
                        },
                        7 => match key {
                            b'0'..=b'9' => {
                                state.app_value = state
                                    .app_value
                                    .saturating_mul(10)
                                    .saturating_add((key - b'0') as u32)
                            }
                            b'A'..=b'D' => {
                                state.app_accum = state.app_value;
                                state.app_value = 0;
                                state.app_op = key;
                            }
                            b'#' => {
                                state.app_value = match state.app_op {
                                    b'A' => state.app_accum.saturating_add(state.app_value),
                                    b'B' => state.app_accum.saturating_sub(state.app_value),
                                    b'C' => state.app_accum.saturating_mul(state.app_value),
                                    b'D' if state.app_value != 0 => {
                                        state.app_accum / state.app_value
                                    }
                                    _ => state.app_value,
                                };
                            }
                            b'*' => {
                                state.app_value = 0;
                                state.app_accum = 0;
                                state.app_op = 0;
                            }
                            _ => {}
                        },
                        9 | 17 => match key {
                            b'1'..=b'9' => {
                                beep(&mut buzzer, aetheris::piano_frequency_hz(key - b'1'), 200)
                                    .await
                            }
                            b'0' => beep(&mut buzzer, aetheris::piano_frequency_hz(9), 200).await,
                            b'A' => beep(&mut buzzer, 880, 100).await,
                            b'B' => beep(&mut buzzer, 660, 150).await,
                            b'C' => beep(&mut buzzer, 440, 200).await,
                            b'D' => beep(&mut buzzer, 220, 250).await,
                            b'*' => state.screen = Screen::Menu,
                            _ => {}
                        },
                        10 | 16 => match key {
                            b'#' | b'A' => {
                                state.screen = Screen::WifiSetup;
                                state.wifi_edit_password = false;
                                let _ = scan_wifi(&mut wifi, &mut state).await;
                            }
                            b'*' => state.screen = Screen::Menu,
                            _ => {}
                        },
                        14 => match key {
                            b'A' => state.app_value = state.app_value.saturating_add(1),
                            b'B' => state.app_value = state.app_value.saturating_sub(1),
                            b'C' => state.app_value = 0,
                            b'*' => state.screen = Screen::Menu,
                            _ => {}
                        },
                        15 => match key {
                            b'#' | b'A' => start_coin_flip(&mut state),
                            b'*' => state.screen = Screen::Menu,
                            _ => {}
                        },
                        12 => {
                            if key == b'*' {
                                state.screen = Screen::Menu;
                            } else if key == b'#' {
                                state.screen = Screen::Settings;
                            }
                        }
                        13 => {
                            if key == b'A' || key == b'B' || key == b'#' {
                                state.wallpaper = state.wallpaper.wrapping_add(1) % 4;
                            }
                            if key == b'*' {
                                state.screen = Screen::Menu;
                            }
                        }
                        18 => {
                            info!("Key tester saw {:?}", key as char);
                            if key == b'*' {
                                state.screen = Screen::Menu;
                            }
                        }
                        20 => match key {
                            b'0'..=b'9' => state.insert_phone_key(key, Instant::now().as_millis()),
                            b'A' => {
                                state.edit_field = if state.edit_field == EditField::Username {
                                    EditField::Password
                                } else {
                                    EditField::Username
                                };
                                state.tap_key = 0;
                            }
                            b'B' | b'*' => state.delete_char(),
                            b'C' => state.clear_field(),
                            b'D' => {
                                state.uppercase = !state.uppercase;
                                state.tap_key = 0;
                            }
                            b'#' if state.username_len > 0 && state.password_len > 0 => {
                                state.account_username.fill(0);
                                state.account_username[..state.username_len]
                                    .copy_from_slice(&state.username[..state.username_len]);
                                state.account_username_len = state.username_len;
                                state.account_password.fill(0);
                                state.account_password[..state.password_len]
                                    .copy_from_slice(&state.password[..state.password_len]);
                                state.account_password_len = state.password_len;
                                info!("Aetheris account changed in RAM (not persistent)");
                                beep(&mut buzzer, 880, 80).await;
                            }
                            _ => {}
                        },
                        21 => {
                            if key == b'#' {
                                error!(
                                    "OTA slots exist, but no network image downloader/signature verifier is configured"
                                );
                                beep(&mut buzzer, 180, 180).await;
                            }
                            if key == b'*' {
                                state.screen = Screen::Menu;
                            }
                        }
                        22 => {
                            if key == b'#' {
                                beep(&mut buzzer, 660, 60).await;
                                esp_hal::system::software_reset();
                            } else if key == b'*' {
                                state.screen = Screen::Menu;
                            }
                        }
                        23 => {
                            if key == b'*' {
                                state.screen = Screen::Menu;
                            } else if !waking {
                                state.screen_off = true;
                                led.set_low();
                                buzzer.set_low();
                                let _ = display.clear(BinaryColor::Off);
                                let _ = display.flush();
                                if second_display_available {
                                    let _ = display2.clear(BinaryColor::Off);
                                    let _ = display2.flush();
                                }
                            }
                        }
                        24 => match key {
                            b'A' => state.dice_sides = match state.dice_sides {
                                4 => 6, 6 => 8, 8 => 10, 10 => 12, 12 => 20, _ => 4,
                            },
                            b'#' | b'B' => start_dice_roll(&mut state),
                            b'*' => state.screen = Screen::Menu,
                            _ => {}
                        },
                        25 => match key {
                            b'A' => state.metronome_bpm = (state.metronome_bpm + 5).min(240),
                            b'B' => state.metronome_bpm = state.metronome_bpm.saturating_sub(5).max(40),
                            b'D' => state.metronome_bpm = 120,
                            b'#' => {
                                state.metronome_running = !state.metronome_running;
                                state.metronome_next_ms = Instant::now().as_millis();
                                state.metronome_beat = 3;
                            }
                            b'*' => {state.metronome_running=false;state.screen=Screen::Menu;}
                            _ => {}
                        },
                        26 => match key {
                            b'A' => snake_turn(&mut state, 0),
                            b'B' => snake_turn(&mut state, 1),
                            b'C' => snake_turn(&mut state, 2),
                            b'D' => snake_turn(&mut state, 3),
                            b'#' => {
                                if state.snake_over {snake_reset(&mut state);}
                                state.snake_running = !state.snake_running;
                                state.snake_last_ms = Instant::now().as_millis();
                            }
                            b'*' => {state.snake_running=false;state.screen=Screen::Menu;}
                            _ => {}
                        },
                        27 => match key {
                            b'*' => {state.reaction_phase=0;state.screen=Screen::Menu;}
                            b'#' => match state.reaction_phase {
                                0 | 3 => {
                                    state.reaction_phase=1;
                                    state.reaction_started_ms=Instant::now().as_millis();
                                    state.reaction_wait_ms=1500+(random_next(&mut state)%2500) as u64;
                                    state.reaction_result_ms=0;
                                }
                                1 => {state.reaction_phase=3;state.reaction_result_ms=0;beep(&mut buzzer,180,130).await;}
                                2 => {
                                    state.reaction_result_ms=Instant::now().as_millis().saturating_sub(state.reaction_ready_ms) as u32;
                                    state.reaction_phase=3;
                                    beep(&mut buzzer,1320,60).await;
                                }
                                _ => {}
                            },
                            _ if state.reaction_phase==1 => {state.reaction_phase=3;state.reaction_result_ms=0;beep(&mut buzzer,180,130).await;}
                            _ => {}
                        },
                        _ => {
                            if key == b'*' {
                                state.screen = Screen::Menu;
                            }
                        }
                    },
                }
            }
            while scan_key(&mut rows, &cols).is_some() {
                Timer::after_millis(20).await;
            }
        }
        // A completed NEC remote-key frame acts as the display power toggle.
        if ir.is_low() {
            if let Some(code) = read_nec(&ir).await {
                state.ir_code = code;
                state.ir_count = state.ir_count.saturating_add(1);
                info!("IR NEC frame: 0x{:08X}", code);
                state.screen_off = !state.screen_off;
                if state.screen_off {
                    let _ = display.clear(BinaryColor::Off);
                    let _ = display.flush();
                    if second_display_available {
                        let _ = display2.clear(BinaryColor::Off);
                        let _ = display2.flush();
                    }
                } else {
                    let _ = draw(&mut display, &state, state.wifi_connected);
                    let _ = display.flush();
                    if second_display_available {
                        draw_aux(&mut display2, &state);
                        let _ = display2.flush();
                    }
                }
            }
            Timer::after_millis(80).await;
        }
        if !state.screen_off {
            draw(&mut display, &state, state.wifi_connected);
            let _ = display.flush();
        }
        if second_display_available && !state.screen_off {
            draw_aux(&mut display2, &state);
            let _ = display2.flush();
        }
        Timer::after_millis(30).await;
    }
}
