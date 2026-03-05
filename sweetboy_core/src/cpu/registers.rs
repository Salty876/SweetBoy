
use bitflags::bitflags;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Registers{
    pub(crate) a_reg: u8,
    b_reg: u8,
    c_reg: u8,
    d_reg: u8,
    e_reg: u8,
    f_reg: Flags,
    h_reg: u8,
    l_reg: u8
}

impl Registers {
    pub fn new() -> Self{
        Registers { 
            a_reg: 0,
            f_reg: Flags::empty(),
            b_reg: 0,
            c_reg: 0,
            d_reg: 0,
            e_reg: 0,
            h_reg: 0,
            l_reg: 0
         }
    }

    pub fn a(&self) -> u8 {
        self.a_reg
    }
    
    pub fn set_a(&mut self, value: u8) {
        self.a_reg = value;
    }

    pub fn f(&self) -> u8 {
        self.f_reg.bits()
    }

    pub fn set_f(&mut self, value: u8) {
        self.f_reg = Flags::from_bits_truncate(value);
    }
    
    pub fn b(&self) -> u8 {
        self.b_reg
    }

    pub fn set_b(&mut self, value: u8) {
        self.b_reg = value;
    }

    pub fn c(&self) -> u8 {
        self.c_reg
    }

    pub fn set_c(&mut self, value: u8) {
        self.c_reg = value;
    }

    pub fn d(&self) -> u8 {
        self.d_reg
    }

    pub fn set_d(&mut self, value: u8) {
        self.d_reg = value;
    }

    pub fn e(&self) -> u8 {
        self.e_reg
    }

    pub fn set_e(&mut self, value: u8) {
        self.e_reg = value;
    }

    pub fn h(&self) -> u8 {
        self.h_reg
    }

    pub fn set_h(&mut self, value: u8) {
        self.h_reg = value;
    }

    pub fn l(&self) -> u8 {
        self.l_reg
    }

    pub fn set_l(&mut self, value: u8) {
        self.l_reg = value;
    }

    pub fn get_af(&self) -> u16 {
        return (self.a_reg as u16) << 8 | self.f_reg.bits() as u16;
    }

    pub fn set_af(&mut self, value: u16) {
        self.a_reg = ((value & 0xFF00) >> 8) as u8;
        self.f_reg = Flags::from_bits_truncate(value as u8);
    }

    pub fn get_bc(&self) -> u16 {
        return (self.b_reg as u16) << 8 | self.c_reg as u16;
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b_reg = ((value & 0xFF00) >> 8) as u8;
        self.c_reg = (value & 0xFF) as u8;
    }

    pub fn get_de(&self) -> u16 {
        return (self.d_reg as u16) << 8 | self.e_reg as u16;
    }

    pub fn set_de(&mut self, value: u16) {
        self.d_reg = ((value & 0xFF00) >> 8) as u8;
        self.e_reg = (value & 0xFF) as u8;
    }

    pub fn get_hl(&self) -> u16 {
        return (self.h_reg as u16) << 8 | self.l_reg as u16;
    }

    pub fn set_hl(&mut self, value: u16) {
        self.h_reg = ((value & 0xFF00) >> 8) as u8;
        self.l_reg = (value & 0xFF) as u8;
    }

    // Getting flags
    pub fn get_z(&self) -> bool{
        self.f_reg.contains(Flags::z_flag)
    }

    pub fn get_n(&self) -> bool{
        self.f_reg.contains(Flags::n_flag)
    }

    pub fn get_hc(&self) -> bool{
        self.f_reg.contains(Flags::h_flag)
    }

    pub fn get_carry(&self) -> bool{
        self.f_reg.contains(Flags::c_flag)
    }

    // Setting flags
    pub fn set_z(&mut self, zf: bool){
        self.f_reg.set(Flags::z_flag, zf);
    }

    pub fn set_n(&mut self, nf: bool){
        self.f_reg.set(Flags::n_flag, nf);
    }

    pub fn set_hc(&mut self, hf: bool){
        self.f_reg.set(Flags::h_flag, hf);
    }

    pub fn set_carry(&mut self, cf: bool){
        self.f_reg.set(Flags::c_flag, cf);
    }
}

// struct Flags {
//      z_flag: bool, //ZERO FLAG; set to 1 if current op results in 0 or two values match a CMP operation
//      n_flag: bool, //SUBSTRACTION FLAG; set to 1 if substraction happens
//      h_flag: bool, //HALF CARRY FLAG; set to 1 if a carry occured from the lower nibble in the last operation
//      c_flag: bool, //CARRY FLAG; set to 1 if a carry occured in the last operation or if A is the smaller value on CP instruction
// }

bitflags! (
    #[derive(Serialize, Deserialize)]
    pub struct Flags: u8{
        const z_flag = 0b_1000_0000;
        const n_flag = 0b_0100_0000;
        const h_flag = 0b_0010_0000;
        const c_flag = 0b_0001_0000;

    }


);