pub struct CPU {
   pub register_a: u8,
   pub status: u8,
   pub register_x: u8,
   pub program_counter: u16,
}

impl CPU {
   pub fn new() -> Self {
       CPU {
           register_a: 0,
           status: 0,
           program_counter: 0,
           register_x: 0,
       }
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



   fn lda(&mut self, value: u8){
        self.register_a = value;
        self.update_status_flag(self.register_a)
   }
    
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

   pub fn interpret(&mut self, program: Vec<u8>) {
    self.program_counter = 0;
    loop {
        let opscode = program[self.program_counter as usize];
        self.program_counter += 1;

        match opscode {
        0xA9 => {
            let param = program[self.program_counter as usize];
            self.program_counter += 1;
            self.lda(param);     
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
        0x00 => {return ;}

            _ => panic!("Unknown opcode encountered"),
        }
    }
   }
}

