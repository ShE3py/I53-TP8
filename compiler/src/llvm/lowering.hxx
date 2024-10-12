#ifndef RAME_LLVM_LOWERING_H
#define RAME_LLVM_LOWERING_H

#include <cstdint>
#include <memory>
#include <string>
#include <variant>
#include <vector>

#include "asa.h"

// High-level Intermediate Representation
namespace hir {

struct TagInt;           // 0
struct TagVar;           // 1
struct TagIndex;         // 2
struct TagBinaryOp;      // 3
struct TagAssignScalar;  // 4
struct TagAssignIndexed; // 5
struct TagBlock;         // 6
struct TagFn;            // 7
struct TagFnCall;        // 8
struct TagReturn;        // 9

/**
 * An abstract syntax tree node.
 */
using asa = std::variant<TagInt, TagVar, TagIndex, TagBinaryOp, TagAssignScalar, TagAssignIndexed, TagBlock, TagFn, TagFnCall, TagReturn>;

/**
 * An integer.
 */
struct TagInt {
    /**
     * The integer.
     */
    uint64_t value;
};

/**
 * The value of a scalar variable.
 */
struct TagVar {
    /**
     * The variable identifier.
     */
    std::string identifier;
};

/**
 * The value of a array variable.
 */
struct TagIndex {
    /**
     * The variable identifier.
     */
    std::string identifier;
    
    /**
     * The index;
     */
    std::unique_ptr<asa> index;
};

/**
 * A binary operation.
 */
struct TagBinaryOp {
    /**
	 * The binary operation.
	 */
	ast::BinaryOp op;
	
	/**
	 * The left hand.
	 */
	std::unique_ptr<asa> lhs;
	
	/**
	 * The right hand.
	 */
	std::unique_ptr<asa> rhs;
};

/**
 * Scalar-to-scalar assignment.
 */
struct TagAssignScalar {
    /**
     * The var the be modified.
     */
    std::string identifier;
    
    /**
     * The value to be assigned.
     */
    std::unique_ptr<asa> expr;
};

/**
 * Scalar-to-indexed-scalar assignment.
 */
struct TagAssignIndexed {
    /**
     * The var the be modified.
     */
    std::string identifier;
    
    /**
     * The index to be modified.
     */
    std::unique_ptr<asa> index;
    
    /**
     * The value to be assigned.
     */
    std::unique_ptr<asa> expr;
};

/**
 * A code block. NOT a `llvm::BasicBlock`!
 */
struct TagBlock {
    /**
     * The instruction list.
     */
    std::vector<std::unique_ptr<asa>> body;
};

/**
 * A function body.
 */
struct TagFn {
    /**
     * The function name.
     */
    std::string identifier;
    
    /**
     * The function parameters.
     */
    std::vector<std::string> params;
    
    /**
     * The function body.
     */
    std::unique_ptr<asa> body;
    
    /**
     * The function symbols table.
     */
    symbol_table *st;
};

/**
 * A function call.
 */
struct TagFnCall {
    /**
     * The function name.
     */
    std::string identifier;
    
    /**
     * The function arguments.
     */
    std::vector<std::unique_ptr<asa>> args;
};

/**
 * A function return.
 */
struct TagReturn {
	/**
	 * The expression to return.
	 */
	 std::unique_ptr<asa> expr;
};

// https://stackoverflow.com/a/52303671
template<typename T, std::size_t index = 0>
consteval size_t tag_index() {
    static_assert(std::variant_size_v<asa> > index, "type not found in variant");
    
    if constexpr (index == std::variant_size_v<asa>) {
        return index;
    } else if constexpr (std::is_same_v<std::variant_alternative_t<index, asa>, T>) {
        return index;
    } else {
        return tag_index<T, index + 1>();
    }
}

template<typename T>
constexpr size_t tag_index_v = tag_index<T>();

/**
 * Lowers an AST node to a HIR node.
 */
std::unique_ptr<asa> lower(ast::asa *p);

}

#endif // RAME_LLVM_LOWERING_H

