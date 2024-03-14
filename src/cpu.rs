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
    #[derive (Copy, Clone)]
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
        self.memory[0x0600 .. (0x0600 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFc ,0x0600);
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
    hi << 8 | lo
 }
    // Load value to register A
   fn lda(&mut self, mode: &AddressingMode){
        let addr = self.get_operand_address(&mode);
        let value = self.mem_read(addr);
        self.register_a = value;
        self.update_status_flag(self.register_a);
   }

   fn ldx(&mut self, mode: &AddressingMode){
        let addr = self.get_operand_address(&mode);
        let value = self.mem_read(addr);
        self.register_x = value;
        self.update_status_flag(self.register_x);
   }
 
   fn ldy(&mut self, mode: &AddressingMode){
        let addr = self.get_operand_address(&mode);
        let value = self.mem_read(addr);
        self.register_y = value;
        self.update_status_flag(self.register_y);
   }
     

   // Transfer register A to register X
   fn tax(&mut self){
        self.register_x = self.register_a;
        self.update_status_flag(self.register_x);
   }

   fn txa(&mut self){
        self.set_register_a(self.register_x);
   }
   
   fn branch(&mut self, condition: bool){
        if condition{
            let jump: i8 = self.mem_read(self.program_counter) as i8;
            let jump_addr = self.program_counter.wrapping_add(1).wrapping_add(jump as u16);
            self.program_counter = jump_addr;
        }
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

   fn lsr_accumulaotor(&mut self){
        let mut value = self.register_a;
        let carry = value > 128;
        if carry {self.status.insert(CpuFlags::CARRY);} else {self.status.remove(CpuFlags::CARRY);}
        value /= 2;
        self.set_register_a(value as u8);
   }

   fn php(&mut self) {
        let mut flags = self.status.clone();
        flags.insert(CpuFlags::BREAK);
        flags.insert(CpuFlags::BREAK2);
        self.stack_push(flags.bits());
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
   
   fn bit(&mut self, mode: &AddressingMode){
        let addr = self.get_operand_address(&mode);
        let value = self.mem_read(addr);
        let result = self.register_a & value;
        // Updating the zero flag based on the result
        if result == 0{self.status.remove(CpuFlags::ZERO);} else{self.status.insert(CpuFlags::ZERO);}
        
        // Updating the Overflow and Negative flags based on the data
        self.status.set(CpuFlags::OVERFLOW, value & 0b01000000 > 0);
        self.status.set(CpuFlags::NEGATIVE, value & 0b10000000 > 0);
   } 

   fn compare(&mut self, mode: &AddressingMode, compare_with: u8){
        let addr = self.get_operand_address(&mode);
        let value = self.mem_read(addr);
        if value <= compare_with{self.status.insert(CpuFlags::CARRY);}
        else {self.status.remove(CpuFlags::CARRY);}
        self.update_status_flag(compare_with.wrapping_sub(1));
   }

   fn inc_mem(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(&mode);
        let mut value = self.mem_read(addr);
        value = value.wrapping_add(1);
        self.mem_write(addr, value);
        self.update_status_flag(value);
   }

   fn dec_mem(&mut self, mode: &AddressingMode){
        let addr = self.get_operand_address(&mode);
        let mut value = self.mem_read(addr);
        value = value.wrapping_sub(1);
        self.mem_write(addr, value);
        self.update_status_flag(value);
   }

   fn clc (&mut self) {
        self.status.remove(CpuFlags::CARRY);
   }

   /* Logical Inclusive OR*/
   fn ora(&mut self, mode: &AddressingMode){
        let addr = self.get_operand_address(&mode);
        let value = self.mem_read(addr);
        self.set_register_a(value | self.register_a);
   }

   pub fn interpret(&mut self) {
        self.run_with_callback(|_| {});
    }

    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut CPU),
    {
   //pub fn interpret(&mut self) {
    loop {
        let opscode = self.mem_read(self.program_counter);
        self.program_counter += 1;
        let program_counter_state = self.program_counter;
        println!("{}", opscode);
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
        
        0xB5 => {
            self.lda(&AddressingMode::ZeroPage);
            self.program_counter += 1;
        }

        0xA2 => {
            self.ldx(&AddressingMode::Immediate);
            self.program_counter += 1;
        }

        0xA6 => {
            self.ldx(&AddressingMode::ZeroPage);
            self.program_counter += 1;
        }


        0xA0 => {
            self.ldy(&AddressingMode::Immediate);
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

        0x35 => {
            self.and(&AddressingMode::ZeroPage_X);
            self.program_counter += 1;
        }

        0x25 => {
            self.and(&AddressingMode::ZeroPage);
            self.program_counter += 1;
        }

        0x2D => {
            self.and(&AddressingMode::Absolute);
            self.program_counter += 2;
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
            self.program_counter += 1;
        }

        0x0D => {
            self.ora(&AddressingMode::Absolute);
            self.program_counter += 2;
        }

        0x15 => {
            self.ora(&AddressingMode::ZeroPage_X);
            self.program_counter += 1;
        }

        0x11 => {
            self.ora(&AddressingMode::Indirect_Y);
            self.program_counter += 1;
        }

        0x19 => {
            self.ora(&AddressingMode::Absolute_Y);
            self.program_counter += 2;
        }

        0x01 => {
            self.ora(&AddressingMode::Indirect_X);
            self.program_counter += 1;
        }

        0x24 => {
            self.bit(&AddressingMode::ZeroPage);
            self.program_counter += 1;
        }

        0x10 => {
            self.branch(!self.status.contains(CpuFlags::NEGATIVE));
            self.program_counter += 1;
        }

        /*0xB0 => {
            self.branch(!self.status.contains(CpuFlags::CARRY));
        }*/

        0xD0 => {
            self.branch(!self.status.contains(CpuFlags::ZERO));
            self.program_counter += 1;
        }

        0xB0 => {
            self.branch(self.status.contains(CpuFlags::CARRY));
        }

        0xF0 => {
            self.branch(self.status.contains(CpuFlags::ZERO));
        }

        0x0E => {
            self.asl(&AddressingMode::Absolute);
            self.program_counter += 1;
        }

        0x1E => {
            self.asl(&AddressingMode::Absolute_X);
            self.program_counter += 1;
        }

        0x85 => {
            self.sta(&AddressingMode::ZeroPage);
            self.program_counter += 1;
        }

        0x95 => {
            self.sta(&AddressingMode::ZeroPage_X);
            self.program_counter += 1;
        }

        0x81 => {
            self.sta(&AddressingMode::Indirect_X);
            self.program_counter += 1;
        }
        
        0x91 => {
            self.sta(&AddressingMode::Indirect_Y);
            self.program_counter += 1;
        }

        0xE6 => {
            self.inc_mem(&AddressingMode::ZeroPage);
        }

        0xFE => {
            self.inc_mem(&AddressingMode::Absolute_X);
        }

        0xC6 => {
            self.dec_mem(&AddressingMode::ZeroPage);
        }

        0x2A => {
            self.rol_accumulator();
        }

        0x26 => {
            self.rol(&AddressingMode::ZeroPage);
        }

        0x4A => {
            self.lsr_accumulaotor();
        }

        0x4C => {
            let address = self.mem_read_u16(self.program_counter);
            self.program_counter = address;
            /* The Flags are not affected */
        }

        /* JSR - Jump to Subroutine
         * The JSR instruction pushes the address (minus one) of the return point 
         * on to the stack and then sets the program counter to the target memory address. */
        0x20 => {
            self.stack_push_u16(self.program_counter + 2 -1);
            let target_memory_address = self.mem_read_u16(self.program_counter);
            self.program_counter = target_memory_address;
        }

        0xC9 => {
            self.compare(&AddressingMode::Immediate, self.register_a);
            self.program_counter += 1 as u16;
        }

        0xC5 => {
            self.compare(&AddressingMode::ZeroPage, self.register_a);
        }

        0xE4 => {
            self.compare(&AddressingMode::ZeroPage, self.register_x);
        }
 
        
        /* RTS - Return from sub routine */
        0x60 => {
            self.program_counter = self.stack_pop_u16() + 1;
        }
        
        0x08 => {
            self.php();
        }

        0x38 => {
            self.sec();
        }
        0xAA => {
            self.tax();
        }

        0x8A => {
            self.txa();
        }

        0xE8 => {
            self.inx();
        }

        0x18 => {
            self.clc();
        }
        

        /* NOP - No Operation
         *The NOP instruction causes no changes to the processor other than the normal 
         incrementing of the program counter to the next instruction.*/
        0xEA => {
            self.program_counter += 1;
        }

        0xCA => {
            self.dex();
        }

        0x00 => return, 

            _ => panic!("Unknown {} opcode encountered", opscode),
        }

        callback(self);
    }
   }
 }
