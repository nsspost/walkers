use egui::{Align2, Context, Painter, Shape, Ui, Vec2};
use walkers::{Map, MapMemory, Position, PositionExt, Tiles};

fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    eframe::run_native(
        "MyApp",
        Default::default(),
        Box::new(|cc| Box::new(MyApp::new(cc.egui_ctx.clone()))),
    )
}

struct MyApp {
    tiles: Tiles,
    geoportal_tiles: Tiles,
    map_memory: MapMemory,
}

impl MyApp {
    fn new(egui_ctx: Context) -> Self {
        Self {
            tiles: Tiles::new(walkers::providers::openstreetmap, egui_ctx.to_owned()),
            geoportal_tiles: Tiles::new(walkers::providers::geoportal, egui_ctx),
            map_memory: MapMemory::default(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Typically this would be a GPS acquired position which is tracked by the map.
            let my_position = places::wroclaw_glowny();

            // Draw the actual map.
            let response = ui.add(Map::new(
                Some(&mut self.tiles),
                &mut self.map_memory,
                my_position,
            ));

            // Draw custom shapes.
            let painter = ui.painter().with_clip_rect(response.rect);
            draw_custom_shapes(ui, painter, &self.map_memory, my_position);

            // Draw utility windows.
            {
                use windows::*;

                zoom(ui, &mut self.map_memory);
                go_to_my_position(ui, &mut self.map_memory);

                orthophotomap(
                    ui,
                    &mut self.geoportal_tiles,
                    &mut self.map_memory,
                    my_position,
                );

                acknowledge(ui);
            }
        });
    }
}

mod places {
    //! Few common places in the city of Wrocław, used in the example app.
    use walkers::Position;

    /// Main train station of the city of Wrocław.
    /// https://en.wikipedia.org/wiki/Wroc%C5%82aw_G%C5%82%C3%B3wny_railway_station
    pub fn wroclaw_glowny() -> Position {
        Position::new(17.03664, 51.09916)
    }

    /// Taking a public bus (line 106) is probably the cheapest option to get from
    /// the train station to the airport.
    /// https://www.wroclaw.pl/en/how-and-where-to-buy-public-transport-tickets-in-wroclaw
    pub fn dworcowa_bus_stop() -> Position {
        Position::new(17.03940, 51.10005)
    }
}

/// Turn geographical position into location on the screen.
fn screen_position(
    position: Position,
    painter: &Painter,
    map_memory: &MapMemory,
    my_position: Position,
) -> Vec2 {
    // Turn that into a flat, mercator projection.
    let projected_position = position.project(map_memory.zoom.round());

    // We also need to know where the map center is.
    let map_center_projected_position = map_memory
        .center_mode
        .position(my_position)
        .project(map_memory.zoom.round());

    // From the two points above we can calculate the actual point on the screen.
    painter.clip_rect().center() + projected_position.to_vec2() - map_center_projected_position
}

/// Shows how to draw various things in the map.
fn draw_custom_shapes(ui: &Ui, painter: Painter, map_memory: &MapMemory, my_position: Position) {
    // Position of the point we want to put our shapes.
    let position = places::dworcowa_bus_stop();
    let screen_position = screen_position(position, &painter, map_memory, my_position);

    // Now we can just use Painter to draw stuff.
    let background = |text: &Shape| {
        Shape::rect_filled(
            text.visual_bounding_rect().expand(5.),
            5.,
            ui.visuals().extreme_bg_color,
        )
    };

    let text = ui.fonts(|fonts| {
        Shape::text(
            fonts,
            screen_position.to_pos2(),
            Align2::LEFT_CENTER,
            "⬉ Here you can board the 106 line\nwhich goes to the airport.",
            Default::default(),
            ui.visuals().text_color(),
        )
    });
    painter.add(background(&text));
    painter.add(text);
}

mod windows {
    use egui::{Align2, RichText, Ui, Window};
    use walkers::{Center, Map, MapMemory, Position, Tiles};

    pub fn acknowledge(ui: &Ui) {
        Window::new("Acknowledge")
            .collapsible(false)
            .resizable(false)
            .title_bar(false)
            .anchor(Align2::LEFT_TOP, [10., 10.])
            .fixed_size([150., 150.])
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.label("following map uses data from");
                    ui.hyperlink("https://www.openstreetmap.org");
                });
            });
    }

    pub fn orthophotomap(
        ui: &Ui,
        tiles: &mut Tiles,
        map_memory: &mut MapMemory,
        my_position: Position,
    ) {
        Window::new("Orthophotomap")
            .collapsible(false)
            .resizable(false)
            .title_bar(false)
            .anchor(Align2::RIGHT_TOP, [-10., 10.])
            .fixed_size([150., 150.])
            .show(ui.ctx(), |ui| {
                ui.add(Map::new(Some(tiles), map_memory, my_position));
            });
    }

    /// Simple GUI to zoom in and out.
    pub fn zoom(ui: &Ui, map_memory: &mut MapMemory) {
        Window::new("Map")
            .collapsible(false)
            .resizable(false)
            .title_bar(false)
            .anchor(Align2::LEFT_BOTTOM, [10., -10.])
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    if ui.button(RichText::new("➕").heading()).clicked() {
                        let _ = map_memory.zoom.zoom_in();
                    }

                    if ui.button(RichText::new("➖").heading()).clicked() {
                        let _ = map_memory.zoom.zoom_out();
                    }
                });
            });
    }

    /// When map is "detached", show a windows with an option to go back to my position.
    pub fn go_to_my_position(ui: &Ui, map_memory: &mut MapMemory) {
        if let Center::Exact(position) = map_memory.center_mode {
            Window::new("Center")
                .collapsible(false)
                .resizable(false)
                .title_bar(false)
                .anchor(Align2::RIGHT_BOTTOM, [-10., -10.])
                .show(ui.ctx(), |ui| {
                    ui.label(format!("{:.04} {:.04}", position.x(), position.y()));
                    if ui
                        .button(RichText::new("go to my (fake) position ").heading())
                        .clicked()
                    {
                        map_memory.center_mode = Center::MyPosition;
                    }
                });
        }
    }
}
