use crate::binder::element_manager::ElementManager;
use crate::figure::base_rect::BaseRect;
use crate::figure::AmountPositionType::{ContentBase, End, Ignore, Start};
use crate::figure::{AmountPositionType, PartType};

pub(crate) struct PartRect {
    pub(crate) x_amounts: Vec<(f64, AmountPositionType)>,
    pub(crate) y_amounts: Vec<(f64, AmountPositionType)>,
    pub(crate) color: String,
    pub(crate) element_index: usize,
    pub(crate) part_type: PartType,
    pub(crate) is_grabbed: bool,
    pub(crate) internal_part_rect: Vec<PartRect>,
}

impl PartRect {
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
    pub(crate) fn adjust(&self, base_rect: &BaseRect, element_manager: &ElementManager) {
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

impl PartRect {
    pub(crate) fn is_inner(&self, x: f64, y: f64, base_rect: &BaseRect) -> bool {
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
