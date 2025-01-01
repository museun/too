use too::renderer::Border;

fn main() -> std::io::Result<()> {
    too::run(|ui| {
        ui.center(|ui| {
            ui.frame(Border::THICK, "This is a test", |ui| {
                ui.vertical(|ui| {
                    ui.label("one");
                    ui.label("two");
                    ui.label("three");
                });
            });
        });
    })
}
