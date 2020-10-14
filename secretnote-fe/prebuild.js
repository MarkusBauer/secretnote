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

        // Get code date from git
        shell.exec("git log -1 --format=%cd", {silent: true}, function (error, stdout, stderr) {
            if (!error) {
                let d = new Date(stdout.trim());
                if (d) {
                    info.build_time = d.toLocaleString();
                    info.build_timestamp = d.getTime();
                }
            }

            // Get version from git tag
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
        });
    }
});