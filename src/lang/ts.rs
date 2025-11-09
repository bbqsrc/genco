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
use alloc::string::String;
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
        }
    }

    /// Add generic type parameters.
    pub fn with_generics(self, generics: Vec<TypeRef>) -> Self {
        Self { generics, ..self }
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
/// let toks: ts::Tokens = quote! {
///     $user_interface
/// };
/// # Ok::<_, genco::fmt::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct Interface {
    name: ItemStr,
    properties: Vec<Property>,
}

impl Interface {
    /// Create a new interface.
    pub fn new<N>(name: N) -> Self
    where
        N: Into<ItemStr>,
    {
        Self {
            name: name.into(),
            properties: Vec::new(),
        }
    }

    /// Add a property to the interface.
    pub fn with_property(mut self, property: Property) -> Self {
        self.properties.push(property);
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

        tokens.space();
        tokens.append("{");
        tokens.indent();

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
/// let toks: ts::Tokens = quote! {
///     $user_id
/// };
/// # Ok::<_, genco::fmt::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct TypeAlias {
    name: ItemStr,
    type_ref: TypeRef,
}

impl TypeAlias {
    /// Create a new type alias.
    pub fn new<N>(name: N, type_ref: TypeRef) -> Self
    where
        N: Into<ItemStr>,
    {
        Self {
            name: name.into(),
            type_ref,
        }
    }
}

impl FormatInto<TypeScript> for TypeAlias {
    fn format_into(self, tokens: &mut Tokens) {
        tokens.append("type");
        tokens.space();
        tokens.append(self.name);
        tokens.space();
        tokens.append("=");
        tokens.space();
        tokens.append(self.type_ref);
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
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn type_alias<N>(name: N, type_ref: TypeRef) -> TypeAlias
where
    N: Into<ItemStr>,
{
    TypeAlias::new(name, type_ref)
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
