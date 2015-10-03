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
extern crate window;
extern crate input;

use window::Window;

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
use piston::input::{
    RenderEvent,
    IdleEvent,
    PressEvent,
    ReleaseEvent,
    AfterRenderEvent,
    CursorEvent,
    FocusEvent,
    ResizeEvent,
    TextEvent,
    UpdateEvent,
};
use input::mouse::MouseRelativeEvent;
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
    image_width: u32,
    image_height: u32,
    image_aspect: f64,
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
            image_width: w,
            image_height: h,
            image_aspect: (w as f64) / (h as f64),
        }
    }

    // Load the first image
    fn initialize(&self) {
        return;
    }

    pub fn get_num_images(&self) -> u32 {
        return self.num_frames;
    }

    pub fn get_window_size(&self) -> Size {
        return Size { width: self.width, height: self.height };
    }

    pub fn set_window_size(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    fn position_image(&mut self) {
        let width = self.width as f64;
        let height = self.height as f64;
        let top_spacing = 100.0;
    
        let mut top_margin = top_spacing as f64;
        let mut effective_height = height - top_margin;
        let mut effective_width = effective_height * self.image_aspect;
        
        // Is the window not wide enough?
        if effective_width > width {
            effective_width = width;
            effective_height = width / self.image_aspect;
            top_margin = (height - effective_height - top_spacing) / 2.0 + top_spacing;
        }
        let left_margin = (width - effective_width) / 2.0;

        // Position the image
        self.image = self.image.rect([left_margin, top_margin, effective_width, effective_height] );
    }
    
    fn position_controls(&mut self, c: Context) {
        self.position_image();
    }

    fn render_image(&mut self, c: Context, gl: &mut GlGraphics) {
        let ref tex = self.textures[&self.current_frame];
        self.image.draw(tex, default_draw_state(), c.transform, gl);
        return;
    }

    fn render_background(&mut self, c:Context, gl: &mut GlGraphics) {
        let COLOR_BACKGROUND = [0.3, 0.3, 0.3, 0.0];
        clear(COLOR_BACKGROUND, gl);
    }

    pub fn render_frame(&mut self, c: Context, gl: &mut GlGraphics) {
        self.position_controls(c);
        if !self.textures.contains_key(&self.current_frame) {
            let new_frame = self.current_frame;
            self.load_missing_frame(new_frame);
        }
        self.render_background(c, gl);
        self.render_image(c, gl);
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
            Size { width: 640, height: 480},
        )
        .opengl(opengl)
        .exit_on_esc(true)
        .samples(4)
        .build()
        .unwrap();

    // This statement should come only after creating the window for some reason
    let mut gl = GlGraphics::new(opengl);

    // The assets directory
    let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();

    // Load the font into the GPU
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    let theme = Theme::default();
    let glyph_cache = GlyphCache::new(&font_path).unwrap();

    let mut ui = Ui::new(glyph_cache, theme);

    // Load the initial frame into memory
    let mut demo = DemoApp::new(args.arg_file, args.flag_frame);
    //window.window.set_inner_size(sz2.width, sz2.height);

    let event_iter = window.events().ups(60).max_fps(60);
    for event in event_iter {
        ui.handle_event(&event);

        // Should render here
        if let Some(args) = event.render_args() {
            gl.draw(args.viewport(), |c, gl| {
                draw_ui(c, gl, &mut ui, &mut demo);
                println!("x = {}, y = {}", ui.mouse.xy[0], ui.mouse.xy[1]);
                ui.draw(c, gl);
            });
        }

        // The mouse was pressed
        if let Some(args) = event.press_args() {
            println!("Mouse pressed");
        }

        // The mouse button was released
        if let Some(args) = event.release_args() {
            println!("Mouse released");
            demo.set_current_frame(10);
        }

        // Idling around - probably a good time to load a new image?
        if let Some(args) = event.idle_args() {
        }

        // Some key was pressed
        if let Some(args) = event.text_args() {
            let char_space = ' ';
            //match args[0] as char {
            //    char_space => println!("Space!"),
            //    _ => println!("yay"),
           // }
        }

        if let Some(args) = event.resize_args() {
            demo.set_window_size(args[0], args[1]);
        }

        event.mouse_relative(|dx, dy| println!("Relative mouse moved '{} {}'", dx, dy));
    }
}



/// Draw the User Interface.
fn draw_ui(c: Context, gl: &mut GlGraphics, ui: &mut Ui, demo: &mut DemoApp) {

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
}

