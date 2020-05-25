// #[derive(Debug, Clone)]
// pub struct WidgetManager {}
//
// impl WidgetManager {
//     fn get_widget(&self, id: WidgetId) -> Option<&dyn Widget> {
//         unimplemented!()
//     }
// }
//
// pub type WidgetId = u32;
//
// pub type Rect = (i32, i32, i32, i32);
//
// pub enum Layout {
//     Maximized,
//     Free,
//     Column,
//     Line,
// }
//
// pub enum WidgetKind {
//     Button,
//     List,
//     CheckBox,
//     Image,
//     Panel {
//         layout: Layout,
//         children: Vec<WidgetId>,
//     },
// }
//
// pub struct Widget {}
//
// impl Widget {
//     fn get_rect(&self) -> Rect {
//         unimplemented!()
//     }
// }
//
// trait Container {
//     fn get_widgets(&self) -> &Vec<WidgetId>;
// }
//
// #[derive(Debug, Clone)]
// pub struct Window {}
//
// #[derive(Debug, Clone)]
// pub struct Panel {}
