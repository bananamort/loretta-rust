use full_moon::ast;
use full_moon::visitors::Visitor;
use loretta_fuzz::{fuzz_input, run_iters};

/// "run" = execute a dry-run of the program: walk the whole tree the way an
/// interpreter would, counting every statement, expression and call site.
struct Runner {
    stmts: usize,
    exprs: usize,
    calls: usize,
}

impl Visitor for Runner {
    fn visit_stmt(&mut self, _: &ast::Stmt) {
        self.stmts += 1;
    }

    fn visit_expression(&mut self, _: &ast::Expression) {
        self.exprs += 1;
    }

    fn visit_function_call(&mut self, _: &ast::FunctionCall) {
        self.calls += 1;
    }
}

fn target(data: &[u8]) {
    let text = String::from_utf8_lossy(data);
    let result = full_moon::parse_fallible(&text, full_moon::ast::LuaVersion::new().with_luajit());
    let mut runner = Runner {
        stmts: 0,
        exprs: 0,
        calls: 0,
    };
    runner.visit_ast(result.ast());
    let _ = (runner.stmts, runner.exprs, runner.calls);
}

fn main() {
    let (iters, seed) = fuzz_input();
    run_iters("run", iters, seed, target);
}
