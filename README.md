# Aetheris

[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![ESP32-C6](https://img.shields.io/badge/ESP32--C6-E7352C?style=for-the-badge&logo=espressif&logoColor=white)](https://www.espressif.com/en/products/socs/esp32-c6)
[![Embassy](https://img.shields.io/badge/Embassy-async-4B6BFB?style=for-the-badge)](https://embassy.dev/)
[![Wokwi](https://img.shields.io/badge/Wokwi-simulator-5533FF?style=for-the-badge)](https://wokwi.com/)
[![SSD1306](https://img.shields.io/badge/SSD1306-OLED-167D9A?style=for-the-badge)](https://github.com/eldruin/ssd1306)
[![GitHub](https://img.shields.io/badge/GitHub-source-181717?style=for-the-badge&logo=github&logoColor=white)](https://github.com/)

**Aetheris — Powered by Seleste Technologies** is a Rust phone-style firmware
for the ESP32-C6. It uses the Espressif ROM and bootloader underneath; it is an
embedded device environment, not a PC-class operating system.

## Technology stack

```text
┌──────────────────────────────────────────────────────────────┐
│ Aetheris: login · home · app launcher · settings · piano      │
│ 24 compact apps · T9 text entry · wallpapers · sound cues       │
├──────────────────────────────────────────────────────────────┤
│ Rust 2024 · no_std · Embassy async runtime · esp-hal 1.2        │
├──────────────────────────────────────────────────────────────┤
│ SSD1306 I²C · embedded-graphics · GPIO matrix · Wi-Fi station   │
│ NEC IR decode · software tone generation · OTA-ready partitions │
├──────────────────────────────────────────────────────────────┤
│ ESP32-C6 · RISC-V · 2.4 GHz Wi-Fi · GPIO · I²C · internal flash │
└──────────────────────────────────────────────────────────────┘
```

```text
                ┌──────────── OLED 1 @ 0x3C ────────────┐
ESP32-C6 I²C ───┤                                      ├─ navigation UI
 GPIO6 SDA      └──────────── OLED 2 @ 0x3D ────────────┘  status dashboard
 GPIO7 SCL              (if address is selectable)

 GPIO0..3 ── keypad rows R1..R4     GPIO10,11,18,19 ── columns C1..C4
 GPIO4     ── IR receiver OUT       GPIO22 ── LED + series resistor
 GPIO23    ── passive buzzer IO     GND shared by every module
```

## Firmware behavior

### Boot, login and home

- The OLED boot sequence shows a falling digital-rain animation, Aetheris
  branding, “Powered by Seleste Technologies”, and a two-note boot cue.
- The login accepts username and password through phone-style multi-tap input.
  Defaults are username `aetheris` and password `1234`.
- On the login screen, `A` switches the username/password field, `B` or `*`
  deletes a character, `C` clears the selected field, and `D` toggles uppercase.
  Number keys use multi-tap groups (`2=abc`, `3=def`, …); `0` is space. Press
  `#` to submit. Repeated presses of the same key within 900 ms cycle its letters.
  Each key's digit is the final character in its group (for example, cycle `2`
  through `a`, `b`, `c`, then `2`).
- The Home screen opens Apps (`A`), Settings (`B`), Piano (`D`), or the Apps
  list (`*`); `#` opens Dice. The 128×64 navigation screen uses large 6×10
  labels, a 10×20 home title, vector drawn app icons, and a separate secondary
  OLED dashboard when a second address is available. The Settings icon is a
  drawn gear, and the apps menu contains 28 entries.
- Username, password, Wi-Fi credentials, notes and wallpaper choice currently
  live in RAM. The Security app can change login credentials for the current
  boot; credentials are not stored across reboot.

### Applications

| App | Behavior |
|---|---|
| Piano | 24 chromatic frequencies, C4–B5 (C♯/D♭ etc. share equal-tempered pitches); number keys plus A/B select notes; C/D choose octave; `#` enables a major triad arpeggio |
| Terminal | USB serial status/help, Wi-Fi scan and reboot commands |
| Matrix | Animated digital-rain display |
| Clock | Uptime clock |
| Timer | Adjustable countdown with completion melody |
| Stopwatch | Start/pause/reset stopwatch |
| Notes | T9 text editor, held in RAM |
| Calculator | 32-bit integer add/subtract/multiply/divide |
| IR Remote | NEC frame monitor and last-code display |
| Buzzer | Test tones |
| Wi-Fi | Scan, select an access point, enter its password and associate |
| System Info | ESP32-C6 identity and Wi-Fi link status |
| Settings | Wallpaper, Wi-Fi setup and reboot |
| Wallpaper | Switch the primary/secondary display theme |
| Counter | A increments, B decrements, C resets |
| Coin Flip | Animated coin toss with short tick sequence and result tone |
| Wi-Fi Scan | Scan and choose nearby access points |
| Sound Test | Play tones from the keypad |
| Key Test | Display/log the last key pressed |
| Display Test | Change display theme and check both OLEDs |
| Security | Edit username/password for the current boot |
| Update | Reports OTA readiness; network download/verification is not implemented |
| Reboot | Software reset |
| Power | OLED-off mode; any keypad key wakes the UI |
| Dice | Animated, audible D4/D6/D8/D10/D12/D20 roll; `A` changes die, `#` rolls |
| Metronome | Audible 40–240 BPM beat; `A/B` adjust tempo, `#` starts/stops |
| Snake | Keypad controlled grid game; `A/B/C/D` steer and `#` starts/pauses |
| Reaction Test | Wait for the GO cue and press `#` to measure reaction time |

The passive buzzer is driven with a full 3.3 V GPIO swing and a 50% tone duty
cycle. Firmware cannot make a directly connected passive buzzer louder than
that; use a transistor driver or audio amplifier for more volume. It cannot
produce speech or recorded voice. Major chords are played as an arpeggio because
one buzzer is a single sound source.

### Wi-Fi and IR

Wi-Fi setup performs an access point scan, shows the discovered SSIDs, lets you
select one, then accepts a T9 password. A successful station association lights
the green LED and shows `WIFI: ON ~))` on the home screen. The Wi-Fi link state
is not a guarantee of internet access; DHCP/IP networking and credential
persistence are not configured. Credentials are held only in RAM.

The IR receiver is sampled as a demodulating receiver and standard 32-bit NEC
frames are decoded/logged. Each valid NEC frame toggles display power, so a
remote button can put the OLEDs to sleep or wake them. Other IR protocols are
not decoded.

### OTA

The 16 MB flash layout reserves a factory image, two 2 MB OTA slots, OTA metadata,
NVS and a storage partition. The application currently does not download or
verify a firmware image or switch slots. A safe OTA client still needs network
transport, signed-image verification, rollback confirmation, and device testing.

## Wiring

Power logic from 3.3 V and share ground. Confirm pin labels and availability on
your exact carrier board/module before wiring. The plan avoids GPIO5, 8, 9 and
15 (boot/strapping-sensitive pins), GPIO12/13 (USB Serial/JTAG), and GPIO16/17
(UART0 console on common C6 boards). GPIO4 is assigned to the IR receiver as
requested; some older ESP32-C6 documentation also identifies GPIO4 as a strap
pin, so use an IR receiver whose OUT rests high and verify normal reset/boot
with the module connected.

| Part | Pin/signal | ESP32-C6 connection |
|---|---|---:|
| Both OLEDs | SDA | GPIO6 |
| Both OLEDs | SCK/SCL | GPIO7 |
| Both OLEDs | VDD | 3V3 |
| Both OLEDs | GND | GND |
| Matrix keypad | R1, R2, R3, R4 | GPIO0, GPIO1, GPIO2, GPIO3 |
| Matrix keypad | C1, C2, C3, C4 | GPIO10, GPIO11, GPIO18, GPIO19 |
| IR sensor | OUT | GPIO4 |
| IR sensor | VCC, GND | 3V3, GND |
| Green LED | GPIO side through 220–1kΩ resistor | GPIO22 |
| Green LED | Cathode | GND |
| Passive buzzer | IO | GPIO23; add a transistor if current requires it |
| Passive buzzer | GND, VSS | GND and module-rated supply per its label |

Use this standard 4×4 scan layout:

```text
R1: 1  2  3  A
R2: 4  5  6  B
R3: 7  8  9  C
R4: *  0  #  D
    C1 C2 C3 C4
```

Your listed key labels contain 15 keys (`0..8`, `A..D`, `*`, `#`), while the
description says the membrane has 8 keys. The firmware assumes the common
eight-wire 4×4 matrix, with one unused position. Verify the ribbon's row/column
mapping with a continuity meter; edit `KEYMAP` and the GPIO assignments if the
membrane is laid out differently.

### OLED placement

OLED 1 at `0x3C` is the primary navigation display. If OLED 2 can be strapped to
`0x3D`, it shows a separate Aetheris/Wi-Fi/current-app dashboard and uses the
alternate wallpaper. Many modules are fixed at `0x3C`; two such screens on the
same wires mirror one another and cannot show different images. Independent
control requires one `0x3D` display or a TCA9548A I²C multiplexer. The yellow/
blue panel's color split is fixed by the screen hardware. This driver expects
SSD1306-compatible modules; SH1106 panels need a matching driver.

## Flash layout

`partitions.csv` defines a 16 MB layout and `partitions.bin` is the generated
ESP-IDF binary table. Local `espflash.toml` selects this table and the 16 MB
flash size. Each application slot is 2 MB; the current release image fits. The
first flash writes this partition table, so it replaces any existing layout.

## Build and flash

```sh
cargo build --release
cargo run --release
```

`.cargo/config.toml` targets `riscv32imac-unknown-none-elf`; `cargo run` uses
`espflash` to flash and open the serial monitor. Connect USB and use the board's
BOOT/RESET download-mode procedure if required. This firmware has been
release-built and checked against the partition-table app slot, but it has not
been run on the physical board; confirm the module variant, OLED controller and
address, keypad wiring, and buzzer current before relying on it.

## Source map

```text
src/bin/main.rs   Hardware initialization, UI loop, Wi-Fi, keypad, IR, buzzer
src/lib.rs        Aetheris identity, app catalog, T9/piano tables
partitions.csv    Factory + OTA0/OTA1 + data partitions for 16 MB flash
partitions.bin    espflash-ready ESP-IDF partition table
espflash.toml     Selects the partition table and flash size
CODEX.md          Generated ESP32-C6 project rules
```
