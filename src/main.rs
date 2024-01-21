use std::env;

use hyprland::{
    data::{Monitors, Workspace},
    dispatch::DispatchType,
    event_listener::{EventListenerMutable, State},
    shared::{HyprData, WorkspaceType},
};

#[tokio::main]
async fn main() -> hyprland::Result<()> {
    let args: Vec<String> = env::args().collect();

    //Defaults to 10 workspaces per monitor if command line arg produces error when parsing.
    let max_workspaces = Box::new(args[1].parse().unwrap_or(10));

    // Because of the jankness of map, this works only if you have 10 workspaces per monitor.
    let monitors = Monitors::get_async().await?;

    // Create keybindings for moving to workspaces and moving window to workspaces.
    // Keybindings only allow access up to 10 workspaces.
    // If the max_workspaces is greater than max(i32), will panic.
    let bindable_workspaces = if *max_workspaces < 10 {
        *max_workspaces
    } else {
        10
    };
    let workspace_ids_per_monitor = (0..bindable_workspaces.try_into().expect(
        "Why does your number of bindable workspaces go over i32. What on earth is going on.",
    ))
        .collect::<Vec<i32>>();

    for i in workspace_ids_per_monitor {
        hyprland::bind!(async; SUPER, Key, "{i}" => Workspace, hyprland::dispatch::WorkspaceIdentifierWithSpecial::Id(i)).await?;
        hyprland::bind!(async; SUPER SHIFT, Key, "{i}" => MoveFocusedWindowToWorkspaceSilent, hyprland::dispatch::WorkspaceIdentifier::Id(i))
            .await?;
    }

    // TODO: Bind workspaces to particular monitors.
    // TODO: Need to update monitors on addition or removal of monitors.

    let mut event_listener = EventListenerMutable::new();

    event_listener.add_workspace_change_handler(|id, state| {
        map_workspace(id, state, &monitors, max_workspaces)
    });
    event_listener.add_workspace_added_handler(|id, state| {
        map_workspace(id, state, &monitors, max_workspaces)
    });
    // event_listener.add_active_window_change_handler(f)

    // let modified_workspace_id =
    // state.active_workspace = WorkspaceType::Regular()
    event_listener.start_listener_async().await
}

fn map_workspace(id: WorkspaceType, state: &mut State, monitors: &Monitors, max_workspaces: usize) {
    let monitor_index = match monitors.iter().position(|x| x.name == state.active_monitor){
        Some(x) => x,
        None => panic!("Should have been able to find initialized monitors and map to available monitors. Likely a monitor was removed or added since the program started.")
    };
    // let monitor_id = match state.active_monitor.as_str() {
    //     "HDMI-A-1" => "",
    //     "DP-1" => "1",
    //     "DP-2" => "2",
    //     _ => panic!("Arya you idjit you messed up the monitors."),
    // }
    // .to_string();
    let new_workspace_id: usize = match id {
        WorkspaceType::Regular(x) => x,
        WorkspaceType::Special(_) => {
            panic!("Changed type to a special workspace instead of a regular one.")
        }
    }
    .parse()
    .expect("Name of workspace should have been parsable");

    let new_workspace_modified_id: usize = (max_workspaces * monitor_index) + new_workspace_id;
    state.active_workspace =
        WorkspaceType::Regular(String::from(new_workspace_modified_id.to_string()));
}
