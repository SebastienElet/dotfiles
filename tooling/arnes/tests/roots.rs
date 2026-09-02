mod support;

use arnes::Roots;
use std::os::unix::fs::symlink;
use std::path::Path;
use support::Fixture;

#[test]
fn repository_and_home_roots_are_injectable() {
    let roots = Roots::new("fixture/repository", "fixture/home");

    assert_eq!(roots.repository(), Path::new("fixture/repository"));
    assert_eq!(roots.home(), Path::new("fixture/home"));
}

#[test]
fn manifest_doctor_rejects_a_deployment_link_without_the_repository_layout() {
    for target in ["manifest.yaml", "home/other.yaml"] {
        let fixture = Fixture::new();
        fixture.write_repository(target, "version: 1\nagents: []\nresources: []\n");
        symlink(
            fixture.repository().join(target),
            fixture.home().join(".arnes.yaml"),
        )
        .unwrap();
        let before = fixture.snapshot();

        let output = fixture.command(["doctor", "manifest"]);

        assert_eq!(output.status.code(), Some(2), "{target}");
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            "Manifest\n✓ 0 healthy\n\nerror manifest: repository: deployed .arnes.yaml must resolve from home/.arnes.yaml\n",
            "{target}"
        );
        assert!(output.stderr.is_empty());
        assert_eq!(fixture.snapshot(), before);
    }
}
