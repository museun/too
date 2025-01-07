use too::{
    format_str,
    layout::Align2,
    view::ViewExt as _,
    views::{label, LabelStyle},
};

fn main() -> std::io::Result<()> {
    let mut counter = 0_i32;
    too::run(|ui| {
        ui.center(|ui| {
            ui.aligned(Align2::RIGHT_TOP, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("inc").clicked() {
                        counter += 1;
                    }
                    if ui.button("dec").clicked() {
                        counter -= 1;
                    }
                    ui.show(
                        label(format_str!("counter: {counter}"))
                            .class_if(LabelStyle::success, counter > 0)
                            .or(|c| c.class(LabelStyle::info), counter < 0)
                            .or(|c| c.class(LabelStyle::warning), counter < -3)
                            .or(|c| c.class(LabelStyle::danger), counter < -5),
                    )
                });
            });

            ui.vertical(|ui| {
                ui.label("normal");
                ui.show(
                    label("hoverable")
                        .hoverable()
                        .class_if(LabelStyle::danger, counter < 0)
                        .or(|s| s.class(LabelStyle::success), counter > 0),
                );
                ui.show(label("disabled").disabled_if(counter >= 3));
            });
        });
    })
}
