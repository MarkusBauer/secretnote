import {Injectable} from '@angular/core';
import {HttpClient} from "@angular/common/http";
import {Observable} from "rxjs";
import {map} from "rxjs/operators";

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

    constructor(private http: HttpClient) {
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

}
