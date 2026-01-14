//! Specialization for TypeScript code generation.
//!
//! TypeScript extends JavaScript with static type annotations and additional
//! language features. This module builds on JavaScript's foundation while
//! adding TypeScript-specific capabilities.
//!
//! # Examples
//!
//! Basic example:
//!
//! ```rust
//! use genco::prelude::*;
//!
//! let toks: ts::Tokens = quote! {
//!     function greet(name: string): string {
//!         return "Hello, " + name;
//!     }
//! };
//!
//! assert_eq!(
//!     vec![
//!         "function greet(name: string): string {",
//!         "    return \"Hello, \" + name;",
//!         "}",
//!     ],
//!     toks.to_file_vec()?
//! );
//! # Ok::<_, genco::fmt::Error>(())
//! ```
//!
//! # String Quoting in TypeScript
//!
//! TypeScript uses JavaScript's string quoting rules, with c-style escape
//! sequences and template literals for interpolation.
//!
//! ```rust
//! use genco::prelude::*;
//!
//! let toks: ts::Tokens = quote!("start π 😊 \n \x7f ÿ $ \\ end");
//! assert_eq!("\"start π 😊 \\n \\x7f ÿ $ \\\\ end\"", toks.to_string()?);
//!
//! let toks: ts::Tokens = quote!($(quoted("start π 😊 \n \x7f ÿ $ \\ end")));
//! assert_eq!("\"start π 😊 \\n \\x7f ÿ $ \\\\ end\"", toks.to_string()?);
//! # Ok::<_, genco::fmt::Error>(())
//! ```

use core::fmt::Write as _;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::fmt;
use crate::tokens::{FormatInto, ItemStr};

use relative_path::{RelativePath, RelativePathBuf};

/// Tokens container specialization for TypeScript.
pub type Tokens = crate::Tokens<TypeScript>;

impl crate::lang::LangSupportsEval for TypeScript {}

impl_lang! {
    /// TypeScript language specialization.
    pub TypeScript {
        type Config = Config;
        type Format = Format;
        type Item = Import;

        /// Start a string quote.
        fn open_quote(
            out: &mut fmt::Formatter<'_>,
            _config: &Self::Config,
            _format: &Self::Format,
            has_eval: bool,
        ) -> fmt::Result {
            if has_eval {
                out.write_char('`')?;
            } else {
                out.write_char('"')?;
            }

            Ok(())
        }

        /// End a string quote.
        fn close_quote(
            out: &mut fmt::Formatter<'_>,
            _config: &Self::Config,
            _format: &Self::Format,
            has_eval: bool,
        ) -> fmt::Result {
            if has_eval {
                out.write_char('`')?;
            } else {
                out.write_char('"')?;
            }

            Ok(())
        }

        fn start_string_eval(
            out: &mut fmt::Formatter<'_>,
            _config: &Self::Config,
            _format: &Self::Format,
        ) -> fmt::Result {
            out.write_str("${")?;
            Ok(())
        }

        fn end_string_eval(
            out: &mut fmt::Formatter<'_>,
            _config: &Self::Config,
            _format: &Self::Format,
        ) -> fmt::Result {
            out.write_char('}')?;
            Ok(())
        }

        fn write_quoted(out: &mut fmt::Formatter<'_>, input: &str) -> fmt::Result {
            // TypeScript uses JavaScript's string escaping rules
            // Reference: https://mathiasbynens.be/notes/javascript-escapes

            for c in input.chars() {
                match c {
                    // backspace
                    '\u{0008}' => out.write_str("\\b")?,
                    // form feed
                    '\u{0012}' => out.write_str("\\f")?,
                    // new line
                    '\n' => out.write_str("\\n")?,
                    // carriage return
                    '\r' => out.write_str("\\r")?,
                    // horizontal tab
                    '\t' => out.write_str("\\t")?,
                    // vertical tab
                    '\u{0011}' => out.write_str("\\v")?,
                    // null character
                    '\0' => out.write_str("\\0")?,
                    '"' => out.write_str("\\\"")?,
                    '\\' => out.write_str("\\\\")?,
                    c if !c.is_control() => out.write_char(c)?,
                    c if (c as u32) < 0x100 => {
                        write!(out, "\\x{:02x}", c as u32)?;
                    }
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
            Self::imports(&mut imports, tokens, config);
            let format = Format::default();
            imports.format(out, config, &format)?;
            tokens.format(out, config, &format)?;
            Ok(())
        }
    }

    Import(Import) {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            let name = match self.kind {
                ImportKind::Named => self.alias.as_ref().unwrap_or(&self.name),
                _ => &self.name,
            };

            out.write_str(name)
        }
    }
}

/// Format state for TypeScript.
#[derive(Debug, Default)]
pub struct Format {}

/// Configuration for TypeScript.
#[derive(Debug, Default)]
pub struct Config {
    module_path: Option<RelativePathBuf>,
}

impl Config {
    /// Configure the path to the current module being rendered.
    ///
    /// This setting will determine what path imports are rendered relative
    /// towards. So importing a module from `"foo/bar.ts"`, and setting this to
    /// `"foo/baz.ts"` will cause the import to be rendered relatively as
    /// `"../bar.ts"`.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::prelude::*;
    /// use genco::fmt;
    ///
    /// let foo1 = ts::import(ts::Module::Path("foo/bar.ts".into()), "Foo1");
    /// let foo2 = ts::import(ts::Module::Path("foo/bar.ts".into()), "Foo2");
    /// let react = ts::import("react", "React").into_default();
    ///
    /// let toks: ts::Tokens = quote! {
    ///     $foo1
    ///     $foo2
    ///     $react
    /// };
    ///
    /// let mut w = fmt::VecWriter::new();
    ///
    /// let config = ts::Config::default().with_module_path("foo/baz.ts");
    /// let fmt = fmt::Config::from_lang::<TypeScript>();
    ///
    /// toks.format_file(&mut w.as_formatter(&fmt), &config)?;
    ///
    /// assert_eq!(
    ///     vec![
    ///         "import {Foo1, Foo2} from \"../bar.ts\";",
    ///         "import React from \"react\";",
    ///         "",
    ///         "Foo1",
    ///         "Foo2",
    ///         "React"
    ///     ],
    ///     w.into_vec()
    /// );
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    pub fn with_module_path<M>(self, module_path: M) -> Self
    where
        M: Into<RelativePathBuf>,
    {
        Self {
            module_path: Some(module_path.into()),
        }
    }
}

/// Internal type to determine the kind of import used.
#[derive(Debug, Clone, Copy, Hash, PartialOrd, Ord, PartialEq, Eq)]
enum ImportKind {
    Named,
    Default,
    Wildcard,
}

/// The import of a TypeScript type `import {foo} from "module.ts"`.
///
/// Created through the [import()] function.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Import {
    /// The kind of the import.
    kind: ImportKind,
    /// Module of the imported name.
    module: Module,
    /// Name imported.
    name: ItemStr,
    /// Alias of an imported item.
    ///
    /// If this is set, you'll get an import like:
    ///
    /// ```text
    /// import {<name> as <alias>} from <module>
    /// ```
    alias: Option<ItemStr>,
    /// Whether this is a type-only import.
    type_only: bool,
}

impl Import {
    /// Change alias of imported item.
    ///
    /// This implies that the import is a named import.
    ///
    /// If this is set, you'll get an import like:
    ///
    /// ```text
    /// import {<name> as <alias>} from <module>
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::prelude::*;
    ///
    /// let a = ts::import("collections", "vec");
    /// let b = ts::import("collections", "vec").with_alias("list");
    ///
    /// let toks = quote! {
    ///     $a
    ///     $b
    /// };
    ///
    /// assert_eq!(
    ///     vec![
    ///         "import {vec, vec as list} from \"collections\";",
    ///         "",
    ///         "vec",
    ///         "list",
    ///     ],
    ///     toks.to_file_vec()?
    /// );
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    pub fn with_alias<N>(self, alias: N) -> Self
    where
        N: Into<ItemStr>,
    {
        Self {
            kind: ImportKind::Named,
            alias: Some(alias.into()),
            ..self
        }
    }

    /// Convert into a default import.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::prelude::*;
    ///
    /// let default_vec = ts::import("collections", "defaultVec").into_default();
    ///
    /// let toks = quote!($default_vec);
    ///
    /// assert_eq!(
    ///     vec![
    ///         "import defaultVec from \"collections\";",
    ///         "",
    ///         "defaultVec",
    ///     ],
    ///     toks.to_file_vec()?
    /// );
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    pub fn into_default(self) -> Self {
        Self {
            kind: ImportKind::Default,
            alias: None,
            ..self
        }
    }

    /// Convert into a wildcard import.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::prelude::*;
    ///
    /// let all = ts::import("collections", "all").into_wildcard();
    ///
    /// let toks = quote!($all);
    ///
    /// assert_eq!(
    ///     vec![
    ///         "import * as all from \"collections\";",
    ///         "",
    ///         "all",
    ///     ],
    ///     toks.to_file_vec()?
    /// );
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    pub fn into_wildcard(self) -> Self {
        Self {
            kind: ImportKind::Wildcard,
            alias: None,
            ..self
        }
    }

    /// Mark this import as type-only.
    ///
    /// Type-only imports are erased at runtime and only used for type checking.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::prelude::*;
    ///
    /// let user = ts::import("./types", "User").into_type_only();
    ///
    /// let toks = quote!($user);
    ///
    /// assert_eq!(
    ///     vec![
    ///         "import type {User} from \"./types\";",
    ///         "",
    ///         "User",
    ///     ],
    ///     toks.to_file_vec()?
    /// );
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    pub fn into_type_only(self) -> Self {
        Self {
            type_only: true,
            ..self
        }
    }
}

/// A module being imported.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum Module {
    /// A module imported from a specific path.
    ///
    /// The path will be relativized according to the module specified in the
    /// [Config::with_module_path].
    Path(RelativePathBuf),
    /// A globally imported module.
    Global(ItemStr),
}

impl<'a> From<&'a str> for Module {
    fn from(value: &'a str) -> Self {
        Self::Global(value.into())
    }
}

impl From<String> for Module {
    fn from(value: String) -> Self {
        Self::Global(value.into())
    }
}

impl From<ItemStr> for Module {
    fn from(value: ItemStr) -> Self {
        Self::Global(value)
    }
}

impl TypeScript {
    /// Translate imports into the necessary tokens.
    fn imports(out: &mut Tokens, tokens: &Tokens, config: &Config) {
        use crate as genco;
        use crate::prelude::*;

        let mut modules = BTreeMap::<(&Module, bool), ResolvedModule<'_>>::new();
        let mut wildcards = BTreeSet::new();

        for import in tokens.iter_lang() {
            match import.kind {
                ImportKind::Named => {
                    let module = modules.entry((&import.module, import.type_only)).or_default();

                    module.set.insert(match &import.alias {
                        None => ImportedElement::Plain(&import.name),
                        Some(alias) => ImportedElement::Aliased(&import.name, alias),
                    });
                }
                ImportKind::Default => {
                    let module = modules.entry((&import.module, import.type_only)).or_default();
                    module.default_import = Some(&import.name);
                }
                ImportKind::Wildcard => {
                    wildcards.insert((&import.module, &import.name, import.type_only));
                }
            }
        }

        if modules.is_empty() && wildcards.is_empty() {
            return;
        }

        for (module, name, type_only) in wildcards {
            out.push();
            if type_only {
                quote_in! { *out =>
                    import type * as $name from $(ref t => render_from(t, config.module_path.as_deref(), module));
                }
            } else {
                quote_in! { *out =>
                    import * as $name from $(ref t => render_from(t, config.module_path.as_deref(), module));
                }
            }
        }

        for ((name, type_only), module) in modules {
            out.push();

            if type_only {
                quote_in! { *out =>
                    import type $(ref tokens => {
                        if let Some(default) = module.default_import {
                            tokens.append(ItemStr::from(default));

                            if !module.set.is_empty() {
                                tokens.append(",");
                                tokens.space();
                            }
                        }

                        if !module.set.is_empty() {
                            tokens.append("{");

                            let mut it = module.set.iter().peekable();

                            while let Some(el) = it.next() {
                                match *el {
                                    ImportedElement::Plain(name) => {
                                        tokens.append(name);
                                    },
                                    ImportedElement::Aliased(name, alias) => {
                                        quote_in!(*tokens => $name as $alias);
                                    }
                                }

                                if it.peek().is_some() {
                                    tokens.append(",");
                                    tokens.space();
                                }
                            }

                            tokens.append("}");
                        }
                    }) from $(ref t => render_from(t, config.module_path.as_deref(), name));
                };
            } else {
                quote_in! { *out =>
                    import $(ref tokens => {
                        if let Some(default) = module.default_import {
                            tokens.append(ItemStr::from(default));

                            if !module.set.is_empty() {
                                tokens.append(",");
                                tokens.space();
                            }
                        }

                        if !module.set.is_empty() {
                            tokens.append("{");

                            let mut it = module.set.iter().peekable();

                            while let Some(el) = it.next() {
                                match *el {
                                    ImportedElement::Plain(name) => {
                                        tokens.append(name);
                                    },
                                    ImportedElement::Aliased(name, alias) => {
                                        quote_in!(*tokens => $name as $alias);
                                    }
                                }

                                if it.peek().is_some() {
                                    tokens.append(",");
                                    tokens.space();
                                }
                            }

                            tokens.append("}");
                        }
                    }) from $(ref t => render_from(t, config.module_path.as_deref(), name));
                };
            }
        }

        out.line();

        #[derive(Default)]
        struct ResolvedModule<'a> {
            default_import: Option<&'a ItemStr>,
            set: BTreeSet<ImportedElement<'a>>,
        }

        #[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
        enum ImportedElement<'a> {
            Plain(&'a ItemStr),
            Aliased(&'a ItemStr, &'a ItemStr),
        }

        fn render_from(t: &mut ts::Tokens, module_path: Option<&RelativePath>, module: &Module) {
            quote_in! { *t =>
                $(match (module_path, module) {
                    (_, Module::Global(from)) => $(quoted(from)),
                    (None, Module::Path(path)) => $(quoted(path.as_str())),
                    (Some(module_path), Module::Path(path)) => $(quoted(module_path.relative(path).as_str())),
                })
            }
        }
    }
}

/// The import of a TypeScript type `import {foo} from "module.ts"`.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let default_vec = ts::import("collections", "defaultVec").into_default();
/// let all = ts::import("collections", "all").into_wildcard();
/// let vec = ts::import("collections", "vec");
/// let vec_as_list = ts::import("collections", "vec").with_alias("list");
///
/// let toks = quote! {
///     $default_vec
///     $all
///     $vec
///     $vec_as_list
/// };
///
/// assert_eq!(
///     vec![
///         "import * as all from \"collections\";",
///         "import defaultVec, {vec, vec as list} from \"collections\";",
///         "",
///         "defaultVec",
///         "all",
///         "vec",
///         "list",
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn import<M, N>(module: M, name: N) -> Import
where
    M: Into<Module>,
    N: Into<ItemStr>,
{
    Import {
        kind: ImportKind::Named,
        module: module.into(),
        name: name.into(),
        alias: None,
        type_only: false,
    }
}

/// A TypeScript type annotation.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let name_type = ts::type_ref("string");
/// let age_type = ts::type_ref("number");
///
/// let toks: ts::Tokens = quote! {
///     let name: $name_type = "Alice";
///     let age: $age_type = 30;
/// };
/// # Ok::<_, genco::fmt::Error>(())
/// ```
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct TypeRef {
    name: ItemStr,
    generics: Vec<TypeRef>,
    index: Option<ItemStr>,
}

impl TypeRef {
    /// Create a new type reference.
    pub fn new<N>(name: N) -> Self
    where
        N: Into<ItemStr>,
    {
        Self {
            name: name.into(),
            generics: Vec::new(),
            index: None,
        }
    }

    /// Add generic type parameters.
    pub fn with_generics(self, generics: Vec<TypeRef>) -> Self {
        Self { generics, ..self }
    }

    /// Add indexed access to this type.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::prelude::*;
    ///
    /// // T[P]
    /// let indexed = ts::type_ref("T").indexed_by("P");
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    pub fn indexed_by<I>(self, index: I) -> Self
    where
        I: Into<ItemStr>,
    {
        Self {
            index: Some(index.into()),
            ..self
        }
    }
}

impl FormatInto<TypeScript> for TypeRef {
    fn format_into(self, tokens: &mut Tokens) {
        tokens.append(self.name);

        if !self.generics.is_empty() {
            tokens.append("<");

            for (i, generic) in self.generics.into_iter().enumerate() {
                if i > 0 {
                    tokens.append(",");
                    tokens.space();
                }
                tokens.append(generic);
            }

            tokens.append(">");
        }

        if let Some(index) = self.index {
            tokens.append("[");
            tokens.append(index);
            tokens.append("]");
        }
    }
}

/// Create a TypeScript type reference.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let string_type = ts::type_ref("string");
/// let number_type = ts::type_ref("number");
/// let promise_type = ts::type_ref("Promise").with_generics(vec![ts::type_ref("User")]);
///
/// let toks: ts::Tokens = quote! {
///     let name: $string_type;
///     let age: $number_type;
///     let user: $promise_type;
/// };
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn type_ref<N>(name: N) -> TypeRef
where
    N: Into<ItemStr>,
{
    TypeRef::new(name)
}

/// A generic type parameter for TypeScript interfaces and type aliases.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// // Simple generic parameter
/// let t = ts::generic_param("T");
///
/// // With constraint
/// let constrained = ts::generic_param("T").with_constraint(ts::type_ref("Base"));
///
/// // With default
/// let with_default = ts::generic_param("T").with_default(ts::type_ref("string"));
///
/// // With both
/// let full = ts::generic_param("T")
///     .with_constraint(ts::type_ref("Base"))
///     .with_default(ts::type_ref("DefaultImpl"));
/// # Ok::<_, genco::fmt::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct GenericParam {
    name: ItemStr,
    constraint: Option<Tokens>,
    default: Option<Tokens>,
}

impl GenericParam {
    /// Create a new generic parameter.
    pub fn new<N>(name: N) -> Self
    where
        N: Into<ItemStr>,
    {
        Self {
            name: name.into(),
            constraint: None,
            default: None,
        }
    }

    /// Add a constraint to this generic parameter (e.g., `T extends Base`).
    pub fn with_constraint<C>(self, constraint: C) -> Self
    where
        C: FormatInto<TypeScript>,
    {
        let mut tokens = Tokens::new();
        tokens.append(constraint);
        Self {
            constraint: Some(tokens),
            ..self
        }
    }

    /// Add a default type to this generic parameter (e.g., `T = string`).
    pub fn with_default<D>(self, default: D) -> Self
    where
        D: FormatInto<TypeScript>,
    {
        let mut tokens = Tokens::new();
        tokens.append(default);
        Self {
            default: Some(tokens),
            ..self
        }
    }
}

impl FormatInto<TypeScript> for GenericParam {
    fn format_into(self, tokens: &mut Tokens) {
        tokens.append(self.name);

        if let Some(constraint) = self.constraint {
            tokens.space();
            tokens.append("extends");
            tokens.space();
            tokens.append(constraint);
        }

        if let Some(default) = self.default {
            tokens.space();
            tokens.append("=");
            tokens.space();
            tokens.append(default);
        }
    }
}

/// Create a generic type parameter.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let t = ts::generic_param("T");
/// let constrained = ts::generic_param("T").with_constraint(ts::type_ref("Base"));
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn generic_param<N>(name: N) -> GenericParam
where
    N: Into<ItemStr>,
{
    GenericParam::new(name)
}

/// A TypeScript index signature type.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// // [key: string]: any
/// let string_index = ts::index_signature_string("key", ts::type_ref("any"));
///
/// // [index: number]: string
/// let number_index = ts::index_signature_number("index", ts::type_ref("string"));
/// # Ok::<_, genco::fmt::Error>(())
/// ```
#[derive(Debug, Clone)]
pub enum IndexSignatureKey {
    /// String index signature: `[key: string]`
    String(ItemStr),
    /// Number index signature: `[index: number]`
    Number(ItemStr),
    /// Symbol index signature: `[key: symbol]`
    Symbol(ItemStr),
}

/// A TypeScript index signature for interfaces.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let map = ts::interface("StringMap")
///     .with_index_signature(ts::index_signature_string("key", ts::type_ref("any")));
/// # Ok::<_, genco::fmt::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct IndexSignature {
    key: IndexSignatureKey,
    value_type: TypeRef,
}

impl IndexSignature {
    /// Create a new index signature.
    pub fn new(key: IndexSignatureKey, value_type: TypeRef) -> Self {
        Self { key, value_type }
    }
}

impl FormatInto<TypeScript> for IndexSignature {
    fn format_into(self, tokens: &mut Tokens) {
        tokens.append("[");
        match self.key {
            IndexSignatureKey::String(name) => {
                tokens.append(name);
                tokens.append(":");
                tokens.space();
                tokens.append("string");
            }
            IndexSignatureKey::Number(name) => {
                tokens.append(name);
                tokens.append(":");
                tokens.space();
                tokens.append("number");
            }
            IndexSignatureKey::Symbol(name) => {
                tokens.append(name);
                tokens.append(":");
                tokens.space();
                tokens.append("symbol");
            }
        }
        tokens.append("]");
        tokens.append(":");
        tokens.space();
        tokens.append(self.value_type);
        tokens.append(";");
    }
}

/// Create a string index signature.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let map = ts::interface("StringMap")
///     .with_index_signature(ts::index_signature_string("key", ts::type_ref("any")));
///
/// let toks: ts::Tokens = quote! {
///     $map
/// };
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn index_signature_string<N>(key_name: N, value_type: TypeRef) -> IndexSignature
where
    N: Into<ItemStr>,
{
    IndexSignature::new(IndexSignatureKey::String(key_name.into()), value_type)
}

/// Create a number index signature.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let array = ts::interface("NumberArray")
///     .with_index_signature(ts::index_signature_number("index", ts::type_ref("string")));
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn index_signature_number<N>(key_name: N, value_type: TypeRef) -> IndexSignature
where
    N: Into<ItemStr>,
{
    IndexSignature::new(IndexSignatureKey::Number(key_name.into()), value_type)
}

/// Create a symbol index signature.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let symbol_map = ts::interface("SymbolMap")
///     .with_index_signature(ts::index_signature_symbol("key", ts::type_ref("number")));
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn index_signature_symbol<N>(key_name: N, value_type: TypeRef) -> IndexSignature
where
    N: Into<ItemStr>,
{
    IndexSignature::new(IndexSignatureKey::Symbol(key_name.into()), value_type)
}

/// A TypeScript interface definition.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let user_interface = ts::interface("User")
///     .with_property(ts::property("id", ts::type_ref("number")))
///     .with_property(ts::property("name", ts::type_ref("string")))
///     .with_property(ts::optional_property("email", ts::type_ref("string")));
///
/// // Generic interface
/// let container = ts::interface("Container")
///     .with_generic_params(vec![ts::generic_param("T")])
///     .with_property(ts::property("value", ts::type_ref("T")));
///
/// // Interface with index signature
/// let map = ts::interface("StringMap")
///     .with_index_signature(ts::index_signature_string("key", ts::type_ref("any")));
///
/// let toks: ts::Tokens = quote! {
///     $user_interface
///     $container
///     $map
/// };
/// # Ok::<_, genco::fmt::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct Interface {
    name: ItemStr,
    generic_params: Vec<GenericParam>,
    extends_interfaces: Vec<TypeRef>,
    call_signatures: Vec<CallSignature>,
    construct_signatures: Vec<ConstructSignature>,
    method_signatures: Vec<MethodSignature>,
    properties: Vec<Property>,
    index_signature: Option<IndexSignature>,
}

impl Interface {
    /// Create a new interface.
    pub fn new<N>(name: N) -> Self
    where
        N: Into<ItemStr>,
    {
        Self {
            name: name.into(),
            generic_params: Vec::new(),
            extends_interfaces: Vec::new(),
            call_signatures: Vec::new(),
            construct_signatures: Vec::new(),
            method_signatures: Vec::new(),
            properties: Vec::new(),
            index_signature: None,
        }
    }

    /// Add generic type parameters to the interface.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::prelude::*;
    ///
    /// let container = ts::interface("Container")
    ///     .with_generic_params(vec![ts::generic_param("T")])
    ///     .with_property(ts::property("value", ts::type_ref("T")));
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    pub fn with_generic_params(mut self, generic_params: Vec<GenericParam>) -> Self {
        self.generic_params = generic_params;
        self
    }

    /// Add extends clause to the interface for inheritance.
    ///
    /// Supports both single and multiple interface inheritance.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::prelude::*;
    ///
    /// // Single inheritance
    /// let user = ts::interface("User")
    ///     .with_extends(vec![ts::type_ref("Base")])
    ///     .with_property(ts::property("name", ts::type_ref("string")));
    ///
    /// // Multiple inheritance
    /// let admin = ts::interface("Admin")
    ///     .with_extends(vec![ts::type_ref("User"), ts::type_ref("Permissions")])
    ///     .with_property(ts::property("role", ts::type_ref("string")));
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    pub fn with_extends(mut self, extends_interfaces: Vec<TypeRef>) -> Self {
        self.extends_interfaces = extends_interfaces;
        self
    }

    /// Add a call signature to the interface.
    ///
    /// Call signatures allow an interface to describe a callable type.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::prelude::*;
    ///
    /// let callable = ts::interface("MyFunction")
    ///     .with_call_signature(ts::call_signature(
    ///         vec![ts::param("x", ts::type_ref("number"))],
    ///         Some(ts::type_ref("string"))
    ///     ));
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    pub fn with_call_signature(mut self, call_signature: CallSignature) -> Self {
        self.call_signatures.push(call_signature);
        self
    }

    /// Add a construct signature to the interface.
    ///
    /// Construct signatures allow an interface to describe a constructable type.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::prelude::*;
    ///
    /// let constructable = ts::interface("MyConstructor")
    ///     .with_construct_signature(ts::construct_signature(
    ///         vec![ts::param("x", ts::type_ref("number"))],
    ///         ts::type_ref("MyClass")
    ///     ));
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    pub fn with_construct_signature(mut self, construct_signature: ConstructSignature) -> Self {
        self.construct_signatures.push(construct_signature);
        self
    }

    /// Add a method signature to the interface.
    ///
    /// Method signatures describe methods in interfaces, separate from properties.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::prelude::*;
    ///
    /// let api = ts::interface("API")
    ///     .with_method_signature(ts::method_signature(
    ///         "fetch",
    ///         vec![ts::param("url", ts::type_ref("string"))],
    ///         Some(ts::type_ref("Promise").with_generics(vec![ts::type_ref("Response")]))
    ///     ));
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    pub fn with_method_signature(mut self, method_signature: MethodSignature) -> Self {
        self.method_signatures.push(method_signature);
        self
    }

    /// Add a property to the interface.
    pub fn with_property(mut self, property: Property) -> Self {
        self.properties.push(property);
        self
    }

    /// Add an index signature to the interface.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::prelude::*;
    ///
    /// let map = ts::interface("StringMap")
    ///     .with_index_signature(ts::index_signature_string("key", ts::type_ref("any")))
    ///     .with_property(ts::property("count", ts::type_ref("number")));
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    pub fn with_index_signature(mut self, index_signature: IndexSignature) -> Self {
        self.index_signature = Some(index_signature);
        self
    }
}

impl FormatInto<TypeScript> for Interface {
    fn format_into(self, tokens: &mut Tokens) {
        use crate as genco;
        use crate::quote_in;

        let name = self.name;

        quote_in! { *tokens =>
            interface $name
        };

        // Add generic parameters if present
        if !self.generic_params.is_empty() {
            tokens.append("<");
            for (i, param) in self.generic_params.into_iter().enumerate() {
                if i > 0 {
                    tokens.append(",");
                    tokens.space();
                }
                tokens.append(param);
            }
            tokens.append(">");
        }

        // Add extends clause if present
        if !self.extends_interfaces.is_empty() {
            tokens.space();
            tokens.append("extends");
            tokens.space();
            for (i, type_ref) in self.extends_interfaces.into_iter().enumerate() {
                if i > 0 {
                    tokens.append(",");
                    tokens.space();
                }
                tokens.append(type_ref);
            }
        }

        tokens.space();
        tokens.append("{");
        tokens.indent();

        // Add call signatures first
        for call_sig in self.call_signatures {
            tokens.push();
            tokens.append(call_sig);
        }

        // Then add construct signatures
        for construct_sig in self.construct_signatures {
            tokens.push();
            tokens.append(construct_sig);
        }

        // Then add method signatures
        for method_sig in self.method_signatures {
            tokens.push();
            tokens.append(method_sig);
        }

        // Add index signature
        if let Some(index_sig) = self.index_signature {
            tokens.push();
            tokens.append(index_sig);
        }

        // Finally add properties
        for prop in self.properties {
            tokens.push();
            tokens.append(prop);
        }

        tokens.unindent();
        tokens.push();
        tokens.append("}");
    }
}

/// Create a TypeScript interface.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let user = ts::interface("User")
///     .with_property(ts::property("id", ts::type_ref("number")))
///     .with_property(ts::property("name", ts::type_ref("string")));
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn interface<N>(name: N) -> Interface
where
    N: Into<ItemStr>,
{
    Interface::new(name)
}

/// A property in a TypeScript interface or type.
#[derive(Debug, Clone)]
pub struct Property {
    name: ItemStr,
    type_ref: TypeRef,
    optional: bool,
    readonly: bool,
}

impl Property {
    /// Create a new property.
    pub fn new<N>(name: N, type_ref: TypeRef) -> Self
    where
        N: Into<ItemStr>,
    {
        Self {
            name: name.into(),
            type_ref,
            optional: false,
            readonly: false,
        }
    }

    /// Mark this property as optional.
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    /// Mark this property as readonly.
    pub fn readonly(mut self) -> Self {
        self.readonly = true;
        self
    }
}

impl FormatInto<TypeScript> for Property {
    fn format_into(self, tokens: &mut Tokens) {
        if self.readonly {
            tokens.append("readonly");
            tokens.space();
        }

        tokens.append(self.name);

        if self.optional {
            tokens.append("?");
        }

        tokens.append(":");
        tokens.space();
        tokens.append(self.type_ref);
        tokens.append(";");
    }
}

/// Create a TypeScript property.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let id_prop = ts::property("id", ts::type_ref("number"));
/// let name_prop = ts::property("name", ts::type_ref("string"));
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn property<N>(name: N, type_ref: TypeRef) -> Property
where
    N: Into<ItemStr>,
{
    Property::new(name, type_ref)
}

/// Create an optional TypeScript property.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let email_prop = ts::optional_property("email", ts::type_ref("string"));
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn optional_property<N>(name: N, type_ref: TypeRef) -> Property
where
    N: Into<ItemStr>,
{
    Property::new(name, type_ref).optional()
}

/// A TypeScript type alias.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let user_id = ts::type_alias("UserID", ts::type_ref("string"));
///
/// // Generic type alias
/// let result = ts::type_alias("Result", ts::union_type(vec![
///     ts::type_ref("Success").with_generics(vec![ts::type_ref("T")]),
///     ts::type_ref("Error"),
/// ]))
/// .with_generic_params(vec![ts::generic_param("T")]);
///
/// let toks: ts::Tokens = quote! {
///     $user_id
///     $result
/// };
/// # Ok::<_, genco::fmt::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct TypeAlias {
    name: ItemStr,
    generic_params: Vec<GenericParam>,
    type_expr: Tokens,
}

impl TypeAlias {
    /// Create a new type alias.
    pub fn new<N, T>(name: N, type_expr: T) -> Self
    where
        N: Into<ItemStr>,
        T: FormatInto<TypeScript>,
    {
        let mut tokens = Tokens::new();
        tokens.append(type_expr);
        Self {
            name: name.into(),
            generic_params: Vec::new(),
            type_expr: tokens,
        }
    }

    /// Add generic type parameters to the type alias.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::prelude::*;
    ///
    /// let nullable = ts::type_alias("Nullable", ts::union_type(vec![
    ///     ts::type_ref("T"),
    ///     ts::type_ref("null"),
    /// ]))
    /// .with_generic_params(vec![ts::generic_param("T")]);
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    pub fn with_generic_params(mut self, generic_params: Vec<GenericParam>) -> Self {
        self.generic_params = generic_params;
        self
    }
}

impl FormatInto<TypeScript> for TypeAlias {
    fn format_into(self, tokens: &mut Tokens) {
        tokens.append("type");
        tokens.space();
        tokens.append(self.name);

        // Add generic parameters if present
        if !self.generic_params.is_empty() {
            tokens.append("<");
            for (i, param) in self.generic_params.into_iter().enumerate() {
                if i > 0 {
                    tokens.append(",");
                    tokens.space();
                }
                tokens.append(param);
            }
            tokens.append(">");
        }

        tokens.space();
        tokens.append("=");
        tokens.space();
        tokens.append(self.type_expr);
        tokens.append(";");
    }
}

/// Create a TypeScript type alias.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let user_id = ts::type_alias("UserID", ts::type_ref("string"));
/// let count = ts::type_alias("Count", ts::type_ref("number"));
///
/// // Can also use union types, tuple types, etc.
/// let nullable = ts::type_alias("Nullable", ts::union_type(vec![
///     ts::type_ref("T"),
///     ts::type_ref("null"),
/// ]))
/// .with_generic_params(vec![ts::generic_param("T")]);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn type_alias<N, T>(name: N, type_expr: T) -> TypeAlias
where
    N: Into<ItemStr>,
    T: FormatInto<TypeScript>,
{
    TypeAlias::new(name, type_expr)
}

/// A TypeScript keyof operator.
///
/// Represents the `keyof T` type operator that creates a union of property keys.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let keys = ts::keyof(ts::type_ref("User"));
/// # Ok::<_, genco::fmt::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct KeyofOperator {
    type_ref: TypeRef,
}

impl KeyofOperator {
    /// Create a new keyof operator.
    pub fn new(type_ref: TypeRef) -> Self {
        Self { type_ref }
    }
}

impl FormatInto<TypeScript> for KeyofOperator {
    fn format_into(self, tokens: &mut Tokens) {
        tokens.append("keyof");
        tokens.space();
        tokens.append(self.type_ref);
    }
}

/// Create a TypeScript keyof operator.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let keys = ts::keyof(ts::type_ref("User"));
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn keyof(type_ref: TypeRef) -> KeyofOperator {
    KeyofOperator::new(type_ref)
}

/// Property modifier for mapped types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyModifier {
    /// Add optional modifier: `?`
    Optional,
    /// Add required modifier: `-?`
    Required,
    /// No modifier
    None,
}

/// A TypeScript mapped type.
///
/// Mapped types allow transforming properties of one type into another.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// // type Readonly<T> = { readonly [P in keyof T]: T[P] }
/// let readonly = ts::type_alias("Readonly",
///     ts::mapped_type("P", ts::keyof(ts::type_ref("T")), ts::type_ref("T").indexed_by("P"))
///         .readonly()
/// )
/// .with_generic_params(vec![ts::generic_param("T")]);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct MappedType {
    key_param: ItemStr,
    constraint: Tokens,
    value_type: Tokens,
    readonly: bool,
    modifier: PropertyModifier,
}

impl MappedType {
    /// Create a new mapped type.
    pub fn new<K, C, V>(key_param: K, constraint: C, value_type: V) -> Self
    where
        K: Into<ItemStr>,
        C: FormatInto<TypeScript>,
        V: FormatInto<TypeScript>,
    {
        let mut constraint_tokens = Tokens::new();
        constraint_tokens.append(constraint);

        let mut value_tokens = Tokens::new();
        value_tokens.append(value_type);

        Self {
            key_param: key_param.into(),
            constraint: constraint_tokens,
            value_type: value_tokens,
            readonly: false,
            modifier: PropertyModifier::None,
        }
    }

    /// Mark properties as readonly.
    pub fn readonly(mut self) -> Self {
        self.readonly = true;
        self
    }

    /// Add optional modifier to properties.
    pub fn optional(mut self) -> Self {
        self.modifier = PropertyModifier::Optional;
        self
    }

    /// Add required modifier to properties (removes optional).
    pub fn required(mut self) -> Self {
        self.modifier = PropertyModifier::Required;
        self
    }
}

impl FormatInto<TypeScript> for MappedType {
    fn format_into(self, tokens: &mut Tokens) {
        tokens.append("{");
        tokens.indent();
        tokens.push();

        if self.readonly {
            tokens.append("readonly");
            tokens.space();
        }

        tokens.append("[");
        tokens.append(self.key_param);
        tokens.space();
        tokens.append("in");
        tokens.space();
        tokens.append(self.constraint);
        tokens.append("]");

        match self.modifier {
            PropertyModifier::Optional => tokens.append("?"),
            PropertyModifier::Required => tokens.append("-?"),
            PropertyModifier::None => {}
        }

        tokens.append(":");
        tokens.space();
        tokens.append(self.value_type);
        tokens.append(";");

        tokens.unindent();
        tokens.push();
        tokens.append("}");
    }
}

/// Create a TypeScript mapped type.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// // { [P in keyof T]: T[P] }
/// let mapped = ts::mapped_type("P", ts::keyof(ts::type_ref("T")), ts::type_ref("T").indexed_by("P"));
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn mapped_type<K, C, V>(key_param: K, constraint: C, value_type: V) -> MappedType
where
    K: Into<ItemStr>,
    C: FormatInto<TypeScript>,
    V: FormatInto<TypeScript>,
{
    MappedType::new(key_param, constraint, value_type)
}

/// A TypeScript conditional type.
///
/// Conditional types select one of two possible types based on a condition.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// // type IsString<T> = T extends string ? true : false
/// let is_string = ts::type_alias("IsString",
///     ts::conditional_type(
///         ts::type_ref("T"),
///         ts::type_ref("string"),
///         ts::literal("true"),
///         ts::literal("false")
///     )
/// )
/// .with_generic_params(vec![ts::generic_param("T")]);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct ConditionalType {
    check_type: Tokens,
    extends_type: Tokens,
    true_type: Tokens,
    false_type: Tokens,
}

impl ConditionalType {
    /// Create a new conditional type.
    pub fn new<C, E, T, F>(check_type: C, extends_type: E, true_type: T, false_type: F) -> Self
    where
        C: FormatInto<TypeScript>,
        E: FormatInto<TypeScript>,
        T: FormatInto<TypeScript>,
        F: FormatInto<TypeScript>,
    {
        let mut check_tokens = Tokens::new();
        check_tokens.append(check_type);

        let mut extends_tokens = Tokens::new();
        extends_tokens.append(extends_type);

        let mut true_tokens = Tokens::new();
        true_tokens.append(true_type);

        let mut false_tokens = Tokens::new();
        false_tokens.append(false_type);

        Self {
            check_type: check_tokens,
            extends_type: extends_tokens,
            true_type: true_tokens,
            false_type: false_tokens,
        }
    }
}

impl FormatInto<TypeScript> for ConditionalType {
    fn format_into(self, tokens: &mut Tokens) {
        tokens.append(self.check_type);
        tokens.space();
        tokens.append("extends");
        tokens.space();
        tokens.append(self.extends_type);
        tokens.space();
        tokens.append("?");
        tokens.space();
        tokens.append(self.true_type);
        tokens.space();
        tokens.append(":");
        tokens.space();
        tokens.append(self.false_type);
    }
}

/// Create a TypeScript conditional type.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// // T extends string ? X : Y
/// let conditional = ts::conditional_type(
///     ts::type_ref("T"),
///     ts::type_ref("string"),
///     ts::type_ref("X"),
///     ts::type_ref("Y")
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn conditional_type<C, E, T, F>(
    check_type: C,
    extends_type: E,
    true_type: T,
    false_type: F,
) -> ConditionalType
where
    C: FormatInto<TypeScript>,
    E: FormatInto<TypeScript>,
    T: FormatInto<TypeScript>,
    F: FormatInto<TypeScript>,
{
    ConditionalType::new(check_type, extends_type, true_type, false_type)
}

/// A TypeScript arrow function type.
///
/// Arrow function types represent function types using arrow syntax.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// // (x: number, y: string) => boolean
/// let arrow = ts::arrow_function_type(
///     vec![
///         ts::param("x", ts::type_ref("number")),
///         ts::param("y", ts::type_ref("string")),
///     ],
///     ts::type_ref("boolean")
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct ArrowFunctionType {
    params: Vec<FunctionParam>,
    return_type: TypeRef,
}

impl ArrowFunctionType {
    /// Create a new arrow function type.
    pub fn new(params: Vec<FunctionParam>, return_type: TypeRef) -> Self {
        Self {
            params,
            return_type,
        }
    }
}

impl FormatInto<TypeScript> for ArrowFunctionType {
    fn format_into(self, tokens: &mut Tokens) {
        tokens.append("(");

        for (i, param) in self.params.into_iter().enumerate() {
            if i > 0 {
                tokens.append(",");
                tokens.space();
            }
            tokens.append(param);
        }

        tokens.append(")");
        tokens.space();
        tokens.append("=>");
        tokens.space();
        tokens.append(self.return_type);
    }
}

/// Create a TypeScript arrow function type.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// // (x: number) => string
/// let arrow = ts::arrow_function_type(
///     vec![ts::param("x", ts::type_ref("number"))],
///     ts::type_ref("string")
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn arrow_function_type(params: Vec<FunctionParam>, return_type: TypeRef) -> ArrowFunctionType {
    ArrowFunctionType::new(params, return_type)
}

/// A TypeScript typeof operator.
///
/// Represents the `typeof value` type operator.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let typeof_val = ts::typeof_operator("myValue");
/// # Ok::<_, genco::fmt::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct TypeofOperator {
    value: ItemStr,
}

impl TypeofOperator {
    /// Create a new typeof operator.
    pub fn new<V>(value: V) -> Self
    where
        V: Into<ItemStr>,
    {
        Self {
            value: value.into(),
        }
    }
}

impl FormatInto<TypeScript> for TypeofOperator {
    fn format_into(self, tokens: &mut Tokens) {
        tokens.append("typeof");
        tokens.space();
        tokens.append(self.value);
    }
}

/// Create a TypeScript typeof operator.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let typeof_val = ts::typeof_operator("myValue");
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn typeof_operator<V>(value: V) -> TypeofOperator
where
    V: Into<ItemStr>,
{
    TypeofOperator::new(value)
}

/// A TypeScript method signature for interfaces.
///
/// Method signatures describe methods in interfaces.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let method = ts::method_signature(
///     "getName",
///     vec![],
///     Some(ts::type_ref("string"))
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct MethodSignature {
    name: ItemStr,
    params: Vec<FunctionParam>,
    return_type: Option<TypeRef>,
}

impl MethodSignature {
    /// Create a new method signature.
    pub fn new<N>(name: N, params: Vec<FunctionParam>, return_type: Option<TypeRef>) -> Self
    where
        N: Into<ItemStr>,
    {
        Self {
            name: name.into(),
            params,
            return_type,
        }
    }
}

impl FormatInto<TypeScript> for MethodSignature {
    fn format_into(self, tokens: &mut Tokens) {
        tokens.append(self.name);
        tokens.append("(");

        for (i, param) in self.params.into_iter().enumerate() {
            if i > 0 {
                tokens.append(",");
                tokens.space();
            }
            tokens.append(param);
        }

        tokens.append(")");

        if let Some(return_type) = self.return_type {
            tokens.append(":");
            tokens.space();
            tokens.append(return_type);
        }

        tokens.append(";");
    }
}

/// Create a TypeScript method signature.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let method = ts::method_signature(
///     "getName",
///     vec![],
///     Some(ts::type_ref("string"))
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn method_signature<N>(
    name: N,
    params: Vec<FunctionParam>,
    return_type: Option<TypeRef>,
) -> MethodSignature
where
    N: Into<ItemStr>,
{
    MethodSignature::new(name, params, return_type)
}

/// A TypeScript template literal type.
///
/// Template literal types allow creating string literal types with interpolation.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// // `${string}-${number}`
/// let template = ts::template_literal_type(vec![
///     ts::type_ref("string"),
///     ts::type_ref("number"),
/// ]);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct TemplateLiteralType {
    parts: Vec<Tokens>,
}

impl TemplateLiteralType {
    /// Create a new template literal type.
    pub fn new<I, T>(parts: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: FormatInto<TypeScript>,
    {
        let parts = parts
            .into_iter()
            .map(|part| {
                let mut tokens = Tokens::new();
                tokens.append(part);
                tokens
            })
            .collect();
        Self { parts }
    }
}

impl FormatInto<TypeScript> for TemplateLiteralType {
    fn format_into(self, tokens: &mut Tokens) {
        tokens.append("`");

        for (i, part) in self.parts.into_iter().enumerate() {
            if i > 0 {
                tokens.append("-");
            }
            tokens.append("${");
            tokens.append(part);
            tokens.append("}");
        }

        tokens.append("`");
    }
}

/// Create a TypeScript template literal type.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// // `${string}-${number}`
/// let template = ts::template_literal_type(vec![
///     ts::type_ref("string"),
///     ts::type_ref("number"),
/// ]);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn template_literal_type<I, T>(parts: I) -> TemplateLiteralType
where
    I: IntoIterator<Item = T>,
    T: FormatInto<TypeScript>,
{
    TemplateLiteralType::new(parts)
}

/// A TypeScript type assertion.
///
/// Type assertions allow overriding the inferred type.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// // value as string
/// let assertion = ts::type_assertion("value", ts::type_ref("string"));
/// # Ok::<_, genco::fmt::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct TypeAssertion {
    value: ItemStr,
    type_ref: TypeRef,
}

impl TypeAssertion {
    /// Create a new type assertion.
    pub fn new<V>(value: V, type_ref: TypeRef) -> Self
    where
        V: Into<ItemStr>,
    {
        Self {
            value: value.into(),
            type_ref,
        }
    }
}

impl FormatInto<TypeScript> for TypeAssertion {
    fn format_into(self, tokens: &mut Tokens) {
        tokens.append(self.value);
        tokens.space();
        tokens.append("as");
        tokens.space();
        tokens.append(self.type_ref);
    }
}

/// Create a TypeScript type assertion using 'as' syntax.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// // value as string
/// let assertion = ts::type_assertion("value", ts::type_ref("string"));
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn type_assertion<V>(value: V, type_ref: TypeRef) -> TypeAssertion
where
    V: Into<ItemStr>,
{
    TypeAssertion::new(value, type_ref)
}

/// Intrinsic string manipulation type helper.
///
/// Creates TypeScript intrinsic string manipulation types.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let uppercase = ts::uppercase(ts::type_ref("T"));
/// let lowercase = ts::lowercase(ts::type_ref("T"));
/// let capitalize = ts::capitalize(ts::type_ref("T"));
/// let uncapitalize = ts::uncapitalize(ts::type_ref("T"));
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn uppercase(type_ref: TypeRef) -> TypeRef {
    use alloc::vec;
    TypeRef::new("Uppercase").with_generics(vec![type_ref])
}

/// Create Lowercase intrinsic type.
pub fn lowercase(type_ref: TypeRef) -> TypeRef {
    use alloc::vec;
    TypeRef::new("Lowercase").with_generics(vec![type_ref])
}

/// Create Capitalize intrinsic type.
pub fn capitalize(type_ref: TypeRef) -> TypeRef {
    use alloc::vec;
    TypeRef::new("Capitalize").with_generics(vec![type_ref])
}

/// Create Uncapitalize intrinsic type.
pub fn uncapitalize(type_ref: TypeRef) -> TypeRef {
    use alloc::vec;
    TypeRef::new("Uncapitalize").with_generics(vec![type_ref])
}

/// A TypeScript non-null assertion.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let assertion = ts::non_null_assertion("value");
/// # Ok::<_, genco::fmt::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct NonNullAssertion {
    value: ItemStr,
}

impl NonNullAssertion {
    /// Create a new non-null assertion.
    pub fn new<V>(value: V) -> Self
    where
        V: Into<ItemStr>,
    {
        Self {
            value: value.into(),
        }
    }
}

impl FormatInto<TypeScript> for NonNullAssertion {
    fn format_into(self, tokens: &mut Tokens) {
        tokens.append(self.value);
        tokens.append("!");
    }
}

/// Create a TypeScript non-null assertion.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let assertion = ts::non_null_assertion("value");
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn non_null_assertion<V>(value: V) -> NonNullAssertion
where
    V: Into<ItemStr>,
{
    NonNullAssertion::new(value)
}

/// A TypeScript type guard (type predicate).
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// // param is Type
/// let guard = ts::type_guard("value", ts::type_ref("string"));
/// # Ok::<_, genco::fmt::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct TypeGuard {
    param: ItemStr,
    type_ref: TypeRef,
}

impl TypeGuard {
    /// Create a new type guard.
    pub fn new<P>(param: P, type_ref: TypeRef) -> Self
    where
        P: Into<ItemStr>,
    {
        Self {
            param: param.into(),
            type_ref,
        }
    }
}

impl FormatInto<TypeScript> for TypeGuard {
    fn format_into(self, tokens: &mut Tokens) {
        tokens.append(self.param);
        tokens.space();
        tokens.append("is");
        tokens.space();
        tokens.append(self.type_ref);
    }
}

/// Create a TypeScript type guard (type predicate).
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// // param is Type
/// let guard = ts::type_guard("value", ts::type_ref("string"));
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn type_guard<P>(param: P, type_ref: TypeRef) -> TypeGuard
where
    P: Into<ItemStr>,
{
    TypeGuard::new(param, type_ref)
}

/// Built-in utility types helpers.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let partial = ts::partial(ts::type_ref("User"));
/// let required = ts::required(ts::type_ref("User"));
/// let readonly = ts::readonly_util(ts::type_ref("User"));
/// let pick = ts::pick(ts::type_ref("User"), ts::type_ref("'name' | 'email'"));
/// let omit = ts::omit(ts::type_ref("User"), ts::type_ref("'password'"));
/// # Ok::<_, genco::fmt::Error>(())
/// ```

/// Create Partial<T> utility type.
pub fn partial(type_ref: TypeRef) -> TypeRef {
    use alloc::vec;
    TypeRef::new("Partial").with_generics(vec![type_ref])
}

/// Create Required<T> utility type.
pub fn required(type_ref: TypeRef) -> TypeRef {
    use alloc::vec;
    TypeRef::new("Required").with_generics(vec![type_ref])
}

/// Create Readonly<T> utility type.
pub fn readonly_util(type_ref: TypeRef) -> TypeRef {
    use alloc::vec;
    TypeRef::new("Readonly").with_generics(vec![type_ref])
}

/// Create Pick<T, K> utility type.
pub fn pick(type_ref: TypeRef, keys: TypeRef) -> TypeRef {
    use alloc::vec;
    TypeRef::new("Pick").with_generics(vec![type_ref, keys])
}

/// Create Omit<T, K> utility type.
pub fn omit(type_ref: TypeRef, keys: TypeRef) -> TypeRef {
    use alloc::vec;
    TypeRef::new("Omit").with_generics(vec![type_ref, keys])
}

/// Create Record<K, T> utility type.
pub fn record(keys: TypeRef, value_type: TypeRef) -> TypeRef {
    use alloc::vec;
    TypeRef::new("Record").with_generics(vec![keys, value_type])
}

/// Create Exclude<T, U> utility type.
pub fn exclude(type_ref: TypeRef, excluded: TypeRef) -> TypeRef {
    use alloc::vec;
    TypeRef::new("Exclude").with_generics(vec![type_ref, excluded])
}

/// Create Extract<T, U> utility type.
pub fn extract(type_ref: TypeRef, union: TypeRef) -> TypeRef {
    use alloc::vec;
    TypeRef::new("Extract").with_generics(vec![type_ref, union])
}

/// Create NonNullable<T> utility type.
pub fn non_nullable(type_ref: TypeRef) -> TypeRef {
    use alloc::vec;
    TypeRef::new("NonNullable").with_generics(vec![type_ref])
}

/// Create ReturnType<T> utility type.
pub fn return_type(func_type: TypeRef) -> TypeRef {
    use alloc::vec;
    TypeRef::new("ReturnType").with_generics(vec![func_type])
}

/// A TypeScript enum.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let direction = ts::enum_type("Direction")
///     .with_variant("Up", None)
///     .with_variant("Down", None)
///     .with_variant("Left", None)
///     .with_variant("Right", None);
///
/// let toks: ts::Tokens = quote! {
///     $direction
/// };
/// # Ok::<_, genco::fmt::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct Enum {
    name: ItemStr,
    variants: Vec<(ItemStr, Option<Tokens>)>,
}

impl Enum {
    /// Create a new enum.
    pub fn new<N>(name: N) -> Self
    where
        N: Into<ItemStr>,
    {
        Self {
            name: name.into(),
            variants: Vec::new(),
        }
    }

    /// Add a variant to the enum.
    pub fn with_variant<N>(mut self, name: N, value: Option<Tokens>) -> Self
    where
        N: Into<ItemStr>,
    {
        self.variants.push((name.into(), value));
        self
    }
}

impl FormatInto<TypeScript> for Enum {
    fn format_into(self, tokens: &mut Tokens) {
        use crate as genco;
        use crate::quote_in;

        let name = self.name;
        let variant_count = self.variants.len();

        quote_in! { *tokens =>
            enum $name
        };

        tokens.space();
        tokens.append("{");
        tokens.indent();

        for (i, (variant_name, value)) in self.variants.into_iter().enumerate() {
            tokens.push();
            tokens.append(variant_name);

            if let Some(val) = value {
                tokens.space();
                tokens.append("=");
                tokens.space();
                tokens.append(val);
            }

            // Add comma for all but last variant
            if i < variant_count - 1 {
                tokens.append(",");
            }
        }

        tokens.unindent();
        tokens.push();
        tokens.append("}");
    }
}

/// Create a TypeScript enum.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let direction = ts::enum_type("Direction")
///     .with_variant("Up", None)
///     .with_variant("Down", None);
///
/// let color = ts::enum_type("Color")
///     .with_variant("Red", Some(quote!("#ff0000")))
///     .with_variant("Green", Some(quote!("#00ff00")));
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn enum_type<N>(name: N) -> Enum
where
    N: Into<ItemStr>,
{
    Enum::new(name)
}

/// A TypeScript union type (e.g., `string | number`).
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let status = ts::union_type(vec![
///     ts::literal("idle").into(),
///     ts::literal("loading").into(),
///     ts::literal("success").into(),
/// ]);
///
/// let toks: ts::Tokens = quote! {
///     type Status = $status;
/// };
/// # Ok::<_, genco::fmt::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct UnionType {
    types: Vec<TypeRef>,
}

impl UnionType {
    /// Create a new union type.
    pub fn new(types: Vec<TypeRef>) -> Self {
        Self { types }
    }
}

impl FormatInto<TypeScript> for UnionType {
    fn format_into(self, tokens: &mut Tokens) {
        for (i, type_ref) in self.types.into_iter().enumerate() {
            if i > 0 {
                tokens.space();
                tokens.append("|");
                tokens.space();
            }
            tokens.append(type_ref);
        }
    }
}

/// Create a TypeScript union type.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let string_or_number = ts::union_type(vec![
///     ts::type_ref("string"),
///     ts::type_ref("number"),
/// ]);
///
/// let toks: ts::Tokens = quote! {
///     let value: $string_or_number;
/// };
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn union_type(types: Vec<TypeRef>) -> UnionType {
    UnionType::new(types)
}

/// A TypeScript intersection type (e.g., `Type1 & Type2`).
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let combined = ts::intersection_type(vec![
///     ts::type_ref("Person"),
///     ts::type_ref("Worker"),
/// ]);
///
/// let toks: ts::Tokens = quote! {
///     type Employee = $combined;
/// };
/// # Ok::<_, genco::fmt::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct IntersectionType {
    types: Vec<TypeRef>,
}

impl IntersectionType {
    /// Create a new intersection type.
    pub fn new(types: Vec<TypeRef>) -> Self {
        Self { types }
    }
}

impl FormatInto<TypeScript> for IntersectionType {
    fn format_into(self, tokens: &mut Tokens) {
        for (i, type_ref) in self.types.into_iter().enumerate() {
            if i > 0 {
                tokens.space();
                tokens.append("&");
                tokens.space();
            }
            tokens.append(type_ref);
        }
    }
}

/// Create a TypeScript intersection type.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let employee = ts::intersection_type(vec![
///     ts::type_ref("Person"),
///     ts::type_ref("Worker"),
/// ]);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn intersection_type(types: Vec<TypeRef>) -> IntersectionType {
    IntersectionType::new(types)
}

/// A TypeScript literal type.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let idle = ts::literal("idle");
/// let count = ts::literal_number(42);
/// let pi = ts::literal_float(3.14);
/// let big = ts::literal_bigint(9007199254740991);
/// let flag = ts::literal_bool(true);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
#[derive(Debug, Clone)]
pub enum LiteralType {
    /// A string literal type.
    String(ItemStr),
    /// An integer literal type.
    Number(i64),
    /// A floating point literal type.
    Float(f64),
    /// A BigInt literal type (e.g., `42n`).
    BigInt(i64),
    /// A boolean literal type.
    Boolean(bool),
}

impl FormatInto<TypeScript> for LiteralType {
    fn format_into(self, tokens: &mut Tokens) {
        match self {
            LiteralType::String(s) => {
                tokens.append("\"");
                tokens.append(s);
                tokens.append("\"");
            }
            LiteralType::Number(n) => {
                tokens.append(n.to_string());
            }
            LiteralType::Float(f) => {
                tokens.append(f.to_string());
            }
            LiteralType::BigInt(n) => {
                tokens.append(n.to_string());
                tokens.append("n");
            }
            LiteralType::Boolean(b) => {
                tokens.append(if b { "true" } else { "false" });
            }
        }
    }
}

impl From<LiteralType> for TypeRef {
    fn from(lit: LiteralType) -> Self {
        // Create a wrapper that formats the literal inline
        match lit {
            LiteralType::String(s) => TypeRef::new(format!("\"{}\"", s)),
            LiteralType::Number(n) => TypeRef::new(n.to_string()),
            LiteralType::Float(f) => TypeRef::new(f.to_string()),
            LiteralType::BigInt(n) => TypeRef::new(format!("{}n", n)),
            LiteralType::Boolean(b) => TypeRef::new(if b { "true" } else { "false" }),
        }
    }
}

/// Create a TypeScript string literal type.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let status = ts::union_type(vec![
///     ts::literal("idle").into(),
///     ts::literal("loading").into(),
///     ts::literal("success").into(),
/// ]);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn literal<S>(value: S) -> LiteralType
where
    S: Into<ItemStr>,
{
    LiteralType::String(value.into())
}

/// Create a TypeScript integer literal type.
pub fn literal_number(value: i64) -> LiteralType {
    LiteralType::Number(value)
}

/// Create a TypeScript floating point literal type.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let pi = ts::literal_float(3.14159);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn literal_float(value: f64) -> LiteralType {
    LiteralType::Float(value)
}

/// Create a TypeScript BigInt literal type.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let big_number = ts::literal_bigint(9007199254740992);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn literal_bigint(value: i64) -> LiteralType {
    LiteralType::BigInt(value)
}

/// Create a TypeScript boolean literal type.
pub fn literal_bool(value: bool) -> LiteralType {
    LiteralType::Boolean(value)
}

/// A TypeScript tuple type (e.g., `[string, number]`).
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let pair = ts::tuple_type(vec![
///     ts::type_ref("string"),
///     ts::type_ref("number"),
/// ]);
///
/// let toks: ts::Tokens = quote! {
///     type Pair = $pair;
/// };
/// # Ok::<_, genco::fmt::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct TupleType {
    elements: Vec<TypeRef>,
}

impl TupleType {
    /// Create a new tuple type.
    pub fn new(elements: Vec<TypeRef>) -> Self {
        Self { elements }
    }
}

impl FormatInto<TypeScript> for TupleType {
    fn format_into(self, tokens: &mut Tokens) {
        tokens.append("[");
        for (i, element) in self.elements.into_iter().enumerate() {
            if i > 0 {
                tokens.append(",");
                tokens.space();
            }
            tokens.append(element);
        }
        tokens.append("]");
    }
}

/// Create a TypeScript tuple type.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let coordinates = ts::tuple_type(vec![
///     ts::type_ref("number"),
///     ts::type_ref("number"),
/// ]);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn tuple_type(elements: Vec<TypeRef>) -> TupleType {
    TupleType::new(elements)
}

/// A TypeScript function parameter.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let name_param = ts::param("name", ts::type_ref("string"));
/// let age_param = ts::param("age", ts::type_ref("number")).optional();
/// # Ok::<_, genco::fmt::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct FunctionParam {
    name: ItemStr,
    type_ref: TypeRef,
    optional: bool,
}

impl FunctionParam {
    /// Create a new function parameter.
    pub fn new<N>(name: N, type_ref: TypeRef) -> Self
    where
        N: Into<ItemStr>,
    {
        Self {
            name: name.into(),
            type_ref,
            optional: false,
        }
    }

    /// Mark this parameter as optional.
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
}

impl FormatInto<TypeScript> for FunctionParam {
    fn format_into(self, tokens: &mut Tokens) {
        tokens.append(self.name);
        if self.optional {
            tokens.append("?");
        }
        tokens.append(":");
        tokens.space();
        tokens.append(self.type_ref);
    }
}

/// Create a TypeScript function parameter.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let param = ts::param("name", ts::type_ref("string"));
/// let optional_param = ts::param("age", ts::type_ref("number")).optional();
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn param<N>(name: N, type_ref: TypeRef) -> FunctionParam
where
    N: Into<ItemStr>,
{
    FunctionParam::new(name, type_ref)
}

/// A TypeScript function signature.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let greet = ts::function_signature(
///     "greet",
///     vec![ts::param("name", ts::type_ref("string"))],
///     Some(ts::type_ref("string")),
/// );
///
/// let toks: ts::Tokens = quote! {
///     $greet {
///         return "Hello, " + name;
///     }
/// };
/// # Ok::<_, genco::fmt::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    name: ItemStr,
    params: Vec<FunctionParam>,
    return_type: Option<TypeRef>,
}

impl FunctionSignature {
    /// Create a new function signature.
    pub fn new<N>(name: N, params: Vec<FunctionParam>, return_type: Option<TypeRef>) -> Self
    where
        N: Into<ItemStr>,
    {
        Self {
            name: name.into(),
            params,
            return_type,
        }
    }
}

impl FormatInto<TypeScript> for FunctionSignature {
    fn format_into(self, tokens: &mut Tokens) {
        tokens.append("function");
        tokens.space();
        tokens.append(self.name);
        tokens.append("(");

        for (i, param) in self.params.into_iter().enumerate() {
            if i > 0 {
                tokens.append(",");
                tokens.space();
            }
            tokens.append(param);
        }

        tokens.append(")");

        if let Some(return_type) = self.return_type {
            tokens.append(":");
            tokens.space();
            tokens.append(return_type);
        }
    }
}

/// Create a TypeScript function signature.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let add = ts::function_signature(
///     "add",
///     vec![
///         ts::param("a", ts::type_ref("number")),
///         ts::param("b", ts::type_ref("number")),
///     ],
///     Some(ts::type_ref("number")),
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn function_signature<N>(
    name: N,
    params: Vec<FunctionParam>,
    return_type: Option<TypeRef>,
) -> FunctionSignature
where
    N: Into<ItemStr>,
{
    FunctionSignature::new(name, params, return_type)
}

/// A TypeScript call signature for interfaces.
///
/// Call signatures allow an interface to describe a callable type.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let callable = ts::interface("MyFunction")
///     .with_call_signature(ts::call_signature(
///         vec![ts::param("x", ts::type_ref("number"))],
///         Some(ts::type_ref("string"))
///     ));
/// # Ok::<_, genco::fmt::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct CallSignature {
    params: Vec<FunctionParam>,
    return_type: Option<TypeRef>,
}

impl CallSignature {
    /// Create a new call signature.
    pub fn new(params: Vec<FunctionParam>, return_type: Option<TypeRef>) -> Self {
        Self {
            params,
            return_type,
        }
    }
}

impl FormatInto<TypeScript> for CallSignature {
    fn format_into(self, tokens: &mut Tokens) {
        tokens.append("(");

        for (i, param) in self.params.into_iter().enumerate() {
            if i > 0 {
                tokens.append(",");
                tokens.space();
            }
            tokens.append(param);
        }

        tokens.append(")");

        if let Some(return_type) = self.return_type {
            tokens.append(":");
            tokens.space();
            tokens.append(return_type);
        }

        tokens.append(";");
    }
}

/// Create a TypeScript call signature.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let sig = ts::call_signature(
///     vec![ts::param("x", ts::type_ref("number"))],
///     Some(ts::type_ref("string"))
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn call_signature(
    params: Vec<FunctionParam>,
    return_type: Option<TypeRef>,
) -> CallSignature {
    CallSignature::new(params, return_type)
}

/// A TypeScript construct signature for interfaces.
///
/// Construct signatures allow an interface to describe a constructable type.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let constructable = ts::interface("MyConstructor")
///     .with_construct_signature(ts::construct_signature(
///         vec![ts::param("x", ts::type_ref("number"))],
///         ts::type_ref("MyClass")
///     ));
/// # Ok::<_, genco::fmt::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct ConstructSignature {
    params: Vec<FunctionParam>,
    return_type: TypeRef,
}

impl ConstructSignature {
    /// Create a new construct signature.
    pub fn new(params: Vec<FunctionParam>, return_type: TypeRef) -> Self {
        Self {
            params,
            return_type,
        }
    }
}

impl FormatInto<TypeScript> for ConstructSignature {
    fn format_into(self, tokens: &mut Tokens) {
        tokens.append("new");
        tokens.space();
        tokens.append("(");

        for (i, param) in self.params.into_iter().enumerate() {
            if i > 0 {
                tokens.append(",");
                tokens.space();
            }
            tokens.append(param);
        }

        tokens.append(")");
        tokens.append(":");
        tokens.space();
        tokens.append(self.return_type);
        tokens.append(";");
    }
}

/// Create a TypeScript construct signature.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let sig = ts::construct_signature(
///     vec![ts::param("x", ts::type_ref("number"))],
///     ts::type_ref("MyClass")
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn construct_signature(
    params: Vec<FunctionParam>,
    return_type: TypeRef,
) -> ConstructSignature {
    ConstructSignature::new(params, return_type)
}
