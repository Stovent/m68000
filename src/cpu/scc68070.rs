pub(crate) const EA_ARI: usize = 4;
pub(crate) const EA_ARIWPO: usize = 4;
pub(crate) const EA_ARIWPR: usize = 7;
pub(crate) const EA_ARIWD: usize = 11;
pub(crate) const EA_ARIWI8: usize = 14;
pub(crate) const EA_ABSSHORT: usize = 8;
pub(crate) const EA_ABSLONG: usize = 12;
pub(crate) const EA_PCIWD: usize = 11;
pub(crate) const EA_PCIWI8: usize = 14;
pub(crate) const EA_IMMEDIATE: usize = 4;

pub(crate) const VECTOR_RESET: usize = 43;
pub(crate) const fn vector_execution_time(vector: u8) -> usize {
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
        _ => VECTOR_RESET, // TODO: what to return with the other vectors?
    }
}

pub(crate) const ABCD_REG: usize = 10;
pub(crate) const ABCD_MEM: usize = 31;

pub(crate) const ADD_REG_BW: usize = 7;
pub(crate) const ADD_REG_L: usize = 7;
pub(crate) const ADD_REG_L_RDIMM: usize = 7;
pub(crate) const ADD_MEM_BW: usize = 11;
pub(crate) const ADD_MEM_L: usize = 15;

pub(crate) const ADDA_WORD: usize = 7;
pub(crate) const ADDA_LONG: usize = 7;
pub(crate) const ADDA_LONG_RDIMM: usize = 7;

pub(crate) const ADDI_REG_BW: usize = 14;
pub(crate) const ADDI_REG_L: usize = 18;
pub(crate) const ADDI_MEM_BW: usize = 18;
pub(crate) const ADDI_MEM_L: usize = 26;

pub(crate) const ADDQ_REG_BW: usize = 7;
pub(crate) const ADDQ_REG_L: usize = 7;
pub(crate) const ADDQ_MEM_BW: usize = 11;
pub(crate) const ADDQ_MEM_L: usize = 15;

pub(crate) const ADDX_REG_BW: usize = 7;
pub(crate) const ADDX_REG_L: usize = 7;
pub(crate) const ADDX_MEM_BW: usize = 28;
pub(crate) const ADDX_MEM_L: usize = 40;

pub(crate) const AND_REG_BW: usize = 7;
pub(crate) const AND_REG_L: usize = 7;
pub(crate) const AND_REG_L_RDIMM: usize = 7;
pub(crate) const AND_MEM_BW: usize = 11;
pub(crate) const AND_MEM_L: usize = 15;

pub(crate) const ANDI_REG_BW: usize = 14;
pub(crate) const ANDI_REG_L: usize = 18;
pub(crate) const ANDI_MEM_BW: usize = 18;
pub(crate) const ANDI_MEM_L: usize = 26;

pub(crate) const ANDICCR: usize = 14;

pub(crate) const ANDISR: usize = 14;

pub(crate) const ASM: usize = 14;

pub(crate) const ASR_COUNT: usize = 3;
pub(crate) const ASR_BW: usize = 13;
pub(crate) const ASR_L: usize = 13;

pub(crate) const BCC_BRANCH: usize = 13;
pub(crate) const BCC_NO_BRANCH_BYTE: usize = 14;
pub(crate) const BCC_NO_BRANCH_WORD: usize = 14;

pub(crate) const BCHG_DYN_REG: usize = 10;
pub(crate) const BCHG_DYN_MEM: usize = 14;
pub(crate) const BCHG_STA_REG: usize = 17;
pub(crate) const BCHG_STA_MEM: usize = 21;

pub(crate) const BCLR_DYN_REG: usize = 10;
pub(crate) const BCLR_DYN_MEM: usize = 14;
pub(crate) const BCLR_STA_REG: usize = 17;
pub(crate) const BCLR_STA_MEM: usize = 21;

pub(crate) const BRA_BYTE: usize = 13;
pub(crate) const BRA_WORD: usize = 14;

pub(crate) const BSET_DYN_REG: usize = 10;
pub(crate) const BSET_DYN_MEM: usize = 14;
pub(crate) const BSET_STA_REG: usize = 17;
pub(crate) const BSET_STA_MEM: usize = 21;

pub(crate) const BSR_BYTE: usize = 17;
pub(crate) const BSR_WORD: usize = 22;

pub(crate) const BTST_DYN_REG: usize = 7;
pub(crate) const BTST_DYN_MEM: usize = 7;
pub(crate) const BTST_STA_REG: usize = 14;
pub(crate) const BTST_STA_MEM: usize = 14;

pub(crate) const CHK_NO_TRAP: usize = 19;

pub(crate) const CLR_REG_BW: usize = 7;
pub(crate) const CLR_REG_L: usize = 7;
pub(crate) const CLR_MEM_BW: usize = 7; // Subtract one read cycle from effective address calculation.
pub(crate) const CLR_MEM_L: usize = 7; // Subtract two read cycles from effective address calculation.

pub(crate) const CMP_BW: usize = 7;
pub(crate) const CMP_L: usize = 7;

pub(crate) const CMPA: usize = 7;

pub(crate) const CMPI_REG_BW: usize = 14;
pub(crate) const CMPI_REG_L: usize = 18;
pub(crate) const CMPI_MEM_BW: usize = 14;
pub(crate) const CMPI_MEM_L: usize = 18;

pub(crate) const CMPM_BW: usize = 18;
pub(crate) const CMPM_L: usize = 26;

pub(crate) const DBCC_TRUE: usize = 14;
pub(crate) const DBCC_FALSE_BRANCH: usize = 17;
pub(crate) const DBCC_FALSE_NO_BRANCH: usize = 17;

pub(crate) const DIVS: usize = 169;

pub(crate) const DIVU: usize = 130;

pub(crate) const EOR_REG_BW: usize = 7;
pub(crate) const EOR_REG_L: usize = 7;
pub(crate) const EOR_MEM_BW: usize = 11;
pub(crate) const EOR_MEM_L: usize = 15;

pub(crate) const EORI_REG_BW: usize = 14;
pub(crate) const EORI_REG_L: usize = 18;
pub(crate) const EORI_MEM_BW: usize = 18;
pub(crate) const EORI_MEM_L: usize = 26;

pub(crate) const EORICCR: usize = 14;

pub(crate) const EORISR: usize = 14;

pub(crate) const EXG: usize = 13;

pub(crate) const EXT: usize = 7;

pub(crate) const JMP_ARI: usize = 7;
pub(crate) const JMP_ARIWD: usize = 14;
pub(crate) const JMP_ARIWI8: usize = 17;
pub(crate) const JMP_ABSSHORT: usize = 14;
pub(crate) const JMP_ABSLONG: usize = 18;
pub(crate) const JMP_PCIWD: usize = 14;
pub(crate) const JMP_PCIWI8: usize = 17;

pub(crate) const JSR_ARI: usize = 18;
pub(crate) const JSR_ARIWD: usize = 25;
pub(crate) const JSR_ARIWI8: usize = 28;
pub(crate) const JSR_ABSSHORT: usize = 25;
pub(crate) const JSR_ABSLONG: usize = 29;
pub(crate) const JSR_PCIWD: usize = 25;
pub(crate) const JSR_PCIWI8: usize = 28;

pub(crate) const LEA_ARI: usize = 7;
pub(crate) const LEA_ARIWD: usize = 14;
pub(crate) const LEA_ARIWI8: usize = 17;
pub(crate) const LEA_ABSSHORT: usize = 14;
pub(crate) const LEA_ABSLONG: usize = 18;
pub(crate) const LEA_PCIWD: usize = 14;
pub(crate) const LEA_PCIWI8: usize = 17;

pub(crate) const LINK: usize = 25;

pub(crate) const LSM: usize = 14;

pub(crate) const LSR_COUNT: usize = 3;
pub(crate) const LSR_BW: usize = 13;
pub(crate) const LSR_L: usize = 13;

pub(crate) const MOVE_DST_ARIWPR: usize = 7;
pub(crate) const MOVE_OTHER: usize = 7;

pub(crate) const MOVEA: usize = 7;

pub(crate) const MOVECCR: usize = 10;

pub(crate) const MOVEFSR_REG: usize = 7;
pub(crate) const MOVEFSR_MEM: usize = 11;

pub(crate) const MOVESR: usize = 10;

pub(crate) const MOVEUSP: usize = 7;

pub(crate) const MOVEM_WORD: usize = 7;
pub(crate) const MOVEM_LONG: usize = 11;
pub(crate) const MOVEM_MTR: usize = 3;
pub(crate) const MOVEM_ARI: usize = 23; // R -> M, do +3 for M -> R.
pub(crate) const MOVEM_ARIWPO: usize = 23;
pub(crate) const MOVEM_ARIWPR: usize = 23;
pub(crate) const MOVEM_ARIWD: usize = 27;
pub(crate) const MOVEM_ARIWI8: usize = 30;
pub(crate) const MOVEM_ABSSHORT: usize = 27;
pub(crate) const MOVEM_ABSLONG: usize = 31;
pub(crate) const MOVEM_PCIWD: usize = 27;
pub(crate) const MOVEM_PCIWI8: usize = 30;

pub(crate) const MOVEP_RTM_WORD: usize = 25;
pub(crate) const MOVEP_RTM_LONG: usize = 39;
pub(crate) const MOVEP_MTR_WORD: usize = 22;
pub(crate) const MOVEP_MTR_LONG: usize = 36;

pub(crate) const MOVEQ: usize = 7;

pub(crate) const MULS: usize = 76;

pub(crate) const MULU: usize = 76;

pub(crate) const NBCD_REG: usize = 10;
pub(crate) const NBCD_MEM: usize = 14;

pub(crate) const NEG_REG_BW: usize = 7;
pub(crate) const NEG_REG_L: usize = 7;
pub(crate) const NEG_MEM_BW: usize = 11;
pub(crate) const NEG_MEM_L: usize = 15;

pub(crate) const NEGX_REG_BW: usize = 7;
pub(crate) const NEGX_REG_L: usize = 7;
pub(crate) const NEGX_MEM_BW: usize = 11;
pub(crate) const NEGX_MEM_L: usize = 15;

pub(crate) const NOP: usize = 7;

pub(crate) const NOT_REG_BW: usize = 7;
pub(crate) const NOT_REG_L: usize = 7;
pub(crate) const NOT_MEM_BW: usize = 11;
pub(crate) const NOT_MEM_L: usize = 15;

pub(crate) const OR_REG_BW: usize = 7;
pub(crate) const OR_REG_L: usize = 7;
pub(crate) const OR_REG_L_RDIMM: usize = 7;
pub(crate) const OR_MEM_BW: usize = 11;
pub(crate) const OR_MEM_L: usize = 15;

pub(crate) const ORI_REG_BW: usize = 14;
pub(crate) const ORI_REG_L: usize = 18;
pub(crate) const ORI_MEM_BW: usize = 18;
pub(crate) const ORI_MEM_L: usize = 26;

pub(crate) const ORICCR: usize = 14;

pub(crate) const ORISR: usize = 14;

pub(crate) const PEA_ARI: usize = 18;
pub(crate) const PEA_ARIWD: usize = 25;
pub(crate) const PEA_ARIWI8: usize = 28;
pub(crate) const PEA_ABSSHORT: usize = 25;
pub(crate) const PEA_ABSLONG: usize = 29;
pub(crate) const PEA_PCIWD: usize = 25;
pub(crate) const PEA_PCIWI8: usize = 28;

pub(crate) const RESET: usize = 154;

pub(crate) const ROM: usize = 14;

pub(crate) const ROR_COUNT: usize = 3;
pub(crate) const ROR_BW: usize = 13;
pub(crate) const ROR_L: usize = 13;

pub(crate) const ROXM: usize = 14;

pub(crate) const ROXR_COUNT: usize = 3;
pub(crate) const ROXR_BW: usize = 13;
pub(crate) const ROXR_L: usize = 13;

pub(crate) const RTE: usize = 39;

pub(crate) const RTR: usize = 22;

pub(crate) const RTS: usize = 15;

pub(crate) const SBCD_REG: usize = 10;
pub(crate) const SBCD_MEM: usize = 31;

pub(crate) const SCC_REG_FALSE: usize = 13;
pub(crate) const SCC_REG_TRUE: usize = 13;
pub(crate) const SCC_MEM_FALSE: usize = 17;
pub(crate) const SCC_MEM_TRUE: usize = 14;

pub(crate) const STOP: usize = 13;

pub(crate) const SUB_REG_BW: usize = 7;
pub(crate) const SUB_REG_L: usize = 7;
pub(crate) const SUB_REG_L_RDIMM: usize = 7;
pub(crate) const SUB_MEM_BW: usize = 11;
pub(crate) const SUB_MEM_L: usize = 15;

pub(crate) const SUBA_WORD: usize = 7;
pub(crate) const SUBA_LONG: usize = 7;
pub(crate) const SUBA_LONG_RDIMM: usize = 7;

pub(crate) const SUBI_REG_BW: usize = 14;
pub(crate) const SUBI_REG_L: usize = 18;
pub(crate) const SUBI_MEM_BW: usize = 18;
pub(crate) const SUBI_MEM_L: usize = 26;

pub(crate) const SUBQ_DREG_BW: usize = 7;
pub(crate) const SUBQ_AREG_BW: usize = 7;
pub(crate) const SUBQ_REG_L: usize = 7;
pub(crate) const SUBQ_MEM_BW: usize = 11;
pub(crate) const SUBQ_MEM_L: usize = 15;

pub(crate) const SUBX_REG_BW: usize = 7;
pub(crate) const SUBX_REG_L: usize = 7;
pub(crate) const SUBX_MEM_BW: usize = 28;
pub(crate) const SUBX_MEM_L: usize = 40;

pub(crate) const SWAP: usize = 7;

pub(crate) const TAS_REG: usize = 10;
pub(crate) const TAS_MEM: usize = 11; // Subtract one read cycle from effective address calculation.

pub(crate) const TRAPV_NO_TRAP: usize = 10;

pub(crate) const TST_REG_BW: usize = 7;
pub(crate) const TST_REG_L: usize = 7;
pub(crate) const TST_MEM_BW: usize = 7;
pub(crate) const TST_MEM_L: usize = 7;

pub(crate) const UNLK: usize = 15;
