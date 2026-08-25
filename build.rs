fn main() {
    println!("cargo:rerun-if-changed=assets/iconv2.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winresource::WindowsResource::new();

        res.set_icon(r"assets\iconv2.ico");
        res.set_language(0x411);
        res.set("FileDescription", "画像変換・メタデータ削除ツール");
        res.set("ProductName", "Image converter tool");
        res.set("LegalCopyright", "Copyright © 2026 yumitomo");

        res.compile().unwrap();
    }
}
