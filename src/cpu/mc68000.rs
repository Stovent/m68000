pub(crate) const EA_ARI: usize = 4;
pub(crate) const EA_ARIWPO: usize = 4;
pub(crate) const EA_ARIWPR: usize = 6;
pub(crate) const EA_ARIWD: usize = 8;
pub(crate) const EA_ARIWI8: usize = 10;
pub(crate) const EA_ABSSHORT: usize = 8;
pub(crate) const EA_ABSLONG: usize = 12;
pub(crate) const EA_PCIWD: usize = 8;
pub(crate) const EA_PCIWI8: usize = 10;
pub(crate) const EA_IMMEDIATE: usize = 4;

pub(crate) const VECTOR_RESET: usize = 40;
pub(crate) const fn vector_execution_time(vector: u8) -> usize {
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
        _ => VECTOR_RESET, // TODO: what to return with the other vectors?
    }
}

pub(crate) const ABCD_REG: usize = 6;
pub(crate) const ABCD_MEM: usize = 18;

pub(crate) const ADD_REG_BW: usize = 4;
pub(crate) const ADD_REG_L: usize = 6;
pub(crate) const ADD_REG_L_RDIMM: usize = 8;
pub(crate) const ADD_MEM_BW: usize = 8;
pub(crate) const ADD_MEM_L: usize = 12;

pub(crate) const ADDA_WORD: usize = 8;
pub(crate) const ADDA_LONG: usize = 6;
pub(crate) const ADDA_LONG_RDIMM: usize = 8;

pub(crate) const ADDI_REG_BW: usize = 8;
pub(crate) const ADDI_REG_L: usize = 16;
pub(crate) const ADDI_MEM_BW: usize = 12;
pub(crate) const ADDI_MEM_L: usize = 20;

pub(crate) const ADDQ_REG_BW: usize = 4;
pub(crate) const ADDQ_REG_L: usize = 8;
pub(crate) const ADDQ_MEM_BW: usize = 8;
pub(crate) const ADDQ_MEM_L: usize = 12;

pub(crate) const ADDX_REG_BW: usize = 4;
pub(crate) const ADDX_REG_L: usize = 8;
pub(crate) const ADDX_MEM_BW: usize = 18;
pub(crate) const ADDX_MEM_L: usize = 30;

pub(crate) const AND_REG_BW: usize = 4;
pub(crate) const AND_REG_L: usize = 6;
pub(crate) const AND_REG_L_RDIMM: usize = 8;
pub(crate) const AND_MEM_BW: usize = 8;
pub(crate) const AND_MEM_L: usize = 12;

pub(crate) const ANDI_REG_BW: usize = 8;
pub(crate) const ANDI_REG_L: usize = 14;
pub(crate) const ANDI_MEM_BW: usize = 12;
pub(crate) const ANDI_MEM_L: usize = 20;

pub(crate) const ANDICCR: usize = 20;

pub(crate) const ANDISR: usize = 20;

pub(crate) const ASM: usize = 8;

pub(crate) const ASR_COUNT: usize = 2;
pub(crate) const ASR_BW: usize = 6;
pub(crate) const ASR_L: usize = 8;

pub(crate) const BCC_BRANCH: usize = 10;
pub(crate) const BCC_NO_BRANCH_BYTE: usize = 8;
pub(crate) const BCC_NO_BRANCH_WORD: usize = 12;

pub(crate) const BCHG_DYN_REG: usize = 8;
pub(crate) const BCHG_DYN_MEM: usize = 8;
pub(crate) const BCHG_STA_REG: usize = 12;
pub(crate) const BCHG_STA_MEM: usize = 12;

pub(crate) const BCLR_DYN_REG: usize = 10;
pub(crate) const BCLR_DYN_MEM: usize = 8;
pub(crate) const BCLR_STA_REG: usize = 14;
pub(crate) const BCLR_STA_MEM: usize = 12;

pub(crate) const BRA_BYTE: usize = 10;
pub(crate) const BRA_WORD: usize = 10;

pub(crate) const BSET_DYN_REG: usize = 8;
pub(crate) const BSET_DYN_MEM: usize = 8;
pub(crate) const BSET_STA_REG: usize = 12;
pub(crate) const BSET_STA_MEM: usize = 12;

pub(crate) const BSR_BYTE: usize = 18;
pub(crate) const BSR_WORD: usize = 18;

pub(crate) const BTST_DYN_REG: usize = 6;
pub(crate) const BTST_DYN_MEM: usize = 4;
pub(crate) const BTST_STA_REG: usize = 10;
pub(crate) const BTST_STA_MEM: usize = 8;

pub(crate) const CHK_NO_TRAP: usize = 10;

pub(crate) const CLR_REG_BW: usize = 4;
pub(crate) const CLR_REG_L: usize = 6;
pub(crate) const CLR_MEM_BW: usize = 8;
pub(crate) const CLR_MEM_L: usize = 12;

pub(crate) const CMP_BW: usize = 4;
pub(crate) const CMP_L: usize = 6;

pub(crate) const CMPA: usize = 6;

pub(crate) const CMPI_REG_BW: usize = 8;
pub(crate) const CMPI_REG_L: usize = 14;
pub(crate) const CMPI_MEM_BW: usize = 8;
pub(crate) const CMPI_MEM_L: usize = 12;

pub(crate) const CMPM_BW: usize = 12;
pub(crate) const CMPM_L: usize = 20;

pub(crate) const DBCC_TRUE: usize = 12;
pub(crate) const DBCC_FALSE_BRANCH: usize = 10;
pub(crate) const DBCC_FALSE_NO_BRANCH: usize = 14;

pub(crate) const DIVS: usize = 158;
pub(crate) const DIVU: usize = 140;

pub(crate) const EOR_REG_BW: usize = 4;
pub(crate) const EOR_REG_L: usize = 8;
pub(crate) const EOR_MEM_BW: usize = 8;
pub(crate) const EOR_MEM_L: usize = 12;

pub(crate) const EORI_REG_BW: usize = 8;
pub(crate) const EORI_REG_L: usize = 16;
pub(crate) const EORI_MEM_BW: usize = 12;
pub(crate) const EORI_MEM_L: usize = 20;

pub(crate) const EORICCR: usize = 20;

pub(crate) const EORISR: usize = 20;

pub(crate) const EXG: usize = 6;

pub(crate) const EXT: usize = 4;

pub(crate) const JMP_ARI: usize = 8;
pub(crate) const JMP_ARIWD: usize = 10;
pub(crate) const JMP_ARIWI8: usize = 14;
pub(crate) const JMP_ABSSHORT: usize = 10;
pub(crate) const JMP_ABSLONG: usize = 12;
pub(crate) const JMP_PCIWD: usize = 10;
pub(crate) const JMP_PCIWI8: usize = 14;

pub(crate) const JSR_ARI: usize = 16;
pub(crate) const JSR_ARIWD: usize = 18;
pub(crate) const JSR_ARIWI8: usize = 22;
pub(crate) const JSR_ABSSHORT: usize = 18;
pub(crate) const JSR_ABSLONG: usize = 20;
pub(crate) const JSR_PCIWD: usize = 18;
pub(crate) const JSR_PCIWI8: usize = 22;

pub(crate) const LEA_ARI: usize = 4;
pub(crate) const LEA_ARIWD: usize = 8;
pub(crate) const LEA_ARIWI8: usize = 12;
pub(crate) const LEA_ABSSHORT: usize = 8;
pub(crate) const LEA_ABSLONG: usize = 12;
pub(crate) const LEA_PCIWD: usize = 8;
pub(crate) const LEA_PCIWI8: usize = 12;

pub(crate) const LINK: usize = 16;

pub(crate) const LSM: usize = 8;

pub(crate) const LSR_COUNT: usize = 2;
pub(crate) const LSR_BW: usize = 6;
pub(crate) const LSR_L: usize = 8;

pub(crate) const MOVE_DST_ARIWPR: usize = 2;
pub(crate) const MOVE_OTHER: usize = 4;

pub(crate) const MOVEA: usize = 4;

pub(crate) const MOVECCR: usize = 12;

pub(crate) const MOVEFSR_REG: usize = 6;
pub(crate) const MOVEFSR_MEM: usize = 8;

pub(crate) const MOVESR: usize = 12;

pub(crate) const MOVEUSP: usize = 4;

pub(crate) const MOVEM_WORD: usize = 4;
pub(crate) const MOVEM_LONG: usize = 8;
pub(crate) const MOVEM_MTR: usize = 4;
pub(crate) const MOVEM_ARI: usize = 8; // R -> M, do +4 for M -> R.
pub(crate) const MOVEM_ARIWPO: usize = 8;
pub(crate) const MOVEM_ARIWPR: usize = 8;
pub(crate) const MOVEM_ARIWD: usize = 12;
pub(crate) const MOVEM_ARIWI8: usize = 14;
pub(crate) const MOVEM_ABSSHORT: usize = 12;
pub(crate) const MOVEM_ABSLONG: usize = 16;
pub(crate) const MOVEM_PCIWD: usize = 12;
pub(crate) const MOVEM_PCIWI8: usize = 14;

pub(crate) const MOVEP_RTM_WORD: usize = 16;
pub(crate) const MOVEP_RTM_LONG: usize = 24;
pub(crate) const MOVEP_MTR_WORD: usize = 16;
pub(crate) const MOVEP_MTR_LONG: usize = 24;

pub(crate) const MOVEQ: usize = 4;

pub(crate) const MULS: usize = 70;

pub(crate) const MULU: usize = 70;

pub(crate) const NBCD_REG: usize = 6;
pub(crate) const NBCD_MEM: usize = 8;

pub(crate) const NEG_REG_BW: usize = 4;
pub(crate) const NEG_REG_L: usize = 6;
pub(crate) const NEG_MEM_BW: usize = 8;
pub(crate) const NEG_MEM_L: usize = 12;

pub(crate) const NEGX_REG_BW: usize = 4;
pub(crate) const NEGX_REG_L: usize = 6;
pub(crate) const NEGX_MEM_BW: usize = 8;
pub(crate) const NEGX_MEM_L: usize = 12;

pub(crate) const NOP: usize = 4;

pub(crate) const NOT_REG_BW: usize = 4;
pub(crate) const NOT_REG_L: usize = 6;
pub(crate) const NOT_MEM_BW: usize = 8;
pub(crate) const NOT_MEM_L: usize = 12;

pub(crate) const OR_REG_BW: usize = 4;
pub(crate) const OR_REG_L: usize = 6;
pub(crate) const OR_REG_L_RDIMM: usize = 8;
pub(crate) const OR_MEM_BW: usize = 8;
pub(crate) const OR_MEM_L: usize = 12;

pub(crate) const ORI_REG_BW: usize = 8;
pub(crate) const ORI_REG_L: usize = 16;
pub(crate) const ORI_MEM_BW: usize = 12;
pub(crate) const ORI_MEM_L: usize = 20;

pub(crate) const ORICCR: usize = 20;

pub(crate) const ORISR: usize = 20;

pub(crate) const PEA_ARI: usize = 12;
pub(crate) const PEA_ARIWD: usize = 16;
pub(crate) const PEA_ARIWI8: usize = 20;
pub(crate) const PEA_ABSSHORT: usize = 16;
pub(crate) const PEA_ABSLONG: usize = 20;
pub(crate) const PEA_PCIWD: usize = 16;
pub(crate) const PEA_PCIWI8: usize = 20;

pub(crate) const RESET: usize = 132;

pub(crate) const ROM: usize = 8;

pub(crate) const ROR_COUNT: usize = 2;
pub(crate) const ROR_BW: usize = 6;
pub(crate) const ROR_L: usize = 8;

pub(crate) const ROXM: usize = 8;

pub(crate) const ROXR_COUNT: usize = 2;
pub(crate) const ROXR_BW: usize = 6;
pub(crate) const ROXR_L: usize = 8;

pub(crate) const RTE: usize = 20;

pub(crate) const RTR: usize = 20;

pub(crate) const RTS: usize = 16;

pub(crate) const SBCD_REG: usize = 6;
pub(crate) const SBCD_MEM: usize = 18;

pub(crate) const SCC_REG_FALSE: usize = 4;
pub(crate) const SCC_REG_TRUE: usize = 6;
pub(crate) const SCC_MEM_FALSE: usize = 8;
pub(crate) const SCC_MEM_TRUE: usize = 8;

pub(crate) const STOP: usize = 4;

pub(crate) const SUB_REG_BW: usize = 4;
pub(crate) const SUB_REG_L: usize = 6;
pub(crate) const SUB_REG_L_RDIMM: usize = 8;
pub(crate) const SUB_MEM_BW: usize = 8;
pub(crate) const SUB_MEM_L: usize = 12;

pub(crate) const SUBA_WORD: usize = 8;
pub(crate) const SUBA_LONG: usize = 6;
pub(crate) const SUBA_LONG_RDIMM: usize = 8;

pub(crate) const SUBI_REG_BW: usize = 8;
pub(crate) const SUBI_REG_L: usize = 16;
pub(crate) const SUBI_MEM_BW: usize = 12;
pub(crate) const SUBI_MEM_L: usize = 20;

pub(crate) const SUBQ_DREG_BW: usize = 4;
pub(crate) const SUBQ_AREG_BW: usize = 8;
pub(crate) const SUBQ_REG_L: usize = 8;
pub(crate) const SUBQ_MEM_BW: usize = 8;
pub(crate) const SUBQ_MEM_L: usize = 12;

pub(crate) const SUBX_REG_BW: usize = 4;
pub(crate) const SUBX_REG_L: usize = 8;
pub(crate) const SUBX_MEM_BW: usize = 18;
pub(crate) const SUBX_MEM_L: usize = 30;

pub(crate) const SWAP: usize = 4;

pub(crate) const TAS_REG: usize = 4;
pub(crate) const TAS_MEM: usize = 14;

pub(crate) const TRAPV_NO_TRAP: usize = 4;

pub(crate) const TST_REG_BW: usize = 4;
pub(crate) const TST_REG_L: usize = 4;
pub(crate) const TST_MEM_BW: usize = 4;
pub(crate) const TST_MEM_L: usize = 4;

pub(crate) const UNLK: usize = 12;
