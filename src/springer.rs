extern crate poppler;
extern crate glib;
extern crate glib_sys;
#[macro_use]
extern crate clap;

//use std::io::Read;

//mod item;


fn validate_page(s: &str, attr: &Vec<poppler::TextAttr>, layout: &Vec<poppler::ffi::PopplerRectangle>) -> 
  Result<(),glib::error::Error> {
    let char_cnt = s.chars().count();
    let end_attr = attr[attr.len() - 1].end_index;
    let layout_len = layout.len();
    if (char_cnt == layout_len) && (char_cnt == end_attr + 1) {
        Ok(())
    }
    else {
        Err(glib::error::Error::new(glib::FileError::Failed,
            "XXX-ginger: Page invariants are broken"))
    }
}


fn get_margins(p: &[poppler::ffi::PopplerRectangle]) -> (f64, f64, f64, f64) {
    assert!(p.len() > 0);
    let mut top = p[0].y1;
    let mut bottom = p[0].y2;
    let mut left = p[0].x1;
    let mut right = p[0].x2;
    for i in 1..p.len() {
        if top > p[i].y1 {top = p[i].y1}
        if bottom < p[i].y2 {bottom = p[i].y2}
        if left > p[i].x1 {left = p[i].x1}
        if right < p[i].x2 {right = p[i].x2}
    }
    return (left, right, top, bottom)
}

fn process_page(doc: &poppler::PopplerDocument, num: usize) -> Result<(),glib::error::Error> {
    let page = doc.get_page(num)?;
    let text = page.get_text();
    let layout = page.get_text_layout()?;
    let attr = page.get_text_attributes();
    validate_page(&text, &attr, &layout)?;

    let (w,h) = page.get_size();
    println!("{} {}", w, h);
    println!("{:?}", get_margins(&layout));
    Ok(())
}

fn springer(fname: &str) -> Result<(),glib::error::Error> {
    let doc = poppler::PopplerDocument::new_from_file(fname, "")?;
    let num_pages = doc.get_n_pages();

    for i in 0 .. num_pages {
        process_page(&doc, i)?;
    }
    Ok(())
}

fn main() {
    let matches = clap::App::new("named")
        .version(crate_version!())
        .arg(clap::Arg::with_name("invert")
            .short("i")
            .long("invert")
            .help("Invert selection"))
        .arg(clap::Arg::with_name("FILE")
            .help("Name of input file")
            .required(true)
            .index(1))
        .get_matches();

    let _invt = matches.is_present("invert");
    let file = matches.value_of("FILE").unwrap();
    springer(file).unwrap();
}
