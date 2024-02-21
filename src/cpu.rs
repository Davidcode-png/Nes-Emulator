pub struct CPU {
   pub register_a: u8,
   pub status: u8,
   pub register_x: u8,
   pub register_y: u8,
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


impl CPU {
   pub fn new() -> Self {
       CPU {
           register_a: 0,
           status: 0,
           program_counter: 0,
           register_x: 0,
           register_y: 0,
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
        self.status = 0;

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
    

 fn update_status_flag(&mut self, register: u8)
   {
        // Update Zero Flag (Z)
        if register == 0 {
            self.status |= 0b0000_0010;
        } else {
            self.status &= 0b1111_1101;
        }

        // Update Sign Flag (N)
        if register & 0b1000_0000 != 0 
        {
            self.status |= 0b1000_0000;
        } else {
            self.status &= 0b0111_1111;
        }
   }


    // Load value to register A
   fn lda(&mut self, mode: &AddressingMode){
        println!("THIS IS THE MODE" );
        let addr = self.get_operand_address(&mode);
        println!("THIS IS THE ADDR {}", addr);
        let value = self.mem_read(addr);
        println!("THIS IS IT");
        println!("{}",value);
        self.register_a = value;
        self.update_status_flag(self.register_a);
        println!("This is register A");
        println!();
        println!("{}",self.register_a);
   }
   

   // Transfer register A to register X
   fn tax(&mut self){
        self.register_x = self.register_a;
        self.update_status_flag(self.register_x);
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

   pub fn interpret(&mut self) {
    loop {
        //self.program_counter += 1;
        let opscode = self.mem_read(self.program_counter);
        // println!("THIS IS THE OPSCODE {}",opscode);
        self.program_counter += 1;

        match opscode {
        0xA9 => {
            // println!("IT WAS MATCHED");
            self.lda(&AddressingMode::Immediate);
            // let param = program[self.program_counter as usize];
            self.program_counter += 1;
            // self.lda(param);     
        }

        0xA5 => {
            self.lda(&AddressingMode::ZeroPage);

            self.program_counter += 1;
        }

        0xAD => {
            self.lda(&AddressingMode::Absolute);
            self.program_counter += 1;
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

            _ => panic!("Unknown opcode encountered"),
        }
    }
   }
 }
