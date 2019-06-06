use std::num::Wrapping as Wr;
use std::fs::File;
use std::path::Path;
use std::io::{BufReader, stdout};
use std::env::args;
use std::io::prelude::*;
use log::{info, debug, error};
use std::fmt;
use env_logger;

const OPCODE_TABLE: [fn(&mut State, a: u8, b: u8, c: u8) -> (); 23] = [
    State::nop, State::add, State::mul, State::sou, State::div, State::cop, State::afc, State::load,
    State::store, State::equ, State::nequ, State::inf, State::infe, State::sup, State::supe, State::jmp, 
    State::jmpc, State::jr,  State::jrc, State::and, State::or, State::xor, State::not
    ];

struct State {
    regs: [u16; 16],
    mem: [u8; 8192],
    bin: Vec<u8>,
    pub pc: u16
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "State {{ pc: {:?}, regs: {:?}}}", self.pc, self.regs)
    }
}

impl State {
    fn new(bin: Vec<u8>) -> State {
        State {regs: [0; 16], mem: [0; 8192], bin, pc: 0}
    }

    fn tick(&mut self) -> Result<(), String> {
        if usize::from(self.pc) + 4 > self.bin.len() {
            return Err("End of binary reached".to_string());
        }

        let pc = usize::from(self.pc);
        self.pc += 4;
        let inst_idx = self.bin[pc];

        let oper_a = self.bin[pc + 1];
        let oper_b = self.bin[pc + 2];
        let oper_c = self.bin[pc + 3];
        OPCODE_TABLE[inst_idx as usize](self, oper_a, oper_b, oper_c);

        Ok(())
    }

    fn make_u16(high: u8, low: u8) -> u16 {
        ((high as u16) << 8) + (low as u16)
    }

    fn nop(&mut self, _: u8, _: u8, _: u8) { }

    fn add(&mut self, reg_st: u8, reg_a: u8, reg_b: u8) {
        self.regs[reg_st as usize] = (Wr(self.regs[reg_a as usize]) + Wr(self.regs[reg_b as usize])).0;
    }

    fn mul(&mut self, reg_st: u8, reg_a: u8, reg_b: u8) {
        self.regs[reg_st as usize] = (Wr(self.regs[reg_a as usize]) * Wr(self.regs[reg_b as usize])).0;
    }

    fn sou(&mut self, reg_st: u8, reg_a: u8, reg_b: u8) {
        self.regs[reg_st as usize] = (Wr(self.regs[reg_a as usize]) - Wr(self.regs[reg_b as usize])).0;
    }

    fn div(&mut self, reg_st: u8, reg_a: u8, reg_b: u8) {
        self.regs[reg_st as usize] = (Wr(self.regs[reg_a as usize]) / Wr(self.regs[reg_b as usize])).0;
    }

    fn cop(&mut self, reg_st: u8, reg_a: u8, _: u8) {
        self.regs[reg_st as usize] = self.regs[reg_a as usize];
    }

    fn afc(&mut self, reg_st: u8, valh: u8, vall: u8) {
        self.regs[reg_st as usize] = Self::make_u16(valh, vall);
    }

    fn load(&mut self, reg_st: u8, addrh: u8, addrl: u8) {
        let addr = Self::make_u16(addrh, addrl) as usize;
        self.regs[reg_st as usize] = Self::make_u16(self.mem[addr], self.mem[addr + 1]);
    }

    fn store(&mut self, addrh: u8, addrl: u8, reg_st: u8) {
        let addr = Self::make_u16(addrh, addrl) as usize;
        let val = self.regs[reg_st as usize];
        self.mem[addr] = (val >> 8) as u8;
        self.mem[addr + 1] = val as u8;
    }

    fn equ(&mut self, reg_st: u8, reg_a: u8, reg_b: u8) {
        self.regs[reg_st as usize] = if self.regs[reg_a as usize] == self.regs[reg_b as usize] {1} else {0};
    }

    fn inf(&mut self, reg_st: u8, reg_a: u8, reg_b: u8) {
        self.regs[reg_st as usize] = if self.regs[reg_a as usize] < self.regs[reg_b as usize] {1} else {0};
    }

    fn infe(&mut self, reg_st: u8, reg_a: u8, reg_b: u8) {
        self.regs[reg_st as usize] = if self.regs[reg_a as usize] <= self.regs[reg_b as usize] {1} else {0};
    }

    fn sup(&mut self, reg_st: u8, reg_a: u8, reg_b: u8) {
        self.regs[reg_st as usize] = if self.regs[reg_a as usize] > self.regs[reg_b as usize] {1} else {0};
    }

    fn supe(&mut self, reg_st: u8, reg_a: u8, reg_b: u8) {
        self.regs[reg_st as usize] = if self.regs[reg_a as usize] < self.regs[reg_b as usize] {1} else {0};
    }

    fn jmp(&mut self, addrh: u8, addrl: u8, _: u8) {
        self.pc = Self::make_u16(addrh, addrl);
    }

    fn jmpc(&mut self, addrh: u8, addrl: u8, reg_a: u8) {
        if self.regs[reg_a as usize] == 0 {
            self.pc = Self::make_u16(addrh, addrl);
        }
    }

    fn nequ(&mut self, reg_st: u8, reg_a: u8, reg_b: u8) {
        self.regs[reg_st as usize] = (self.regs[reg_a as usize] != self.regs[reg_b as usize]) as u16;
    }

    fn jr(&mut self, addrh: u8, addrl: u8, reg_a: u8) {
        unimplemented!();
    }

    fn jrc(&mut self, addrh: u8, addrl: u8, reg_a: u8) {
        unimplemented!();
    }

    fn and(&mut self, reg_st: u8, reg_a: u8, reg_b: u8) {
        self.regs[reg_st as usize] = self.regs[reg_a as usize] & self.regs[reg_b as usize];
    }

    fn or(&mut self, reg_st: u8, reg_a: u8, reg_b: u8) {
        self.regs[reg_st as usize] = self.regs[reg_a as usize] | self.regs[reg_b as usize];
    }

    fn xor(&mut self, reg_st: u8, reg_a: u8, reg_b: u8) {
        self.regs[reg_st as usize] = self.regs[reg_a as usize] ^ self.regs[reg_b as usize];
    }

    fn not(&mut self, reg_st: u8, reg_a: u8, _: u8) {
        self.regs[reg_st as usize] = !self.regs[reg_a as usize]
    }

}

fn main() {
    env_logger::init();
    info!("Starting interpreter");

    let mut args = args().skip(1);
    let path = args.next().expect("No file provided");
    let path = Path::new(&path);
    let bin_file = File::open(path).expect("Error opening bin file");
    let bin_read = BufReader::new(bin_file);
    let bin: Vec<u8> = bin_read.bytes().map(|e| e.unwrap()).collect();

    let mut state = State::new(bin);
    
    loop {
        println!("{:?}", state);
        match state.tick() {
            Ok(()) => {}
            Err(err) => {println!("{}", err); break;}
        }
    }
}
