
#[macro_use]
extern crate clap;

use std::io::Read;

mod item;

fn get_authors(s: &str) -> Vec<item::Author> {
    let mut result = vec![];
    let authors = s.split(",").collect::<Vec<_>>();
    for author in authors {
        let t = author.split(".").collect::<Vec<_>>();
        assert!(t.len() == 2);
        let r = item::Author {
            first_name: String::from(t[1].trim()),
            last_name: String::from(t[0].trim())
        };
        result.push(r);
    }
    result
}

fn get_publisher(_s: &str) -> (Option<String>,Option<u32>) {
    (None, None)
}

fn get_item(s: &str) -> Option<item::Item>{
    let v = s.split("--").collect::<Vec<_>>();
    if v.len() != 3 {
        return None
    }
    let (p,d) = get_publisher(v[2]);
    Some(item::Item {
        title: String::from(v[1].trim()),
        authors: get_authors(v[0]),
        publisher: p,
        pub_date: d,
        global_id: None
    })
}

fn named(fname: &str) {
    let path = std::path::Path::new(fname);
    let zz = path.file_stem().unwrap().to_str().unwrap();
    if let Some(x) = get_item(zz) {
            println!("{:?}", x);
    }
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

    let mut f = std::fs::File::open(file).expect("file not found");

    let mut buf = String::new();
    f.read_to_string(&mut buf).expect("something went wrong reading the file");
    for l in buf.lines() {
        match item::item_type(l) {
            Some((&item::ItemType::Springer, x)) => println!("{}", x),
            _ => ()
        }
    }
}
