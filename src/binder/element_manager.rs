use web_sys::{window, Document, Element};
pub(crate) struct ElementManager {
    pub(crate) document: Document,
    pub(crate) container_id: String,
    pub(crate) elements: Vec<Element>,
    pub(crate) offset_x: f64,
    pub(crate) offset_y: f64,
    pub(crate) scale: f64,
    pub(crate) figure_groups: Vec<Element>,
    pub(crate) figure_group_order: Vec<usize>,
}

impl ElementManager {
    pub(crate) fn get_container(&self) -> Element {
        self.document
            .get_element_by_id(self.container_id.as_str())
            .unwrap()
    }
    pub(crate) fn new(container_id: &str) -> ElementManager {
        ElementManager {
            document: window().unwrap().document().unwrap(),
            container_id: container_id.to_string(),
            elements: vec![],
            offset_x: 0.0,
            offset_y: 0.0,
            scale: 1.0,
            figure_groups: vec![],
            figure_group_order: vec![],
        }
    }

    pub(crate) fn re_append_figure(&mut self, group_index: usize) {
        let temporary_id = "temporary_id_re_append_figure";
        self.figure_groups[group_index].set_id(temporary_id);
        if let Some((index, _)) = self
            .figure_group_order
            .iter()
            .enumerate()
            .find(|(_, value)| **value == group_index)
        {
            self.figure_group_order.remove(index);
            self.figure_group_order.insert(0, group_index);
        };
        let element = self.document.get_element_by_id(temporary_id).unwrap();
        element.remove_attribute("id").unwrap();
        self.get_container().append_child(&*element).unwrap();
    }
    pub(crate) fn create_figure_group(&mut self, container: &Element) -> usize {
        let group = self
            .document
            .create_element_ns(Option::from("http://www.w3.org/2000/svg"), "g")
            .unwrap();
        container.append_child(&*group).unwrap();
        self.figure_groups.push(group);
        let group_index = self.figure_groups.len() - 1;
        self.figure_group_order.push(group_index);
        group_index
    }

    pub(crate) fn create_element(&mut self, container: &Element) -> usize {
        let rect = self
            .document
            .create_element_ns(Option::from("http://www.w3.org/2000/svg"), "rect")
            .unwrap();
        container.append_child(&*rect).unwrap();
        self.elements.push(rect);
        self.elements.len() - 1
    }

    pub(crate) fn create_element_with_defs_id(&mut self, container: &Element, id: &str) -> usize {
        // TODO
        // get_element_by_idして、違う親に append_child すると、それは要素の移動になる！！！
        // (元のJSからそう…）
        container
            .append_child(
                &self
                    .document
                    .get_element_by_id(id)
                    .unwrap()
                    .clone_node()
                    .unwrap(),
            )
            .unwrap();
        let children = container.children();
        let children_length = children.length();
        let element = children.item(children_length - 1).unwrap();
        element.remove_attribute("id").unwrap();
        self.elements.push(element);
        self.elements.len() - 1
    }

    pub(crate) fn create_element_with_symbol_id(&mut self, container: &Element, id: &str) -> usize {
        let symbol = self.document.get_element_by_id(id).unwrap();
        let symbol_children = symbol.children();
        let symbol_children_length = symbol_children.length();
        for n in 0..symbol_children_length{
            // 要素の clone (deep copy) は Node 単位でしかできない
            // 変換後は Node になるが、child_nodes() だと 空白Node もコピーすることになるので children() を使う
            let element = symbol_children.item(n).unwrap();
            // clone_node() では、孫要素は clone されないので、先に inner_html を取得しておく
            let inner_html = element.inner_html();
            let element_node = element.clone_node().unwrap();
            container.append_child(&element_node).unwrap();
            // symbol 配下の最初の要素を rect とみなす暗黙ルール…
            let container_children = container.children();
            let container_children_length = container_children.length();
            let copied_element = container_children
                .item(container_children_length - 1)
                .unwrap();
            // Node として親へ append_child したあとに Element として再取得ができているので inner_html 書き戻し
            copied_element.set_inner_html(inner_html.as_str());
            if n == 0 {
                self.elements.push(copied_element);
            }
        }
        self.elements.len() - 1
    }
    pub(crate) fn create_element_with_group(&mut self, container: &Element) -> usize {
        let rect = self
            .document
            .create_element_ns(Option::from("http://www.w3.org/2000/svg"), "rect")
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
    pub(crate) fn get_internal_xy(&self, x: f64, y: f64) -> (f64, f64) {
        (
            (x - self.offset_x) / self.scale,
            (y - self.offset_y) / self.scale,
        )
    }
}
