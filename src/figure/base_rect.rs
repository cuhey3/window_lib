use crate::binder::element_manager::ElementManager;
use crate::figure::{PartType, RectLength};
use crate::math::{Amount, Point};

#[derive(Clone)]
pub(crate) struct BaseRect {
    pub(crate) x_amount: Amount,
    pub(crate) y_amount: Amount,
    pub(crate) width: RectLength,
    pub(crate) height: RectLength,
    pub(crate) color: String,
    pub(crate) part_type: PartType,
    pub(crate) is_grabbed: bool,
    pub(crate) x_fixed: bool,
    pub(crate) y_fixed: bool,
    pub(crate) element_index: usize,
}

impl BaseRect {
    pub(crate) fn x_value(&self) -> f64 {
        0.0
    }
    pub(crate) fn y_value(&self) -> f64 {
        0.0
    }

    pub(crate) fn width_value(&self) -> f64 {
        self.width.value()
    }
    pub(crate) fn height_value(&self) -> f64 {
        self.height.value()
    }
    fn near_point_x(&self, x: f64) -> usize {
        if x < self.width.amount.base / 2.0 {
            0
        } else {
            1
        }
    }
    fn near_point_y(&self, y: f64) -> usize {
        if y < self.height.amount.base / 2.0 {
            0
        } else {
            1
        }
    }
    fn move_x(&mut self, start_x: f64, delta_x: f64, always_fixed: bool) {
        if self.width.is_fixed || always_fixed {
            self.x_amount.delta = delta_x;
            // TODO
            // constraint の処理をまとめる
            if (self.x_amount.base + delta_x).is_sign_negative() {
                self.x_amount.delta = -self.x_amount.base;
            }
        } else {
            let index = self.near_point_x(start_x);
            if index == 0 {
                self.width.amount.delta = -delta_x;
                self.width.delta_constraint();
                self.x_amount.delta = -self.width.amount.delta;
                if self.x_amount.value().is_sign_negative() {
                    self.x_amount.delta = -self.x_amount.base;
                    self.width.amount.delta = -self.x_amount.delta;
                }
            } else {
                self.width.amount.delta = delta_x;
                self.width.delta_constraint();
            }
        }
    }
    fn move_y(&mut self, start_y: f64, delta_y: f64, always_fixed: bool) {
        if self.height.is_fixed || always_fixed {
            self.y_amount.delta = delta_y;
            // TODO
            // constraint の処理をまとめる
            if (self.y_amount.base + delta_y).is_sign_negative() {
                self.y_amount.delta = -self.y_amount.base;
            }
        } else {
            let index = self.near_point_y(start_y);
            if index == 0 {
                self.height.amount.delta = -delta_y;
                self.height.delta_constraint();
                self.y_amount.delta = -self.height.amount.delta;
                if self.y_amount.value().is_sign_negative() {
                    self.y_amount.delta = -self.y_amount.base;
                    self.height.amount.delta = -self.y_amount.delta;
                }
            } else {
                self.height.amount.delta = delta_y;
                self.height.delta_constraint();
            }
        }
    }
    pub(crate) fn move_xy(&mut self, start_point: &Point, delta_point: &Point, always_fixed: bool) {
        if !self.x_fixed {
            self.move_x(start_point.x, delta_point.x, always_fixed);
        }
        if !self.y_fixed {
            self.move_y(start_point.y, delta_point.y, always_fixed);
        }
    }

    pub(crate) fn update_base(&mut self) {
        self.x_amount.update_base();
        self.y_amount.update_base();
        self.width.fix();
        self.height.fix();
    }

    pub(crate) fn adjust(&mut self, element_manager: &mut ElementManager) {
        let element = &element_manager.elements[self.element_index];
        element
            .set_attribute("width", self.width_value().to_string().as_str())
            .unwrap();
        element
            .set_attribute("height", self.height_value().to_string().as_str())
            .unwrap();
    }
    pub(crate) fn initial_adjust(&self, element_manager: &ElementManager) {
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
