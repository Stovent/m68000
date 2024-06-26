#ifndef M68000_H
#define M68000_H

#include <stdbool.h>
#include <stdint.h>





/**
 * Specify the direction of the operation.
 *
 * `RegisterToMemory` and `MemoryToRegister` are used by MOVEM and MOVEP.
 *
 * `DstReg` and `DstEa` are used by ADD, AND, OR and SUB.
 *
 * `Left` and `Right` are used by the Shift and Rotate instructions.
 *
 * `RegisterToUsp` and `UspToRegister` are used by MOVE USP.
 *
 * `RegisterToRegister` and `MemoryToMemory` are used by ABCD, ADDX, SBCD and SUBX.
 */
typedef enum m68000_direction_t
{
    /**
     * Transfert from a register to memory.
     */
    RegisterToMemory,
    /**
     * Transfert from memory to a register.
     */
    MemoryToRegister,
    /**
     * Destination is a register.
     */
    DstReg,
    /**
     * Destination is in memory.
     */
    DstEa,
    /**
     * Left shift or rotation.
     */
    Left,
    /**
     * Right shift or rotation.
     */
    Right,
    /**
     * For MOVE USP only.
     */
    RegisterToUsp,
    /**
     * For MOVE USP only.
     */
    UspToRegister,
    /**
     * Register to register operation.
     */
    RegisterToRegister,
    /**
     * Memory to Memory operation.
     */
    MemoryToMemory,
    /**
     * Exchange Data Registers (EXG only).
     */
    ExchangeData,
    /**
     * Exchange Address Registers (EXG only).
     */
    ExchangeAddress,
    /**
     * Exchange Data and Address Registers (EXG only).
     */
    ExchangeDataAddress,
} m68000_direction_t;

/**
 * Size of an operation.
 */
typedef enum m68000_size_t
{
    Byte = 1,
    Word = 2,
    Long = 4,
} m68000_size_t;

/**
 * Exception vectors of the 68000.
 *
 * You can directly cast the enum to u8 to get the vector number.
 * ```
 * use m68000::exception::Vector;
 * assert_eq!(Vector::AccessError as u8, 2);
 * ```
 *
 * The `FormatError` and `OnChipInterrupt` vectors are only used by the SCC68070.
 */
enum m68000_vector_t
#ifdef __cplusplus
  : uint8_t
#endif // __cplusplus

{
    ResetSspPc = 0,
    /**
     * Bus error. Sent when the accessed address is not in the memory range of the system.
     */
    AccessError = 2,
    AddressError,
    IllegalInstruction,
    ZeroDivide,
    ChkInstruction,
    TrapVInstruction,
    PrivilegeViolation,
    Trace,
    LineAEmulator,
    LineFEmulator,
    FormatError = 14,
    UninitializedInterrupt,
    /**
     * The spurious interrupt vector is taken when there is a bus error indication during interrupt processing.
     */
    SpuriousInterrupt = 24,
    Level1Interrupt,
    Level2Interrupt,
    Level3Interrupt,
    Level4Interrupt,
    Level5Interrupt,
    Level6Interrupt,
    Level7Interrupt,
    Trap0Instruction,
    Trap1Instruction,
    Trap2Instruction,
    Trap3Instruction,
    Trap4Instruction,
    Trap5Instruction,
    Trap6Instruction,
    Trap7Instruction,
    Trap8Instruction,
    Trap9Instruction,
    Trap10Instruction,
    Trap11Instruction,
    Trap12Instruction,
    Trap13Instruction,
    Trap14Instruction,
    Trap15Instruction,
    Level1OnChipInterrupt = 57,
    Level2OnChipInterrupt,
    Level3OnChipInterrupt,
    Level4OnChipInterrupt,
    Level5OnChipInterrupt,
    Level6OnChipInterrupt,
    Level7OnChipInterrupt,
    UserInterrupt,
};
#ifndef __cplusplus
typedef uint8_t m68000_vector_t;
#endif // __cplusplus

/**
 * Raw Brief Extension Word.
 */
typedef struct m68000_brief_extension_word_t
{
    uint16_t _0;
} m68000_brief_extension_word_t;

/**
 * Addressing modes.
 */
typedef enum m68000_addressing_mode_t_Tag
{
    /**
     * Data Register Direct.
     */
    Drd,
    /**
     * Address Register Direct.
     */
    Ard,
    /**
     * Address Register Indirect.
     */
    Ari,
    /**
     * Address Register Indirect With POstincrement.
     */
    Ariwpo,
    /**
     * Address Register Indirect With PRedecrement.
     */
    Ariwpr,
    /**
     * Address Register Indirect With Displacement (address reg, displacement).
     */
    Ariwd,
    /**
     * Address Register Indirect With Index 8 (address reg, brief extension word).
     */
    Ariwi8,
    /**
     * Absolute Short.
     */
    AbsShort,
    /**
     * Absolute Long.
     */
    AbsLong,
    /**
     * Program Counter Indirect With Displacement (PC value, displacement).
     *
     * When using it with the assembler, the PC value is ignored.
     */
    Pciwd,
    /**
     * Program Counter Indirect With Index 8 (PC value, brief extension word).
     *
     * When using it with the assembler, the PC value is ignored.
     */
    Pciwi8,
    /**
     * Immediate Data (cast this variant to the correct type when used).
     */
    Immediate,
} m68000_addressing_mode_t_Tag;

typedef struct Ariwd_Body
{
    uint8_t _0;
    int16_t _1;
} Ariwd_Body;

typedef struct Ariwi8_Body
{
    uint8_t _0;
    struct m68000_brief_extension_word_t _1;
} Ariwi8_Body;

typedef struct Pciwd_Body
{
    uint32_t _0;
    int16_t _1;
} Pciwd_Body;

typedef struct Pciwi8_Body
{
    uint32_t _0;
    struct m68000_brief_extension_word_t _1;
} Pciwi8_Body;

typedef struct m68000_addressing_mode_t
{
    m68000_addressing_mode_t_Tag tag;
    union
    {
        struct
        {
            uint8_t drd;
        };
        struct
        {
            uint8_t ard;
        };
        struct
        {
            uint8_t ari;
        };
        struct
        {
            uint8_t ariwpo;
        };
        struct
        {
            uint8_t ariwpr;
        };
        Ariwd_Body ariwd;
        Ariwi8_Body ariwi8;
        struct
        {
            uint16_t abs_short;
        };
        struct
        {
            uint32_t abs_long;
        };
        Pciwd_Body pciwd;
        Pciwi8_Body pciwi8;
        struct
        {
            uint32_t immediate;
        };
    };
} m68000_addressing_mode_t;

/**
 * Operands of an instruction.
 */
typedef enum m68000_operands_t_Tag
{
    /**
     * ILLEGAL, NOP, RESET, RTE, RTR, RTS, TRAPV
     */
    NoOperands,
    /**
     * ANDI/EORI/ORI CCR/SR, STOP
     */
    Immediate_,
    /**
     * ADDI, ANDI, CMPI, EORI, ORI, SUBI
     */
    SizeEffectiveAddressImmediate,
    /**
     * BCHG, BCLR, BSET, BTST
     */
    EffectiveAddressCount,
    /**
     * JMP, JSR, MOVE (f) SR CCR, NBCD, PEA, TAS
     */
    EffectiveAddress,
    /**
     * CLR, NEG, NEGX, NOT, TST
     */
    SizeEffectiveAddress,
    /**
     * CHK, DIVS, DIVU, LEA, MULS, MULU
     */
    RegisterEffectiveAddress,
    /**
     * MOVEP
     */
    RegisterDirectionSizeRegisterDisplacement,
    /**
     * MOVEA
     */
    SizeRegisterEffectiveAddress,
    /**
     * MOVE
     */
    SizeEffectiveAddressEffectiveAddress,
    /**
     * EXG
     */
    RegisterOpmodeRegister,
    /**
     * EXT
     */
    OpmodeRegister,
    /**
     * TRAP
     */
    Vector,
    /**
     * LINK
     */
    RegisterDisplacement,
    /**
     * SWAP, UNLK
     */
    Register,
    /**
     * MOVE USP
     */
    DirectionRegister,
    /**
     * MOVEM
     */
    DirectionSizeEffectiveAddressList,
    /**
     * ADDQ, SUBQ
     */
    DataSizeEffectiveAddress,
    /**
     * Scc
     */
    ConditionEffectiveAddress,
    /**
     * DBcc
     */
    ConditionRegisterDisplacement,
    /**
     * BRA, BSR
     */
    Displacement,
    /**
     * Bcc
     */
    ConditionDisplacement,
    /**
     * MOVEQ
     */
    RegisterData,
    /**
     * ADD, AND, CMP, EOR, OR, SUB
     */
    RegisterDirectionSizeEffectiveAddress,
    /**
     * ADDA, CMPA, SUBA
     */
    RegisterSizeEffectiveAddress,
    /**
     * ABCD, ADDX, SBCD, SUBX
     */
    RegisterSizeModeRegister,
    /**
     * CMPM
     */
    RegisterSizeRegister,
    /**
     * ASm, LSm, ROm, ROXm
     */
    DirectionEffectiveAddress,
    /**
     * ASr, LSr, ROr, ROXr
     */
    RotationDirectionSizeModeRegister,
} m68000_operands_t_Tag;

typedef struct SizeEffectiveAddressImmediate_Body
{
    enum m68000_size_t _0;
    struct m68000_addressing_mode_t _1;
    uint32_t _2;
} SizeEffectiveAddressImmediate_Body;

typedef struct EffectiveAddressCount_Body
{
    struct m68000_addressing_mode_t _0;
    uint8_t _1;
} EffectiveAddressCount_Body;

typedef struct SizeEffectiveAddress_Body
{
    enum m68000_size_t _0;
    struct m68000_addressing_mode_t _1;
} SizeEffectiveAddress_Body;

typedef struct RegisterEffectiveAddress_Body
{
    uint8_t _0;
    struct m68000_addressing_mode_t _1;
} RegisterEffectiveAddress_Body;

typedef struct RegisterDirectionSizeRegisterDisplacement_Body
{
    uint8_t _0;
    enum m68000_direction_t _1;
    enum m68000_size_t _2;
    uint8_t _3;
    int16_t _4;
} RegisterDirectionSizeRegisterDisplacement_Body;

typedef struct SizeRegisterEffectiveAddress_Body
{
    enum m68000_size_t _0;
    uint8_t _1;
    struct m68000_addressing_mode_t _2;
} SizeRegisterEffectiveAddress_Body;

typedef struct SizeEffectiveAddressEffectiveAddress_Body
{
    enum m68000_size_t _0;
    struct m68000_addressing_mode_t _1;
    struct m68000_addressing_mode_t _2;
} SizeEffectiveAddressEffectiveAddress_Body;

typedef struct RegisterOpmodeRegister_Body
{
    uint8_t _0;
    enum m68000_direction_t _1;
    uint8_t _2;
} RegisterOpmodeRegister_Body;

typedef struct OpmodeRegister_Body
{
    uint8_t _0;
    uint8_t _1;
} OpmodeRegister_Body;

typedef struct RegisterDisplacement_Body
{
    uint8_t _0;
    int16_t _1;
} RegisterDisplacement_Body;

typedef struct DirectionRegister_Body
{
    enum m68000_direction_t _0;
    uint8_t _1;
} DirectionRegister_Body;

typedef struct DirectionSizeEffectiveAddressList_Body
{
    enum m68000_direction_t _0;
    enum m68000_size_t _1;
    struct m68000_addressing_mode_t _2;
    uint16_t _3;
} DirectionSizeEffectiveAddressList_Body;

typedef struct DataSizeEffectiveAddress_Body
{
    uint8_t _0;
    enum m68000_size_t _1;
    struct m68000_addressing_mode_t _2;
} DataSizeEffectiveAddress_Body;

typedef struct ConditionEffectiveAddress_Body
{
    uint8_t _0;
    struct m68000_addressing_mode_t _1;
} ConditionEffectiveAddress_Body;

typedef struct ConditionRegisterDisplacement_Body
{
    uint8_t _0;
    uint8_t _1;
    int16_t _2;
} ConditionRegisterDisplacement_Body;

typedef struct ConditionDisplacement_Body
{
    uint8_t _0;
    int16_t _1;
} ConditionDisplacement_Body;

typedef struct RegisterData_Body
{
    uint8_t _0;
    int8_t _1;
} RegisterData_Body;

typedef struct RegisterDirectionSizeEffectiveAddress_Body
{
    uint8_t _0;
    enum m68000_direction_t _1;
    enum m68000_size_t _2;
    struct m68000_addressing_mode_t _3;
} RegisterDirectionSizeEffectiveAddress_Body;

typedef struct RegisterSizeEffectiveAddress_Body
{
    uint8_t _0;
    enum m68000_size_t _1;
    struct m68000_addressing_mode_t _2;
} RegisterSizeEffectiveAddress_Body;

typedef struct RegisterSizeModeRegister_Body
{
    uint8_t _0;
    enum m68000_size_t _1;
    enum m68000_direction_t _2;
    uint8_t _3;
} RegisterSizeModeRegister_Body;

typedef struct RegisterSizeRegister_Body
{
    uint8_t _0;
    enum m68000_size_t _1;
    uint8_t _2;
} RegisterSizeRegister_Body;

typedef struct DirectionEffectiveAddress_Body
{
    enum m68000_direction_t _0;
    struct m68000_addressing_mode_t _1;
} DirectionEffectiveAddress_Body;

typedef struct RotationDirectionSizeModeRegister_Body
{
    uint8_t _0;
    enum m68000_direction_t _1;
    enum m68000_size_t _2;
    bool _3;
    uint8_t _4;
} RotationDirectionSizeModeRegister_Body;

typedef struct m68000_operands_t
{
    m68000_operands_t_Tag tag;
    union
    {
        struct
        {
            uint16_t immediate;
        };
        SizeEffectiveAddressImmediate_Body size_effective_address_immediate;
        EffectiveAddressCount_Body effective_address_count;
        struct
        {
            struct m68000_addressing_mode_t effective_address;
        };
        SizeEffectiveAddress_Body size_effective_address;
        RegisterEffectiveAddress_Body register_effective_address;
        RegisterDirectionSizeRegisterDisplacement_Body register_direction_size_register_displacement;
        SizeRegisterEffectiveAddress_Body size_register_effective_address;
        SizeEffectiveAddressEffectiveAddress_Body size_effective_address_effective_address;
        RegisterOpmodeRegister_Body register_opmode_register;
        OpmodeRegister_Body opmode_register;
        struct
        {
            uint8_t vector;
        };
        RegisterDisplacement_Body register_displacement;
        struct
        {
            uint8_t register_;
        };
        DirectionRegister_Body direction_register;
        DirectionSizeEffectiveAddressList_Body direction_size_effective_address_list;
        DataSizeEffectiveAddress_Body data_size_effective_address;
        ConditionEffectiveAddress_Body condition_effective_address;
        ConditionRegisterDisplacement_Body condition_register_displacement;
        struct
        {
            int16_t displacement;
        };
        ConditionDisplacement_Body condition_displacement;
        RegisterData_Body register_data;
        RegisterDirectionSizeEffectiveAddress_Body register_direction_size_effective_address;
        RegisterSizeEffectiveAddress_Body register_size_effective_address;
        RegisterSizeModeRegister_Body register_size_mode_register;
        RegisterSizeRegister_Body register_size_register;
        DirectionEffectiveAddress_Body direction_effective_address;
        RotationDirectionSizeModeRegister_Body rotation_direction_size_mode_register;
    };
} m68000_operands_t;

/**
 * M68000 instruction.
 */
typedef struct m68000_instruction_t
{
    /**
     * The opcode itself.
     */
    uint16_t opcode;
    /**
     * The address of the instruction.
     */
    uint32_t pc;
    /**
     * The operands.
     */
    struct m68000_operands_t operands;
} m68000_instruction_t;

/**
 * M68000 status register.
 *
 * [StatusRegister::default] returns a Status Register set to 0x2700 (supervisor bit set, interrupt mask to 7).
 */
typedef struct m68000_status_register_t
{
    /**
     * Trace
     */
    bool t;
    /**
     * Supervisor
     */
    bool s;
    /**
     * Interrupt Priority Mask
     */
    uint8_t interrupt_mask;
    /**
     * Extend
     */
    bool x;
    /**
     * Negate
     */
    bool n;
    /**
     * Zero
     */
    bool z;
    /**
     * Overflow
     */
    bool v;
    /**
     * Carry
     */
    bool c;
} m68000_status_register_t;
/**
 * The default raw value of 0x2700 (supervisor bit set, interrupt mask to 7).
 */
#define m68000_status_register_t_DEFAULT 9984

/**
 * M68000 registers.
 */
typedef struct m68000_registers_t
{
    /**
     * Data registers.
     */
    uint32_t d[8];
    /**
     * Address registers.
     */
    uint32_t a[7];
    /**
     * User Stack Pointer.
     */
    uint32_t usp;
    /**
     * System Stack Pointer.
     */
    uint32_t ssp;
    /**
     * Status Register.
     */
    struct m68000_status_register_t sr;
    /**
     * Program Counter.
     */
    uint32_t pc;
} m68000_registers_t;

#endif /* M68000_H */


