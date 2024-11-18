use web_sys::{window, Document, Element};
pub(crate) struct ElementManager {
    pub(crate) document: Document,
    pub(crate) container_id: String,
    pub(crate) elements: Vec<Element>,
    clip_paths: Vec<Element>,
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
            clip_paths: vec![],
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
            .find(|(index, value)| **value == group_index)
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
    pub(crate) fn create_element_with_clip_path(
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

    pub(crate) fn create_clip_path(&mut self, container: &Element) -> (usize, usize) {
        let clip_path = self
            .document
            .create_element_ns(Option::from("http://www.w3.org/2000/svg"), "clipPath")
            .unwrap();
        clip_path.set_id(format!("clip-path-{}", self.clip_paths.len()).as_str());
        let rect = self
            .document
            .create_element_ns(Option::from("http://www.w3.org/2000/svg"), "rect")
            .unwrap();
        clip_path.append_child(&*rect).unwrap();
        container.append_child(&*clip_path).unwrap();
        self.elements.push(rect);
        self.clip_paths.push(clip_path);
        (self.clip_paths.len() - 1, self.elements.len() - 1)
    }
    pub(crate) fn get_internal_xy(&self, x: f64, y: f64) -> (f64, f64) {
        (
            (x - self.offset_x) / self.scale,
            (y - self.offset_y) / self.scale,
        )
    }
}
