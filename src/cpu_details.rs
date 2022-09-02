//! Trait that defines the instruction execution times and exception stack frame of the emulated CPU.

mod mc68000;
mod scc68070;

pub use mc68000::Mc68000;
pub use scc68070::Scc68070;

/// The emulated stack formats.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum StackFormat {
    MC68000,
    SCC68070,
}

/// Low level details of the emulated CPU.
///
/// m68000 emulates the ISA, but CPU implementations may have different instruction execution timings and exceptions processing.
/// The [M68000](crate::M68000) structure is generic of an instance of this trait, and m68000 takes all the specific details
/// from this instance so it can behave as specified.
///
/// To implement your own details, implement this trait on an empty structure and fill each constant.
/// See the documentation of each constant for more details.
/// - `STACK_FORMAT` is the stack format to use.
/// - `EA_*` is the calculation time of each addressing mode for the byte and word sizes.
/// For long size m68000 automatically adds 4 to these values.
/// - `VECTOR_RESET` is the time the CPU takes to reset itself (RESET vector 0).
/// - [`vector_execution_time`](CpuDetails::vector_execution_time) returns the time it takes to process the given exception vector.

pub trait CpuDetails : Default {
    /// The stack format to use.
    const STACK_FORMAT: StackFormat;

    /// Calculation time of the Address Register Indirect addressing mode in byte/word size.
    const EA_ARI: usize;
    /// Calculation time of the Address Register Indirect With POst increment addressing mode in byte/word size.
    const EA_ARIWPO: usize;
    /// Calculation time of the Address Register Indirect With PRe decrement addressing mode in byte/word size.
    const EA_ARIWPR: usize;
    /// Calculation time of the Address Register Indirect With Displacement addressing mode in byte/word size.
    const EA_ARIWD: usize;
    /// Calculation time of the Address Register Indirect With Index addressing mode in byte/word size.
    const EA_ARIWI8: usize;
    /// Calculation time of the Absolute Short addressing mode in byte/word size.
    const EA_ABSSHORT: usize;
    /// Calculation time of the Absolute Long addressing mode in byte/word size.
    const EA_ABSLONG: usize;
    /// Calculation time of the Program Counter Indirect With Displacement addressing mode in byte/word size.
    const EA_PCIWD: usize;
    /// Calculation time of the Program Counter Indirect With Index addressing mode in byte/word size.
    const EA_PCIWI8: usize;
    /// Calculation time of the Immediate addressing mode in byte/word size.
    const EA_IMMEDIATE: usize;

    const ABCD_REG: usize;
    const ABCD_MEM: usize;

    const ADD_REG_BW: usize;
    const ADD_REG_L: usize;
    const ADD_REG_L_RDIMM: usize;
    const ADD_MEM_BW: usize;
    const ADD_MEM_L: usize;

    const ADDA_WORD: usize;
    const ADDA_LONG: usize;
    const ADDA_LONG_RDIMM: usize;

    const ADDI_REG_BW: usize;
    const ADDI_REG_L: usize;
    const ADDI_MEM_BW: usize;
    const ADDI_MEM_L: usize;

    const ADDQ_REG_BW: usize;
    const ADDQ_REG_L: usize;
    const ADDQ_MEM_BW: usize;
    const ADDQ_MEM_L: usize;

    const ADDX_REG_BW: usize;
    const ADDX_REG_L: usize;
    const ADDX_MEM_BW: usize;
    const ADDX_MEM_L: usize;

    const AND_REG_BW: usize;
    const AND_REG_L: usize;
    const AND_REG_L_RDIMM: usize;
    const AND_MEM_BW: usize;
    const AND_MEM_L: usize;

    const ANDI_REG_BW: usize;
    const ANDI_REG_L: usize;
    const ANDI_MEM_BW: usize;
    const ANDI_MEM_L: usize;

    const ANDICCR: usize;

    const ANDISR: usize;

    const ASM: usize;

    const ASR_COUNT: usize;
    const ASR_BW: usize;
    const ASR_L: usize;

    const BCC_BRANCH: usize;
    const BCC_NO_BRANCH_BYTE: usize;
    const BCC_NO_BRANCH_WORD: usize;

    const BCHG_DYN_REG: usize;
    const BCHG_DYN_MEM: usize;
    const BCHG_STA_REG: usize;
    const BCHG_STA_MEM: usize;

    const BCLR_DYN_REG: usize;
    const BCLR_DYN_MEM: usize;
    const BCLR_STA_REG: usize;
    const BCLR_STA_MEM: usize;

    const BRA_BYTE: usize;
    const BRA_WORD: usize;

    const BSET_DYN_REG: usize;
    const BSET_DYN_MEM: usize;
    const BSET_STA_REG: usize;
    const BSET_STA_MEM: usize;

    const BSR_BYTE: usize;
    const BSR_WORD: usize;

    const BTST_DYN_REG: usize;
    const BTST_DYN_MEM: usize;
    const BTST_STA_REG: usize;
    const BTST_STA_MEM: usize;

    const CHK_NO_TRAP: usize;

    const CLR_REG_BW: usize;
    const CLR_REG_L: usize;
    const CLR_MEM_BW: usize; // Subtract one read cycle from effective address calculation.
    const CLR_MEM_L: usize; // Subtract two read cycles from effective address calculation.

    const CMP_BW: usize;
    const CMP_L: usize;

    const CMPA: usize;

    const CMPI_REG_BW: usize;
    const CMPI_REG_L: usize;
    const CMPI_MEM_BW: usize;
    const CMPI_MEM_L: usize;

    const CMPM_BW: usize;
    const CMPM_L: usize;

    const DBCC_TRUE: usize;
    const DBCC_FALSE_BRANCH: usize;
    const DBCC_FALSE_NO_BRANCH: usize;

    const DIVS: usize;

    const DIVU: usize;

    const EOR_REG_BW: usize;
    const EOR_REG_L: usize;
    const EOR_MEM_BW: usize;
    const EOR_MEM_L: usize;

    const EORI_REG_BW: usize;
    const EORI_REG_L: usize;
    const EORI_MEM_BW: usize;
    const EORI_MEM_L: usize;

    const EORICCR: usize;

    const EORISR: usize;

    const EXG: usize;

    const EXT: usize;

    const JMP_ARI: usize;
    const JMP_ARIWD: usize;
    const JMP_ARIWI8: usize;
    const JMP_ABSSHORT: usize;
    const JMP_ABSLONG: usize;
    const JMP_PCIWD: usize;
    const JMP_PCIWI8: usize;

    const JSR_ARI: usize;
    const JSR_ARIWD: usize;
    const JSR_ARIWI8: usize;
    const JSR_ABSSHORT: usize;
    const JSR_ABSLONG: usize;
    const JSR_PCIWD: usize;
    const JSR_PCIWI8: usize;

    const LEA_ARI: usize;
    const LEA_ARIWD: usize;
    const LEA_ARIWI8: usize;
    const LEA_ABSSHORT: usize;
    const LEA_ABSLONG: usize;
    const LEA_PCIWD: usize;
    const LEA_PCIWI8: usize;

    const LINK: usize;

    const LSM: usize;

    const LSR_COUNT: usize;
    const LSR_BW: usize;
    const LSR_L: usize;

    const MOVE_DST_ARIWPR: usize;
    const MOVE_OTHER: usize;

    const MOVEA: usize;

    const MOVECCR: usize;

    const MOVEFSR_REG: usize;
    const MOVEFSR_MEM: usize;

    const MOVESR: usize;

    const MOVEUSP: usize;

    const MOVEM_WORD: usize;
    const MOVEM_LONG: usize;
    const MOVEM_MTR: usize;
    const MOVEM_ARI: usize; // R -> M, do +3 for M -> R.
    const MOVEM_ARIWPO: usize;
    const MOVEM_ARIWPR: usize;
    const MOVEM_ARIWD: usize;
    const MOVEM_ARIWI8: usize;
    const MOVEM_ABSSHORT: usize;
    const MOVEM_ABSLONG: usize;
    const MOVEM_PCIWD: usize;
    const MOVEM_PCIWI8: usize;

    const MOVEP_RTM_WORD: usize;
    const MOVEP_RTM_LONG: usize;
    const MOVEP_MTR_WORD: usize;
    const MOVEP_MTR_LONG: usize;

    const MOVEQ: usize;

    const MULS: usize;

    const MULU: usize;

    const NBCD_REG: usize;
    const NBCD_MEM: usize;

    const NEG_REG_BW: usize;
    const NEG_REG_L: usize;
    const NEG_MEM_BW: usize;
    const NEG_MEM_L: usize;

    const NEGX_REG_BW: usize;
    const NEGX_REG_L: usize;
    const NEGX_MEM_BW: usize;
    const NEGX_MEM_L: usize;

    const NOP: usize;

    const NOT_REG_BW: usize;
    const NOT_REG_L: usize;
    const NOT_MEM_BW: usize;
    const NOT_MEM_L: usize;

    const OR_REG_BW: usize;
    const OR_REG_L: usize;
    const OR_REG_L_RDIMM: usize;
    const OR_MEM_BW: usize;
    const OR_MEM_L: usize;

    const ORI_REG_BW: usize;
    const ORI_REG_L: usize;
    const ORI_MEM_BW: usize;
    const ORI_MEM_L: usize;

    const ORICCR: usize;

    const ORISR: usize;

    const PEA_ARI: usize;
    const PEA_ARIWD: usize;
    const PEA_ARIWI8: usize;
    const PEA_ABSSHORT: usize;
    const PEA_ABSLONG: usize;
    const PEA_PCIWD: usize;
    const PEA_PCIWI8: usize;

    const RESET: usize;

    const ROM: usize;

    const ROR_COUNT: usize;
    const ROR_BW: usize;
    const ROR_L: usize;

    const ROXM: usize;

    const ROXR_COUNT: usize;
    const ROXR_BW: usize;
    const ROXR_L: usize;

    const RTE: usize;

    const RTR: usize;

    const RTS: usize;

    const SBCD_REG: usize;
    const SBCD_MEM: usize;

    const SCC_REG_FALSE: usize;
    const SCC_REG_TRUE: usize;
    const SCC_MEM_FALSE: usize;
    const SCC_MEM_TRUE: usize;

    const STOP: usize;

    const SUB_REG_BW: usize;
    const SUB_REG_L: usize;
    const SUB_REG_L_RDIMM: usize;
    const SUB_MEM_BW: usize;
    const SUB_MEM_L: usize;

    const SUBA_WORD: usize;
    const SUBA_LONG: usize;
    const SUBA_LONG_RDIMM: usize;

    const SUBI_REG_BW: usize;
    const SUBI_REG_L: usize;
    const SUBI_MEM_BW: usize;
    const SUBI_MEM_L: usize;

    const SUBQ_DREG_BW: usize;
    const SUBQ_AREG_BW: usize;
    const SUBQ_REG_L: usize;
    const SUBQ_MEM_BW: usize;
    const SUBQ_MEM_L: usize;

    const SUBX_REG_BW: usize;
    const SUBX_REG_L: usize;
    const SUBX_MEM_BW: usize;
    const SUBX_MEM_L: usize;

    const SWAP: usize;

    const TAS_REG: usize;
    const TAS_MEM: usize; // Subtract one read cycle from effective address calculation.

    const TRAPV_NO_TRAP: usize;

    const TST_REG_BW: usize;
    const TST_REG_L: usize;
    const TST_MEM_BW: usize;
    const TST_MEM_L: usize;

    const UNLK: usize;

    /// The time the CPU takes to reset itself (RESET vector 0).
    const VECTOR_RESET: usize;
    /// Returns the time it takes to process the given exception vector.
    fn vector_execution_time(vector: u8) -> usize;
}
