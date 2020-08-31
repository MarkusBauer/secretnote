import {Injectable} from '@angular/core';
import {HttpClient} from "@angular/common/http";
import {Observable} from "rxjs";
import {map} from "rxjs/operators";
import {PlatformLocation} from "@angular/common";
import {Router} from "@angular/router";
import {webSocket, WebSocketSubject} from "rxjs/webSocket";

interface NoteStoreResponse {
    ident: string;
}

interface NoteCheckResponse {
    ident: string;
    exists: boolean;
}

interface NoteRetrieveResponse {
    ident: string;
    data: string;
}

@Injectable({
    providedIn: 'root'
})
export class BackendService {

    base = '/';
    wsbase = 'ws://localhost:8080/'

    constructor(private http: HttpClient, private platformLocation: PlatformLocation, private router: Router) {
    }

    generatePublicUrl(ident: string, key: string): string {
        return window.location.origin + this.router.createUrlTree(['/note', ident], {fragment: key});
    }

    generatePrivateUrl(ident: string, key: string): string {
        return window.location.origin + this.router.createUrlTree(['/note/admin', ident], {fragment: key});
    }

    storeNote(text: string): Observable<string> {
        return this.http.post<NoteStoreResponse>(this.base + 'api/note/store', {data: text})
            .pipe(map(response => response.ident));
    }

    checkNote(ident: string): Observable<boolean> {
        return this.http.get<NoteCheckResponse>(this.base + 'api/note/check/' + ident)
            .pipe(map(response => response.exists));
    }

    retrieveNote(ident: string): Observable<string> {
        return this.http.post<NoteRetrieveResponse>(this.base + 'api/note/retrieve', {ident: ident})
            .pipe(map(response => response.data));
    }

    connectToChat(channel: string): WebSocketSubject<ArrayBuffer> {
        // let socket = new WebSocket('wss://echo.websocket.org');
        let ws = webSocket<ArrayBuffer>({
            url: this.wsbase + "api/websocket/" + channel,
            binaryType: 'arraybuffer',
            deserializer: ({data}) => data,
            serializer: data => data,
        });
        return ws;
    }

}
