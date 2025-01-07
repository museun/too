use too::{layout::CrossAlign, views::list};

fn main() -> std::io::Result<()> {
    let mut value = 0.0;
    too::run(|ui| {
        let fill_list = list().vertical().cross_align(CrossAlign::Fill);
        ui.show_children(fill_list, |ui| {
            ui.progress(value);
            ui.slider(&mut value);
        });
    })
}
