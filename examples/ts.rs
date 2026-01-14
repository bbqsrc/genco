use genco::fmt;
use genco::prelude::*;

fn main() -> anyhow::Result<()> {
    // Import dependencies
    let react = &ts::import("react", "React").into_default();
    let user_type = &ts::import("./types", "User").into_type_only();
    let api_service = &ts::import("./services/api", "ApiService");

    // Define interfaces
    let props_interface = ts::interface("AppProps")
        .with_property(ts::property("title", ts::type_ref("string")))
        .with_property(ts::optional_property("userId", ts::type_ref("number")))
        .with_property(ts::property("onLoad", ts::type_ref("Function")));

    let state_interface = ts::interface("AppState")
        .with_property(ts::property(
            "user",
            ts::type_ref("User").with_generics(vec![]),
        ))
        .with_property(ts::property("loading", ts::type_ref("boolean")))
        .with_property(ts::optional_property("error", ts::type_ref("string")));

    // Define type aliases
    let user_id_type = ts::type_alias("UserID", ts::type_ref("number"));

    // Create union types (will be used inline in quote!{})
    let status_union = ts::union_type(vec![
        ts::literal("idle").into(),
        ts::literal("loading").into(),
        ts::literal("success").into(),
        ts::literal("error").into(),
    ]);

    let result_union = ts::union_type(vec![
        ts::type_ref("Success").with_generics(vec![ts::type_ref("T")]),
        ts::type_ref("Error"),
    ]);

    // Define an enum for action types
    let action_enum = ts::enum_type("ActionType")
        .with_variant("LOAD_START", Some(quote!("LOAD_START")))
        .with_variant("LOAD_SUCCESS", Some(quote!("LOAD_SUCCESS")))
        .with_variant("LOAD_ERROR", Some(quote!("LOAD_ERROR")));

    // Generate the TypeScript code
    let tokens = quote! {
        $(props_interface)

        $(state_interface)

        $(user_id_type)

        type LoadStatus = $status_union;

        type Result<T> = $result_union;

        $(action_enum)

        export default class App extends $react.Component<AppProps, AppState> {
            state: AppState = {
                user: null,
                loading: false,
                error: undefined,
            };

            async componentDidMount(): Promise<void> {
                const { userId, onLoad } = this.props;

                if (userId) {
                    await this.loadUser(userId);
                    onLoad();
                }
            }

            async loadUser(id: UserID): Promise<void> {
                this.setState({ loading: true, error: undefined });

                try {
                    const user: $user_type = await $api_service.fetchUser(id);
                    this.setState({ user, loading: false });
                } catch (err) {
                    this.setState({
                        loading: false,
                        error: err instanceof Error ? err.message : "Unknown error",
                    });
                }
            }

            render(): JSX.Element {
                const { title } = this.props;
                const { user, loading, error } = this.state;

                return (
                    <div className="app">
                        <h1>{title}</h1>
                        {loading && <div>Loading...</div>}
                        {error && <div className="error">{error}</div>}
                        {user && (
                            <div className="user-info">
                                <h2>{user.name}</h2>
                                <p>{user.email}</p>
                            </div>
                        )}
                    </div>
                );
            }
        }
    };

    // Output the generated TypeScript code
    let stdout = std::io::stdout();
    let mut w = fmt::IoWriter::new(stdout.lock());

    let fmt = fmt::Config::from_lang::<TypeScript>();
    let config = ts::Config::default();

    tokens.format_file(&mut w.as_formatter(&fmt), &config)?;
    Ok(())
}
