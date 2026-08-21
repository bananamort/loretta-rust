use full_moon::ast::luau::{ExportedTypeDeclaration, TypeDeclaration, TypeInfo};
use full_moon::visitors::Visitor;
use loretta_fuzz::{fuzz_input, run_iters};

/// typeck_defs = type definitions: parse under Luau and walk every
/// `type X = ...` / `export type X = ...` declaration body.
struct TypeDefChecker {
    declarations: usize,
    exports: usize,
    infos: usize,
}

impl Visitor for TypeDefChecker {
    fn visit_type_declaration(&mut self, _: &TypeDeclaration) {
        self.declarations += 1;
    }

    fn visit_exported_type_declaration(&mut self, _: &ExportedTypeDeclaration) {
        self.exports += 1;
    }

    fn visit_type_info(&mut self, _: &TypeInfo) {
        self.infos += 1;
    }
}

fn target(data: &[u8]) {
    let text = String::from_utf8_lossy(data);
    let result = full_moon::parse_fallible(&text, full_moon::ast::LuaVersion::new().with_luau());
    let mut checker = TypeDefChecker {
        declarations: 0,
        exports: 0,
        infos: 0,
    };
    checker.visit_ast(result.ast());
    let _ = (checker.declarations, checker.exports, checker.infos);
}

fn main() {
    let (iters, seed) = fuzz_input();
    run_iters("typeck_defs", iters, seed, target);
}
