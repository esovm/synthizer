/// should_pass: expects no errors.
/// should_fail: expects errors.
/// should_warn: expects warnings.
#[macro_export]
macro_rules! run_test {
    ( $( $prop:ident ( $( $val:ident ),* ) ),*
      => $source:expr ) => {{
        use ::interpreter::common::Context;
        use ::interpreter::compiler::Compiler;

        use std::collections::HashSet;

        let mut should_pass: HashSet<&'static str> = HashSet::new();
        let mut should_fail: HashSet<&'static str> = HashSet::new();
        let mut should_warn: HashSet<&'static str> = HashSet::new();
        let mut should_run: HashSet<&'static str> = HashSet::new();
        $(
            $(
                let val = stringify!($val);
                should_run.insert(val);
                match stringify!($prop) {
                    "should_pass" => { should_pass.insert(val); },
                    "should_fail" => { should_fail.insert(val); },
                    "should_warn" => { should_warn.insert(val); },
                    _ => panic!("unknown property"),
                }
            )*
        )*
        let ctxt = Context::new("<test>".into(), $source.into());
        let mut compiler = Compiler::new(&ctxt);
        compiler.define_intrinsics();

        if should_run.contains(&"lex") {
            compiler.lex();
            let err = ctxt.issues.borrow().has_errors();
            let warn = ctxt.issues.borrow().has_warnings();
            if should_pass.contains(&"lex") && err {
                panic!("lex should have passed:\n{}", *ctxt.issues.borrow());
            }
            if should_fail.contains(&"lex") && !err {
                panic!("lex should have produced errors:\n{}", *ctxt.issues.borrow());
            }
            if should_warn.contains(&"lex") && !warn {
                panic!("lex should have produced warnings:\n{}", *ctxt.issues.borrow());
            }
            ctxt.issues.borrow_mut().clear();
        }

        if should_run.contains(&"parse") {
            compiler.parse();
            let err = ctxt.issues.borrow().has_errors();
            let warn = ctxt.issues.borrow().has_warnings();
            if should_pass.contains(&"parse") && err {
                panic!("parse should have passed:\n{}", *ctxt.issues.borrow());
            }
            if should_fail.contains(&"parse") && !err {
                panic!("parse should have produced errors:\n{}", *ctxt.issues.borrow());
            }
            if should_warn.contains(&"parse") && !warn {
                panic!("parse should have produced warnings:\n{}", *ctxt.issues.borrow());
            }
            ctxt.issues.borrow_mut().clear();
        }

        if should_run.contains(&"typecheck") {
            compiler.typecheck();
            let err = ctxt.issues.borrow().has_errors();
            let warn = ctxt.issues.borrow().has_warnings();
            if should_pass.contains(&"typecheck") && err {
                panic!("typecheck should have passed:\n{}", *ctxt.issues.borrow());
            }
            if should_fail.contains(&"typecheck") && !err {
                panic!("typecheck should have produced errors:\n{}", *ctxt.issues.borrow());
            }
            if should_warn.contains(&"typecheck") && !warn {
                panic!("typecheck should have produced warnings:\n{}", *ctxt.issues.borrow());
            }
            ctxt.issues.borrow_mut().clear();
        }

        if should_run.contains(&"codegen") {
            compiler.codegen();
            let err = ctxt.issues.borrow().has_errors();
            let warn = ctxt.issues.borrow().has_warnings();
            if should_pass.contains(&"codegen") && err {
                panic!("codegen should have passed:\n{}", *ctxt.issues.borrow());
            }
            if should_fail.contains(&"codegen") && !err {
                panic!("codegen should have produced errors:\n{}", *ctxt.issues.borrow());
            }
            if should_warn.contains(&"codegen") && !warn {
                panic!("codegen should have produced warnings:\n{}", *ctxt.issues.borrow());
            }
        }
    }};
}
