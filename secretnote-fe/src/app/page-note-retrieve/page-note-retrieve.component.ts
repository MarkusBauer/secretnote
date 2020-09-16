import {Component, OnInit} from '@angular/core';
import {ActivatedRoute} from "@angular/router";
import {BackendService} from "../backend.service";
import {CryptoService, NoteContent} from "../crypto.service";
import {UiService} from "../ui.service";

@Component({
    selector: 'app-page-note-retrieve',
    templateUrl: './page-note-retrieve.component.html',
    styleUrls: ['./page-note-retrieve.component.less']
})
export class PageNoteRetrieveComponent implements OnInit {

    ident: string = undefined;
    key: string = undefined;
    state: string = "loading";
    note: NoteContent;

    constructor(private route: ActivatedRoute, private backend: BackendService, private crypto: CryptoService, private ui: UiService) {
    }

    ngOnInit() {
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
        if (this.ident === undefined || this.key === undefined) {
            return;
        }
        if (this.ident.length != 28) {
            this.ui.error("This link is invalid");
            this.state = "error";
            return;
        }
        if (!this.crypto.isValidKey(this.key)) {
            this.ui.error("This link is invalid (encryption key corrupted)");
            this.state = "error";
            return;
        }

        this.state = "loading";
        this.backend.checkNote(this.ident).subscribe(exists => {
            this.state = exists ? "ready" : "missing";
        }, this.ui.httpErrorHandler);
    }

    retrieveNote() {
        this.state = "loading";
        this.backend.retrieveNote(this.ident).subscribe(encryptedNote => {
            try {
                this.note = this.crypto.decryptNote(encryptedNote, this.key);
                this.state = "decrypted";
            } catch (e) {
                console.error(e);
                this.ui.error(e, {header: "Decryption failed!"});
                this.state = "error";
            }
        }, this.ui.httpErrorHandler);
    }

}
