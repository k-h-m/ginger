

#[derive(Debug)]
pub struct Author {
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug)]
pub enum GlobalId {
    DOI(String),
    ISBN(String),
    ARXIV(String),
    CITEER(String),
    HAL(String),
}

#[derive(Debug)]
pub struct Item {
    pub title: String,
    pub authors: Vec<Author>,
    pub publisher: Option<String>,
    pub pub_date: Option<u32>,
    pub global_id: Option<GlobalId>
}

#[derive(Debug)]
pub enum ItemType {
    Unknown,
    Named,
    Arxiv,
    Springer,
}

static ITEM_TYPES: [(ItemType,&str); 4] = [(ItemType::Unknown, "unknown: "), (ItemType::Named, "named: "),
                            (ItemType::Arxiv, "arxiv: "), (ItemType::Springer, "springer: ")];

pub fn item_type(s: &str) -> Option<(&ItemType,&str)> {
    for &(ref i, ref m) in ITEM_TYPES.iter() {
        if s.starts_with(m) {
            return Some((i, &s[m.len()..]))
        }
    }
    return None
}

