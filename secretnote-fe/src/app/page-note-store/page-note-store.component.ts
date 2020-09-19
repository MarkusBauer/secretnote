import {Component, ElementRef, OnInit, ViewChild} from '@angular/core';
import {CryptoService} from "../crypto.service";
import {BackendService} from "../backend.service";
import {Router} from "@angular/router";
import {UiService} from "../ui.service";

@Component({
    selector: 'app-page-note-store',
    templateUrl: './page-note-store.component.html',
    styleUrls: ['./page-note-store.component.less']
})
export class PageNoteStoreComponent implements OnInit {

    text: string = "";
    @ViewChild('textInput') textInput: ElementRef;

    constructor(private crypto: CryptoService, private backend: BackendService, private ui: UiService, private router: Router) {
    }

    ngOnInit(): void {
    }

    store() {
        let text = this.text.trim();
        if (!text) {
            this.ui.warning($localize`:@@warn_noteempty:Note is empty, please enter a note!`);
            this.textInput.nativeElement.focus();
            return;
        }
        let key = this.crypto.generateKey();
        let encryptedNote = this.crypto.encryptNote({text: text}, key);
        this.backend.storeNote(encryptedNote).subscribe(response => {
            this.ui.success($localize`:@@success_note:You can save the links now`, {header: $localize`:@@success_note_header:Note has been created!`});
            this.router.navigate(['/note/admin', response.admin_ident], {fragment: key});
        }, err => {
            console.error(err);
            this.ui.error('Server responded: '+err, {header: 'Connection error'});
        });
    }

}
