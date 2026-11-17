//! Handling of KDE-compatible blur.
//!
//! Bound from our copy of the protocol, which adds the requests that set the blur's strength,
//! saturation, frosted tint and border.

use sctk::reexports::client::globals::{BindError, GlobalList};
use sctk::reexports::client::protocol::wl_surface::WlSurface;
use sctk::reexports::client::{delegate_dispatch, Connection, Dispatch, Proxy, QueueHandle};

use sctk::globals::GlobalData;

use crate::platform_impl::wayland::state::WinitState;
use crate::platform_impl::wayland::types::xdg_popup::PopupBlurStyle;

pub mod protocol {
    use wayland_client;
    use wayland_client::protocol::*;

    pub mod __interfaces {
        use wayland_client::protocol::__interfaces::*;
        wayland_scanner::generate_interfaces!("resources/protocols/kwin-blur.xml");
    }
    use self::__interfaces::*;

    wayland_scanner::generate_client_code!("resources/protocols/kwin-blur.xml");
}

pub use protocol::org_kde_kwin_blur::OrgKdeKwinBlur;
pub use protocol::org_kde_kwin_blur_manager::OrgKdeKwinBlurManager;

/// KWin blur manager.
#[derive(Debug, Clone)]
pub struct KWinBlurManager {
    manager: OrgKdeKwinBlurManager,
}

impl KWinBlurManager {
    pub fn new(
        globals: &GlobalList,
        queue_handle: &QueueHandle<WinitState>,
    ) -> Result<Self, BindError> {
        let manager = globals.bind(queue_handle, 1..=3, GlobalData)?;
        Ok(Self { manager })
    }

    pub fn blur(
        &self,
        surface: &WlSurface,
        queue_handle: &QueueHandle<WinitState>,
    ) -> OrgKdeKwinBlur {
        self.manager.create(surface, queue_handle, ())
    }

    pub fn unset(&self, surface: &WlSurface) {
        self.manager.unset(surface)
    }
}

/// Send `style` on a blur object before its commit. Each value goes out only where the bound
/// version has its request; one left out keeps the compositor's default.
pub fn send_style(blur: &OrgKdeKwinBlur, style: &PopupBlurStyle) {
    for request in style_requests(style, blur.version()) {
        match request {
            StyleRequest::Radius(radius) => blur.set_radius(radius),
            StyleRequest::Saturation(saturation) => blur.set_saturation(saturation),
            StyleRequest::Tint(tint) => blur.set_tint(tint),
            StyleRequest::Border(border) => blur.set_border(border),
        }
    }
}

/// A style request and its value, in the protocol's 8-bit fixed point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StyleRequest {
    Radius(i32),
    Saturation(i32),
    Tint(i32),
    Border(i32),
}

fn style_requests(style: &PopupBlurStyle, version: u32) -> Vec<StyleRequest> {
    let fixed = |value: f32| (value * 256.0) as i32;
    let mut requests = Vec::new();
    if version >= 2 {
        requests.extend(style.radius.map(|radius| StyleRequest::Radius(fixed(radius))));
    }
    if version >= 3 {
        requests.extend(style.saturation.map(|value| StyleRequest::Saturation(fixed(value))));
        requests.extend(style.tint.map(|value| StyleRequest::Tint(fixed(value))));
        requests.extend(style.border.map(|value| StyleRequest::Border(fixed(value))));
    }
    requests
}

impl Dispatch<OrgKdeKwinBlurManager, GlobalData, WinitState> for KWinBlurManager {
    fn event(
        _: &mut WinitState,
        _: &OrgKdeKwinBlurManager,
        _: <OrgKdeKwinBlurManager as Proxy>::Event,
        _: &GlobalData,
        _: &Connection,
        _: &QueueHandle<WinitState>,
    ) {
        unreachable!("no events defined for org_kde_kwin_blur_manager");
    }
}

impl Dispatch<OrgKdeKwinBlur, (), WinitState> for KWinBlurManager {
    fn event(
        _: &mut WinitState,
        _: &OrgKdeKwinBlur,
        _: <OrgKdeKwinBlur as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WinitState>,
    ) {
        unreachable!("no events defined for org_kde_kwin_blur");
    }
}

delegate_dispatch!(WinitState: [OrgKdeKwinBlurManager: GlobalData] => KWinBlurManager);
delegate_dispatch!(WinitState: [OrgKdeKwinBlur: ()] => KWinBlurManager);

#[cfg(test)]
mod tests {
    use super::{style_requests, StyleRequest};
    use crate::platform_impl::wayland::types::xdg_popup::PopupBlurStyle;

    const GLASS: PopupBlurStyle = PopupBlurStyle {
        radius: Some(40.0),
        saturation: Some(1.3),
        tint: Some(0.0),
        border: Some(0.0),
    };

    #[test]
    fn a_styled_blur_sends_every_value_in_fixed_point() {
        assert_eq!(style_requests(&GLASS, 3), vec![
            StyleRequest::Radius(10240),
            StyleRequest::Saturation(332),
            StyleRequest::Tint(0),
            StyleRequest::Border(0),
        ]);
    }

    #[test]
    fn an_older_compositor_gets_only_what_its_version_knows() {
        assert_eq!(style_requests(&GLASS, 2), vec![StyleRequest::Radius(10240)]);
        assert!(style_requests(&GLASS, 1).is_empty());
    }

    #[test]
    fn an_unstyled_blur_keeps_the_compositor_defaults() {
        assert!(style_requests(&PopupBlurStyle::default(), 3).is_empty());
    }
}
