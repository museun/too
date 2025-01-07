use too::{layout::CrossAlign, views::list};

fn main() -> std::io::Result<()> {
    let mut value = 0;
    too::run(|ui| {
        let center_list = list().vertical().cross_align(CrossAlign::Center);

        ui.show_children(center_list, |ui| {
            if ui.button("Increment").clicked() {
                value += 1;
            }
            ui.label(value);
            if ui.button("Decrement").clicked() {
                value -= 1;
            }
        });
    })
}
