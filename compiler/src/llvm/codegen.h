#ifndef RAME_LLVM_CODEGEN_H
#define RAME_LLVM_CODEGEN_H

#include "asa.h"

#ifdef __cplusplus
using ast::asa_list;

extern "C" {
#endif

/**
 * Generate LLVM IR for the specified program.
 */
void codegen_llvm(asa_list fns);

#ifdef __cplusplus
}
#endif

#endif // RAME_LLVM_CODEGEN_H

