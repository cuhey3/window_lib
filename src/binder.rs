use crate::binder::element_manager::ElementManager;
use crate::binder::mouse_state::MouseState;
use crate::figure::Figure;
use crate::math::Point;
use wasm_bindgen::prelude::wasm_bindgen;

pub(crate) mod element_manager;
mod mouse_state;

#[wasm_bindgen]
pub struct Binder {
    figures: Vec<Figure>,
    mouse_state: MouseState,
    element_manager: ElementManager,
    pub(crate) has_update: bool,
}

#[wasm_bindgen]
impl Binder {
    pub fn new() -> Binder {
        let binder = Binder {
            figures: vec![],
            mouse_state: MouseState::new(),
            element_manager: ElementManager::new(),
            has_update: false,
        };
        binder.adjust();
        binder
    }
    #[wasm_bindgen(constructor)]
    pub fn new_for_dev() -> Binder {
        let mut element_manager = ElementManager::new();
        let binder = Binder {
            figures: vec![Figure::new_window_dev(&mut element_manager)],
            mouse_state: MouseState::new(),
            element_manager,
            has_update: false,
        };
        binder.adjust();
        binder
    }
    pub fn update(&mut self) {
        if self.has_update {
            self.adjust();
            self.has_update = false;
        }
    }
    pub fn mouse_up(&mut self) {
        self.mouse_state.is_dragged = false;
        self.update_base();
        self.has_update = true;
    }

    pub fn mouse_down(&mut self, raw_x: f64, raw_y: f64) {
        let (x, y) = self.element_manager.get_internal_xy(raw_x, raw_y);
        // mouse_down() => mouse_down() イベントを念の為抑制
        if self.mouse_state.is_dragged {
            self.mouse_up();
        }
        // 現状、一度につかめる Figure は一つだけ
        // Figure の重なりは考慮していない（先頭の一つがつかまれる）
        if let Some(found_figure) = self.figures.iter_mut().find(|figure| figure.is_inner(x, y)) {
            if found_figure.grab(x, y) {
                self.mouse_state.is_dragged = true;
                self.mouse_state.drag_start_point = Point { x, y };
            };
        };
        self.has_update = true;
    }

    pub fn mouse_move(&mut self, raw_x: f64, raw_y: f64) {
        let (x, y) = self.element_manager.get_internal_xy(raw_x, raw_y);
        if !self.mouse_state.is_dragged {
            return;
        }
        let delta_point = Point {
            x: x - self.mouse_state.drag_start_point.x,
            y: y - self.mouse_state.drag_start_point.y,
        };
        for figure in self.figures.iter_mut() {
            figure.move_xy(
                &self.mouse_state.drag_start_point,
                &delta_point,
                &self.element_manager,
            );
        }
        self.has_update = true;
    }

    pub fn set_ref_points(&mut self, offset_x: f64, offset_y: f64, max_y: f64) {
        self.element_manager.offset_x = offset_x;
        self.element_manager.offset_y = offset_y;
        // TODO
        // パラメータ化
        self.element_manager.scale = (max_y - offset_y) / 800.0;
    }
}

impl Binder {
    pub(crate) fn adjust(&self) {
        for figure in self.figures.iter() {
            figure.base_rect.adjust(&self.element_manager);
            for part_rect in figure.parts.iter() {
                part_rect.adjust(&figure.base_rect, &self.element_manager);
            }
        }
    }

    pub(crate) fn update_base(&mut self) {
        for figure in self.figures.iter_mut() {
            figure.update_base();
        }
    }
}
