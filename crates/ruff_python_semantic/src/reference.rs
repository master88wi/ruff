use std::ops::Deref;

use bitflags::bitflags;

use ruff_index::{newtype_index, IndexSlice, IndexVec};
use ruff_python_ast::Ranged;
use ruff_source_file::Locator;
use ruff_text_size::TextRange;

use crate::context::ExecutionContext;
use crate::scope::ScopeId;
use crate::{Exceptions, ExpressionId, SemanticModelFlags};

/// A resolved read reference to a name in a program.
#[derive(Debug, Clone)]
pub struct ResolvedReference {
    /// The expression that the reference occurs in. `None` if the reference is a global
    /// reference or a reference via an augmented assignment.
    expression_id: Option<ExpressionId>,
    /// The scope in which the reference is defined.
    scope_id: ScopeId,
    /// The range of the reference in the source code.
    range: TextRange,
    /// The model state in which the reference occurs.
    flags: SemanticModelFlags,
}

impl ResolvedReference {
    /// The expression that the reference occurs in.
    pub const fn expression_id(&self) -> Option<ExpressionId> {
        self.expression_id
    }

    /// The scope in which the reference is defined.
    pub const fn scope_id(&self) -> ScopeId {
        self.scope_id
    }

    /// The [`ExecutionContext`] of the reference.
    pub const fn context(&self) -> ExecutionContext {
        if self.flags.intersects(SemanticModelFlags::TYPING_CONTEXT) {
            ExecutionContext::Typing
        } else {
            ExecutionContext::Runtime
        }
    }

    /// Return `true` if the context is in a type annotation.
    pub const fn in_annotation(&self) -> bool {
        self.flags.intersects(SemanticModelFlags::ANNOTATION)
    }

    /// Return `true` if the context is in a typing-only type annotation.
    pub const fn in_typing_only_annotation(&self) -> bool {
        self.flags
            .intersects(SemanticModelFlags::TYPING_ONLY_ANNOTATION)
    }

    /// Return `true` if the context is in a runtime-required type annotation.
    pub const fn in_runtime_evaluated_annotation(&self) -> bool {
        self.flags
            .intersects(SemanticModelFlags::RUNTIME_EVALUATED_ANNOTATION)
    }

    /// Return `true` if the context is in a type definition.
    pub const fn in_type_definition(&self) -> bool {
        self.flags.intersects(SemanticModelFlags::TYPE_DEFINITION)
    }

    /// Return `true` if the context is in a "simple" string type definition.
    pub const fn in_simple_string_type_definition(&self) -> bool {
        self.flags
            .intersects(SemanticModelFlags::SIMPLE_STRING_TYPE_DEFINITION)
    }

    /// Return `true` if the context is in a "complex" string type definition.
    pub const fn in_complex_string_type_definition(&self) -> bool {
        self.flags
            .intersects(SemanticModelFlags::COMPLEX_STRING_TYPE_DEFINITION)
    }

    /// Return `true` if the context is in a `__future__` type definition.
    pub const fn in_future_type_definition(&self) -> bool {
        self.flags
            .intersects(SemanticModelFlags::FUTURE_TYPE_DEFINITION)
    }

    /// Return `true` if the context is in any kind of deferred type definition.
    pub const fn in_deferred_type_definition(&self) -> bool {
        self.flags
            .intersects(SemanticModelFlags::DEFERRED_TYPE_DEFINITION)
    }

    /// Return `true` if the context is in a forward type reference.
    ///
    /// Includes deferred string types, and future types in annotations.
    ///
    /// ## Examples
    /// ```python
    /// from __future__ import annotations
    ///
    /// from threading import Thread
    ///
    ///
    /// x: Thread  # Forward reference
    /// cast("Thread", x)  # Forward reference
    /// cast(Thread, x)  # Non-forward reference
    /// ```
    pub const fn in_forward_reference(&self) -> bool {
        self.in_simple_string_type_definition()
            || self.in_complex_string_type_definition()
            || (self.in_future_type_definition() && self.in_typing_only_annotation())
    }

    /// Return `true` if the context is in an exception handler.
    pub const fn in_exception_handler(&self) -> bool {
        self.flags.intersects(SemanticModelFlags::EXCEPTION_HANDLER)
    }

    /// Return `true` if the context is in an f-string.
    pub const fn in_f_string(&self) -> bool {
        self.flags.intersects(SemanticModelFlags::F_STRING)
    }

    /// Return `true` if the context is in boolean test.
    pub const fn in_boolean_test(&self) -> bool {
        self.flags.intersects(SemanticModelFlags::BOOLEAN_TEST)
    }

    /// Return `true` if the context is in a `typing::Literal` annotation.
    pub const fn in_literal(&self) -> bool {
        self.flags.intersects(SemanticModelFlags::LITERAL)
    }

    /// Return `true` if the context is in a subscript expression.
    pub const fn in_subscript(&self) -> bool {
        self.flags.intersects(SemanticModelFlags::SUBSCRIPT)
    }

    /// Return `true` if the context is in a type-checking block.
    pub const fn in_type_checking_block(&self) -> bool {
        self.flags
            .intersects(SemanticModelFlags::TYPE_CHECKING_BLOCK)
    }

    /// Return `true` if the context has traversed past the "top-of-file" import boundary.
    pub const fn seen_import_boundary(&self) -> bool {
        self.flags.intersects(SemanticModelFlags::IMPORT_BOUNDARY)
    }

    /// Return `true` if the context has traverse past the `__future__` import boundary.
    pub const fn seen_futures_boundary(&self) -> bool {
        self.flags.intersects(SemanticModelFlags::FUTURES_BOUNDARY)
    }

    /// Return `true` if `__future__`-style type annotations are enabled.
    pub const fn future_annotations(&self) -> bool {
        self.flags
            .intersects(SemanticModelFlags::FUTURE_ANNOTATIONS)
    }
}

impl Ranged for ResolvedReference {
    /// The range of the reference in the source code.
    fn range(&self) -> TextRange {
        self.range
    }
}

/// Id uniquely identifying a read reference in a program.
#[newtype_index]
pub struct ResolvedReferenceId;

/// The references of a program indexed by [`ResolvedReferenceId`].
#[derive(Debug, Default)]
pub(crate) struct ResolvedReferences(IndexVec<ResolvedReferenceId, ResolvedReference>);

impl ResolvedReferences {
    /// Pushes a new [`ResolvedReference`] and returns its [`ResolvedReferenceId`].
    pub(crate) fn push(
        &mut self,
        range: TextRange,
        scope_id: ScopeId,
        expression_id: Option<ExpressionId>,
        flags: SemanticModelFlags,
    ) -> ResolvedReferenceId {
        self.0.push(ResolvedReference {
            expression_id,
            scope_id,
            range,
            flags,
        })
    }
}

impl Deref for ResolvedReferences {
    type Target = IndexSlice<ResolvedReferenceId, ResolvedReference>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// An unresolved read reference to a name in a program.
#[derive(Debug, Clone)]
pub struct UnresolvedReference {
    /// The range of the reference in the source code.
    range: TextRange,
    /// The set of exceptions that were handled when resolution was attempted.
    exceptions: Exceptions,
    /// Flags indicating the context in which the reference occurs.
    flags: UnresolvedReferenceFlags,
}

impl UnresolvedReference {
    /// Returns the name of the reference.
    pub fn name<'a>(&self, locator: &Locator<'a>) -> &'a str {
        locator.slice(self.range)
    }

    /// The range of the reference in the source code.
    pub const fn range(&self) -> TextRange {
        self.range
    }

    /// The set of exceptions that were handled when resolution was attempted.
    pub const fn exceptions(&self) -> Exceptions {
        self.exceptions
    }

    /// Returns `true` if the unresolved reference may be resolved by a wildcard import.
    pub const fn is_wildcard_import(&self) -> bool {
        self.flags
            .intersects(UnresolvedReferenceFlags::WILDCARD_IMPORT)
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug)]
    pub struct UnresolvedReferenceFlags: u8 {
        /// The unresolved reference may be resolved by a wildcard import.
        ///
        /// For example, the reference `x` in the following code may be resolved by the wildcard
        /// import of `module`:
        /// ```python
        /// from module import *
        ///
        /// print(x)
        /// ```
        const WILDCARD_IMPORT = 1 << 0;
    }
}

#[derive(Debug, Default)]
pub(crate) struct UnresolvedReferences(Vec<UnresolvedReference>);

impl UnresolvedReferences {
    /// Pushes a new [`UnresolvedReference`].
    pub(crate) fn push(
        &mut self,
        range: TextRange,
        exceptions: Exceptions,
        flags: UnresolvedReferenceFlags,
    ) {
        self.0.push(UnresolvedReference {
            range,
            exceptions,
            flags,
        });
    }
}

impl Deref for UnresolvedReferences {
    type Target = Vec<UnresolvedReference>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
