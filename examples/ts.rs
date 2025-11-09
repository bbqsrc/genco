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
    let callback_type = ts::type_alias(
        "LoadCallback",
        ts::type_ref("Function"), // In real usage, you'd define proper function signature
    );

    // Define an enum
    let status_enum = ts::enum_type("LoadStatus")
        .with_variant("Idle", Some(quote!("idle")))
        .with_variant("Loading", Some(quote!("loading")))
        .with_variant("Success", Some(quote!("success")))
        .with_variant("Error", Some(quote!("error")));

    // Generate the TypeScript code
    let tokens = quote! {
        $(props_interface)

        $(state_interface)

        $(user_id_type)

        $(callback_type)

        $(status_enum)

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
