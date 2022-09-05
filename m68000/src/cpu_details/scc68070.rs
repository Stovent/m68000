use crate::cpu_details::{CpuDetails, StackFormat};

/// The CPU details of the SCC68070 microcontroller.
#[derive(Clone, Copy, Debug, Default)]
pub struct Scc68070;

impl CpuDetails for Scc68070 {
    const STACK_FORMAT: StackFormat = StackFormat::SCC68070;

    const VECTOR_RESET: usize = 43;
    fn vector_execution_time(vector: u8) -> usize {
        match vector {
            2 => 158, // Access Error
            3 => 158, // Address Error
            4 => 55, // Illegal
            5 => 64, // Zero Divide
            6 => 64, // Chk
            7 => 55, // Trapv
            8 => 55, // Privilege Violation
            9 => 55, // Trace
            24..=31 => 65, // Interrupt
            32..=47 => 52, // Trap
            _ => Self::VECTOR_RESET, // TODO: what to return with the other vectors?
        }
    }

    const EA_ARI: usize = 4;
    const EA_ARIWPO: usize = 4;
    const EA_ARIWPR: usize = 7;
    const EA_ARIWD: usize = 11;
    const EA_ARIWI8: usize = 14;
    const EA_ABSSHORT: usize = 8;
    const EA_ABSLONG: usize = 12;
    const EA_PCIWD: usize = 11;
    const EA_PCIWI8: usize = 14;
    const EA_IMMEDIATE: usize = 4;

    const ABCD_REG: usize = 10;
    const ABCD_MEM: usize = 31;

    const ADD_REG_BW: usize = 7;
    const ADD_REG_L: usize = 7;
    const ADD_REG_L_RDIMM: usize = 7;
    const ADD_MEM_BW: usize = 11;
    const ADD_MEM_L: usize = 15;

    const ADDA_WORD: usize = 7;
    const ADDA_LONG: usize = 7;
    const ADDA_LONG_RDIMM: usize = 7;

    const ADDI_REG_BW: usize = 14;
    const ADDI_REG_L: usize = 18;
    const ADDI_MEM_BW: usize = 18;
    const ADDI_MEM_L: usize = 26;

    const ADDQ_REG_BW: usize = 7;
    const ADDQ_REG_L: usize = 7;
    const ADDQ_MEM_BW: usize = 11;
    const ADDQ_MEM_L: usize = 15;

    const ADDX_REG_BW: usize = 7;
    const ADDX_REG_L: usize = 7;
    const ADDX_MEM_BW: usize = 28;
    const ADDX_MEM_L: usize = 40;

    const AND_REG_BW: usize = 7;
    const AND_REG_L: usize = 7;
    const AND_REG_L_RDIMM: usize = 7;
    const AND_MEM_BW: usize = 11;
    const AND_MEM_L: usize = 15;

    const ANDI_REG_BW: usize = 14;
    const ANDI_REG_L: usize = 18;
    const ANDI_MEM_BW: usize = 18;
    const ANDI_MEM_L: usize = 26;

    const ANDICCR: usize = 14;

    const ANDISR: usize = 14;

    const ASM: usize = 14;

    const ASR_COUNT: usize = 3;
    const ASR_BW: usize = 13;
    const ASR_L: usize = 13;

    const BCC_BRANCH: usize = 13;
    const BCC_NO_BRANCH_BYTE: usize = 14;
    const BCC_NO_BRANCH_WORD: usize = 14;

    const BCHG_DYN_REG: usize = 10;
    const BCHG_DYN_MEM: usize = 14;
    const BCHG_STA_REG: usize = 17;
    const BCHG_STA_MEM: usize = 21;

    const BCLR_DYN_REG: usize = 10;
    const BCLR_DYN_MEM: usize = 14;
    const BCLR_STA_REG: usize = 17;
    const BCLR_STA_MEM: usize = 21;

    const BRA_BYTE: usize = 13;
    const BRA_WORD: usize = 14;

    const BSET_DYN_REG: usize = 10;
    const BSET_DYN_MEM: usize = 14;
    const BSET_STA_REG: usize = 17;
    const BSET_STA_MEM: usize = 21;

    const BSR_BYTE: usize = 17;
    const BSR_WORD: usize = 22;

    const BTST_DYN_REG: usize = 7;
    const BTST_DYN_MEM: usize = 7;
    const BTST_STA_REG: usize = 14;
    const BTST_STA_MEM: usize = 14;

    const CHK_NO_TRAP: usize = 19;

    const CLR_REG_BW: usize = 7;
    const CLR_REG_L: usize = 7;
    const CLR_MEM_BW: usize = 7; // Subtract one read cycle from effective address calculation.
    const CLR_MEM_L: usize = 7; // Subtract two read cycles from effective address calculation.

    const CMP_BW: usize = 7;
    const CMP_L: usize = 7;

    const CMPA: usize = 7;

    const CMPI_REG_BW: usize = 14;
    const CMPI_REG_L: usize = 18;
    const CMPI_MEM_BW: usize = 14;
    const CMPI_MEM_L: usize = 18;

    const CMPM_BW: usize = 18;
    const CMPM_L: usize = 26;

    const DBCC_TRUE: usize = 14;
    const DBCC_FALSE_BRANCH: usize = 17;
    const DBCC_FALSE_NO_BRANCH: usize = 17;

    const DIVS: usize = 169;

    const DIVU: usize = 130;

    const EOR_REG_BW: usize = 7;
    const EOR_REG_L: usize = 7;
    const EOR_MEM_BW: usize = 11;
    const EOR_MEM_L: usize = 15;

    const EORI_REG_BW: usize = 14;
    const EORI_REG_L: usize = 18;
    const EORI_MEM_BW: usize = 18;
    const EORI_MEM_L: usize = 26;

    const EORICCR: usize = 14;

    const EORISR: usize = 14;

    const EXG: usize = 13;

    const EXT: usize = 7;

    const JMP_ARI: usize = 7;
    const JMP_ARIWD: usize = 14;
    const JMP_ARIWI8: usize = 17;
    const JMP_ABSSHORT: usize = 14;
    const JMP_ABSLONG: usize = 18;
    const JMP_PCIWD: usize = 14;
    const JMP_PCIWI8: usize = 17;

    const JSR_ARI: usize = 18;
    const JSR_ARIWD: usize = 25;
    const JSR_ARIWI8: usize = 28;
    const JSR_ABSSHORT: usize = 25;
    const JSR_ABSLONG: usize = 29;
    const JSR_PCIWD: usize = 25;
    const JSR_PCIWI8: usize = 28;

    const LEA_ARI: usize = 7;
    const LEA_ARIWD: usize = 14;
    const LEA_ARIWI8: usize = 17;
    const LEA_ABSSHORT: usize = 14;
    const LEA_ABSLONG: usize = 18;
    const LEA_PCIWD: usize = 14;
    const LEA_PCIWI8: usize = 17;

    const LINK: usize = 25;

    const LSM: usize = 14;

    const LSR_COUNT: usize = 3;
    const LSR_BW: usize = 13;
    const LSR_L: usize = 13;

    const MOVE_DST_ARIWPR: usize = 7;
    const MOVE_OTHER: usize = 7;

    const MOVEA: usize = 7;

    const MOVECCR: usize = 10;

    const MOVEFSR_REG: usize = 7;
    const MOVEFSR_MEM: usize = 11;

    const MOVESR: usize = 10;

    const MOVEUSP: usize = 7;

    const MOVEM_WORD: usize = 7;
    const MOVEM_LONG: usize = 11;
    const MOVEM_MTR: usize = 3;
    const MOVEM_ARI: usize = 23; // R -> M, do +3 for M -> R.
    const MOVEM_ARIWPO: usize = 23;
    const MOVEM_ARIWPR: usize = 23;
    const MOVEM_ARIWD: usize = 27;
    const MOVEM_ARIWI8: usize = 30;
    const MOVEM_ABSSHORT: usize = 27;
    const MOVEM_ABSLONG: usize = 31;
    const MOVEM_PCIWD: usize = 27;
    const MOVEM_PCIWI8: usize = 30;

    const MOVEP_RTM_WORD: usize = 25;
    const MOVEP_RTM_LONG: usize = 39;
    const MOVEP_MTR_WORD: usize = 22;
    const MOVEP_MTR_LONG: usize = 36;

    const MOVEQ: usize = 7;

    const MULS: usize = 76;

    const MULU: usize = 76;

    const NBCD_REG: usize = 10;
    const NBCD_MEM: usize = 14;

    const NEG_REG_BW: usize = 7;
    const NEG_REG_L: usize = 7;
    const NEG_MEM_BW: usize = 11;
    const NEG_MEM_L: usize = 15;

    const NEGX_REG_BW: usize = 7;
    const NEGX_REG_L: usize = 7;
    const NEGX_MEM_BW: usize = 11;
    const NEGX_MEM_L: usize = 15;

    const NOP: usize = 7;

    const NOT_REG_BW: usize = 7;
    const NOT_REG_L: usize = 7;
    const NOT_MEM_BW: usize = 11;
    const NOT_MEM_L: usize = 15;

    const OR_REG_BW: usize = 7;
    const OR_REG_L: usize = 7;
    const OR_REG_L_RDIMM: usize = 7;
    const OR_MEM_BW: usize = 11;
    const OR_MEM_L: usize = 15;

    const ORI_REG_BW: usize = 14;
    const ORI_REG_L: usize = 18;
    const ORI_MEM_BW: usize = 18;
    const ORI_MEM_L: usize = 26;

    const ORICCR: usize = 14;

    const ORISR: usize = 14;

    const PEA_ARI: usize = 18;
    const PEA_ARIWD: usize = 25;
    const PEA_ARIWI8: usize = 28;
    const PEA_ABSSHORT: usize = 25;
    const PEA_ABSLONG: usize = 29;
    const PEA_PCIWD: usize = 25;
    const PEA_PCIWI8: usize = 28;

    const RESET: usize = 154;

    const ROM: usize = 14;

    const ROR_COUNT: usize = 3;
    const ROR_BW: usize = 13;
    const ROR_L: usize = 13;

    const ROXM: usize = 14;

    const ROXR_COUNT: usize = 3;
    const ROXR_BW: usize = 13;
    const ROXR_L: usize = 13;

    const RTE: usize = 39;

    const RTR: usize = 22;

    const RTS: usize = 15;

    const SBCD_REG: usize = 10;
    const SBCD_MEM: usize = 31;

    const SCC_REG_FALSE: usize = 13;
    const SCC_REG_TRUE: usize = 13;
    const SCC_MEM_FALSE: usize = 17;
    const SCC_MEM_TRUE: usize = 14;

    const STOP: usize = 13;

    const SUB_REG_BW: usize = 7;
    const SUB_REG_L: usize = 7;
    const SUB_REG_L_RDIMM: usize = 7;
    const SUB_MEM_BW: usize = 11;
    const SUB_MEM_L: usize = 15;

    const SUBA_WORD: usize = 7;
    const SUBA_LONG: usize = 7;
    const SUBA_LONG_RDIMM: usize = 7;

    const SUBI_REG_BW: usize = 14;
    const SUBI_REG_L: usize = 18;
    const SUBI_MEM_BW: usize = 18;
    const SUBI_MEM_L: usize = 26;

    const SUBQ_DREG_BW: usize = 7;
    const SUBQ_AREG_BW: usize = 7;
    const SUBQ_REG_L: usize = 7;
    const SUBQ_MEM_BW: usize = 11;
    const SUBQ_MEM_L: usize = 15;

    const SUBX_REG_BW: usize = 7;
    const SUBX_REG_L: usize = 7;
    const SUBX_MEM_BW: usize = 28;
    const SUBX_MEM_L: usize = 40;

    const SWAP: usize = 7;

    const TAS_REG: usize = 10;
    const TAS_MEM: usize = 11; // Subtract one read cycle from effective address calculation.

    const TRAPV_NO_TRAP: usize = 10;

    const TST_REG_BW: usize = 7;
    const TST_REG_L: usize = 7;
    const TST_MEM_BW: usize = 7;
    const TST_MEM_L: usize = 7;

    const UNLK: usize = 15;
}
