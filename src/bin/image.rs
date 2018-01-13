extern crate gtk;
extern crate cairo;
extern crate gdk;
extern crate gdk_pixbuf;

use gtk::prelude::*;
use gtk::DrawingArea;
use cairo::{Context, Format, ImageSurface};
use gdk_pixbuf::{Pixbuf};
use gdk::ContextExt;

struct Diff {
    width:     i32,
    height:    i32,
    channel:   i32,
    rowstride: i32,
    img1:      Pixbuf,
    img2:      Pixbuf,
    buf1:      *const u8,
    buf2:      *const u8
}

impl Diff {
    pub fn new(path1: &'static str, path2: &'static str, width: i32, height: i32) -> Diff {
        let img1 = Pixbuf::new_from_file_at_size(path1, width, height).unwrap();
        let channel = img1.get_n_channels();
        let rowstride = img1.get_rowstride();
        let img2 = Pixbuf::new_from_file_at_size(path2, width, height).unwrap();
        let buf1;
        let buf2;
        unsafe {
            buf1 = img1.get_pixels(). as_mut_ptr();
            buf2 = img2.get_pixels(). as_mut_ptr();
        }
        Diff {
            width:     width,
            height:    height,
            channel:   channel,
            rowstride: rowstride,
            img1:      img1,
            img2:      img2,
            buf1:      buf1,
            buf2:      buf2
        }
    }

    fn get_colors_from_buf1_by_coordinates(&self, x: i32, y: i32) -> (u8, u8, u8) {
        unsafe {
            (
                *self.buf1.offset((x * self.channel + y * self.rowstride) as isize),
                *self.buf1.offset((x * self.channel + y * self.rowstride + 1) as isize),
                *self.buf1.offset((x * self.channel + y * self.rowstride + 2) as isize)
            )
        }
    }

    fn get_colors_from_buf2_by_coordinates(&self, x: i32, y: i32) -> (u8, u8, u8) {
        unsafe {
            (
                *self.buf2.offset((x * self.channel + y * self.rowstride) as isize),
                *self.buf2.offset((x * self.channel + y * self.rowstride + 1) as isize),
                *self.buf2.offset((x * self.channel + y * self.rowstride + 2) as isize)
            )
        }
    }

    pub fn compare(&mut self) -> Vec<u8> {
        let mut vec = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let pix_buf1 = self.get_colors_from_buf1_by_coordinates(x, y);
                let pix_buf2 = self.get_colors_from_buf2_by_coordinates(x, y);
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
        vec
    }
}


fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("Diff Images");
    window.set_position(gtk::WindowPosition::Center);
    let drawing_area = Box::new(DrawingArea::new)();
    window.set_default_size(800, 600);
        window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });
    window.add(&drawing_area);

    let mut diff = Diff::new(
        "file1.1836x3264.png",
        "file2.1836x3264.png",
        1836,
        3264
    );
    let vec = diff.compare();
    let img = Pixbuf::new_from_vec(vec, 0, false, 8, diff.width, diff.height, diff.rowstride);

    drawing_area.connect_draw(move |w, cr| {
        cr.scale(0.2, 0.2);
        cr.set_source_pixbuf(&diff.img1, 0f64, 0f64);
        cr.paint();

        cr.translate(400., 0.);
        //cr.scale(5., 5.);
        cr.set_source_pixbuf(&diff.img2, 0f64, 0f64);
        cr.paint();
        //cr.paint_with_alpha(0.5);

        cr.translate(400., 0.);
        cr.set_source_pixbuf(&img, 0f64, 0f64);
        cr.paint();
        Inhibit(false)
    });
    window.show_all();
    gtk::main();
}
