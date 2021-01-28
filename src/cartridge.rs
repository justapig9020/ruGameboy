use std::rc::Rc;
use std::cell::RefCell;
use crate::bus::Device;
use crate::memory::{Memory, Permission};
use log::{info, error};

const ROM_START: u16 = 0x0000;
const ROM_END: u16 = 0x3fff;
const SWITCHABLE_ROM_START: u16 = 0x4000;
const SWITCHABLE_ROM_END:   u16 = 0x7fff;
const CARTRIDGE_TYPE_ADDR: usize = 0x147;
const ROM_SIZE_ADDR: usize = 0x148;
const RAM_SIZE_ADDR: usize = 0x149;
const KBYTE: usize = 1024;
const MBYTE: usize = 1024 * 1024;
const BANK_SIZE: usize = 16 * KBYTE;

/// The cartridge with type code: 0
/// This kind of cartridge has signle ROM bank only.
struct Type0 {
    rom: Memory,
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

/// The cartridge with type code: 1
/// This kind of cartridge has a static ROM bank, several switchable
/// ROM banks and MBC1 as its memory controller.
struct Type1 {
    rom: Memory,
    rom_bank: Vec<Rc<RefCell<Memory>>>,
    rom_mapping: Rc<RefCell<Memory>>,
}

fn new_rom_banks(base: usize, bank_num: usize, binary: Vec<u8>) -> Vec<Rc<RefCell<Memory>>>    {
    let mut rom_bank = Vec::with_capacity(bank_num - 1);

    let rom_bank_addr_base = base + BANK_SIZE;
    for i in 1..bank_num {
        let mut rom_slice = vec![0; BANK_SIZE];
        let start = i * BANK_SIZE;
        let end = start + BANK_SIZE;
        rom_slice.clone_from_slice(&binary[start..end]);
        let bank = Rc::new(RefCell::new(Memory::new(rom_bank_addr_base, rom_slice, Permission::ReadOnly)));
        rom_bank.push(bank);
    }
    rom_bank
}

impl Type1 {
    fn new(base: usize, binary: Vec<u8>) -> Box<Type1> {
        let mut rom = vec![0; BANK_SIZE];
        rom.clone_from_slice(&binary[0..BANK_SIZE]);
        let rom = Memory::new(base, rom, Permission::ReadOnly);
        let bank_num = identify_rom_size(binary[ROM_SIZE_ADDR]);
        let rom_bank = new_rom_banks(base, bank_num, binary);
        let rom_mapping = rom_bank[0].clone();
        Box::new(Type1 {
            rom,
            rom_bank,
            rom_mapping,
        })
    }
    fn switch(&mut self, mut idx: usize) {
        idx -= 1;
        self.rom_mapping = self.rom_bank[idx].clone();
    }
}

impl Device for Type1 {
    fn load(&self, addr: u16) -> Result<u8, ()> {
        match addr {
            ROM_START..=ROM_END => self.rom.load(addr),
            SWITCHABLE_ROM_START..=SWITCHABLE_ROM_END => self.rom_mapping.borrow().load(addr),
            _ => Err(())
        }
    }
    fn store(&mut self, addr: u16, mut value: u8) -> Result<(), ()> {
        match addr {
            0x2000..=0x3fff => {
                if value == 0 {
                    value = 1;
                }
                info!("Switch to bank {}", value);
                self.switch(value as usize);
                Ok(())
            },
            _ => Err(())
        }
    }
}

pub fn cartridge_factory(base: usize, binary: Vec<u8>) -> Box<dyn Device> {
    let code = binary[CARTRIDGE_TYPE_ADDR];
    match code {
        0x0 => Type0::new(base, binary),
        0x1 => Type1::new(base, binary),
        _ => panic!("Not supporting tyupe"),
    }
}

fn identify_rom_size(code: u8) -> usize {
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
    bytes / BANK_SIZE
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
