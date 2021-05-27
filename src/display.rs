// The MIT License (MIT)

// Copyright (c) 2021 AnonymousDapper

pub const DISPLAY_HEIGHT: usize = 32;
pub const DISPLAY_WIDTH: usize = 64;

pub fn init<S: std::fmt::Display>(name: S) {
    let mut buffer = String::new();
    print!("{}\x1b[2J", termion::cursor::Hide);

    let header = &format!("{}\x1b[1B\x1b[0G", "█".repeat((DISPLAY_WIDTH * 2) + 4));
    let buf = &format!("\x1b[7C██\x1b[{}C██\x1b[1B\x1b[0G", (DISPLAY_WIDTH * 2));

    buffer.push_str(&format!(
        "\x1b[3;56HRusty-8 - CHIP-8 Emulator ({})\x1b[4;8H",
        name
    ));
    buffer.push_str(header);
    for _ in 0..DISPLAY_HEIGHT {
        buffer.push_str(buf);
    }

    buffer.push_str("\x1b[7C");
    buffer.push_str(header);

    print!("{}\x1b[45;0H", buffer);
}

pub fn write_display(buffer: &[u8]) {
    let mut display = String::new();
    print!("\x1b[5;10H");

    for (y, line) in buffer.chunks_exact(DISPLAY_WIDTH).enumerate() {
        display.push_str(&format!("\x1b[{};10H", y + 5));
        for (_x, pixel) in line.iter().enumerate() {
            display.push_str(&format!(
                "\x1b[{}m██\x1b[0m",
                if pixel & 0x1 == 1 { "97" } else { "30" }
            ));
        }
    }

    print!("{}\x1b[45;0H", display);
}
