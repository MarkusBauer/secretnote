import {Component, ElementRef, OnInit, ViewChild} from '@angular/core';
import {CryptoService} from "../crypto.service";
import {BackendService} from "../backend.service";
import {Router} from "@angular/router";

@Component({
    selector: 'app-page-note-store',
    templateUrl: './page-note-store.component.html',
    styleUrls: ['./page-note-store.component.less']
})
export class PageNoteStoreComponent implements OnInit {

    text: string = "";
    @ViewChild('textInput') textInput: ElementRef;

    constructor(private crypto: CryptoService, private backend: BackendService, private router: Router) {
    }

    ngOnInit(): void {
    }

    store() {
        let text = this.text.trim();
        if (!text) {
            this.textInput.nativeElement.focus();
            return;
        }
        let key = this.crypto.generateKey();
        let encryptedNote = this.crypto.encryptNote({text: text}, key);
        this.backend.storeNote(encryptedNote).subscribe(response => {
            console.log('ident=', response, '  key=', key);
            this.router.navigate(['/note/admin', response.admin_ident], {fragment: key});
        });
    }

}
