#include "lowering.hxx"

#include <iostream>

#pragma GCC diagnostic ignored "-Wc99-designator"

namespace hir {

/**
 * Lowers an AST node to a HIR node.
 */
std::unique_ptr<asa> lower(ast::asa *p) {
    if(!p || p == ast::NOP) {
        return std::make_unique<asa>();
    }

    switch(p->tag) {
	    case ast::TagInt: {
	        return std::make_unique<asa>(TagInt { .value = static_cast<uint64_t>(p->tag_int.value) });
	    }
	    
	    case ast::TagVar: {
	        return std::make_unique<asa>(TagVar { .identifier = p->tag_var.identifier });
	    }
	    
	    case ast::TagBinaryOp: {
	        return std::make_unique<asa>(TagBinaryOp {
	            .op = p->tag_binary_op.op,
	            .lhs = lower(p->tag_binary_op.lhs),
	            .rhs = lower(p->tag_binary_op.rhs),
	        });
	    }
	    
	    case ast::TagAssignScalar: {
	        return std::make_unique<asa>(TagAssignScalar {
	            .identifier = p->tag_assign_scalar.identifier,
	            .expr = lower(p->tag_assign_scalar.expr),
	        });
	    }
	    
	    case ast::TagRead: {
	        return std::make_unique<asa>(TagAssignScalar {
	            .identifier = p->tag_read.identifier,
	            .expr = std::make_unique<asa>(TagFnCall {
	                .identifier = "intrinsics.READ",
	                .args = {},
	            })
	        });
	    }
	    
	    case ast::TagPrint: {
	        std::vector<std::unique_ptr<asa>> arg;
	        arg.push_back(lower(p->tag_print.expr));
	    
	        return std::make_unique<asa>(TagFnCall {
	            .identifier = "intrinsics.WRITE",
	            .args = std::move(arg),
	        });
	    }
	    
	    case ast::TagBlock: {
	        std::vector<std::unique_ptr<asa>> body;
	        ast::asa *b = p;
	        
	        do {
	            body.push_back(lower(b->tag_block.stmt));
	            b = b->tag_block.next;
	        }
	        while(b);
	        
	        return std::make_unique<asa>(TagBlock {
	            .body = std::move(body),
	        });
        }
	    
	    case ast::TagFn: {
	        std::vector<std::string> params;
	        params.reserve(p->tag_fn.params.len);
	        ast::id_list_node *param = p->tag_fn.params.head;
	        
	        while(param) {
	            params.push_back(param->value);
	            param = param->next;
	        }
	    
	        return std::make_unique<asa>(TagFn {
	            .identifier = p->tag_fn.identifier,
	            .params = std::move(params),
	            .body = lower(p->tag_fn.body),
	            .st = p->tag_fn.st,
	        });
        }
        
        case ast::TagFnCall: {
            std::vector<std::unique_ptr<asa>> args;
            args.reserve(p->tag_fn_call.args.len);
	        ast::asa_list_node *arg = p->tag_fn_call.args.head;
	        
	        while(arg) {
	            args.push_back(lower(arg->value));
	            arg = arg->next;
	        }
	    
	        return std::make_unique<asa>(TagFnCall {
	            .identifier = p->tag_fn_call.identifier,
	            .args = std::move(args),
	        });
        }
        
        case ast::TagReturn: {
	        return std::make_unique<asa>(TagReturn {
	            .expr = lower(p->tag_return.expr),
	        });
	    }
	    
	    default:
	        std::cerr << "warning: unimplemented tag lowering: " << ast::tag_name(p->tag) << std::endl;
	        return std::make_unique<asa>(TagInt {});
    }
} // namespace hir

}

