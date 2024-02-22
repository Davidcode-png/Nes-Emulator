use crate::cpu::CPU;


#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn test_0xa9_lda(){
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x55);
        println!("THIS IS THE MEM READ");
        println!("{}",cpu.mem_read(0x10));
        cpu.load_and_run(vec![0xa5, 0x10, 0x00]);
        assert_eq!(cpu.register_a, 0x55);
    }

    #[test]
    fn test_0xa9_lda_zero_flag(){
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9,0x00, 0x00]);
        assert_eq!(cpu.status.bits() & 0b0000_0010, 0b10);
        
    }

    #[test]
    fn test_moving_register_a_to_reg_x(){
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0x04, 0xAA,0x00]);
        assert_eq!(cpu.register_x, cpu.register_a);
        assert_eq!(cpu.register_x, 0x04);
    }

    #[test]
    fn test_increment_reg_x(){
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x03,0xAA,0xE8, 0x00]);
        assert_eq!(cpu.register_x, 4);
    }

    #[test]
   fn test_5_operations_working_together() {
       let mut cpu = CPU::new();
       cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
 
       assert_eq!(cpu.register_x, 0xc1);
   }
    

   #[test]
   fn test_decrement_reg_x(){
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0xAA, 0xCA, 0x00]);
        assert_eq!(cpu.register_x, 4);
   }

    #[test]
   fn test_add_with_carry(){
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0x10, 0x69, 0x01,0x00]);
        assert_eq!(cpu.register_a, 0x11);

   }


}

