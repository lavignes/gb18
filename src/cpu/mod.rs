use std::{mem};

use mmu::Mmu;

#[derive(Default)]
pub struct Cpu {
    pc: u16,
    sp: u16,
    af: u16,
    bc: u16,
    de: u16,
    hl: u16,

    stopped: bool,
}

#[derive(Copy, Clone)]
enum WideRegister {
    PC,
    SP,
    AF,
    BC,
    DE,
    HL,
}

#[derive(Copy, Clone)]
enum Register {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Copy, Clone)]
enum Flag {
    Zero      = 0x80,
    Negative  = 0x40,
    HalfCarry = 0x20,
    Carry     = 0x10,
}

#[derive(Copy, Clone)]
enum Condition {
    Zero,
    NotZero,
    Carry,
    NotCarry,
}

impl Cpu {
    #[inline]
    fn parts_mut<'a>(value: &'a mut u16) -> &'a mut [u8; 2] {
        // TODO: This may not be that efficient
        unsafe { mem::transmute(value) }
    }

    #[inline]
    fn parts(value: u16) -> [u8; 2] {
        // TODO: This may not be that efficient
        unsafe { mem::transmute(value) }
    }

    #[inline]
    fn flag(&self, flag: Flag) -> bool {
        (self.af & (flag as u16)) != 0
    }

    #[inline]
    fn set_flag(&mut self, flag: Flag, value: bool) {
        if value {
            self.af |= flag as u16;
        } else {
            self.af &= !(flag as u16);
        }
    }

    #[inline]
    fn read(&mut self, mmu: &impl Mmu) -> u8 {
        let value = mmu.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        value
    }

    #[inline]
    fn read_wide(&mut self, mmu: &impl Mmu) -> u16 {
        (self.read(mmu) as u16) | ((self.read(mmu) as u16) << 8)
    }

    #[inline]
    fn register(&self, reg: Register) -> u8 {
        match reg {
            Register::A => Self::parts(self.af)[1],
            Register::B => Self::parts(self.bc)[1],
            Register::C => Self::parts(self.bc)[0],
            Register::D => Self::parts(self.de)[1],
            Register::E => Self::parts(self.de)[0],
            Register::H => Self::parts(self.hl)[1],
            Register::L => Self::parts(self.hl)[0],
        }
    }

    #[inline]
    fn set_register(&mut self, reg: Register, value: u8) {
        match reg {
            Register::A => Self::parts_mut(&mut self.af)[1] = value,
            Register::B => Self::parts_mut(&mut self.bc)[1] = value,
            Register::C => Self::parts_mut(&mut self.bc)[0] = value,
            Register::D => Self::parts_mut(&mut self.de)[1] = value,
            Register::E => Self::parts_mut(&mut self.de)[0] = value,
            Register::H => Self::parts_mut(&mut self.hl)[1] = value,
            Register::L => Self::parts_mut(&mut self.hl)[0] = value,
        }
    }

    #[inline]
    fn wide_register(&self, reg: WideRegister) -> u16 {
        match reg {
            WideRegister::PC => self.pc,
            WideRegister::SP => self.sp,
            WideRegister::AF => self.af,
            WideRegister::BC => self.bc,
            WideRegister::DE => self.de,
            WideRegister::HL => self.hl,
        }
    }

    #[inline]
    fn set_wide_register(&mut self, reg: WideRegister, value: u16) {
        match reg {
            WideRegister::PC => self.pc = value,
            WideRegister::SP => self.sp = value,
            WideRegister::AF => self.af = value,
            WideRegister::BC => self.bc = value,
            WideRegister::DE => self.de = value,
            WideRegister::HL => self.hl = value,
        }
    }

    #[inline]
    fn nop(&self) -> usize {
        4
    }

    #[inline]
    fn read_wide_immediate(&mut self, reg: WideRegister, mmu: &impl Mmu) -> usize {
        let value = self.read_wide(mmu);
        self.set_wide_register(reg, value);
        12
    }

    #[inline]
    fn read_immediate(&mut self, reg: Register, mmu: &impl Mmu) -> usize {
        let value = self.read(mmu);
        self.set_register(reg, value);
        8
    }

    #[inline]
    fn write_a(&self, address: WideRegister, mmu: &mut impl Mmu) -> usize {
        mmu.write(self.wide_register(address), self.register(Register::A));
        8
    }

    #[inline]
    fn inc_wide(&mut self, reg: WideRegister) -> usize {
        let value = self.wide_register(reg).wrapping_add(1);
        self.set_wide_register(reg, value);
        8
    }

    #[inline]
    fn dec_wide(&mut self, reg: WideRegister) -> usize {
        let value = self.wide_register(reg).wrapping_sub(1);
        self.set_wide_register(reg, value);
        8
    }

    #[inline]
    fn inc(&mut self, reg: Register) -> usize {
        // TODO: inc/dec half-carry might be wrong
        let mut value = self.register(reg);
        self.set_flag(Flag::HalfCarry, (value & 0x0F) == 0x0F);
        value = value.wrapping_add(1);
        self.set_register(reg, value);
        self.set_flag(Flag::Zero, value == 0x00);
        self.set_flag(Flag::Negative, false);
        4
    }

    #[inline]
    fn dec(&mut self, reg: Register) -> usize {
        let mut value = self.register(reg);
        self.set_flag(Flag::HalfCarry, (value & 0x0F) == 0x00);
        value = value.wrapping_sub(1);
        self.set_register(reg, value);
        self.set_flag(Flag::Zero, value == 0x00);
        self.set_flag(Flag::Negative, true);
        4
    }

    #[inline]
    fn rlc(&mut self, reg: Register) -> usize {
        self.set_flag(Flag::Negative, false);
        self.set_flag(Flag::HalfCarry, false);
        let mut value = self.register(reg);
        let carry = (value & 0x80) >> 7;
        value = (value << 1) | carry;
        self.set_register(reg, value);
        self.set_flag(Flag::Carry, carry != 0);
        self.set_flag(Flag::Zero, value == 0x00);
        8
    }

    #[inline]
    fn rl(&mut self, reg: Register) -> usize {
        self.set_flag(Flag::Negative, false);
        self.set_flag(Flag::HalfCarry, false);
        let mut value = self.register(reg);
        let carry = (value & 0x80) >> 7;
        value = (value << 1) | if self.flag(Flag::Carry) { 0x01 } else { 0x00 };
        self.set_register(reg, value);
        self.set_flag(Flag::Carry, carry != 0);
        self.set_flag(Flag::Zero, value == 0x00);
        8
    }

    #[inline]
    fn rrc(&mut self, reg: Register) -> usize {
        self.set_flag(Flag::Negative, false);
        self.set_flag(Flag::HalfCarry, false);
        let mut value = self.register(reg);
        let carry = (value & 0x01) << 7;
        value = (value >> 1) | carry;
        self.set_register(reg, value);
        self.set_flag(Flag::Carry, carry != 0);
        self.set_flag(Flag::Zero, value == 0x00);
        8
    }

    #[inline]
    fn rr(&mut self, reg: Register) -> usize {
        self.set_flag(Flag::Negative, false);
        self.set_flag(Flag::HalfCarry, false);
        let mut value = self.register(reg);
        let carry = (value & 0x01) << 7;
        value = (value >> 1) | if self.flag(Flag::Carry) { 0x80 } else { 0x00 };
        self.set_register(reg, value);
        self.set_flag(Flag::Carry, carry != 0);
        self.set_flag(Flag::Zero, value == 0x00);
        8
    }

    #[inline]
    fn rlca(&mut self) -> usize {
        self.rlc(Register::A);
        self.set_flag(Flag::Zero, false);
        4
    }

    #[inline]
    fn rla(&mut self) -> usize {
        self.rl(Register::A);
        self.set_flag(Flag::Zero, false);
        4
    }

    #[inline]
    fn rrca(&mut self) -> usize {
        self.rrc(Register::A);
        self.set_flag(Flag::Zero, false);
        4
    }

    #[inline]
    fn rra(&mut self) -> usize {
        self.rr(Register::A);
        self.set_flag(Flag::Zero, false);
        4
    }

    #[inline]
    fn write_wide(&self, address: u16, value: u16, mmu: &mut impl Mmu,) {
        mmu.write(address, (value & 0x00FF) as u8);
        mmu.write(address.wrapping_add(1), ((value >> 8) & 0x00FF) as u8);
    }

    #[inline]
    fn write_stack_immediate(&mut self, mmu: &mut impl Mmu) -> usize {
        let address = self.read_wide(mmu);
        self.write_wide(address, self.sp, mmu);
        12 // TODO: GB manual says this is 20
    }

    #[inline]
    fn add_wide(&mut self, reg: WideRegister) -> usize {
        // TODO: half carry??
        let hl = self.hl as u32;
        let value = hl.wrapping_add(self.wide_register(reg) as u32);
        self.set_flag(Flag::Negative, false);
        self.set_flag(Flag::Carry, value > 0xFFFF);
        self.set_flag(Flag::HalfCarry, (value & 0x0FFF) < (hl & 0x0FFF));
        self.hl = (value & 0xFFFF) as u16;
        8
    }

    #[inline]
    fn read_a(&mut self, reg: WideRegister, mmu: &impl Mmu) -> usize {
        let value = mmu.read(self.wide_register(reg));
        self.set_register(Register::A, value);
        8
    }

    #[inline]
    fn stop(&mut self, mmu: &impl Mmu) -> usize {
        self.stopped = true;
        self.read(mmu);
        4
    }

    #[inline]
    fn jr(&mut self, mmu: &impl Mmu) -> usize {
        // TODO: How does wrapping work here?
        let pc = (self.pc as i16).wrapping_add((self.read(mmu) as i8) as i16);
        self.pc = pc as u16;
        12
    }

    #[inline]
    fn jr_condition(&mut self, condition: Condition, mmu: &impl Mmu) -> usize {
        // TODO: How does wrapping work here?
        let pc = (self.pc as i16).wrapping_add((self.read(mmu) as i8) as i16);
        let met = match condition {
            Condition::Zero => self.flag(Flag::Zero),
            Condition::NotZero => !self.flag(Flag::Zero),
            Condition::Carry => self.flag(Flag::Carry),
            Condition::NotCarry => !self.flag(Flag::Carry),
        };
        if met {
            self.pc = pc as u16;
            12
        } else {
            8
        }
    }

    #[inline]
    fn write_a_hli(&mut self, mmu: &mut impl Mmu) -> usize {
        let address = self.wide_register(WideRegister::HL);
        mmu.write(address, self.register(Register::A));
        self.hl = address.wrapping_add(1);
        8
    }

    #[inline]
    fn write_a_hld(&mut self, mmu: &mut impl Mmu) -> usize {
        let address = self.wide_register(WideRegister::HL);
        mmu.write(address, self.register(Register::A));
        self.hl = address.wrapping_sub(1);
        8
    }

    #[inline]
    fn read_a_hli(&mut self, mmu: &impl Mmu) -> usize {
        let address = self.wide_register(WideRegister::HL);
        let value = mmu.read(address);
        self.set_register(Register::A, value);
        self.hl = address.wrapping_add(1);
        8
    }

    #[inline]
    fn read_a_hld(&mut self, mmu: &impl Mmu) -> usize {
        let address = self.wide_register(WideRegister::HL);
        let value = mmu.read(address);
        self.set_register(Register::A, value);
        self.hl = address.wrapping_sub(1);
        8
    }


    #[inline]
    fn cpl(&mut self) -> usize {
        let a = self.register(Register::A);
        self.set_register(Register::A, !a);
        4
    }

    #[inline]
    fn inc_mem(&mut self, mmu: &mut impl Mmu) -> usize {
        let address = self.wide_register(WideRegister::HL);
        let mut value = mmu.read(address);
        self.set_flag(Flag::HalfCarry, (value & 0x0F) == 0x0F);
        value = value.wrapping_add(1);
        mmu.write(address, value);
        self.set_flag(Flag::Zero, value == 0x00);
        self.set_flag(Flag::Negative, false);
        12
    }

    #[inline]
    fn dec_mem(&mut self, mmu: &mut impl Mmu) -> usize {
        let address = self.wide_register(WideRegister::HL);
        let mut value = mmu.read(address);
        self.set_flag(Flag::HalfCarry, (value & 0x0F) == 0x00);
        value = value.wrapping_sub(1);
        mmu.write(address, value);
        self.set_flag(Flag::Zero, value == 0x00);
        self.set_flag(Flag::Negative, true);
        12
    }

    #[inline]
    fn write_mem_immediate(&mut self, mmu: &mut impl Mmu) -> usize {
        let value = self.read(mmu);
        mmu.write(self.wide_register(WideRegister::HL), value);
        12
    }

    pub fn cycle(&mut self, mmu: &mut impl Mmu) -> usize {
        let opcode = self.read(mmu);
        match opcode {
            0x00 => { self.nop() },
            0x01 => { self.read_wide_immediate(WideRegister::BC, mmu) }
            0x02 => { self.write_a(WideRegister::BC, mmu) }
            0x03 => { self.inc_wide(WideRegister::BC) }
            0x04 => { self.inc(Register::B) }
            0x05 => { self.dec(Register::B) }
            0x06 => { self.read_immediate(Register::B, mmu) }
            0x07 => { self.rlca() }
            0x08 => { self.write_stack_immediate(mmu) }
            0x09 => { self.add_wide(WideRegister::BC) }
            0x0A => { self.read_a(WideRegister::BC, mmu) }
            0x0B => { self.dec_wide(WideRegister::BC) }
            0x0C => { self.inc(Register::C) }
            0x0D => { self.dec(Register::C) }
            0x0E => { self.read_immediate(Register::C, mmu) }
            0x0F => { self.rrca() }

            0x10 => { self.stop(mmu) }
            0x11 => { self.read_wide_immediate(WideRegister::DE, mmu) }
            0x12 => { self.write_a(WideRegister::DE, mmu) }
            0x13 => { self.inc_wide(WideRegister::DE) }
            0x14 => { self.inc(Register::D) }
            0x15 => { self.dec(Register::D) }
            0x16 => { self.read_immediate(Register::D, mmu) }
            0x17 => { self.rla() }
            0x18 => { self.jr(mmu) }
            0x19 => { self.add_wide(WideRegister::DE) }
            0x1A => { self.read_a(WideRegister::DE, mmu) }
            0x1B => { self.dec_wide(WideRegister::DE) }
            0x1C => { self.inc(Register::E) }
            0x1D => { self.dec(Register::E) }
            0x1E => { self.read_immediate(Register::E, mmu) }
            0x1F => { self.rra() }

            0x20 => { self.jr_condition(Condition::NotZero, mmu) }
            0x21 => { self.read_wide_immediate(WideRegister::HL, mmu) }
            0x22 => { self.write_a_hli(mmu) }
            0x23 => { self.inc_wide(WideRegister::HL) }
            0x24 => { self.inc(Register::H) }
            0x25 => { self.dec(Register::H) }
            0x26 => { self.read_immediate(Register::H, mmu) }
            0x27 => { unimplemented!() }
            0x28 => { self.jr_condition(Condition::Zero, mmu) }
            0x29 => { self.add_wide(WideRegister::HL) }
            0x2A => { self.read_a_hli(mmu) }
            0x2B => { self.dec_wide(WideRegister::HL) }
            0x2C => { self.inc(Register::L) }
            0x2D => { self.dec(Register::L) }
            0x2E => { self.read_immediate(Register::L, mmu) }
            0x2F => { self.cpl() }

            0x30 => { self.jr_condition(Condition::NotCarry, mmu) }
            0x31 => { self.read_wide_immediate(WideRegister::SP, mmu) }
            0x32 => { self.write_a_hld(mmu) }
            0x33 => { self.inc_wide(WideRegister::SP) }
            0x34 => { self.inc_mem(mmu) }
            0x35 => { self.dec_mem(mmu) }
            0x36 => { self.write_mem_immediate(mmu) }
            0x37 => { unimplemented!() }
            0x38 => { self.jr_condition(Condition::Carry, mmu) }
            0x39 => { self.add_wide(WideRegister::SP) }
            0x3A => { self.read_a_hld(mmu) }
            0x3B => { self.dec_wide(WideRegister::SP) }
            0x3C => { self.inc(Register::A) }
            0x3D => { self.dec(Register::A) }
            0x3E => { self.read_immediate(Register::A, mmu) }
            0x3F => { unimplemented!() }

            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Mmu for Vec<u8> {
        fn read(&self, address: u16) -> u8 {
            self[address as usize]
        }

        fn write(&mut self, address: u16, value: u8) {
            self[address as usize] = value;
        }
    }

    #[test]
    fn nop() {
        let mut cpu = Cpu::default();
        assert_eq!(4, cpu.nop());
    }

    #[test]
    fn read_wide_immediate() {
        let mut cpu = Cpu::default();
        let mut mmu = vec!(0xAD, 0xDE);
        assert_eq!(12, cpu.read_wide_immediate(WideRegister::BC, &mut mmu));
        assert_eq!(0xDEAD, cpu.wide_register(WideRegister::BC));
        assert_eq!(0x0002, cpu.pc);
    }

    #[test]
    fn write_a() {
        let mut cpu = Cpu::default();
        cpu.set_wide_register(WideRegister::BC, 0x01);
        cpu.set_register(Register::A, 0x42);
        let mut mmu = vec!(0x00, 0x00);
        assert_eq!(8, cpu.write_a(WideRegister::BC, &mut mmu));
        assert_eq!(0x42, mmu.read(0x01));
    }

    #[test]
    fn inc_wide() {
        let mut cpu = Cpu::default();
        cpu.set_wide_register(WideRegister::BC, 0xFFFE);
        assert_eq!(8, cpu.inc_wide(WideRegister::BC));
        assert_eq!(0xFFFF, cpu.wide_register(WideRegister::BC));
        assert_eq!(8, cpu.inc_wide(WideRegister::BC));
        assert_eq!(0x0000, cpu.wide_register(WideRegister::BC));
    }

    #[test]
    fn inc() {
        let mut cpu = Cpu::default();
        cpu.set_register(Register::A, 0x0F);
        assert_eq!(4, cpu.inc(Register::A));
        assert_eq!(0x10, cpu.register(Register::A));
        assert_eq!(false, cpu.flag(Flag::Negative));
        assert_eq!(true, cpu.flag(Flag::HalfCarry));
        assert_eq!(false, cpu.flag(Flag::Zero));

        let mut cpu = Cpu::default();
        cpu.set_register(Register::B, 0xFF);
        assert_eq!(4, cpu.inc(Register::B));
        assert_eq!(0x00, cpu.register(Register::B));
        assert_eq!(false, cpu.flag(Flag::Negative));
        assert_eq!(true, cpu.flag(Flag::HalfCarry));
        assert_eq!(true, cpu.flag(Flag::Zero));

        let mut cpu = Cpu::default();
        cpu.set_register(Register::C, 0x41);
        assert_eq!(4, cpu.inc(Register::C));
        assert_eq!(0x42, cpu.register(Register::C));
        assert_eq!(false, cpu.flag(Flag::Negative));
        assert_eq!(false, cpu.flag(Flag::HalfCarry));
        assert_eq!(false, cpu.flag(Flag::Zero));
    }

    #[test]
    fn dec() {
        let mut cpu = Cpu::default();
        cpu.set_register(Register::A, 0x10);
        assert_eq!(4, cpu.dec(Register::A));
        assert_eq!(0x0F, cpu.register(Register::A));
        assert_eq!(true, cpu.flag(Flag::Negative));
        assert_eq!(true, cpu.flag(Flag::HalfCarry));
        assert_eq!(false, cpu.flag(Flag::Zero));

        let mut cpu = Cpu::default();
        cpu.set_register(Register::B, 0x00);
        assert_eq!(4, cpu.dec(Register::B));
        assert_eq!(0xFF, cpu.register(Register::B));
        assert_eq!(true, cpu.flag(Flag::Negative));
        assert_eq!(true, cpu.flag(Flag::HalfCarry));
        assert_eq!(false, cpu.flag(Flag::Zero));

        let mut cpu = Cpu::default();
        cpu.set_register(Register::C, 0x01);
        assert_eq!(4, cpu.dec(Register::C));
        assert_eq!(0x00, cpu.register(Register::C));
        assert_eq!(true, cpu.flag(Flag::Negative));
        assert_eq!(false, cpu.flag(Flag::HalfCarry));
        assert_eq!(true, cpu.flag(Flag::Zero));
    }

    #[test]
    fn read_immediate() {
        let mut cpu = Cpu::default();
        let mut mmu = vec!(0x42);
        assert_eq!(8, cpu.read_immediate(Register::B, &mut mmu));
        assert_eq!(0x42, cpu.register(Register::B));
        assert_eq!(0x0001, cpu.pc);
    }

    #[test]
    fn rlc() {
        let mut cpu = Cpu::default();
        cpu.set_register(Register::B, 0x08);
        assert_eq!(8, cpu.rlc(Register::B));
        assert_eq!(0x10, cpu.register(Register::B));
        assert_eq!(false, cpu.flag(Flag::Negative));
        assert_eq!(false, cpu.flag(Flag::HalfCarry));
        assert_eq!(false, cpu.flag(Flag::Zero));
        assert_eq!(false, cpu.flag(Flag::Carry));

        let mut cpu = Cpu::default();
        cpu.set_register(Register::C, 0x80);
        assert_eq!(8, cpu.rlc(Register::C));
        assert_eq!(0x01, cpu.register(Register::C));
        assert_eq!(false, cpu.flag(Flag::Negative));
        assert_eq!(false, cpu.flag(Flag::HalfCarry));
        assert_eq!(false, cpu.flag(Flag::Zero));
        assert_eq!(true, cpu.flag(Flag::Carry));

        let mut cpu = Cpu::default();
        cpu.set_register(Register::D, 0x00);
        assert_eq!(8, cpu.rlc(Register::D));
        assert_eq!(0x00, cpu.register(Register::D));
        assert_eq!(false, cpu.flag(Flag::Negative));
        assert_eq!(false, cpu.flag(Flag::HalfCarry));
        assert_eq!(true, cpu.flag(Flag::Zero));
        assert_eq!(false, cpu.flag(Flag::Carry));
    }

    #[test]
    fn rlca() {
        let mut cpu = Cpu::default();
        cpu.set_register(Register::A, 0x08);
        assert_eq!(4, cpu.rlca());
        assert_eq!(0x10, cpu.register(Register::A));
        assert_eq!(false, cpu.flag(Flag::Negative));
        assert_eq!(false, cpu.flag(Flag::HalfCarry));
        assert_eq!(false, cpu.flag(Flag::Zero));
        assert_eq!(false, cpu.flag(Flag::Carry));

        let mut cpu = Cpu::default();
        cpu.set_register(Register::A, 0x80);
        assert_eq!(4, cpu.rlca());
        assert_eq!(0x01, cpu.register(Register::A));
        assert_eq!(false, cpu.flag(Flag::Negative));
        assert_eq!(false, cpu.flag(Flag::HalfCarry));
        assert_eq!(false, cpu.flag(Flag::Zero));
        assert_eq!(true, cpu.flag(Flag::Carry));

        let mut cpu = Cpu::default();
        cpu.set_register(Register::A, 0x00);
        assert_eq!(4, cpu.rlca());
        assert_eq!(0x00, cpu.register(Register::A));
        assert_eq!(false, cpu.flag(Flag::Negative));
        assert_eq!(false, cpu.flag(Flag::HalfCarry));
        assert_eq!(false, cpu.flag(Flag::Zero));
        assert_eq!(false, cpu.flag(Flag::Carry));
    }

    #[test]
    fn write_stack_immediate() {
        let mut cpu = Cpu::default();
        let mut mmu = vec!(0x03, 0x00, 0x00, 0x00, 0x00);
        cpu.set_wide_register(WideRegister::SP, 0xDEAD);
        assert_eq!(12, cpu.write_stack_immediate(&mut mmu));
        assert_eq!(0xAD, mmu.read(0x03));
        assert_eq!(0xDE, mmu.read(0x04));
    }

    #[test]
    fn add_wide() {
        let mut cpu = Cpu::default();
        cpu.set_wide_register(WideRegister::HL, 0x0040);
        cpu.set_wide_register(WideRegister::BC, 0x0002);
        assert_eq!(8, cpu.add_wide(WideRegister::BC));
        assert_eq!(0x0042, cpu.wide_register(WideRegister::HL));
        assert_eq!(0x0002, cpu.wide_register(WideRegister::BC));
        assert_eq!(false, cpu.flag(Flag::Negative));
        assert_eq!(false, cpu.flag(Flag::Carry));
        assert_eq!(false, cpu.flag(Flag::HalfCarry));

        let mut cpu = Cpu::default();
        cpu.set_wide_register(WideRegister::HL, 0xFFFF);
        cpu.set_wide_register(WideRegister::BC, 0x0001);
        assert_eq!(8, cpu.add_wide(WideRegister::BC));
        assert_eq!(0x0000, cpu.wide_register(WideRegister::HL));
        assert_eq!(0x0001, cpu.wide_register(WideRegister::BC));
        assert_eq!(false, cpu.flag(Flag::Negative));
        assert_eq!(true, cpu.flag(Flag::Carry));
        assert_eq!(true, cpu.flag(Flag::HalfCarry));

        let mut cpu = Cpu::default();
        cpu.set_wide_register(WideRegister::HL, 0x0FFE);
        cpu.set_wide_register(WideRegister::BC, 0x0002);
        assert_eq!(8, cpu.add_wide(WideRegister::BC));
        assert_eq!(0x1000, cpu.wide_register(WideRegister::HL));
        assert_eq!(0x0002, cpu.wide_register(WideRegister::BC));
        assert_eq!(false, cpu.flag(Flag::Negative));
        assert_eq!(false, cpu.flag(Flag::Carry));
        assert_eq!(true, cpu.flag(Flag::HalfCarry));
    }

    #[test]
    fn read_a() {
        let mut cpu = Cpu::default();
        let mut mmu = vec!(0x00, 0x00, 0x42);
        cpu.set_wide_register(WideRegister::BC, 0x02);
        assert_eq!(8, cpu.read_a(WideRegister::BC, &mut mmu));
        assert_eq!(0x42, cpu.register(Register::A));
    }

    #[test]
    fn dec_wide() {
        let mut cpu = Cpu::default();
        cpu.set_wide_register(WideRegister::BC, 0x0001);
        assert_eq!(8, cpu.dec_wide(WideRegister::BC));
        assert_eq!(0x0000, cpu.wide_register(WideRegister::BC));
        assert_eq!(8, cpu.dec_wide(WideRegister::BC));
        assert_eq!(0xFFFF, cpu.wide_register(WideRegister::BC));
    }

    #[test]
    fn rrc() {
        let mut cpu = Cpu::default();
        cpu.set_register(Register::B, 0x10);
        assert_eq!(8, cpu.rrc(Register::B));
        assert_eq!(0x08, cpu.register(Register::B));
        assert_eq!(false, cpu.flag(Flag::Negative));
        assert_eq!(false, cpu.flag(Flag::HalfCarry));
        assert_eq!(false, cpu.flag(Flag::Zero));
        assert_eq!(false, cpu.flag(Flag::Carry));

        let mut cpu = Cpu::default();
        cpu.set_register(Register::C, 0x01);
        assert_eq!(8, cpu.rrc(Register::C));
        assert_eq!(0x80, cpu.register(Register::C));
        assert_eq!(false, cpu.flag(Flag::Negative));
        assert_eq!(false, cpu.flag(Flag::HalfCarry));
        assert_eq!(false, cpu.flag(Flag::Zero));
        assert_eq!(true, cpu.flag(Flag::Carry));

        let mut cpu = Cpu::default();
        cpu.set_register(Register::D, 0x00);
        assert_eq!(8, cpu.rrc(Register::D));
        assert_eq!(0x00, cpu.register(Register::D));
        assert_eq!(false, cpu.flag(Flag::Negative));
        assert_eq!(false, cpu.flag(Flag::HalfCarry));
        assert_eq!(true, cpu.flag(Flag::Zero));
        assert_eq!(false, cpu.flag(Flag::Carry));
    }

    #[test]
    fn rrca() {
        let mut cpu = Cpu::default();
        cpu.set_register(Register::A, 0x10);
        assert_eq!(4, cpu.rrca());
        assert_eq!(0x08, cpu.register(Register::A));
        assert_eq!(false, cpu.flag(Flag::Negative));
        assert_eq!(false, cpu.flag(Flag::HalfCarry));
        assert_eq!(false, cpu.flag(Flag::Zero));
        assert_eq!(false, cpu.flag(Flag::Carry));

        let mut cpu = Cpu::default();
        cpu.set_register(Register::A, 0x01);
        assert_eq!(4, cpu.rrca());
        assert_eq!(0x80, cpu.register(Register::A));
        assert_eq!(false, cpu.flag(Flag::Negative));
        assert_eq!(false, cpu.flag(Flag::HalfCarry));
        assert_eq!(false, cpu.flag(Flag::Zero));
        assert_eq!(true, cpu.flag(Flag::Carry));

        let mut cpu = Cpu::default();
        cpu.set_register(Register::A, 0x00);
        assert_eq!(4, cpu.rrca());
        assert_eq!(0x00, cpu.register(Register::A));
        assert_eq!(false, cpu.flag(Flag::Negative));
        assert_eq!(false, cpu.flag(Flag::HalfCarry));
        assert_eq!(false, cpu.flag(Flag::Zero));
        assert_eq!(false, cpu.flag(Flag::Carry));
    }

    #[test]
    fn stop() {
        let mut cpu = Cpu::default();
        let mut mmu = vec!(0x00);
        cpu.stop(&mut mmu);
        assert_eq!(0x01, cpu.pc);
        assert_eq!(true, cpu.stopped);
    }

    #[test]
    fn rl() {
        let mut cpu = Cpu::default();
        cpu.set_register(Register::B, 0x08);
        assert_eq!(8, cpu.rl(Register::B));
        assert_eq!(0x10, cpu.register(Register::B));
        assert_eq!(false, cpu.flag(Flag::Negative));
        assert_eq!(false, cpu.flag(Flag::HalfCarry));
        assert_eq!(false, cpu.flag(Flag::Zero));
        assert_eq!(false, cpu.flag(Flag::Carry));

        let mut cpu = Cpu::default();
        cpu.set_register(Register::C, 0x80);
        assert_eq!(8, cpu.rl(Register::C));
        assert_eq!(0x00, cpu.register(Register::C));
        assert_eq!(false, cpu.flag(Flag::Negative));
        assert_eq!(false, cpu.flag(Flag::HalfCarry));
        assert_eq!(true, cpu.flag(Flag::Zero));
        assert_eq!(true, cpu.flag(Flag::Carry));

        let mut cpu = Cpu::default();
        cpu.set_register(Register::D, 0x00);
        cpu.set_flag(Flag::Carry, true);
        assert_eq!(8, cpu.rl(Register::D));
        assert_eq!(0x01, cpu.register(Register::D));
        assert_eq!(false, cpu.flag(Flag::Negative));
        assert_eq!(false, cpu.flag(Flag::HalfCarry));
        assert_eq!(false, cpu.flag(Flag::Zero));
        assert_eq!(false, cpu.flag(Flag::Carry));
    }

    #[test]
    fn rla() {
        let mut cpu = Cpu::default();
        cpu.set_register(Register::A, 0x08);
        assert_eq!(4, cpu.rla());
        assert_eq!(0x10, cpu.register(Register::A));
        assert_eq!(false, cpu.flag(Flag::Negative));
        assert_eq!(false, cpu.flag(Flag::HalfCarry));
        assert_eq!(false, cpu.flag(Flag::Zero));
        assert_eq!(false, cpu.flag(Flag::Carry));

        let mut cpu = Cpu::default();
        cpu.set_register(Register::A, 0x80);
        assert_eq!(4, cpu.rla());
        assert_eq!(0x00, cpu.register(Register::A));
        assert_eq!(false, cpu.flag(Flag::Negative));
        assert_eq!(false, cpu.flag(Flag::HalfCarry));
        assert_eq!(false, cpu.flag(Flag::Zero));
        assert_eq!(true, cpu.flag(Flag::Carry));

        let mut cpu = Cpu::default();
        cpu.set_register(Register::A, 0x00);
        cpu.set_flag(Flag::Carry, true);
        assert_eq!(4, cpu.rla());
        assert_eq!(0x01, cpu.register(Register::A));
        assert_eq!(false, cpu.flag(Flag::Negative));
        assert_eq!(false, cpu.flag(Flag::HalfCarry));
        assert_eq!(false, cpu.flag(Flag::Zero));
        assert_eq!(false, cpu.flag(Flag::Carry));
    }

    #[test]
    fn rr() {
        let mut cpu = Cpu::default();
        cpu.set_register(Register::B, 0x10);
        assert_eq!(8, cpu.rr(Register::B));
        assert_eq!(0x08, cpu.register(Register::B));
        assert_eq!(false, cpu.flag(Flag::Negative));
        assert_eq!(false, cpu.flag(Flag::HalfCarry));
        assert_eq!(false, cpu.flag(Flag::Zero));
        assert_eq!(false, cpu.flag(Flag::Carry));

        let mut cpu = Cpu::default();
        cpu.set_register(Register::C, 0x01);
        assert_eq!(8, cpu.rr(Register::C));
        assert_eq!(0x00, cpu.register(Register::C));
        assert_eq!(false, cpu.flag(Flag::Negative));
        assert_eq!(false, cpu.flag(Flag::HalfCarry));
        assert_eq!(true, cpu.flag(Flag::Zero));
        assert_eq!(true, cpu.flag(Flag::Carry));

        let mut cpu = Cpu::default();
        cpu.set_register(Register::D, 0x00);
        cpu.set_flag(Flag::Carry, true);
        assert_eq!(8, cpu.rr(Register::D));
        assert_eq!(0x80, cpu.register(Register::D));
        assert_eq!(false, cpu.flag(Flag::Negative));
        assert_eq!(false, cpu.flag(Flag::HalfCarry));
        assert_eq!(false, cpu.flag(Flag::Zero));
        assert_eq!(false, cpu.flag(Flag::Carry));
    }

    #[test]
    fn rra() {
        let mut cpu = Cpu::default();
        cpu.set_register(Register::A, 0x10);
        assert_eq!(4, cpu.rra());
        assert_eq!(0x08, cpu.register(Register::A));
        assert_eq!(false, cpu.flag(Flag::Negative));
        assert_eq!(false, cpu.flag(Flag::HalfCarry));
        assert_eq!(false, cpu.flag(Flag::Zero));
        assert_eq!(false, cpu.flag(Flag::Carry));

        let mut cpu = Cpu::default();
        cpu.set_register(Register::A, 0x01);
        assert_eq!(4, cpu.rra());
        assert_eq!(0x00, cpu.register(Register::A));
        assert_eq!(false, cpu.flag(Flag::Negative));
        assert_eq!(false, cpu.flag(Flag::HalfCarry));
        assert_eq!(false, cpu.flag(Flag::Zero));
        assert_eq!(true, cpu.flag(Flag::Carry));

        let mut cpu = Cpu::default();
        cpu.set_register(Register::A, 0x00);
        cpu.set_flag(Flag::Carry, true);
        assert_eq!(4, cpu.rra());
        assert_eq!(0x80, cpu.register(Register::A));
        assert_eq!(false, cpu.flag(Flag::Negative));
        assert_eq!(false, cpu.flag(Flag::HalfCarry));
        assert_eq!(false, cpu.flag(Flag::Zero));
        assert_eq!(false, cpu.flag(Flag::Carry));
    }

    #[test]
    fn jr() {
        let mut cpu = Cpu::default();
        let mut mmu = vec!(0x42);
        assert_eq!(12, cpu.jr(&mut mmu));
        assert_eq!(0x0042, cpu.wide_register(WideRegister::PC));

        let mut cpu = Cpu::default();
        let mut mmu = vec!(0x00, -0x01i8 as u8);
        cpu.set_wide_register(WideRegister::PC, 0x0001);
        assert_eq!(12, cpu.jr(&mut mmu));
        assert_eq!(0x0000, cpu.wide_register(WideRegister::PC));

        let mut cpu = Cpu::default();
        let mut mmu = vec!(-0x01i8 as u8);
        assert_eq!(12, cpu.jr(&mut mmu));
        assert_eq!(0xFFFF, cpu.wide_register(WideRegister::PC));
    }

    #[test]
    fn jr_condition() {
        let mut cpu = Cpu::default();
        let mut mmu = vec!(0x42);
        assert_eq!(8, cpu.jr_condition(Condition::Zero, &mut mmu));
        assert_eq!(0x0001, cpu.wide_register(WideRegister::PC));

        let mut cpu = Cpu::default();
        let mut mmu = vec!(0x42);
        cpu.set_flag(Flag::Zero, true);
        assert_eq!(12, cpu.jr_condition(Condition::Zero, &mut mmu));
        assert_eq!(0x0042, cpu.wide_register(WideRegister::PC));

        let mut cpu = Cpu::default();
        let mut mmu = vec!(0x42);
        assert_eq!(12, cpu.jr_condition(Condition::NotZero, &mut mmu));
        assert_eq!(0x0042, cpu.wide_register(WideRegister::PC));

        let mut cpu = Cpu::default();
        let mut mmu = vec!(0x42);
        cpu.set_flag(Flag::Zero, true);
        assert_eq!(8, cpu.jr_condition(Condition::NotZero, &mut mmu));
        assert_eq!(0x0001, cpu.wide_register(WideRegister::PC));

        let mut cpu = Cpu::default();
        let mut mmu = vec!(0x42);
        assert_eq!(8, cpu.jr_condition(Condition::Carry, &mut mmu));
        assert_eq!(0x0001, cpu.wide_register(WideRegister::PC));

        let mut cpu = Cpu::default();
        let mut mmu = vec!(0x42);
        cpu.set_flag(Flag::Carry, true);
        assert_eq!(12, cpu.jr_condition(Condition::Carry, &mut mmu));
        assert_eq!(0x0042, cpu.wide_register(WideRegister::PC));

        let mut cpu = Cpu::default();
        let mut mmu = vec!(0x42);
        assert_eq!(12, cpu.jr_condition(Condition::NotCarry, &mut mmu));
        assert_eq!(0x0042, cpu.wide_register(WideRegister::PC));

        let mut cpu = Cpu::default();
        let mut mmu = vec!(0x42);
        cpu.set_flag(Flag::Carry, true);
        assert_eq!(8, cpu.jr_condition(Condition::NotCarry, &mut mmu));
        assert_eq!(0x0001, cpu.wide_register(WideRegister::PC));
    }

    #[test]
    fn write_a_hli() {

    }

    #[test]
    fn read_a_hli() {

    }

    #[test]
    fn cpl() {

    }

    #[test]
    fn write_a_hld() {

    }

    #[test]
    fn read_a_hld() {

    }

    #[test]
    fn inc_mem() {

    }

    #[test]
    fn dec_mem() {

    }

    #[test]
    fn write_mem_immediate() {

    }
}