#ifndef RAME_LLVM_LOWERING_H
#define RAME_LLVM_LOWERING_H

#include <cstdint>
#include <string>
#include <vector>
#include <memory>

#include "asa.h"

// High-level Intermediate Representation
namespace hir {

/**
 * Possible nodes values.
 */
enum NodeTag {
    /**
     * An integer.
     */
    TagInt,
    
    /**
     * A variable.
     */
    TagVar,
    
    /**
     * A binary operation.
     */
    TagBinaryOp,
    
    /**
     * The `READ` intrinsic.
     */
    TagRead,
    
    /**
     * The `WRITE` instrinsic.
     */
    TagPrint,
    
    /**
     * A code block.
     */
    TagBlock,
    
    /**
     * A function body.
     */
    TagFn,
};

/**
 * An abstract syntax tree node.
 */
struct asa {
    /**
     * The node kind.
     */
    NodeTag tag;
    
    union NodePayload {
        /**
         * The payload of a `TagInt` node.
         */
        struct {
            /**
             * The integer.
             */
            uint64_t value;
        } tag_int;
        
         /**
         * The payload of a `TagVar` node.
         */
        struct {
            /**
             * The variable identifier.
             */
            std::string identifier;
        } tag_var;
        
        /**
         * The payload of a `TagBinaryOp` node.
         */
        struct {
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
        } tag_binary_op;
        
        /**
		 * The payload of a `TagRead` node.
		 */
		struct {
			/**
			 * The var to be modified.
			 */
			std::string identifier;
		} tag_read;
		
		/**
		 * The payload of a `TagPrint` node.
		 */
		struct {
			/**
			 * The expression to print.
			 */
			 std::unique_ptr<asa> expr;
		} tag_print;
        
        /**
         * The payload of a `TagBlock` node.
         */
        struct {
            /**
             * The instruction list.
             */
            std::vector<std::unique_ptr<asa>> body;
        } tag_block;
        
        /**
         * The payload of a `TagFn` node.
         */
        struct {
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
        } tag_fn;
        
        ~NodePayload();
    } p;
    
    ~asa() = default;
    
    /**
     * Returns a new integer node.
     */
    static std::unique_ptr<asa> Int(uint64_t v);
    
    /**
     * Returns a new variable node.
     */
    static std::unique_ptr<asa> Var(std::string identifier);
};

/**
 * Lowers an AST node to a HIR node.
 */
std::unique_ptr<asa> lower(ast::asa *p);

}

#endif // RAME_LLVM_LOWERING_H

