extern crate gtk;
extern crate gdk;
extern crate gdk_pixbuf;

use std::cell::RefCell;
use std::rc::Rc;
use std::path::PathBuf;
use gtk::{FileChooserAction, FileChooserButton};
use gtk::prelude::*;
use gtk::DrawingArea;
use gdk_pixbuf::Pixbuf;
use gdk::ContextExt;

type DrawingAreaRc = Rc<RefCell<gtk::DrawingArea>>;
type FileChooserButtonRc = Rc<RefCell<FileChooserButton>>;
type PathBufRc = Rc<RefCell<Option<PathBuf>>>;
type PixbufRc = Rc<RefCell<Option<Pixbuf>>>;

#[derive(Debug)]
struct Diff {
    width:        i32,
    height:       i32,
    img1:         PixbufRc,
    img2:         PixbufRc,
    channel:      i32,
    rowstride:    i32
}

fn str_to_pixbuf(path: PathBufRc, width: i32, height: i32) -> Option<gdk_pixbuf::Pixbuf> {
    match path.borrow().clone() {
        None => {
            None
        }
        Some(p) => {
            Some(Pixbuf::new_from_file_at_size(p.to_str().unwrap(), width, height).unwrap())
        }
    }
}

impl Diff {
    pub fn new(path1: PathBufRc, path2: PathBufRc, width: i32, height: i32) -> Diff {
        let img1 = PixbufRc::new(RefCell::new(
            str_to_pixbuf(path1, width, height)
        ));
        let img2 = PixbufRc::new(RefCell::new(
            str_to_pixbuf(path2, width, height)
        ));
        let channel      = 3;
        let rowstride    = width * ((3 * 8 + 7) / 8);
        Diff {
            width:        width,
            height:       height,
            img1:         img1,
            img2:         img2,
            channel:      channel,
            rowstride:    rowstride
        }
    }

    fn get_colors(&self, buf: &mut [u8], x: i32, y: i32, rowstride: i32) -> (u8, u8, u8) {
        (
            buf[(x * self.channel + y * rowstride) as usize],
            buf[(x * self.channel + y * rowstride + 1) as usize],
            buf[(x * self.channel + y * rowstride + 2) as usize]
        )
    }

    pub fn compare(&mut self) -> Result<Vec<u8>, String> {
        let img1 = self.img1.borrow().clone().unwrap();
        let img2 = self.img2.borrow().clone().unwrap();
        if img1.get_byte_length() != img2.get_byte_length() {
            return Err("Fichiers de tailles différents : impossible de les comparer.".to_string());
        }
        let buf1;
        let buf2;
        unsafe {
            buf1 = img1.get_pixels();
            buf2 = img2.get_pixels();
        }
        let mut vec = Vec::new();
        let rowstride = img1.get_rowstride(); 
        for y in 0..self.height {
            for x in 0..self.width {
                let pix_buf1 = self.get_colors(buf1, x, y, rowstride);
                let pix_buf2 = self.get_colors(buf2, x, y, rowstride);
                if pix_buf1 == pix_buf2 {
                    vec.push(0);
                    vec.push(0);
                    vec.push(0);
                }
                else {
                    vec.push(0);
                    vec.push(255);
                    vec.push(0);
                }
            }
        }
        Ok(vec)
    }
    
    pub fn result(&mut self) -> Result<Option<gdk_pixbuf::Pixbuf>, String> {
        if self.img1.borrow().clone() == None || self.img2.borrow().clone() == None {
            return Ok(None);
        }
        else {
            match self.compare() {
                Ok(vec) => {
                    let last_row_len = self.width * ((3 * 8 + 7) / 8);
                    let mut has_alpha = false;
                    if self.channel > 3 {
                        has_alpha = true;
                    }
                    Ok(Some(Pixbuf::new_from_vec(
                        vec,
                        0,
                        has_alpha,
                        8,
                        self.width,
                        self.height,
                        last_row_len
                    )))
                },
                Err(err) => {
                    Err(err)
                }
            }
        }
    }
}

fn display_pictures(drawing_area: &gtk::DrawingArea, file1: PathBufRc, file2: PathBufRc) {
    drawing_area.connect_draw(move |_w, cr| {
        let mut diff = Diff::new(
            file1.clone(),
            file2.clone(),
            346,
            382
        );
        match &diff.img1.borrow().clone() {
            &None => { }
            &Some(ref i) => {
                cr.scale(1., 1.);
                cr.set_source_pixbuf(&i, 0f64, 0f64);
                cr.paint();
            }
        }
        match &diff.img2.borrow().clone() {
            &None => { }
            &Some(ref i) => {
                cr.translate(350., 0.);
                cr.scale(1., 1.);
                cr.set_source_pixbuf(&i, 0f64, 0f64);
                cr.paint();
            }
        }
        match diff.result() {
            Err(err) => {
                println!("{}", err);
            }
            Ok(option_pixbuf) => {
                match option_pixbuf {
                    None => { },
                    Some(p) => {
                        cr.paint_with_alpha(0.5);
                        cr.translate(350., 0.);
                        cr.set_source_pixbuf(&p, 0f64, 0f64);
                        cr.paint();
                    }
                }
            }
        }
        Inhibit(false)
    });
}

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("Whatschanging : compare 2 pictures");
    window.set_position(gtk::WindowPosition::Center);
    let drawing_area = DrawingAreaRc::new(RefCell::new(
        Box::new(DrawingArea::new)()
    ));
    let drawing_area_borrow = drawing_area.borrow().clone();
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });
    let container_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let file_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let file_chooser_one = FileChooserButtonRc::new(RefCell::new(
        FileChooserButton::new("Fichier 1", FileChooserAction::Open)
    ));
    let file_chooser_one_borrow = file_chooser_one.borrow().clone();
    file_box.add(&file_chooser_one_borrow);
    let file_chooser_two = FileChooserButtonRc::new(RefCell::new(
        FileChooserButton::new("Fichier 2", FileChooserAction::Open)
    ));
    let file_chooser_two_borrow = file_chooser_two.borrow().clone();
    file_box.add(&file_chooser_two_borrow);
    container_box.add(&file_box);
    drawing_area_borrow.set_size_request(1046, 382);
    container_box.add(&drawing_area_borrow);
    file_chooser_one_borrow.connect_file_set(move |file| {
        let path_buf1 = PathBufRc::new(RefCell::new(
            file.get_filename().clone()
        ));
        let path_buf2 = PathBufRc::new(RefCell::new(
            file_chooser_two.borrow().clone().get_filename().clone()
        ));
        display_pictures(
            &drawing_area_borrow,
            path_buf1,
            path_buf2
        );
    });
    file_chooser_two_borrow.connect_file_set(move |file| {
        let path_buf1 = PathBufRc::new(RefCell::new(
            file_chooser_one.borrow().clone().get_filename().clone()
        ));
        let path_buf2 = PathBufRc::new(RefCell::new(
            file.get_filename().clone()
        ));
        display_pictures(
            &drawing_area.borrow().clone(),
            path_buf1,
            path_buf2
        );
    });
    window.add(&container_box);
    window.show_all();
    gtk::main();
}
