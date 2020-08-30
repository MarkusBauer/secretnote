import {Injectable} from '@angular/core';


export interface NoteContent {
    text: string;
}


@Injectable({
    providedIn: 'root'
})
export class CryptoService {

    constructor() {
    }

    encryptNote(note: NoteContent, key: string): string {
        return btoa(JSON.stringify(note) + "               ");
    }

    decryptNote(encryptedNote: string, key: string): NoteContent {
        return JSON.parse(atob(encryptedNote));
    }

    generateKey(): string {
        return "TODO";
    }
}
