use crate::binder::element_manager::ElementManager;
use crate::content::{ColumnStyle, StringBinder, TableContentState, TextAnchorType};
use crate::figure::part_rect::{ButtonType, MinimizeOption, PartRect, ShowContentOption};
use crate::figure::AmountPositionType::{End, Start};
use crate::math::{Amount, Point};
use base_rect::BaseRect;

mod base_rect;
pub(crate) mod part_rect;

#[derive(Clone)]
pub(crate) struct RectLength {
    pub(crate) min: f64,
    pub(crate) max: f64,
    pub(crate) default: f64,
    pub(crate) amount: Amount,
    pub(crate) is_fixed: bool,
}

impl RectLength {
    pub(crate) fn value(&self) -> f64 {
        self.amount.value().max(self.min)
    }
    pub(crate) fn fix(&mut self) {
        self.amount.base = (self.amount.base + self.amount.delta).max(self.min);
        self.amount.delta = 0.0;
    }
    pub(crate) fn delta_constraint(&mut self) {
        if self.amount.base + self.amount.delta < self.min {
            self.amount.delta = self.min - self.amount.base;
        }
    }
    pub(crate) fn new_with_min(length: f64, min_length: f64) -> RectLength {
        RectLength {
            min: min_length,
            max: 0.0,
            default: 0.0,
            amount: Amount::new(length),
            is_fixed: false,
        }
    }
}

#[derive(Clone)]
pub(crate) enum AmountPositionType {
    Start,
    End,
    ContentBase,
    Ignore,
}

#[derive(Clone, Debug)]
pub(crate) enum PartType {
    Ignore,
    Button(ButtonType),
    Expand,
    Drag,
    Scrollable,
    ScrollBarX(ScrollBarState),
    ScrollBarY(ScrollBarState),
    TableContent(TableContentState),
}

#[derive(Clone, Debug)]
pub(crate) struct ScrollBarState {
    start_amount: Amount,
    percentage: f64,
    length: f64,
}

impl ScrollBarState {
    pub(crate) fn new() -> ScrollBarState {
        ScrollBarState {
            start_amount: Amount {
                base: 0.0,
                delta: 0.0,
            },
            percentage: 0.0,
            length: 0.0,
        }
    }
    fn update_delta_in_constraint(&mut self, min: f64, max: f64) {
        let value = self.start_amount.value();
        if value < min {
            self.start_amount.delta = min - self.start_amount.base;
        } else if value >= max {
            self.start_amount.delta = max - self.start_amount.base;
        }
    }
    fn update(&mut self, content: f64, scrollable: f64) {
        if content <= scrollable {
            self.length = 0.0;
        } else {
            self.length = scrollable * scrollable / content;
            let full_delta = scrollable - self.length;
            let delta = self.percentage * full_delta;
            self.start_amount.delta = delta - self.start_amount.base;
            self.update_delta_in_constraint(0.0, full_delta);
        }
    }
}

pub(crate) struct Figure {
    pub(crate) base_rect: BaseRect,
    pub(crate) parts: Vec<PartRect>,
    pub(crate) is_grabbed: bool,
    pub(crate) is_pushed: bool,
    pub(crate) group_index: usize,
}

impl Figure {
    pub(crate) fn button_pressed(&mut self, x: f64, y: f64, element_manager: &ElementManager) {
        self.is_pushed = false;
        let cloned_base_rect = self.base_rect.clone();
        if let Some(found_parts) = self.parts.iter_mut().find(|parts| parts.is_pushed) {
            found_parts.is_pushed = false;
            if !found_parts.is_inner(x, y, &cloned_base_rect) {
                return;
            }
            if let PartType::Button(button_type) = &found_parts.part_type {
                match button_type.clone() {
                    ButtonType::Minimize(minimize_option) => {
                        minimize_option.minimize_window(self);
                    }
                    ButtonType::ShowContent(show_content_option) => {
                        show_content_option.adjust_to_show_content(self, element_manager);
                    }
                }
            }
        };
    }

    pub(crate) fn adjust(&self, element_manager: &ElementManager) {
        let group_element = &element_manager.figure_groups[self.group_index];
        group_element
            .set_attribute(
                "transform",
                format!(
                    "translate({}, {})",
                    self.base_rect.x_amount.value(),
                    self.base_rect.y_amount.value()
                )
                .as_str(),
            )
            .unwrap()
    }
    pub(crate) fn new_log_window_dev(
        x: f64,
        y: f64,
        frame_color: &str,
        table_content_token: &str,
        element_manager: &mut ElementManager,
    ) -> Figure {
        let container = element_manager.get_container();
        let group_index = element_manager.create_figure_group(&container);
        let mut table_content_state = TableContentState::new(table_content_token);
        table_content_state.tbody_data = vec![
            vec![
                StringBinder::new_with_str("行動順"),
                StringBinder::new_with_str("後攻"),
            ],
            vec![
                StringBinder::new_with_str("HP/MHP"),
                StringBinder::new_with_str("50/50"),
            ],
            vec![
                StringBinder::new_with_str("被ダメ"),
                StringBinder::new_with_str("5"),
            ],
        ];
        table_content_state.tbody_column_styles = vec![
            ColumnStyle {
                defs_id: "".to_string(),
                text_anchor_type: TextAnchorType::Start,
                x: 5.0,
                font_size: 20.0,
                first_y: 25.0,
                dy: 25.0,
            },
            ColumnStyle {
                defs_id: "".to_string(),
                text_anchor_type: TextAnchorType::End,
                x: 205.0,
                font_size: 20.0,
                first_y: 25.0,
                dy: 25.0,
            },
        ];
        Figure::default_window(
            x,
            y,
            RectLength::new_with_min(1000.0, 150.0),
            RectLength::new_with_min(90.0, 30.0),
            frame_color,
            5.0,
            25.0,
            PartType::TableContent(table_content_state),
            element_manager,
            group_index,
        )
    }
    pub(crate) fn new_window_dev(
        x: f64,
        y: f64,
        frame_color: &str,
        table_content_token: &str,
        element_manager: &mut ElementManager,
    ) -> Figure {
        let container = element_manager.get_container();
        let group_index = element_manager.create_figure_group(&container);
        let mut table_content_state = TableContentState::new(table_content_token);
        table_content_state.tbody_data = vec![
            vec![
                StringBinder::new_with_str("行動順"),
                StringBinder::new_with_str("後攻"),
            ],
            vec![
                StringBinder::new_with_str("HP/MHP"),
                StringBinder::new_with_str("50/50"),
            ],
            vec![
                StringBinder::new_with_str("被ダメ"),
                StringBinder::new_with_str("5"),
            ],
        ];
        table_content_state.tbody_column_styles = vec![
            ColumnStyle {
                defs_id: "".to_string(),
                text_anchor_type: TextAnchorType::Start,
                x: 5.0,
                font_size: 20.0,
                first_y: 25.0,
                dy: 25.0,
            },
            ColumnStyle {
                defs_id: "".to_string(),
                text_anchor_type: TextAnchorType::End,
                x: 205.0,
                font_size: 20.0,
                first_y: 25.0,
                dy: 25.0,
            },
        ];
        Figure::default_window(
            x,
            y,
            RectLength::new_with_min(200.0, 150.0),
            RectLength::new_with_min(300.0, 30.0),
            frame_color,
            5.0,
            25.0,
            PartType::TableContent(table_content_state),
            element_manager,
            group_index,
        )
    }
    pub(crate) fn default_window(
        start_x: f64,
        start_y: f64,
        width: RectLength,
        height: RectLength,
        frame_color: &str,
        margin: f64,
        title_height: f64,
        content_part_type: PartType,
        element_manager: &mut ElementManager,
        group_index: usize,
    ) -> Figure {
        let offset_x = 0.0;
        let offset_y = title_height;
        let scroll_bar_thickness = 10.0;
        let button_size = 20.0;
        let min_width = width.min;
        let min_height = height.min;
        // TODO
        // clone しなくてよくはならないか
        let group_element = &mut element_manager.figure_groups[group_index].clone();
        Figure {
            base_rect: BaseRect {
                x_amount: Amount::new(start_x),
                y_amount: Amount::new(start_y),
                width,
                height,
                color: frame_color.to_string(),
                element_index: element_manager
                    .create_element_with_defs_id(&group_element, "def-default-window-base"),
                is_grabbed: false,
                x_fixed: false,
                y_fixed: false,
                // TODO
                // 現在は base_rect は全て Expand で記述されているが、そうとも限らない
                // 他の要件が明らかになったところで実装を見直す
                part_type: PartType::Expand,
            },
            parts: vec![
                PartRect::default_scrollable(
                    margin,
                    offset_x,
                    offset_y,
                    scroll_bar_thickness,
                    "white",
                    content_part_type,
                    element_manager,
                    group_element,
                ),
                PartRect::default_title_bg(
                    margin,
                    offset_y,
                    frame_color,
                    element_manager.create_element_with_defs_id(
                        &group_element,
                        "def-default-window-title-background",
                    ),
                ),
                PartRect::default_button(
                    (-margin - button_size - margin - button_size, End),
                    (margin, Start),
                    button_size,
                    "white",
                    element_manager.create_element_with_group(&group_element),
                    element_manager,
                    ButtonType::Minimize(MinimizeOption {
                        minimized_width: min_width,
                        minimized_height: min_height,
                    }),
                ),
                PartRect::default_button(
                    (-margin - button_size, End),
                    (margin, Start),
                    button_size,
                    "white",
                    element_manager.create_element_with_group(&group_element),
                    element_manager,
                    ButtonType::ShowContent(ShowContentOption {}),
                ),
            ],
            is_grabbed: false,
            is_pushed: false,
            group_index,
        }
    }

    pub(crate) fn update_base(&mut self) {
        if !self.is_grabbed {
            return;
        }
        self.base_rect.update_base();
        self.base_rect.is_grabbed = false;
        self.is_grabbed = false;
        for parts in self.parts.iter_mut() {
            parts.update_base();
        }
    }
    pub(crate) fn grab(&mut self, x: f64, y: f64) -> bool {
        // 自分を更新しながら part_rect.is_inner で base_rect を参照しているので clone 不可避
        let clone = self.base_rect.clone();
        // TODO
        // 重なり順で最後の要素を掴みたい（プッシュしたい）と思っているので filter().last() を使用しているけど妥当か
        // reverse() 的なことをするのと基本的には同じである
        if let Some(found_parts) = self
            .parts
            .iter_mut()
            .filter(|parts| {
                let result = parts.is_inner(x, y, &clone);
                result
            })
            .last()
        {
            self.is_grabbed = found_parts.grab(x, y, &clone);
            self.is_pushed = found_parts.is_pushed;
        } else {
            self.base_rect.is_grabbed = true;
            self.is_grabbed = true
        }
        self.is_grabbed
    }

    pub(crate) fn move_xy(
        &mut self,
        raw_drag_start_point: &Point,
        delta_point: &Point,
        element_manager: &ElementManager,
    ) {
        if self.is_grabbed {
            let drag_start_point = &Point {
                x: raw_drag_start_point.x - self.base_rect.x_amount.base,
                y: raw_drag_start_point.y - self.base_rect.y_amount.base,
            };
            if let Some(parts) = self.parts.iter_mut().find(|parts| parts.is_grabbed) {
                let parent_width = parts.width_value(&self.base_rect);
                let parent_height = parts.height_value(&self.base_rect);
                if let PartType::Drag = parts.part_type {
                    self.base_rect
                        .move_xy(&drag_start_point, &delta_point, true);
                } else if let PartType::Scrollable = parts.part_type {
                    if let Some(internal) = parts
                        .internal_part_rect
                        .iter_mut()
                        .find(|internal| internal.is_grabbed)
                    {
                        match &mut internal.part_type {
                            PartType::ScrollBarX(scroll_bar_state) => {
                                scroll_bar_state.start_amount.delta = delta_point.x;
                                let full_delta = parent_width - scroll_bar_state.length;
                                scroll_bar_state.percentage =
                                    scroll_bar_state.start_amount.value() / full_delta;
                            }
                            PartType::ScrollBarY(scroll_bar_state) => {
                                scroll_bar_state.start_amount.delta = delta_point.y;
                                let full_delta = parent_height - scroll_bar_state.length;
                                scroll_bar_state.percentage =
                                    scroll_bar_state.start_amount.value() / full_delta;
                            }
                            _ => {}
                        }
                    }
                }
            } else {
                self.base_rect
                    .move_xy(&drag_start_point, &delta_point, false);
            }
            // スクロールバーを触っていない状態でも、スクロールバーはスタート位置と長さの再計算が必要
            for parts in self.parts.iter_mut() {
                if let PartType::Scrollable = parts.part_type {
                    let mut internal_max_width: f64 = 0.0;
                    let mut internal_max_height: f64 = 0.0;
                    let scrollable_width = parts.width_value(&self.base_rect);
                    let scrollable_height = parts.height_value(&self.base_rect);
                    for internal in parts.internal_part_rect.iter() {
                        match internal.part_type {
                            PartType::ScrollBarX(..) | PartType::ScrollBarY(..) => {}
                            _ => {
                                internal_max_width =
                                    internal_max_width.max(internal.width_value(&self.base_rect));
                                internal_max_height =
                                    internal_max_height.max(internal.height_value(&self.base_rect));
                                if let Some(sibling) = element_manager.elements
                                    [internal.element_index]
                                    .next_element_sibling()
                                {
                                    if sibling.tag_name() == "g" {
                                        internal_max_width = internal_max_width.max(
                                            sibling.get_bounding_client_rect().width()
                                                / element_manager.scale
                                                + 10.0,
                                        );
                                        internal_max_height = internal_max_height.max(
                                            sibling.get_bounding_client_rect().height()
                                                / element_manager.scale
                                                + 10.0,
                                        );
                                    }
                                }
                            }
                        }
                    }
                    for internal in parts.internal_part_rect.iter_mut() {
                        match &mut internal.part_type {
                            PartType::ScrollBarX(scroll_bar_state) => {
                                scroll_bar_state.update(internal_max_width, scrollable_width);
                            }
                            PartType::ScrollBarY(scroll_bar_state) => {
                                scroll_bar_state.update(internal_max_height, scrollable_height);
                            }
                            _ => {}
                        }
                    }
                }
            }
        } else if self.is_pushed {
            if let Some(found_parts) = self.parts.iter_mut().find(|parts| parts.is_pushed) {
                if !found_parts.is_inner(
                    raw_drag_start_point.x + delta_point.x,
                    raw_drag_start_point.y + delta_point.y,
                    &self.base_rect,
                ) {
                    self.is_pushed = false;
                    found_parts.is_pushed = false;
                }
            } else {
                self.is_pushed = false;
            }
        }
    }

    pub(crate) fn is_inner(&self, raw_x: f64, raw_y: f64) -> bool {
        let x = raw_x - self.base_rect.x_amount.value();
        let y = raw_y - self.base_rect.y_amount.value();
        // base_rect からはみ出している PartRect がない前提の実装
        let x_value = self.base_rect.x_value();
        // マウスポインターの形が変わっても領域外だったりするので
        // スケール変更も考えて外側に 2px は拡大して内部判定を許容
        if x_value > x + 2.0 {
            false
        } else if x_value + self.base_rect.width_value() < x - 2.0 {
            false
        } else {
            let y_value = self.base_rect.y_value();
            if y_value > y + 2.0 {
                false
            } else if y_value + self.base_rect.height_value() < y - 2.0 {
                false
            } else {
                true
            }
        }
    }
}

pub(crate) struct TemporaryState {}
