import {Component, OnInit} from '@angular/core';
import {ActivatedRoute} from "@angular/router";
import {BackendService} from "../backend.service";
import {CryptoService} from "../crypto.service";

@Component({
    selector: 'app-page-note-admin',
    templateUrl: './page-note-admin.component.html',
    styleUrls: ['./page-note-admin.component.less']
})
export class PageNoteAdminComponent implements OnInit {

    ident: string;
    adminIdent: string;
    key: string;
    state: string = "loading";
    url: string;
    adminUrl: string;

    constructor(private route: ActivatedRoute, private backend: BackendService, private crypto: CryptoService) {
    }

    ngOnInit(): void {
        this.route.paramMap.subscribe(map => {
            this.adminIdent = map.get("ident");
            this.ident = this.crypto.adminIdentToIdent(this.adminIdent);
            console.log(this.adminIdent, "=>", this.ident);
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
        this.adminUrl = this.backend.generatePrivateUrl(this.adminIdent, this.key);

        this.backend.checkNote(this.ident).subscribe(exists => {
            this.state = exists ? "ready" : "missing";
        });
    }


}
