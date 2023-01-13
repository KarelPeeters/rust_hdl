//! This Source Code Form is subject to the terms of the Mozilla Public
//! License, v. 2.0. If a copy of the MPL was not distributed with this file,
//! You can obtain one at http://mozilla.org/MPL/2.0/.
//!
//! Copyright (c) 2018, Olof Kraigher olof.kraigher@gmail.com
#![allow(clippy::only_used_in_recursion)]

use super::analyze::*;
use super::region::*;
use super::target::AssignmentType;
use crate::ast::*;
use crate::data::*;

impl<'a> AnalyzeContext<'a> {
    // @TODO maybe make generic function for expression/waveform.
    // wait until type checking to see if it makes sense
    pub fn analyze_expr_assignment(
        &self,
        scope: &Scope<'_>,
        target: &mut WithPos<Target>,
        assignment_type: AssignmentType,
        rhs: &mut AssignmentRightHand<WithPos<Expression>>,
        diagnostics: &mut dyn DiagnosticHandler,
    ) -> FatalNullResult {
        let ttyp = self.resolve_target(scope, target, assignment_type, diagnostics)?;
        match rhs {
            AssignmentRightHand::Simple(expr) => {
                self.analyze_expression_for_target(scope, ttyp.as_ref(), expr, diagnostics)?;
            }
            AssignmentRightHand::Conditional(conditionals) => {
                let Conditionals {
                    conditionals,
                    else_item,
                } = conditionals;
                for conditional in conditionals {
                    let Conditional { condition, item } = conditional;
                    self.analyze_expression_for_target(scope, ttyp.as_ref(), item, diagnostics)?;
                    self.analyze_expression(scope, condition, diagnostics)?;
                }
                if let Some(expr) = else_item {
                    self.analyze_expression_for_target(scope, ttyp.as_ref(), expr, diagnostics)?;
                }
            }
            AssignmentRightHand::Selected(selection) => {
                let Selection {
                    expression,
                    alternatives,
                } = selection;
                self.analyze_expression(scope, expression, diagnostics)?;
                for Alternative { choices, item } in alternatives.iter_mut() {
                    self.analyze_expression_for_target(scope, ttyp.as_ref(), item, diagnostics)?;
                    self.analyze_choices(scope, choices, diagnostics)?;
                }
            }
        }
        Ok(())
    }

    pub fn analyze_waveform_assignment(
        &self,
        scope: &Scope<'_>,
        target: &mut WithPos<Target>,
        assignment_type: AssignmentType,
        rhs: &mut AssignmentRightHand<Waveform>,
        diagnostics: &mut dyn DiagnosticHandler,
    ) -> FatalNullResult {
        self.resolve_target(scope, target, assignment_type, diagnostics)?;
        match rhs {
            AssignmentRightHand::Simple(wavf) => {
                self.analyze_waveform(scope, wavf, diagnostics)?;
            }
            AssignmentRightHand::Conditional(conditionals) => {
                let Conditionals {
                    conditionals,
                    else_item,
                } = conditionals;
                for conditional in conditionals {
                    let Conditional { condition, item } = conditional;
                    self.analyze_waveform(scope, item, diagnostics)?;
                    self.analyze_expression(scope, condition, diagnostics)?;
                }
                if let Some(wavf) = else_item {
                    self.analyze_waveform(scope, wavf, diagnostics)?;
                }
            }
            AssignmentRightHand::Selected(selection) => {
                let Selection {
                    expression,
                    alternatives,
                } = selection;
                self.analyze_expression(scope, expression, diagnostics)?;
                for Alternative { choices, item } in alternatives.iter_mut() {
                    self.analyze_waveform(scope, item, diagnostics)?;
                    self.analyze_choices(scope, choices, diagnostics)?;
                }
            }
        }
        Ok(())
    }

    fn analyze_waveform(
        &self,
        scope: &Scope<'_>,
        wavf: &mut Waveform,
        diagnostics: &mut dyn DiagnosticHandler,
    ) -> FatalNullResult {
        match wavf {
            Waveform::Elements(ref mut elems) => {
                for elem in elems.iter_mut() {
                    let WaveformElement { value, after } = elem;
                    self.analyze_expression(scope, value, diagnostics)?;
                    if let Some(expr) = after {
                        self.analyze_expression(scope, expr, diagnostics)?;
                    }
                }
            }
            Waveform::Unaffected => {}
        }
        Ok(())
    }

    pub fn analyze_expression_for_target(
        &self,
        scope: &Scope<'_>,
        ttyp: Option<&TypeEnt>,
        expr: &mut WithPos<Expression>,
        diagnostics: &mut dyn DiagnosticHandler,
    ) -> FatalNullResult {
        if let Some(ttyp) = ttyp {
            self.analyze_expression_with_target_type(
                scope,
                ttyp,
                &expr.pos,
                &mut expr.item,
                diagnostics,
            )?;
        } else {
            self.analyze_expression_pos(scope, &expr.pos, &mut expr.item, diagnostics)?;
        }
        Ok(())
    }
}