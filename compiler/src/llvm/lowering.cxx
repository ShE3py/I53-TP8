#include "lowering.hxx"

#include <iostream>

#pragma GCC diagnostic ignored "-Wc99-designator"

namespace hir {

/**
 * Returns a new integer node.
 */
std::unique_ptr<asa> asa::Int(uint64_t v) {
    return std::unique_ptr<asa>(new asa { .tag = TagInt, .p.tag_int.value = v });
}

/**
 * Returns a new variable node.
 */
std::unique_ptr<asa> asa::Var(std::string identifier) {
    return std::unique_ptr<asa>(new asa { .tag = TagVar, .p.tag_var.identifier = identifier });
}

asa::NodePayload::~NodePayload() {}

/**
 * Lowers an AST node to a HIR node.
 */
std::unique_ptr<asa> lower(ast::asa *p) {
    if(!p || p == ast::NOP) {
        return std::unique_ptr<asa>(nullptr);
    }

    switch(p->tag) {
	    case ast::TagInt: {
	        return asa::Int(static_cast<uint64_t>(p->tag_int.value));
	    }
	    
	    case ast::TagVar: {
	        return asa::Var(p->tag_var.identifier);
	    }
	    
	    case ast::TagBinaryOp: {
	        return std::unique_ptr<asa>(new asa {
	            .tag = TagBinaryOp,
	            .p.tag_binary_op.op = p->tag_binary_op.op,
	            .p.tag_binary_op.lhs = lower(p->tag_binary_op.lhs),
	            .p.tag_binary_op.rhs = lower(p->tag_binary_op.rhs),
	        });
	    }
	    
	    case ast::TagAssignScalar: {
	        return std::unique_ptr<asa>(new asa {
	            .tag = TagAssignScalar,
	            .p.tag_assign_scalar.identifier = p->tag_assign_scalar.identifier,
	            .p.tag_assign_scalar.expr = lower(p->tag_assign_scalar.expr),
	        });
	    }
	    
	    case ast::TagRead: {
	        return std::unique_ptr<asa>(new asa {
	            .tag = TagAssignScalar,
	            .p.tag_assign_scalar.identifier = p->tag_read.identifier,
	            .p.tag_assign_scalar.expr = std::unique_ptr<asa>(new asa {
	                .tag = TagFnCall,
	                .p.tag_fn_call.identifier = "intrinsics.READ",
	                .p.tag_fn_call.args = std::vector<std::unique_ptr<asa>>(),
	            })
	        });
	    }
	    
	    case ast::TagPrint: {
	        std::vector<std::unique_ptr<asa>> arg;
	        arg.push_back(lower(p->tag_print.expr));
	    
	        return std::unique_ptr<asa>(new asa {
	            .tag = TagFnCall,
	            .p.tag_fn_call.identifier = "intrinsics.WRITE",
	            .p.tag_fn_call.args = std::move(arg),
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
	        
	        return std::unique_ptr<asa>(new asa {
	            .tag = TagBlock,
	            .p.tag_block.body = std::move(body),
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
	    
	        return std::unique_ptr<asa>(new asa {
	            .tag = TagFn,
	            .p.tag_fn.identifier = p->tag_fn.identifier,
	            .p.tag_fn.params = params,
	            .p.tag_fn.body = lower(p->tag_fn.body),
	            .p.tag_fn.st = p->tag_fn.st,
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
	    
	        return std::unique_ptr<asa>(new asa {
	            .tag = TagFnCall,
	            .p.tag_fn_call.identifier = p->tag_fn_call.identifier,
	            .p.tag_fn_call.args = std::move(args),
	        });
        }
        
        case ast::TagReturn: {
	        return std::unique_ptr<asa>(new asa {
	            .tag = TagReturn,
	            .p.tag_return.expr = lower(p->tag_return.expr),
	        });
	    }
	    
	    default:
	        std::cerr << "warning: unimplemented tag lowering: " << ast::tag_name(p->tag) << std::endl;
	        return asa::Int(0);
    }
}

}

