// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // `PingPort.exe mcp-bridge` runs the MCP stdio bridge instead of the GUI,
    // letting AI clients talk to the MCP server inside the running app.
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "mcp-bridge" {
        pingport_lib::mcp::bridge::run_bridge();
        return;
    }

    pingport_lib::run()
}
