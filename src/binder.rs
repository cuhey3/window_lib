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
        let mut binder = Binder {
            figures: vec![],
            mouse_state: MouseState::new(),
            element_manager: ElementManager::new("container"),
            has_update: false,
        };
        binder.initial_adjust();
        binder
    }
    #[wasm_bindgen(constructor)]
    pub fn new_for_dev() -> Binder {
        let mut element_manager = ElementManager::new("container");
        let mut binder = Binder {
            figures: vec![
                Figure::new_window_dev(100.0, 100.0, "#999", "status1", &mut element_manager),
                Figure::new_window_dev(300.0, 300.0, "#999", "status2", &mut element_manager),
                Figure::new_log_window_dev(50.0, 700.0, "#999", "log", &mut element_manager),
            ],
            mouse_state: MouseState::new(),
            element_manager,
            has_update: false,
        };
        binder.initial_adjust();
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
        // TODO
        // element_manager 上の group_index と、binder の figures の index が一致しているという前提に基づいたロジック
        // figure の作成順と g 要素登録順がズレるとバグる（そんなケースありますか？）
        if let Some(group_index) = self
            .element_manager
            .figure_group_order
            .iter()
            .find(|group_index| self.figures[**group_index].is_inner(x, y))
        {
            let found_figure = &mut self.figures[*group_index];
            self.element_manager
                .re_append_figure(found_figure.group_index);
            if found_figure.grab(x, y) {
                self.mouse_state.is_dragged = true;
                self.mouse_state.drag_start_point = Point { x, y };
            };
        }
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
    pub(crate) fn adjust(&mut self) {
        for figure in self.figures.iter_mut() {
            figure.adjust(&self.element_manager);
            figure.base_rect.adjust(&mut self.element_manager);
            for part_rect in figure.parts.iter_mut() {
                part_rect.adjust(&figure.base_rect, &mut self.element_manager);
            }
        }
    }
    pub(crate) fn initial_adjust(&mut self) {
        for figure in self.figures.iter_mut() {
            figure.adjust(&self.element_manager);
            figure.base_rect.initial_adjust(&self.element_manager);
            for part_rect in figure.parts.iter_mut() {
                part_rect.adjust(&figure.base_rect, &mut self.element_manager);
            }
        }
    }

    pub(crate) fn update_base(&mut self) {
        for figure in self.figures.iter_mut() {
            figure.update_base();
        }
    }
}
