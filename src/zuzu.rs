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

struct Node {
    pred: Vec<usize>,
    succ: Vec<usize>
}

struct Poset {
    min: Vec<usize>,
    max: Vec<usize>,
    elt: Vec<Node>,
    idx: usize,
    chld: Option<usize>
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
    Poset{min, max, elt, idx: 0, chld: None}
}

/*
fn extend_chain(&self, chain: &mut Vec<usize>, from: usize) -> Boundary {
    let mut next = from;
    while self.elt[next].pred.len() == 1 && self.elt[next].succ.len() == 1 {
        chain.push(next);
        next = self.elt[next].succ[0];
    }
    chain.push(next);
    if self.elt[next].pred.len() == 1 {
        Incl
    }
    else {
        Excl
    }
}

fn shrink_poset(&self) {
    let mut dt = Vec::new(); dt.resize(self.elt.len(), 0);
    let elt = &self.elt;
    for i in 0 .. elt.len() {
        let preds = elt[i].pred.len();
        let succs = elt[i].succ.len();
        if preds == 0 && succs == 0 {
            let mut chain = vec![i];
            chain_fn(Incl, Incl, &chain, &mut dt);
        }
        else if preds == 0 && succs == 1 {
            let mut chain = vec![i];
            let end = self.extend_chain(&mut chain, elt[i].succ[0]);
            chain_fn(Incl, end, &chain, &mut dt);
        }
        else if preds == 0 && succs > 1 {
            chain_fn(Incl, Incl, &vec![i], &mut dt);
            for n in 0 .. succs {
                let mut chain = vec![i];
                let end = self.extend_chain(&mut chain, elt[i].succ[n]);
                chain_fn(Excl, end, &chain, &mut dt);
            }
        }
        else if preds == 1 && succs <= 1 {
            ()  //linear or max node; nothing to do
        }
        else if preds == 1 && succs > 1 {
            for n in 0 .. succs {
                let mut chain = vec![i];
                let end = self.extend_chain(&mut chain, elt[i].succ[n]);
                chain_fn(Excl, end, &chain, &mut dt);
            }
        }
        else if preds > 1 && succs == 0 {
            chain_fn(Incl, Incl, &vec![i], &mut dt);
        }
        else if preds > 1 && succs == 1 {
            let mut chain = vec![i];
            let end = self.extend_chain(&mut chain, elt[i].succ[0]);
            chain_fn(Incl, end, &chain, &mut dt);
        }
        else if preds > 1 && succs > 1 {
            chain_fn(Incl, Incl, &vec![i], &mut dt);
            for n in 0 .. succs {
                let mut chain = vec![i];
                let end = self.extend_chain(&mut chain, elt[i].succ[n]);
                chain_fn(Excl, end, &chain, &mut dt);
            }
        }
        else {
            panic!("Invalid node type");
        }
    }
    for i in 0 .. dt.len() {
        assert!(dt[i] == 1);
    }
}
*/

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

fn shrink_poset(&self, x: &[TT]) {
    let mut dt = Vec::new(); dt.resize(self.elt.len(), 0);
    let elt = &self.elt;
    for i in 0 .. elt.len() {
        let preds = elt[i].pred.len();
        let succs = elt[i].succ.len();
        if preds == 0 && succs == 0 {
            let mut chain = vec![i];
            chain_fn(&chain, x, &mut dt);
        }
        else if preds == 0 && succs == 1 {
            let mut chain = vec![i];
            self.extend_chain(&mut chain, elt[i].succ[0]);
            chain_fn(&chain, x, &mut dt);
        }
        else if preds == 0 && succs > 1 {
            chain_fn(&vec![i], x, &mut dt);
            for n in 0 .. succs {
                let mut chain = vec![];
                self.extend_chain(&mut chain, elt[i].succ[n]);
                chain_fn(&chain, x, &mut dt);
            }
        }
        else if preds == 1 && succs <= 1 {
            ()  //linear or max node; nothing to do
        }
        else if preds == 1 && succs > 1 {
            for n in 0 .. succs {
                let mut chain = vec![];
                self.extend_chain(&mut chain, elt[i].succ[n]);
                chain_fn(&chain, x, &mut dt);
            }
        }
        else if preds > 1 && succs == 0 {
            chain_fn(&vec![i], x, &mut dt);
        }
        else if preds > 1 && succs == 1 {
            let mut chain = vec![i];
            self.extend_chain(&mut chain, elt[i].succ[0]);
            chain_fn(&chain, x, &mut dt);
        }
        else if preds > 1 && succs > 1 {
            chain_fn(&vec![i], x, &mut dt);
            for n in 0 .. succs {
                let mut chain = vec![];
                self.extend_chain(&mut chain, elt[i].succ[n]);
                chain_fn(&chain, x, &mut dt);
            }
        }
        else {
            panic!("Invalid node type");
        }
    }
    //for i in 0 .. dt.len() {
    //    assert!(dt[i] == 1);
    //}
}

}

fn chain_fn(c: &[usize], x: &[TT], _dt: &mut[usize]) {
    println!("===");
    for i in 0 .. c.len() {
        println!("{}", x[c[i]].text);
    }
}

impl Iterator for Poset {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        let elt = &self.elt;
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
            self.extend_chain(&mut chain, elt[self.idx].succ[0]);
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
                    self.extend_chain(&mut chain, elt[self.idx].succ[n]);
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
            self.extend_chain(&mut chain, elt[self.idx].succ[n]);
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

fn check_iterator(x: &[TT]) -> bool {
    let p = Poset::create_poset(x);
    assert!(p.elt.len() == x.len());
    let mut dt = Vec::new(); dt.resize(p.elt.len(), 0);
    for c in p {
        for i in 0 .. c.len() {
            dt[c[i]] += 1;
        }
    }
    for i in 0 .. dt.len() {
        assert!(dt[i] == 1);
    }
    true
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
    let magic_num1 = x[c[0]].b_box.width();
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


fn zuzu1(x: &[TT]) -> bool {
    let p = Poset::create_poset(x);
    assert!(p.elt.len() == x.len());
    let mut dt = Vec::new(); dt.resize(p.elt.len(), 0);
    for c in p {
        let sf = split_by_font(x, &c);
        for i in 0 .. sf.len() {
            let sd = split_by_distance(x, sf[i]);
            for j in 0 .. sd.len() {
                let w = &sd[j];
                println!("===");
                for k in 0 .. w.len() {
                    println!("{}", x[w[k]].text);
                }
            }
        }
    }
    true
}

fn zuzu(x: &[TT]) -> bool {
    let _poset = Poset::create_poset(x);
    let mut r = vec![];
    for i in 0 .. x.len() {
        r.push((x[i].font_size, String::from(x[i].font_name.as_str())));
    }
    r.sort_by(|&(s1, ref n1), &(s2, ref n2)| if s1 == s2 {n1.cmp(n2)} else {s1.partial_cmp(&s2).unwrap()});
    r.dedup_by(|a,b| *a == *b);

    for i in 0 .. r.len() {
        let mut cl = vec![];
        let (font_size, ref font_name) = r[i];
        println!("=== {} {}", font_name, font_size);
        for j in 0 .. x.len() {
            for k in 0 .. x.len() {
                if x[j].font_size != font_size || x[j].font_name != *font_name {
                    continue
                }
                if x[k].font_size != font_size || x[k].font_name != *font_name {
                    continue
                }
                if !is_succ(&x, &x[j], &x[k]) {
                    continue
                }
                //cl.push((x[j].b_box.dist(&x[k].b_box), j, k));
                cl.push(x[j].b_box.dist(&x[k].b_box));
                //println!("### {}", x[j].b_box.dist(&x[k].b_box));
                //println!("  {}\n  {}", x[j].text, x[k].text);
            }
        }
       // if cl.is_empty() {
       //     cl.push((0.0, j, j));
       // }
        cl.sort_by(|a,b| a.partial_cmp(b).unwrap());
        println!("{:?}", cl);
        let mut avg = 0.0;
        for x1 in 0 .. cl.len() {
                avg += cl[x1];
        }
        //vvv.sort_by(|a,b| a.partial_cmp(b).unwrap());
        if cl.len() > 0 {
            println!("--> {:?}", avg/(cl.len() as f64));
        }
    }
    return true
}

/*
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
*/

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

    zuzu1(&strings);
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
