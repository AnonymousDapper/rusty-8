// The MIT License (MIT)

// Copyright (c) 2021 AnonymousDapper

use super::System;

use std::collections::VecDeque;

#[derive(Debug)]
pub struct Disassembler {
    dis_buffer: VecDeque<String>,
}

impl Disassembler {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            dis_buffer: VecDeque::new(),
        }
    }

    pub fn print_state(&self, system: &System) {
        print!("\x1b[4;145H");
        let mut buffer = String::from("\x1b[32;40;4;1m< Registers >\x1b[0m\x1b[6;145H");

        buffer.push_str(&format!(
            "PC : \x1b[93;40m{:#06x}\x1b[0m\x1b[7;145H",
            system.pc
        ));

        buffer.push_str(&format!(
            "IR : \x1b[93;40m{:#06x}\x1b[0m\x1b[9;145H",
            system.ir
        ));

        buffer.push_str(&format!(
            "DT : \x1b[94;40m{:#04x}\x1b[0m\x1b[10;145H",
            system.dt.get_no_mod()
        ));

        buffer.push_str(&format!(
            "ST : \x1b[94;40m{:#04x}\x1b[0m\x1b[12;145H",
            system.st
        ));

        for (i, v) in system.registers.iter().enumerate() {
            buffer.push_str(&format!(
                "\x1b[95mV{:x}\x1b[30m : {:#04x}\x1b[{};145H",
                i,
                v,
                i + 13
            ));
        }

        buffer.push_str("\x1b[4;170H\x1b[32;40;4;1m< Stack >\x1b[0m\x1b[6;170H");

        for entry in system.mem.stack.iter().rev() {
            buffer.push_str(&format!("[ {:#04x} ]\x1b[1B\x1b[9D", entry));
        }

        buffer.push_str("\x1b[4;200H\x1b[32;40;4;1m< Disassembly >\x1b[0m");

        print!("{}\x1b[45;0H", buffer);
    }

    pub fn print_dis(&mut self, system: &System) {
        print!("\x1b[6;200H");

        let mut buffer = String::new();
        let (raw_op, decoded, helper) = system.read_decode();

        let dis_str = match decoded {
            (0, 0, 0xe, 0) => String::from("CLS"),
            (0, 0, 0xe, 0xe) => String::from("RET"),
            (0, _, _, _) => String::from("HALT"), //format!("SYS {:#05x} \x1b[37m[{:#06x}]\x1b[30m",helper.nnn,system.mem.read_u16(helper.nnn)),
            (0x1, _, _, _) => format!(
                "JP {:#05x} \x1b[37m[{:#06x}]\x1b[30m",
                helper.nnn,
                system.mem.read_u16(helper.nnn)
            ),
            (0x2, _, _, _) => format!(
                "CALL {:#05x} \x1b[37m[{:#06x}]\x1b[30m",
                helper.nnn,
                system.mem.read_u16(helper.nnn)
            ),
            (0x3, _, _, _) => format!("SE \x1b[95mV{:01x}\x1b[30m {:#04x}", helper.x, helper.nn),
            (0x4, _, _, _) => format!("SNE \x1b[95mV{:01x}\x1b[30m {:#04x}", helper.x, helper.nn),
            (0x5, _, _, _) => format!(
                "SE \x1b[95mV{:01x}\x1b[30m \x1b[95mV{:01x}\x1b[30m",
                helper.x, helper.y
            ),
            (0x6, _, _, _) => format!("LD \x1b[95mV{:01x}\x1b[30m {:#04x}", helper.x, helper.nn),
            (0x7, _, _, _) => format!("ADD \x1b[95mV{:01x}\x1b[30m {:#04x}", helper.x, helper.nn),
            (0x8, _, _, 0) => format!(
                "LD \x1b[95mV{:01x}\x1b[30m \x1b[95mV{:01x}\x1b[30m",
                helper.x, helper.y
            ),
            (0x8, _, _, 1) => format!(
                "OR \x1b[95mV{:01x}\x1b[30m \x1b[95mV{:01x}\x1b[30m",
                helper.x, helper.y
            ),
            (0x8, _, _, 2) => format!(
                "AND \x1b[95mV{:01x}\x1b[30m \x1b[95mV{:01x}\x1b[30m",
                helper.x, helper.y
            ),
            (0x8, _, _, 3) => format!(
                "XOR \x1b[95mV{:01x}\x1b[30m \x1b[95mV{:01x}\x1b[30m",
                helper.x, helper.y
            ),
            (0x8, _, _, 4) => format!(
                "ADD \x1b[95mV{:01x}\x1b[30m \x1b[95mV{:01x}\x1b[30m",
                helper.x, helper.y
            ),
            (0x8, _, _, 5) => format!(
                "SUB \x1b[95mV{:01x}\x1b[30m \x1b[95mV{:01x}\x1b[30m",
                helper.x, helper.y
            ),
            (0x8, _, _, 6) => format!(
                "SHR \x1b[95mV{:01x}\x1b[30m \x1b[95mV{:01x}\x1b[30m",
                helper.x, helper.y
            ),
            (0x8, _, _, 7) => format!(
                "SUBN \x1b[95mV{:01x}\x1b[30m \x1b[95mV{:01x}\x1b[30m",
                helper.x, helper.y
            ),
            (0x8, _, _, 0xe) => format!(
                "SHL \x1b[95mV{:01x}\x1b[30m \x1b[95mV{:01x}\x1b[30m",
                helper.x, helper.y
            ),
            (0x9, _, _, 0) => format!(
                "SNE \x1b[95mV{:01x}\x1b[30m \x1b[95mV{:01x}\x1b[30m",
                helper.x, helper.y
            ),
            (0xa, _, _, _) => format!(
                "LD \x1b[33mI\x1b[30m {:#05x} \x1b[37m[{:#06x}]\x1b[30m",
                helper.nnn,
                system.mem.read_u16(helper.nnn)
            ),
            (0xb, _, _, _) => format!(
                "JP \x1b[95mV0\x1b[30m {:#05x} \x1b[37m[{:#06x}]\x1b[30m",
                helper.nnn,
                system.mem.read_u16(helper.nnn)
            ),
            (0xc, _, _, _) => format!("RND \x1b[95mV{:01x}\x1b[30m {:#04x}", helper.x, helper.nn),
            (0xd, _, _, n) => format!(
                "DRW \x1b[95mV{:01x}\x1b[30m \x1b[95mV{:01x}\x1b[30m {:#03x}",
                helper.x, helper.y, n
            ),
            (0xe, _, 9, 0xe) => format!("SKP \x1b[95mV{:01x}\x1b[30m", helper.x),
            (0xe, _, 0xa, 1) => format!("SKNP \x1b[95mV{:01x}\x1b[30m", helper.x),
            (0xf, _, 0, 7) => format!("LD \x1b[95mV{:01x}\x1b[30m \x1b[34mDT\x1b[30m", helper.x),
            (0xf, _, 0, 0xa) => format!("LD \x1b[95mV{:01x}\x1b[30m \x1b[97mK\x1b[30m", helper.x),
            (0xf, _, 1, 5) => format!("LD \x1b[34mDT\x1b[30m \x1b[95mV{:01x}\x1b[30m", helper.x),
            (0xf, _, 1, 8) => format!("LD \x1b[34mST\x1b[30m \x1b[95mV{:01x}\x1b[30m", helper.x),
            (0xf, _, 1, 0xe) => format!("ADD \x1b[33mI\x1b[30m \x1b[95mV{:01x}\x1b[30m", helper.x),
            (0xf, _, 2, 9) => format!("LD \x1b[33mI\x1b[30m \x1b[95mV{:01x}\x1b[30m", helper.x),
            (0xf, _, 3, 3) => format!("BCD \x1b[95mV{:01x}\x1b[30m", helper.x),
            (0xf, _, 5, 5) => format!("LD [\x1b[33mI\x1b[30m] \x1b[95mV{:01x}\x1b[30m", helper.x),
            (0xf, _, 6, 5) => format!("LD \x1b[95mV{:01x}\x1b[30m [\x1b[33mI\x1b[30m]", helper.x),
            (0xf, 0xf, 0xf, 0xf) => String::from("* \x1b[31mBREAKPOINT\x1b[30m *"),

            (_, _, _, _) => String::from("! \x1b[101mUNKNOWN\x1b[30m"),
        };

        if self.dis_buffer.len() >= 32 {
            self.dis_buffer.pop_front();
        }
        self.dis_buffer.push_back(format!(
            "\x1b[0m{:#06x} | \x1b[90m({:#06x})\x1b[30m \x1b[1m{}",
            system.pc, raw_op, dis_str
        ));

        for (i, line) in self.dis_buffer.iter().enumerate() {
            buffer.push_str(&format!("\x1b[K{}\x1b[0m\x1b[{};200H", line, i + 7));
        }

        print!("{}\x1b[45;0H", buffer);
    }
}
