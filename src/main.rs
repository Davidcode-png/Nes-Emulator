pub struct CPU {
   pub register_a: u8,
   pub status: u8,
   pub register_x: u8,
   pub program_counter: u16,
}


pub fn update_status_flag(register: u8, status: &mut u8)
   {
        // Update Zero Flag (Z)
        if register == 0 {
            *status |= 0b0000_0010;
        } else {
            *status &= 0b1111_1101;
        }

        // Update Sign Flag (N)
        if register & 0b1000_0000 != 0 
        {
            *status |= 0b1000_0000;
        } else {
            *status &= 0b0111_1111;
        }
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



 
   pub fn interpret(&mut self, program: Vec<u8>) {
    self.program_counter = 0;
    loop {
        let opscode = program[self.program_counter as usize];
        self.program_counter += 1;

        match opscode {
        0xA9 => {
            let param = program[self.program_counter as usize];
            self.program_counter += 1;
            self.register_a = param;
            
            update_status_flag(self.register_a, &mut self.status);
        }

        0xAA => {
            self.register_x = self.register_a;
            update_status_flag(self.register_x, &mut self.status);
        }
        0x00 => {return ;}

            _ => todo!()
        }
    }
   }
}

#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn test_0xa9_lda(){
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 0x05);
        assert!(cpu.status & 0b0000_0010 == 0b00);
    }

    #[test]
    fn test_0xa9_lda_zero_flag(){
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9,0x00, 0x00]);
        assert_eq!(cpu.status & 0b0000_0010, 0b10);
        
    }

    #[test]
    fn test_moving_register_a_to_reg_x(){
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x04, 0x00]);
        assert_eq!(cpu.register_a, 0x04);
        cpu.interpret(vec![0xAA, 0x00]);
        assert_eq!(cpu.register_x, cpu.register_a);
        assert_eq!(cpu.register_x, 0x04);
    }
}


pub fn main(){
    
}


