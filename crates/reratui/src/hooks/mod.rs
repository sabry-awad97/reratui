//! React-like hooks with proper semantics.
//!
//! This module provides v2 versions of hooks that implement React-like behavior:
//! - Effects run after commit, not during render
//! - State updates are batched
//! - Context providers have proper lifecycle

mod async_hooks;
mod context;
mod effect;
mod effect_event;
mod event;
mod form;
mod history;
mod id;
mod keyboard;
mod layout;
mod memo;
mod mouse;
mod reducer;
mod r#ref;
mod state;
mod timing;

pub use async_hooks::{
    FutureHandleV2, FutureState, MutationHandleV2, MutationOptions, MutationState, MutationStatus,
    QueryOptions, QueryResultV2, QueryStatus, clear_query_cache, use_future_once, use_future_v2,
    use_mutation_v2, use_query_v2,
};
pub use context::{try_use_context_v2, use_context_provider_v2, use_context_v2};
pub use effect::{use_async_effect_once, use_async_effect_v2, use_effect_once, use_effect_v2};
pub use effect_event::{EffectEventV2, use_effect_event_v2};
pub use event::use_event;
pub use form::{
    FieldRegistrationV2, FormConfigBuilderV2, FormConfigV2, FormHandleV2, FormStateV2, ValidatorV2,
    try_use_form_context_v2, use_form_context_v2, use_form_v2, use_watch_all_v2,
    use_watch_multiple_v2, use_watch_v2,
};
pub use history::{HistoryHandle, use_history_v2};
pub use id::use_id_v2;
pub use keyboard::{use_keyboard_press_v2, use_keyboard_shortcut_v2, use_keyboard_v2};
pub use layout::{
    ComponentArea, FrameContext, FrameInfo, try_use_area_v2, try_use_frame_v2, use_area_v2,
    use_frame_info_v2, use_frame_v2, use_media_query_v2, use_on_resize_v2, use_resize_v2,
};
pub use memo::{use_callback_v2, use_memo_v2};
pub use mouse::{
    DragInfo, use_double_click_v2, use_mouse_click_v2, use_mouse_drag_v2, use_mouse_hover_v2,
    use_mouse_position_v2, use_mouse_v2,
};
pub use reducer::{DispatchV2, use_reducer_v2};
pub use r#ref::{RefV2, use_ref_v2};
pub use state::{StateSetterV2, use_state_v2};
pub use timing::{IntervalHandle, TimeoutHandle, use_interval_v2, use_timeout_v2};
