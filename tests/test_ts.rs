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
fn test_union_type() -> genco::fmt::Result {
    let string_or_number = ts::union_type(vec![
        ts::type_ref("string"),
        ts::type_ref("number"),
    ]);

    let toks: ts::Tokens = quote! {
        type StringOrNumber = $string_or_number;
    };

    assert_eq!(
        vec!["type StringOrNumber = string | number;"],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_union_type_with_literals() -> genco::fmt::Result {
    let status = ts::union_type(vec![
        ts::literal("idle").into(),
        ts::literal("loading").into(),
        ts::literal("success").into(),
        ts::literal("error").into(),
    ]);

    let toks: ts::Tokens = quote! {
        type Status = $status;
    };

    assert_eq!(
        vec!["type Status = \"idle\" | \"loading\" | \"success\" | \"error\";"],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_intersection_type() -> genco::fmt::Result {
    let employee = ts::intersection_type(vec![
        ts::type_ref("Person"),
        ts::type_ref("Worker"),
    ]);

    let toks: ts::Tokens = quote! {
        type Employee = $employee;
    };

    assert_eq!(
        vec!["type Employee = Person & Worker;"],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_literal_types() -> genco::fmt::Result {
    let string_lit = ts::literal("success");
    let number_lit = ts::literal_number(42);
    let float_lit = ts::literal_float(3.14);
    let bigint_lit = ts::literal_bigint(9007199254740992);
    let bool_lit = ts::literal_bool(true);

    let toks: ts::Tokens = quote! {
        type A = $string_lit;
        type B = $number_lit;
        type C = $float_lit;
        type D = $bigint_lit;
        type E = $bool_lit;
    };

    assert_eq!(
        vec![
            "type A = \"success\";",
            "type B = 42;",
            "type C = 3.14;",
            "type D = 9007199254740992n;",
            "type E = true;",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_tuple_type() -> genco::fmt::Result {
    let pair = ts::tuple_type(vec![
        ts::type_ref("string"),
        ts::type_ref("number"),
    ]);

    let toks: ts::Tokens = quote! {
        type Pair = $pair;
    };

    assert_eq!(
        vec!["type Pair = [string, number];"],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_function_signature() -> genco::fmt::Result {
    let greet = ts::function_signature(
        "greet",
        vec![ts::param("name", ts::type_ref("string"))],
        Some(ts::type_ref("string")),
    );

    let toks: ts::Tokens = quote! {
        $greet {
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
fn test_function_signature_multiple_params() -> genco::fmt::Result {
    let add = ts::function_signature(
        "add",
        vec![
            ts::param("a", ts::type_ref("number")),
            ts::param("b", ts::type_ref("number")),
        ],
        Some(ts::type_ref("number")),
    );

    let toks: ts::Tokens = quote! {
        $add {
            return a + b;
        }
    };

    assert_eq!(
        vec![
            "function add(a: number, b: number): number {",
            "    return a + b;",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_function_signature_optional_params() -> genco::fmt::Result {
    let greet = ts::function_signature(
        "greet",
        vec![
            ts::param("name", ts::type_ref("string")),
            ts::param("age", ts::type_ref("number")).optional(),
        ],
        Some(ts::type_ref("string")),
    );

    let toks: ts::Tokens = quote! {
        $greet {
            return name;
        }
    };

    assert_eq!(
        vec![
            "function greet(name: string, age?: number): string {",
            "    return name;",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_function_signature_no_return() -> genco::fmt::Result {
    let log = ts::function_signature(
        "log",
        vec![ts::param("message", ts::type_ref("string"))],
        None,
    );

    let toks: ts::Tokens = quote! {
        $log {
            console.log(message);
        }
    };

    assert_eq!(
        vec![
            "function log(message: string) {",
            "    console.log(message);",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_complex_types_combination() -> genco::fmt::Result {
    let result_type = ts::union_type(vec![
        ts::type_ref("Success").with_generics(vec![ts::type_ref("T")]),
        ts::type_ref("Error"),
    ]);

    let handler_params = ts::tuple_type(vec![
        ts::type_ref("string"),
        ts::type_ref("number"),
    ]);

    let toks: ts::Tokens = quote! {
        type Result<T> = $result_type;
        type Handler = $handler_params;
    };

    assert_eq!(
        vec![
            "type Result<T> = Success<T> | Error;",
            "type Handler = [string, number];",
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

#[test]
fn test_generic_interface_simple() -> genco::fmt::Result {
    let container = ts::interface("Container")
        .with_generic_params(vec![ts::generic_param("T")])
        .with_property(ts::property("value", ts::type_ref("T")));

    let toks: ts::Tokens = quote! {
        $container
    };

    assert_eq!(
        vec![
            "interface Container<T> {",
            "    value: T;",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_generic_interface_multiple_params() -> genco::fmt::Result {
    let map = ts::interface("Map")
        .with_generic_params(vec![ts::generic_param("K"), ts::generic_param("V")])
        .with_property(ts::property("key", ts::type_ref("K")))
        .with_property(ts::property("value", ts::type_ref("V")));

    let toks: ts::Tokens = quote! {
        $map
    };

    assert_eq!(
        vec![
            "interface Map<K, V> {",
            "    key: K;",
            "    value: V;",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_generic_interface_with_constraint() -> genco::fmt::Result {
    let constrained = ts::interface("Constrained")
        .with_generic_params(vec![
            ts::generic_param("T").with_constraint(ts::type_ref("Base"))
        ])
        .with_property(ts::property("item", ts::type_ref("T")));

    let toks: ts::Tokens = quote! {
        $constrained
    };

    assert_eq!(
        vec![
            "interface Constrained<T extends Base> {",
            "    item: T;",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_generic_interface_with_default() -> genco::fmt::Result {
    let with_default = ts::interface("WithDefault")
        .with_generic_params(vec![
            ts::generic_param("T").with_default(ts::type_ref("string"))
        ])
        .with_property(ts::property("value", ts::type_ref("T")));

    let toks: ts::Tokens = quote! {
        $with_default
    };

    assert_eq!(
        vec![
            "interface WithDefault<T = string> {",
            "    value: T;",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_generic_interface_constraint_and_default() -> genco::fmt::Result {
    let full = ts::interface("Full")
        .with_generic_params(vec![
            ts::generic_param("T")
                .with_constraint(ts::type_ref("Base"))
                .with_default(ts::type_ref("DefaultImpl"))
        ])
        .with_property(ts::property("item", ts::type_ref("T")));

    let toks: ts::Tokens = quote! {
        $full
    };

    assert_eq!(
        vec![
            "interface Full<T extends Base = DefaultImpl> {",
            "    item: T;",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_generic_type_alias() -> genco::fmt::Result {
    let nullable = ts::type_alias(
        "Nullable",
        ts::union_type(vec![ts::type_ref("T"), ts::type_ref("null")]),
    )
    .with_generic_params(vec![ts::generic_param("T")]);

    let toks: ts::Tokens = quote! {
        $nullable
    };

    assert_eq!(
        vec!["type Nullable<T> = T | null;"],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_generic_type_alias_with_constraint() -> genco::fmt::Result {
    let constrained_result = ts::type_alias(
        "Result",
        ts::union_type(vec![
            ts::type_ref("Success").with_generics(vec![ts::type_ref("T")]),
            ts::type_ref("Error"),
        ]),
    )
    .with_generic_params(vec![
        ts::generic_param("T").with_constraint(ts::type_ref("Serializable"))
    ]);

    let toks: ts::Tokens = quote! {
        $constrained_result
    };

    assert_eq!(
        vec!["type Result<T extends Serializable> = Success<T> | Error;"],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_generic_type_alias_with_default() -> genco::fmt::Result {
    let optional = ts::type_alias(
        "Optional",
        ts::union_type(vec![ts::type_ref("T"), ts::type_ref("undefined")]),
    )
    .with_generic_params(vec![
        ts::generic_param("T").with_default(ts::type_ref("any"))
    ]);

    let toks: ts::Tokens = quote! {
        $optional
    };

    assert_eq!(
        vec!["type Optional<T = any> = T | undefined;"],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_generic_type_alias_multiple_params() -> genco::fmt::Result {
    let key_value = ts::type_alias(
        "KeyValue",
        ts::tuple_type(vec![ts::type_ref("K"), ts::type_ref("V")]),
    )
    .with_generic_params(vec![ts::generic_param("K"), ts::generic_param("V")]);

    let toks: ts::Tokens = quote! {
        $key_value
    };

    assert_eq!(
        vec!["type KeyValue<K, V> = [K, V];"],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_complex_generics() -> genco::fmt::Result {
    // Interface with multiple generic params
    let response = ts::interface("Response")
        .with_generic_params(vec![
            ts::generic_param("T"),
            ts::generic_param("E").with_default(ts::type_ref("Error")),
        ])
        .with_property(ts::optional_property("data", ts::type_ref("T")))
        .with_property(ts::optional_property("error", ts::type_ref("E")));

    // Type alias using the generic interface
    let api_response = ts::type_alias(
        "ApiResponse",
        ts::type_ref("Response").with_generics(vec![ts::type_ref("T")]),
    )
    .with_generic_params(vec![ts::generic_param("T")]);

    let toks: ts::Tokens = quote! {
        $response

        $api_response
    };

    assert_eq!(
        vec![
            "interface Response<T, E = Error> {",
            "    data?: T;",
            "    error?: E;",
            "}",
            "",
            "type ApiResponse<T> = Response<T>;",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_index_signature_string() -> genco::fmt::Result {
    let map = ts::interface("StringMap")
        .with_index_signature(ts::index_signature_string("key", ts::type_ref("any")));

    let toks: ts::Tokens = quote! {
        $map
    };

    assert_eq!(
        vec![
            "interface StringMap {",
            "    [key: string]: any;",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_index_signature_number() -> genco::fmt::Result {
    let array = ts::interface("NumberArray")
        .with_index_signature(ts::index_signature_number("index", ts::type_ref("string")));

    let toks: ts::Tokens = quote! {
        $array
    };

    assert_eq!(
        vec![
            "interface NumberArray {",
            "    [index: number]: string;",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_index_signature_symbol() -> genco::fmt::Result {
    let symbol_map = ts::interface("SymbolMap")
        .with_index_signature(ts::index_signature_symbol("sym", ts::type_ref("number")));

    let toks: ts::Tokens = quote! {
        $symbol_map
    };

    assert_eq!(
        vec![
            "interface SymbolMap {",
            "    [sym: symbol]: number;",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_index_signature_with_properties() -> genco::fmt::Result {
    let map = ts::interface("StringMapWithLength")
        .with_index_signature(ts::index_signature_string("key", ts::type_ref("any")))
        .with_property(ts::property("length", ts::type_ref("number")))
        .with_property(ts::optional_property("name", ts::type_ref("string")));

    let toks: ts::Tokens = quote! {
        $map
    };

    assert_eq!(
        vec![
            "interface StringMapWithLength {",
            "    [key: string]: any;",
            "    length: number;",
            "    name?: string;",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_index_signature_with_generics() -> genco::fmt::Result {
    let dict = ts::interface("Dictionary")
        .with_generic_params(vec![ts::generic_param("T")])
        .with_index_signature(ts::index_signature_string("key", ts::type_ref("T")))
        .with_property(ts::property("count", ts::type_ref("number")));

    let toks: ts::Tokens = quote! {
        $dict
    };

    assert_eq!(
        vec![
            "interface Dictionary<T> {",
            "    [key: string]: T;",
            "    count: number;",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_interface_extends_single() -> genco::fmt::Result {
    let user = ts::interface("User")
        .with_extends(vec![ts::type_ref("Base")])
        .with_property(ts::property("name", ts::type_ref("string")))
        .with_property(ts::property("email", ts::type_ref("string")));

    let toks: ts::Tokens = quote! {
        $user
    };

    assert_eq!(
        vec![
            "interface User extends Base {",
            "    name: string;",
            "    email: string;",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_interface_extends_multiple() -> genco::fmt::Result {
    let admin = ts::interface("Admin")
        .with_extends(vec![ts::type_ref("User"), ts::type_ref("Permissions")])
        .with_property(ts::property("role", ts::type_ref("string")));

    let toks: ts::Tokens = quote! {
        $admin
    };

    assert_eq!(
        vec![
            "interface Admin extends User, Permissions {",
            "    role: string;",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_interface_extends_with_generics() -> genco::fmt::Result {
    let container = ts::interface("Container")
        .with_generic_params(vec![ts::generic_param("T")])
        .with_extends(vec![ts::type_ref("Base").with_generics(vec![ts::type_ref("T")])])
        .with_property(ts::property("value", ts::type_ref("T")));

    let toks: ts::Tokens = quote! {
        $container
    };

    assert_eq!(
        vec![
            "interface Container<T> extends Base<T> {",
            "    value: T;",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_interface_extends_generic_constraint() -> genco::fmt::Result {
    let repository = ts::interface("Repository")
        .with_generic_params(vec![
            ts::generic_param("T").with_constraint(ts::type_ref("Entity"))
        ])
        .with_extends(vec![ts::type_ref("BaseRepository").with_generics(vec![ts::type_ref("T")])])
        .with_property(ts::property("items", ts::type_ref("T").with_generics(vec![])));

    let toks: ts::Tokens = quote! {
        $repository
    };

    assert_eq!(
        vec![
            "interface Repository<T extends Entity> extends BaseRepository<T> {",
            "    items: T;",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_interface_extends_with_index_signature() -> genco::fmt::Result {
    let extended_map = ts::interface("ExtendedMap")
        .with_extends(vec![ts::type_ref("BaseMap")])
        .with_index_signature(ts::index_signature_string("key", ts::type_ref("any")))
        .with_property(ts::property("size", ts::type_ref("number")));

    let toks: ts::Tokens = quote! {
        $extended_map
    };

    assert_eq!(
        vec![
            "interface ExtendedMap extends BaseMap {",
            "    [key: string]: any;",
            "    size: number;",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_call_signature_simple() -> genco::fmt::Result {
    let callable = ts::interface("MyFunction")
        .with_call_signature(ts::call_signature(
            vec![ts::param("x", ts::type_ref("number"))],
            Some(ts::type_ref("string")),
        ));

    let toks: ts::Tokens = quote! {
        $callable
    };

    assert_eq!(
        vec![
            "interface MyFunction {",
            "    (x: number): string;",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_call_signature_multiple_params() -> genco::fmt::Result {
    let callable = ts::interface("Comparator")
        .with_call_signature(ts::call_signature(
            vec![
                ts::param("a", ts::type_ref("T")),
                ts::param("b", ts::type_ref("T")),
            ],
            Some(ts::type_ref("number")),
        ))
        .with_generic_params(vec![ts::generic_param("T")]);

    let toks: ts::Tokens = quote! {
        $callable
    };

    assert_eq!(
        vec![
            "interface Comparator<T> {",
            "    (a: T, b: T): number;",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_call_signature_no_return() -> genco::fmt::Result {
    let callable = ts::interface("VoidFunction")
        .with_call_signature(ts::call_signature(
            vec![ts::param("msg", ts::type_ref("string"))],
            None,
        ));

    let toks: ts::Tokens = quote! {
        $callable
    };

    assert_eq!(
        vec![
            "interface VoidFunction {",
            "    (msg: string);",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_construct_signature_simple() -> genco::fmt::Result {
    let constructable = ts::interface("MyConstructor")
        .with_construct_signature(ts::construct_signature(
            vec![ts::param("x", ts::type_ref("number"))],
            ts::type_ref("MyClass"),
        ));

    let toks: ts::Tokens = quote! {
        $constructable
    };

    assert_eq!(
        vec![
            "interface MyConstructor {",
            "    new (x: number): MyClass;",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_construct_signature_multiple_params() -> genco::fmt::Result {
    let constructable = ts::interface("PersonConstructor")
        .with_construct_signature(ts::construct_signature(
            vec![
                ts::param("name", ts::type_ref("string")),
                ts::param("age", ts::type_ref("number")),
            ],
            ts::type_ref("Person"),
        ));

    let toks: ts::Tokens = quote! {
        $constructable
    };

    assert_eq!(
        vec![
            "interface PersonConstructor {",
            "    new (name: string, age: number): Person;",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_call_and_construct_signatures_combined() -> genco::fmt::Result {
    let combined = ts::interface("Dual")
        .with_call_signature(ts::call_signature(
            vec![ts::param("x", ts::type_ref("string"))],
            Some(ts::type_ref("number")),
        ))
        .with_construct_signature(ts::construct_signature(
            vec![ts::param("x", ts::type_ref("string"))],
            ts::type_ref("DualInstance"),
        ))
        .with_property(ts::property("value", ts::type_ref("string")));

    let toks: ts::Tokens = quote! {
        $combined
    };

    assert_eq!(
        vec![
            "interface Dual {",
            "    (x: string): number;",
            "    new (x: string): DualInstance;",
            "    value: string;",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_call_signature_with_optional_params() -> genco::fmt::Result {
    let callable = ts::interface("OptionalCallback")
        .with_call_signature(ts::call_signature(
            vec![
                ts::param("required", ts::type_ref("string")),
                ts::param("optional", ts::type_ref("number")).optional(),
            ],
            Some(ts::type_ref("void")),
        ));

    let toks: ts::Tokens = quote! {
        $callable
    };

    assert_eq!(
        vec![
            "interface OptionalCallback {",
            "    (required: string, optional?: number): void;",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_keyof_operator() -> genco::fmt::Result {
    let keys_type = ts::type_alias("UserKeys", ts::keyof(ts::type_ref("User")));

    let toks: ts::Tokens = quote! {
        $keys_type
    };

    assert_eq!(
        vec!["type UserKeys = keyof User;"],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_indexed_access_type() -> genco::fmt::Result {
    let indexed = ts::type_alias("UserName", ts::type_ref("User").indexed_by("name"));

    let toks: ts::Tokens = quote! {
        $indexed
    };

    assert_eq!(
        vec!["type UserName = User[name];"],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_mapped_type_simple() -> genco::fmt::Result {
    let mapped = ts::type_alias(
        "StringMap",
        ts::mapped_type("K", ts::type_ref("string"), ts::type_ref("string")),
    );

    let toks: ts::Tokens = quote! {
        $mapped
    };

    assert_eq!(
        vec![
            "type StringMap = {",
            "    [K in string]: string;",
            "};"
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_mapped_type_readonly() -> genco::fmt::Result {
    let readonly = ts::type_alias(
        "Readonly",
        ts::mapped_type("P", ts::keyof(ts::type_ref("T")), ts::type_ref("T").indexed_by("P"))
            .readonly(),
    )
    .with_generic_params(vec![ts::generic_param("T")]);

    let toks: ts::Tokens = quote! {
        $readonly
    };

    assert_eq!(
        vec![
            "type Readonly<T> = {",
            "    readonly [P in keyof T]: T[P];",
            "};"
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_mapped_type_optional() -> genco::fmt::Result {
    let partial = ts::type_alias(
        "Partial",
        ts::mapped_type("P", ts::keyof(ts::type_ref("T")), ts::type_ref("T").indexed_by("P"))
            .optional(),
    )
    .with_generic_params(vec![ts::generic_param("T")]);

    let toks: ts::Tokens = quote! {
        $partial
    };

    assert_eq!(
        vec![
            "type Partial<T> = {",
            "    [P in keyof T]?: T[P];",
            "};"
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_mapped_type_required() -> genco::fmt::Result {
    let required = ts::type_alias(
        "Required",
        ts::mapped_type("P", ts::keyof(ts::type_ref("T")), ts::type_ref("T").indexed_by("P"))
            .required(),
    )
    .with_generic_params(vec![ts::generic_param("T")]);

    let toks: ts::Tokens = quote! {
        $required
    };

    assert_eq!(
        vec![
            "type Required<T> = {",
            "    [P in keyof T]-?: T[P];",
            "};"
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_mapped_type_pick() -> genco::fmt::Result {
    let pick = ts::type_alias(
        "Pick",
        ts::mapped_type("P", ts::type_ref("K"), ts::type_ref("T").indexed_by("P")),
    )
    .with_generic_params(vec![
        ts::generic_param("T"),
        ts::generic_param("K").with_constraint(ts::keyof(ts::type_ref("T"))),
    ]);

    let toks: ts::Tokens = quote! {
        $pick
    };

    assert_eq!(
        vec![
            "type Pick<T, K extends keyof T> = {",
            "    [P in K]: T[P];",
            "};"
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_mapped_type_record() -> genco::fmt::Result {
    let record = ts::type_alias(
        "Record",
        ts::mapped_type("P", ts::type_ref("K"), ts::type_ref("T")),
    )
    .with_generic_params(vec![
        ts::generic_param("K").with_constraint(ts::union_type(vec![
            ts::type_ref("string"),
            ts::type_ref("number"),
            ts::type_ref("symbol"),
        ])),
        ts::generic_param("T"),
    ]);

    let toks: ts::Tokens = quote! {
        $record
    };

    assert_eq!(
        vec![
            "type Record<K extends string | number | symbol, T> = {",
            "    [P in K]: T;",
            "};"
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_conditional_type_simple() -> genco::fmt::Result {
    let is_string = ts::type_alias(
        "IsString",
        ts::conditional_type(
            ts::type_ref("T"),
            ts::type_ref("string"),
            ts::literal("true"),
            ts::literal("false"),
        ),
    )
    .with_generic_params(vec![ts::generic_param("T")]);

    let toks: ts::Tokens = quote! {
        $is_string
    };

    assert_eq!(
        vec!["type IsString<T> = T extends string ? \"true\" : \"false\";"],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_conditional_type_with_types() -> genco::fmt::Result {
    let is_array = ts::type_alias(
        "IsArray",
        ts::conditional_type(
            ts::type_ref("T"),
            ts::type_ref("Array").with_generics(vec![ts::type_ref("any")]),
            ts::type_ref("T"),
            ts::type_ref("never"),
        ),
    )
    .with_generic_params(vec![ts::generic_param("T")]);

    let toks: ts::Tokens = quote! {
        $is_array
    };

    assert_eq!(
        vec!["type IsArray<T> = T extends Array<any> ? T : never;"],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_conditional_type_non_nullable() -> genco::fmt::Result {
    let non_nullable = ts::type_alias(
        "NonNullable",
        ts::conditional_type(
            ts::type_ref("T"),
            ts::union_type(vec![ts::type_ref("null"), ts::type_ref("undefined")]),
            ts::type_ref("never"),
            ts::type_ref("T"),
        ),
    )
    .with_generic_params(vec![ts::generic_param("T")]);

    let toks: ts::Tokens = quote! {
        $non_nullable
    };

    assert_eq!(
        vec!["type NonNullable<T> = T extends null | undefined ? never : T;"],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_conditional_type_extract() -> genco::fmt::Result {
    let extract = ts::type_alias(
        "Extract",
        ts::conditional_type(
            ts::type_ref("T"),
            ts::type_ref("U"),
            ts::type_ref("T"),
            ts::type_ref("never"),
        ),
    )
    .with_generic_params(vec![ts::generic_param("T"), ts::generic_param("U")]);

    let toks: ts::Tokens = quote! {
        $extract
    };

    assert_eq!(
        vec!["type Extract<T, U> = T extends U ? T : never;"],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_conditional_type_exclude() -> genco::fmt::Result {
    let exclude = ts::type_alias(
        "Exclude",
        ts::conditional_type(
            ts::type_ref("T"),
            ts::type_ref("U"),
            ts::type_ref("never"),
            ts::type_ref("T"),
        ),
    )
    .with_generic_params(vec![ts::generic_param("T"), ts::generic_param("U")]);

    let toks: ts::Tokens = quote! {
        $exclude
    };

    assert_eq!(
        vec!["type Exclude<T, U> = T extends U ? never : T;"],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_conditional_type_nested() -> genco::fmt::Result {
    let nested = ts::type_alias(
        "TypeName",
        ts::conditional_type(
            ts::type_ref("T"),
            ts::type_ref("string"),
            ts::literal("string"),
            ts::conditional_type(
                ts::type_ref("T"),
                ts::type_ref("number"),
                ts::literal("number"),
                ts::literal("other"),
            ),
        ),
    )
    .with_generic_params(vec![ts::generic_param("T")]);

    let toks: ts::Tokens = quote! {
        $nested
    };

    assert_eq!(
        vec![
            "type TypeName<T> = T extends string ? \"string\" : T extends number ? \"number\" : \"other\";"
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_arrow_function_type_simple() -> genco::fmt::Result {
    let handler = ts::type_alias(
        "Handler",
        ts::arrow_function_type(
            vec![ts::param("event", ts::type_ref("Event"))],
            ts::type_ref("void"),
        ),
    );

    let toks: ts::Tokens = quote! {
        $handler
    };

    assert_eq!(
        vec!["type Handler = (event: Event) => void;"],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_arrow_function_type_multiple_params() -> genco::fmt::Result {
    let comparator = ts::type_alias(
        "Comparator",
        ts::arrow_function_type(
            vec![
                ts::param("a", ts::type_ref("number")),
                ts::param("b", ts::type_ref("number")),
            ],
            ts::type_ref("boolean"),
        ),
    );

    let toks: ts::Tokens = quote! {
        $comparator
    };

    assert_eq!(
        vec!["type Comparator = (a: number, b: number) => boolean;"],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_typeof_operator() -> genco::fmt::Result {
    let config_type = ts::type_alias("ConfigType", ts::typeof_operator("defaultConfig"));

    let toks: ts::Tokens = quote! {
        $config_type
    };

    assert_eq!(
        vec!["type ConfigType = typeof defaultConfig;"],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_method_signature_simple() -> genco::fmt::Result {
    let api = ts::interface("API")
        .with_method_signature(ts::method_signature(
            "fetch",
            vec![ts::param("url", ts::type_ref("string"))],
            Some(ts::type_ref("Promise").with_generics(vec![ts::type_ref("Response")])),
        ));

    let toks: ts::Tokens = quote! {
        $api
    };

    assert_eq!(
        vec![
            "interface API {",
            "    fetch(url: string): Promise<Response>;",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_method_signature_multiple() -> genco::fmt::Result {
    let repository = ts::interface("Repository")
        .with_method_signature(ts::method_signature(
            "findById",
            vec![ts::param("id", ts::type_ref("string"))],
            Some(ts::type_ref("T")),
        ))
        .with_method_signature(ts::method_signature(
            "save",
            vec![ts::param("entity", ts::type_ref("T"))],
            Some(ts::type_ref("void")),
        ))
        .with_generic_params(vec![ts::generic_param("T")]);

    let toks: ts::Tokens = quote! {
        $repository
    };

    assert_eq!(
        vec![
            "interface Repository<T> {",
            "    findById(id: string): T;",
            "    save(entity: T): void;",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}

#[test]
fn test_method_signature_with_properties() -> genco::fmt::Result {
    let mixed = ts::interface("Mixed")
        .with_method_signature(ts::method_signature(
            "doSomething",
            vec![],
            Some(ts::type_ref("void")),
        ))
        .with_property(ts::property("count", ts::type_ref("number")));

    let toks: ts::Tokens = quote! {
        $mixed
    };

    assert_eq!(
        vec![
            "interface Mixed {",
            "    doSomething(): void;",
            "    count: number;",
            "}",
        ],
        toks.to_file_vec()?
    );
    Ok(())
}
