use std::fs::read_to_string;

fn main() {
    let file = read_to_string(std::env::var("COMPL_FILE").unwrap()).unwrap();
    let doc = compl::read::parse_doc(&file);

    let input: Vec<_> = std::env::args().skip(1).collect();
    let rule = doc.rule_book.iter().find_map(|r| r.try_rule(&input));
    if let Some(active_rule) = rule {
        let words = active_rule.words(&doc);
        for w in words {
            print!("{w} ");
        }
    }
}
