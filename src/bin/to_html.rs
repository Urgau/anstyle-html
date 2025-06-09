use anstyle_html::Term;

fn main() {
    let input = std::env::args().nth(1).unwrap();
    let output = std::env::args().nth(2).unwrap();

    let vte = std::fs::read_to_string(&input).unwrap();
    let html = Term::new().render_html(&vte);

    let _ = std::fs::write(&output, html).unwrap();
}
