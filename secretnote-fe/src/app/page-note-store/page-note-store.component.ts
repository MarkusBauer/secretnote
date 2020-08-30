import {Component, OnInit} from '@angular/core';
import {CryptoService} from "../crypto.service";
import {BackendService} from "../backend.service";

@Component({
    selector: 'app-page-note-store',
    templateUrl: './page-note-store.component.html',
    styleUrls: ['./page-note-store.component.less']
})
export class PageNoteStoreComponent implements OnInit {

    text: string = "";

    constructor(private crypto: CryptoService, private backend: BackendService) {
    }

    ngOnInit(): void {
    }

    store() {
        let key = this.crypto.generateKey();
        let encryptedNote = this.crypto.encryptNote({text: this.text}, key);
        this.backend.storeNote(encryptedNote).subscribe(ident => {
            console.log('ident=', ident, '  key=', key);
        });
    }

}
