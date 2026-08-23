use arnes::Roots;
use std::path::Path;

#[test]
fn repository_and_home_roots_are_injectable() {
    let roots = Roots::new("fixture/repository", "fixture/home");

    assert_eq!(roots.repository(), Path::new("fixture/repository"));
    assert_eq!(roots.home(), Path::new("fixture/home"));
}
