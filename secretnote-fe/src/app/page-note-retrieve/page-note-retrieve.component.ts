import {Component, OnInit} from '@angular/core';
import {ActivatedRoute} from "@angular/router";
import {BackendService} from "../backend.service";
import {CryptoService, NoteContent} from "../crypto.service";

@Component({
    selector: 'app-page-note-retrieve',
    templateUrl: './page-note-retrieve.component.html',
    styleUrls: ['./page-note-retrieve.component.less']
})
export class PageNoteRetrieveComponent implements OnInit {

    ident: string;
    key: string;
    state: string = "loading";
    note: NoteContent;

    constructor(private route: ActivatedRoute, private backend: BackendService, private crypto: CryptoService) {
    }

    ngOnInit() {
        this.route.paramMap.subscribe(map => {
            this.ident = map.get("ident");
            this.updateIdentKey();
        });
        this.route.fragment.subscribe(f => {
            this.key = f;
            this.updateIdentKey();
        })
    }

    updateIdentKey() {
        if (!this.ident || this.ident.length != 24) {
            // TODO report error
            this.state = "error";
        }
        // TODO check key

        this.state = "loading";
        this.backend.checkNote(this.ident).subscribe(exists => {
            this.state = exists ? "ready" : "missing";
        });
    }

    retrieveNote() {
        this.state = "loading";
        this.backend.retrieveNote(this.ident).subscribe(encryptedNote => {
            this.note = this.crypto.decryptNote(encryptedNote, this.key);
            this.state = "decrypted";
        });
    }

}
