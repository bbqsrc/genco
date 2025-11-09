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

    AsyncModifier(AsyncModifier) {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            match self {
                AsyncModifier::Async => out.write_str("async"),
                AsyncModifier::Await => out.write_str("await"),
                AsyncModifier::Throws => out.write_str("throws"),
            }
        }
    }

    IsolationModifier(IsolationModifier) {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            match self {
                IsolationModifier::Nonisolated => out.write_str("nonisolated"),
                IsolationModifier::Isolated => out.write_str("isolated"),
            }
        }
    }

    ActorType(ActorType) {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            out.write_str("actor")
        }
    }

    TypeModifier(TypeModifier) {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            match self {
                TypeModifier::Any => out.write_str("any"),
                TypeModifier::Some => out.write_str("some"),
            }
        }
    }

    TypedThrows(TypedThrows) {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            out.write_str("throws(")?;
            out.write_str(&self.error_type)?;
            out.write_char(')')
        }
    }

    ObservableDecorator(ObservableDecorator) {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            out.write_str("@Observable")
        }
    }

    GlobalActor(GlobalActor) {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            out.write_char('@')?;
            out.write_str(&self.name)
        }
    }

    AccessModifier(AccessModifier) {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            match self {
                AccessModifier::Package => out.write_str("package"),
            }
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
#[derive(Debug, Clone, Copy, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct ResultBuilder {}

/// MainActor decorator for actor isolation.
///
/// Marks types or functions as isolated to the main actor.
///
/// Created through the [main_actor()] function.
#[derive(Debug, Clone, Copy, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct MainActor {}

/// Async modifier for concurrency (Swift 5.5+).
///
/// Swift introduced structured concurrency with async/await in Swift 5.5.
///
/// Created through helper functions like [async_modifier()], [await_keyword()], or [throws_modifier()].
#[derive(Debug, Clone, Copy, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum AsyncModifier {
    /// `async` - Marks a function as asynchronous
    Async,
    /// `await` - Used to call async functions
    Await,
    /// `throws` - Marks a function that can throw errors
    Throws,
}

/// Isolation modifier for actor isolation (Swift 5.5+).
///
/// Controls how declarations interact with actor isolation.
///
/// Created through helper functions like [nonisolated_modifier()] or [isolated_modifier()].
#[derive(Debug, Clone, Copy, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum IsolationModifier {
    /// `nonisolated` - Opts out of actor isolation
    Nonisolated,
    /// `isolated` - Explicitly isolated to an actor
    Isolated,
}

/// Actor type keyword (Swift 5.5+).
///
/// Declares an actor type for safe concurrent access.
///
/// Created through the [actor_type()] function.
#[derive(Debug, Clone, Copy, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct ActorType {}

/// Type modifier for existential and opaque types (Swift 5.6+).
///
/// Swift 5.6 introduced `any` for existential types and `some` for opaque types.
///
/// Created through helper functions like [any_type()] or [some_type()].
#[derive(Debug, Clone, Copy, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum TypeModifier {
    /// `any` - Existential type (type-erased protocol)
    Any,
    /// `some` - Opaque type (concrete type with preserved identity)
    Some,
}

/// Typed throws for specific error types (Swift 6.0+).
///
/// Swift 6.0 allows specifying the error type a function can throw.
///
/// Created through the [typed_throws()] function.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct TypedThrows {
    /// The error type (e.g., "NetworkError")
    error_type: ItemStr,
}

/// Observable decorator (Swift 5.9+).
///
/// The @Observable macro simplifies observable object creation.
///
/// Created through the [observable()] function.
#[derive(Debug, Clone, Copy, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct ObservableDecorator {}

/// Custom global actor decorator (Swift 5.5+).
///
/// Custom global actors for actor isolation.
///
/// Created through the [global_actor()] function.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct GlobalActor {
    /// The global actor name (e.g., "UIActor", "DatabaseActor")
    name: ItemStr,
}

/// Access control modifier (Swift 5.9+).
///
/// Swift 5.9 introduced `package` access control.
///
/// Created through helper functions like [package_access()].
#[derive(Debug, Clone, Copy, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum AccessModifier {
    /// `package` - Package-level access control
    Package,
}

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
                | AnyKind::MainActor(_)
                | AnyKind::AsyncModifier(_)
                | AnyKind::IsolationModifier(_)
                | AnyKind::ActorType(_)
                | AnyKind::TypeModifier(_)
                | AnyKind::TypedThrows(_)
                | AnyKind::ObservableDecorator(_)
                | AnyKind::GlobalActor(_)
                | AnyKind::AccessModifier(_) => {}
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

/// Creates an `async` modifier for async functions.
///
/// The `async` keyword marks a function as asynchronous in Swift's structured concurrency.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let async_mod = swift::async_modifier();
/// let toks = quote!(func fetchData() $async_mod -> Data);
///
/// assert_eq!("func fetchData() async -> Data", toks.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn async_modifier() -> AsyncModifier {
    AsyncModifier::Async
}

/// Creates an `await` keyword for calling async functions.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let await_kw = swift::await_keyword();
/// let toks = quote!(let data = $await_kw fetchData());
///
/// assert_eq!("let data = await fetchData()", toks.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn await_keyword() -> AsyncModifier {
    AsyncModifier::Await
}

/// Creates a `throws` modifier for functions that can throw errors.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let throws_mod = swift::throws_modifier();
/// let toks = quote!(func riskyOperation() $throws_mod -> Result);
///
/// assert_eq!("func riskyOperation() throws -> Result", toks.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn throws_modifier() -> AsyncModifier {
    AsyncModifier::Throws
}

/// Creates a `nonisolated` modifier for opting out of actor isolation.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let nonisolated_mod = swift::nonisolated_modifier();
/// let toks = quote!($nonisolated_mod func helper() -> String);
///
/// assert_eq!("nonisolated func helper() -> String", toks.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn nonisolated_modifier() -> IsolationModifier {
    IsolationModifier::Nonisolated
}

/// Creates an `isolated` modifier for explicit actor isolation.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let isolated_mod = swift::isolated_modifier();
/// let toks = quote!(func process($isolated_mod actor: MyActor));
///
/// assert_eq!("func process(isolated actor: MyActor)", toks.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn isolated_modifier() -> IsolationModifier {
    IsolationModifier::Isolated
}

/// Creates an `actor` keyword for declaring actor types.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let actor_kw = swift::actor_type();
/// let toks = quote! {
///     $actor_kw Counter {
///         var value: Int = 0
///     }
/// };
///
/// assert_eq!(
///     vec![
///         "actor Counter {",
///         "    var value: Int = 0",
///         "}",
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn actor_type() -> ActorType {
    ActorType {}
}

/// Creates an `any` type modifier for existential types.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let any_type = swift::any_type();
/// let toks = quote!(let items: [$any_type Collection]);
///
/// assert_eq!("let items: [any Collection]", toks.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn any_type() -> TypeModifier {
    TypeModifier::Any
}

/// Creates a `some` type modifier for opaque types.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let some_type = swift::some_type();
/// let toks = quote!(var body: $some_type View);
///
/// assert_eq!("var body: some View", toks.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn some_type() -> TypeModifier {
    TypeModifier::Some
}

/// Creates a typed throws specification.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let typed_throws = swift::typed_throws("NetworkError");
/// let toks = quote!(func fetch() $typed_throws -> Data);
///
/// assert_eq!("func fetch() throws(NetworkError) -> Data", toks.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn typed_throws(error_type: impl Into<ItemStr>) -> TypedThrows {
    TypedThrows {
        error_type: error_type.into(),
    }
}

/// Creates an `@Observable` decorator.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let observable = swift::observable();
/// let toks = quote! {
///     $observable
///     class DataModel {
///         var name: String = ""
///     }
/// };
///
/// assert_eq!(
///     vec![
///         "@Observable",
///         "class DataModel {",
///         "    var name: String = \"\"",
///         "}",
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn observable() -> ObservableDecorator {
    ObservableDecorator {}
}

/// Creates a custom global actor decorator.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let ui_actor = swift::global_actor("UIActor");
/// let toks = quote! {
///     $ui_actor
///     class UIManager {
///     }
/// };
///
/// assert_eq!(
///     vec![
///         "@UIActor",
///         "class UIManager {",
///         "}",
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn global_actor(name: impl Into<ItemStr>) -> GlobalActor {
    GlobalActor {
        name: name.into(),
    }
}

/// Creates a `package` access modifier.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let package_mod = swift::package_access();
/// let toks = quote!($package_mod class InternalUtility {});
///
/// assert_eq!("package class InternalUtility {}", toks.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn package_access() -> AccessModifier {
    AccessModifier::Package
}
