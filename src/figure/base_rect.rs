use crate::binder::element_manager::ElementManager;
use crate::figure::{PartType, RectLength};
use crate::math::{Amount, Point};

impl BaseRect {
    pub(crate) fn x_value(&self) -> f64 {
        self.x_amounts[0].value()
    }
    pub(crate) fn y_value(&self) -> f64 {
        self.y_amounts[0].value()
    }
    pub(crate) fn width_value(&self) -> f64 {
        self.width.value()
    }
    pub(crate) fn height_value(&self) -> f64 {
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
    pub(crate) fn move_xy(&mut self, start_point: &Point, delta_point: &Point, always_fixed: bool) {
        if !self.x_fixed {
            self.move_x(start_point.x, delta_point.x, always_fixed);
        }
        if !self.y_fixed {
            self.move_y(start_point.y, delta_point.y, always_fixed);
        }
    }

    pub(crate) fn update_base(&mut self) {
        for amount in self.x_amounts.iter_mut() {
            amount.update_base();
        }
        for amount in self.y_amounts.iter_mut() {
            amount.update_base();
        }
        self.width.fix();
        self.height.fix();
    }

    pub(crate) fn adjust(&self, element_manager: &ElementManager) {
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

#[derive(Clone)]
pub(crate) struct BaseRect {
    pub(crate) x_amounts: Vec<Amount>,
    pub(crate) y_amounts: Vec<Amount>,
    pub(crate) width: RectLength,
    pub(crate) height: RectLength,
    pub(crate) color: String,
    pub(crate) part_type: PartType,
    pub(crate) is_grabbed: bool,
    pub(crate) x_fixed: bool,
    pub(crate) y_fixed: bool,
    pub(crate) element_index: usize,
}
