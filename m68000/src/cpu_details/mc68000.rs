// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::cpu_details::{CpuDetails, StackFormat};

/// The CPU details of a MC68000 CPU as described in the M68000 8-/16-/32-Bit Microprocessors User’s Manual, Ninth Edition.
#[derive(Clone, Copy, Debug, Default)]
pub struct Mc68000;

impl CpuDetails for Mc68000 {
    const STACK_FORMAT: StackFormat = StackFormat::MC68000;

    const VECTOR_RESET: usize = 40;
    fn vector_execution_time(vector: u8) -> usize {
        match vector {
            2 => 50, // Access Error
            3 => 50, // Address Error
            4 => 34, // Illegal
            5 => 38, // Zero Divide
            6 => 40, // Chk
            7 => 34, // Trapv
            8 => 34, // Privilege Violation
            9 => 34, // Trace
            24..=31 => 44, // Interrupt
            32..=47 => 34, // Trap
            _ => Self::VECTOR_RESET, // TODO: what to return with the other vectors?
        }
    }

    const EA_ARI: usize = 4;
    const EA_ARIWPO: usize = 4;
    const EA_ARIWPR: usize = 6;
    const EA_ARIWD: usize = 8;
    const EA_ARIWI8: usize = 10;
    const EA_ABSSHORT: usize = 8;
    const EA_ABSLONG: usize = 12;
    const EA_PCIWD: usize = 8;
    const EA_PCIWI8: usize = 10;
    const EA_IMMEDIATE: usize = 4;

    const ABCD_REG: usize = 6;
    const ABCD_MEM: usize = 18;

    const ADD_REG_BW: usize = 4;
    const ADD_REG_L: usize = 6;
    const ADD_REG_L_RDIMM: usize = 8;
    const ADD_MEM_BW: usize = 8;
    const ADD_MEM_L: usize = 12;

    const ADDA_WORD: usize = 8;
    const ADDA_LONG: usize = 6;
    const ADDA_LONG_RDIMM: usize = 8;

    const ADDI_REG_BW: usize = 8;
    const ADDI_REG_L: usize = 16;
    const ADDI_MEM_BW: usize = 12;
    const ADDI_MEM_L: usize = 20;

    const ADDQ_REG_BW: usize = 4;
    const ADDQ_REG_L: usize = 8;
    const ADDQ_MEM_BW: usize = 8;
    const ADDQ_MEM_L: usize = 12;

    const ADDX_REG_BW: usize = 4;
    const ADDX_REG_L: usize = 8;
    const ADDX_MEM_BW: usize = 18;
    const ADDX_MEM_L: usize = 30;

    const AND_REG_BW: usize = 4;
    const AND_REG_L: usize = 6;
    const AND_REG_L_RDIMM: usize = 8;
    const AND_MEM_BW: usize = 8;
    const AND_MEM_L: usize = 12;

    const ANDI_REG_BW: usize = 8;
    const ANDI_REG_L: usize = 14;
    const ANDI_MEM_BW: usize = 12;
    const ANDI_MEM_L: usize = 20;

    const ANDICCR: usize = 20;

    const ANDISR: usize = 20;

    const ASM: usize = 8;

    const ASR_COUNT: usize = 2;
    const ASR_BW: usize = 6;
    const ASR_L: usize = 8;

    const BCC_BRANCH: usize = 10;
    const BCC_NO_BRANCH_BYTE: usize = 8;
    const BCC_NO_BRANCH_WORD: usize = 12;

    const BCHG_DYN_REG: usize = 8;
    const BCHG_DYN_MEM: usize = 8;
    const BCHG_STA_REG: usize = 12;
    const BCHG_STA_MEM: usize = 12;

    const BCLR_DYN_REG: usize = 10;
    const BCLR_DYN_MEM: usize = 8;
    const BCLR_STA_REG: usize = 14;
    const BCLR_STA_MEM: usize = 12;

    const BRA_BYTE: usize = 10;
    const BRA_WORD: usize = 10;

    const BSET_DYN_REG: usize = 8;
    const BSET_DYN_MEM: usize = 8;
    const BSET_STA_REG: usize = 12;
    const BSET_STA_MEM: usize = 12;

    const BSR_BYTE: usize = 18;
    const BSR_WORD: usize = 18;

    const BTST_DYN_REG: usize = 6;
    const BTST_DYN_MEM: usize = 4;
    const BTST_STA_REG: usize = 10;
    const BTST_STA_MEM: usize = 8;

    const CHK_NO_TRAP: usize = 10;

    const CLR_REG_BW: usize = 4;
    const CLR_REG_L: usize = 6;
    const CLR_MEM_BW: usize = 8;
    const CLR_MEM_L: usize = 12;

    const CMP_BW: usize = 4;
    const CMP_L: usize = 6;

    const CMPA: usize = 6;

    const CMPI_REG_BW: usize = 8;
    const CMPI_REG_L: usize = 14;
    const CMPI_MEM_BW: usize = 8;
    const CMPI_MEM_L: usize = 12;

    const CMPM_BW: usize = 12;
    const CMPM_L: usize = 20;

    const DBCC_TRUE: usize = 12;
    const DBCC_FALSE_BRANCH: usize = 10;
    const DBCC_FALSE_NO_BRANCH: usize = 14;

    const DIVS: usize = 158;
    const DIVU: usize = 140;

    const EOR_REG_BW: usize = 4;
    const EOR_REG_L: usize = 8;
    const EOR_MEM_BW: usize = 8;
    const EOR_MEM_L: usize = 12;

    const EORI_REG_BW: usize = 8;
    const EORI_REG_L: usize = 16;
    const EORI_MEM_BW: usize = 12;
    const EORI_MEM_L: usize = 20;

    const EORICCR: usize = 20;

    const EORISR: usize = 20;

    const EXG: usize = 6;

    const EXT: usize = 4;

    const JMP_ARI: usize = 8;
    const JMP_ARIWD: usize = 10;
    const JMP_ARIWI8: usize = 14;
    const JMP_ABSSHORT: usize = 10;
    const JMP_ABSLONG: usize = 12;
    const JMP_PCIWD: usize = 10;
    const JMP_PCIWI8: usize = 14;

    const JSR_ARI: usize = 16;
    const JSR_ARIWD: usize = 18;
    const JSR_ARIWI8: usize = 22;
    const JSR_ABSSHORT: usize = 18;
    const JSR_ABSLONG: usize = 20;
    const JSR_PCIWD: usize = 18;
    const JSR_PCIWI8: usize = 22;

    const LEA_ARI: usize = 4;
    const LEA_ARIWD: usize = 8;
    const LEA_ARIWI8: usize = 12;
    const LEA_ABSSHORT: usize = 8;
    const LEA_ABSLONG: usize = 12;
    const LEA_PCIWD: usize = 8;
    const LEA_PCIWI8: usize = 12;

    const LINK: usize = 16;

    const LSM: usize = 8;

    const LSR_COUNT: usize = 2;
    const LSR_BW: usize = 6;
    const LSR_L: usize = 8;

    const MOVE_DST_ARIWPR: usize = 2;
    const MOVE_OTHER: usize = 4;

    const MOVEA: usize = 4;

    const MOVECCR: usize = 12;

    const MOVEFSR_REG: usize = 6;
    const MOVEFSR_MEM: usize = 8;

    const MOVESR: usize = 12;

    const MOVEUSP: usize = 4;

    const MOVEM_WORD: usize = 4;
    const MOVEM_LONG: usize = 8;
    const MOVEM_MTR: usize = 4;
    const MOVEM_ARI: usize = 8; // R -> M, do +4 for M -> R.
    const MOVEM_ARIWPO: usize = 8;
    const MOVEM_ARIWPR: usize = 8;
    const MOVEM_ARIWD: usize = 12;
    const MOVEM_ARIWI8: usize = 14;
    const MOVEM_ABSSHORT: usize = 12;
    const MOVEM_ABSLONG: usize = 16;
    const MOVEM_PCIWD: usize = 12;
    const MOVEM_PCIWI8: usize = 14;

    const MOVEP_RTM_WORD: usize = 16;
    const MOVEP_RTM_LONG: usize = 24;
    const MOVEP_MTR_WORD: usize = 16;
    const MOVEP_MTR_LONG: usize = 24;

    const MOVEQ: usize = 4;

    const MULS: usize = 70;

    const MULU: usize = 70;

    const NBCD_REG: usize = 6;
    const NBCD_MEM: usize = 8;

    const NEG_REG_BW: usize = 4;
    const NEG_REG_L: usize = 6;
    const NEG_MEM_BW: usize = 8;
    const NEG_MEM_L: usize = 12;

    const NEGX_REG_BW: usize = 4;
    const NEGX_REG_L: usize = 6;
    const NEGX_MEM_BW: usize = 8;
    const NEGX_MEM_L: usize = 12;

    const NOP: usize = 4;

    const NOT_REG_BW: usize = 4;
    const NOT_REG_L: usize = 6;
    const NOT_MEM_BW: usize = 8;
    const NOT_MEM_L: usize = 12;

    const OR_REG_BW: usize = 4;
    const OR_REG_L: usize = 6;
    const OR_REG_L_RDIMM: usize = 8;
    const OR_MEM_BW: usize = 8;
    const OR_MEM_L: usize = 12;

    const ORI_REG_BW: usize = 8;
    const ORI_REG_L: usize = 16;
    const ORI_MEM_BW: usize = 12;
    const ORI_MEM_L: usize = 20;

    const ORICCR: usize = 20;

    const ORISR: usize = 20;

    const PEA_ARI: usize = 12;
    const PEA_ARIWD: usize = 16;
    const PEA_ARIWI8: usize = 20;
    const PEA_ABSSHORT: usize = 16;
    const PEA_ABSLONG: usize = 20;
    const PEA_PCIWD: usize = 16;
    const PEA_PCIWI8: usize = 20;

    const RESET: usize = 132;

    const ROM: usize = 8;

    const ROR_COUNT: usize = 2;
    const ROR_BW: usize = 6;
    const ROR_L: usize = 8;

    const ROXM: usize = 8;

    const ROXR_COUNT: usize = 2;
    const ROXR_BW: usize = 6;
    const ROXR_L: usize = 8;

    const RTE: usize = 20;

    const RTR: usize = 20;

    const RTS: usize = 16;

    const SBCD_REG: usize = 6;
    const SBCD_MEM: usize = 18;

    const SCC_REG_FALSE: usize = 4;
    const SCC_REG_TRUE: usize = 6;
    const SCC_MEM_FALSE: usize = 8;
    const SCC_MEM_TRUE: usize = 8;

    const STOP: usize = 4;

    const SUB_REG_BW: usize = 4;
    const SUB_REG_L: usize = 6;
    const SUB_REG_L_RDIMM: usize = 8;
    const SUB_MEM_BW: usize = 8;
    const SUB_MEM_L: usize = 12;

    const SUBA_WORD: usize = 8;
    const SUBA_LONG: usize = 6;
    const SUBA_LONG_RDIMM: usize = 8;

    const SUBI_REG_BW: usize = 8;
    const SUBI_REG_L: usize = 16;
    const SUBI_MEM_BW: usize = 12;
    const SUBI_MEM_L: usize = 20;

    const SUBQ_DREG_BW: usize = 4;
    const SUBQ_AREG_BW: usize = 8;
    const SUBQ_REG_L: usize = 8;
    const SUBQ_MEM_BW: usize = 8;
    const SUBQ_MEM_L: usize = 12;

    const SUBX_REG_BW: usize = 4;
    const SUBX_REG_L: usize = 8;
    const SUBX_MEM_BW: usize = 18;
    const SUBX_MEM_L: usize = 30;

    const SWAP: usize = 4;

    const TAS_REG: usize = 4;
    const TAS_MEM: usize = 14;

    const TRAPV_NO_TRAP: usize = 4;

    const TST_REG_BW: usize = 4;
    const TST_REG_L: usize = 4;
    const TST_MEM_BW: usize = 4;
    const TST_MEM_L: usize = 4;

    const UNLK: usize = 12;
}
