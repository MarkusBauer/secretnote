import {Component, OnInit} from '@angular/core';
import {ActivatedRoute} from "@angular/router";
import {BackendService} from "../backend.service";

@Component({
    selector: 'app-page-note-admin',
    templateUrl: './page-note-admin.component.html',
    styleUrls: ['./page-note-admin.component.less']
})
export class PageNoteAdminComponent implements OnInit {

    ident: string;
    key: string;
    state: string = "loading";
    url: string;
    adminUrl: string;

    constructor(private route: ActivatedRoute, private backend: BackendService) {
    }

    ngOnInit(): void {
        this.route.paramMap.subscribe(map => {
            this.ident = map.get("ident");
            this.updateIdentKey();
        });
        this.route.fragment.subscribe(f => {
            this.key = f;
            this.updateIdentKey();
        });
    }


    updateIdentKey() {
        if (!this.ident || this.ident.length != 24) {
            // TODO report error
            this.state = "error";
        }
        // TODO check key

        this.url = this.backend.generatePublicUrl(this.ident, this.key);
        this.adminUrl = this.backend.generatePrivateUrl(this.ident, this.key);

        this.backend.checkNote(this.ident).subscribe(exists => {
            this.state = exists ? "ready" : "missing";
        });
    }


}
