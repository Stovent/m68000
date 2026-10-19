// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Test the behavior of the SR trace bit as explained in M68000 User Manual, chapter `6.3.8 Tracing`.
//!
//! # TODO
//!
//! - Test the 6.3.8 Tracing behavior:
//! > During the execution of the instruction, if an exception is forced by that instruction, the exception processing
//! > for the instruction exception occurs before that of the trace exception.

use m68000::M68000;
use m68000::addressing_modes::AddressingMode;
use m68000::cpu_details::Mc68000;
use m68000::assembler::*;
use m68000::exception::Vector;

#[test]
fn tracing() {
    let move_to_sr = movesr(AddressingMode::Immediate(0x0000));
    let mut memory = [
        nop(),
        nop(),
        nop(),
        illegal(),
        move_to_sr[0],
        move_to_sr[1],
    ];
    let mut cpu = M68000::<Mc68000>::new_no_reset();

    let (_, ex) = cpu.interpreter_exception(&mut memory[..]); // nop
    assert_eq!(ex, None); // Tracing not enabled

    cpu.regs.sr.t = true;
    let (_, ex) = cpu.interpreter_exception(&mut memory[..]); // nop
    assert_eq!(ex, Some(Vector::Trace as u8)); // Tracing enabled

    cpu.regs.sr.s = true;
    let (_, ex) = cpu.interpreter_exception(&mut memory[..]); // nop
    assert_eq!(ex, Some(Vector::Trace as u8)); // Tracing enabled while supervisor

    let (_, ex) = cpu.interpreter_exception(&mut memory[..]); // illegal
    assert_eq!(ex, Some(Vector::IllegalInstruction as u8)); // Illegal instruction

    let (_, ex) = cpu.interpreter_exception(&mut memory[..]); // move to sr
    assert_eq!(ex, None); // Privileged instructions are not tracable.

    let (_, ex) = cpu.interpreter_exception(&mut memory[..]); // Bus error
    assert_eq!(ex, Some(Vector::AccessError as u8)); // Bus error occured
}
