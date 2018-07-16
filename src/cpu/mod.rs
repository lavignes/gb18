#[cfg(test)]
mod tests;

use std::{mem};
use mmu::{Mmu, Port};

#[derive(Default)]
pub struct Cpu {
    pc: u16,
    sp: u16,
    af: u16,
    bc: u16,
    de: u16,
    hl: u16,

    interrupts_enabled: bool,

    stopped: bool,

    halted: bool,
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
    fn copy_register(&mut self, dest: Register, src: Register) -> usize {
        let value = self.register(src);
        self.set_register(dest, value);
        4
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
    fn nop(&mut self) -> usize {
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
    fn write_register(&self, address: WideRegister, reg: Register, mmu: &mut impl Mmu) -> usize {
        mmu.write(self.wide_register(address), self.register(reg));
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

    fn rlc_value(&mut self, value: u8) -> u8 {
        self.set_flag(Flag::Negative, false);
        self.set_flag(Flag::HalfCarry, false);
        let carry = (value & 0x80) >> 7;
        let value = (value << 1) | carry;
        self.set_flag(Flag::Carry, carry != 0);
        self.set_flag(Flag::Zero, value == 0x00);
        value
    }

    #[inline]
    fn rlc(&mut self, reg: Register) -> usize {
        let mut value = self.register(reg);
        value = self.rlc_value(value);
        self.set_register(reg, value);
        8
    }

    #[inline]
    fn rlc_mem(&mut self, mmu: &mut impl Mmu) -> usize {
        let address = self.wide_register(WideRegister::HL);
        let mut value = mmu.read(address);
        value = self.rlc_value(value);
        mmu.write(address, value);
        16
    }

    #[inline]
    fn rl_value(&mut self, value: u8) -> u8 {
        self.set_flag(Flag::Negative, false);
        self.set_flag(Flag::HalfCarry, false);
        let carry = (value & 0x80) >> 7;
        let value = (value << 1) | if self.flag(Flag::Carry) { 0x01 } else { 0x00 };
        self.set_flag(Flag::Carry, carry != 0);
        self.set_flag(Flag::Zero, value == 0x00);
        value
    }

    #[inline]
    fn rl(&mut self, reg: Register) -> usize {
        let mut value = self.register(reg);
        value = self.rl_value(value);
        self.set_register(reg, value);
        8
    }

    #[inline]
    fn rl_mem(&mut self, mmu: &mut impl Mmu) -> usize {
        let address = self.wide_register(WideRegister::HL);
        let mut value = mmu.read(address);
        value = self.rl_value(value);
        mmu.write(address, value);
        16
    }

    #[inline]
    fn rrc_value(&mut self, value: u8) -> u8 {
        self.set_flag(Flag::Negative, false);
        self.set_flag(Flag::HalfCarry, false);
        let carry = (value & 0x01) << 7;
        let value = (value >> 1) | carry;
        self.set_flag(Flag::Carry, carry != 0);
        self.set_flag(Flag::Zero, value == 0x00);
        value
    }

    #[inline]
    fn rrc(&mut self, reg: Register) -> usize {
        let mut value = self.register(reg);
        value = self.rrc_value(value);
        self.set_register(reg, value);
        8
    }

    #[inline]
    fn rrc_mem(&mut self, mmu: &mut impl Mmu) -> usize {
        let address = self.wide_register(WideRegister::HL);
        let mut value = mmu.read(address);
        value = self.rrc_value(value);
        mmu.write(address, value);
        16
    }

    #[inline]
    fn rr_value(&mut self, value: u8) -> u8 {
        self.set_flag(Flag::Negative, false);
        self.set_flag(Flag::HalfCarry, false);
        let carry = (value & 0x01) << 7;
        let value = (value >> 1) | if self.flag(Flag::Carry) { 0x80 } else { 0x00 };
        self.set_flag(Flag::Carry, carry != 0);
        self.set_flag(Flag::Zero, value == 0x00);
        value
    }

    #[inline]
    fn rr(&mut self, reg: Register) -> usize {
        let mut value = self.register(reg);
        value = self.rr_value(value);
        self.set_register(reg, value);
        8
    }

    #[inline]
    fn rr_mem(&mut self, mmu: &mut impl Mmu) -> usize {
        let address = self.wide_register(WideRegister::HL);
        let mut value = mmu.read(address);
        value = self.rr_value(value);
        mmu.write(address, value);
        16
    }

    #[inline]
    fn rlca(&mut self) -> usize {
        let mut value = self.register(Register::A);
        value = self.rlc_value(value);
        self.set_register(Register::A, value);
        self.set_flag(Flag::Zero, false);
        4
    }

    #[inline]
    fn rla(&mut self) -> usize {
        let mut value = self.register(Register::A);
        value = self.rl_value(value);
        self.set_register(Register::A, value);
        self.set_flag(Flag::Zero, false);
        4
    }

    #[inline]
    fn rrca(&mut self) -> usize {
        let mut value = self.register(Register::A);
        value = self.rrc_value(value);
        self.set_register(Register::A, value);
        self.set_flag(Flag::Zero, false);
        4
    }

    #[inline]
    fn rra(&mut self) -> usize {
        let mut value = self.register(Register::A);
        value = self.rr_value(value);
        self.set_register(Register::A, value);
        self.set_flag(Flag::Zero, false);
        4
    }

    #[inline]
    fn sla_value(&mut self, value: u8) -> u8 {
        self.set_flag(Flag::Negative, false);
        self.set_flag(Flag::HalfCarry, false);
        let carry = value & 0x80;
        let value = value << 1;
        self.set_flag(Flag::Carry, carry != 0);
        self.set_flag(Flag::Zero, value == 0x00);
        value
    }

    #[inline]
    fn sla(&mut self, reg: Register) -> usize {
        let mut value = self.register(reg);
        value = self.sla_value(value);
        self.set_register(reg, value);
        8
    }

    #[inline]
    fn sla_mem(&mut self, mmu: &mut impl Mmu) -> usize {
        let address = self.wide_register(WideRegister::HL);
        let mut value = mmu.read(address);
        value = self.sla_value(value);
        mmu.write(address, value);
        16
    }

    #[inline]
    fn sra_value(&mut self, value: u8) -> u8 {
        self.set_flag(Flag::Negative, false);
        self.set_flag(Flag::HalfCarry, false);
        let value = (value as i8) >> 1;
        self.set_flag(Flag::Carry, false);
        self.set_flag(Flag::Zero, value == 0x00);
        value as u8
    }

    #[inline]
    fn sra(&mut self, reg: Register) -> usize {
        let mut value = self.register(reg);
        value = self.sra_value(value);
        self.set_register(reg, value);
        8
    }

    #[inline]
    fn sra_mem(&mut self, mmu: &mut impl Mmu) -> usize {
        let address = self.wide_register(WideRegister::HL);
        let mut value = mmu.read(address);
        value = self.sra_value(value);
        mmu.write(address, value);
        16
    }

    #[inline]
    fn srl_value(&mut self, value: u8) -> u8 {
        self.set_flag(Flag::Negative, false);
        self.set_flag(Flag::HalfCarry, false);
        let value = value >> 1;
        self.set_flag(Flag::Carry, false);
        self.set_flag(Flag::Zero, value == 0x00);
        value
    }

    #[inline]
    fn srl(&mut self, reg: Register) -> usize {
        let mut value = self.register(reg);
        value = self.srl_value(value);
        self.set_register(reg, value);
        8
    }

    #[inline]
    fn srl_mem(&mut self, mmu: &mut impl Mmu) -> usize {
        let address = self.wide_register(WideRegister::HL);
        let mut value = mmu.read(address);
        value = self.srl_value(value);
        mmu.write(address, value);
        16
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
    fn read_register(&mut self, address: WideRegister, reg: Register, mmu: &impl Mmu) -> usize {
        let value = mmu.read(self.wide_register(address));
        self.set_register(reg, value);
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
    fn pop_value(&mut self, mmu: &impl Mmu) -> u8 {
        let value = mmu.read(self.sp);
        self.sp = self.sp.wrapping_add(1);
        value
    }

    #[inline]
    fn pop_wide_value(&mut self, mmu: &impl Mmu) -> u16 {
        (self.pop_value(mmu) as u16) | ((self.pop_value(mmu) as u16) << 8)
    }

    #[inline]
    fn push_value(&mut self, value: u8, mmu: &mut impl Mmu) {
        mmu.write(self.sp, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    #[inline]
    fn push_wide_value(&mut self, value: u16, mmu: &mut impl Mmu) {
        self.sp = self.sp.wrapping_sub(2);
        self.write_wide(self.sp, value, mmu);
    }

    #[inline]
    fn ret(&mut self, mmu: &impl Mmu) -> usize {
        self.pc = self.pop_wide_value(mmu);
        16
    }

    #[inline]
    fn ret_condition(&mut self, condition: Condition, mmu: &impl Mmu) -> usize {
        let met = match condition {
            Condition::Zero => self.flag(Flag::Zero),
            Condition::NotZero => !self.flag(Flag::Zero),
            Condition::Carry => self.flag(Flag::Carry),
            Condition::NotCarry => !self.flag(Flag::Carry),
        };
        if met {
            self.pc = self.pop_wide_value(mmu);
            20
        } else {
            8
        }
    }

    #[inline]
    fn daa(&mut self) -> usize {
        unimplemented!();
        4
    }

    #[inline]
    fn scf(&mut self) -> usize {
        self.set_flag(Flag::HalfCarry, false);
        self.set_flag(Flag::Negative, false);
        self.set_flag(Flag::Carry, true);
        4
    }

    #[inline]
    fn ccf(&mut self) -> usize {
        self.set_flag(Flag::HalfCarry, false);
        self.set_flag(Flag::Negative, false);
        let carry = self.flag(Flag::Carry);
        self.set_flag(Flag::Carry, !carry);
        4
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

    #[inline]
    fn write_register_immediate(&mut self, reg: Register, mmu: &mut impl Mmu) -> usize {
        let address = self.read_wide(mmu);
        let value = self.register(reg);
        mmu.write(address, value);
        16
    }

    #[inline]
    fn read_register_immediate(&mut self, reg: Register, mmu: &mut impl Mmu) -> usize {
        let address = self.read_wide(mmu);
        let value = mmu.read(address);
        self.set_register(reg, value);
        16
    }

    #[inline]
    fn halt(&mut self) -> usize {
        self.halted = true;
        4
    }

    #[inline]
    fn add_value(&mut self, value: u8, carry: bool) {
        let a = self.register(Register::A) as u16;
        let value = value as u16;
        let carry = if carry { 1u16 } else { 0u16 };
        let overflow = a + value + carry;
        self.set_flag(Flag::HalfCarry, ((a & 0x000F) + (value & 0x000F) + carry) > 0x000F);
        self.set_flag(Flag::Carry, overflow > 0x00FF);
        self.set_flag(Flag::Negative, false);
        let result = (overflow & 0x00FF) as u8;
        self.set_register(Register::A, result);
        self.set_flag(Flag::Zero, result == 0x00);
    }

    #[inline]
    fn add(&mut self, reg: Register) -> usize {
        let value = self.register(reg);
        self.add_value(value, false);
        4
    }

    #[inline]
    fn add_mem(&mut self, mmu: &impl Mmu) -> usize {
        let address = self.wide_register(WideRegister::HL);
        let value = mmu.read(address);
        self.add_value(value, false);
        8
    }

    #[inline]
    fn add_carry(&mut self, reg: Register) -> usize {
        let value = self.register(reg);
        let carry = self.flag(Flag::Carry);
        self.add_value(value, carry);
        4
    }

    #[inline]
    fn add_carry_mem(&mut self, mmu: &impl Mmu) -> usize {
        let address = self.wide_register(WideRegister::HL);
        let value = mmu.read(address);
        let carry = self.flag(Flag::Carry);
        self.add_value(value, carry);
        8
    }

    #[inline]
    fn sub_value(&mut self, value: u8, carry: bool) {
        let a = self.register(Register::A) as i16;
        let value = value as i16;
        let carry = if carry { 1i16 } else { 0i16 };
        let overflow = a - value - carry;
        self.set_flag(Flag::HalfCarry, ((a & 0x000F) - (value & 0x000F) - carry) < 0x0000);
        self.set_flag(Flag::Carry, overflow < 0x0000);
        self.set_flag(Flag::Negative, true);
        let result = (overflow & 0x00FF) as u8;
        self.set_register(Register::A, result);
        self.set_flag(Flag::Zero, result == 0x00);
    }

    #[inline]
    fn sub(&mut self, reg: Register) -> usize {
        let value = self.register(reg);
        self.sub_value(value, false);
        4
    }

    #[inline]
    fn sub_mem(&mut self, mmu: &impl Mmu) -> usize {
        let address = self.wide_register(WideRegister::HL);
        let value = mmu.read(address);
        self.sub_value(value, false);
        8
    }

    #[inline]
    fn sub_carry(&mut self, reg: Register) -> usize {
        let value = self.register(reg);
        let carry = self.flag(Flag::Carry);
        self.sub_value(value, carry);
        4
    }

    #[inline]
    fn sub_carry_mem(&mut self, mmu: &impl Mmu) -> usize {
        let address = self.wide_register(WideRegister::HL);
        let value = mmu.read(address);
        let carry = self.flag(Flag::Carry);
        self.sub_value(value, carry);
        8
    }

    #[inline]
    fn and_value(&mut self, value: u8) {
        let mut a = self.register(Register::A);
        a &= value;
        self.set_register(Register::A, a);
        self.set_flag(Flag::Zero, a == 0x00);
        self.set_flag(Flag::Carry, false);
        self.set_flag(Flag::HalfCarry, true);
        self.set_flag(Flag::Carry, false);
    }

    #[inline]
    fn and(&mut self, reg: Register) -> usize {
        let value = self.register(reg);
        self.and_value(value);
        4
    }

    #[inline]
    fn and_mem(&mut self, mmu: &impl Mmu) -> usize {
        let address = self.wide_register(WideRegister::HL);
        let value = mmu.read(address);
        self.and_value(value);
        8
    }

    #[inline]
    fn and_immediate(&mut self, mmu: &impl Mmu) -> usize {
        let value = self.read(mmu);
        self.and_value(value);
        8
    }

    #[inline]
    fn xor_value(&mut self, value: u8) {
        let mut a = self.register(Register::A);
        a ^= value;
        self.set_register(Register::A, a);
        self.set_flag(Flag::Zero, a == 0x00);
        self.set_flag(Flag::Carry, false);
        self.set_flag(Flag::HalfCarry, false);
        self.set_flag(Flag::Carry, false);
    }

    #[inline]
    fn xor(&mut self, reg: Register) -> usize {
        let value = self.register(reg);
        self.xor_value(value);
        4
    }

    #[inline]
    fn xor_mem(&mut self, mmu: &impl Mmu) -> usize {
        let address = self.wide_register(WideRegister::HL);
        let value = mmu.read(address);
        self.xor_value(value);
        8
    }

    #[inline]
    fn xor_immediate(&mut self, mmu: &impl Mmu) -> usize {
        let value = self.read(mmu);
        self.xor_value(value);
        8
    }

    #[inline]
    fn or_value(&mut self, value: u8) {
        let mut a = self.register(Register::A);
        a |= value;
        self.set_register(Register::A, a);
        self.set_flag(Flag::Zero, a == 0x00);
        self.set_flag(Flag::Carry, false);
        self.set_flag(Flag::HalfCarry, false);
        self.set_flag(Flag::Carry, false);
    }

    #[inline]
    fn or(&mut self, reg: Register) -> usize {
        let value = self.register(reg);
        self.or_value(value);
        4
    }

    #[inline]
    fn or_mem(&mut self, mmu: &impl Mmu) -> usize {
        let address = self.wide_register(WideRegister::HL);
        let value = mmu.read(address);
        self.or_value(value);
        8
    }

    #[inline]
    fn or_immediate(&mut self, mmu: &impl Mmu) -> usize {
        let value = self.read(mmu);
        self.or_value(value);
        8
    }

    #[inline]
    fn cp_value(&mut self, value: u8, carry: bool) {
        let a = self.register(Register::A) as i16;
        let value = value as i16;
        let carry = if carry { 1i16 } else { 0i16 };
        let overflow = a - value - carry;
        self.set_flag(Flag::HalfCarry, ((a & 0x000F) - (value & 0x000F) - carry) < 0x0000);
        self.set_flag(Flag::Carry, overflow < 0x0000);
        self.set_flag(Flag::Negative, true);
        let result = (overflow & 0x00FF) as u8;
        self.set_flag(Flag::Zero, result == 0x00);
    }

    #[inline]
    fn cp(&mut self, reg: Register) -> usize {
        let value = self.register(reg);
        self.cp_value(value, false);
        4
    }

    #[inline]
    fn cp_mem(&mut self, mmu: &impl Mmu) -> usize {
        let address = self.wide_register(WideRegister::HL);
        let value = mmu.read(address);
        self.cp_value(value, false);
        8
    }

    #[inline]
    fn cp_immediate(&mut self, mmu: &impl Mmu) -> usize {
        let value = self.read(mmu);
        self.cp_value(value, false);
        8
    }

    #[inline]
    fn pop_wide(&mut self, reg: WideRegister, mmu: &impl Mmu) -> usize {
        let value = self.pop_wide_value(mmu);
        self.set_wide_register(reg, value);
        12
    }

    #[inline]
    fn push_wide(&mut self, reg: WideRegister, mmu: &mut impl Mmu) -> usize {
        let value = self.wide_register(reg);
        self.push_wide_value(value, mmu);
        16
    }

    #[inline]
    fn jmp(&mut self, mmu: &impl Mmu) -> usize {
        self.pc = self.read_wide(mmu);
        16
    }

    #[inline]
    fn jmp_condition(&mut self, condition: Condition, mmu: &impl Mmu) -> usize {
        let address = self.read_wide(mmu);
        let met = match condition {
            Condition::Zero => self.flag(Flag::Zero),
            Condition::NotZero => !self.flag(Flag::Zero),
            Condition::Carry => self.flag(Flag::Carry),
            Condition::NotCarry => !self.flag(Flag::Carry),
        };
        if met {
            self.pc = address;
            16
        } else {
            12
        }
    }

    #[inline]
    fn call(&mut self, mmu: &mut impl Mmu) -> usize {
        let address = self.read_wide(mmu);
        self.push_wide(WideRegister::PC, mmu);
        self.pc = address;
        24
    }

    #[inline]
    fn call_condition(&mut self, condition: Condition, mmu: &mut impl Mmu) -> usize {
        let address = self.read_wide(mmu);
        let met = match condition {
            Condition::Zero => self.flag(Flag::Zero),
            Condition::NotZero => !self.flag(Flag::Zero),
            Condition::Carry => self.flag(Flag::Carry),
            Condition::NotCarry => !self.flag(Flag::Carry),
        };
        if met {
            self.push_wide(WideRegister::PC, mmu);
            self.pc = address;
            24
        } else {
            12
        }
    }

    #[inline]
    fn add_immediate(&mut self, mmu: &impl Mmu) -> usize {
        let value = self.read(mmu);
        self.add_value(value, false);
        8
    }

    #[inline]
    fn add_carry_immediate(&mut self, mmu: &impl Mmu) -> usize {
        let value = self.read(mmu);
        let carry = self.flag(Flag::Carry);
        self.add_value(value, carry);
        8
    }

    #[inline]
    fn sub_immediate(&mut self, mmu: &impl Mmu) -> usize {
        let value = self.read(mmu);
        self.sub_value(value, false);
        8
    }

    #[inline]
    fn sub_carry_immediate(&mut self, mmu: &impl Mmu) -> usize {
        let value = self.read(mmu);
        let carry = self.flag(Flag::Carry);
        self.sub_value(value, carry);
        8
    }

    #[inline]
    fn rst(&mut self, address: u16, mmu: &mut impl Mmu) -> usize {
        self.push_wide(WideRegister::PC, mmu);
        self.pc = address;
        16
    }

    #[inline]
    fn reti(&mut self, mmu: &impl Mmu) -> usize {
        self.interrupts_enabled = true;
        self.ret(mmu)
    }

    #[inline]
    fn write_high_offset(&mut self, offset: u8, value: u8, mmu: &mut impl Mmu) {
        let address = 0xFF00 + (offset as u16);
        mmu.write(address, value);
    }

    #[inline]
    fn write_high_immediate(&mut self, reg: Register, mmu: &mut impl Mmu) -> usize {
        let offset = self.read(mmu);
        let value = self.register(reg);
        self.write_high_offset(offset, value, mmu);
        12
    }

    #[inline]
    fn write_high_register(&mut self, off: Register, reg: Register, mmu: &mut impl Mmu) -> usize {
        let offset = self.register(off);
        let value = self.register(reg);
        self.write_high_offset(offset, value, mmu);
        8
    }

    #[inline]
    fn read_high_offset(&mut self, offset: u8, mmu: &mut impl Mmu) -> u8 {
        let address = 0xFF00 + (offset as u16);
        mmu.read(address)
    }

    #[inline]
    fn read_high_immediate(&mut self, reg: Register, mmu: &mut impl Mmu) -> usize {
        let offset = self.read(mmu);
        let value = self.read_high_offset(offset, mmu);
        self.set_register(reg, value);
        12
    }

    #[inline]
    fn read_high_register(&mut self, off: Register, reg: Register, mmu: &mut impl Mmu) -> usize {
        let offset = self.register(off);
        let value = self.read_high_offset(offset, mmu);
        self.set_register(reg, value);
        8
    }

    #[inline]
    fn add_sp(&mut self, mmu: &impl Mmu) -> usize {
        let sp = self.sp as i32;
        let value = (self.read(mmu) as i8) as i32;
        let overflow = sp + value;
        self.set_flag(Flag::Negative, false);
        self.set_flag(Flag::Zero, false);
        self.set_flag(Flag::HalfCarry, (overflow & 0x0FFF) < (sp & 0x0FFFF));
        self.sp = (overflow & 0xFFFF) as u16;
        self.set_flag(Flag::Carry, overflow > 0xFFFF);
        16
    }

    #[inline]
    fn jmp_hl(&mut self, mmu: &impl Mmu) -> usize {
        unimplemented!();
        4
    }

    #[inline]
    fn di(&mut self) -> usize {
        self.interrupts_enabled = false;
        4
    }

    #[inline]
    fn ei(&mut self) -> usize {
        self.interrupts_enabled = true;
        4
    }

    #[inline]
    fn copy_wide_register(&mut self, dest: WideRegister, src: WideRegister) -> usize {
        let value = self.wide_register(src);
        self.set_wide_register(dest, value);
        8
    }

    #[inline]
    fn swap_value(&mut self, value: u8) -> u8 {
        self.set_flag(Flag::Carry, false);
        self.set_flag(Flag::HalfCarry, false);
        self.set_flag(Flag::Negative, false);
        let value = ((value << 4) & 0xF0) | ((value >> 4) & 0x0F);
        self.set_flag(Flag::Zero, value == 0x00);
        value
    }

    #[inline]
    fn swap(&mut self, reg: Register) -> usize {
        let mut value = self.register(reg);
        value = self.swap_value(value);
        self.set_register(reg, value);
        8
    }

    #[inline]
    fn swap_mem(&mut self, mmu: &mut impl Mmu) -> usize {
        let address = self.wide_register(WideRegister::HL);
        let mut value = mmu.read(address);
        value = self.swap_value(value);
        mmu.write(address, value);
        16
    }

    #[inline]
    fn bit_value(&mut self, bit: u8, value: u8) {
        self.set_flag(Flag::Negative, false);
        self.set_flag(Flag::HalfCarry, true);
        self.set_flag(Flag::Zero, (value & (1 << bit)) == 0x00);
    }

    #[inline]
    fn bit(&mut self, bit: u8, reg: Register) -> usize {
        let value = self.register(reg);
        self.bit_value(bit, value);
        8
    }

    #[inline]
    fn bit_mem(&mut self, bit: u8, mmu: &impl Mmu) -> usize {
        let address = self.wide_register(WideRegister::HL);
        let value = mmu.read(address);
        self.bit_value(bit, value);
        16
    }

    #[inline]
    fn reset_bit_value(&mut self, bit: u8, value: u8) -> u8 {
        value & !(1 << bit)
    }

    #[inline]
    fn reset_bit(&mut self, bit: u8, reg: Register) -> usize {
        let mut value = self.register(reg);
        value = self.reset_bit_value(bit, value);
        self.set_register(reg, value);
        8
    }

    #[inline]
    fn reset_bit_mem(&mut self, bit: u8, mmu: &mut impl Mmu) -> usize {
        let address = self.wide_register(WideRegister::HL);
        let mut value = mmu.read(address);
        value = self.reset_bit_value(bit, value);
        mmu.write(address, value);
        16
    }

    #[inline]
    fn set_bit_value(&mut self, bit: u8, value: u8) -> u8 {
        value | (1 << bit)
    }

    #[inline]
    fn set_bit(&mut self, bit: u8, reg: Register) -> usize {
        let mut value = self.register(reg);
        value = self.set_bit_value(bit, value);
        self.set_register(reg, value);
        8
    }

    #[inline]
    fn set_bit_mem(&mut self, bit: u8, mmu: &mut impl Mmu) -> usize {
        let address = self.wide_register(WideRegister::HL);
        let mut value = mmu.read(address);
        value = self.set_bit_value(bit, value);
        mmu.write(address, value);
        16
    }

    #[inline]
    fn service_interrupts(&mut self, mmu: &mut impl Mmu) {
        if !self.interrupts_enabled {
            return;
        }
        let if_value = mmu.io_read(Port::IF);
        let flags = mmu.io_read(Port::IE) & if_value;
        if flags == 0x00 {
            return;
        } else if (flags & 0x01) != 0x00 {
            self.rst(0x0040, mmu);
            mmu.io_write(Port::IF, if_value ^ 0x01);
            self.interrupts_enabled = false;
        } else if (flags & 0x02) != 0x00 {
            self.rst(0x0048, mmu);
            mmu.io_write(Port::IF, if_value ^ 0x02);
            self.interrupts_enabled = false;
        } else if (flags & 0x04) != 0x00 {
            self.rst(0x0050, mmu);
            mmu.io_write(Port::IF, if_value ^ 0x04);
            self.interrupts_enabled = false;
        } else if (flags & 0x08) != 0x00 {
            self.rst(0x0058, mmu);
            mmu.io_write(Port::IF, if_value ^ 0x08);
            self.interrupts_enabled = false;
        } else if (flags & 0x10) != 0x00 {
            self.rst(0x0060, mmu);
            mmu.io_write(Port::IF, if_value ^ 0x10);
            self.interrupts_enabled = false;
        }
    }

    pub fn cycle(&mut self, mmu: &mut impl Mmu) -> usize {
        if self.halted {
            // TODO: exit halt state
            return 4;
        }
        self.service_interrupts(mmu);
        let opcode = self.read(mmu);
        match opcode {
            0x00 => { self.nop() },
            0x01 => { self.read_wide_immediate(WideRegister::BC, mmu) }
            0x02 => { self.write_register(WideRegister::BC, Register::A, mmu) }
            0x03 => { self.inc_wide(WideRegister::BC) }
            0x04 => { self.inc(Register::B) }
            0x05 => { self.dec(Register::B) }
            0x06 => { self.read_immediate(Register::B, mmu) }
            0x07 => { self.rlca() }
            0x08 => { self.write_stack_immediate(mmu) }
            0x09 => { self.add_wide(WideRegister::BC) }
            0x0A => { self.read_register(WideRegister::BC, Register::A, mmu) }
            0x0B => { self.dec_wide(WideRegister::BC) }
            0x0C => { self.inc(Register::C) }
            0x0D => { self.dec(Register::C) }
            0x0E => { self.read_immediate(Register::C, mmu) }
            0x0F => { self.rrca() }

            0x10 => { self.stop(mmu) }
            0x11 => { self.read_wide_immediate(WideRegister::DE, mmu) }
            0x12 => { self.write_register(WideRegister::DE, Register::A, mmu) }
            0x13 => { self.inc_wide(WideRegister::DE) }
            0x14 => { self.inc(Register::D) }
            0x15 => { self.dec(Register::D) }
            0x16 => { self.read_immediate(Register::D, mmu) }
            0x17 => { self.rla() }
            0x18 => { self.jr(mmu) }
            0x19 => { self.add_wide(WideRegister::DE) }
            0x1A => { self.read_register(WideRegister::DE, Register::A, mmu) }
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
            0x27 => { self.daa() }
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
            0x37 => { self.scf() }
            0x38 => { self.jr_condition(Condition::Carry, mmu) }
            0x39 => { self.add_wide(WideRegister::SP) }
            0x3A => { self.read_a_hld(mmu) }
            0x3B => { self.dec_wide(WideRegister::SP) }
            0x3C => { self.inc(Register::A) }
            0x3D => { self.dec(Register::A) }
            0x3E => { self.read_immediate(Register::A, mmu) }
            0x3F => { self.ccf() }

            0x40 => { self.copy_register(Register::B, Register::B) }
            0x41 => { self.copy_register(Register::B, Register::C) }
            0x42 => { self.copy_register(Register::B, Register::D) }
            0x43 => { self.copy_register(Register::B, Register::E) }
            0x44 => { self.copy_register(Register::B, Register::H) }
            0x45 => { self.copy_register(Register::B, Register::L) }
            0x46 => { self.read_register(WideRegister::HL, Register::B, mmu) }
            0x47 => { self.copy_register(Register::B, Register::A) }
            0x48 => { self.copy_register(Register::C, Register::B) }
            0x49 => { self.copy_register(Register::C, Register::C) }
            0x4A => { self.copy_register(Register::C, Register::D) }
            0x4B => { self.copy_register(Register::C, Register::E) }
            0x4C => { self.copy_register(Register::C, Register::H) }
            0x4D => { self.copy_register(Register::C, Register::L) }
            0x4E => { self.read_register(WideRegister::HL, Register::C, mmu) }
            0x4F => { self.copy_register(Register::C, Register::A) }

            0x50 => { self.copy_register(Register::D, Register::B) }
            0x51 => { self.copy_register(Register::D, Register::C) }
            0x52 => { self.copy_register(Register::D, Register::D) }
            0x53 => { self.copy_register(Register::D, Register::E) }
            0x54 => { self.copy_register(Register::D, Register::H) }
            0x55 => { self.copy_register(Register::D, Register::L) }
            0x56 => { self.read_register(WideRegister::HL, Register::D, mmu) }
            0x57 => { self.copy_register(Register::D, Register::A) }
            0x58 => { self.copy_register(Register::E, Register::B) }
            0x59 => { self.copy_register(Register::E, Register::C) }
            0x5A => { self.copy_register(Register::E, Register::D) }
            0x5B => { self.copy_register(Register::E, Register::E) }
            0x5C => { self.copy_register(Register::E, Register::H) }
            0x5D => { self.copy_register(Register::E, Register::L) }
            0x5E => { self.read_register(WideRegister::HL, Register::E, mmu) }
            0x5F => { self.copy_register(Register::E, Register::A) }

            0x60 => { self.copy_register(Register::H, Register::B) }
            0x61 => { self.copy_register(Register::H, Register::C) }
            0x62 => { self.copy_register(Register::H, Register::D) }
            0x63 => { self.copy_register(Register::H, Register::E) }
            0x64 => { self.copy_register(Register::H, Register::H) }
            0x65 => { self.copy_register(Register::H, Register::L) }
            0x66 => { self.read_register(WideRegister::HL, Register::H, mmu) }
            0x67 => { self.copy_register(Register::H, Register::A) }
            0x68 => { self.copy_register(Register::L, Register::B) }
            0x69 => { self.copy_register(Register::L, Register::C) }
            0x6A => { self.copy_register(Register::L, Register::D) }
            0x6B => { self.copy_register(Register::L, Register::E) }
            0x6C => { self.copy_register(Register::L, Register::H) }
            0x6D => { self.copy_register(Register::L, Register::L) }
            0x6E => { self.read_register(WideRegister::HL, Register::L, mmu) }
            0x6F => { self.copy_register(Register::L, Register::A) }

            0x70 => { self.write_register(WideRegister::HL, Register::B, mmu) }
            0x71 => { self.write_register(WideRegister::HL, Register::C, mmu) }
            0x72 => { self.write_register(WideRegister::HL, Register::D, mmu) }
            0x73 => { self.write_register(WideRegister::HL, Register::E, mmu) }
            0x74 => { self.write_register(WideRegister::HL, Register::H, mmu) }
            0x75 => { self.write_register(WideRegister::HL, Register::L, mmu) }
            0x76 => { self.halt() }
            0x77 => { self.write_register(WideRegister::HL, Register::A, mmu) }
            0x78 => { self.copy_register(Register::A, Register::B) }
            0x79 => { self.copy_register(Register::A, Register::C) }
            0x7A => { self.copy_register(Register::A, Register::D) }
            0x7B => { self.copy_register(Register::A, Register::E) }
            0x7C => { self.copy_register(Register::A, Register::H) }
            0x7D => { self.copy_register(Register::A, Register::L) }
            0x7E => { self.read_register(WideRegister::HL, Register::A, mmu) }
            0x7F => { self.copy_register(Register::A, Register::A) }

            0x80 => { self.add(Register::B) }
            0x81 => { self.add(Register::C) }
            0x82 => { self.add(Register::D) }
            0x83 => { self.add(Register::E) }
            0x84 => { self.add(Register::H) }
            0x85 => { self.add(Register::L) }
            0x86 => { self.add_mem(mmu) }
            0x87 => { self.add(Register::A) }
            0x88 => { self.add_carry(Register::B) }
            0x89 => { self.add_carry(Register::C) }
            0x8A => { self.add_carry(Register::D) }
            0x8B => { self.add_carry(Register::E) }
            0x8C => { self.add_carry(Register::H) }
            0x8D => { self.add_carry(Register::L) }
            0x8E => { self.add_carry_mem(mmu) }
            0x8F => { self.add_carry(Register::A) }

            0x90 => { self.sub(Register::B) }
            0x91 => { self.sub(Register::C) }
            0x92 => { self.sub(Register::D) }
            0x93 => { self.sub(Register::E) }
            0x94 => { self.sub(Register::H) }
            0x95 => { self.sub(Register::L) }
            0x96 => { self.sub_mem(mmu) }
            0x97 => { self.sub(Register::A) }
            0x98 => { self.sub_carry(Register::B) }
            0x99 => { self.sub_carry(Register::C) }
            0x9A => { self.sub_carry(Register::D) }
            0x9B => { self.sub_carry(Register::E) }
            0x9C => { self.sub_carry(Register::H) }
            0x9D => { self.sub_carry(Register::L) }
            0x9E => { self.sub_carry_mem(mmu) }
            0x9F => { self.sub_carry(Register::A) }

            0xA0 => { self.and(Register::B) }
            0xA1 => { self.and(Register::C) }
            0xA2 => { self.and(Register::D) }
            0xA3 => { self.and(Register::E) }
            0xA4 => { self.and(Register::H) }
            0xA5 => { self.and(Register::L) }
            0xA6 => { self.and_mem(mmu) }
            0xA7 => { self.and(Register::A) }
            0xA8 => { self.xor(Register::B) }
            0xA9 => { self.xor(Register::C) }
            0xAA => { self.xor(Register::D) }
            0xAB => { self.xor(Register::E) }
            0xAC => { self.xor(Register::H) }
            0xAD => { self.xor(Register::L) }
            0xAE => { self.xor_mem(mmu) }
            0xAF => { self.xor(Register::A) }

            0xB0 => { self.or(Register::B) }
            0xB1 => { self.or(Register::C) }
            0xB2 => { self.or(Register::D) }
            0xB3 => { self.or(Register::E) }
            0xB4 => { self.or(Register::H) }
            0xB5 => { self.or(Register::L) }
            0xB6 => { self.or_mem(mmu) }
            0xB7 => { self.or(Register::A) }
            0xB8 => { self.cp(Register::B) }
            0xB9 => { self.cp(Register::C) }
            0xBA => { self.cp(Register::D) }
            0xBB => { self.cp(Register::E) }
            0xBC => { self.cp(Register::H) }
            0xBD => { self.cp(Register::L) }
            0xBE => { self.cp_mem(mmu) }
            0xBF => { self.cp(Register::A) }

            0xC0 => { self.ret_condition(Condition::NotZero, mmu) }
            0xC1 => { self.pop_wide(WideRegister::BC, mmu) }
            0xC2 => { self.jmp_condition(Condition::NotZero, mmu) }
            0xC3 => { self.jmp(mmu) }
            0xC4 => { self.call_condition(Condition::NotZero, mmu) }
            0xC5 => { self.push_wide(WideRegister::BC, mmu) }
            0xC6 => { self.add_immediate(mmu) }
            0xC7 => { self.rst(0x0000, mmu) }
            0xC8 => { self.ret_condition(Condition::Zero, mmu) }
            0xC9 => { self.ret(mmu) }
            0xCA => { self.jmp_condition(Condition::Zero, mmu) }
            0xCB => { self.cb(mmu) }
            0xCC => { self.call_condition(Condition::Zero, mmu) }
            0xCD => { self.call(mmu) }
            0xCE => { self.add_carry_immediate(mmu) }
            0xCF => { self.rst(0x0008, mmu) }

            0xD0 => { self.ret_condition(Condition::NotCarry, mmu) }
            0xD1 => { self.pop_wide(WideRegister::DE, mmu) }
            0xD2 => { self.jmp_condition(Condition::NotCarry, mmu) }
            0xD3 => { unimplemented!() }
            0xD4 => { self.call_condition(Condition::NotCarry, mmu) }
            0xD5 => { self.push_wide(WideRegister::DE, mmu) }
            0xD6 => { self.sub_immediate(mmu) }
            0xD7 => { self.rst(0x0010, mmu) }
            0xD8 => { self.ret_condition(Condition::Carry, mmu) }
            0xD9 => { self.reti(mmu) }
            0xDA => { self.jmp_condition(Condition::Carry, mmu) }
            0xDB => { unimplemented!() }
            0xDC => { self.call_condition(Condition::Carry, mmu) }
            0xDD => { unimplemented!() }
            0xDE => { self.sub_carry_immediate(mmu) }
            0xDF => { self.rst(0x0018, mmu) }

            0xE0 => { self.write_high_immediate(Register::A, mmu) }
            0xE1 => { self.pop_wide(WideRegister::HL, mmu) }
            0xE2 => { self.write_high_register(Register::C, Register::A, mmu) }
            0xE3 => { unimplemented!() }
            0xE4 => { unimplemented!() }
            0xE5 => { self.push_wide(WideRegister::HL, mmu) }
            0xE6 => { self.and_immediate(mmu) }
            0xE7 => { self.rst(0x0020, mmu) }
            0xE8 => { self.add_sp(mmu) }
            0xE9 => { self.jmp_hl(mmu) }
            0xEA => { self.write_register_immediate(Register::A, mmu) }
            0xEB => { unimplemented!() }
            0xEC => { unimplemented!() }
            0xED => { unimplemented!() }
            0xEE => { self.xor_immediate(mmu) }
            0xEF => { self.rst(0x0028, mmu) }

            0xF0 => { self.read_high_immediate(Register::A, mmu) }
            0xF1 => { self.pop_wide(WideRegister::AF, mmu) }
            0xF2 => { self.read_high_register(Register::C, Register::A, mmu) }
            0xF3 => { self.di() }
            0xF4 => { unimplemented!() }
            0xF5 => { self.push_wide(WideRegister::AF, mmu) }
            0xF6 => { self.or_immediate(mmu) }
            0xF7 => { self.rst(0x0030, mmu)}
            0xF8 => { unimplemented!() }
            0xF9 => { self.copy_wide_register(WideRegister::SP, WideRegister::HL) }
            0xFA => { self.read_register_immediate(Register::A, mmu) }
            0xFB => { self.ei() }
            0xFC => { unimplemented!() }
            0xFD => { unimplemented!() }
            0xFE => { self.cp_immediate(mmu) }
            0xFF => { self.rst(0x0038, mmu) }

            _ => { unreachable!() }
        }
    }

    #[inline]
    fn cb(&mut self, mmu: &mut impl Mmu) -> usize {
        let opcode = self.read(mmu);
        match opcode {
            0x00 => { self.rlc(Register::B) }
            0x01 => { self.rlc(Register::C) }
            0x02 => { self.rlc(Register::D) }
            0x03 => { self.rlc(Register::E) }
            0x04 => { self.rlc(Register::H) }
            0x05 => { self.rlc(Register::L) }
            0x06 => { self.rlc_mem(mmu) }
            0x07 => { self.rlc(Register::A) }
            0x08 => { self.rrc(Register::B) }
            0x09 => { self.rrc(Register::C) }
            0x0A => { self.rrc(Register::D) }
            0x0B => { self.rrc(Register::E) }
            0x0C => { self.rrc(Register::H) }
            0x0D => { self.rrc(Register::L) }
            0x0E => { self.rrc_mem(mmu) }
            0x0F => { self.rrc(Register::A) }

            0x10 => { self.rl(Register::B) }
            0x11 => { self.rl(Register::C) }
            0x12 => { self.rl(Register::D) }
            0x13 => { self.rl(Register::E) }
            0x14 => { self.rl(Register::H) }
            0x15 => { self.rl(Register::L) }
            0x16 => { self.rl_mem(mmu) }
            0x17 => { self.rl(Register::A) }
            0x18 => { self.rr(Register::B) }
            0x19 => { self.rr(Register::C) }
            0x1A => { self.rr(Register::D) }
            0x1B => { self.rr(Register::E) }
            0x1C => { self.rr(Register::H) }
            0x1D => { self.rr(Register::L) }
            0x1E => { self.rr_mem(mmu) }
            0x1F => { self.rr(Register::A) }

            0x20 => { self.sla(Register::B) }
            0x21 => { self.sla(Register::C) }
            0x22 => { self.sla(Register::D) }
            0x23 => { self.sla(Register::E) }
            0x24 => { self.sla(Register::H) }
            0x25 => { self.sla(Register::L) }
            0x26 => { self.sla_mem(mmu) }
            0x27 => { self.sla(Register::A) }
            0x28 => { self.sra(Register::B) }
            0x29 => { self.sra(Register::C) }
            0x2A => { self.sra(Register::D) }
            0x2B => { self.sra(Register::E) }
            0x2C => { self.sra(Register::H) }
            0x2D => { self.sra(Register::L) }
            0x2E => { self.sra_mem(mmu) }
            0x2F => { self.sra(Register::A) }

            0x30 => { self.swap(Register::B) }
            0x31 => { self.swap(Register::C) }
            0x32 => { self.swap(Register::D) }
            0x33 => { self.swap(Register::E) }
            0x34 => { self.swap(Register::H) }
            0x35 => { self.swap(Register::L) }
            0x36 => { self.swap_mem(mmu) }
            0x37 => { self.swap(Register::A) }
            0x38 => { self.srl(Register::B) }
            0x39 => { self.srl(Register::C) }
            0x3A => { self.srl(Register::D) }
            0x3B => { self.srl(Register::E) }
            0x3C => { self.srl(Register::H) }
            0x3D => { self.srl(Register::L) }
            0x3E => { self.srl_mem(mmu) }
            0x3F => { self.srl(Register::A) }

            0x40 => { self.bit(0x00, Register::B) }
            0x41 => { self.bit(0x00, Register::C) }
            0x42 => { self.bit(0x00, Register::D) }
            0x43 => { self.bit(0x00, Register::E) }
            0x44 => { self.bit(0x00, Register::H) }
            0x45 => { self.bit(0x00, Register::L) }
            0x46 => { self.bit_mem(0x00, mmu) }
            0x47 => { self.bit(0x00, Register::A) }
            0x48 => { self.bit(0x01, Register::B) }
            0x49 => { self.bit(0x01, Register::C) }
            0x4A => { self.bit(0x01, Register::D) }
            0x4B => { self.bit(0x01, Register::E) }
            0x4C => { self.bit(0x01, Register::H) }
            0x4D => { self.bit(0x01, Register::L) }
            0x4E => { self.bit_mem(0x01, mmu) }
            0x4F => { self.bit(0x01, Register::A) }

            0x50 => { self.bit(0x02, Register::B) }
            0x51 => { self.bit(0x02, Register::C) }
            0x52 => { self.bit(0x02, Register::D) }
            0x53 => { self.bit(0x02, Register::E) }
            0x54 => { self.bit(0x02, Register::H) }
            0x55 => { self.bit(0x02, Register::L) }
            0x56 => { self.bit_mem(0x02, mmu) }
            0x57 => { self.bit(0x02, Register::A) }
            0x58 => { self.bit(0x03, Register::B) }
            0x59 => { self.bit(0x03, Register::C) }
            0x5A => { self.bit(0x03, Register::D) }
            0x5B => { self.bit(0x03, Register::E) }
            0x5C => { self.bit(0x03, Register::H) }
            0x5D => { self.bit(0x03, Register::L) }
            0x5E => { self.bit_mem(0x03, mmu) }
            0x5F => { self.bit(0x03, Register::A) }

            0x60 => { self.bit(0x04, Register::B) }
            0x61 => { self.bit(0x04, Register::C) }
            0x62 => { self.bit(0x04, Register::D) }
            0x63 => { self.bit(0x04, Register::E) }
            0x64 => { self.bit(0x04, Register::H) }
            0x65 => { self.bit(0x04, Register::L) }
            0x66 => { self.bit_mem(0x04, mmu) }
            0x67 => { self.bit(0x04, Register::A) }
            0x68 => { self.bit(0x05, Register::B) }
            0x69 => { self.bit(0x05, Register::C) }
            0x6A => { self.bit(0x05, Register::D) }
            0x6B => { self.bit(0x05, Register::E) }
            0x6C => { self.bit(0x05, Register::H) }
            0x6D => { self.bit(0x05, Register::L) }
            0x6E => { self.bit_mem(0x05, mmu) }
            0x6F => { self.bit(0x05, Register::A) }

            0x70 => { self.bit(0x06, Register::B) }
            0x71 => { self.bit(0x06, Register::C) }
            0x72 => { self.bit(0x06, Register::D) }
            0x73 => { self.bit(0x06, Register::E) }
            0x74 => { self.bit(0x06, Register::H) }
            0x75 => { self.bit(0x06, Register::L) }
            0x76 => { self.bit_mem(0x06, mmu) }
            0x77 => { self.bit(0x06, Register::A) }
            0x78 => { self.bit(0x07, Register::B) }
            0x79 => { self.bit(0x07, Register::C) }
            0x7A => { self.bit(0x07, Register::D) }
            0x7B => { self.bit(0x07, Register::E) }
            0x7C => { self.bit(0x07, Register::H) }
            0x7D => { self.bit(0x07, Register::L) }
            0x7E => { self.bit_mem(0x07, mmu) }
            0x7F => { self.bit(0x07, Register::A) }

            0x80 => { self.reset_bit(0x00, Register::B) }
            0x81 => { self.reset_bit(0x00, Register::C) }
            0x82 => { self.reset_bit(0x00, Register::D) }
            0x83 => { self.reset_bit(0x00, Register::E) }
            0x84 => { self.reset_bit(0x00, Register::H) }
            0x85 => { self.reset_bit(0x00, Register::L) }
            0x86 => { self.reset_bit_mem(0x00, mmu) }
            0x87 => { self.reset_bit(0x00, Register::A) }
            0x88 => { self.reset_bit(0x01, Register::B) }
            0x89 => { self.reset_bit(0x01, Register::C) }
            0x8A => { self.reset_bit(0x01, Register::D) }
            0x8B => { self.reset_bit(0x01, Register::E) }
            0x8C => { self.reset_bit(0x01, Register::H) }
            0x8D => { self.reset_bit(0x01, Register::L) }
            0x8E => { self.reset_bit_mem(0x01, mmu) }
            0x8F => { self.reset_bit(0x01, Register::A) }

            0x90 => { self.reset_bit(0x02, Register::B) }
            0x91 => { self.reset_bit(0x02, Register::C) }
            0x92 => { self.reset_bit(0x02, Register::D) }
            0x93 => { self.reset_bit(0x02, Register::E) }
            0x94 => { self.reset_bit(0x02, Register::H) }
            0x95 => { self.reset_bit(0x02, Register::L) }
            0x96 => { self.reset_bit_mem(0x02, mmu) }
            0x97 => { self.reset_bit(0x02, Register::A) }
            0x98 => { self.reset_bit(0x03, Register::B) }
            0x99 => { self.reset_bit(0x03, Register::C) }
            0x9A => { self.reset_bit(0x03, Register::D) }
            0x9B => { self.reset_bit(0x03, Register::E) }
            0x9C => { self.reset_bit(0x03, Register::H) }
            0x9D => { self.reset_bit(0x03, Register::L) }
            0x9E => { self.reset_bit_mem(0x03, mmu) }
            0x9F => { self.reset_bit(0x03, Register::A) }

            0xA0 => { self.reset_bit(0x04, Register::B) }
            0xA1 => { self.reset_bit(0x04, Register::C) }
            0xA2 => { self.reset_bit(0x04, Register::D) }
            0xA3 => { self.reset_bit(0x04, Register::E) }
            0xA4 => { self.reset_bit(0x04, Register::H) }
            0xA5 => { self.reset_bit(0x04, Register::L) }
            0xA6 => { self.reset_bit_mem(0x04, mmu) }
            0xA7 => { self.reset_bit(0x04, Register::A) }
            0xA8 => { self.reset_bit(0x05, Register::B) }
            0xA9 => { self.reset_bit(0x05, Register::C) }
            0xAA => { self.reset_bit(0x05, Register::D) }
            0xAB => { self.reset_bit(0x05, Register::E) }
            0xAC => { self.reset_bit(0x05, Register::H) }
            0xAD => { self.reset_bit(0x05, Register::L) }
            0xAE => { self.reset_bit_mem(0x05, mmu) }
            0xAF => { self.reset_bit(0x05, Register::A) }

            0xB0 => { self.reset_bit(0x06, Register::B) }
            0xB1 => { self.reset_bit(0x06, Register::C) }
            0xB2 => { self.reset_bit(0x06, Register::D) }
            0xB3 => { self.reset_bit(0x06, Register::E) }
            0xB4 => { self.reset_bit(0x06, Register::H) }
            0xB5 => { self.reset_bit(0x06, Register::L) }
            0xB6 => { self.reset_bit_mem(0x06, mmu) }
            0xB7 => { self.reset_bit(0x06, Register::A) }
            0xB8 => { self.reset_bit(0x07, Register::B) }
            0xB9 => { self.reset_bit(0x07, Register::C) }
            0xBA => { self.reset_bit(0x07, Register::D) }
            0xBB => { self.reset_bit(0x07, Register::E) }
            0xBC => { self.reset_bit(0x07, Register::H) }
            0xBD => { self.reset_bit(0x07, Register::L) }
            0xBE => { self.reset_bit_mem(0x07, mmu) }
            0xBF => { self.reset_bit(0x07, Register::A) }

            0xC0 => { self.set_bit(0x00, Register::B) }
            0xC1 => { self.set_bit(0x00, Register::C) }
            0xC2 => { self.set_bit(0x00, Register::D) }
            0xC3 => { self.set_bit(0x00, Register::E) }
            0xC4 => { self.set_bit(0x00, Register::H) }
            0xC5 => { self.set_bit(0x00, Register::L) }
            0xC6 => { self.set_bit_mem(0x00, mmu) }
            0xC7 => { self.set_bit(0x00, Register::A) }
            0xC8 => { self.set_bit(0x01, Register::B) }
            0xC9 => { self.set_bit(0x01, Register::C) }
            0xCA => { self.set_bit(0x01, Register::D) }
            0xCB => { self.set_bit(0x01, Register::E) }
            0xCC => { self.set_bit(0x01, Register::H) }
            0xCD => { self.set_bit(0x01, Register::L) }
            0xCE => { self.set_bit_mem(0x01, mmu) }
            0xCF => { self.set_bit(0x01, Register::A) }

            0xD0 => { self.set_bit(0x02, Register::B) }
            0xD1 => { self.set_bit(0x02, Register::C) }
            0xD2 => { self.set_bit(0x02, Register::D) }
            0xD3 => { self.set_bit(0x02, Register::E) }
            0xD4 => { self.set_bit(0x02, Register::H) }
            0xD5 => { self.set_bit(0x02, Register::L) }
            0xD6 => { self.set_bit_mem(0x02, mmu) }
            0xD7 => { self.set_bit(0x02, Register::A) }
            0xD8 => { self.set_bit(0x03, Register::B) }
            0xD9 => { self.set_bit(0x03, Register::C) }
            0xDA => { self.set_bit(0x03, Register::D) }
            0xDB => { self.set_bit(0x03, Register::E) }
            0xDC => { self.set_bit(0x03, Register::H) }
            0xDD => { self.set_bit(0x03, Register::L) }
            0xDE => { self.set_bit_mem(0x03, mmu) }
            0xDF => { self.set_bit(0x03, Register::A) }

            0xE0 => { self.set_bit(0x04, Register::B) }
            0xE1 => { self.set_bit(0x04, Register::C) }
            0xE2 => { self.set_bit(0x04, Register::D) }
            0xE3 => { self.set_bit(0x04, Register::E) }
            0xE4 => { self.set_bit(0x04, Register::H) }
            0xE5 => { self.set_bit(0x04, Register::L) }
            0xE6 => { self.set_bit_mem(0x04, mmu) }
            0xE7 => { self.set_bit(0x04, Register::A) }
            0xE8 => { self.set_bit(0x05, Register::B) }
            0xE9 => { self.set_bit(0x05, Register::C) }
            0xEA => { self.set_bit(0x05, Register::D) }
            0xEB => { self.set_bit(0x05, Register::E) }
            0xEC => { self.set_bit(0x05, Register::H) }
            0xED => { self.set_bit(0x05, Register::L) }
            0xEE => { self.set_bit_mem(0x05, mmu) }
            0xEF => { self.set_bit(0x05, Register::A) }

            0xF0 => { self.set_bit(0x06, Register::B) }
            0xF1 => { self.set_bit(0x06, Register::C) }
            0xF2 => { self.set_bit(0x06, Register::D) }
            0xF3 => { self.set_bit(0x06, Register::E) }
            0xF4 => { self.set_bit(0x06, Register::H) }
            0xF5 => { self.set_bit(0x06, Register::L) }
            0xF6 => { self.set_bit_mem(0x06, mmu) }
            0xF7 => { self.set_bit(0x06, Register::A) }
            0xF8 => { self.set_bit(0x07, Register::B) }
            0xF9 => { self.set_bit(0x07, Register::C) }
            0xFA => { self.set_bit(0x07, Register::D) }
            0xFB => { self.set_bit(0x07, Register::E) }
            0xFC => { self.set_bit(0x07, Register::H) }
            0xFD => { self.set_bit(0x07, Register::L) }
            0xFE => { self.set_bit_mem(0x07, mmu) }
            0xFF => { self.set_bit(0x07, Register::A) }

            _ => { unreachable!() }
        }
    }
}