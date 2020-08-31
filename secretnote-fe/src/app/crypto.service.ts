import {Injectable} from '@angular/core';
import * as sjcl from 'sjcl';


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

    encryptNote(note: NoteContent, keystring: string): string {
        let key = sjcl.codec.base64url.toBits(keystring);
        console.log('key =', key);
        let cipher = new sjcl.cipher.aes(key);
        let p = sjcl.codec.utf8String.toBits(JSON.stringify(note));
        let iv = sjcl.random.randomWords(4,0);
        let c = sjcl.mode.gcm.encrypt(cipher, p, iv);
        return sjcl.codec.base64.fromBits(sjcl.bitArray.concat(iv, c));
    }

    decryptNote(encryptedNote: string, keystring: string): NoteContent {
        let key = sjcl.codec.base64url.toBits(keystring);
        console.log('key =', key);
        let cipher = new sjcl.cipher.aes(key);
        let c_and_iv = sjcl.codec.base64.toBits(encryptedNote);
        let iv = sjcl.bitArray.bitSlice(c_and_iv, 0, 128);
        let c = sjcl.bitArray.bitSlice(c_and_iv, 128, sjcl.bitArray.bitLength(c_and_iv));
        let p = sjcl.mode.gcm.decrypt(cipher, c, iv);
        return JSON.parse(sjcl.codec.utf8String.fromBits(p));
    }

    generateKey(): string {
        return sjcl.codec.base64url.fromBits(sjcl.random.randomWords(4));
    }

    encryptChatMessage(msg: ChatMessage, key: string): ArrayBuffer {
        return new TextEncoder().encode(JSON.stringify(msg));
    }

    decryptChatMessage(bin: ArrayBuffer, key: string): ChatMessage {
        return JSON.parse(new TextDecoder().decode(bin));
    }

    test() {
        let key = this.generateKey();
        let p1 = {text: "abc"};
        let c = this.encryptNote(p1, key);
        let p2 = this.decryptNote(c, key);
        console.log(p1, p2, key, c);
    }
}
