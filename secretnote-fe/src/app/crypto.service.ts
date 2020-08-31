import {Injectable} from '@angular/core';


export interface NoteContent {
    text: string;
}


export interface ChatMessage {
    sender: string;
    ts: number;
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

    encryptChatMessage(msg: ChatMessage, key: string): ArrayBuffer {
        return new TextEncoder().encode(JSON.stringify(msg));
    }

    decryptChatMessage(bin: ArrayBuffer, key: string): ChatMessage {
        return JSON.parse(new TextDecoder().decode(bin));
    }
}
