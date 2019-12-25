// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2019, Olof Kraigher olof.kraigher@gmail.com

use crate::analysis::library::DesignRoot;
use crate::diagnostic::Diagnostic;
use crate::latin_1::Latin1String;
use crate::source::Source;
use crate::symbol_table::{Symbol, SymbolTable};
use crate::test_util::*;
use pretty_assertions::assert_eq;
use std::collections::{hash_map::Entry, HashMap};
use std::sync::Arc;

pub struct LibraryBuilder {
    code_builder: CodeBuilder,
    libraries: HashMap<Symbol, Vec<Code>>,
}

impl LibraryBuilder {
    pub fn new() -> LibraryBuilder {
        LibraryBuilder {
            code_builder: CodeBuilder::new(),
            libraries: HashMap::default(),
        }
    }

    fn add_code(&mut self, library_name: &str, code: Code) {
        let library_name = self.code_builder.symbol(library_name);
        match self.libraries.entry(library_name) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().push(code.clone());
            }
            Entry::Vacant(entry) => {
                entry.insert(vec![code.clone()]);
            }
        }
    }

    pub fn code(&mut self, library_name: &str, code: &str) -> Code {
        let code = self.code_builder.code(code);
        self.add_code(library_name, code.clone());
        code
    }

    pub fn get_analyzed_root(&self) -> (DesignRoot, Vec<Diagnostic>) {
        let mut root = DesignRoot::new(self.code_builder.symtab.clone());
        let mut diagnostics = Vec::new();

        add_standard_library(self.symtab(), &mut root);

        for (library_name, codes) in self.libraries.iter() {
            for code in codes {
                root.add_design_file(library_name.clone(), code.design_file());
            }
        }

        root.analyze(&mut diagnostics);

        (root, diagnostics)
    }

    pub fn take_code(self) -> Vec<(Symbol, Code)> {
        let mut res = Vec::new();
        for (library_name, codes) in self.libraries.into_iter() {
            for code in codes.into_iter() {
                res.push((library_name.clone(), code));
            }
        }
        res
    }

    pub fn symtab(&self) -> Arc<SymbolTable> {
        self.code_builder.symtab.clone()
    }

    pub fn analyze(&self) -> Vec<Diagnostic> {
        self.get_analyzed_root().1
    }
}

pub fn add_standard_library(symtab: Arc<SymbolTable>, root: &mut DesignRoot) {
    let builder = CodeBuilder {
        symtab: symtab.clone(),
    };
    let std_standard = builder.code_from_source(Source::inline(
        "standard.vhd",
        &Latin1String::new(include_bytes!(
            "../../../../example_project/vhdl_libraries/2008/std/standard.vhd"
        ))
        .to_string(),
    ));
    let std_textio = builder.code_from_source(Source::inline(
        "textio.vhd",
        &Latin1String::new(include_bytes!(
            "../../../../example_project/vhdl_libraries/2008/std/textio.vhd"
        ))
        .to_string(),
    ));
    let std_env = builder.code_from_source(Source::inline(
        "env.vhd",
        &Latin1String::new(include_bytes!(
            "../../../../example_project/vhdl_libraries/2008/std/env.vhd"
        ))
        .to_string(),
    ));
    let std_sym = symtab.insert_utf8("std");

    root.add_design_file(std_sym.clone(), std_standard.design_file());
    root.add_design_file(std_sym.clone(), std_textio.design_file());
    root.add_design_file(std_sym.clone(), std_env.design_file());
}

pub fn missing(code: &Code, name: &str, occ: usize) -> Diagnostic {
    Diagnostic::error(code.s(name, occ), format!("No declaration of '{}'", name))
}

pub fn duplicate(code: &Code, name: &str, occ1: usize, occ2: usize) -> Diagnostic {
    Diagnostic::error(
        code.s(&name, occ2),
        format!("Duplicate declaration of '{}'", &name),
    )
    .related(code.s(&name, occ1), "Previously defined here")
}

pub fn duplicates(code: &Code, names: &[&str]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for name in names {
        diagnostics.push(duplicate(code, name, 1, 2));
    }
    diagnostics
}

pub fn duplicate_in_two_files(code1: &Code, code2: &Code, names: &[&str]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for name in names {
        diagnostics.push(
            Diagnostic::error(
                code2.s1(&name),
                format!("Duplicate declaration of '{}'", &name),
            )
            .related(code1.s1(&name), "Previously defined here"),
        )
    }
    diagnostics
}

pub fn check_missing(contents: &str) {
    let mut builder = LibraryBuilder::new();
    let code = builder.code("libname", contents);
    let diagnostics = builder.analyze();
    let occurences = contents.matches("missing").count();
    assert!(occurences > 0);
    check_diagnostics(
        diagnostics,
        (1..=occurences)
            .map(|idx| missing(&code, "missing", idx))
            .collect(),
    );
}

pub fn check_search_reference(contents: &str) {
    let mut builder = LibraryBuilder::new();
    let code = builder.code("libname", contents);
    let occurences = contents.matches("decl").count();
    assert!(occurences > 0);

    let (root, diagnostics) = builder.get_analyzed_root();
    check_no_diagnostics(&diagnostics);

    let mut references = Vec::new();
    for idx in 1..=occurences {
        assert_eq!(
            root.search_reference(code.source(), code.s("decl", idx).end()),
            Some(code.s("decl", 1).pos()),
            "{}",
            idx
        );
        references.push(code.s("decl", idx).pos());
    }
    assert_eq!(
        root.find_all_references(&code.s("decl", 1).pos()),
        references,
    );
}
