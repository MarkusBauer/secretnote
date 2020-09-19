import {Component, OnInit} from '@angular/core';
import {ActivatedRoute} from "@angular/router";
import {BackendService} from "../backend.service";
import {CryptoService} from "../crypto.service";
import {UiService} from "../ui.service";

@Component({
    selector: 'app-page-note-admin',
    templateUrl: './page-note-admin.component.html',
    styleUrls: ['./page-note-admin.component.less']
})
export class PageNoteAdminComponent implements OnInit {

    ident: string = undefined;
    adminIdent: string = undefined;
    key: string;
    state: string = "loading";
    url: string;
    adminUrl: string;

    constructor(private route: ActivatedRoute, private backend: BackendService, private crypto: CryptoService, private ui: UiService) {
    }

    ngOnInit(): void {
        this.route.paramMap.subscribe(map => {
            this.adminIdent = map.get("ident");
            this.ident = this.crypto.adminIdentToIdent(this.adminIdent);
            this.updateIdentKey();
        });
        this.route.fragment.subscribe(f => {
            this.key = f;
            this.updateIdentKey();
        });
    }


    updateIdentKey() {
        if (this.ident === undefined || this.key === undefined) return;
        if (this.ident.length != 28) {
            this.ui.error($localize`:@@error_link_invalid:This link is invalid`);
            this.state = "error";
            return;
        }
        if (!this.crypto.isValidKey(this.key)) {
            this.ui.error($localize`:@@error_link_invalid_key:This link is invalid (encryption key corrupted)`);
            this.state = "error";
            return;
        }

        this.url = this.backend.generatePublicUrl(this.ident, this.key);
        this.adminUrl = this.backend.generatePrivateUrl(this.adminIdent, this.key);

        this.backend.checkNote(this.ident).subscribe(exists => {
            this.state = exists ? "ready" : "missing";
        }, this.ui.httpErrorHandler);
    }


}
