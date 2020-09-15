const fs = require('fs');

const DIR = "node_modules/@primer/octicons/build/";


function findIcons(fname, icons) {
    if (fs.statSync(fname).isDirectory()) {
        for (let file of fs.readdirSync(fname)) {
            findIcons(fname + "/" + file, icons);
        }
    } else if (fname.endsWith(".html")) {
        let content = fs.readFileSync(fname).toString();
        for (let m of content.matchAll(/octicon="([^"]+)"/g)) {
            icons.add(m[1]);
        }
    }
}

let usedIcons = new Set();
findIcons("./src", usedIcons);
console.log("Used icons:", usedIcons);

// Create backup of all icons
if (!fs.existsSync(DIR + "data-all.json")) {
    fs.copyFileSync(DIR + "data.json", DIR + "data-all.json");
}

// Read all icons
let icons = JSON.parse(fs.readFileSync(DIR + "data-all.json").toString());
let iconsFiltered = {};
for (let key in icons) {
    if (icons.hasOwnProperty(key)) {
        if (usedIcons.has(key)) {
            iconsFiltered[key] = icons[key];
        }
    }
}
fs.writeFileSync(DIR + "data.json", JSON.stringify(iconsFiltered));
