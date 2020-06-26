use crate::utils::span_lint;
use rustc_hir::{BorrowKind, Expr, ExprKind, Mutability};
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::ty::subst::Subst;
use rustc_middle::ty::{self, Ty};
use rustc_session::{declare_lint_pass, declare_tool_lint};

declare_clippy_lint! {
    /// **What it does:** Detects passing a mutable reference to a function that only
    /// requires an immutable reference.
    ///
    /// **Why is this bad?** The immutable reference rules out all other references
    /// to the value. Also the code misleads about the intent of the call site.
    ///
    /// **Known problems:** None.
    ///
    /// **Example:**
    /// ```ignore
    /// // Bad
    /// my_vec.push(&mut value)
    ///
    /// // Good
    /// my_vec.push(&value)
    /// ```
    pub UNNECESSARY_MUT_PASSED,
    style,
    "an argument passed as a mutable reference although the callee only demands an immutable reference"
}

declare_lint_pass!(UnnecessaryMutPassed => [UNNECESSARY_MUT_PASSED]);

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for UnnecessaryMutPassed {
    fn check_expr(&mut self, cx: &LateContext<'a, 'tcx>, e: &'tcx Expr<'_>) {
        match e.kind {
            ExprKind::Call(ref fn_expr, ref arguments) => {
                if let ExprKind::Path(ref path) = fn_expr.kind {
                    check_arguments(
                        cx,
                        arguments,
                        cx.tables().expr_ty(fn_expr),
                        &rustc_hir_pretty::to_string(rustc_hir_pretty::NO_ANN, |s| s.print_qpath(path, false)),
                    );
                }
            },
            ExprKind::MethodCall(ref path, _, ref arguments, _) => {
                let def_id = cx.tables().type_dependent_def_id(e.hir_id).unwrap();
                let substs = cx.tables().node_substs(e.hir_id);
                let method_type = cx.tcx.type_of(def_id).subst(cx.tcx, substs);
                check_arguments(cx, arguments, method_type, &path.ident.as_str())
            },
            _ => (),
        }
    }
}

fn check_arguments<'a, 'tcx>(
    cx: &LateContext<'a, 'tcx>,
    arguments: &[Expr<'_>],
    type_definition: Ty<'tcx>,
    name: &str,
) {
    match type_definition.kind {
        ty::FnDef(..) | ty::FnPtr(_) => {
            let parameters = type_definition.fn_sig(cx.tcx).skip_binder().inputs();
            for (argument, parameter) in arguments.iter().zip(parameters.iter()) {
                match parameter.kind {
                    ty::Ref(_, _, Mutability::Not)
                    | ty::RawPtr(ty::TypeAndMut {
                        mutbl: Mutability::Not, ..
                    }) => {
                        if let ExprKind::AddrOf(BorrowKind::Ref, Mutability::Mut, _) = argument.kind {
                            span_lint(
                                cx,
                                UNNECESSARY_MUT_PASSED,
                                argument.span,
                                &format!("The function/method `{}` doesn't need a mutable reference", name),
                            );
                        }
                    },
                    _ => (),
                }
            }
        },
        _ => (),
    }
}