extern crate rustc_serialize;
extern crate docopt;
extern crate piston;
#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate graphics;
extern crate opengl_graphics;
extern crate glutin_window;
extern crate vecmath;
extern crate image;
extern crate texture;


use conrod::{
    Background,
    Button,
    Color,
    Colorable,
    DropDownList,
    EnvelopeEditor,
    Frameable,
    Label,
    Labelable,
    NumberDialer,
    Point,
    Positionable,
    Slider,
    Sizeable,
    TextBox,
    Theme,
    Toggle,
    Widget,
    WidgetMatrix,
    XYPad,
};
use docopt::Docopt;
use conrod::color::{self, rgb, white, black, red, green, blue, purple};
use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};
use opengl_graphics::glyph_cache::GlyphCache;
use piston::event_loop::{Events, EventLoop};
use piston::input::{RenderEvent};
use piston::window::{WindowSettings, Size};

use std::path::Path;
use std::collections::HashMap;
use graphics::Context;
use graphics::default_draw_state;
use graphics::clear;
use texture::ImageSize;

type Ui = conrod::Ui<GlyphCache<'static>>;

const USAGE: &'static str = "
Naval Fate.

Usage:
  seq_view [<file>...]
  seq_view [<file>...] -f <value>
  seq_view [<file>...] --frame <value>
  seq_view (-h | --help)
  seq_view --version

Options:
  -f --frame <value>   Speed in knots [default: 0].
  -h --help            Show this screen.
  --version            Show version.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_file: Vec<String>,
    flag_f: u32,
    flag_frame: u32,
}

struct DemoApp {
    num_frames: u32,
    img_paths: Vec<String>,
    current_frame: u32,
    textures: HashMap<u32, opengl_graphics::Texture>,
    width: u32,
    height: u32,
    image: graphics::Image,
}

impl DemoApp {
    fn new(s: Vec<String>, current_frame: u32) -> DemoApp {
        let mut hm = HashMap::new();
        let ref p = s[current_frame as usize];
        let tex = opengl_graphics::Texture::from_path(Path::new(p)).unwrap();
        let (w, h) = tex.get_size();
        hm.insert(current_frame, tex);

        return DemoApp {
            num_frames: 0,
            img_paths: s.clone(),
            current_frame: current_frame,
            textures: hm,
            height: 0,
            width: 0,
            image: graphics::Image::new().rect([0.0, 0.0, w as f64, h as f64]),
        }
    }

    // Load the first image
    fn initialize(&self) {
        return;
    }

    pub fn get_num_images(&self) -> u32 {
        return self.num_frames;
    }

    pub fn render_frame(&mut self, c: Context, gl: &mut GlGraphics) {
        if !self.textures.contains_key(&self.current_frame) {
            let new_frame = self.current_frame;
            self.load_missing_frame(new_frame);
        }
        let ref tex = self.textures[&self.current_frame];
        println!("Rendering frame {}", self.current_frame);
        self.image.draw(tex, default_draw_state(), c.transform, gl);
        return;
    }

    pub fn set_current_frame(&mut self, new_frame: u32) {
        assert!(new_frame < self.img_paths.len() as u32);
        self.current_frame = new_frame;
        return;
    }

    fn load_missing_frame(&mut self, ref new_frame: u32) {
        assert!(self.textures.contains_key(new_frame) != true);

        let ref p = self.img_paths[*new_frame as usize];
        let tex = opengl_graphics::Texture::from_path(Path::new(p)).unwrap();
        self.textures.insert(*new_frame, tex);
        return;
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());
    println!("{:?}", args);
    
    let opengl = OpenGL::V3_2;
    let window: GlutinWindow =
        WindowSettings::new(
            "Hello Conrod".to_string(),
            Size { width: 1100, height: 550 }
        )
        .opengl(opengl)
        .exit_on_esc(true)
        .samples(4)
        .build()
        .unwrap();
    let event_iter = window.events().ups(60).max_fps(60);
    let mut gl = GlGraphics::new(opengl);

    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    let theme = Theme::default();
    let glyph_cache = GlyphCache::new(&font_path).unwrap();
    let mut ui = Ui::new(glyph_cache, theme);
    let mut demo = DemoApp::new(args.arg_file, args.flag_frame);

    let mut frame = 0;

    for event in event_iter {
        ui.handle_event(&event);
        if let Some(args) = event.render_args() {
            gl.draw(args.viewport(), |c, gl| {
                clear([1.0, 0.0, 0.4, 0.0], gl);
                draw_ui(c, gl, &mut ui, &mut demo);
                ui.draw(c, gl);
            });
            if frame > 10 {
                demo.set_current_frame(10);
            }
            frame += 1;
        }
    }
}



/// Draw the User Interface.
fn draw_ui(c: Context, gl: &mut GlGraphics, ui: &mut Ui, demo: &mut DemoApp) {

    // Sets a color to clear the background with before the Ui draws the widgets.
    // Background::new().color(rgb(0.2, 0.2, 0.2)).set(ui);

    // Calculate x and y coords for title (temporary until `Canvas`es are implemented, see #380).
    let title_x = (ui.win_w / 2.0) + 185.0;
    let title_y = (ui.win_h / 2.0) - 50.0;
    println!("title_y = {}", title_y);

    // Label example.
    Label::new("Widget Demonstration")
        .xy(100.0, 100.0)
        .font_size(32)
        .color(rgb(1.0,1.0,1.0))
        .set(TITLE, ui);

    demo.render_frame(c, gl);

}

widget_ids! {
    TITLE,
    BUTTON,
    TITLE_PAD_SLIDER,
    TOGGLE,
    COLOR_SLIDER with 3,
    SLIDER_HEIGHT,
    FRAME_WIDTH,
    TOGGLE_MATRIX with 64,
    COLOR_SELECT,
    CIRCLE_POSITION,
    ENVELOPE_EDITOR with 4
}

