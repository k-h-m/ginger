extern crate poppler;
extern crate glib;
extern crate glib_sys;
#[macro_use]
extern crate clap;

//use std::io::Read;
//mod item;
//
use std::cmp::Ordering;

#[derive(Debug)]
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
        if b1.top < b2.top && b2.top < b1.bottom && b1.bottom < b2.bottom {
            return Some(Ordering::Less)  //valid overlap
        }
        if b2.top < b1.top && b1.top < b2.bottom && b2.bottom < b1.bottom {
            return Some(Ordering::Greater) //valid overlap
        }
        panic!("Invalid overlap detected!"); 
    }
}

impl BBox {
    fn width(&self) -> f64 {
        self.bottom - self.top
    }

    fn length(&self) -> f64 {
        self.right - self.left
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

fn is_complete_order(x: &[TT]) -> bool {
    let mut r = vec![];
    for i in 0 .. x.len() {
        for j in 0 .. x.len() {
            for k in 0 .. x.len() {
                if (x[i].font_size != x[j].font_size) || (x[j].font_size != x[k].font_size) {
                    continue
                }
                if !is_succ(x, &x[i], &x[j]) || !is_succ(x, &x[j], &x[k]) {
                    continue
                }
                let magic1 = 1.0;
                let magic2 = 0.3;
                let d1 = x[i].b_box.dist(&x[j].b_box);
                let d2 = x[j].b_box.dist(&x[k].b_box);
                if d1 > magic1 * x[i].font_size {
                    continue 
                }
                if (d1 - d2).abs() > magic2 * x[i].font_size  {
                    continue
                }
                r.push((i,j,k));
            }
        }
    }
//    println!("{:?} {:?} {:?} {:?}", is_succ(x,&x[29],&x[30]), x[30].b_box.dist(&x[31].b_box), x[29].font_size, x[30].font_size);
//    println!("{:?}", x[4]);
//    println!("{:?}", x[5]);
    let mut clusters: Vec<Vec<(usize,usize,usize)>> = vec![];
    for i in 0 .. r.len() {
        let mut added = false;
        for j in 0 .. clusters.len() {
            let cl = &mut clusters[j];
            for k in 0 .. cl.len() {
                let (c1,c2,c3) = cl[k];
                let (r1,r2,r3) = r[i];
                if ((c1==r2) && (c2==r3)) || ((c2==r1) && (c3==r2)) {
                    cl.push(r[i]);
                    added = true;
                    break
                }
            }
        }
        if !added {
            clusters.push(vec![r[i]]);
        }
    }
    for i in 0 .. clusters.len() {
        let cl = &clusters[i];
        let mut f = vec![];
        for j in 0 .. cl.len() {
            let (a,b,c) = cl[j];
            f.push(a); f.push(b); f.push(c);
        }
        f.sort();
        f.dedup_by(|a,b| *a == *b);
        println!("===========");
        for j in 0 .. f.len() {
            println!("{} {}", f[j], x[f[j]].text);
        }
    }
    return true
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

    is_complete_order(&strings);
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
    springer(file).unwrap();
}
