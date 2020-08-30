import {Injectable} from '@angular/core';

@Injectable({
    providedIn: 'root'
})
export class CryptoService {

    constructor() {
    }

    encryptNote<T>(note: T, key: string): string {
        return btoa(JSON.stringify(note));
    }

    decryptNote<T>(encryptedNote: string, key: string): T {
        return JSON.parse(atob(encryptedNote));
    }

    generateKey(): string {
        return "TODO";
    }
}
