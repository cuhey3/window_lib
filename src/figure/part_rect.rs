use crate::binder::element_manager::ElementManager;
use crate::binder::ContentManager;
use crate::figure::base_rect::BaseRect;
use crate::figure::AmountPositionType::{ContentBase, End, Ignore, Start};
use crate::figure::{AmountPositionType, Figure, PartType, ScrollBarState, TitleState};
use crate::math::Amount;
use web_sys::Element;

pub(crate) struct PartRect {
    pub(crate) x_amounts: Vec<(f64, AmountPositionType)>,
    pub(crate) y_amounts: Vec<(f64, AmountPositionType)>,
    pub(crate) color: String,
    pub(crate) element_index: usize,
    pub(crate) part_type: PartType,
    pub(crate) is_grabbed: bool,
    pub(crate) is_pushed: bool,
    pub(crate) internal_part_rect: Vec<PartRect>,
    pub(crate) is_initialized: bool,
    pub(crate) has_content: bool,
}

impl PartRect {
    pub(crate) fn default_scrollable(
        margin: f64,
        offset_x: f64,
        offset_y: f64,
        thickness: f64,
        color: &str,
        content_part_type: PartType,
        element_manager: &mut ElementManager,
        group_element: &Element,
    ) -> PartRect {
        PartRect {
            x_amounts: vec![(margin + offset_x, Start), (-margin, End)],
            y_amounts: vec![(margin + offset_y, Start), (-margin, End)],
            color: color.to_string(),
            element_index: element_manager
                .create_element_with_defs_id(group_element, "def-default-scroll-area"),
            part_type: PartType::Scrollable,
            is_grabbed: false,
            is_pushed: false,
            internal_part_rect: vec![
                PartRect::default_content(
                    margin,
                    0.0,
                    offset_y,
                    content_part_type,
                    element_manager.create_element_with_group(group_element),
                ),
                PartRect::default_scroll_bar_xy(
                    thickness,
                    margin,
                    offset_x,
                    PartType::ScrollBarX(ScrollBarState::new()),
                    "",
                    element_manager
                        .create_element_with_defs_id(&group_element, "def-default-scroll-bar-x"),
                ),
                PartRect::default_scroll_bar_xy(
                    thickness,
                    margin,
                    offset_y,
                    PartType::ScrollBarY(ScrollBarState::new()),
                    "",
                    element_manager
                        .create_element_with_defs_id(&group_element, "def-default-scroll-bar-y"),
                ),
            ],
            is_initialized: false,
            has_content: false,
        }
    }
    pub(crate) fn default_content(
        margin: f64,
        offset_x: f64,
        offset_y: f64,
        part_type: PartType,
        element_index: usize,
    ) -> PartRect {
        PartRect {
            x_amounts: vec![(margin + offset_x, Start), (0.0, Ignore)],
            y_amounts: vec![(margin + offset_y, Start), (0.0, Ignore)],
            color: "".to_string(),
            element_index,
            part_type,
            is_grabbed: false,
            is_pushed: false,
            internal_part_rect: vec![],
            is_initialized: false,
            has_content: false,
        }
    }
    pub(crate) fn default_scroll_bar_xy(
        thickness: f64,
        margin: f64,
        offset: f64,
        part_type: PartType,
        color: &str,
        element_index: usize,
    ) -> PartRect {
        match part_type {
            PartType::ScrollBarX(..) => PartRect {
                x_amounts: vec![(margin + offset, Start), (0.0, Ignore)],
                y_amounts: vec![(-margin - thickness, End), (-margin, End)],
                color: color.to_string(),
                element_index,
                part_type,
                is_grabbed: false,
                is_pushed: false,
                internal_part_rect: vec![],
                is_initialized: false,
                has_content: false,
            },
            PartType::ScrollBarY(..) => PartRect {
                x_amounts: vec![(-margin - thickness, End), (-margin, End)],
                y_amounts: vec![(margin + offset, Start), (0.0, Ignore)],
                color: color.to_string(),
                element_index,
                part_type,
                is_grabbed: false,
                is_pushed: false,
                internal_part_rect: vec![],
                is_initialized: false,
                has_content: false,
            },
            _ => panic!(),
        }
    }
    pub(crate) fn default_title_bg(
        title: &str,
        margin: f64,
        bg_height: f64,
        color: &str,
        element_index: usize,
    ) -> PartRect {
        PartRect {
            x_amounts: vec![(margin, Start), (-margin, End)],
            y_amounts: vec![(margin, Start), (margin + bg_height, Start)],
            color: color.to_string(),
            element_index,
            part_type: PartType::Title(TitleState {
                title: title.to_string(),
            }),
            is_grabbed: false,
            is_pushed: false,
            internal_part_rect: vec![],
            is_initialized: false,
            has_content: false,
        }
    }
    pub(crate) fn default_button(
        x_amount: (f64, AmountPositionType),
        y_amount: (f64, AmountPositionType),
        size: f64,
        color: &str,
        element_index: usize,
        element_manager: &mut ElementManager,
        button_type: ButtonType,
    ) -> PartRect {
        match button_type {
            ButtonType::Minimize(..) => ButtonType::draw_button(
                element_manager,
                element_index,
                "def-default-minimize-button",
            ),
            ButtonType::ShowContent(..) => ButtonType::draw_button(
                element_manager,
                element_index,
                "def-default-show-content-button",
            ),
        }
        PartRect {
            x_amounts: vec![x_amount.clone(), (x_amount.0 + size, x_amount.1)],
            y_amounts: vec![y_amount.clone(), (y_amount.0 + size, y_amount.1)],
            color: color.to_string(),
            element_index,
            part_type: PartType::Button(button_type),
            is_grabbed: false,
            is_pushed: false,
            internal_part_rect: vec![],
            is_initialized: false,
            has_content: true,
        }
    }
    pub(crate) fn update_base(&mut self) {
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
    pub(crate) fn grab(&mut self, x: f64, y: f64, base_rect: &BaseRect) -> bool {
        self.is_grabbed = match self.part_type {
            PartType::Ignore
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
            PartType::Expand | PartType::Drag | PartType::Title(..) => {
                true
            }
            PartType::Button(_) => {
                self.is_pushed = true;
                false
            }
        };
        self.is_grabbed
    }
    pub(crate) fn adjust(
        &mut self,
        base_rect: &BaseRect,
        element_manager: &mut ElementManager,
        content_manager: &ContentManager,
    ) {
        let element = &element_manager.elements[self.element_index];
        if !self.is_initialized {
            if !self.color.is_empty() {
                element.set_attribute("fill", self.color.as_str()).unwrap();
            }
            if let PartType::Title(title_status) = &self.part_type {
                element_manager.elements[self.element_index]
                    .next_element_sibling()
                    .unwrap()
                    .set_inner_html(title_status.title.as_str());
            }
            self.is_initialized = true;
        }
        let x_value = self.x_value(base_rect).to_string();
        let y_value = self.y_value(base_rect).to_string();
        element.set_attribute("x", x_value.as_str()).unwrap();
        element.set_attribute("y", y_value.as_str()).unwrap();
        element
            .set_attribute("width", self.width_value(base_rect).to_string().as_str())
            .unwrap();
        // TODO
        // 最小化で0.0チェックを追加したが違和感
        let height = self.height_value(base_rect);
        if height > 0.0 {
            element
                .set_attribute("height", height.to_string().as_str())
                .unwrap();
        }
        if self.has_content {
            if let Some(content_element) =
                element_manager.elements[self.element_index].next_element_sibling()
            {
                if content_element.tag_name() == "g" {
                    content_element
                        .set_attribute(
                            "transform",
                            format!("translate({}, {})", x_value, y_value).as_str(),
                        )
                        .unwrap();
                }
            }
        }
        for internal in self.internal_part_rect.iter_mut() {
            internal.adjust(base_rect, element_manager, content_manager);
        }
        if let PartType::Scrollable = self.part_type {
            self.adjust_scrollable(base_rect, element_manager, content_manager);
        }
    }

    fn hide(&self, element_manager: &ElementManager) {
        let element = &element_manager.elements[self.element_index];
        element.set_attribute("width", "0").unwrap();
    }

    fn adjust_scrollable(
        &self,
        base_rect: &BaseRect,
        element_manager: &ElementManager,
        content_manager: &ContentManager,
    ) {
        let (has_content, group_x, group_y, content_element_index) =
            self.get_internal_content_size(element_manager);
        if !has_content {
            return;
        }
        let internal_max_width = group_x / element_manager.scale + 10.0;
        let internal_max_height = group_y / element_manager.scale + 10.0;
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
        let mut table_content_x = 0.0;
        let mut table_content_y = 0.0;
        if width_ratio_with_bar <= threshold
            || (width_ratio <= threshold && height_ratio <= threshold)
            // TODO
            // 最小化対応で height <= 0.0 の時に ScrollBarXも隠すようにしたが違和感
            || base_height <= 0.0
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
                    let content = &element_manager.elements[content_element_index];
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
                    let content = &element_manager.elements[content_element_index];
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
                // TODO
                // 最小化対応で height >= 0.0 チェックを追加したが違和感
                sibling_group.set_inner_html(format!("<clipPath id='clip-path-table-content-{}'><rect fill='white' x='{}' y='{}' width='{}' height='{}'></rect></clipPath>", table_content_state.content_id_token, -table_content_x, -table_content_y, self.width_value(base_rect), self.height_value(base_rect).max(0.0)).as_str());
                table_content_state.init(element_manager, &sibling_group, content_manager);
                table_content_x += self.x_value(base_rect);
                table_content_y += self.y_value(base_rect);
                sibling_group
                    .set_attribute(
                        "transform",
                        format!("translate({}, {})", table_content_x, table_content_y).as_str(),
                    )
                    .unwrap();
                sibling_group
                    .set_attribute(
                        "clip-path",
                        format!(
                            "url(#clip-path-table-content-{})",
                            table_content_state.content_id_token
                        )
                        .as_str(),
                    )
                    .unwrap();
            }
        }
        // TODO
        // 最小化対応で height <= 0.0 の時に Scrollableを隠すようにしたが違和感
        if base_height <= 0.0 {
            element_manager.elements[self.element_index]
                .set_attribute("height", "0")
                .unwrap();
        }
    }
    pub(crate) fn get_internal_content_size(
        &self,
        element_manager: &ElementManager,
    ) -> (bool, f64, f64, usize) {
        let content_part = self.internal_part_rect.iter().find(|internal| {
            if let PartType::TableContent(..) = internal.part_type {
                true
            } else {
                false
            }
        });
        if content_part.is_none() {
            return (false, 0.0, 0.0, 0);
        }
        let found_content_part = content_part.unwrap();
        let content_index = found_content_part.element_index;
        if let Some(sibling) = element_manager.elements[content_index].next_element_sibling() {
            if sibling.tag_name() == "g" {
                let bounding = sibling.get_bounding_client_rect();
                return (true, bounding.width(), bounding.height(), content_index);
            }
        }
        (false, 0.0, 0.0, 0)
    }
}

impl PartRect {
    pub(crate) fn is_inner(&self, raw_x: f64, raw_y: f64, base_rect: &BaseRect) -> bool {
        let x = raw_x - base_rect.x_amount.value();
        let y = raw_y - base_rect.y_amount.value();
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
    pub(crate) fn width_value(&self, base_rect: &BaseRect) -> f64 {
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
    pub(crate) fn height_value(&self, base_rect: &BaseRect) -> f64 {
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
pub(crate) struct MinimizeOption {
    pub(crate) minimized_width: f64,
    pub(crate) minimized_height: f64,
}

impl MinimizeOption {
    pub(crate) fn minimize_window(&self, figure: &mut Figure) {
        figure.base_rect.height.amount.base = self.minimized_height;
        figure.base_rect.width.amount.base = self.minimized_width;
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ShowContentOption {}

impl ShowContentOption {
    pub(crate) fn adjust_to_show_content(
        &self,
        figure: &mut Figure,
        element_manager: &ElementManager,
    ) {
        let mut content_width = 0.0;
        let mut content_height = 0.0;
        if let Some(_) = figure.parts.iter().find(|parts| {
            if let PartType::Scrollable = parts.part_type {
                let (content_flag, width, height, ..) =
                    parts.get_internal_content_size(element_manager);
                content_width = width / element_manager.scale;
                content_height = height / element_manager.scale;
                content_flag
            } else {
                false
            }
        }) {
            // TODO
            // padding がめっちゃハードコーディング
            figure.base_rect.width.amount = Amount::new(content_width + 28.0);
            figure.base_rect.height.amount = Amount::new(content_height + 50.0);
        }
    }
}

#[derive(Clone, Debug)]
pub enum ButtonType {
    Minimize(MinimizeOption),
    ShowContent(ShowContentOption),
}

impl ButtonType {
    fn draw_button(element_manager: &mut ElementManager, element_index: usize, symbol_id: &str) {
        let content_group = element_manager.elements[element_index]
            .next_element_sibling()
            .unwrap();
        let symbol = element_manager
            .document
            .get_element_by_id(symbol_id)
            .unwrap();
        for n in 0..symbol.child_nodes().length() {
            let node = symbol.child_nodes().item(n).unwrap().clone_node().unwrap();
            content_group.append_child(&node).unwrap();
        }
    }
}
