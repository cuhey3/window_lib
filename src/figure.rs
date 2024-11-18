use crate::binder::element_manager::ElementManager;
use crate::content::{ColumnStyle, StringBinder, TableContentState, TextAnchorType};
use crate::figure::part_rect::PartRect;
use crate::figure::AmountPositionType::{End, Ignore, Start};
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
}

pub(crate) enum AmountPositionType {
    Start,
    End,
    ContentBase,
    Ignore,
}

#[derive(Clone, Debug)]
pub(crate) enum PartType {
    Ignore,
    Expand,
    Drag,
    Scrollable,
    ScrollBarX(ScrollBarState),
    ScrollBarY(ScrollBarState),
    TableContent(TableContentState),
    ClipPath,
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
    pub(crate) group_index: usize,
}

impl Figure {
    pub(crate) fn new_log_window_dev(element_manager: &mut ElementManager) -> Figure {
        let container = element_manager.get_container();
        let group_index = element_manager.create_figure_group(&container);
        let container = &mut element_manager.figure_groups[group_index].clone();
        let (clip_path_index_1, clip_path_element_index_1) =
            element_manager.create_clip_path(&container);
        let mut table_content_state = TableContentState::new("log");
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
        Figure {
            base_rect: BaseRect {
                x_amounts: vec![Amount::new(50.0), Amount::new(1050.0)],
                y_amounts: vec![Amount::new(700.0), Amount::new(790.0)],
                width: RectLength {
                    min: 120.0,
                    max: 0.0,
                    default: 0.0,
                    amount: Amount::new(1000.0),
                    is_fixed: false,
                },
                height: RectLength {
                    min: 60.0,
                    max: 0.0,
                    default: 0.0,
                    amount: Amount::new(90.0),
                    is_fixed: false,
                },
                color: "gray".to_string(),
                part_type: PartType::Expand,
                is_grabbed: false,
                x_fixed: false,
                y_fixed: false,
                element_index: element_manager
                    .create_element_with_defs_id(&container, "def-default-window-base"),
            },
            parts: vec![
                PartRect {
                    x_amounts: vec![(5.0, Start), (-5.0, End)],
                    y_amounts: vec![(30.0, Start), (-5.0, End)],
                    color: "white".to_string(),
                    element_index: element_manager.create_element(&container),
                    part_type: PartType::Ignore,
                    is_grabbed: false,
                    internal_part_rect: vec![],
                },
                PartRect {
                    x_amounts: vec![(5.0, Start), (-5.0, End)],
                    y_amounts: vec![(30.0, Start), (-5.0, End)],
                    color: "white".to_string(),
                    element_index: element_manager.create_element(&container),
                    part_type: PartType::Scrollable,
                    is_grabbed: false,
                    internal_part_rect: vec![
                        PartRect {
                            x_amounts: vec![(5.0, Start), (5.0, Start)],
                            y_amounts: vec![(30.0, Start), (30.0, Start)],
                            color: "orange".to_string(),
                            element_index: element_manager
                                .create_element_with_clip_path(&container, clip_path_index_1),
                            part_type: PartType::TableContent(table_content_state),
                            is_grabbed: false,
                            internal_part_rect: vec![],
                        },
                        PartRect {
                            x_amounts: vec![(5.0, Start), (0.0, Ignore)],
                            y_amounts: vec![(-15.0, End), (-5.0, End)],
                            color: "".to_string(),
                            element_index: element_manager.create_element_with_defs_id(
                                &container,
                                "def-default-scroll-bar-x",
                            ),
                            part_type: PartType::ScrollBarX(ScrollBarState::new()),
                            is_grabbed: false,
                            internal_part_rect: vec![],
                        },
                        PartRect {
                            x_amounts: vec![(-15.0, End), (-5.0, End)],
                            y_amounts: vec![(30.0, Start), (0.0, Ignore)],
                            color: "".to_string(),
                            element_index: element_manager.create_element_with_defs_id(
                                &container,
                                "def-default-scroll-bar-y",
                            ),
                            part_type: PartType::ScrollBarY(ScrollBarState::new()),
                            is_grabbed: false,
                            internal_part_rect: vec![],
                        },
                        PartRect {
                            x_amounts: vec![(5.0, Start), (-5.0, End)],
                            y_amounts: vec![(30.0, Start), (-5.0, End)],
                            color: "white".to_string(),
                            element_index: clip_path_element_index_1,
                            part_type: PartType::ClipPath,
                            is_grabbed: false,
                            internal_part_rect: vec![],
                        },
                    ],
                },
                PartRect {
                    x_amounts: vec![(5.0, Start), (-55.0, End)],
                    y_amounts: vec![(5.0, Start), (30.0, Start)],
                    color: "".to_string(),
                    element_index: element_manager.create_element_with_defs_id(
                        &container,
                        "def-default-window-title-background",
                    ),
                    part_type: PartType::Drag,
                    is_grabbed: false,
                    internal_part_rect: vec![],
                },
                PartRect {
                    x_amounts: vec![(-25.0, End), (-5.0, End)],
                    y_amounts: vec![(5.0, Start), (25.0, Start)],
                    color: "white".to_string(),
                    element_index: element_manager.create_element(&container),
                    part_type: PartType::Ignore,
                    is_grabbed: false,
                    internal_part_rect: vec![],
                },
                PartRect {
                    x_amounts: vec![(-50.0, End), (-30.0, End)],
                    y_amounts: vec![(5.0, Start), (25.0, Start)],
                    color: "white".to_string(),
                    element_index: element_manager.create_element(&container),
                    part_type: PartType::Ignore,
                    is_grabbed: false,
                    internal_part_rect: vec![],
                },
            ],
            is_grabbed: false,
            group_index,
        }
    }
    pub(crate) fn new_window_dev(element_manager: &mut ElementManager) -> Figure {
        let container = element_manager.get_container();
        let group_index = element_manager.create_figure_group(&container);
        // TODO
        // clone しなくてよくはならないか
        let container = &mut element_manager.figure_groups[group_index].clone();
        let (clip_path_index_1, clip_path_element_index_1) =
            element_manager.create_clip_path(&container);
        let mut table_content_state = TableContentState::new("status");
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
        Figure {
            base_rect: BaseRect {
                x_amounts: vec![Amount::new(100.0), Amount::new(300.0)],
                y_amounts: vec![Amount::new(100.0), Amount::new(400.0)],
                width: RectLength {
                    min: 80.0,
                    max: 0.0,
                    default: 0.0,
                    amount: Amount::new(200.0),
                    is_fixed: false,
                },
                height: RectLength {
                    min: 80.0,
                    max: 0.0,
                    default: 0.0,
                    amount: Amount::new(300.0),
                    is_fixed: false,
                },
                color: "".to_string(),
                element_index: element_manager
                    .create_element_with_defs_id(&container, "def-default-window-base"),
                is_grabbed: false,
                x_fixed: false,
                y_fixed: false,
                // TODO
                // 現在は base_rect は全て Expand で記述されているが、そうとも限らない
                // 他の要件が明らかになったところで実装を見直す
                part_type: PartType::Expand,
            },
            parts: vec![
                PartRect {
                    x_amounts: vec![(5.0, Start), (-5.0, End)],
                    y_amounts: vec![(30.0, Start), (-5.0, End)],
                    color: "white".to_string(),
                    element_index: element_manager.create_element(&container),
                    part_type: PartType::Scrollable,
                    is_grabbed: false,
                    internal_part_rect: vec![
                        PartRect {
                            x_amounts: vec![(5.0, Start), (5.0, Start)],
                            y_amounts: vec![(30.0, Start), (30.0, Start)],
                            color: "orange".to_string(),
                            element_index: element_manager
                                .create_element_with_clip_path(&container, clip_path_index_1),
                            part_type: PartType::TableContent(table_content_state),
                            is_grabbed: false,
                            internal_part_rect: vec![],
                        },
                        PartRect {
                            x_amounts: vec![(5.0, Start), (0.0, Ignore)],
                            y_amounts: vec![(-15.0, End), (-5.0, End)],
                            color: "".to_string(),
                            element_index: element_manager.create_element_with_defs_id(
                                &container,
                                "def-default-scroll-bar-x",
                            ),
                            part_type: PartType::ScrollBarX(ScrollBarState::new()),
                            is_grabbed: false,
                            internal_part_rect: vec![],
                        },
                        PartRect {
                            x_amounts: vec![(-15.0, End), (-5.0, End)],
                            y_amounts: vec![(30.0, Start), (0.0, Ignore)],
                            color: "".to_string(),
                            element_index: element_manager.create_element_with_defs_id(
                                &container,
                                "def-default-scroll-bar-y",
                            ),
                            part_type: PartType::ScrollBarY(ScrollBarState::new()),
                            is_grabbed: false,
                            internal_part_rect: vec![],
                        },
                        PartRect {
                            x_amounts: vec![(5.0, Start), (-5.0, End)],
                            y_amounts: vec![(30.0, Start), (-5.0, End)],
                            color: "white".to_string(),
                            element_index: clip_path_element_index_1,
                            part_type: PartType::ClipPath,
                            is_grabbed: false,
                            internal_part_rect: vec![],
                        },
                    ],
                },
                PartRect {
                    x_amounts: vec![(-25.0, End), (-5.0, End)],
                    y_amounts: vec![(5.0, Start), (25.0, Start)],
                    color: "white".to_string(),
                    element_index: element_manager.create_element(&container),
                    part_type: PartType::Ignore,
                    is_grabbed: false,
                    internal_part_rect: vec![],
                },
                PartRect {
                    x_amounts: vec![(-50.0, End), (-30.0, End)],
                    y_amounts: vec![(5.0, Start), (25.0, Start)],
                    color: "white".to_string(),
                    element_index: element_manager.create_element(&container),
                    part_type: PartType::Ignore,
                    is_grabbed: false,
                    internal_part_rect: vec![],
                },
                PartRect {
                    x_amounts: vec![(5.0, Start), (-55.0, End)],
                    y_amounts: vec![(5.0, Start), (30.0, Start)],
                    color: "".to_string(),
                    element_index: element_manager.create_element_with_defs_id(
                        &container,
                        "def-default-window-title-background",
                    ),
                    part_type: PartType::Drag,
                    is_grabbed: false,
                    internal_part_rect: vec![],
                },
            ],
            is_grabbed: false,
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
        if let Some(found_parts) = self
            .parts
            .iter_mut()
            .find(|parts| parts.is_inner(x, y, &clone))
        {
            self.is_grabbed = found_parts.grab(x, y, &clone)
        } else {
            self.base_rect.is_grabbed = true;
            self.is_grabbed = true
        }
        self.is_grabbed
    }

    pub(crate) fn move_xy(
        &mut self,
        drag_start_point: &Point,
        delta_point: &Point,
        element_manager: &ElementManager,
    ) {
        if self.is_grabbed {
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
                            PartType::ScrollBarX(..)
                            | PartType::ScrollBarY(..)
                            | PartType::ClipPath => {}
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
        }
    }

    pub(crate) fn is_inner(&self, x: f64, y: f64) -> bool {
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
