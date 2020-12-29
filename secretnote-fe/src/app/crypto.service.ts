import {Injectable} from '@angular/core';
import * as sjcl from '../../libs/sjcl';


export interface NoteContent {
    text: string;
}


export interface ChatMessage {
    sender: string;
    ts: number;
    text: string;
}

export interface Keypair {
    secret: string;
    public: string;
}


@Injectable({
    providedIn: 'root'
})
export class CryptoService {

    constructor() {
    }

    private static encrypt(p: any, keystring: string): any {
        let key = sjcl.codec.base64url.toBits(keystring);
        let cipher = new sjcl.cipher.aes(key);
        let iv = sjcl.random.randomWords(4, 0);
        let c = sjcl.mode.gcm.encrypt(cipher, p, iv);
        return sjcl.bitArray.concat(iv, c);
    }

    private static decrypt(c_and_iv: any, keystring: string): any {
        let key = sjcl.codec.base64url.toBits(keystring);
        console.log("size_c_and_iv =", sjcl.bitArray.bitLength(c_and_iv)/8);
        console.log("key =", sjcl.bitArray.bitLength(key), sjcl.codec.hex.fromBits(key));
        let cipher = new sjcl.cipher.aes(key);
        let iv = sjcl.bitArray.bitSlice(c_and_iv, 0, 128);
        let c = sjcl.bitArray.bitSlice(c_and_iv, 128, sjcl.bitArray.bitLength(c_and_iv));
        console.log("iv =", sjcl.bitArray.bitLength(iv), sjcl.codec.hex.fromBits(iv));
        return sjcl.mode.gcm.decrypt(cipher, c, iv);
    }

    encryptNote(note: NoteContent, keystring: string): string {
        let p = sjcl.codec.utf8String.toBits(JSON.stringify(note));
        return sjcl.codec.base64.fromBits(CryptoService.encrypt(p, keystring));
    }

    decryptNote(encryptedNote: string, keystring: string): NoteContent {
        let c_and_iv = sjcl.codec.base64.toBits(encryptedNote);
        let p = CryptoService.decrypt(c_and_iv, keystring);
        return JSON.parse(sjcl.codec.utf8String.fromBits(p));
    }

    generateKey(): string {
        return sjcl.codec.base64url.fromBits(sjcl.random.randomWords(4));
    }

    isValidKey(key: string): boolean {
        try {
            let bits = sjcl.codec.base64url.toBits(key);
            let bitsize = sjcl.bitArray.bitLength(bits);
            return bitsize == 128 || bitsize == 192 || bitsize == 256;
        } catch (e) {
            console.error(e);
            return false;
        }
    }

    adminIdentToIdent(adminIdent: string): string {
        return sjcl.codec.base64url.fromBits(sjcl.hash.sha256.hash(sjcl.codec.utf8String.toBits(adminIdent))).substring(0, 28);
    }

    generateChannel(): string {
        //24bytes base64 = //6*3 bytes = 4.5 words
        return sjcl.codec.base64url.fromBits(sjcl.random.randomWords(5)).substring(0, 24);
    }

    private static parseEccSecretKey(sec: string): any {
        return new sjcl.ecc.ecdsa.secretKey(sjcl.ecc.curves.c256, sjcl.ecc.curves.c256.field.fromBits(sjcl.codec.base64url.toBits(sec)));
    }

    generatePublicPrivateKeys(): Keypair {
        // Curve: NIST P-256
        let keys = sjcl.ecc.ecdsa.generateKeys(sjcl.ecc.curves.c256);
        let secretKey = sjcl.codec.base64url.fromBits(keys.sec.get());
        let xy = keys.pub.get();
        let publicKey = sjcl.codec.base64url.fromBits(sjcl.bitArray.concat(xy.x, xy.y));
        return {secret: secretKey, public: publicKey}

        /*
        let sec2 = this.parseEccSecretKey(secretKey);
        console.log(keys.sec, sec2);

        let pub2 = new sjcl.ecc.ecdsa.publicKey(sjcl.ecc.curves.c256, sjcl.codec.base64url.toBits(publicKey));
        console.log(keys.pub, pub2);

        let sig = keys.sec.sign(sjcl.hash.sha256.hash("Hello World!"));
        console.log(keys.pub.verify(sjcl.hash.sha256.hash("Hello World!"), sig));
        console.log(pub2.verify(sjcl.hash.sha256.hash("Hello World!"), sig));
        let sig2 = sec2.sign(sjcl.hash.sha256.hash("Hello World!"));
        console.log(keys.pub.verify(sjcl.hash.sha256.hash("Hello World!"), sig2));
        console.log(pub2.verify(sjcl.hash.sha256.hash("Hello World!"), sig2));
         */
    }

    getPublicFromSecretKey(sec: string): string {
        let secret = CryptoService.parseEccSecretKey(sec);
        let point = sjcl.ecc.curves.c256.G.mult(sjcl.bn.fromBits(secret.get()));
        let pubkey = new sjcl.ecc.ecdsa.publicKey(sjcl.ecc.curves.c256, point);
        let xy = pubkey.get();
        return sjcl.codec.base64url.fromBits(sjcl.bitArray.concat(xy.x, xy.y));
    }

    encryptChatMessage(msg: ChatMessage, key: string, sec: string): ArrayBuffer {
        let messageBytes = sjcl.codec.utf8String.toBits(JSON.stringify(msg));
        let sig = CryptoService.parseEccSecretKey(sec).sign(sjcl.hash.sha256.hash(messageBytes));
        let signedMessage = sjcl.bitArray.concat(sig, messageBytes);
        let encryptedMessage = CryptoService.encrypt(signedMessage, key);
        let result = new Uint8Array(sjcl.codec.bytes.fromBits(encryptedMessage)).buffer;
        return result;
    }

    decryptChatMessage(bin: ArrayBuffer | string, key: string): ChatMessage {
        let bits;
        if (typeof bin === "string") {
            bits = sjcl.codec.base64.toBits(bin);
        } else {
            bits = sjcl.codec.bytes.toBits(new Uint8Array(bin));
        }
        let signedMessage = CryptoService.decrypt(bits, key);
        let signature = sjcl.bitArray.bitSlice(signedMessage, 0, 512);
        let messageBytes = sjcl.bitArray.bitSlice(signedMessage, 512, sjcl.bitArray.bitLength(signedMessage));
        let chatMessage: ChatMessage = JSON.parse(sjcl.codec.utf8String.fromBits(messageBytes));
        let pubkey = new sjcl.ecc.ecdsa.publicKey(sjcl.ecc.curves.c256, sjcl.codec.base64url.toBits(chatMessage.sender));
        let verifyResult = pubkey.verify(sjcl.hash.sha256.hash(messageBytes), signature);
        if (!verifyResult) {
            throw "Signature broken!";
        }
        return chatMessage;
    }

    test() {
        let key = this.generateKey();
        let pair = this.generatePublicPrivateKeys();
        let sender = this.getPublicFromSecretKey(pair.secret);
        let msg: ChatMessage = {sender: sender, ts: Date.now(), text: "Hello world!"};
        let bin = this.encryptChatMessage(msg, key, pair.secret);

        let msg2 = this.decryptChatMessage(bin, key);
        console.log(msg, "==", msg2, "?");
    }
}
