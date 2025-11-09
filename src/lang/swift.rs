//! Specialization for Swift code generation.
//!
//! # String Quoting in Swift
//!
//! Swift uses UTF-8 internally, string quoting is with the exception of escape
//! sequences a one-to-one translation.
//!
//! ```rust
//! use genco::prelude::*;
//!
//! let toks: swift::Tokens = quote!("start π 😊 \n \x7f ÿ $ end");
//! assert_eq!("\"start π 😊 \\n \\u{7f} ÿ $ end\"", toks.to_string()?);
//! # Ok::<_, genco::fmt::Error>(())
//! ```

use core::fmt::Write as _;

use alloc::collections::BTreeSet;

use crate::fmt;
use crate::tokens::ItemStr;

/// Tokens container specialization for Rust.
pub type Tokens = crate::Tokens<Swift>;

impl_lang! {
    /// Swift token specialization.
    pub Swift {
        type Config = Config;
        type Format = Format;
        type Item = Any;

        fn write_quoted(out: &mut fmt::Formatter<'_>, input: &str) -> fmt::Result {
            // From: https://docs.swift.org/swift-book/LanguageGuide/StringsAndCharacters.html

            for c in input.chars() {
                match c {
                    '\0' => out.write_str("\\0")?,
                    '\\' => out.write_str("\\\\")?,
                    '\t' => out.write_str("\\t")?,
                    '\n' => out.write_str("\\n")?,
                    '\r' => out.write_str("\\r")?,
                    '\'' => out.write_str("\\'")?,
                    '"' => out.write_str("\\\"")?,
                    c if !c.is_control() => out.write_char(c)?,
                    c => {
                        write!(out, "\\u{{{:x}}}", c as u32)?;
                    }
                };
            }

            Ok(())
        }

        fn format_file(
            tokens: &Tokens,
            out: &mut fmt::Formatter<'_>,
            config: &Self::Config,
        ) -> fmt::Result {
            let mut imports = Tokens::new();
            Self::imports(&mut imports, tokens);
            let format = Format::default();
            imports.format(out, config, &format)?;
            tokens.format(out, config, &format)?;
            Ok(())
        }
    }

    Import(Import) {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            out.write_str(&self.name)
        }
    }

    ImportImplementationOnly(ImportImplementationOnly) {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            out.write_str(&self.name)
        }
    }

    OwnershipModifier(OwnershipModifier) {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            match self {
                OwnershipModifier::Consuming => out.write_str("consuming"),
                OwnershipModifier::Borrowing => out.write_str("borrowing"),
                OwnershipModifier::Inout => out.write_str("inout"),
            }
        }
    }

    ProtocolConformance(ProtocolConformance) {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            if self.negated {
                out.write_char('~')?;
            }
            out.write_str(&self.protocol)
        }
    }

    PropertyWrapper(PropertyWrapper) {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            out.write_char('@')?;
            out.write_str(&self.name)?;
            if let Some(ref args) = self.arguments {
                out.write_char('(')?;
                out.write_str(args)?;
                out.write_char(')')?;
            }
            Ok(())
        }
    }

    AttachedMacro(AttachedMacro) {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            out.write_str("@attached(")?;
            out.write_str(&self.macro_type)?;
            if !self.names.is_empty() {
                out.write_str(", names: ")?;
                out.write_str(&self.names)?;
            }
            out.write_char(')')
        }
    }

    FreestandingMacro(FreestandingMacro) {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            out.write_str("@freestanding(")?;
            out.write_str(&self.macro_type)?;
            out.write_char(')')
        }
    }

    ResultBuilder(ResultBuilder) {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            out.write_str("@resultBuilder")
        }
    }

    MainActor(MainActor) {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            out.write_str("@MainActor")
        }
    }
}

/// Format state for Swift code.
#[derive(Debug, Default)]
pub struct Format {}

/// Configuration for formatting Swift code.
#[derive(Debug, Default)]
pub struct Config {}

/// The import of a Swift type `import UIKit`.
///
/// Created through the [import()] function.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Import {
    /// Module of the imported name.
    module: ItemStr,
    /// Name imported.
    name: ItemStr,
}

/// The implementation-only import of a Swift type `@_implementationOnly import UIKit`.
///
/// Created through the [import_implementation_only()] function.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct ImportImplementationOnly {
    /// Module of the imported name.
    module: ItemStr,
    /// Name imported.
    name: ItemStr,
}

/// The type of import statement to use when importing a Swift module.
/// - Standard imports that make the module's public API available
/// - Implementation-only imports that hide the imported module from clients
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
enum ImportType {
    /// A standard Swift import statement: `import ModuleName`
    Import,
    /// An implementation-only import statement: `@_implementationOnly import ModuleName`
    ///
    /// This type of import hides the imported module from the public API,
    /// preventing clients from depending on it transitively.
    ImportImplementationOnly,
}

/// Ownership modifier for function parameters (Swift 6.0+).
///
/// Swift 6.0 introduced explicit ownership modifiers to control how values are passed:
/// - `consuming` - Takes ownership of the value (move semantics)
/// - `borrowing` - Borrows the value immutably (no ownership transfer)
/// - `inout` - Borrows the value mutably
///
/// Created through helper functions like [consuming()], [borrowing()], or [inout_modifier()].
#[derive(Debug, Clone, Copy, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum OwnershipModifier {
    /// `consuming` - Takes ownership of the value (move semantics)
    Consuming,
    /// `borrowing` - Borrows the value immutably
    Borrowing,
    /// `inout` - Borrows the value mutably
    Inout,
}

/// Protocol conformance with optional negation (Swift 6.0+).
///
/// Swift 6.0 introduced negative types with `~Copyable` to create non-copyable types.
/// This struct represents a protocol conformance that can be negated.
///
/// Created through the [protocol_conformance()] or [non_copyable()] functions.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct ProtocolConformance {
    /// Protocol name (e.g., "Copyable", "Sendable")
    protocol: ItemStr,
    /// Whether this is a negative constraint (e.g., `~Copyable`)
    negated: bool,
}

/// Property wrapper decorator (Swift 5.1+, common in Swift 6.0).
///
/// Property wrappers like `@State`, `@Binding`, `@Published` are common in SwiftUI
/// and modern Swift code.
///
/// Created through the [property_wrapper()] function.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct PropertyWrapper {
    /// Wrapper name (e.g., "State", "Binding", "Published")
    name: ItemStr,
    /// Optional arguments (e.g., for @State(initialValue: 0))
    arguments: Option<ItemStr>,
}

/// Attached macro decorator (Swift 5.9+).
///
/// Attached macros can add members, attributes, accessors, peers, or conformances.
///
/// Created through the [attached_macro()] function.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct AttachedMacro {
    /// Type of attached macro: "member", "memberAttribute", "accessor", "peer", "conformance"
    macro_type: ItemStr,
    /// Names specification (e.g., "named(_:)")
    names: ItemStr,
}

/// Freestanding macro decorator (Swift 5.9+).
///
/// Freestanding macros are expression or declaration macros.
///
/// Created through the [freestanding_macro()] function.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct FreestandingMacro {
    /// Type of freestanding macro: "expression" or "declaration"
    macro_type: ItemStr,
}

/// Result builder decorator.
///
/// Result builders enable DSL-style syntax (e.g., SwiftUI view builders).
///
/// Created through the [result_builder()] function.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct ResultBuilder {}

/// MainActor decorator for actor isolation.
///
/// Marks types or functions as isolated to the main actor.
///
/// Created through the [main_actor()] function.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct MainActor {}

impl Swift {
    fn imports(out: &mut Tokens, tokens: &Tokens) {
        use crate as genco;
        use crate::quote_in;

        let mut modules = BTreeSet::new();

        for import in tokens.iter_lang() {
            match import.kind() {
                AnyKind::Import(ref i) => {
                    modules.insert((&i.module, ImportType::Import));
                }
                AnyKind::ImportImplementationOnly(ref i) => {
                    modules.insert((&i.module, ImportType::ImportImplementationOnly));
                }
                // Other types are not imports, so we ignore them here
                AnyKind::OwnershipModifier(_)
                | AnyKind::ProtocolConformance(_)
                | AnyKind::PropertyWrapper(_)
                | AnyKind::AttachedMacro(_)
                | AnyKind::FreestandingMacro(_)
                | AnyKind::ResultBuilder(_)
                | AnyKind::MainActor(_) => {}
            }
        }

        if !modules.is_empty() {
            for (module, import_type) in modules {
                match import_type {
                    ImportType::Import => {
                        quote_in! { *out => $['\r']import $module}
                    }
                    ImportType::ImportImplementationOnly => {
                        quote_in! { *out => $['\r']@_implementationOnly import $module}
                    }
                }
            }
        }

        out.line();
    }
}

/// The import of a Swift type `import UIKit`.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let toks = quote!($(swift::import("Foo", "Debug")));
///
/// assert_eq!(
///     vec![
///         "import Foo",
///         "",
///         "Debug",
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn import<M, N>(module: M, name: N) -> Import
where
    M: Into<ItemStr>,
    N: Into<ItemStr>,
{
    Import {
        module: module.into(),
        name: name.into(),
    }
}

/// The implementation-only import of a Swift type `@_implementationOnly import UIKit`.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let toks = quote!($(swift::import_implementation_only("Foo", "Debug")));
///
/// assert_eq!(
///     vec![
///         "@_implementationOnly import Foo",
///         "",
///         "Debug",
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn import_implementation_only(
    module: impl Into<ItemStr>,
    name: impl Into<ItemStr>,
) -> ImportImplementationOnly {
    ImportImplementationOnly {
        module: module.into(),
        name: name.into(),
    }
}

/// Creates a `consuming` ownership modifier for Swift 6.0+ function parameters.
///
/// The `consuming` keyword indicates that the parameter takes ownership of the value,
/// using move semantics.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let consuming = swift::consuming();
/// let toks = quote!($consuming value: String);
///
/// assert_eq!("consuming value: String", toks.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn consuming() -> OwnershipModifier {
    OwnershipModifier::Consuming
}

/// Creates a `borrowing` ownership modifier for Swift 6.0+ function parameters.
///
/// The `borrowing` keyword indicates that the parameter borrows the value immutably,
/// without taking ownership.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let borrowing = swift::borrowing();
/// let toks = quote!($borrowing value: String);
///
/// assert_eq!("borrowing value: String", toks.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn borrowing() -> OwnershipModifier {
    OwnershipModifier::Borrowing
}

/// Creates an `inout` ownership modifier for Swift function parameters.
///
/// The `inout` keyword indicates that the parameter borrows the value mutably,
/// allowing modifications that are visible to the caller.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let inout_mod = swift::inout_modifier();
/// let toks = quote!($inout_mod value: String);
///
/// assert_eq!("inout value: String", toks.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn inout_modifier() -> OwnershipModifier {
    OwnershipModifier::Inout
}

/// Creates a protocol conformance constraint.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let sendable = swift::protocol_conformance("Sendable", false);
/// let toks = quote!($sendable);
///
/// assert_eq!("Sendable", toks.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn protocol_conformance(
    protocol: impl Into<ItemStr>,
    negated: bool,
) -> ProtocolConformance {
    ProtocolConformance {
        protocol: protocol.into(),
        negated,
    }
}

/// Creates a `~Copyable` constraint for Swift 6.0+ non-copyable types.
///
/// Non-copyable types use move semantics and cannot be implicitly copied,
/// useful for resource-managing types like file handles.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let non_copy = swift::non_copyable();
/// let toks = quote! {
///     struct FileHandle: $non_copy {
///         let descriptor: Int32
///     }
/// };
///
/// assert_eq!(
///     vec![
///         "struct FileHandle: ~Copyable {",
///         "    let descriptor: Int32",
///         "}",
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn non_copyable() -> ProtocolConformance {
    ProtocolConformance {
        protocol: "Copyable".into(),
        negated: true,
    }
}

/// Creates a `~Sendable` constraint for types that are not thread-safe.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let non_sendable = swift::non_sendable();
/// let toks = quote!($non_sendable);
///
/// assert_eq!("~Sendable", toks.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn non_sendable() -> ProtocolConformance {
    ProtocolConformance {
        protocol: "Sendable".into(),
        negated: true,
    }
}

/// Creates a property wrapper decorator.
///
/// Property wrappers are common in SwiftUI for state management.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let state = swift::property_wrapper("State", None::<&str>);
/// let toks = quote!($state var count: Int = 0);
///
/// assert_eq!("@State var count: Int = 0", toks.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
///
/// With arguments:
///
/// ```
/// use genco::prelude::*;
///
/// let published = swift::property_wrapper("Published", Some("initialValue: 0"));
/// let toks = quote!($published var count: Int);
///
/// assert_eq!("@Published(initialValue: 0) var count: Int", toks.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn property_wrapper(
    name: impl Into<ItemStr>,
    arguments: Option<impl Into<ItemStr>>,
) -> PropertyWrapper {
    PropertyWrapper {
        name: name.into(),
        arguments: arguments.map(|a| a.into()),
    }
}

/// Creates an attached macro decorator.
///
/// Attached macros can modify declarations by adding members, attributes, etc.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let macro_decl = swift::attached_macro("member", "named(_:)");
/// let toks = quote!($macro_decl);
///
/// assert_eq!("@attached(member, names: named(_:))", toks.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn attached_macro(
    macro_type: impl Into<ItemStr>,
    names: impl Into<ItemStr>,
) -> AttachedMacro {
    AttachedMacro {
        macro_type: macro_type.into(),
        names: names.into(),
    }
}

/// Creates a freestanding macro decorator.
///
/// Freestanding macros are used as expressions or declarations.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let expr_macro = swift::freestanding_macro("expression");
/// let toks = quote!($expr_macro);
///
/// assert_eq!("@freestanding(expression)", toks.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn freestanding_macro(macro_type: impl Into<ItemStr>) -> FreestandingMacro {
    FreestandingMacro {
        macro_type: macro_type.into(),
    }
}

/// Creates a `@resultBuilder` decorator.
///
/// Result builders enable DSL-style syntax, commonly used in SwiftUI.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let builder = swift::result_builder();
/// let toks = quote! {
///     $builder
///     struct ViewBuilder {
///     }
/// };
///
/// assert_eq!(
///     vec![
///         "@resultBuilder",
///         "struct ViewBuilder {",
///         "}",
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn result_builder() -> ResultBuilder {
    ResultBuilder {}
}

/// Creates a `@MainActor` decorator.
///
/// MainActor marks types or functions as isolated to the main actor,
/// ensuring they run on the main thread.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let main_actor = swift::main_actor();
/// let toks = quote! {
///     $main_actor
///     class ViewController {
///     }
/// };
///
/// assert_eq!(
///     vec![
///         "@MainActor",
///         "class ViewController {",
///         "}",
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn main_actor() -> MainActor {
    MainActor {}
}
