const minify = require('html-minifier').minify;
const fs = require('fs');

function minify_file(fname) {
    let text = fs.readFileSync(fname).toString();
    // html-minifier ../fe/de/index.html --collapse-whitespace --remove-comments --remove-optional-tags --remove-redundant-attributes --remove-script-type-attributes --use-short-doctype -o ../fe/de/index.html
    // html-minifier ../fe/en/index.html --collapse-whitespace --remove-comments --remove-optional-tags --remove-redundant-attributes --remove-script-type-attributes --use-short-doctype -o ../fe/en/index.html
    //*
    text = minify(text, {
        collapseWhitespace: true,
        removeComments: true,
        removeOptionalTags: true,
        removeRedundantAttributes: true,
        removeScriptTypeAttributes: true,
        useShortDoctype: true,
        // ignoreCustomFragments: [/<app-root[\s\S]*?<\/app-root>/]
    }); // */
    // Additional patches
    text = text.replace('class="collapse navbar-collapse show"', 'class="collapse navbar-collapse"');
    text = text.replace(' defer="" ', ' defer ');
    // Additional patches end
    fs.writeFileSync(fname, text);
    console.log('Compressed ' + fname);
}

function minify_all_html(fname) {
    if (fs.statSync(fname).isDirectory()) {
        for (let file of fs.readdirSync(fname)) {
            minify_all_html(fname + "/" + file);
        }
    } else if (fname.endsWith(".html")) {
        minify_file(fname);
    }
}

function postbuild_language(dir) {
    minify_all_html(dir);
    fs.copyFileSync(dir + "/index.html", dir + "/index.all.html");
    fs.copyFileSync(dir + "/note/store/index.html", dir + "/index.html");
}

// Minify HTML files
//minify_file("../fe/en/index.html");
//minify_file("../fe/de/index.html");
postbuild_language("../fe/en");
postbuild_language("../fe/de");

// Favicon
fs.copyFileSync("../fe/en/favicon.ico", "../fe/favicon.ico");

console.log("post-build steps finished.")
