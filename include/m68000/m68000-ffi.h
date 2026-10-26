#ifndef M68000_M68000_FFI_H
#define M68000_M68000_FFI_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

/**
 * Exception vector numbers of the 68000.
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
#if __STDC_VERSION__ >= 202311L
  : uint8_t
#endif // __STDC_VERSION__ >= 202311L

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
#if __STDC_VERSION__ >= 202311L
typedef enum m68000_vector_t m68000_vector_t;
#else
typedef uint8_t m68000_vector_t;
#endif // __STDC_VERSION__ >= 202311L

/**
 * Cached interpreter which stores instruction blocks in a flat array.
 */
typedef struct CachedInterpreterFlatBlock CachedInterpreterFlatBlock;

/**
 * Cached interpreter with a single block that stores individually-cached instructions.
 */
typedef struct CachedInterpreterSingleBlock CachedInterpreterSingleBlock;

/**
 * Cached interpreter with a trie look-up table.
 *
 * https://web.archive.org/web/20210301060701/https://ps1.asuramaru.com/emulator-development/cached-interpreters
 */
typedef struct CachedInterpreterTrieBlock CachedInterpreterTrieBlock;

/**
 * The state of the CPU.
 */
typedef struct M68000_Mc68000 M68000_Mc68000;

/**
 * The state of the CPU.
 */
typedef struct M68000_Scc68070 M68000_Scc68070;

/**
 * Return type of the memory callback functions.
 */
typedef struct m68000_memory_result_t
{
    /**
     * Set to the value to be returned. Only the low order bytes are read depending on the size. Unused with SetResult.
     */
    uint32_t data;
    /**
     * Set to 0 if read successfully, set to 2 (Access Error) otherwise (Address errors are automatically detected by the library).
     *
     * If used as the return value of `m68000_*_peek_next_word`, this field contains the exception vector that occured when trying to read the next word.
     */
    uint8_t exception;
} m68000_memory_result_t;

/**
 * Memory callbacks sent to the interpreter methods.
 *
 * Every member must be a valid function pointer, no pointer checks are done when calling the callbacks.
 *
 * The set functions returns false when a bus error occured, or true when the write is successful.
 *
 * The void* argument passed on each callback is the [user_data](Self::user_data) member,
 * and its usage is let to the user of this library. For example, this can be used to allow the usage of C++ objects,
 * where [user_data](Self::user_data) has the value of the `this` pointer of the object.
 */
typedef struct m68000_callbacks_t
{
    struct m68000_memory_result_t (*get_byte)(uint32_t addr, void *user_data);
    struct m68000_memory_result_t (*get_word)(uint32_t addr, void *user_data);
    struct m68000_memory_result_t (*get_long)(uint32_t addr, void *user_data);
    bool (*set_byte)(uint32_t addr, uint8_t data, void *user_data);
    bool (*set_word)(uint32_t addr, uint16_t data, void *user_data);
    bool (*set_long)(uint32_t addr, uint32_t data, void *user_data);
    void (*reset_instruction)(void *user_data);
    void *user_data;
} m68000_callbacks_t;

/**
 * Return type of the `m68000_*_cycle_until_exception`, `m68000_*_loop_until_exception_stop` and
 * `m68000_*_interpreter_exception` functions.
 */
typedef struct m68000_exception_result_t
{
    /**
     * The number of cycles executed.
     */
    size_t cycles;
    /**
     * 0 if no exception occured, the vector number that occured otherwise.
     */
    uint8_t exception;
} m68000_exception_result_t;

/**
 * Return type of the `m68000_*_disassembler_interpreter` functions.
 */
typedef struct m68000_disassembler_result_t
{
    /**
     * The number of cycles executed.
     */
    size_t cycles;
    /**
     * The address of the instruction that has been executed if any.
     */
    uint32_t pc;
} m68000_disassembler_result_t;

/**
 * Return type of the `m68000_*_disassembler_interpreter_exception` functions.
 */
typedef struct m68000_disassembler_exception_result_t
{
    /**
     * The number of cycles executed.
     */
    size_t cycles;
    /**
     * The address of the instruction that has been executed if any.
     */
    uint32_t pc;
    /**
     * 0 if no exception occured, the vector number that occured otherwise.
     */
    uint8_t exception;
} m68000_disassembler_exception_result_t;

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

/**
 * Fast memory access with a software paging.
 */
typedef struct m68000_fastmem_t
{
    /**
     * Array of pointers to the pages.
     */
    const uint8_t *const *page_read;
    /**
     * Array of pointers to the pages.
     */
    uint8_t *const *page_write;
    /**
     * Shift applied to the address for indexing the first pointer.
     *
     * Can be different than [Self::offset_mask].
     */
    uint32_t page_shift;
    /**
     * Mask applied to the address for indexing the second pointer.
     */
    uint32_t offset_mask;
    /**
     * User callbacks for when the address is slow memory.
     */
    struct m68000_callbacks_t slow_memory;
} m68000_fastmem_t;



/**
 * Allocates a new flat block cached interpreter.
 */
struct CachedInterpreterFlatBlock *m68000_cached_interpreter_flat_block_new(uint32_t begin, uint32_t end);

/**
 * Frees a flat block cached interpreter.
 */
void m68000_cached_interpreter_flat_block_delete(struct CachedInterpreterFlatBlock *ptr);

/**
 * Allocates a new single block cached interpreter.
 */
struct CachedInterpreterSingleBlock *m68000_cached_interpreter_single_block_new(uint32_t begin, uint32_t end);

/**
 * Frees a single block cached interpreter.
 */
void m68000_cached_interpreter_single_block_delete(struct CachedInterpreterSingleBlock *ptr);

/**
 * Allocates a new trie block cached interpreter.
 */
struct CachedInterpreterTrieBlock *m68000_cached_interpreter_trie_block_new(void);

/**
 * Frees a trie block cached interpreter.
 */
void m68000_cached_interpreter_trie_block_delete(struct CachedInterpreterTrieBlock *ptr);

/**
 * Allocates a new core and returns the pointer to it.
 *
 * The created core has a [Reset vector](m68000::exception::Vector::ResetSspPc) pushed, so that the first call to an
 * interpreter method will first fetch the reset vectors, then will execute the first instruction.
 *
 * It is not managed by Rust, so you have to delete it after usage with `m68000_*_delete`.
 */
struct M68000_Mc68000 *m68000_mc68000_new(void);

/**
 * `m68000_*_new` but without the initial reset vector, so you can initialize the core as you want.
 */
struct M68000_Mc68000 *m68000_mc68000_new_no_reset(void);

/**
 * Frees the memory of the given core.
 */
void m68000_mc68000_delete(struct M68000_Mc68000 *m68000);

/**
 * Runs the CPU for `cycles` number of cycles.
 *
 * This function executes **at least** the given number of cycles.
 * Returns the number of cycles actually executed.
 *
 * If you ask to execute 4 cycles but the next instruction takes 6 cycles to execute,
 * it will be executed and the 2 extra cycles will be subtracted in the next call.
 */
size_t m68000_mc68000_cycle(struct M68000_Mc68000 *m68000, struct m68000_callbacks_t *memory, size_t cycles);

/**
 * Runs the CPU until either an exception occurs or `cycle` cycles have been executed.
 *
 * This function executes **at least** the given number of cycles.
 * Returns the number of cycles actually executed, and the exception that occured if any.
 *
 * If you ask to execute 4 cycles but the next instruction takes 6 cycles to execute,
 * it will be executed and the 2 extra cycles will be subtracted in the next call.
 */
struct m68000_exception_result_t m68000_mc68000_cycle_until_exception(struct M68000_Mc68000 *m68000, struct m68000_callbacks_t *memory, size_t cycles);

/**
 * Runs indefinitely until an exception or STOP instruction occurs.
 *
 * Returns the number of cycles executed and the exception that occured.
 * If exception is None, this means the CPU has executed a STOP instruction.
 */
struct m68000_exception_result_t m68000_mc68000_loop_until_exception_stop(struct M68000_Mc68000 *m68000, struct m68000_callbacks_t *memory);

/**
 * Executes the next instruction, returning the cycle count necessary to execute it.
 */
size_t m68000_mc68000_interpreter(struct M68000_Mc68000 *m68000, struct m68000_callbacks_t *memory);

/**
 * Executes the next instruction, returning the cycle count necessary to execute it,
 * and the vector of the exception that occured during the execution if any.
 *
 * To process the returned exception, call `m68000_*_exception`.
 */
struct m68000_exception_result_t m68000_mc68000_interpreter_exception(struct M68000_Mc68000 *m68000, struct m68000_callbacks_t *memory);

/**
 * Executes and disassembles the next instruction, returning the disassembler string and the cycle count necessary to execute it.
 *
 * `str` is a pointer to a C string buffer where the disassembled instruction will be written.
 * `len` is the maximum size of the buffer, null-charactere included.
 */
struct m68000_disassembler_result_t m68000_mc68000_disassembler_interpreter(struct M68000_Mc68000 *m68000, struct m68000_callbacks_t *memory, char *str, size_t len);

/**
 * Executes and disassembles the next instruction, returning the disassembled string, the cycle count necessary to execute it,
 * and the vector of the exception that occured during the execution if any.
 *
 * To process the returned exception, call `m68000_*_exception`.
 *
 * `str` is a pointer to a C string buffer where the disassembled instruction will be written.
 * `len` is the maximum size of the buffer.
 */
struct m68000_disassembler_exception_result_t m68000_mc68000_disassembler_interpreter_exception(struct M68000_Mc68000 *m68000, struct m68000_callbacks_t *memory, char *str, size_t len);

/**
 * Requests the CPU to process the given exception vector.
 */
void m68000_mc68000_exception(struct M68000_Mc68000 *m68000, m68000_vector_t vector);

/**
 * Returns the 16-bits word at the current PC value of the given core.
 */
struct m68000_memory_result_t m68000_mc68000_peek_next_word(const struct M68000_Mc68000 *m68000, struct m68000_callbacks_t *memory);

/**
 * Returns the 16-bits word at the current PC value of the given core and advances PC by 2.
 */
struct m68000_memory_result_t m68000_mc68000_get_next_word(struct M68000_Mc68000 *m68000, struct m68000_callbacks_t *memory);

/**
 * Returns the 32-bits long at the current PC value of the given core and advances PC by 4.
 */
struct m68000_memory_result_t m68000_mc68000_get_next_long(struct M68000_Mc68000 *m68000, struct m68000_callbacks_t *memory);

/**
 * Returns a const pointer to the registers of the given core.
 */
const struct m68000_registers_t *m68000_mc68000_registers(const struct M68000_Mc68000 *m68000);

/**
 * Returns a mutable pointer to the registers of the given core.
 */
struct m68000_registers_t *m68000_mc68000_registers_mut(struct M68000_Mc68000 *m68000);

/**
 * Returns a copy of the registers of the given core.
 */
struct m68000_registers_t m68000_mc68000_get_registers(const struct M68000_Mc68000 *m68000);

/**
 * Sets the registers of the core to the given value.
 */
void m68000_mc68000_set_registers(struct M68000_Mc68000 *m68000, struct m68000_registers_t regs);

/**
 * Returns a mutable pointer to the stop state flag of the given core.
 */
bool *m68000_mc68000_stop_mut(struct M68000_Mc68000 *m68000);

/**
 * Runs the CPU for `cycles` number of cycles.
 *
 * This function executes **at least** the given number of cycles.
 * Returns the number of cycles actually executed.
 *
 * If you ask to execute 4 cycles but the next instruction takes 6 cycles to execute,
 * it will be executed and the 2 extra cycles will be subtracted in the next call.
 */
size_t m68000_mc68000_fastmem_cycle(struct M68000_Mc68000 *m68000, struct m68000_fastmem_t *memory, size_t cycles);

/**
 * Runs the CPU until either an exception occurs or `cycle` cycles have been executed.
 *
 * This function executes **at least** the given number of cycles.
 * Returns the number of cycles actually executed, and the exception that occured if any.
 *
 * If you ask to execute 4 cycles but the next instruction takes 6 cycles to execute,
 * it will be executed and the 2 extra cycles will be subtracted in the next call.
 */
struct m68000_exception_result_t m68000_mc68000_fastmem_cycle_until_exception(struct M68000_Mc68000 *m68000, struct m68000_fastmem_t *memory, size_t cycles);

/**
 * Runs indefinitely until an exception or STOP instruction occurs.
 *
 * Returns the number of cycles executed and the exception that occured.
 * If exception is None, this means the CPU has executed a STOP instruction.
 */
struct m68000_exception_result_t m68000_mc68000_fastmem_loop_until_exception_stop(struct M68000_Mc68000 *m68000, struct m68000_fastmem_t *memory);

/**
 * Executes the next instruction, returning the cycle count necessary to execute it.
 */
size_t m68000_mc68000_fastmem_interpreter(struct M68000_Mc68000 *m68000, struct m68000_fastmem_t *memory);

/**
 * Executes the next instruction, returning the cycle count necessary to execute it,
 * and the vector of the exception that occured during the execution if any.
 *
 * To process the returned exception, call `m68000_*_exception`.
 */
struct m68000_exception_result_t m68000_mc68000_fastmem_interpreter_exception(struct M68000_Mc68000 *m68000, struct m68000_fastmem_t *memory);

/**
 * Executes and disassembles the next instruction, returning the disassembler string and the cycle count necessary to execute it.
 *
 * `str` is a pointer to a C string buffer where the disassembled instruction will be written.
 * `len` is the maximum size of the buffer, null-charactere included.
 */
struct m68000_disassembler_result_t m68000_mc68000_fastmem_disassembler_interpreter(struct M68000_Mc68000 *m68000, struct m68000_fastmem_t *memory, char *str, size_t len);

/**
 * Executes and disassembles the next instruction, returning the disassembled string, the cycle count necessary to execute it,
 * and the vector of the exception that occured during the execution if any.
 *
 * To process the returned exception, call `m68000_*_exception`.
 *
 * `str` is a pointer to a C string buffer where the disassembled instruction will be written.
 * `len` is the maximum size of the buffer.
 */
struct m68000_disassembler_exception_result_t m68000_mc68000_fastmem_disassembler_interpreter_exception(struct M68000_Mc68000 *m68000, struct m68000_fastmem_t *memory, char *str, size_t len);

/**
 * Returns the 16-bits word at the current PC value of the given core.
 */
struct m68000_memory_result_t m68000_mc68000_fastmem_peek_next_word(const struct M68000_Mc68000 *m68000, struct m68000_fastmem_t *memory);

/**
 * Returns the 16-bits word at the current PC value of the given core and advances PC by 2.
 */
struct m68000_memory_result_t m68000_mc68000_fastmem_get_next_word(struct M68000_Mc68000 *m68000, struct m68000_fastmem_t *memory);

/**
 * Returns the 32-bits long at the current PC value of the given core and advances PC by 4.
 */
struct m68000_memory_result_t m68000_mc68000_fastmem_get_next_long(struct M68000_Mc68000 *m68000, struct m68000_fastmem_t *memory);

/**
 * Executes the next instruction or block of instructions, returning the cycle count necessary to execute it.
 */
size_t m68000_mc68000_cached_interpreter_flat_block_interpreter(struct CachedInterpreterFlatBlock *interpreter, struct M68000_Mc68000 *m68000, struct m68000_callbacks_t *memory);

/**
 * Executes the next instruction or block of instructions, returning the cycle count necessary to execute it,
 * and the vector of the exception that occured during the execution if any.
 *
 * To process the returned exception, call `m68000_*_exception`.
 */
struct m68000_exception_result_t m68000_mc68000_cached_interpreter_flat_block_interpreter_exception(struct CachedInterpreterFlatBlock *interpreter, struct M68000_Mc68000 *m68000, struct m68000_callbacks_t *memory);

/**
 * Executes the next instruction or block of instructions, returning the cycle count necessary to execute it.
 */
size_t m68000_mc68000_fastmem_cached_interpreter_flat_block_interpreter(struct CachedInterpreterFlatBlock *interpreter, struct M68000_Mc68000 *m68000, struct m68000_fastmem_t *memory);

/**
 * Executes the next instruction or block of instructions, returning the cycle count necessary to execute it,
 * and the vector of the exception that occured during the execution if any.
 *
 * To process the returned exception, call `m68000_*_exception`.
 */
struct m68000_exception_result_t m68000_mc68000_fastmem_cached_interpreter_flat_block_interpreter_exception(struct CachedInterpreterFlatBlock *interpreter, struct M68000_Mc68000 *m68000, struct m68000_fastmem_t *memory);

/**
 * Executes the next instruction or block of instructions, returning the cycle count necessary to execute it.
 */
size_t m68000_mc68000_cached_interpreter_single_block_interpreter(struct CachedInterpreterSingleBlock *interpreter, struct M68000_Mc68000 *m68000, struct m68000_callbacks_t *memory);

/**
 * Executes the next instruction or block of instructions, returning the cycle count necessary to execute it,
 * and the vector of the exception that occured during the execution if any.
 *
 * To process the returned exception, call `m68000_*_exception`.
 */
struct m68000_exception_result_t m68000_mc68000_cached_interpreter_single_block_interpreter_exception(struct CachedInterpreterSingleBlock *interpreter, struct M68000_Mc68000 *m68000, struct m68000_callbacks_t *memory);

/**
 * Executes the next instruction or block of instructions, returning the cycle count necessary to execute it.
 */
size_t m68000_mc68000_fastmem_cached_interpreter_single_block_interpreter(struct CachedInterpreterSingleBlock *interpreter, struct M68000_Mc68000 *m68000, struct m68000_fastmem_t *memory);

/**
 * Executes the next instruction or block of instructions, returning the cycle count necessary to execute it,
 * and the vector of the exception that occured during the execution if any.
 *
 * To process the returned exception, call `m68000_*_exception`.
 */
struct m68000_exception_result_t m68000_mc68000_fastmem_cached_interpreter_single_block_interpreter_exception(struct CachedInterpreterSingleBlock *interpreter, struct M68000_Mc68000 *m68000, struct m68000_fastmem_t *memory);

/**
 * Executes the next instruction or block of instructions, returning the cycle count necessary to execute it.
 */
size_t m68000_mc68000_cached_interpreter_trie_block_interpreter(struct CachedInterpreterTrieBlock *interpreter, struct M68000_Mc68000 *m68000, struct m68000_callbacks_t *memory);

/**
 * Executes the next instruction or block of instructions, returning the cycle count necessary to execute it,
 * and the vector of the exception that occured during the execution if any.
 *
 * To process the returned exception, call `m68000_*_exception`.
 */
struct m68000_exception_result_t m68000_mc68000_cached_interpreter_trie_block_interpreter_exception(struct CachedInterpreterTrieBlock *interpreter, struct M68000_Mc68000 *m68000, struct m68000_callbacks_t *memory);

/**
 * Executes the next instruction or block of instructions, returning the cycle count necessary to execute it.
 */
size_t m68000_mc68000_fastmem_cached_interpreter_trie_block_interpreter(struct CachedInterpreterTrieBlock *interpreter, struct M68000_Mc68000 *m68000, struct m68000_fastmem_t *memory);

/**
 * Executes the next instruction or block of instructions, returning the cycle count necessary to execute it,
 * and the vector of the exception that occured during the execution if any.
 *
 * To process the returned exception, call `m68000_*_exception`.
 */
struct m68000_exception_result_t m68000_mc68000_fastmem_cached_interpreter_trie_block_interpreter_exception(struct CachedInterpreterTrieBlock *interpreter, struct M68000_Mc68000 *m68000, struct m68000_fastmem_t *memory);

/**
 * Allocates a new core and returns the pointer to it.
 *
 * The created core has a [Reset vector](m68000::exception::Vector::ResetSspPc) pushed, so that the first call to an
 * interpreter method will first fetch the reset vectors, then will execute the first instruction.
 *
 * It is not managed by Rust, so you have to delete it after usage with `m68000_*_delete`.
 */
struct M68000_Scc68070 *m68000_scc68070_new(void);

/**
 * `m68000_*_new` but without the initial reset vector, so you can initialize the core as you want.
 */
struct M68000_Scc68070 *m68000_scc68070_new_no_reset(void);

/**
 * Frees the memory of the given core.
 */
void m68000_scc68070_delete(struct M68000_Scc68070 *m68000);

/**
 * Runs the CPU for `cycles` number of cycles.
 *
 * This function executes **at least** the given number of cycles.
 * Returns the number of cycles actually executed.
 *
 * If you ask to execute 4 cycles but the next instruction takes 6 cycles to execute,
 * it will be executed and the 2 extra cycles will be subtracted in the next call.
 */
size_t m68000_scc68070_cycle(struct M68000_Scc68070 *m68000, struct m68000_callbacks_t *memory, size_t cycles);

/**
 * Runs the CPU until either an exception occurs or `cycle` cycles have been executed.
 *
 * This function executes **at least** the given number of cycles.
 * Returns the number of cycles actually executed, and the exception that occured if any.
 *
 * If you ask to execute 4 cycles but the next instruction takes 6 cycles to execute,
 * it will be executed and the 2 extra cycles will be subtracted in the next call.
 */
struct m68000_exception_result_t m68000_scc68070_cycle_until_exception(struct M68000_Scc68070 *m68000, struct m68000_callbacks_t *memory, size_t cycles);

/**
 * Runs indefinitely until an exception or STOP instruction occurs.
 *
 * Returns the number of cycles executed and the exception that occured.
 * If exception is None, this means the CPU has executed a STOP instruction.
 */
struct m68000_exception_result_t m68000_scc68070_loop_until_exception_stop(struct M68000_Scc68070 *m68000, struct m68000_callbacks_t *memory);

/**
 * Executes the next instruction, returning the cycle count necessary to execute it.
 */
size_t m68000_scc68070_interpreter(struct M68000_Scc68070 *m68000, struct m68000_callbacks_t *memory);

/**
 * Executes the next instruction, returning the cycle count necessary to execute it,
 * and the vector of the exception that occured during the execution if any.
 *
 * To process the returned exception, call `m68000_*_exception`.
 */
struct m68000_exception_result_t m68000_scc68070_interpreter_exception(struct M68000_Scc68070 *m68000, struct m68000_callbacks_t *memory);

/**
 * Executes and disassembles the next instruction, returning the disassembler string and the cycle count necessary to execute it.
 *
 * `str` is a pointer to a C string buffer where the disassembled instruction will be written.
 * `len` is the maximum size of the buffer, null-charactere included.
 */
struct m68000_disassembler_result_t m68000_scc68070_disassembler_interpreter(struct M68000_Scc68070 *m68000, struct m68000_callbacks_t *memory, char *str, size_t len);

/**
 * Executes and disassembles the next instruction, returning the disassembled string, the cycle count necessary to execute it,
 * and the vector of the exception that occured during the execution if any.
 *
 * To process the returned exception, call `m68000_*_exception`.
 *
 * `str` is a pointer to a C string buffer where the disassembled instruction will be written.
 * `len` is the maximum size of the buffer.
 */
struct m68000_disassembler_exception_result_t m68000_scc68070_disassembler_interpreter_exception(struct M68000_Scc68070 *m68000, struct m68000_callbacks_t *memory, char *str, size_t len);

/**
 * Requests the CPU to process the given exception vector.
 */
void m68000_scc68070_exception(struct M68000_Scc68070 *m68000, m68000_vector_t vector);

/**
 * Returns the 16-bits word at the current PC value of the given core.
 */
struct m68000_memory_result_t m68000_scc68070_peek_next_word(const struct M68000_Scc68070 *m68000, struct m68000_callbacks_t *memory);

/**
 * Returns the 16-bits word at the current PC value of the given core and advances PC by 2.
 */
struct m68000_memory_result_t m68000_scc68070_get_next_word(struct M68000_Scc68070 *m68000, struct m68000_callbacks_t *memory);

/**
 * Returns the 32-bits long at the current PC value of the given core and advances PC by 4.
 */
struct m68000_memory_result_t m68000_scc68070_get_next_long(struct M68000_Scc68070 *m68000, struct m68000_callbacks_t *memory);

/**
 * Returns a const pointer to the registers of the given core.
 */
const struct m68000_registers_t *m68000_scc68070_registers(const struct M68000_Scc68070 *m68000);

/**
 * Returns a mutable pointer to the registers of the given core.
 */
struct m68000_registers_t *m68000_scc68070_registers_mut(struct M68000_Scc68070 *m68000);

/**
 * Returns a copy of the registers of the given core.
 */
struct m68000_registers_t m68000_scc68070_get_registers(const struct M68000_Scc68070 *m68000);

/**
 * Sets the registers of the core to the given value.
 */
void m68000_scc68070_set_registers(struct M68000_Scc68070 *m68000, struct m68000_registers_t regs);

/**
 * Returns a mutable pointer to the stop state flag of the given core.
 */
bool *m68000_scc68070_stop_mut(struct M68000_Scc68070 *m68000);

/**
 * Runs the CPU for `cycles` number of cycles.
 *
 * This function executes **at least** the given number of cycles.
 * Returns the number of cycles actually executed.
 *
 * If you ask to execute 4 cycles but the next instruction takes 6 cycles to execute,
 * it will be executed and the 2 extra cycles will be subtracted in the next call.
 */
size_t m68000_scc68070_fastmem_cycle(struct M68000_Scc68070 *m68000, struct m68000_fastmem_t *memory, size_t cycles);

/**
 * Runs the CPU until either an exception occurs or `cycle` cycles have been executed.
 *
 * This function executes **at least** the given number of cycles.
 * Returns the number of cycles actually executed, and the exception that occured if any.
 *
 * If you ask to execute 4 cycles but the next instruction takes 6 cycles to execute,
 * it will be executed and the 2 extra cycles will be subtracted in the next call.
 */
struct m68000_exception_result_t m68000_scc68070_fastmem_cycle_until_exception(struct M68000_Scc68070 *m68000, struct m68000_fastmem_t *memory, size_t cycles);

/**
 * Runs indefinitely until an exception or STOP instruction occurs.
 *
 * Returns the number of cycles executed and the exception that occured.
 * If exception is None, this means the CPU has executed a STOP instruction.
 */
struct m68000_exception_result_t m68000_scc68070_fastmem_loop_until_exception_stop(struct M68000_Scc68070 *m68000, struct m68000_fastmem_t *memory);

/**
 * Executes the next instruction, returning the cycle count necessary to execute it.
 */
size_t m68000_scc68070_fastmem_interpreter(struct M68000_Scc68070 *m68000, struct m68000_fastmem_t *memory);

/**
 * Executes the next instruction, returning the cycle count necessary to execute it,
 * and the vector of the exception that occured during the execution if any.
 *
 * To process the returned exception, call `m68000_*_exception`.
 */
struct m68000_exception_result_t m68000_scc68070_fastmem_interpreter_exception(struct M68000_Scc68070 *m68000, struct m68000_fastmem_t *memory);

/**
 * Executes and disassembles the next instruction, returning the disassembler string and the cycle count necessary to execute it.
 *
 * `str` is a pointer to a C string buffer where the disassembled instruction will be written.
 * `len` is the maximum size of the buffer, null-charactere included.
 */
struct m68000_disassembler_result_t m68000_scc68070_fastmem_disassembler_interpreter(struct M68000_Scc68070 *m68000, struct m68000_fastmem_t *memory, char *str, size_t len);

/**
 * Executes and disassembles the next instruction, returning the disassembled string, the cycle count necessary to execute it,
 * and the vector of the exception that occured during the execution if any.
 *
 * To process the returned exception, call `m68000_*_exception`.
 *
 * `str` is a pointer to a C string buffer where the disassembled instruction will be written.
 * `len` is the maximum size of the buffer.
 */
struct m68000_disassembler_exception_result_t m68000_scc68070_fastmem_disassembler_interpreter_exception(struct M68000_Scc68070 *m68000, struct m68000_fastmem_t *memory, char *str, size_t len);

/**
 * Returns the 16-bits word at the current PC value of the given core.
 */
struct m68000_memory_result_t m68000_scc68070_fastmem_peek_next_word(const struct M68000_Scc68070 *m68000, struct m68000_fastmem_t *memory);

/**
 * Returns the 16-bits word at the current PC value of the given core and advances PC by 2.
 */
struct m68000_memory_result_t m68000_scc68070_fastmem_get_next_word(struct M68000_Scc68070 *m68000, struct m68000_fastmem_t *memory);

/**
 * Returns the 32-bits long at the current PC value of the given core and advances PC by 4.
 */
struct m68000_memory_result_t m68000_scc68070_fastmem_get_next_long(struct M68000_Scc68070 *m68000, struct m68000_fastmem_t *memory);

/**
 * Executes the next instruction or block of instructions, returning the cycle count necessary to execute it.
 */
size_t m68000_scc68070_cached_interpreter_flat_block_interpreter(struct CachedInterpreterFlatBlock *interpreter, struct M68000_Scc68070 *m68000, struct m68000_callbacks_t *memory);

/**
 * Executes the next instruction or block of instructions, returning the cycle count necessary to execute it,
 * and the vector of the exception that occured during the execution if any.
 *
 * To process the returned exception, call `m68000_*_exception`.
 */
struct m68000_exception_result_t m68000_scc68070_cached_interpreter_flat_block_interpreter_exception(struct CachedInterpreterFlatBlock *interpreter, struct M68000_Scc68070 *m68000, struct m68000_callbacks_t *memory);

/**
 * Executes the next instruction or block of instructions, returning the cycle count necessary to execute it.
 */
size_t m68000_scc68070_fastmem_cached_interpreter_flat_block_interpreter(struct CachedInterpreterFlatBlock *interpreter, struct M68000_Scc68070 *m68000, struct m68000_fastmem_t *memory);

/**
 * Executes the next instruction or block of instructions, returning the cycle count necessary to execute it,
 * and the vector of the exception that occured during the execution if any.
 *
 * To process the returned exception, call `m68000_*_exception`.
 */
struct m68000_exception_result_t m68000_scc68070_fastmem_cached_interpreter_flat_block_interpreter_exception(struct CachedInterpreterFlatBlock *interpreter, struct M68000_Scc68070 *m68000, struct m68000_fastmem_t *memory);

/**
 * Executes the next instruction or block of instructions, returning the cycle count necessary to execute it.
 */
size_t m68000_scc68070_cached_interpreter_single_block_interpreter(struct CachedInterpreterSingleBlock *interpreter, struct M68000_Scc68070 *m68000, struct m68000_callbacks_t *memory);

/**
 * Executes the next instruction or block of instructions, returning the cycle count necessary to execute it,
 * and the vector of the exception that occured during the execution if any.
 *
 * To process the returned exception, call `m68000_*_exception`.
 */
struct m68000_exception_result_t m68000_scc68070_cached_interpreter_single_block_interpreter_exception(struct CachedInterpreterSingleBlock *interpreter, struct M68000_Scc68070 *m68000, struct m68000_callbacks_t *memory);

/**
 * Executes the next instruction or block of instructions, returning the cycle count necessary to execute it.
 */
size_t m68000_scc68070_fastmem_cached_interpreter_single_block_interpreter(struct CachedInterpreterSingleBlock *interpreter, struct M68000_Scc68070 *m68000, struct m68000_fastmem_t *memory);

/**
 * Executes the next instruction or block of instructions, returning the cycle count necessary to execute it,
 * and the vector of the exception that occured during the execution if any.
 *
 * To process the returned exception, call `m68000_*_exception`.
 */
struct m68000_exception_result_t m68000_scc68070_fastmem_cached_interpreter_single_block_interpreter_exception(struct CachedInterpreterSingleBlock *interpreter, struct M68000_Scc68070 *m68000, struct m68000_fastmem_t *memory);

/**
 * Executes the next instruction or block of instructions, returning the cycle count necessary to execute it.
 */
size_t m68000_scc68070_cached_interpreter_trie_block_interpreter(struct CachedInterpreterTrieBlock *interpreter, struct M68000_Scc68070 *m68000, struct m68000_callbacks_t *memory);

/**
 * Executes the next instruction or block of instructions, returning the cycle count necessary to execute it,
 * and the vector of the exception that occured during the execution if any.
 *
 * To process the returned exception, call `m68000_*_exception`.
 */
struct m68000_exception_result_t m68000_scc68070_cached_interpreter_trie_block_interpreter_exception(struct CachedInterpreterTrieBlock *interpreter, struct M68000_Scc68070 *m68000, struct m68000_callbacks_t *memory);

/**
 * Executes the next instruction or block of instructions, returning the cycle count necessary to execute it.
 */
size_t m68000_scc68070_fastmem_cached_interpreter_trie_block_interpreter(struct CachedInterpreterTrieBlock *interpreter, struct M68000_Scc68070 *m68000, struct m68000_fastmem_t *memory);

/**
 * Executes the next instruction or block of instructions, returning the cycle count necessary to execute it,
 * and the vector of the exception that occured during the execution if any.
 *
 * To process the returned exception, call `m68000_*_exception`.
 */
struct m68000_exception_result_t m68000_scc68070_fastmem_cached_interpreter_trie_block_interpreter_exception(struct CachedInterpreterTrieBlock *interpreter, struct M68000_Scc68070 *m68000, struct m68000_fastmem_t *memory);

#endif  /* M68000_M68000_FFI_H */
