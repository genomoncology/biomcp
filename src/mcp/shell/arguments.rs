use rmcp::handler::server::{common::FromContextPart, tool::ToolCallContext};
use rmcp::{ErrorData, schemars};
use serde::de::DeserializeOwned;
use serde_json::Value;

/// Keep typed parsing and field paths at one boundary for every MCP tool.
#[derive(Debug)]
pub(super) struct Parameters<T>(pub T);

impl<T: schemars::JsonSchema> schemars::JsonSchema for Parameters<T> {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        T::schema_name()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        T::json_schema(generator)
    }
}

impl<T: DeserializeOwned, S> FromContextPart<ToolCallContext<'_, S>> for Parameters<T> {
    fn from_context_part(context: &mut ToolCallContext<S>) -> Result<Self, ErrorData> {
        let arguments = Value::Object(context.arguments.take().unwrap_or_default());
        serde_path_to_error::deserialize(arguments)
            .map(Self)
            .map_err(|error| {
                ErrorData::invalid_params(format!("Invalid tool arguments: {error}"), None)
            })
    }
}
