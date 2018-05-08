extern crate poppler;
extern crate glib;
extern crate glib_sys;
#[macro_use] extern crate clap;
extern crate walkdir;

mod item;

use item::ItemType;

fn substr(s: &str, from: usize, to: usize) -> String {
    s.chars().skip(from).take(to).collect()
}

fn max_font(v: &Vec<poppler::TextAttr>) -> (f64, usize, usize) {
    assert!(v.len() > 0);
    v.iter().fold((v[0].font_size, v[0].start_index, v[0].end_index),
        |acc, x| if x.font_size > acc.0 {(x.font_size, x.start_index, x.end_index)} else {acc})
}

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

fn named(s: &str) -> bool {
    let path = std::path::Path::new(s);
    let filename = path.file_stem().unwrap().to_str().unwrap();
    let v1 = filename.split("--").collect::<Vec<_>>();
    if v1.len() == 3 {
        return true
    }
    let v2 = filename.split("..").collect::<Vec<_>>();
    v2.len() == 3
}

fn springer(doc: &poppler::PopplerDocument) -> Result<bool,glib::error::Error> {
    if doc.get_n_pages() < 2 {
        return Ok(false)
    }
    let page = doc.get_page(0)?;
    let text = page.get_text();
    let layout = page.get_text_layout()?;
    let attr = page.get_text_attributes();

    validate_page(&text, &attr, &layout)?;

    if text.lines().count() < 10 {
        return Ok(false)
    }
    let yy = text.lines().nth(1).unwrap();
    //println!("{}", yy.trim());
    Ok(yy.starts_with("DOI"))

}

fn arxiv(doc: &poppler::PopplerDocument) -> Result<bool,glib::error::Error> {
    if doc.get_n_pages() < 2 {
        return Ok(false)
    }
    let page = doc.get_page(0)?;
    let text = page.get_text();
    let layout = page.get_text_layout()?;
    let attr = page.get_text_attributes();

    validate_page(&text, &attr, &layout)?;

    let (_, s, e) = max_font(&attr);
    //let zz = attr.sort_by(|a,b| a.font_size.partial_cmp(&b.font_size).unwrap());
    let yy = substr(&text, s, e-s+1);
    //println!("{}", yy.trim());
    Ok(yy.starts_with("arXiv"))
}

fn run(filename: &str) -> Result<(), glib::error::Error> {
    //let filename = "test.pdf";
    let doc = poppler::PopplerDocument::new_from_file(filename, "")?;
    let num_pages = doc.get_n_pages();

    println!("Document has {} page(s)", num_pages);

    // FIXME: move iterator to poppler
    for page_num in 0..num_pages {
        let page = doc.get_page(page_num).unwrap();
        let (w, h) = page.get_size();
        println!("page {} has size {}, {}", page_num, w, h);

        let text = page.get_text();
        let text_lossy = page.get_text_lossy();
        let layout = page.get_text_layout().unwrap();
        let attr = page.get_text_attributes();
        let bb = text.chars().count();
        let cc = attr[attr.len() - 1].end_index;
        assert!(bb == layout.len());
        assert!(bb == cc + 1);
        println!("exact={}, lossy={}, chars={}, layout={}, attr={}", text.len(), text_lossy.len(), bb, layout.len(), cc);

        //println!("vec={:?}", page.get_text_attributes());
        let (_, s, e) = max_font(&attr);
        //let zz = attr.sort_by(|a,b| a.font_size.partial_cmp(&b.font_size).unwrap());
        let yy = substr(&text, s, e-s+1);
        println!("{}", yy);

//        for (c,r) in text.chars().zip(layout.iter()) {
//            println!("{} {:?}", c, r);
//        }
    }
    //         g_object_unref (page);

    Ok(())
}

fn process_file(file: &str) -> Result<ItemType, glib::error::Error> {
    if named(file) {
        return Ok(ItemType::Named)
    }

    let doc = poppler::PopplerDocument::new_from_file(file, "")?;

    if arxiv(&doc)? {
        return Ok(ItemType::Arxiv)
    }

    if springer(&doc)? {
        return Ok(ItemType::Springer)
    }

    Ok(ItemType::Unknown)
}

fn main() {
    let matches = clap::App::new("ginger")
        .version(crate_version!())
        .arg(clap::Arg::with_name("recursive")
            .short("r")
            .long("recursive")
            .help("Recursive traversal"))
        .arg(clap::Arg::with_name("FILE")
            .help("Name of input file")
            .required(true)
            .index(1))
        .get_matches();

    let file = matches.value_of("FILE").unwrap();

    if matches.is_present("recursive") {
        for entry in walkdir::WalkDir::new(file) {
            let entry = entry.unwrap();
            let path = entry.path();
            let meta = std::fs::metadata(path).unwrap();
            if meta.file_type().is_file() {
                let file = entry.path().to_str().unwrap();
                match process_file(file) {
                    Err(_) => println!("error: {}", file),
                    Ok(ItemType::Unknown) => println!("unknown: {}", file),
                    Ok(ItemType::Arxiv) => println!("arxiv: {}", file),
                    Ok(ItemType::Springer) => println!("springer: {}", file),
                    Ok(ItemType::Named) => println!("named: {}", file)
                }
            }
        }
    }
    else {
        println!("{:?}", process_file(file));
    }

    //match run(&name) {
    //    Ok(()) => (),
    //    Err(e) => {
    //        println!("ERROR: {}", e);
    //    }
    //};
}

