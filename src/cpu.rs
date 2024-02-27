use bitflags::Flags;

pub struct CPU {
   pub register_a: u8,
   pub status: CpuFlags,
   pub register_x: u8,
   pub register_y: u8,
   pub stack_pointer: u8,
   pub program_counter: u16,
   memory: [u8; 0xFFFF]
}

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

bitflags!{

    pub struct CpuFlags: u8{
       const CARRY = 0b00000001;
       const ZERO = 0b00000010;
       const INTERRUPT_DISABLE = 0b00000100;
       const DECIMAL_MODE = 0b00001000;
       const BREAK = 0b00010000;
       const BREAK2 = 0b00100000;
       const OVERFLOW = 0b01000000;
       const NEGATIVE = 0b10000000;
    }

  }
const STACK:u16 = 0x1000;
const STACK_RESET: u8 = 0xfd;

impl CPU {
   pub fn new() -> Self {
       CPU {
           register_a: 0,
           status: CpuFlags::from_bits_truncate(0b100100),
           program_counter: 0,
           register_x: 0,
           register_y: 0,
           stack_pointer: STACK_RESET,
           memory: [0; 0xFFFF]
       }
   }

   fn get_operand_address(& mut self, mode: &AddressingMode) -> u16
   {
        match mode{
            AddressingMode::Immediate => self.program_counter,

            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,

            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),

            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_x) as u16;
                addr
            }

            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_y);
                addr.into() 
            }

            AddressingMode::Absolute_X => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_x.into());
                addr
            }

            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_y.into());
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

   pub fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }


   pub fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    fn mem_read_u16(&mut self, pos: u16) -> u16{
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

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.status = CpuFlags::from_bits_truncate(0b100100);
        self.stack_pointer = STACK_RESET;
        self.program_counter = self.mem_read_u16(0xFFFC);
    }


    pub fn load_and_run(&mut self, program: Vec<u8>){
        self.load(program);
        self.reset();
        self.interpret();
    }
    
    pub fn load(&mut self, program: Vec<u8>){
        self.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFc ,0x8000);
    }
   
    fn set_register_a(&mut self, value: u8) {
        self.register_a = value;
        self.update_status_flag(self.register_a);
    }

 fn update_status_flag(&mut self, register: u8)
   {
        // Update Zero Flag (Z)
        if register == 0 {
            self.status.insert(CpuFlags::ZERO);
        } else {
            self.status.remove(CpuFlags::ZERO);
        }

        // Update Sign Flag (N)
        if register & 0b1000_0000 != 0 
        {
            self.status.insert(CpuFlags::NEGATIVE);
        } else {
            self.status.remove(CpuFlags::NEGATIVE);
        }
   }

    
 fn stack_push(&mut self, data: u8){
   self.mem_write((STACK as u16) + self.stack_pointer as u16, data);
   self.stack_pointer = self.stack_pointer.wrapping_sub(1);

 }
 fn stack_push_u16(&mut self, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.stack_push(hi);
        self.stack_push(lo);
 }
    fn stack_pop(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.mem_read((STACK as u16) + self.stack_pointer as u16)
    }
 fn stack_pop_u16(&mut self) -> u16{
    let lo = self.stack_pop() as u16;
    let hi = self.stack_pop() as u16;
    
    hi << 8 | lo;
 }
    // Load value to register A
   fn lda(&mut self, mode: &AddressingMode){
        let addr = self.get_operand_address(&mode);
        let value = self.mem_read(addr);
        self.register_a = value;
        self.update_status_flag(self.register_a);
   }
   

   // Transfer register A to register X
   fn tax(&mut self){
        self.register_x = self.register_a;
        self.update_status_flag(self.register_x);
   }
    
    

   // Arithmetic Shift Left
   fn asl_accumulator(&mut self){
        let mut value = self.register_a;
        let carry = value > 128;
        if carry {self.status.insert(CpuFlags::CARRY);} else {self.status.remove(CpuFlags::CARRY);}
        value *= 2;
        self.set_register_a(value as u8);     
   }

   fn asl(&mut self, mode: &AddressingMode) -> u8{
        let addr = self.get_operand_address(&mode);
        let mut value = self.mem_read(addr);
        let carry = value > 128;
        if carry {self.status.insert(CpuFlags::CARRY);} else {self.status.remove(CpuFlags::CARRY);}
        value *= 2;
        self.mem_write(addr, value);
        self.update_status_flag(value);
        value
    }
    
    // INX (INcrement X)
   fn inx(&mut self){
       self.register_x = self.register_x.wrapping_add(1);
       self.update_status_flag(self.register_x);
   }
    
   // DEX (DEcrement X)
   fn dex(&mut self){
        self.register_x = self.register_x.wrapping_sub(1);
        self.update_status_flag(self.register_x);
   }
   
   fn add_to_register_a(&mut self, data: u8){
        let sum = self.register_a as u16 + data as u16 + (if self.status.contains(CpuFlags::CARRY){1} else {0}) as u16;
        let carry = sum > 255;
        if carry {self.status.insert(CpuFlags::CARRY);} else {self.status.remove(CpuFlags::CARRY);}
        let result = sum as u8;
        if (data ^ result) & (result ^ self.register_a) & 0x80 != 0 {self.status.insert(CpuFlags::OVERFLOW);} else {self.status.remove(CpuFlags::OVERFLOW);}
        self.set_register_a(result);

   } 
   // Add with Carry
   fn adc(&mut self, mode: &AddressingMode){
        let addr = self.get_operand_address(&mode);
        let value = self.mem_read(addr);
        self.add_to_register_a(value);
   }
   
   fn bitwise_and(&mut self, data: u8){
        let value = self.register_a & data;
        let zero :bool = value == 0;
        if zero {self.status.insert(CpuFlags::ZERO);} else {self.status.remove(CpuFlags::ZERO);}
        let result = value as u8;
        self.set_register_a(result);
   } 
   // Bitwise AND with accumulator
   fn and(&mut self, mode: &AddressingMode){
        let addr = self.get_operand_address(&mode);
        let value = self.mem_read(addr);
        self.bitwise_and(value);
   }

   fn sec(&mut self){
        self.status.insert(CpuFlags::CARRY);
   }

   fn rol_accumulator(&mut self){
        let value = self.register_a;
        let carry = self.status.contains(CpuFlags::CARRY);
        let new_carry = (value & 0b10000000) != 0;
        let shifted_value = (value << 1) | carry as u8;
        if new_carry{self.status.insert(CpuFlags::CARRY);} else {self.status.remove(CpuFlags::CARRY);}
        self.set_register_a(shifted_value);
   }
   
   fn rol(&mut self, mode: &AddressingMode) -> u8{
        let addr = self.get_operand_address(&mode);
        let value = self.mem_read(addr);
        let carry = self.status.contains(CpuFlags::CARRY);
        let new_carry = (value & 0b10000000) != 0;
        let shifted_value = (value << 1) | carry as u8;
        if new_carry{self.status.insert(CpuFlags::CARRY);} else {self.status.remove(CpuFlags::CARRY);}
        self.mem_write(addr, shifted_value);
        self.update_status_flag(value);
        shifted_value
   }

   fn sta(&mut self, mode: &AddressingMode){
       let addr = self.get_operand_address(&mode);
       self.mem_write(addr, self.register_a); 
   }

   /* Logical Inclusive OR*/
   fn ora(&mut self, mode: &AddressingMode){
        let addr = self.get_operand_address(&mode);
        let value = self.mem_read(addr);
        self.set_register_a(value | self.register_a);
   }

   pub fn interpret(&mut self) {
    loop {
        let opscode = self.mem_read(self.program_counter);
        self.program_counter += 1;

        match opscode {
        0xA9 => {
            self.lda(&AddressingMode::Immediate);
            self.program_counter += 1;
        }

        0xA5 => {
            self.lda(&AddressingMode::ZeroPage);
            self.program_counter += 1;
        }

        0xAD => {
            self.lda(&AddressingMode::Absolute);
            self.program_counter += 1;
        }

        0x69 => {
            self.adc(&AddressingMode::Immediate);
            self.program_counter += 1;
        }

        0x65 => {
            self.adc(&AddressingMode::ZeroPage);
            self.program_counter += 1;
        }

        0x6D => {
            self.adc(&AddressingMode::Absolute);
            self.program_counter += 1;
        }
        
        0x29 => {
            self.and(&AddressingMode::Immediate);
            self.program_counter += 1;
        }

        0x25 => {
            self.and(&AddressingMode::ZeroPage);
            self.program_counter += 1;
        }

        0x2D => {
            self.and(&AddressingMode::Absolute);
            self.program_counter += 1;
        }
        
        0x0A => self.asl_accumulator(),
    
        0x06 => {
            self.asl(&AddressingMode::ZeroPage);
            self.program_counter += 1;
        }

        0x16 => {
            self.asl(&AddressingMode::ZeroPage_X);
            self.program_counter += 1;
        }        


        0x09 => {
            self.ora(&AddressingMode::Immediate);
        }

        0x0D => {
            self.ora(&AddressingMode::Absolute);
        }

        0x11 => {
            self.ora(&AddressingMode::Indirect_Y);
        }

        0x0E => {
            self.asl(&AddressingMode::Absolute);
            self.program_counter += 1;
        }

        0x0E => {
            self.asl(&AddressingMode::Absolute_X);
            self.program_counter += 1;
        }

        0x85 => {
            self.sta(&AddressingMode::ZeroPage);
        }


        0x2A => {
            self.rol_accumulator();
        }
        

        /* JSR - Jump to Subroutine
         * The JSR instruction pushes the address (minus one) of the return point 
         * on to the stack and then sets the program counter to the target memory address. */
        0x20 => {
            self.stack_push_u16(self.program_counter + 2 -1);
            let target_memory_address = self.mem_read_u16(self.program_counter);
            self.program_counter = target_memory_address;
        }
        
        /* RTS - Return from sub routine */
        0x60 => {
            self.program_counter = self.stack_pop_u16() + 1;
        }

        0x38 => {
            self.sec();
        }
        0xAA => {
            self.tax();
        }

        0xE8 => {
            self.inx();
        }

        0xCA => {
            self.dex();
        }
        0x00 => return, 

            _ => panic!("Unknown {} opcode encountered", opscode),
        }
    }
   }
 }
