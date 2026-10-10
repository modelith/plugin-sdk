//! 生成物が Rust の型と一致しているかを検証する。
//! `UPDATE_GENERATED=1` を付けて実行すると書き出す。

use std::path::Path;

#[test]
fn generated_files_are_up_to_date() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let update = std::env::var_os("UPDATE_GENERATED").is_some();
    let mut stale = Vec::new();
    for (rel, content) in modelith_plugin_protocol::generate::files() {
        let path = root.join(rel);
        if update {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, &content).unwrap();
        } else if std::fs::read_to_string(&path).ok().as_deref() != Some(content.as_str()) {
            stale.push(rel);
        }
    }
    assert!(
        stale.is_empty(),
        "stale generated files: {stale:?}\n\
         run: UPDATE_GENERATED=1 cargo test -p modelith-plugin-protocol --test generated"
    );
}
