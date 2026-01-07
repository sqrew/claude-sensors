use display_info::DisplayInfo;
use rmcp::{
    handler::server::{router::tool::ToolRouter, ServerHandler, wrapper::Parameters},
    model::*,
    ErrorData as McpError,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Parameters for get_display_at_point
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PointParams {
    #[schemars(description = "X coordinate on screen")]
    pub x: i32,
    #[schemars(description = "Y coordinate on screen")]
    pub y: i32,
}

/// Parameters for get_display_by_name
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct NameParams {
    #[schemars(description = "Display name to search for")]
    pub name: String,
}

#[derive(Debug)]
pub struct DisplayServer {
    pub tool_router: ToolRouter<Self>,
}

impl Default for DisplayServer {
    fn default() -> Self {
        Self::new()
    }
}

impl DisplayServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    fn format_single_display(d: &DisplayInfo) -> String {
        let mut result = String::new();

        // Header with name and primary indicator
        let primary = if d.is_primary { " (primary)" } else { "" };
        result.push_str(&format!(
            "{}{}\n",
            if d.friendly_name.is_empty() { &d.name } else { &d.friendly_name },
            primary
        ));

        // Resolution and position
        result.push_str(&format!("  Resolution: {}x{}\n", d.width, d.height));
        result.push_str(&format!("  Position: ({}, {})\n", d.x, d.y));

        // Physical size if available
        if d.width_mm > 0 && d.height_mm > 0 {
            let diag_mm = ((d.width_mm.pow(2) + d.height_mm.pow(2)) as f32).sqrt();
            let diag_inches = diag_mm / 25.4;
            result.push_str(&format!(
                "  Physical: {}mm x {}mm (~{:.1}\")\n",
                d.width_mm, d.height_mm, diag_inches
            ));
        }

        // Refresh rate
        if d.frequency > 0.0 {
            result.push_str(&format!("  Refresh: {:.0}Hz\n", d.frequency));
        }

        // Scale factor
        if d.scale_factor != 1.0 {
            result.push_str(&format!("  Scale: {:.0}%\n", d.scale_factor * 100.0));
        }

        // Rotation
        if d.rotation != 0.0 {
            result.push_str(&format!("  Rotation: {}°\n", d.rotation as i32));
        }

        result
    }

    fn format_display_info(displays: &[DisplayInfo]) -> String {
        let mut result = String::from("Display Information:\n\n");

        if displays.is_empty() {
            result.push_str("No displays detected.\n");
            return result;
        }

        for (i, d) in displays.iter().enumerate() {
            result.push_str(&format!("Display {}: ", i + 1));
            result.push_str(&Self::format_single_display(d));
            result.push('\n');
        }

        result.push_str(&format!("Total displays: {}\n", displays.len()));
        result
    }
}

#[rmcp::tool_router]
impl DisplayServer {
    #[rmcp::tool(description = "Get display/monitor information (connected displays, resolutions, physical sizes)")]
    pub async fn get_display_info(&self) -> Result<CallToolResult, McpError> {
        let displays = DisplayInfo::all()
            .map_err(|e| McpError::internal_error(format!("Failed to get display info: {}", e), None))?;

        let formatted = Self::format_display_info(&displays);

        Ok(CallToolResult::success(vec![Content::text(formatted)]))
    }

    #[rmcp::tool(description = "Get display info at specific screen coordinates (useful for determining which monitor contains a point)")]
    pub async fn get_display_at_point(
        &self,
        Parameters(params): Parameters<PointParams>,
    ) -> Result<CallToolResult, McpError> {
        let display = DisplayInfo::from_point(params.x, params.y)
            .map_err(|e| McpError::internal_error(format!("Failed to get display at ({}, {}): {}", params.x, params.y, e), None))?;

        let formatted = format!(
            "Display at ({}, {}):\n{}",
            params.x, params.y,
            Self::format_single_display(&display)
        );

        Ok(CallToolResult::success(vec![Content::text(formatted)]))
    }

    #[rmcp::tool(description = "Get display info by name")]
    pub async fn get_display_by_name(
        &self,
        Parameters(params): Parameters<NameParams>,
    ) -> Result<CallToolResult, McpError> {
        let display = DisplayInfo::from_name(&params.name)
            .map_err(|e| McpError::internal_error(format!("Failed to get display '{}': {}", params.name, e), None))?;

        let formatted = Self::format_single_display(&display);

        Ok(CallToolResult::success(vec![Content::text(formatted)]))
    }
}

#[rmcp::tool_handler]
impl ServerHandler for DisplayServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("Cross-platform display/monitor information server".into()),
        }
    }
}
