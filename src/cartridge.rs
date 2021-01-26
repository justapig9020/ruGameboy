use crate::bus::Device;
use crate::memory::{Memory, Permission};
use log::{info, error};

const CARTRIDGE_TYPE_ADDR: usize = 0x147;
const ROM_SIZE_ADDR: usize = 0x148;
const RAM_SIZE_ADDR: usize = 0x149;
const KBYTE: usize = 1024;
const MBYTE: usize = 1024 * 1024;

struct Type0 {
    rom: Memory
}

impl Type0 {
    fn new(base: usize, binary: Vec<u8>) -> Box<Type0> {
        Box::new(Type0 {
            rom: Memory::new(base, binary, Permission::ReadOnly),
        })
    }
}

impl Device for Type0 {
    fn load(&self, addr: u16) -> Result<u8, ()> {
        self.rom.load(addr)
    }
    fn store(&mut self, addr: u16, value: u8) -> Result<(), ()> {
        self.rom.store(addr, value)
    }
}

pub fn cartridge_factory(base: usize, binary: Vec<u8>) -> Box<dyn Device> {
    let code = binary[CARTRIDGE_TYPE_ADDR];
    match code {
        0x0 => Type0::new(base, binary),
        _ => panic!("Not supporting tyupe"),
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
