use crate::binder::element_manager::ElementManager;
use crate::binder::mouse_state::MouseState;
use crate::figure::Figure;
use crate::math::Point;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_test::console_log;

pub(crate) mod element_manager;
mod mouse_state;

#[wasm_bindgen]
pub struct Binder {
    figures: Vec<Figure>,
    mouse_state: MouseState,
    element_manager: ElementManager,
    pub(crate) has_update: bool,
    pub(crate) content_manager: ContentManager,
}

impl Binder {
    pub fn set_table_content_state(&mut self, table_content: Box<dyn TableContent>) {
        self.content_manager.table_content = Some(table_content);
    }
}
#[wasm_bindgen]
impl Binder {
    pub fn new() -> Binder {
        let mut binder = Binder {
            figures: vec![],
            mouse_state: MouseState::new(),
            element_manager: ElementManager::new("container"),
            has_update: false,
            content_manager: ContentManager {
                table_content: None,
            },
        };
        binder.initial_adjust();
        binder
    }
    #[wasm_bindgen(constructor)]
    pub fn new_for_dev() -> Binder {
        let mut element_manager = ElementManager::new("container");
        let mut binder = Binder {
            figures: vec![
                Figure::new_window_dev(
                    "プレイヤー1",
                    100.0,
                    100.0,
                    "#333",
                    "status1",
                    &mut element_manager,
                ),
                Figure::new_window_dev(
                    "プレイヤー2",
                    350.0,
                    100.0,
                    "#333",
                    "status2",
                    &mut element_manager,
                ),
                Figure::new_log_window_dev(
                    "ゲームログ",
                    100.0,
                    650.0,
                    "#333",
                    "log",
                    &mut element_manager,
                ),
            ],
            mouse_state: MouseState::new(),
            element_manager,
            has_update: false,
            content_manager: ContentManager {
                table_content: None,
            },
        };
        binder.initial_adjust();
        binder
    }

    pub fn set_dummy_state(&mut self) {
        self.content_manager.table_content = Some(Box::new(DummyState {}));
    }
    pub fn update(&mut self) {
        if self.has_update {
            self.adjust();
            self.has_update = false;
        }
    }
    pub fn mouse_up(&mut self, raw_x: f64, raw_y: f64) {
        if self.mouse_state.is_button_pushed {
            let (x, y) = self.element_manager.get_internal_xy(raw_x, raw_y);
            if let Some(found_figure) = self.figures.iter_mut().find(|figure| figure.is_pushed) {
                found_figure.button_pressed(x, y, &self.element_manager);
            }
            self.mouse_state.is_button_pushed = false;
        }
        self.mouse_state.is_dragged = false;
        self.update_base();
        self.has_update = true;
    }

    pub fn mouse_down(&mut self, raw_x: f64, raw_y: f64) {
        let (x, y) = self.element_manager.get_internal_xy(raw_x, raw_y);
        // mouse_down() => mouse_down() イベントを念の為抑制
        if self.mouse_state.is_dragged {
            self.mouse_up(raw_x, raw_y);
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
            }
            if found_figure.is_pushed {
                self.mouse_state.is_button_pushed = true;
            }
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
                part_rect.adjust(
                    &figure.base_rect,
                    &mut self.element_manager,
                    &mut self.content_manager,
                );
            }
        }
    }
    pub(crate) fn initial_adjust(&mut self) {
        for figure in self.figures.iter_mut() {
            figure.base_rect.initial_adjust(&self.element_manager);
            for part_rect in figure.parts.iter_mut() {
                part_rect.adjust(
                    &figure.base_rect,
                    &mut self.element_manager,
                    &self.content_manager,
                );
            }
            figure.adjust(&self.element_manager);
        }
        self.adjust();
    }

    pub(crate) fn update_base(&mut self) {
        for figure in self.figures.iter_mut() {
            figure.update_base();
        }
    }
}

pub struct ContentManager {
    pub(crate) table_content: Option<Box<dyn TableContent>>,
}

impl ContentManager {
    pub(crate) fn get_tbody(&self, key: &str) -> Option<Vec<Vec<String>>> {
        if let Some(content) = &self.table_content {
            Some(content.get_tbody(key))
        } else {
            None
        }
    }
}
pub trait TableContent {
    fn get_thead(&self, key: &str) -> Vec<String> {
        match key {
            _ => vec![],
        }
    }
    fn get_tbody(&self, key: &str) -> Vec<Vec<String>> {
        match key {
            _ => vec![
                vec![String::from("行動順"), String::from("後攻")],
                vec![String::from("HP/MHP"), String::from("50/50")],
                vec![String::from("被ダメ"), String::from("5")],
            ],
        }
    }
}

struct DummyState {}

impl TableContent for DummyState {}
