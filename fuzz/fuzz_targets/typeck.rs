use full_moon::ast::luau::{TypeAssertion, TypeInfo, TypeSpecifier};
use full_moon::visitors::Visitor;
use loretta_fuzz::{fuzz_input, run_iters};

/// typeck = check type annotations: parse under Luau (the dialect whose type
/// syntax this is) and walk every type node the way a checker would.
struct TypeChecker {
    infos: usize,
    assertions: usize,
    specifiers: usize,
}

impl Visitor for TypeChecker {
    fn visit_type_info(&mut self, _: &TypeInfo) {
        self.infos += 1;
    }

    fn visit_type_assertion(&mut self, _: &TypeAssertion) {
        self.assertions += 1;
    }

    fn visit_type_specifier(&mut self, _: &TypeSpecifier) {
        self.specifiers += 1;
    }
}

fn target(data: &[u8]) {
    let text = String::from_utf8_lossy(data);
    let result = full_moon::parse_fallible(&text, full_moon::ast::LuaVersion::new().with_luau());
    let mut checker = TypeChecker {
        infos: 0,
        assertions: 0,
        specifiers: 0,
    };
    checker.visit_ast(result.ast());
    let _ = (checker.infos, checker.assertions, checker.specifiers);
}

fn main() {
    let (iters, seed) = fuzz_input();
    run_iters("typeck", iters, seed, target);
}
