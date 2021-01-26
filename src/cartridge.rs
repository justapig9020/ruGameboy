use std::fs::File;
use std::io::prelude::*;
use crate::bus::Device;
use crate::memory::{Memory, Permission};
use log::{info, error};

const CARTRIDGE_TYPE_ADDR: usize = 0x147;
const ROM_SIZE_ADDR: usize = 0x148;
const RAM_SIZE_ADDR: usize = 0x149;
const KBYTE: usize = 1024;
const MBYTE: usize = 1024 * 1024;

pub fn cartridge_factory(base: usize, binary: Vec<u8>) -> Box<dyn Device> {
    identify_cartridge_type(binary[CARTRIDGE_TYPE_ADDR]);
    identify_rom_size(binary[ROM_SIZE_ADDR]);
    identify_ram_size(binary[RAM_SIZE_ADDR]);
    Box::new(Memory::new(base, binary, Permission::ReadOnly))
}

fn identify_cartridge_type(code: u8) {
    match code {
        0x0 => info!("ROM ONLY"),
        0x12 => info!("ROM+MBC3+RAM"),
        0x1 => info!("ROM+MBC1"),
        0x13 => info!("ROM+MBC3+RAM+BATT"),
        0x2 => info!("ROM+MBC1+RAM"),
        0x19 => info!("ROM+MBC5"),
        0x3 => info!("ROM+MBC1+RAM+BATT"),
        0x1A => info!("ROM+MBC5+RAM"),
        0x5 => info!("ROM+MBC2"),
        0x1B => info!("ROM+MBC5+RAM+BATT"),
        0x6 => info!("ROM+MBC2+BATTERY"),
        0x1C => info!("ROM+MBC5+RUMBLE"),
        0x8 => info!("ROM+RAM"),
        0x1D => info!("ROM+MBC5+RUMBLE+SRAM"),
        0x9 => info!("ROM+RAM+BATTERY"),
        0x1E => info!("ROM+MBC5+RUMBLE+SRAM+BATT"),
        0xB => info!("ROM+MMM01"),
        0x1F => info!("Pocket Camera"),
        0xC => info!("ROM+MMM01+SRAM"),
        0xFD => info!("Bandai TAMA5"),
        0xD => info!("ROM+MMM01+SRAM+BATT"),
        0xFE  => info!("Hudson HuC3"),
        0xF => info!("ROM+MBC3+TIMER+BATT"),
        0xFF  => info!("Hudson HuC1"),
        0x10 => info!("ROM+MBC3+TIMER+RAM+BATT"),
        0x11 => info!("ROM+MBC3"),
        _ => error!("Unknow type"),
    }
}

fn identify_rom_size(code: u8) {
    let bytes = match code {
        0x0 => 32 * KBYTE,
        0x1 => 64 * KBYTE,
        0x2 => 128 * KBYTE,
        0x3 => 256 * KBYTE,
        0x4 => 512 * KBYTE,
        0x5 => 1 * MBYTE,
        0x6 => 2 * MBYTE,
        // TODO: Some special spec not covered
        _ => {
            error!("Unknow rom size");
            0
        },
    };
    info!("ROM size: {} KBYTEs", bytes / KBYTE);
}

fn identify_ram_size(code: u8) {
    let bytes = match code {
        0 => 0,
        1 => 2,
        2 => 8,
        3 => 32,
        4 => 128,
        _ => {
            error!("Unknow ram size");
            0
        },
    } * KBYTE;
    info!("RAM size: {} KBYTEs", bytes / KBYTE);
}
