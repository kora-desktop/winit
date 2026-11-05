//! xdg-dialog (`xdg_wm_dialog_v1`) client support: marks a toplevel modal to
//! its parent, so the compositor keeps input on it until it closes.

use sctk::globals::GlobalData;
use sctk::reexports::client::globals::{BindError, GlobalList};
use sctk::reexports::client::{delegate_dispatch, Connection, Dispatch, Proxy, QueueHandle};
use sctk::reexports::protocols::xdg::shell::client::xdg_toplevel::XdgToplevel;
use wayland_protocols::xdg::dialog::v1::client::xdg_dialog_v1::XdgDialogV1;
use wayland_protocols::xdg::dialog::v1::client::xdg_wm_dialog_v1::XdgWmDialogV1;

use crate::platform_impl::wayland::state::WinitState;

/// Wrapper around the `xdg_wm_dialog_v1` global.
#[derive(Debug, Clone)]
pub struct XdgDialogManager {
    manager: XdgWmDialogV1,
}

impl XdgDialogManager {
    pub fn new(
        globals: &GlobalList,
        queue_handle: &QueueHandle<WinitState>,
    ) -> Result<Self, BindError> {
        let manager = globals.bind(queue_handle, 1..=1, GlobalData)?;
        Ok(Self { manager })
    }

    /// Mark `toplevel` modal to its parent until the returned dialog drops.
    pub fn modal(
        &self,
        toplevel: &XdgToplevel,
        queue_handle: &QueueHandle<WinitState>,
    ) -> XdgDialog {
        let dialog = self.manager.get_xdg_dialog(toplevel, queue_handle, GlobalData);
        dialog.set_modal();
        XdgDialog { dialog }
    }
}

/// A toplevel's dialog hints; dropping it removes them.
#[derive(Debug)]
pub struct XdgDialog {
    dialog: XdgDialogV1,
}

impl Drop for XdgDialog {
    fn drop(&mut self) {
        self.dialog.destroy();
    }
}

impl Dispatch<XdgWmDialogV1, GlobalData, WinitState> for XdgDialogManager {
    fn event(
        _: &mut WinitState,
        _: &XdgWmDialogV1,
        _: <XdgWmDialogV1 as Proxy>::Event,
        _: &GlobalData,
        _: &Connection,
        _: &QueueHandle<WinitState>,
    ) {
    }
}

impl Dispatch<XdgDialogV1, GlobalData, WinitState> for XdgDialogManager {
    fn event(
        _: &mut WinitState,
        _: &XdgDialogV1,
        _: <XdgDialogV1 as Proxy>::Event,
        _: &GlobalData,
        _: &Connection,
        _: &QueueHandle<WinitState>,
    ) {
    }
}

delegate_dispatch!(WinitState: [XdgWmDialogV1: GlobalData] => XdgDialogManager);
delegate_dispatch!(WinitState: [XdgDialogV1: GlobalData] => XdgDialogManager);
