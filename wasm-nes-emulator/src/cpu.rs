use std::collections::HashMap;
use crate::opcodes;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
   Immediate,
   ZeroPage,
   ZeroPage_X,
   ZeroPage_Y,
   Absolute,
   Absolute_X,
   Absolute_Y,
   Indirect_X,
   Indirect_Y,
   NoneAddressing,
}

//#[wasm_bindgen]
pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    //NV1BDIZC
    pub status: u8,
    pub program_counter: u16,
    memory: [u8; 0xFFFF]
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0,
            program_counter: 0,
            memory: [0; 0xFFFF],
        }
    }
    
    pub fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }
    
    pub fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }
    
    fn mem_read_u16(&mut self, pos: u16) -> u16 {
       let lo = self.mem_read(pos) as u16;
       let hi = self.mem_read(pos + 1) as u16;
       (hi << 8) | (lo as u16)
   }
    
    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }
    
    fn get_operand_address(&mut self, mode: &AddressingMode) -> u16 {

       match mode {
           AddressingMode::Immediate => self.program_counter,

           AddressingMode::ZeroPage  => self.mem_read(self.program_counter) as u16,
          
           AddressingMode::Absolute => self.mem_read_u16(self.program_counter),
        
           AddressingMode::ZeroPage_X => {
               let pos = self.mem_read(self.program_counter);
               let addr = pos.wrapping_add(self.register_x) as u16;
               addr
           }
           AddressingMode::ZeroPage_Y => {
               let pos = self.mem_read(self.program_counter);
               let addr = pos.wrapping_add(self.register_y) as u16;
               addr
           }

           AddressingMode::Absolute_X => {
               let base = self.mem_read_u16(self.program_counter);
               let addr = base.wrapping_add(self.register_x as u16);
               addr
           }
           AddressingMode::Absolute_Y => {
               let base = self.mem_read_u16(self.program_counter);
               let addr = base.wrapping_add(self.register_y as u16);
               addr
           }

           AddressingMode::Indirect_X => {
               let base = self.mem_read(self.program_counter);

               let ptr: u8 = (base as u8).wrapping_add(self.register_x);
               let lo = self.mem_read(ptr as u16);
               let hi = self.mem_read(ptr.wrapping_add(1) as u16);
               (hi as u16) << 8 | (lo as u16)
           }
           AddressingMode::Indirect_Y => {
               let base = self.mem_read(self.program_counter);

               let lo = self.mem_read(base as u16);
               let hi = self.mem_read((base as u8).wrapping_add(1) as u16);
               let deref_base = (hi as u16) << 8 | (lo as u16);
               let deref = deref_base.wrapping_add(self.register_y as u16);
               deref
           }
         
           AddressingMode::NoneAddressing => {
               panic!("mode {:?} is not supported", mode);
           }
       }
    }
    
    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000);
    }
    
    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.status = 0;
        self.program_counter = self.mem_read_u16(0xFFFC);
    }
    
    pub fn load_and_run(&mut self, program: Vec<u8>) {
       self.load(program);
       self.reset();
       self.run()
   }

    pub fn get_value(&mut self, mode: &AddressingMode) -> u8 {
        let addr = self.get_operand_address(mode);
        self.mem_read(addr)
    }

}

impl CPU {

    pub fn run(&mut self) {        
    let ref opcodes: HashMap<u8, &'static opcodes::OpCode> = *opcodes::OPSCODES_MAP;
    
        loop {
            //let opscode = self.mem_read(self.program_counter);
            let code = self.mem_read(self.program_counter);
            self.program_counter += 1;
            let program_counter_state = self.program_counter;
            
            let opcode = opcodes.get(&code).expect(&format!("Code: {:x} not found", code));
            
            match code {
                0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 => self.write_reg(&opcode.address_mode, self.register_a),
                0x86 | 0x96 | 0x8E => self.write_reg(&opcode.address_mode, self.register_x),
                0x84 | 0x94 | 0x8C => self.write_reg(&opcode.address_mode, self.register_y),
                0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => self.lda(&opcode.address_mode),
                0xA2 | 0xA6 | 0xB6 | 0xAE |0xBE => self.ldx(&opcode.address_mode),
                0xA0 | 0xA4 | 0xB4 | 0xAC |0xBC => self.ldy(&opcode.address_mode),
                0xAA => self.tax(),
                0x8A => self.txa(),
                0xCA => self.dex(),
                0xE8 => self.inx(),
                0xA8 => self.tay(),
                0x98 => self.tya(),
                0x88 => self.dey(),
                0xC8 => self.iny(),
                0x29 | 0x25 | 0x35 | 0x2D | 0x3D | 0x39 | 0x21 | 0x31 => self.and(&opcode.address_mode),
                0x49 | 0x45 | 0x55 | 0x4D | 0x5D | 0x59 | 0x41 | 0x51 => self.eor(&opcode.address_mode),
                0x09 | 0x05 | 0x15 | 0x0D | 0x1D | 0x19 | 0x01 | 0x11 => self.ora(&opcode.address_mode),
                0x69 | 0x65 | 0x75 | 0x6D | 0x7D | 0x79 | 0x61 | 0x71 => self.adc(&opcode.address_mode),
                0xE9 | 0xE5 | 0xF5 | 0xED | 0xFD | 0xF9 | 0xE1 | 0xF1 => self.sbc(&opcode.address_mode),
                0xEA => (),
                0x00 => {
                    return
                }
                _ => todo!()
            }
            
            
            if program_counter_state == self.program_counter {
                self.program_counter += (opcode.bytes - 1) as u16;
            }
            
        //println!("pc: {}, a: {}, x: {}, y: {}, op: {:#04x}", self.program_counter, self.register_a, self.register_x, self.register_y, opscode);
        }
    }
    
    fn write_reg(&mut self, mode: &AddressingMode, reg: u8) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, reg);
    }
    
    fn add_to_reg_a(&mut self, value: u8) {
        let mut sum: u16 = (self.register_a as u16) + (value as u16);
        if sum > 0xFF {
            sum = sum - 256;
            self.status = self.status | 0b0000_0001; //add carry flag
        } else {
             self.status = self.status & 0b1111_1110; //remove carry flag
        }
        if (value ^ (sum as u8)) & ((sum as u8) ^ self.register_a) & 0x80 != 0 {
             self.status = self.status | 0b0100_0000; //add overflow flag
        } else {
            self.status = self.status & 0b1011_1111; //remove overflow flag
        }
        self.register_a = sum as u8;
        self.update_zero_and_negative_flags(self.register_a);
    }
    
    fn adc(&mut self, mode: &AddressingMode) {
        let value = self.get_value(mode);
        self.add_to_reg_a(value);
    }
    
    fn sbc(&mut self, mode: &AddressingMode) {
        let value = self.get_value(mode);
        self.add_to_reg_a((value as i8).wrapping_neg().wrapping_sub(1) as u8);
    }
        
    fn lda(&mut self, mode: &AddressingMode) {
       let value = self.get_value(mode);
       self.register_a = value;
       self.update_zero_and_negative_flags(self.register_a);
   }
   
   fn ldx(&mut self, mode: &AddressingMode) {
       let value = self.get_value(mode);
       self.register_x = value;
       self.update_zero_and_negative_flags(self.register_x);
   }
   
   fn ldy(&mut self, mode: &AddressingMode) {
       let value = self.get_value(mode);
       self.register_y = value;
       self.update_zero_and_negative_flags(self.register_y);
   }
    
    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }
    
     fn txa(&mut self) {
        self.register_a = self.register_x;
        self.update_zero_and_negative_flags(self.register_x);
    }
    
    fn inx(&mut self) {
        if self.register_x == 255 {
            self.register_x = 0;
        } else {
            self.register_x += 1;
        }
        self.update_zero_and_negative_flags(self.register_x);
    }
    
    fn dex(&mut self) {
        if self.register_x == 0 {
            self.register_x = 255;
        } else {
            self.register_x -= 1;
        }
        self.update_zero_and_negative_flags(self.register_x);
    }
    
    fn tay(&mut self) {
        self.register_y = self.register_a;
        self.update_zero_and_negative_flags(self.register_y);
    }
    
     fn tya(&mut self) {
        self.register_a = self.register_y;
        self.update_zero_and_negative_flags(self.register_y);
    }
    
    fn iny(&mut self) {
        if self.register_y == 255 {
            self.register_y = 0;
        } else {
            self.register_y += 1;
        }
        self.update_zero_and_negative_flags(self.register_y);
    }
    
    fn dey(&mut self) {
        if self.register_y == 0 {
            self.register_y = 255;
        } else {
            self.register_y -= 1;
        }
        self.update_zero_and_negative_flags(self.register_y);
    }
    
    fn and(&mut self, mode: &AddressingMode) {
        let value = self.get_value(mode);
        self.register_a = self.register_a & value;
        self.update_zero_and_negative_flags(self.register_a);
    }
    
    fn ora(&mut self, mode: &AddressingMode) {
        let value = self.get_value(mode);
        self.register_a = self.register_a | value;
        self.update_zero_and_negative_flags(self.register_a);
    }
    
    fn eor(&mut self, mode: &AddressingMode) {
        let value = self.get_value(mode);
        self.register_a = self.register_a ^ value;
        self.update_zero_and_negative_flags(self.register_a);
    }
    
    fn update_zero_and_negative_flags(&mut self, result: u8) {
        if result == 0 {
            self.status = self.status | 0b0000_0010;
        } else {
            self.status = self.status & 0b1111_1101;
        }

        if result & 0b1000_0000 != 0 {
            self.status = self.status | 0b1000_0000;
        } else {
            self.status = self.status & 0b0111_1111;
        }
    }
    
}