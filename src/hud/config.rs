// Configuration: values we choose for the HUD.
// Window size and behavior.
pub(crate) const HUD_WIDTH: f32 = 340.0;
pub(crate) const HUD_HEIGHT: f32 = 420.0;
pub(crate) const HUD_DECORATIONS: bool = false;
pub(crate) const HUD_TRANSPARENT: bool = true;

// Border shape and the space reserved for future resize handles.
pub(super) const BORDER_WIDTH: f32 = 8.0;
pub(super) const CORNER_RADIUS: f32 = 12.0;
pub(super) const RESIZE_CORNER_SIZE: f32 = 16.0;

// Colors in red, green, blue, alpha order. Alpha controls transparency.
pub(super) const WORKSPACE_RGBA: [u8; 4] = [28, 37, 52, 200];
pub(super) const BORDER_RGBA: [u8; 4] = [17, 25, 40, 200];

// Window title and temporary workspace content.
pub(crate) const HUD_TITLE: &str = "FIFO Tasker";
pub(super) const PLACEHOLDER_TEXT: &str = "Hello, FIFO Tasker!";
pub(super) const CONTENT_TOP_SPACING: f32 = 20.0;

// Derived values: calculated from the configuration above.
pub(super) const BORDER_INSET: f32 = BORDER_WIDTH / 2.0;
