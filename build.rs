fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winresource::WindowsResource::new();

        //res.set_icon("assets/icon.jpg");
        res.set_language(0x411);
        res.set("FileDescription", "画像変換・メタデータ削除ツール");
        res.set("ProductName", "Image converter tool");
        res.set("LegalCopyright", "Copyright © 2026 yumitomo");

        res.compile().unwrap();
    }
}
