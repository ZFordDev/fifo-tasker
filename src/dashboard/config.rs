// Configuration: values we choose for the dashboard.
// Window size and behavior.
pub(crate) const DASHBOARD_WIDTH: f32 = 1000.0; // nice and big clearly different to the hud
pub(crate) const DASHBOARD_HEIGHT: f32 = 650.0;
pub(crate) const DASHBOARD_DECORATIONS: bool = true;
pub(crate) const DASHBOARD_TRANSPARENT: bool = false;

// Custom workspace border.
pub(super) const BORDER_WIDTH: f32 = 8.0;
pub(super) const CORNER_RADIUS: f32 = 12.0;

// Colors in red, green, blue, alpha order. Alpha controls transparency.
pub(super) const WORKSPACE_RGBA: [u8; 4] = [28, 37, 52, 200];
pub(super) const BORDER_RGBA: [u8; 4] = [17, 25, 40, 200];

// Window title and content spacing.
pub(crate) const DASHBOARD_TITLE: &str = "FIFO Tasker Dashboard";
pub(super) const CONTENT_TOP_SPACING: f32 = 20.0;

// Derived values: calculated from the configuration above.
pub(super) const BORDER_INSET: f32 = BORDER_WIDTH / 2.0;
