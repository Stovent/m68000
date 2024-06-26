#ifndef M68000_FFI_H
#define M68000_FFI_H

#include <stdint.h>
#include "m68000.h"

typedef struct m68000_mc68000_s m68000_mc68000_t;
typedef struct m68000_scc68070_s m68000_scc68070_t;

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
    m68000_vector_t exception;
} m68000_memory_result_t;

/**
 * Memory callbacks sent to the interpreter methods.
 *
 * Every member must be a valid function pointer, no pointer checks are done when calling the callbacks.
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
    struct m68000_memory_result_t (*set_byte)(uint32_t addr, uint8_t data, void *user_data);
    struct m68000_memory_result_t (*set_word)(uint32_t addr, uint16_t data, void *user_data);
    struct m68000_memory_result_t (*set_long)(uint32_t addr, uint32_t data, void *user_data);
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
    m68000_vector_t exception;
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
    m68000_vector_t exception;
} m68000_disassembler_exception_result_t;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * Allocates a new core and returns the pointer to it.
 *
 * The created core has a [Reset vector](m68000::exception::Vector::ResetSspPc) pushed, so that the first call to an
 * interpreter method will first fetch the reset vectors, then will execute the first instruction.
 *
 * It is not managed by Rust, so you have to delete it after usage with `m68000_*_delete`.
 */
m68000_mc68000_t *m68000_mc68000_new(void);

/**
 * `m68000_*_new` but without the initial reset vector, so you can initialize the core as you want.
 */
m68000_mc68000_t *m68000_mc68000_new_no_reset(void);

/**
 * Frees the memory of the given core.
 */
void m68000_mc68000_delete(m68000_mc68000_t *m68000);

/**
 * Runs the CPU for `cycles` number of cycles.
 *
 * This function executes **at least** the given number of cycles.
 * Returns the number of cycles actually executed.
 *
 * If you ask to execute 4 cycles but the next instruction takes 6 cycles to execute,
 * it will be executed and the 2 extra cycles will be subtracted in the next call.
 */
size_t m68000_mc68000_cycle(m68000_mc68000_t *m68000, struct m68000_callbacks_t *memory, size_t cycles);

/**
 * Runs the CPU until either an exception occurs or `cycle` cycles have been executed.
 *
 * This function executes **at least** the given number of cycles.
 * Returns the number of cycles actually executed, and the exception that occured if any.
 *
 * If you ask to execute 4 cycles but the next instruction takes 6 cycles to execute,
 * it will be executed and the 2 extra cycles will be subtracted in the next call.
 */
struct m68000_exception_result_t m68000_mc68000_cycle_until_exception(m68000_mc68000_t *m68000, struct m68000_callbacks_t *memory, size_t cycles);

/**
 * Runs indefinitely until an exception or STOP instruction occurs.
 *
 * Returns the number of cycles executed and the exception that occured.
 * If exception is None, this means the CPU has executed a STOP instruction.
 */
struct m68000_exception_result_t m68000_mc68000_loop_until_exception_stop(m68000_mc68000_t *m68000, struct m68000_callbacks_t *memory);

/**
 * Executes the next instruction, returning the cycle count necessary to execute it.
 */
size_t m68000_mc68000_interpreter(m68000_mc68000_t *m68000, struct m68000_callbacks_t *memory);

/**
 * Executes the next instruction, returning the cycle count necessary to execute it,
 * and the vector of the exception that occured during the execution if any.
 *
 * To process the returned exception, call `m68000_*_exception`.
 */
struct m68000_exception_result_t m68000_mc68000_interpreter_exception(m68000_mc68000_t *m68000, struct m68000_callbacks_t *memory);

/**
 * Executes and disassembles the next instruction, returning the disassembler string and the cycle count necessary to execute it.
 *
 * `str` is a pointer to a C string buffer where the disassembled instruction will be written.
 * `len` is the maximum size of the buffer, null-charactere included.
 */
struct m68000_disassembler_result_t m68000_mc68000_disassembler_interpreter(m68000_mc68000_t *m68000, struct m68000_callbacks_t *memory, char *str, size_t len);

/**
 * Executes and disassembles the next instruction, returning the disassembled string, the cycle count necessary to execute it,
 * and the vector of the exception that occured during the execution if any.
 *
 * To process the returned exception, call `m68000_*_exception`.
 *
 * `str` is a pointer to a C string buffer where the disassembled instruction will be written.
 * `len` is the maximum size of the buffer.
 */
struct m68000_disassembler_exception_result_t m68000_mc68000_disassembler_interpreter_exception(m68000_mc68000_t *m68000, struct m68000_callbacks_t *memory, char *str, size_t len);

/**
 * Requests the CPU to process the given exception vector.
 */
void m68000_mc68000_exception(m68000_mc68000_t *m68000, m68000_vector_t vector);

/**
 * Returns the 16-bits word at the current PC value of the given core and advances PC by 2.
 */
struct m68000_memory_result_t m68000_mc68000_get_next_word(m68000_mc68000_t *m68000, struct m68000_callbacks_t *memory);

/**
 * Returns the 32-bits long at the current PC value of the given core and advances PC by 4.
 */
struct m68000_memory_result_t m68000_mc68000_get_next_long(m68000_mc68000_t *m68000, struct m68000_callbacks_t *memory);

/**
 * Returns the 16-bits word at the current PC value of the given core.
 */
struct m68000_memory_result_t m68000_mc68000_peek_next_word(const m68000_mc68000_t *m68000, struct m68000_callbacks_t *memory);

/**
 * Returns a const pointer to the registers of the given core.
 */
const m68000_registers_t *m68000_mc68000_registers(const m68000_mc68000_t *m68000);

/**
 * Returns a mutable pointer to the registers of the given core.
 */
m68000_registers_t *m68000_mc68000_registers_mut(m68000_mc68000_t *m68000);

/**
 * Returns a copy of the registers of the given core.
 */
m68000_registers_t m68000_mc68000_get_registers(const m68000_mc68000_t *m68000);

/**
 * Sets the registers of the core to the given value.
 */
void m68000_mc68000_set_registers(m68000_mc68000_t *m68000, m68000_registers_t regs);

/**
 * Allocates a new core and returns the pointer to it.
 *
 * The created core has a [Reset vector](m68000::exception::Vector::ResetSspPc) pushed, so that the first call to an
 * interpreter method will first fetch the reset vectors, then will execute the first instruction.
 *
 * It is not managed by Rust, so you have to delete it after usage with `m68000_*_delete`.
 */
m68000_scc68070_t *m68000_scc68070_new(void);

/**
 * `m68000_*_new` but without the initial reset vector, so you can initialize the core as you want.
 */
m68000_scc68070_t *m68000_scc68070_new_no_reset(void);

/**
 * Frees the memory of the given core.
 */
void m68000_scc68070_delete(m68000_scc68070_t *m68000);

/**
 * Runs the CPU for `cycles` number of cycles.
 *
 * This function executes **at least** the given number of cycles.
 * Returns the number of cycles actually executed.
 *
 * If you ask to execute 4 cycles but the next instruction takes 6 cycles to execute,
 * it will be executed and the 2 extra cycles will be subtracted in the next call.
 */
size_t m68000_scc68070_cycle(m68000_scc68070_t *m68000, struct m68000_callbacks_t *memory, size_t cycles);

/**
 * Runs the CPU until either an exception occurs or `cycle` cycles have been executed.
 *
 * This function executes **at least** the given number of cycles.
 * Returns the number of cycles actually executed, and the exception that occured if any.
 *
 * If you ask to execute 4 cycles but the next instruction takes 6 cycles to execute,
 * it will be executed and the 2 extra cycles will be subtracted in the next call.
 */
struct m68000_exception_result_t m68000_scc68070_cycle_until_exception(m68000_scc68070_t *m68000, struct m68000_callbacks_t *memory, size_t cycles);

/**
 * Runs indefinitely until an exception or STOP instruction occurs.
 *
 * Returns the number of cycles executed and the exception that occured.
 * If exception is None, this means the CPU has executed a STOP instruction.
 */
struct m68000_exception_result_t m68000_scc68070_loop_until_exception_stop(m68000_scc68070_t *m68000, struct m68000_callbacks_t *memory);

/**
 * Executes the next instruction, returning the cycle count necessary to execute it.
 */
size_t m68000_scc68070_interpreter(m68000_scc68070_t *m68000, struct m68000_callbacks_t *memory);

/**
 * Executes the next instruction, returning the cycle count necessary to execute it,
 * and the vector of the exception that occured during the execution if any.
 *
 * To process the returned exception, call `m68000_*_exception`.
 */
struct m68000_exception_result_t m68000_scc68070_interpreter_exception(m68000_scc68070_t *m68000, struct m68000_callbacks_t *memory);

/**
 * Executes and disassembles the next instruction, returning the disassembler string and the cycle count necessary to execute it.
 *
 * `str` is a pointer to a C string buffer where the disassembled instruction will be written.
 * `len` is the maximum size of the buffer, null-charactere included.
 */
struct m68000_disassembler_result_t m68000_scc68070_disassembler_interpreter(m68000_scc68070_t *m68000, struct m68000_callbacks_t *memory, char *str, size_t len);

/**
 * Executes and disassembles the next instruction, returning the disassembled string, the cycle count necessary to execute it,
 * and the vector of the exception that occured during the execution if any.
 *
 * To process the returned exception, call `m68000_*_exception`.
 *
 * `str` is a pointer to a C string buffer where the disassembled instruction will be written.
 * `len` is the maximum size of the buffer.
 */
struct m68000_disassembler_exception_result_t m68000_scc68070_disassembler_interpreter_exception(m68000_scc68070_t *m68000, struct m68000_callbacks_t *memory, char *str, size_t len);

/**
 * Requests the CPU to process the given exception vector.
 */
void m68000_scc68070_exception(m68000_scc68070_t *m68000, m68000_vector_t vector);

/**
 * Returns the 16-bits word at the current PC value of the given core and advances PC by 2.
 */
struct m68000_memory_result_t m68000_scc68070_get_next_word(m68000_scc68070_t *m68000, struct m68000_callbacks_t *memory);

/**
 * Returns the 32-bits long at the current PC value of the given core and advances PC by 4.
 */
struct m68000_memory_result_t m68000_scc68070_get_next_long(m68000_scc68070_t *m68000, struct m68000_callbacks_t *memory);

/**
 * Returns the 16-bits word at the current PC value of the given core.
 */
struct m68000_memory_result_t m68000_scc68070_peek_next_word(const m68000_scc68070_t *m68000, struct m68000_callbacks_t *memory);

/**
 * Returns a const pointer to the registers of the given core.
 */
const m68000_registers_t *m68000_scc68070_registers(const m68000_scc68070_t *m68000);

/**
 * Returns a mutable pointer to the registers of the given core.
 */
m68000_registers_t *m68000_scc68070_registers_mut(m68000_scc68070_t *m68000);

/**
 * Returns a copy of the registers of the given core.
 */
m68000_registers_t m68000_scc68070_get_registers(const m68000_scc68070_t *m68000);

/**
 * Sets the registers of the core to the given value.
 */
void m68000_scc68070_set_registers(m68000_scc68070_t *m68000, m68000_registers_t regs);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

#endif /* M68000_FFI_H */
