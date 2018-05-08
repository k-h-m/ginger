extern crate poppler;
extern crate glib;
extern crate glib_sys;
#[macro_use]
extern crate clap;

//use std::io::Read;

//mod item;


fn validate_page(s: &str, attr: &[poppler::TextAttr], layout: &[poppler::ffi::PopplerRectangle]) -> 
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

fn intersect_intervals(x1: usize, y1: usize, x2: usize, y2: usize) -> usize {
    assert!(x1 <= y1 && x2 <= y2);
    if y1 < x2 {return 0}
    if y2 < x1 {return 0}
    if x1 < x2 {
        if y1 < y2 {return y1 - x2 + 1}
        return y2 - x2 + 1
    }
    if y1 > y2 {return y2 -x1 + 1}
    return y1 - x1 + 1
}

fn font(attr: &[poppler::TextAttr], from: usize, to: usize) -> &poppler::TextAttr {
    let mut acc = vec![];

    for i in 0.. attr.len() {
        let l = intersect_intervals(attr[i].start_index, attr[i].end_index, from, to);
        acc.push((l,i));
    }
    acc.sort_by(|&(l1,_i1),&(l2,_i2)| l1.cmp(&l2));
    &attr[acc[acc.len() - 1].1]
}

fn b_box(p: &[poppler::ffi::PopplerRectangle]) -> (f64, f64, f64, f64) {
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
    return (left, top, right, bottom)
}


fn process_page(doc: &poppler::PopplerDocument, num: usize) -> Result<(),glib::error::Error> {
    let page = doc.get_page(num)?;
    let text = page.get_text();
    let layout = page.get_text_layout()?;
    let attr = page.get_text_attributes();
    validate_page(&text, &attr, &layout)?;

    println!(":- discontiguous b_box/5.");
    println!(":- discontiguous text/2.");
    println!(":- discontiguous font/3.");
    println!("");

    let mut from = 0;
    let mut from_l = 0;
    let mut char_cnt = 0;
    let mut line_cnt = 0;

    for (i,c) in text.char_indices() {
        if c == '\n' {
            let s = &text[from..i];
            let (left,top,right,bottom) = b_box(&layout[from_l..char_cnt+1]);
            let f = font(&attr, from_l, char_cnt);
            let fnn = match f.font_name {
                Some(ref x) => x,
                None => panic!("Font name not found")
            };
            println!("text(s{},\'{}\').", line_cnt, s);
            println!("b_box(s{},{},{},{},{}).", line_cnt, left, top, right, bottom);
            println!("font(s{},\'{}\',{}).", line_cnt, fnn, f.font_size);
            from = i + 1;
            from_l = char_cnt + 1;
            line_cnt += 1;
        }
        char_cnt += 1;
    }

    //println!("text={} cc={}", text.chars().count(), cc + text.lines().count());

    Ok(())
}

fn springer(fname: &str) -> Result<(),glib::error::Error> {
    let doc = poppler::PopplerDocument::new_from_file(fname, "")?;

    process_page(&doc, 0)?;
    
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
