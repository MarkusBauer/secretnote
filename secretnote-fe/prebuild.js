const fs = require('fs');
const shell = require('child_process');

let info = {
    build_time: new Date().toLocaleString(),
    build_timestamp: new Date().getTime(),
};


shell.exec("git rev-parse HEAD", {silent: true}, function (error, stdout, stderr) {
    if (error) {
        console.error(error)
    } else {
        info.git_hash = stdout.trim();

        shell.exec("git describe --abbrev=0 --tags", {silent: true}, function (error, stdout, stderr) {
            info.version = error ? "" : stdout.trim()
            console.log(info);

            // Save infos
            const js = 'export const buildinfo = ' + JSON.stringify(info) + ';';
            fs.writeFile("src/environments/buildinfo.js", js, function (err) {
                if (err) {
                    console.error(err);
                } else {
                    console.log("Build information stored");
                }
            });
        });
    }
});