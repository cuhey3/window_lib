use crate::math::Point;

pub(crate) struct MouseState {
    pub(crate) is_dragged: bool,
    pub(crate) drag_start_point: Point,
}

impl MouseState {
    pub(crate) fn new() -> MouseState {
        MouseState {
            is_dragged: false,
            drag_start_point: Point::new(),
        }
    }
}
