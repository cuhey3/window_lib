use crate::binder::element_manager::ElementManager;
use crate::figure::TemporaryState;
use web_sys::{Document, Element};

#[derive(Clone, Debug)]
pub(crate) struct TableContentState {
    thead_data: Vec<StringBinder>,
    thead_column_styles: Vec<ColumnStyle>,
    pub(crate) tbody_data: Vec<Vec<StringBinder>>,
    pub(crate) tbody_column_styles: Vec<ColumnStyle>,
    pub(crate) content_id_token: String,
}

impl TableContentState {
    pub(crate) fn new(token: &str) -> TableContentState {
        TableContentState {
            thead_data: vec![],
            thead_column_styles: vec![],
            tbody_data: vec![],
            tbody_column_styles: vec![],
            content_id_token: token.to_string(),
        }
    }
    pub(crate) fn init(&self, element_manager: &ElementManager, table_container: &Element) {
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
                // TODO
                // adjust の中では mut にできない
                // for m in 0..self.tbody_data[n].len() {
                //   self.tbody_data[n][m].check_and_update_value(&State {});
                // }
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
pub(crate) struct StringBinder {
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
    pub(crate) fn new_with_str(arg: &str) -> StringBinder {
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
pub(crate) struct ColumnStyle {
    // 以下の要素以外の装飾をしたい時に defs を使う
    // 使わない時はブランクでOK
    pub(crate) defs_id: String,
    pub(crate) text_anchor_type: TextAnchorType,
    pub(crate) x: f64,
    pub(crate) font_size: f64,
    pub(crate) first_y: f64,
    pub(crate) dy: f64,
}

impl ColumnStyle {
    fn get_element_by_defs_id(&self, document: &Document) -> Element {
        // TODO
        // この実装はうまく動きません
        // なぜならclone()では要素が移動してしまうから
        let element = document
            .get_element_by_id(self.defs_id.as_str())
            .unwrap()
            .clone();
        element.remove_attribute("id").unwrap();
        element
    }
}

#[derive(Clone, Debug)]
pub(crate) enum TextAnchorType {
    Start,
    End,
    // 用途があれば…
    // Middle,
}
