const minify = require('html-minifier').minify;
const fs = require('fs');

function minify_file(fname) {
	let text = fs.readFileSync(fname).toString();
	// html-minifier ../fe/de/index.html --collapse-whitespace --remove-comments --remove-optional-tags --remove-redundant-attributes --remove-script-type-attributes --use-short-doctype -o ../fe/de/index.html
	// html-minifier ../fe/en/index.html --collapse-whitespace --remove-comments --remove-optional-tags --remove-redundant-attributes --remove-script-type-attributes --use-short-doctype -o ../fe/en/index.html
	text = minify(text, {
		collapseWhitespace: true,
		removeComments: true,
		removeOptionalTags: true,
		removeRedundantAttributes: true,
		removeScriptTypeAttributes: true,
		useShortDoctype: true
	});
	fs.writeFileSync(fname, text);
}

// Minify HTML files
minify_file("../fe/en/index.html");
minify_file("../fe/de/index.html");

// TODO SSR

// Favicon
fs.copyFileSync("../fe/en/favicon.ico", "../fe/favicon.ico");

console.log("post-build steps finished.")
