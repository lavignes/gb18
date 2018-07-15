use std::collections::HashMap;
use super::*;

impl Mmu for Vec<u8> {
    fn read(&self, address: u16) -> u8 {
        self[address as usize]
    }

    fn write(&mut self, address: u16, value: u8) {
        self[address as usize] = value;
    }
}

impl Mmu for HashMap<u16, u8> {
    fn read(&self, address: u16) -> u8 {
        *self.get(&address).unwrap_or(&0x00)
    }

    fn write(&mut self, address: u16, value: u8) {
        self.insert(address, value);
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
fn write_register() {
    let mut cpu = Cpu::default();
    cpu.set_wide_register(WideRegister::BC, 0x01);
    cpu.set_register(Register::A, 0x42);
    let mut mmu = vec!(0x00, 0x00);
    assert_eq!(8, cpu.write_register(WideRegister::BC, Register::A, &mut mmu));
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
fn rlc_mem() {
    // TODO: this
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
fn read_register() {
    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0x00, 0x42);
    cpu.set_wide_register(WideRegister::BC, 0x02);
    assert_eq!(8, cpu.read_register(WideRegister::BC, Register::A, &mut mmu));
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
    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00);
    cpu.set_register(Register::A, 0x42);
    assert_eq!(8, cpu.write_a_hli(&mut mmu));
    assert_eq!(0x0001, cpu.wide_register(WideRegister::HL));
    assert_eq!(0x42, mmu.read(0x0000));
}

#[test]
fn read_a_hli() {
    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x42);
    assert_eq!(8, cpu.read_a_hli(&mut mmu));
    assert_eq!(0x0001, cpu.wide_register(WideRegister::HL));
    assert_eq!(0x42, cpu.register(Register::A));
}

#[test]
fn cpl() {
    let mut cpu = Cpu::default();
    cpu.set_register(Register::A, 0xAA);
    assert_eq!(4, cpu.cpl());
    assert_eq!(0x55, cpu.register(Register::A));
}

#[test]
fn write_a_hld() {
    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00);
    cpu.set_register(Register::A, 0x42);
    assert_eq!(8, cpu.write_a_hld(&mut mmu));
    assert_eq!(0xFFFF, cpu.wide_register(WideRegister::HL));
    assert_eq!(0x42, mmu.read(0x0000));
}

#[test]
fn read_a_hld() {
    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x42);
    assert_eq!(8, cpu.read_a_hld(&mut mmu));
    assert_eq!(0xFFFF, cpu.wide_register(WideRegister::HL));
    assert_eq!(0x42, cpu.register(Register::A));
}

#[test]
fn daa() {

}

#[test]
fn scf() {
    let mut cpu = Cpu::default();
    cpu.set_flag(Flag::Negative, true);
    cpu.set_flag(Flag::HalfCarry, true);
    assert_eq!(4, cpu.scf());
    assert_eq!(true, cpu.flag(Flag::Carry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::HalfCarry));
}

#[test]
fn ccf() {
    let mut cpu = Cpu::default();
    cpu.set_flag(Flag::Negative, true);
    cpu.set_flag(Flag::HalfCarry, true);
    assert_eq!(4, cpu.ccf());
    assert_eq!(true, cpu.flag(Flag::Carry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::HalfCarry));

    let mut cpu = Cpu::default();
    cpu.set_flag(Flag::Negative, true);
    cpu.set_flag(Flag::HalfCarry, true);
    cpu.set_flag(Flag::Carry, true);
    assert_eq!(4, cpu.ccf());
    assert_eq!(false, cpu.flag(Flag::Carry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::HalfCarry));
}

#[test]
fn inc_mem() {
    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0x0F);
    cpu.set_wide_register(WideRegister::HL, 0x0001);
    assert_eq!(12, cpu.inc_mem(&mut mmu));
    assert_eq!(0x10, mmu.read(0x0001));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Zero));

    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0xFF);
    cpu.set_wide_register(WideRegister::HL, 0x0001);
    assert_eq!(12, cpu.inc_mem(&mut mmu));
    assert_eq!(0x00, mmu.read(0x0001));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(true, cpu.flag(Flag::Zero));

    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0x41);
    cpu.set_wide_register(WideRegister::HL, 0x0001);
    assert_eq!(12, cpu.inc_mem(&mut mmu));
    assert_eq!(0x42, mmu.read(0x0001));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Zero));
}

#[test]
fn dec_mem() {
    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0x10);
    cpu.set_wide_register(WideRegister::HL, 0x0001);
    assert_eq!(12, cpu.dec_mem(&mut mmu));
    assert_eq!(0x0F, mmu.read(0x0001));
    assert_eq!(true, cpu.flag(Flag::Negative));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Zero));

    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0x00);
    cpu.set_wide_register(WideRegister::HL, 0x0001);
    assert_eq!(12, cpu.dec_mem(&mut mmu));
    assert_eq!(0xFF, mmu.read(0x0001));
    assert_eq!(true, cpu.flag(Flag::Negative));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Zero));

    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0x01);
    cpu.set_wide_register(WideRegister::HL, 0x0001);
    assert_eq!(12, cpu.dec_mem(&mut mmu));
    assert_eq!(0x00, mmu.read(0x0001));
    assert_eq!(true, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::HalfCarry));
    assert_eq!(true, cpu.flag(Flag::Zero));
}

#[test]
fn write_mem_immediate() {
    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x42, 0x00, 0x00, 0x00);
    cpu.set_wide_register(WideRegister::HL, 0x0002);
    assert_eq!(12, cpu.write_mem_immediate(&mut mmu));
    assert_eq!(0x42, mmu.read(0x0002));
}

#[test]
fn copy_register() {
    let mut cpu = Cpu::default();
    cpu.set_register(Register::E, 0x42);
    assert_eq!(4, cpu.copy_register(Register::B, Register::E));
    assert_eq!(0x42, cpu.register(Register::B));
    assert_eq!(4, cpu.copy_register(Register::A, Register::B));
    assert_eq!(0x42, cpu.register(Register::A));
}

#[test]
fn add() {
    let mut cpu = Cpu::default();
    cpu.set_register(Register::A, 0x0F);
    cpu.set_register(Register::B, 0x02);
    assert_eq!(4, cpu.add(Register::B));
    assert_eq!(0x11, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    cpu.set_register(Register::A, 0xFF);
    cpu.set_register(Register::C, 0x02);
    assert_eq!(4, cpu.add(Register::C));
    assert_eq!(0x01, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(true, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    cpu.set_register(Register::A, 0x01);
    cpu.set_register(Register::H, 0xFF);
    assert_eq!(4, cpu.add(Register::H));
    assert_eq!(0x00, cpu.register(Register::A));
    assert_eq!(true, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(true, cpu.flag(Flag::Carry));
}

#[test]
fn add_mem() {
    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0x02);
    cpu.set_register(Register::A, 0x0F);
    cpu.set_wide_register(WideRegister::HL, 0x01);
    assert_eq!(8, cpu.add_mem(&mut mmu));
    assert_eq!(0x11, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0x02);
    cpu.set_register(Register::A, 0xFF);
    cpu.set_wide_register(WideRegister::HL, 0x01);
    assert_eq!(8, cpu.add_mem(&mut mmu));
    assert_eq!(0x01, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(true, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0xFF);
    cpu.set_register(Register::A, 0x01);
    cpu.set_wide_register(WideRegister::HL, 0x01);
    assert_eq!(8, cpu.add_mem(&mut mmu));
    assert_eq!(0x00, cpu.register(Register::A));
    assert_eq!(true, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(true, cpu.flag(Flag::Carry));
}

#[test]
fn add_carry() {
    let mut cpu = Cpu::default();
    cpu.set_register(Register::A, 0x0E);
    cpu.set_register(Register::B, 0x01);
    cpu.set_flag(Flag::Carry, true);
    assert_eq!(4, cpu.add_carry(Register::B));
    assert_eq!(0x10, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    cpu.set_register(Register::A, 0xFE);
    cpu.set_register(Register::L, 0x01);
    cpu.set_flag(Flag::Carry, true);
    assert_eq!(4, cpu.add_carry(Register::L));
    assert_eq!(0x00, cpu.register(Register::A));
    assert_eq!(true, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(true, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    cpu.set_register(Register::A, 0xFE);
    cpu.set_register(Register::L, 0x01);
    cpu.set_flag(Flag::Carry, false);
    assert_eq!(4, cpu.add_carry(Register::L));
    assert_eq!(0xFF, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(false, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::Carry));
}

#[test]
fn add_carry_mem() {
    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0x01);
    cpu.set_register(Register::A, 0x0E);
    cpu.set_wide_register(WideRegister::HL, 0x01);
    cpu.set_flag(Flag::Carry, true);
    assert_eq!(8, cpu.add_carry_mem(&mut mmu));
    assert_eq!(0x10, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0x01);
    cpu.set_register(Register::A, 0xFE);
    cpu.set_wide_register(WideRegister::HL, 0x01);
    cpu.set_flag(Flag::Carry, true);
    assert_eq!(8, cpu.add_carry_mem(&mut mmu));
    assert_eq!(0x00, cpu.register(Register::A));
    assert_eq!(true, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(true, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0x01);
    cpu.set_register(Register::A, 0xFE);
    cpu.set_wide_register(WideRegister::HL, 0x01);
    cpu.set_flag(Flag::Carry, false);
    assert_eq!(8, cpu.add_carry_mem(&mut mmu));
    assert_eq!(0xFF, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(false, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::Carry));
}

#[test]
fn sub() {
    let mut cpu = Cpu::default();
    cpu.set_register(Register::A, 0x10);
    cpu.set_register(Register::B, 0x02);
    assert_eq!(4, cpu.sub(Register::B));
    assert_eq!(0x0E, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(true, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    cpu.set_register(Register::A, 0x00);
    cpu.set_register(Register::C, 0x02);
    assert_eq!(4, cpu.sub(Register::C));
    assert_eq!(0xFE, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(true, cpu.flag(Flag::Negative));
    assert_eq!(true, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    cpu.set_register(Register::A, 0x01);
    cpu.set_register(Register::H, 0xFF);
    assert_eq!(4, cpu.sub(Register::H));
    assert_eq!(0x02, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(true, cpu.flag(Flag::Negative));
    assert_eq!(true, cpu.flag(Flag::Carry));
}

#[test]
fn sub_mem() {
    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0x02);
    cpu.set_register(Register::A, 0x10);
    cpu.set_wide_register(WideRegister::HL, 0x01);
    assert_eq!(8, cpu.sub_mem(&mut mmu));
    assert_eq!(0x0E, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(true, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0x02);
    cpu.set_register(Register::A, 0x00);
    cpu.set_wide_register(WideRegister::HL, 0x01);
    assert_eq!(8, cpu.sub_mem(&mut mmu));
    assert_eq!(0xFE, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(true, cpu.flag(Flag::Negative));
    assert_eq!(true, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0xFF);
    cpu.set_register(Register::A, 0x01);
    cpu.set_wide_register(WideRegister::HL, 0x01);
    assert_eq!(8, cpu.sub_mem(&mut mmu));
    assert_eq!(0x02, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(true, cpu.flag(Flag::Negative));
    assert_eq!(true, cpu.flag(Flag::Carry));
}

#[test]
fn sub_carry() {
    let mut cpu = Cpu::default();
    cpu.set_register(Register::A, 0x11);
    cpu.set_register(Register::B, 0x01);
    cpu.set_flag(Flag::Carry, true);
    assert_eq!(4, cpu.sub_carry(Register::B));
    assert_eq!(0x0F, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(true, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    cpu.set_register(Register::A, 0x00);
    cpu.set_register(Register::L, 0xFF);
    cpu.set_flag(Flag::Carry, true);
    assert_eq!(4, cpu.sub_carry(Register::L));
    assert_eq!(0x00, cpu.register(Register::A));
    assert_eq!(true, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(true, cpu.flag(Flag::Negative));
    assert_eq!(true, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    cpu.set_register(Register::A, 0x00);
    cpu.set_register(Register::L, 0xFF);
    cpu.set_flag(Flag::Carry, false);
    assert_eq!(4, cpu.sub_carry(Register::L));
    assert_eq!(0x01, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(true, cpu.flag(Flag::Negative));
    assert_eq!(true, cpu.flag(Flag::Carry));
}

#[test]
fn sub_carry_mem() {
    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0x01);
    cpu.set_register(Register::A, 0x11);
    cpu.set_wide_register(WideRegister::HL, 0x01);
    cpu.set_flag(Flag::Carry, true);
    assert_eq!(8, cpu.sub_carry_mem(&mut mmu));
    assert_eq!(0x0F, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(true, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0xFF);
    cpu.set_register(Register::A, 0x00);
    cpu.set_wide_register(WideRegister::HL, 0x01);
    cpu.set_flag(Flag::Carry, true);
    assert_eq!(8, cpu.sub_carry_mem(&mut mmu));
    assert_eq!(0x00, cpu.register(Register::A));
    assert_eq!(true, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(true, cpu.flag(Flag::Negative));
    assert_eq!(true, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0xFF);
    cpu.set_register(Register::A, 0x00);
    cpu.set_wide_register(WideRegister::HL, 0x01);
    cpu.set_flag(Flag::Carry, false);
    assert_eq!(8, cpu.sub_carry_mem(&mut mmu));
    assert_eq!(0x01, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(true, cpu.flag(Flag::Negative));
    assert_eq!(true, cpu.flag(Flag::Carry));
}

#[test]
fn and() {
    let mut cpu = Cpu::default();
    cpu.set_register(Register::A, 0x01);
    cpu.set_register(Register::B, 0x01);
    assert_eq!(4, cpu.and(Register::B));
    assert_eq!(0x01, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    cpu.set_register(Register::A, 0x01);
    cpu.set_register(Register::C, 0x00);
    assert_eq!(4, cpu.and(Register::C));
    assert_eq!(0x00, cpu.register(Register::A));
    assert_eq!(true, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::Carry));
}

#[test]
fn and_mem() {
    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0x01);
    cpu.set_register(Register::A, 0x01);
    cpu.set_wide_register(WideRegister::HL, 0x01);
    assert_eq!(8, cpu.and_mem(&mut mmu));
    assert_eq!(0x01, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0x00);
    cpu.set_register(Register::A, 0x01);
    cpu.set_wide_register(WideRegister::HL, 0x01);
    assert_eq!(8, cpu.and_mem(&mut mmu));
    assert_eq!(0x00, cpu.register(Register::A));
    assert_eq!(true, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::Carry));
}

#[test]
fn xor() {
    let mut cpu = Cpu::default();
    cpu.set_register(Register::A, 0x01);
    cpu.set_register(Register::B, 0x01);
    assert_eq!(4, cpu.xor(Register::B));
    assert_eq!(0x00, cpu.register(Register::A));
    assert_eq!(true, cpu.flag(Flag::Zero));
    assert_eq!(false, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    cpu.set_register(Register::A, 0x01);
    cpu.set_register(Register::C, 0x00);
    assert_eq!(4, cpu.xor(Register::C));
    assert_eq!(0x01, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(false, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::Carry));
}

#[test]
fn xor_mem() {
    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0x01);
    cpu.set_register(Register::A, 0x01);
    cpu.set_wide_register(WideRegister::HL, 0x01);
    assert_eq!(8, cpu.xor_mem(&mut mmu));
    assert_eq!(0x00, cpu.register(Register::A));
    assert_eq!(true, cpu.flag(Flag::Zero));
    assert_eq!(false, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0x00);
    cpu.set_register(Register::A, 0x01);
    cpu.set_wide_register(WideRegister::HL, 0x01);
    assert_eq!(8, cpu.xor_mem(&mut mmu));
    assert_eq!(0x01, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(false, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::Carry));
}

#[test]
fn or() {
    let mut cpu = Cpu::default();
    cpu.set_register(Register::A, 0x01);
    cpu.set_register(Register::B, 0x01);
    assert_eq!(4, cpu.or(Register::B));
    assert_eq!(0x01, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(false, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    cpu.set_register(Register::A, 0x01);
    cpu.set_register(Register::C, 0x00);
    assert_eq!(4, cpu.or(Register::C));
    assert_eq!(0x01, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(false, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::Carry));
}

#[test]
fn or_mem() {
    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0x01);
    cpu.set_register(Register::A, 0x01);
    cpu.set_wide_register(WideRegister::HL, 0x01);
    assert_eq!(8, cpu.or_mem(&mut mmu));
    assert_eq!(0x01, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(false, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0x00);
    cpu.set_register(Register::A, 0x01);
    cpu.set_wide_register(WideRegister::HL, 0x01);
    assert_eq!(8, cpu.or_mem(&mut mmu));
    assert_eq!(0x01, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(false, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::Carry));
}

#[test]
fn cp() {
    let mut cpu = Cpu::default();
    cpu.set_register(Register::A, 0x01);
    cpu.set_register(Register::B, 0x01);
    assert_eq!(4, cpu.cp(Register::B));
    assert_eq!(0x01, cpu.register(Register::A));
    assert_eq!(true, cpu.flag(Flag::Zero));
    assert_eq!(false, cpu.flag(Flag::HalfCarry));
    assert_eq!(true, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    cpu.set_register(Register::A, 0x01);
    cpu.set_register(Register::C, 0x00);
    assert_eq!(4, cpu.cp(Register::C));
    assert_eq!(0x01, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(false, cpu.flag(Flag::HalfCarry));
    assert_eq!(true, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    cpu.set_register(Register::A, 0x01);
    cpu.set_register(Register::H, 0x02);
    assert_eq!(4, cpu.cp(Register::H));
    assert_eq!(0x01, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(true, cpu.flag(Flag::Negative));
    assert_eq!(true, cpu.flag(Flag::Carry));
}

#[test]
fn cp_mem() {
    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0x01);
    cpu.set_register(Register::A, 0x01);
    cpu.set_wide_register(WideRegister::HL, 0x01);
    assert_eq!(8, cpu.cp_mem(&mut mmu));
    assert_eq!(0x01, cpu.register(Register::A));
    assert_eq!(true, cpu.flag(Flag::Zero));
    assert_eq!(false, cpu.flag(Flag::HalfCarry));
    assert_eq!(true, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0x00);
    cpu.set_register(Register::A, 0x01);
    cpu.set_wide_register(WideRegister::HL, 0x01);
    assert_eq!(8, cpu.cp_mem(&mut mmu));
    assert_eq!(0x01, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(false, cpu.flag(Flag::HalfCarry));
    assert_eq!(true, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0x02);
    cpu.set_register(Register::A, 0x01);
    cpu.set_wide_register(WideRegister::HL, 0x01);
    assert_eq!(8, cpu.cp_mem(&mut mmu));
    assert_eq!(0x01, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(true, cpu.flag(Flag::Negative));
    assert_eq!(true, cpu.flag(Flag::Carry));
}

#[test]
fn ret() {
    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0x00, 0x00, 0x00, 0x02, 0x00);
    cpu.set_wide_register(WideRegister::SP, 0x0004);
    assert_eq!(16, cpu.ret(&mut mmu));
    assert_eq!(0x0006, cpu.wide_register(WideRegister::SP));
    assert_eq!(0x0002, cpu.wide_register(WideRegister::PC));
}

#[test]
fn ret_condition() {
    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0x00, 0x00, 0x00, 0x02, 0x00);
    cpu.set_wide_register(WideRegister::SP, 0x0004);
    cpu.set_flag(Flag::Zero, true);
    assert_eq!(20, cpu.ret_condition(Condition::Zero, &mut mmu));
    assert_eq!(0x0006, cpu.wide_register(WideRegister::SP));
    assert_eq!(0x0002, cpu.wide_register(WideRegister::PC));

    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0x00, 0x00, 0x00, 0x02, 0x00);
    cpu.set_wide_register(WideRegister::SP, 0x0004);
    cpu.set_flag(Flag::Zero, false);
    assert_eq!(8, cpu.ret_condition(Condition::Zero, &mut mmu));
    assert_eq!(0x0004, cpu.wide_register(WideRegister::SP));
    assert_eq!(0x0000, cpu.wide_register(WideRegister::PC));

    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0x00, 0x00, 0x00, 0x02, 0x00);
    cpu.set_wide_register(WideRegister::SP, 0x0004);
    cpu.set_flag(Flag::Carry, true);
    assert_eq!(8, cpu.ret_condition(Condition::NotCarry, &mut mmu));
    assert_eq!(0x0004, cpu.wide_register(WideRegister::SP));
    assert_eq!(0x0000, cpu.wide_register(WideRegister::PC));

    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0x00, 0x00, 0x00, 0x02, 0x00);
    cpu.set_wide_register(WideRegister::SP, 0x0004);
    cpu.set_flag(Flag::Carry, false);
    assert_eq!(20, cpu.ret_condition(Condition::NotCarry, &mut mmu));
    assert_eq!(0x0006, cpu.wide_register(WideRegister::SP));
    assert_eq!(0x0002, cpu.wide_register(WideRegister::PC));
}

#[test]
fn pop_wide() {
    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0x00, 0x00, 0x00, 0xEF, 0xBE);
    cpu.set_wide_register(WideRegister::SP, 0x0004);
    assert_eq!(12, cpu.pop_wide(WideRegister::DE, &mut mmu));
    assert_eq!(0xBEEF, cpu.wide_register(WideRegister::DE));
    assert_eq!(0x0006, cpu.wide_register(WideRegister::SP));
}

#[test]
fn push_wide() {
    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0x00, 0x00, 0x00, 0x00, 0x00);
    cpu.set_wide_register(WideRegister::HL, 0xBEEF);
    cpu.set_wide_register(WideRegister::SP, 0x0006);
    assert_eq!(16, cpu.push_wide(WideRegister::HL, &mut mmu));
    assert_eq!(0x0004, cpu.wide_register(WideRegister::SP));
    assert_eq!(0xEF, mmu.read(0x0004));
    assert_eq!(0xBE, mmu.read(0x0005));
}

#[test]
fn jmp_condition() {
    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x04, 0x00, 0x00, 0x00, 0x00, 0x00);
    assert_eq!(16, cpu.jmp_condition(Condition::NotCarry, &mut mmu));
    assert_eq!(0x0004, cpu.wide_register(WideRegister::PC));

    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x04, 0x00, 0x00, 0x00, 0x00, 0x00);
    cpu.set_flag(Flag::Carry, true);
    assert_eq!(12, cpu.jmp_condition(Condition::NotCarry, &mut mmu));
    assert_eq!(0x0002, cpu.wide_register(WideRegister::PC));
}

#[test]
fn jmp() {
    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x04, 0x00, 0x00, 0x00, 0x00, 0x00);
    assert_eq!(16, cpu.jmp(&mut mmu));
    assert_eq!(0x0004, cpu.wide_register(WideRegister::PC));
}

#[test]
fn call_condition() {
    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00);
    cpu.set_wide_register(WideRegister::SP, 0x0008);
    assert_eq!(24, cpu.call_condition(Condition::NotZero, &mut mmu));
    assert_eq!(0x0003, cpu.wide_register(WideRegister::PC));
    assert_eq!(0x0006, cpu.wide_register(WideRegister::SP));
    assert_eq!(0x02, mmu.read(0x0006));
    assert_eq!(0x00, mmu.read(0x0007));

    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00);
    cpu.set_wide_register(WideRegister::SP, 0x0008);
    cpu.set_flag(Flag::Zero, true);
    assert_eq!(12, cpu.call_condition(Condition::NotZero, &mut mmu));
    assert_eq!(0x0002, cpu.wide_register(WideRegister::PC));
    assert_eq!(0x0008, cpu.wide_register(WideRegister::SP));
    assert_eq!(0x00, mmu.read(0x0006));
    assert_eq!(0x00, mmu.read(0x0007));
}

#[test]
fn call() {
    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00);
    cpu.set_wide_register(WideRegister::SP, 0x0008);
    assert_eq!(24, cpu.call(&mut mmu));
    assert_eq!(0x0003, cpu.wide_register(WideRegister::PC));
    assert_eq!(0x0006, cpu.wide_register(WideRegister::SP));
    assert_eq!(0x02, mmu.read(0x0006));
    assert_eq!(0x00, mmu.read(0x0007));
}

#[test]
fn add_immediate() {
    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x02);
    cpu.set_register(Register::A, 0x0F);
    assert_eq!(8, cpu.add_immediate(&mut mmu));
    assert_eq!(0x11, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x02);
    cpu.set_register(Register::A, 0xFF);
    assert_eq!(8, cpu.add_immediate(&mut mmu));
    assert_eq!(0x01, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(true, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    let mut mmu = vec!(0xFF);
    cpu.set_register(Register::A, 0x01);
    assert_eq!(8, cpu.add_immediate(&mut mmu));
    assert_eq!(0x00, cpu.register(Register::A));
    assert_eq!(true, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(true, cpu.flag(Flag::Carry));
}

#[test]
fn add_carry_immediate() {
    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x01);
    cpu.set_register(Register::A, 0x0E);
    cpu.set_flag(Flag::Carry, true);
    assert_eq!(8, cpu.add_carry_immediate(&mut mmu));
    assert_eq!(0x10, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x01);
    cpu.set_register(Register::A, 0xFE);
    cpu.set_flag(Flag::Carry, true);
    assert_eq!(8, cpu.add_carry_immediate(&mut mmu));
    assert_eq!(0x00, cpu.register(Register::A));
    assert_eq!(true, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(true, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x01);
    cpu.set_register(Register::A, 0xFE);
    cpu.set_flag(Flag::Carry, false);
    assert_eq!(8, cpu.add_carry_immediate(&mut mmu));
    assert_eq!(0xFF, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(false, cpu.flag(Flag::HalfCarry));
    assert_eq!(false, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::Carry));
}

#[test]
fn sub_immediate() {
    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x02);
    cpu.set_register(Register::A, 0x10);
    assert_eq!(8, cpu.sub_immediate(&mut mmu));
    assert_eq!(0x0E, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(true, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x02);
    cpu.set_register(Register::A, 0x00);
    assert_eq!(8, cpu.sub_immediate(&mut mmu));
    assert_eq!(0xFE, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(true, cpu.flag(Flag::Negative));
    assert_eq!(true, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    let mut mmu = vec!(0xFF);
    cpu.set_register(Register::A, 0x01);
    assert_eq!(8, cpu.sub_immediate(&mut mmu));
    assert_eq!(0x02, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(true, cpu.flag(Flag::Negative));
    assert_eq!(true, cpu.flag(Flag::Carry));
}

#[test]
fn sub_carry_immediate() {
    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x01);
    cpu.set_register(Register::A, 0x11);
    cpu.set_flag(Flag::Carry, true);
    assert_eq!(8, cpu.sub_carry_immediate(&mut mmu));
    assert_eq!(0x0F, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(true, cpu.flag(Flag::Negative));
    assert_eq!(false, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    let mut mmu = vec!(0xFF);
    cpu.set_register(Register::A, 0x00);
    cpu.set_flag(Flag::Carry, true);
    assert_eq!(8, cpu.sub_carry_immediate(&mut mmu));
    assert_eq!(0x00, cpu.register(Register::A));
    assert_eq!(true, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(true, cpu.flag(Flag::Negative));
    assert_eq!(true, cpu.flag(Flag::Carry));

    let mut cpu = Cpu::default();
    let mut mmu = vec!(0xFF);
    cpu.set_register(Register::A, 0x00);
    cpu.set_flag(Flag::Carry, false);
    assert_eq!(8, cpu.sub_carry_immediate(&mut mmu));
    assert_eq!(0x01, cpu.register(Register::A));
    assert_eq!(false, cpu.flag(Flag::Zero));
    assert_eq!(true, cpu.flag(Flag::HalfCarry));
    assert_eq!(true, cpu.flag(Flag::Negative));
    assert_eq!(true, cpu.flag(Flag::Carry));
}

#[test]
fn rst() {
    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00);
    cpu.set_wide_register(WideRegister::SP, 0x0008);
    cpu.set_wide_register(WideRegister::PC, 0x0001);
    assert_eq!(16, cpu.rst(0x0003, &mut mmu));
    assert_eq!(0x0003, cpu.wide_register(WideRegister::PC));
    assert_eq!(0x0006, cpu.wide_register(WideRegister::SP));
    assert_eq!(0x01, mmu.read(0x0006));
    assert_eq!(0x00, mmu.read(0x0007));
}

#[test]
fn reti() {
    let mut cpu = Cpu::default();
    let mut mmu = vec!(0x00, 0x00, 0x00, 0x00, 0x02, 0x00);
    cpu.set_wide_register(WideRegister::SP, 0x0004);
    cpu.interrupts_enabled = false;
    assert_eq!(16, cpu.reti(&mut mmu));
    assert_eq!(true, cpu.interrupts_enabled);
    assert_eq!(0x0006, cpu.wide_register(WideRegister::SP));
    assert_eq!(0x0002, cpu.wide_register(WideRegister::PC));
}

#[test]
fn write_high_immediate() {
    let mut cpu = Cpu::default();
    let mut mmu = HashMap::new();
    mmu.write(0x0000, 0x04);
    cpu.set_register(Register::A, 0x42);
    assert_eq!(12, cpu.write_high_immediate(Register::A, &mut mmu));
    assert_eq!(0x42, mmu.read(0xFF04));
}

#[test]
fn write_high_register() {
    let mut cpu = Cpu::default();
    let mut mmu = HashMap::new();
    cpu.set_register(Register::A, 0x42);
    cpu.set_register(Register::C, 0xBE);
    assert_eq!(8, cpu.write_high_register(Register::C, Register::A, &mut mmu));
    assert_eq!(0x42, mmu.read(0xFFBE));
}

#[test]
fn read_high_immediate() {

}

#[test]
fn read_high_register() {

}

#[test]
fn and_immediate() {

}

#[test]
fn or_immediate() {

}

#[test]
fn xor_immediate() {

}

#[test]
fn cp_immediate() {

}

#[test]
fn add_sp() {

}

#[test]
fn jmp_hl() {

}

#[test]
fn write_register_immediate() {

}

#[test]
fn read_register_immediate() {

}

#[test]
fn copy_wide_register() {

}