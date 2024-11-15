mod utils;

use crate::AmountPositionType::{ContentBase, End, Ignore, Start};
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::console_log;
use web_sys::{window, Document, Element};

#[derive(Clone)]
struct Length {
    min: f64,
    max: f64,
    default: f64,
    amount: Amount,
    is_fixed: bool,
}

impl Length {
    pub fn value(&self) -> f64 {
        self.amount.value().max(self.min)
    }
    pub fn fix(&mut self) {
        self.amount.base = (self.amount.base + self.amount.delta).max(self.min);
        self.amount.delta = 0.0;
    }
    pub fn delta_constraint(&mut self) {
        if self.amount.base + self.amount.delta < self.min {
            self.amount.delta = self.min - self.amount.base;
        }
    }
}
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    pub fn new() -> Point {
        Point { x: 0.0, y: 0.0 }
    }
}

#[derive(Clone, Debug)]
struct Amount {
    base: f64,
    delta: f64,
}

impl Amount {
    pub fn new(value: f64) -> Amount {
        Amount {
            delta: 0.0,
            base: value,
        }
    }
    pub fn update_base(&mut self) {
        self.base += self.delta;
        self.delta = 0.0;
    }
    pub fn value(&self) -> f64 {
        self.base + self.delta
    }
}

#[derive(Clone)]
struct BaseRect {
    x_amounts: Vec<Amount>,
    y_amounts: Vec<Amount>,
    width: Length,
    height: Length,
    color: String,
    part_type: PartType,
    is_grabbed: bool,
    x_fixed: bool,
    y_fixed: bool,
    element_index: usize,
}

impl BaseRect {
    fn x_value(&self) -> f64 {
        self.x_amounts[0].value()
    }
    fn y_value(&self) -> f64 {
        self.y_amounts[0].value()
    }
    fn width_value(&self) -> f64 {
        self.width.value()
    }
    fn height_value(&self) -> f64 {
        self.height.value()
    }
    fn near_point_x(&self, x: f64) -> usize {
        if self.x_amounts[0].delta != 0.0 {
            0
        } else if self.x_amounts[1].delta != 0.0 {
            1
        } else if (x - self.x_amounts[0].value()).abs() < (x - self.x_amounts[1].value()).abs() {
            0
        } else {
            1
        }
    }
    fn near_point_y(&self, y: f64) -> usize {
        if self.y_amounts[0].delta != 0.0 {
            0
        } else if self.y_amounts[1].delta != 0.0 {
            1
        } else if (y - self.y_amounts[0].value()).abs() < (y - self.y_amounts[1].value()).abs() {
            0
        } else {
            1
        }
    }
    fn move_x(&mut self, start_x: f64, delta_x: f64, always_fixed: bool) {
        if self.width.is_fixed || always_fixed {
            self.x_amounts[0].delta = delta_x;
            self.x_amounts[1].delta = delta_x;
            // TODO
            // constraint の処理をまとめる
            if (self.x_amounts[0].base + delta_x).is_sign_negative() {
                self.x_amounts[0].delta = -self.x_amounts[0].base;
                self.x_amounts[1].delta = -self.x_amounts[0].base;
            }
        } else {
            let index = self.near_point_x(start_x);
            if index == 0 {
                self.width.amount.delta = -delta_x;
                self.width.delta_constraint();
                if (self.x_amounts[index].base - self.width.amount.delta).is_sign_negative() {
                    self.width.amount.delta = self.x_amounts[index].base;
                }
                self.x_amounts[index].delta = -self.width.amount.delta;
            } else {
                self.width.amount.delta = delta_x;
                self.width.delta_constraint();
                self.x_amounts[index].delta = self.width.amount.delta;
            }
        }
    }
    fn move_y(&mut self, start_y: f64, delta_y: f64, always_fixed: bool) {
        if self.height.is_fixed || always_fixed {
            self.y_amounts[0].delta = delta_y;
            self.y_amounts[1].delta = delta_y;
            if (self.y_amounts[0].base + delta_y).is_sign_negative() {
                self.y_amounts[0].delta = -self.y_amounts[0].base;
                self.y_amounts[1].delta = -self.y_amounts[0].base;
            }
        } else {
            let index = self.near_point_y(start_y);
            if index == 0 {
                self.height.amount.delta = -delta_y;
                self.height.delta_constraint();
                if (self.y_amounts[index].base - self.height.amount.delta).is_sign_negative() {
                    self.height.amount.delta = self.y_amounts[index].base;
                }
                self.y_amounts[index].delta = -self.height.amount.delta;
            } else {
                self.height.amount.delta = delta_y;
                self.height.delta_constraint();
                self.y_amounts[index].delta = self.height.amount.delta;
            }
        }
    }
    fn move_xy(&mut self, start_point: &Point, delta_point: &Point, always_fixed: bool) {
        if !self.x_fixed {
            self.move_x(start_point.x, delta_point.x, always_fixed);
        }
        if !self.y_fixed {
            self.move_y(start_point.y, delta_point.y, always_fixed);
        }
    }

    fn update_base(&mut self) {
        for amount in self.x_amounts.iter_mut() {
            amount.update_base();
        }
        for amount in self.y_amounts.iter_mut() {
            amount.update_base();
        }
        self.width.fix();
        self.height.fix();
    }

    fn adjust(&self, element_manager: &ElementManager) {
        let element = &element_manager.elements[self.element_index];
        if !self.color.is_empty() {
            element.set_attribute("fill", self.color.as_str()).unwrap();
        }
        element
            .set_attribute("x", self.x_value().to_string().as_str())
            .unwrap();
        element
            .set_attribute("y", self.y_value().to_string().as_str())
            .unwrap();
        element
            .set_attribute("width", self.width_value().to_string().as_str())
            .unwrap();
        element
            .set_attribute("height", self.height_value().to_string().as_str())
            .unwrap();
    }
}

struct PartRect {
    x_amounts: Vec<(f64, AmountPositionType)>,
    y_amounts: Vec<(f64, AmountPositionType)>,
    color: String,
    element_index: usize,
    part_type: PartType,
    is_grabbed: bool,
    internal_part_rect: Vec<PartRect>,
}

impl PartRect {
    fn update_base(&mut self) {
        if !self.is_grabbed {
            return;
        }
        self.is_grabbed = false;
        for internal in self.internal_part_rect.iter_mut() {
            internal.is_grabbed = false;
            match &mut internal.part_type {
                PartType::ScrollBarX(scroll_bar_state, ..) => {
                    scroll_bar_state.start_amount.update_base();
                }
                PartType::ScrollBarY(scroll_bar_state, ..) => {
                    scroll_bar_state.start_amount.update_base();
                }
                _ => {}
            }
        }
    }
    fn grab(&mut self, x: f64, y: f64, base_rect: &BaseRect) -> bool {
        self.is_grabbed = match self.part_type {
            PartType::Ignore
            | PartType::ClipPath
            // ScrollBarX, ScrollBarY は、直接は grab できず、
            // Scrollable の internal_pert_rect でのみ grab できる
            | PartType::ScrollBarX(..)
            | PartType::ScrollBarY(..)
            | PartType::TableContent(..) => false,
            PartType::Scrollable => {
                if let Some(internal) =
                    self.internal_part_rect
                        .iter_mut()
                        .find(|part| match &part.part_type {
                            // Scrollable 内の挙動を決定する分岐
                            PartType::ScrollBarX(_) => {
                                part.is_inner(x, y, base_rect)
                            }
                             PartType::ScrollBarY(_) => {
                                part.is_inner(x, y, base_rect)
                            }
                            _ => false,
                        })
                {
                    internal.is_grabbed = true;
                    true
                } else {
                    // Scrollable 内の特定の要素でなければ grab させない
                    false
                }
            }
            PartType::Expand | PartType::Drag => {
                true
            }
        };
        self.is_grabbed
    }
    fn adjust(&self, base_rect: &BaseRect, element_manager: &ElementManager) {
        let element = &element_manager.elements[self.element_index];
        if !self.color.is_empty() {
            element.set_attribute("fill", self.color.as_str()).unwrap();
        }
        element
            .set_attribute("x", self.x_value(base_rect).to_string().as_str())
            .unwrap();
        element
            .set_attribute("y", self.y_value(base_rect).to_string().as_str())
            .unwrap();
        element
            .set_attribute("width", self.width_value(base_rect).to_string().as_str())
            .unwrap();
        element
            .set_attribute("height", self.height_value(base_rect).to_string().as_str())
            .unwrap();
        let mut internal_max_width: f64 = 0.0;
        let mut internal_max_height: f64 = 0.0;
        let mut internal_max_width_element_index: usize = 0;
        let mut internal_max_height_element_index: usize = 0;
        for internal in self.internal_part_rect.iter() {
            internal.adjust(base_rect, element_manager);
            match internal.part_type {
                PartType::ScrollBarX(..) | PartType::ScrollBarY(..) | PartType::ClipPath => {}
                _ => {
                    let mut sibling_g_width = 0.0;
                    let mut sibling_g_height = 0.0;
                    if let Some(sibling) =
                        element_manager.elements[internal.element_index].next_element_sibling()
                    {
                        if sibling.tag_name() == "g" {
                            sibling_g_width = sibling.get_bounding_client_rect().width()
                                / element_manager.scale
                                + 10.0;
                            sibling_g_height = sibling.get_bounding_client_rect().height()
                                / element_manager.scale
                                + 10.0;
                        }
                    };

                    if internal_max_width < internal.width_value(base_rect)
                        || internal_max_width < sibling_g_width
                    {
                        internal_max_width = internal.width_value(base_rect).max(sibling_g_width);
                        internal_max_width_element_index = internal.element_index;
                    }
                    if internal_max_height < internal.height_value(base_rect)
                        || internal_max_height < sibling_g_height
                    {
                        internal_max_height =
                            internal.height_value(base_rect).max(sibling_g_height);
                        internal_max_height_element_index = internal.element_index;
                    }
                }
            }
        }
        // TODO
        // 2値のハードコーディングをやめる
        let scroll_bar_x_height = 10.0;
        let scroll_bar_y_width = 10.0;
        let base_width = self.width_value(base_rect);
        let base_height = self.height_value(base_rect);
        let width_ratio = internal_max_width / base_width;
        let width_ratio_with_bar = (internal_max_width + scroll_bar_y_width) / base_width;
        let height_ratio = internal_max_height / base_height;
        let height_ratio_with_bar = (internal_max_height + scroll_bar_x_height) / base_height;
        let threshold = 1.02;
        if let PartType::Scrollable = self.part_type {
            let mut table_content_x = 0.0;
            let mut table_content_y = 0.0;
            if width_ratio_with_bar <= threshold
                || (width_ratio <= threshold && height_ratio <= threshold)
            {
                for internal in self.internal_part_rect.iter() {
                    if let PartType::ScrollBarX(..) = internal.part_type {
                        internal.hide(element_manager);
                    }
                }
            } else {
                for internal in self.internal_part_rect.iter() {
                    if let PartType::ScrollBarX(scroll_bar_state) = &internal.part_type {
                        let using_width_ratio = if height_ratio > threshold {
                            width_ratio_with_bar
                        } else {
                            width_ratio
                        };
                        // let bar_width = self.width_value(base_rect) / using_width_ratio;
                        let bar_width = scroll_bar_state.length;
                        let max_delta = base_width - bar_width;
                        let offset_max_width = internal_max_width - base_width;
                        let offset_x =
                            scroll_bar_state.start_amount.value() / max_delta * offset_max_width;
                        let content = &element_manager.elements[internal_max_width_element_index];
                        let content_x: f64 = content.get_attribute("x").unwrap().parse().unwrap();
                        content
                            .set_attribute("x", (content_x - offset_x).to_string().as_str())
                            .unwrap();
                        table_content_x -= offset_x;
                    }
                }
            }
            if height_ratio_with_bar <= threshold
                || (width_ratio <= threshold && height_ratio <= threshold)
            {
                for internal in self.internal_part_rect.iter() {
                    if let PartType::ScrollBarY(..) = internal.part_type {
                        internal.hide(element_manager);
                    }
                }
            } else {
                for internal in self.internal_part_rect.iter() {
                    if let PartType::ScrollBarY(scroll_bar_state) = &internal.part_type {
                        let using_height_ratio = if width_ratio > threshold {
                            height_ratio_with_bar
                        } else {
                            height_ratio
                        };
                        // let bar_height = self.height_value(base_rect) / using_height_ratio;
                        let bar_height = scroll_bar_state.length;
                        let max_delta = base_height - bar_height;
                        let offset_max_height = internal_max_height - base_height;
                        let offset_y =
                            scroll_bar_state.start_amount.value() / max_delta * offset_max_height;
                        let content = &element_manager.elements[internal_max_height_element_index];
                        let content_y: f64 = content.get_attribute("y").unwrap().parse().unwrap();
                        content
                            .set_attribute("y", (content_y - offset_y).to_string().as_str())
                            .unwrap();
                        table_content_y -= offset_y;
                    }
                }
            }
            for internal in self.internal_part_rect.iter() {
                if let PartType::TableContent(table_content_state) = &internal.part_type {
                    let sibling_group = element_manager.elements[internal.element_index]
                        .next_element_sibling()
                        .unwrap();
                    sibling_group.set_inner_html(format!("<clipPath id='clip-path-test'><rect fill='white' x='{}' y='{}' width='{}' height='{}'></rect></clipPath>", -table_content_x, -table_content_y, self.width_value(base_rect), self.height_value(base_rect)).as_str());
                    table_content_state.init(element_manager, &sibling_group);
                    table_content_x += self.x_value(base_rect);
                    table_content_y += self.y_value(base_rect);
                    sibling_group
                        .set_attribute(
                            "transform",
                            format!("translate({}, {})", table_content_x, table_content_y).as_str(),
                        )
                        .unwrap();
                    sibling_group
                        .set_attribute("clip-path", "url(#clip-path-test)")
                        .unwrap();
                }
            }
        }
    }
    fn hide(&self, element_manager: &ElementManager) {
        let element = &element_manager.elements[self.element_index];
        element.set_attribute("width", "0").unwrap();
    }
}

enum AmountPositionType {
    Start,
    End,
    ContentBase,
    Ignore,
}

#[derive(Clone, Debug)]
enum PartType {
    Ignore,
    Expand,
    Drag,
    Scrollable,
    ScrollBarX(ScrollBarState),
    ScrollBarY(ScrollBarState),
    TableContent(TableContentState),
    ClipPath,
}

impl PartRect {
    pub fn is_inner(&self, x: f64, y: f64, base_rect: &BaseRect) -> bool {
        let x_value = self.x_value(base_rect);
        if x_value > x {
            return false;
        } else if x_value + self.width_value(base_rect) < x {
            return false;
        }
        let y_value = self.y_value(base_rect);
        if y_value > y {
            return false;
        } else if y_value + self.height_value(base_rect) < y {
            return false;
        }
        true
    }

    fn x_value(&self, base_rect: &BaseRect) -> f64 {
        let (ref amount, ref amount_position_type) = self.x_amounts[0];
        let mut amount = match amount_position_type {
            Start | ContentBase => base_rect.x_value() + amount,
            End => base_rect.x_value() + base_rect.width_value() + amount,
            Ignore => 0.0,
        };
        match &self.part_type {
            PartType::ScrollBarX(scroll_bar_state) => {
                amount += scroll_bar_state.start_amount.value()
            }
            _ => {}
        }
        amount
    }
    fn y_value(&self, base_rect: &BaseRect) -> f64 {
        let (ref amount, ref amount_position_type) = self.y_amounts[0];
        let mut amount = match amount_position_type {
            Start | ContentBase => base_rect.y_value() + amount,
            End => base_rect.y_value() + base_rect.height_value() + amount,
            Ignore => 0.0,
        };
        match &self.part_type {
            PartType::ScrollBarY(scroll_bar_state) => {
                amount += scroll_bar_state.start_amount.value()
            }
            _ => {}
        }
        amount
    }
    fn width_value(&self, base_rect: &BaseRect) -> f64 {
        let (ref amount, ref amount_position_type) = self.x_amounts[1];
        if let PartType::ScrollBarX(scroll_bar_state) = &self.part_type {
            return scroll_bar_state.length;
        }
        match amount_position_type {
            Start | ContentBase => base_rect.x_value() + amount - self.x_value(base_rect),
            End => base_rect.x_value() + base_rect.width_value() + amount - self.x_value(base_rect),
            Ignore => 0.0,
        }
    }
    fn height_value(&self, base_rect: &BaseRect) -> f64 {
        let (ref amount, ref amount_position_type) = self.y_amounts[1];
        if let PartType::ScrollBarY(scroll_bar_state) = &self.part_type {
            return scroll_bar_state.length;
        }
        match amount_position_type {
            Start | ContentBase => base_rect.y_value() + amount - self.y_value(base_rect),
            End => {
                base_rect.y_value() + base_rect.height_value() + amount - self.y_value(base_rect)
            }
            Ignore => 0.0,
        }
    }
}

#[derive(Clone, Debug)]
struct TableContentState {
    thead_data: Vec<StringBinder>,
    thead_column_styles: Vec<ColumnStyle>,
    tbody_data: Vec<Vec<StringBinder>>,
    tbody_column_styles: Vec<ColumnStyle>,
    content_id_token: String,
}

impl TableContentState {
    fn new() -> TableContentState {
        TableContentState {
            thead_data: vec![],
            thead_column_styles: vec![],
            tbody_data: vec![],
            tbody_column_styles: vec![],
            content_id_token: "".to_string(),
        }
    }
    fn init(&self, element_manager: &ElementManager, table_container: &Element) {
        // TODO
        // StringBinderから値を取り出す
        // thead が空でない場合は thead の値を計算
        if !self.thead_data.is_empty() {
            // check_and_update
        }
        // tbody が空でない場合は tbody の値を計算
        if !self.tbody_data.is_empty() {
            let mut tbody_column_elements = vec![];
            for n in 0..self.tbody_data.len() {
                for m in 0..self.tbody_data[n].len() {
                    // TODO
                    // adjust の中では mut にできない
                    // self.tbody_data[n][m].check_and_update_value(&State {});
                }
                let tbody_column_id = format!("{}-tbody-col-{}", self.content_id_token, n);
                if let Some(tbody_col) = element_manager
                    .document
                    .get_element_by_id(tbody_column_id.as_str())
                {
                    tbody_col.remove();
                }
                // text要素を column の数だけ生成
                let tbody_column = element_manager
                    .document
                    .create_element_ns(Option::from("http://www.w3.org/2000/svg"), "text")
                    .unwrap();
                tbody_column.set_id(tbody_column_id.as_str());
                table_container.append_child(&*tbody_column).unwrap();
                tbody_column_elements.push(tbody_column);
            }
            // 各text要素にtspanを追加
            for n in 0..self.tbody_data.len() {
                for m in 0..self.tbody_data[n].len() {
                    let tbody_tspan = element_manager
                        .document
                        .create_element_ns(Option::from("http://www.w3.org/2000/svg"), "tspan")
                        .unwrap();
                    let value = &self.tbody_data[n][m].current_value;
                    tbody_tspan.set_inner_html(value.as_str());
                    let style = &self.tbody_column_styles[m];
                    if style.first_y != 0.0 {
                        tbody_tspan
                            .set_attribute("y", style.first_y.to_string().as_str())
                            .unwrap();
                    }
                    if let TextAnchorType::End = style.text_anchor_type {
                        tbody_tspan.set_attribute("text-anchor", "end").unwrap();
                    }
                    tbody_tspan
                        .set_attribute("x", style.x.to_string().as_str())
                        .unwrap();
                    tbody_tspan
                        .set_attribute("dy", (style.dy * n as f64).to_string().as_str())
                        .unwrap();
                    tbody_tspan
                        .set_attribute("font-size", style.font_size.to_string().as_str())
                        .unwrap();
                    tbody_column_elements[m]
                        .append_child(&*tbody_tspan)
                        .unwrap();
                }
            }
        }
    }
    fn update(&self) {}
}
#[derive(Clone, Debug)]
struct ScrollBarState {
    start_amount: Amount,
    percentage: f64,
    length: f64,
}

impl ScrollBarState {
    fn new() -> ScrollBarState {
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

struct MouseState {
    is_dragged: bool,
    drag_start_point: Point,
}

impl MouseState {
    fn new() -> MouseState {
        MouseState {
            is_dragged: false,
            drag_start_point: Point::new(),
        }
    }
}

struct Figure {
    base_rect: BaseRect,
    parts: Vec<PartRect>,
    is_grabbed: bool,
}

impl Figure {
    pub fn update_base(&mut self) {
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
    pub fn grab(&mut self, x: f64, y: f64) -> bool {
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

    pub fn move_xy(
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

    pub fn is_inner(&self, x: f64, y: f64) -> bool {
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

struct ElementManager {
    document: Document,
    elements: Vec<Element>,
    clip_paths: Vec<Element>,
    offset_x: f64,
    offset_y: f64,
    scale: f64,
}

impl ElementManager {
    fn new() -> ElementManager {
        ElementManager {
            document: window().unwrap().document().unwrap(),
            elements: vec![],
            clip_paths: vec![],
            offset_x: 0.0,
            offset_y: 0.0,
            scale: 1.0,
        }
    }
    fn create_element(&mut self, container: &Element) -> usize {
        let rect = self
            .document
            .create_element_ns(Option::from("http://www.w3.org/2000/svg"), "rect")
            .unwrap();
        container.append_child(&*rect).unwrap();
        self.elements.push(rect);
        self.elements.len() - 1
    }

    fn create_element_with_html(&mut self, container: &Element, html: &str) -> usize {
        let g = self
            .document
            .create_element_ns(Option::from("http://www.w3.org/2000/svg"), "g")
            .unwrap();
        g.set_inner_html(html);
        let rect = g.first_element_child().unwrap();
        container.append_child(&*rect).unwrap();
        self.elements.push(rect);
        self.elements.len() - 1
    }

    fn create_element_with_defs_id(&mut self, container: &Element, id: &str) -> usize {
        let element = self.document.get_element_by_id(id).unwrap().clone();
        element.set_id("");
        container.append_child(&*element).unwrap();
        self.elements.push(element);
        self.elements.len() - 1
    }
    fn create_element_with_clip_path(
        &mut self,
        container: &Element,
        clip_path_index: usize,
    ) -> usize {
        let rect = self
            .document
            .create_element_ns(Option::from("http://www.w3.org/2000/svg"), "rect")
            .unwrap();
        rect.set_attribute(
            "clip-path",
            format!("url(#clip-path-{})", clip_path_index).as_str(),
        )
        .unwrap();
        container.append_child(&*rect).unwrap();
        let g = self
            .document
            .create_element_ns(Option::from("http://www.w3.org/2000/svg"), "g")
            .unwrap();
        container.append_child(&*g).unwrap();
        self.elements.push(rect);
        self.elements.len() - 1
    }

    fn create_clip_path(&mut self, document: &Document, container: &Element) -> (usize, usize) {
        let clip_path = self
            .document
            .create_element_ns(Option::from("http://www.w3.org/2000/svg"), "clipPath")
            .unwrap();
        clip_path.set_id(format!("clip-path-{}", self.clip_paths.len()).as_str());
        let rect = document
            .create_element_ns(Option::from("http://www.w3.org/2000/svg"), "rect")
            .unwrap();
        clip_path.append_child(&*rect).unwrap();
        container.append_child(&*clip_path).unwrap();
        self.elements.push(rect);
        self.clip_paths.push(clip_path);
        (self.clip_paths.len() - 1, self.elements.len() - 1)
    }
    fn get_internal_xy(&self, x: f64, y: f64) -> (f64, f64) {
        (
            (x - self.offset_x) / self.scale,
            (y - self.offset_y) / self.scale,
        )
    }
}

struct TemporaryState {}

#[derive(Clone, Debug)]
struct StringBinder {
    value_func: fn(&StringBinder, &TemporaryState) -> String,
    args_string: Vec<String>,
    current_value: String,
}

impl StringBinder {
    fn args_string_func() -> fn(&StringBinder, &TemporaryState) -> String {
        fn get_args_string(string_binder: &StringBinder, _: &TemporaryState) -> String {
            if let Some(arg) = string_binder.args_string.get(0) {
                arg.to_owned()
            } else {
                "".to_string()
            }
        }
        get_args_string
    }
    fn new_with_str(arg: &str) -> StringBinder {
        StringBinder {
            value_func: StringBinder::args_string_func(),
            args_string: vec![arg.to_string()],
            current_value: arg.to_string(),
        }
    }
    fn get_value(&self, state: &TemporaryState) -> String {
        let value_func = self.value_func;
        value_func(self, state)
    }

    fn check_and_update_value(&mut self, state: &TemporaryState) -> bool {
        let new_value = self.get_value(state);
        if self.current_value != new_value {
            self.current_value = new_value;
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Debug)]
enum TextAnchorType {
    Start,
    End,
    Middle,
}

#[derive(Clone, Debug)]
struct ColumnStyle {
    // 以下の要素以外の装飾をしたい時に defs を使う
    // 使わない時はブランクでOK
    defs_id: String,
    text_anchor_type: TextAnchorType,
    x: f64,
    font_size: f64,
    first_y: f64,
    dy: f64,
}

impl ColumnStyle {
    fn get_element_by_defs_id(&self, document: &Document) -> Element {
        let element = document
            .get_element_by_id(self.defs_id.as_str())
            .unwrap()
            .clone();
        element.set_id("");
        element
    }
}
#[wasm_bindgen]
pub struct Binder {
    figures: Vec<Figure>,
    mouse_state: MouseState,
    element_manager: ElementManager,
    pub has_update: bool,
}

#[wasm_bindgen]
impl Binder {
    pub fn new() -> Binder {
        let mut element_manager = ElementManager::new();
        let binder = Binder {
            figures: vec![],
            mouse_state: MouseState::new(),
            element_manager,
            has_update: false,
        };
        binder.adjust();
        binder
    }
    #[wasm_bindgen(constructor)]
    pub fn new_for_dev() -> Binder {
        let mut element_manager = ElementManager::new();
        let document = window().unwrap().document().unwrap();
        let container = document.get_element_by_id("container").unwrap();
        let (clip_path_index_1, clip_path_element_index_1) =
            element_manager.create_clip_path(&document, &container);
        let mut table_content_state = TableContentState::new();
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
        let figure = Figure {
            base_rect: BaseRect {
                x_amounts: vec![Amount::new(100.0), Amount::new(300.0)],
                y_amounts: vec![Amount::new(100.0), Amount::new(400.0)],
                width: Length {
                    min: 80.0,
                    max: 0.0,
                    default: 0.0,
                    amount: Amount::new(200.0),
                    is_fixed: false,
                },
                height: Length {
                    min: 80.0,
                    max: 0.0,
                    default: 0.0,
                    amount: Amount::new(300.0),
                    is_fixed: false,
                },
                color: "".to_string(),
                element_index: element_manager
                    .create_element_with_defs_id(&container, "def-default-window-base"),
                part_type: PartType::Expand,
                is_grabbed: false,
                x_fixed: false,
                y_fixed: false,
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
                    x_amounts: vec![(5.0, Start), (-30.0, End)],
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
        };

        let binder = Binder {
            figures: vec![figure],
            mouse_state: MouseState::new(),
            element_manager,
            has_update: false,
        };
        binder.adjust();
        binder
    }
    pub fn adjust(&self) {
        for figure in self.figures.iter() {
            figure.base_rect.adjust(&self.element_manager);
            for part_rect in figure.parts.iter() {
                part_rect.adjust(&figure.base_rect, &self.element_manager);
            }
        }
    }

    pub fn fix(&mut self) {
        for figure in self.figures.iter_mut() {
            figure.update_base();
        }
    }

    pub fn update(&mut self) {
        if self.has_update {
            self.adjust();
            self.has_update = false;
        }
    }
    pub fn mouse_up(&mut self) {
        self.mouse_state.is_dragged = false;
        self.fix();
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
