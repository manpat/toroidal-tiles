use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

const INDEX_HTML_TEMPLATE: &'static str = 
r##"<html>
	<head>
		<meta charset="utf-8" />
		<meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, minimum-scale=1.0, user-scalable=no" />
		<meta name="apple-mobile-web-app-capable" content="yes" />
		<meta name="mobile-web-app-capable" content="yes" />

		<meta name="theme-color" content="#222" />
		<meta name="msapplication-navbutton-color" content="#222" />
		<meta name="apple-mobile-web-app-status-bar-style" content="#222" />

		<style>
			* {
				margin: 0;
				padding: 0;
				user-select: none;
				-moz-user-select: none;
				-khtml-user-select: none;
				-webkit-user-select: none;
				-o-user-select: none;
			}

			html, body {
				width: 100vw;
				height: 100vh;
				position: fixed;
				overflow: hidden;
			}

			canvas {
				position: absolute;
				top: 0;
				left: 0;
				width: 100%;
				height: 100%;

				overflow: hidden;
				display: block;
			}
		</style>
	</head>

	<body>
		<canvas id="canvas"></canvas>
		<script src="/[[pkg_name]]/[[build_type]].js"></script>
	</body>
</html>"##;


const MAPPING_TEMPLATE: &'static str = 
r##"/[[pkg_name]] => target/html/[[pkg_name]].html
/[[pkg_name]]/debug.js => target/asmjs-unknown-emscripten/debug/[[pkg_name]].js
/[[pkg_name]]/release.js => target/asmjs-unknown-emscripten/release/[[pkg_name]].js

"##;

fn main() {
	let profile = env::var("PROFILE").unwrap();

	let html_target_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
	let html_target_dir = Path::new(&html_target_dir).join("target/html");
	std::fs::create_dir_all(&html_target_dir).unwrap();

	let pkg_name = env!("CARGO_PKG_NAME");

	let index_html = INDEX_HTML_TEMPLATE.to_owned()
		.replace("[[build_type]]", &profile)
		.replace("[[pkg_name]]", pkg_name);

	let mapping = MAPPING_TEMPLATE.to_owned()
		.replace("[[pkg_name]]", pkg_name);
	
	let html_path = html_target_dir.join(format!("{}.html", pkg_name));

	File::create(&html_path).unwrap()
		.write_all(index_html.as_bytes()).unwrap();
	File::create("mappings.sb").unwrap()
		.write_all(mapping.as_bytes()).unwrap();

	if profile == "debug" {
		println!("cargo:rustc-cfg=debug");
	}

	// println!("cargo:rustc-cfg=dom_console");
}
