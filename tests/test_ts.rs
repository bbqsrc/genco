use genco::prelude::*;

#[test]
fn test_basic_quote() -> genco::fmt::Result {
    let toks: ts::Tokens = quote! {
        function greet(name: string): string {
            return "Hello, " + name;
        }
    };

    assert_eq!(
        vec![
            "function greet(name: string): string {",
            "    return \"Hello, \" + name;",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_string_quoting() -> genco::fmt::Result {
    let toks: ts::Tokens = quote!("start π 😊 \n \x7f ÿ $ \\ end");
    assert_eq!("\"start π 😊 \\n \\x7f ÿ $ \\\\ end\"", toks.to_string()?);

    let toks: ts::Tokens = quote!($(quoted("start π 😊 \n \x7f ÿ $ \\ end")));
    assert_eq!(
        "\"start π 😊 \\n \\x7f ÿ $ \\\\ end\"",
        toks.to_string()?
    );
    Ok(())
}

#[test]
fn test_template_literals() -> genco::fmt::Result {
    let toks: ts::Tokens = quote!($[str](Hello $(World)));
    assert_eq!("`Hello ${World}`", toks.to_string()?);
    Ok(())
}

#[test]
fn test_named_imports() -> genco::fmt::Result {
    let foo = ts::import("module", "Foo");
    let bar = ts::import("module", "Bar");

    let toks: ts::Tokens = quote! {
        $foo
        $bar
    };

    assert_eq!(
        vec![
            "import {Bar, Foo} from \"module\";",
            "",
            "Foo",
            "Bar",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_default_import() -> genco::fmt::Result {
    let react = ts::import("react", "React").into_default();

    let toks: ts::Tokens = quote! {
        $react
    };

    assert_eq!(
        vec!["import React from \"react\";", "", "React"],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_wildcard_import() -> genco::fmt::Result {
    let utils = ts::import("utils", "Utils").into_wildcard();

    let toks: ts::Tokens = quote! {
        $utils
    };

    assert_eq!(
        vec!["import * as Utils from \"utils\";", "", "Utils"],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_import_with_alias() -> genco::fmt::Result {
    let foo = ts::import("module", "Foo");
    let bar = ts::import("module", "Foo").with_alias("Bar");

    let toks: ts::Tokens = quote! {
        $foo
        $bar
    };

    assert_eq!(
        vec![
            "import {Foo, Foo as Bar} from \"module\";",
            "",
            "Foo",
            "Bar",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_type_only_import() -> genco::fmt::Result {
    let user = ts::import("./types", "User").into_type_only();

    let toks: ts::Tokens = quote! {
        $user
    };

    assert_eq!(
        vec!["import type {User} from \"./types\";", "", "User"],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_mixed_imports() -> genco::fmt::Result {
    let user_type = ts::import("./types", "User").into_type_only();
    let api = ts::import("./api", "API").into_default();
    let utils = ts::import("./utils", "formatDate");

    let toks: ts::Tokens = quote! {
        $user_type
        $api
        $utils
    };

    assert_eq!(
        vec![
            "import API from \"./api\";",
            "import type {User} from \"./types\";",
            "import {formatDate} from \"./utils\";",
            "",
            "User",
            "API",
            "formatDate",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_type_ref() -> genco::fmt::Result {
    let string_type = ts::type_ref("string");
    let number_type = ts::type_ref("number");

    let toks: ts::Tokens = quote! {
        let name: $string_type;
        let age: $number_type;
    };

    assert_eq!(
        vec!["let name: string;", "let age: number;"],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_generic_type_ref() -> genco::fmt::Result {
    let promise_user = ts::type_ref("Promise").with_generics(vec![ts::type_ref("User")]);
    let array_string = ts::type_ref("Array").with_generics(vec![ts::type_ref("string")]);

    let toks: ts::Tokens = quote! {
        let user: $promise_user;
        let names: $array_string;
    };

    assert_eq!(
        vec!["let user: Promise<User>;", "let names: Array<string>;"],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_interface() -> genco::fmt::Result {
    let user_interface = ts::interface("User")
        .with_property(ts::property("id", ts::type_ref("number")))
        .with_property(ts::property("name", ts::type_ref("string")))
        .with_property(ts::optional_property("email", ts::type_ref("string")));

    let toks: ts::Tokens = quote! {
        $user_interface
    };

    assert_eq!(
        vec![
            "interface User {",
            "    id: number;",
            "    name: string;",
            "    email?: string;",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_interface_with_readonly() -> genco::fmt::Result {
    let config_interface = ts::interface("Config")
        .with_property(ts::property("apiUrl", ts::type_ref("string")).readonly())
        .with_property(ts::property("timeout", ts::type_ref("number")));

    let toks: ts::Tokens = quote! {
        $config_interface
    };

    assert_eq!(
        vec![
            "interface Config {",
            "    readonly apiUrl: string;",
            "    timeout: number;",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_type_alias() -> genco::fmt::Result {
    let user_id = ts::type_alias("UserID", ts::type_ref("string"));

    let toks: ts::Tokens = quote! {
        $user_id
    };

    assert_eq!(vec!["type UserID = string;"], toks.to_file_vec()?);
    Ok(())
}

#[test]
fn test_enum() -> genco::fmt::Result {
    let direction = ts::enum_type("Direction")
        .with_variant("Up", None)
        .with_variant("Down", None)
        .with_variant("Left", None)
        .with_variant("Right", None);

    let toks: ts::Tokens = quote! {
        $direction
    };

    assert_eq!(
        vec![
            "enum Direction {",
            "    Up,",
            "    Down,",
            "    Left,",
            "    Right",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_enum_with_values() -> genco::fmt::Result {
    let color = ts::enum_type("Color")
        .with_variant("Red", Some(quote!("#ff0000")))
        .with_variant("Green", Some(quote!("#00ff00")))
        .with_variant("Blue", Some(quote!("#0000ff")));

    let toks: ts::Tokens = quote! {
        $color
    };

    assert_eq!(
        vec![
            "enum Color {",
            "    Red = \"#ff0000\",",
            "    Green = \"#00ff00\",",
            "    Blue = \"#0000ff\"",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_complex_example() -> genco::fmt::Result {
    let user_type = ts::import("./types", "User").into_type_only();
    let api = ts::import("./api", "API");

    let user_interface = ts::interface("User")
        .with_property(ts::property("id", ts::type_ref("number")))
        .with_property(ts::property("name", ts::type_ref("string")))
        .with_property(ts::optional_property("email", ts::type_ref("string")));

    let user_id_type = ts::type_alias("UserID", ts::type_ref("number"));

    let toks: ts::Tokens = quote! {
        $user_interface

        $user_id_type

        function getUser(id: UserID): Promise<$user_type> {
            return $api.fetchUser(id);
        }
    };

    assert_eq!(
        vec![
            "import {API} from \"./api\";",
            "import type {User} from \"./types\";",
            "",
            "interface User {",
            "    id: number;",
            "    name: string;",
            "    email?: string;",
            "}",
            "",
            "type UserID = number;",
            "",
            "function getUser(id: UserID): Promise<User> {",
            "    return API.fetchUser(id);",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_module_path_resolution() -> genco::fmt::Result {
    use genco::fmt;

    let foo1 = ts::import(ts::Module::Path("foo/bar.ts".into()), "Foo1");
    let foo2 = ts::import(ts::Module::Path("foo/bar.ts".into()), "Foo2");
    let react = ts::import("react", "React").into_default();

    let toks: ts::Tokens = quote! {
        $foo1
        $foo2
        $react
    };

    let mut w = fmt::VecWriter::new();

    let config = ts::Config::default().with_module_path("foo/baz.ts");
    let fmt = fmt::Config::from_lang::<TypeScript>();

    toks.format_file(&mut w.as_formatter(&fmt), &config)?;

    assert_eq!(
        vec![
            "import {Foo1, Foo2} from \"../bar.ts\";",
            "import React from \"react\";",
            "",
            "Foo1",
            "Foo2",
            "React"
        ],
        w.into_vec()
    );
    Ok(())
}
