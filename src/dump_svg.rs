extern crate poppler;
extern crate glib;
extern crate glib_sys;
#[macro_use]
extern crate clap;

extern crate svg;
use svg::Document;
use svg::node::element::Path;
use svg::node::element::path::Data;

use std::cmp::Ordering;

#[derive(Debug,Clone)]
struct BBox {
    top: f64,
    left: f64,
    bottom: f64,
    right: f64
}

impl PartialEq for BBox {
    fn eq(&self, other: &BBox) -> bool {
        self.top == other.top &&
        self.left == other.left &&
        self.bottom == other.bottom &&
        self.right == other.right
    }
}

impl PartialOrd for BBox {
    fn partial_cmp(&self, other: &BBox) -> Option<Ordering> {
        let b1 = self; let b2 = other;
        if b1.eq(b2) {
            return Some(Ordering::Equal)
        }
        if (b1.right < b2.left) || (b2.right < b1.left) {
            return None  //no horizontal overlap
        }
        if b1.bottom < b2.top {
            return Some(Ordering::Less)  //no vertical overlap
        }
        if b1.top > b2.bottom {
            return Some(Ordering::Greater)  //no vertical overlap
        }
        //overlap detected!
        if b1.top < b2.top && b2.top < b1.bottom && b1.bottom < b2.bottom {
            return Some(Ordering::Less)
        }
        if b2.top < b1.top && b1.top < b2.bottom && b2.bottom < b1.bottom {
            return Some(Ordering::Greater)
        }
        //println!("box1={:?} box2={:?}", b1, b2);
        //panic!("Invalid overlap detected!"); 
        return None
    }
}

impl BBox {
    fn width(&self) -> f64 {
        self.bottom - self.top
    }

    fn dist(&self, other: &BBox) -> f64 {
        //assert!(self.bottom < other.top);
        if self.bottom < other.top {
            other.top - self.bottom
        }
        else {
            0.0
        }
    }
}

#[derive(Debug)]
struct TT {
    text: String,
    b_box: BBox,
    font_name: String,
    font_size: f64
}

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
    assert!(attr.len() > 0);
    let mut acc = vec![];
    for i in 0.. attr.len() {
        let l = intersect_intervals(attr[i].start_index, attr[i].end_index, from, to);
        acc.push((l,i));
    }
    acc.sort_by(|&(l1,_i1),&(l2,_i2)| l1.cmp(&l2));
    &attr[acc[acc.len() - 1].1]
}

fn b_box(p: &[poppler::ffi::PopplerRectangle]) -> BBox {
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
    return BBox{left, top, right, bottom}
}

fn merge_boxes<'a, I: Iterator<Item = &'a BBox>>(mut boxes: I) -> BBox {
    let b0 = boxes.next().unwrap();
    let mut top = b0.top;
    let mut bottom = b0.bottom;
    let mut left = b0.left;
    let mut right = b0.right;
    for b in boxes {
        if top > b.top {top = b.top}
        if bottom < b.bottom {bottom = b.bottom}
        if left > b.left {left = b.left}
        if right < b.right {right = b.right}
    }
    return BBox{left,top,right,bottom}
}

struct Node {
    pred: Vec<usize>,
    succ: Vec<usize>
}

struct Poset {
    min: Vec<usize>,
    elt: Vec<Node>,
}

impl Poset {
fn create_poset(x: &[TT]) -> Poset {
    let mut min = vec![];
    let mut max = vec![];
    let mut elt = vec![];
    for i in 0 .. x.len() {
        let mut pred = vec![];
        let mut succ = vec![];
        for j in 0 .. x.len() {
            if is_pred(x, &x[i], &x[j]) {
                pred.push(j);
            }
            if is_succ(x, &x[i], &x[j]) {
                succ.push(j);
            }
        }
        if pred.is_empty() {
            min.push(i);
        }
        if succ.is_empty() {
            max.push(i);
        }
        elt.push(Node{pred, succ});
    }
    Poset{min, elt}
}

fn extend_chain(&self, chain: &mut Vec<usize>, from: usize) {
    let mut next = from;
    while self.elt[next].pred.len() == 1 && self.elt[next].succ.len() == 1 {
        chain.push(next);
        next = self.elt[next].succ[0];
    }
    if self.elt[next].pred.len() == 1 {
        chain.push(next);
    }
}

fn chain_iter<'a>(&'a self) -> ChainIterator<'a> {
    ChainIterator{poset: self, idx: 0, chld: None}
}

}

struct ChainIterator<'a> {
    poset: &'a Poset,
    idx: usize,
    chld: Option<usize>
}

impl<'a> Iterator for ChainIterator<'a> {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        let elt = &self.poset.elt;
        while self.idx < elt.len() {
            let e = &elt[self.idx];
            if e.pred.len() != 1 || e.succ.len() > 1 {
                break
            }
            self.idx += 1;
        }
        if self.idx >= elt.len() {
            return None
        }
        let preds = elt[self.idx].pred.len();
        let succs = elt[self.idx].succ.len();
        if (preds == 0 && succs == 0) || (preds > 1 && succs == 0) {
            let chain = vec![self.idx];
            self.idx += 1;
            self.chld = None;
            Some(chain)
        }
        else if (preds == 0 && succs == 1) || (preds > 1 && succs == 1) {
            let mut chain = vec![self.idx];
            self.poset.extend_chain(&mut chain, elt[self.idx].succ[0]);
            self.idx += 1;
            self.chld = None;
            Some(chain)
        }
        else if (preds == 0 && succs > 1) || (preds > 1 && succs > 1) {
            match self.chld {
                None => {
                    self.chld = Some(0);
                    Some(vec![self.idx])
                },
                Some(n) => {
                    let mut chain = vec![];
                    self.poset.extend_chain(&mut chain, elt[self.idx].succ[n]);
                    if n + 1 < succs {
                        self.chld = Some(n + 1);
                    }
                    else {
                        self.chld = None;
                        self.idx += 1;
                    }
                    Some(chain)
                }
            }
        }
        else if preds == 1 && succs > 1 {
            let n = match self.chld {None => 0, Some(x) => x};
            let mut chain = vec![];
            self.poset.extend_chain(&mut chain, elt[self.idx].succ[n]);
            if n + 1 < succs {
                self.chld = Some(n + 1);
            }
            else {
                self.chld = None;
                self.idx += 1;
            }
            Some(chain)
        }
        else {
            //unreachable code
            panic!("Unexpected error in Poset iterator");
        }
    }
}


fn split_by_font<'a>(x: &[TT], c: &'a[usize]) -> Vec<&'a[usize]> {
    assert!(c.len() > 0);
    let mut r = vec![];
    let mut from = 0;
    let mut font_size = x[c[0]].font_size;
    let mut font_name = &x[c[0]].font_name;
    for i in 1 .. c.len() {
        if x[c[i]].font_size != font_size || x[c[i]].font_name != *font_name {
            r.push(&c[from .. i]);
            from = i;
            font_size = x[c[i]].font_size;
            font_name = &x[c[i]].font_name;
        }
    }
    r.push(&c[from .. c.len()]);
    r
}

fn split_by_distance<'a>(x: &[TT], c: &'a[usize]) -> Vec<&'a[usize]> {
    if c.len() < 2 {
        return vec![c]
    }
    let mut dv = vec![];
    for i in 1 .. c.len() {
        dv.push(x[c[i-1]].b_box.dist(&x[c[i]].b_box));
    }
    let mut from = 0;
    let magic_num1 = 0.4 * x[c[0]].b_box.width();
    let mut r = vec![];
    for i in 1 .. c.len() {
        if dv[i-1] > magic_num1 {
            r.push(&c[from .. i]);
            from = i;
        }
    }
    r.push(&c[from .. c.len()]);
    r
}

fn merge_tt(t: &[TT], c: &[usize]) -> TT {
   let b = merge_boxes(c.iter().map(|&x| &t[x].b_box));
   let s = c.iter().fold("".to_string(), |acc,&x| acc + &t[x].text + "\n");
   TT{text: s, b_box: b, font_size: t[c[0]].font_size, font_name: String::from(t[c[0]].font_name.as_str())}
}

fn zuzu1(tt: &[TT], w: f64, h: f64) -> bool {
    let mut document = Document::new()
        .set("viewBox", (0, 0, w, h));

    for i in tt {
        let b = &i.b_box;
        let data = Data::new()
            .move_to((b.left, b.top))
            .line_by((0, b.bottom - b.top))
            .line_by((b.right - b.left, 0))
            .line_by((0, b.top - b.bottom))
            .close();

        let path = Path::new()
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-width", 0.5)
            .set("d", data);

        document = document.add(path);
    }
    
    svg::save("image.svg", &document).unwrap();

    true
}


fn is_succ(t: &[TT], x1: &TT, x2: &TT) -> bool {
    if !x1.b_box.lt(&x2.b_box) {
        return false
    }
    for i in 0 .. t.len() {
        if x1.b_box.lt(&t[i].b_box) && t[i].b_box.lt(&x2.b_box) {
            return false
        }
    }
    return true
}

fn is_pred(t: &[TT], x1: &TT, x2: &TT) -> bool {
    is_succ(t, x2, x1)
}

fn process_page(doc: &poppler::PopplerDocument, num: usize) -> Result<(),glib::error::Error> {
    let page = doc.get_page(num)?;
    let text = page.get_text();
    let layout = page.get_text_layout()?;
    let attr = page.get_text_attributes();
    validate_page(&text, &attr, &layout)?;

    let mut from = 0;
    let mut from_l = 0;
    let mut char_cnt = 0;
    let mut strings = vec![];

    for (i,c) in text.char_indices() {
        if c == '\n' {
            let f = font(&attr, from_l, char_cnt);
            let tt = TT {
                text: String::from(&text[from..i]),
                b_box: b_box(&layout[from_l..char_cnt+1]),
                font_name: match f.font_name {
                            Some(ref x) => String::from(x.as_str()),
                            None => panic!("Font name not found")},
                font_size: f.font_size
            };
            strings.push(tt);
            from = i + 1;
            from_l = char_cnt + 1;
        }
        char_cnt += 1;
    }

    let (w,h) = page.get_size();
    zuzu1(&strings, w, h);
    //is_complete_order(&strings);
    //println!("{:?}", strings);

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
    match springer(file) {
        Ok(()) => (),
        Err(e) => {
            println!("ERROR: {}", e);
        }
    }
}
